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

/// Output format for the assembled machine code.
#[derive(Clone, Copy, PartialEq)]
enum EmitFormat {
    /// LC-3 binary object file: big-endian origin word + code words (default).
    Obj,
    /// Intel HEX ASCII text — portable across tools/simulators.
    Hex,
}

struct Args {
    /// Path to input .asm file, or "-" for stdin.
    input: String,
    /// Path for the output file.  None = auto-derive from input.
    output: Option<String>,
    /// Path for an optional listing (.lst) file.
    listing: Option<String>,
    /// Path for an optional symbol-table (.sym) file.
    symbols: Option<String>,
    /// Output format (binary obj or Intel HEX).
    emit: EmitFormat,
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
        let mut symbols_path: Option<String> = None;
        let mut emit = EmitFormat::Obj;
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
                "-s" | "--symbols" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("error: -s requires a filename argument");
                        std::process::exit(1);
                    }
                    symbols_path = Some(args[i].to_string());
                }
                "--emit" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("error: --emit requires a format argument (obj|hex)");
                        std::process::exit(1);
                    }
                    emit = match args[i] {
                        "obj" => EmitFormat::Obj,
                        "hex" => EmitFormat::Hex,
                        other => {
                            eprintln!("error: unknown emit format '{other}' (expected: obj, hex)");
                            std::process::exit(1);
                        }
                    };
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
            symbols: symbols_path,
            emit,
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
        if all_warnings.is_empty() {
            eprintln!(
                "ok — '{}' assembles without errors or warnings",
                display_name
            );
        } else {
            eprintln!(
                "ok — '{}' assembles without errors ({} warning{})",
                display_name,
                all_warnings.len(),
                if all_warnings.len() == 1 { "" } else { "s" }
            );
        }
        std::process::exit(0);
    }

    // ── Determine output path ────────────────────────────────────────────────

    let default_ext = match args.emit {
        EmitFormat::Obj => "obj",
        EmitFormat::Hex => "hex",
    };

    let output_path: String = match &args.output {
        Some(p) => p.clone(),
        None => {
            if args.input == "-" {
                "".to_string() // handled below (stdout)
            } else {
                Path::new(&args.input)
                    .with_extension(default_ext)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    };

    // ── Write output file ─────────────────────────────────────────────────────

    if args.input == "-" && args.output.is_none() {
        // stdin → stdout
        match args.emit {
            EmitFormat::Obj => {
                write_obj_stdout(encoded.orig_address, &encoded.machine_code).unwrap_or_else(
                    |err| {
                        eprintln!("error: failed to write to stdout: {err}");
                        std::process::exit(1);
                    },
                );
            }
            EmitFormat::Hex => {
                use std::io::Write as _;
                let hex = intel_hex(encoded.orig_address, &encoded.machine_code);
                std::io::stdout()
                    .write_all(hex.as_bytes())
                    .unwrap_or_else(|err| {
                        eprintln!("error: failed to write to stdout: {err}");
                        std::process::exit(1);
                    });
            }
        }
    } else {
        match args.emit {
            EmitFormat::Obj => {
                write_obj_file(&output_path, encoded.orig_address, &encoded.machine_code)
                    .unwrap_or_else(|err| {
                        eprintln!("error: failed to write '{}': {err}", output_path);
                        std::process::exit(1);
                    });
            }
            EmitFormat::Hex => {
                let hex = intel_hex(encoded.orig_address, &encoded.machine_code);
                fs::write(&output_path, hex).unwrap_or_else(|err| {
                    eprintln!("error: failed to write '{}': {err}", output_path);
                    std::process::exit(1);
                });
            }
        }
    }

    // ── Write listing file ────────────────────────────────────────────────────

    if let Some(ref lst_path) = args.listing {
        let lst = listing::generate(&source, &first, &encoded, &display_name);
        fs::write(lst_path, &lst).unwrap_or_else(|err| {
            eprintln!("error: failed to write listing '{}': {err}", lst_path);
            std::process::exit(1);
        });
    }

    // ── Write symbol table file ───────────────────────────────────────────────

    if let Some(ref sym_path) = args.symbols {
        let sym = listing::generate_sym_file(&first.symbol_table, &display_name);
        fs::write(sym_path, &sym).unwrap_or_else(|err| {
            eprintln!("error: failed to write symbols '{}': {err}", sym_path);
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
        eprintln!("listing  → '{lst_path}'");
    }
    if let Some(ref sym_path) = args.symbols {
        eprintln!(
            "symbols  → '{sym_path}'  [{} label{}]",
            first.symbol_table.len(),
            if first.symbol_table.len() == 1 {
                ""
            } else {
                "s"
            }
        );
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
    println!("  -             Read source from stdin; write output to stdout");
    println!();
    println!("OPTIONS:");
    println!("  -o, --output <file>    Write machine code to <file> (default: <input>.obj)");
    println!("  -l, --listing <file>   Write a human-readable listing (includes symbol table)");
    println!("  -s, --symbols <file>   Write the symbol table to <file>");
    println!("      --emit <format>    Output format: obj (default) or hex (Intel HEX)");
    println!("      --check            Validate only; do not write any output files");
    println!("      --no-color         Disable ANSI colour in diagnostics");
    println!("  -h, --help             Print this help message");
    println!("  -V, --version          Print version information");
    println!();
    println!("EXAMPLES:");
    println!("  lc3-assembler program.asm                      # Creates program.obj");
    println!("  lc3-assembler program.asm -o out.obj           # Explicit output path");
    println!("  lc3-assembler program.asm -l prog.lst          # Listing with symbol table");
    println!("  lc3-assembler program.asm -s prog.sym          # Symbol table only");
    println!("  lc3-assembler program.asm --emit hex           # Intel HEX output");
    println!("  lc3-assembler --check program.asm              # Validate without writing");
    println!("  lc3-assembler - < program.asm > program.obj    # stdin → stdout");
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

/// Generate an Intel HEX representation of the assembled program.
///
/// Each data record holds up to 16 bytes (8 words).  The origin address is the
/// byte address of the first word (word_addr * 2).  All records use the standard
/// `:LLAAAATT…CC` format where CC is the two's-complement checksum byte.
fn intel_hex(orig: u16, code: &[u16]) -> String {
    // Byte address of the first word (LC-3 word addresses → byte addresses × 2).
    let byte_origin = (orig as u32) * 2;

    // Flatten 16-bit words into big-endian bytes.
    let bytes: Vec<u8> = code
        .iter()
        .flat_map(|&w| [((w >> 8) & 0xFF) as u8, (w & 0xFF) as u8])
        .collect();

    let mut out = String::new();
    const CHUNK: usize = 16; // bytes per data record

    for (chunk_idx, chunk) in bytes.chunks(CHUNK).enumerate() {
        let addr = byte_origin + (chunk_idx * CHUNK) as u32;
        // Intel HEX only supports 16-bit addresses in the basic format.
        // Extended records would be needed above 0xFFFF bytes (0x7FFF words).
        let addr16 = addr as u16;
        let byte_count = chunk.len() as u8;

        // Checksum = two's complement of (byte_count + addr_hi + addr_lo + 0x00 + data…)
        let mut sum: u8 = byte_count
            .wrapping_add((addr16 >> 8) as u8)
            .wrapping_add((addr16 & 0xFF) as u8);
        for &b in chunk {
            sum = sum.wrapping_add(b);
        }
        let checksum = (!sum).wrapping_add(1);

        out.push_str(&format!(":{:02X}{:04X}00", byte_count, addr16));
        for &b in chunk {
            out.push_str(&format!("{:02X}", b));
        }
        out.push_str(&format!("{:02X}\n", checksum));
    }

    // End-of-file record
    out.push_str(":00000001FF\n");
    out
}
