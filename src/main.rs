use std::fs;
use std::io::{self, Read};
use std::path::Path;

use lc3_assembler::diagnostic::RichDiagnostics;
use lc3_assembler::encoder::encode;
use lc3_assembler::error::{AsmError, ErrorKind, Span};
use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::listing;
use lc3_assembler::macro_expand;
use lc3_assembler::parser::parse_lines;
use lc3_assembler::preprocessor;

// ── CLI argument parsing ──────────────────────────────────────────────────────

struct Args {
    /// Path to input .asm file, or "-" for stdin.
    input: String,
    /// Path for the output .obj file.  None = auto-derive from input.
    output: Option<String>,
    /// Path for an optional listing (.lst) file.
    listing: Option<String>,
    /// Validate-only; do not write any output files.
    check: bool,
    /// Disable ANSI colour output regardless of TTY detection.
    no_color: bool,
}

impl Args {
    fn parse() -> Self {
        let raw: Vec<String> = std::env::args().collect();
        let args: Vec<&str> = raw.iter().map(|s| s.as_str()).collect();

        // Version / help flags can appear anywhere.
        if args.iter().skip(1).any(|a| *a == "--version" || *a == "-V") {
            println!("lc3-assembler {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        if args.len() < 2 || args.iter().skip(1).any(|a| *a == "--help" || *a == "-h") {
            print_help();
            std::process::exit(if args.len() < 2 { 1 } else { 0 });
        }

        let mut input: Option<String> = None;
        let mut output: Option<String> = None;
        let mut listing_path: Option<String> = None;
        let mut check = false;
        let mut no_color = false;

        let mut i = 1usize;
        while i < args.len() {
            match args[i] {
                "-o" | "--output" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("error: -o requires a filename argument");
                        std::process::exit(1);
                    }
                    output = Some(args[i].to_string());
                }
                "-l" | "--listing" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("error: -l requires a filename argument");
                        std::process::exit(1);
                    }
                    listing_path = Some(args[i].to_string());
                }
                "--check" => {
                    check = true;
                }
                "--no-color" => {
                    no_color = true;
                }
                // Anything else is treated as the positional input file.
                other => {
                    if input.is_some() {
                        eprintln!("error: unexpected argument '{other}'");
                        print_help();
                        std::process::exit(1);
                    }
                    input = Some(other.to_string());
                }
            }
            i += 1;
        }

        let Some(input) = input else {
            eprintln!("error: no input file specified");
            print_help();
            std::process::exit(1);
        };

        Args {
            input,
            output,
            listing: listing_path,
            check,
            no_color,
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    // ── Read source ──────────────────────────────────────────────────────────

    let (source, display_name) = if args.input == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|err| {
            eprintln!("error: failed to read stdin: {err}");
            std::process::exit(1);
        });
        (buf, "<stdin>".to_string())
    } else {
        let src = fs::read_to_string(&args.input).unwrap_or_else(|err| {
            eprintln!("error: failed to read '{}': {err}", args.input);
            std::process::exit(1);
        });
        (src, args.input.clone())
    };

    // ── Stage 0: Preprocessing (.INCLUDE expansion) ──────────────────────────

    let prep = preprocessor::preprocess(&display_name, Some(&source));

    // Convert preprocessor errors to AsmError so they flow through the same
    // diagnostic machinery as everything else.
    let prep_errors: Vec<AsmError> = prep
        .errors
        .iter()
        .map(|e| {
            AsmError::new(
                ErrorKind::IoError,
                e.message.clone(),
                Span {
                    line: e.line.max(1),
                    col: 1,
                },
            )
        })
        .collect();

    // ── Stage 1: Macro expansion ─────────────────────────────────────────────

    let macro_result = macro_expand::expand(&prep.source);
    let macro_errors: Vec<AsmError> = macro_result
        .errors
        .iter()
        .map(|e| {
            AsmError::new(
                ErrorKind::MacroError,
                e.message.clone(),
                Span {
                    line: e.line.max(1),
                    col: 1,
                },
            )
        })
        .collect();

    // Use the macro-expanded source for all downstream stages.
    //
    // If *preprocessing* failed (e.g., an included file could not be found), the
    // expanded source may be truncated or missing whole sections, so we fall back
    // to the original source to avoid a cascade of spurious errors.
    //
    // If only *macro expansion* had errors (wrong argument count, recursive call,
    // etc.), the expanded source is still structurally complete: bad invocations
    // are replaced with blank lines to preserve line numbering, and passing it
    // downstream gives more accurate diagnostics than re-using the original text.
    let expanded_source = if prep.has_errors() {
        source.clone()
    } else {
        macro_result.source.clone()
    };

    // ── Stage 2–5: Lex → Parse → First pass → Encode ─────────────────────────

    let lexed = tokenize(&expanded_source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(parsed.lines);
    let encoded = encode(&first);

    // ── Diagnostics ──────────────────────────────────────────────────────────

    // Use the *original* source for diagnostics (line numbers from prep/macro
    // stages point into the original file; lex/parse/encode point into the
    // expanded source, which only differs when macros/includes were used).
    let diag = RichDiagnostics::new(&source, &display_name).with_color(!args.no_color);

    // Collect all errors (preprocess → macro → lex → parse → first-pass → encode)
    let all_errors: Vec<_> = prep_errors
        .iter()
        .chain(macro_errors.iter())
        .chain(lexed.errors.iter())
        .chain(parsed.errors.iter())
        .chain(first.errors.iter())
        .chain(encoded.errors.iter())
        .collect();

    // Collect all warnings (first-pass → encode)
    let all_warnings: Vec<_> = first
        .warnings
        .iter()
        .chain(encoded.warnings.iter())
        .cloned()
        .collect();

    // Emit warnings first (they don't block assembly)
    diag.emit_all_warnings(&all_warnings);

    // Emit errors
    diag.emit_all_errors(&all_errors);

    if !all_errors.is_empty() {
        std::process::exit(1);
    }

    // ── Check mode: stop here (no file output) ────────────────────────────────

    if args.check {
        // Print a brief success summary and exit cleanly
        eprintln!("ok — '{}' assembles without errors", display_name);
        std::process::exit(0);
    }

    // ── Determine output path ────────────────────────────────────────────────

    let output_path: String = match &args.output {
        Some(p) => p.clone(),
        None => {
            if args.input == "-" {
                // stdin input: write to stdout in binary (pipe-friendly)
                "".to_string() // handled below
            } else {
                Path::new(&args.input)
                    .with_extension("obj")
                    .to_string_lossy()
                    .into_owned()
            }
        }
    };

    // ── Write .obj file ───────────────────────────────────────────────────────

    if args.input == "-" && args.output.is_none() {
        // stdin → stdout (binary)
        write_obj_stdout(encoded.orig_address, &encoded.machine_code).unwrap_or_else(|err| {
            eprintln!("error: failed to write to stdout: {err}");
            std::process::exit(1);
        });
    } else {
        write_obj_file(&output_path, encoded.orig_address, &encoded.machine_code).unwrap_or_else(
            |err| {
                eprintln!("error: failed to write '{}': {err}", output_path);
                std::process::exit(1);
            },
        );
    }

    // ── Write listing file ────────────────────────────────────────────────────

    if let Some(ref lst_path) = args.listing {
        let lst = listing::generate(&source, &first, &encoded, &display_name);
        fs::write(lst_path, &lst).unwrap_or_else(|err| {
            eprintln!("error: failed to write listing '{}': {err}", lst_path);
            std::process::exit(1);
        });
    }

    // ── Success banner ────────────────────────────────────────────────────────

    let warnings_note = if all_warnings.is_empty() {
        String::new()
    } else {
        format!(
            " ({} warning{})",
            all_warnings.len(),
            if all_warnings.len() == 1 { "" } else { "s" }
        )
    };

    if args.input != "-" || args.output.is_some() {
        eprintln!(
            "assembled '{}' → '{}'  [{} word{}, origin x{:04X}]{}",
            display_name,
            if args.input == "-" {
                args.output.as_deref().unwrap_or("-")
            } else {
                &output_path
            },
            encoded.machine_code.len(),
            if encoded.machine_code.len() == 1 {
                ""
            } else {
                "s"
            },
            encoded.orig_address,
            warnings_note,
        );
    }

    if let Some(ref lst_path) = args.listing {
        eprintln!("listing written to '{lst_path}'");
    }
}

// ── Help text ─────────────────────────────────────────────────────────────────

fn print_help() {
    println!("lc3-assembler {}", env!("CARGO_PKG_VERSION"));
    println!("A production-ready assembler for the LC-3 educational computer architecture");
    println!();
    println!("USAGE:");
    println!("  lc3-assembler [OPTIONS] <input.asm|->");
    println!();
    println!("ARGS:");
    println!("  <input.asm>   Path to the LC-3 assembly source file");
    println!("  -             Read source from stdin; write .obj to stdout");
    println!();
    println!("OPTIONS:");
    println!("  -o, --output <file>    Write machine code to <file> (default: <input>.obj)");
    println!("  -l, --listing <file>   Write a human-readable listing to <file>");
    println!("      --check            Validate only; do not write any output files");
    println!("      --no-color         Disable ANSI colour in diagnostics");
    println!("  -h, --help             Print this help message");
    println!("  -V, --version          Print version information");
    println!();
    println!("EXAMPLES:");
    println!("  lc3-assembler program.asm                   # Creates program.obj");
    println!("  lc3-assembler program.asm -o out.obj        # Explicit output path");
    println!("  lc3-assembler program.asm -l program.lst    # Also write listing file");
    println!("  lc3-assembler --check program.asm           # Validate without writing");
    println!("  lc3-assembler - < program.asm > program.obj # stdin → stdout");
}

// ── File I/O helpers ──────────────────────────────────────────────────────────

/// Write an LC-3 object file (big-endian origin word followed by code words).
fn write_obj_file(path: &str, orig: u16, code: &[u16]) -> io::Result<()> {
    let mut buf = Vec::with_capacity((1 + code.len()) * 2);
    buf.extend_from_slice(&orig.to_be_bytes());
    for &word in code {
        buf.extend_from_slice(&word.to_be_bytes());
    }
    fs::write(path, &buf)
}

/// Write an LC-3 object file to stdout (used when input is "-" and no -o given).
fn write_obj_stdout(orig: u16, code: &[u16]) -> io::Result<()> {
    use io::Write as _;
    let mut out = io::stdout();
    out.write_all(&orig.to_be_bytes())?;
    for &word in code {
        out.write_all(&word.to_be_bytes())?;
    }
    out.flush()
}
