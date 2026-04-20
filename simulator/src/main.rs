mod disasm;
mod machine;
mod memory;
mod trap;
mod tui;

use std::{collections::HashMap, fs, path::Path};

use lc3_assembler::{
    encoder::encode, first_pass::first_pass, lexer::tokenize, parser::parse_lines,
};
use machine::{Machine, StepResult};
use tui::app::App;

// ── CLI ───────────────────────────────────────────────────────────────────────

struct Args {
    /// .obj or .asm input file.
    input: String,
    /// Load a .sym file for label display.
    symbols: Option<String>,
    /// Run headlessly (no TUI) and print output to stdout.
    run: bool,
}

impl Args {
    fn parse() -> Self {
        let raw: Vec<String> = std::env::args().collect();
        let args: Vec<&str> = raw.iter().map(|s| s.as_str()).collect();

        if args.len() < 2 || args.iter().skip(1).any(|a| *a == "--help" || *a == "-h") {
            print_help();
            std::process::exit(if args.len() < 2 { 1 } else { 0 });
        }
        if args.iter().skip(1).any(|a| *a == "--version" || *a == "-V") {
            println!("lc3-sim {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }

        let mut input = None;
        let mut symbols = None;
        let mut run = false;
        let mut i = 1usize;

        while i < args.len() {
            match args[i] {
                "-s" | "--symbols" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("error: -s requires a filename");
                        std::process::exit(1);
                    }
                    symbols = Some(args[i].to_string());
                }
                "--run" => run = true,
                other => {
                    if input.is_some() {
                        eprintln!("error: unexpected argument '{other}'");
                        std::process::exit(1);
                    }
                    input = Some(other.to_string());
                }
            }
            i += 1;
        }

        Args {
            input: input.unwrap_or_else(|| {
                eprintln!("error: no input file");
                print_help();
                std::process::exit(1);
            }),
            symbols,
            run,
        }
    }
}

fn print_help() {
    println!("lc3-sim {}", env!("CARGO_PKG_VERSION"));
    println!("TUI debugger and simulator for the LC-3 educational computer");
    println!();
    println!("USAGE:");
    println!("  lc3-sim [OPTIONS] <input.obj|input.asm>");
    println!();
    println!("OPTIONS:");
    println!("  -s, --symbols <file>   Load .sym file for label display in TUI");
    println!("      --run              Run headlessly; print output to stdout");
    println!("  -h, --help             Print this help message");
    println!("  -V, --version          Print version information");
    println!();
    println!("TUI KEYS:");
    println!("  s        Single step");
    println!("  c        Continue (run to halt/breakpoint)");
    println!("  p        Pause");
    println!("  r        Reset (reload program)");
    println!("  b x3000  Toggle breakpoint at address");
    println!("  g x3000  Scroll memory panel to address");
    println!("  ↑ ↓      Scroll memory panel");
    println!("  q        Quit");
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    // ── Load program ──────────────────────────────────────────────────────────

    let (obj_bytes, sym_table) = load_program(&args);

    // ── Headless run ──────────────────────────────────────────────────────────

    if args.run {
        run_headless(obj_bytes);
        return;
    }

    // ── TUI mode ──────────────────────────────────────────────────────────────

    let app = App::new(obj_bytes, sym_table).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    if let Err(e) = tui::run(app) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Load the input (assembling from .asm if needed) and parse any .sym file.
/// Returns (raw obj bytes, address→label map).
fn load_program(args: &Args) -> (Vec<u8>, HashMap<u16, String>) {
    let path = &args.input;
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let (obj_bytes, asm_syms) = if ext.eq_ignore_ascii_case("asm") {
        assemble_from_source(path)
    } else {
        let bytes = fs::read(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {e}", path);
            std::process::exit(1);
        });
        (bytes, HashMap::new())
    };

    // Prefer explicit .sym file; fall back to symbols from assembler pass.
    let sym_table = if let Some(ref sym_path) = args.symbols {
        load_sym_file(sym_path)
    } else {
        asm_syms
    };

    (obj_bytes, sym_table)
}

/// Assemble an .asm source file in-memory.  Exits on errors.
fn assemble_from_source(path: &str) -> (Vec<u8>, HashMap<u16, String>) {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: cannot read '{}': {e}", path);
        std::process::exit(1);
    });

    let lexed = tokenize(&source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(parsed.lines);
    let encoded = encode(&first);

    let all_errors: Vec<_> = lexed
        .errors
        .iter()
        .chain(parsed.errors.iter())
        .chain(first.errors.iter())
        .chain(encoded.errors.iter())
        .collect();

    if !all_errors.is_empty() {
        for e in &all_errors {
            eprintln!("error:{}: {}", e.span.line, e.message);
        }
        std::process::exit(1);
    }

    // Serialize to the same .obj binary format the assembler writes.
    let mut bytes = Vec::with_capacity((1 + encoded.machine_code.len()) * 2);
    bytes.extend_from_slice(&encoded.orig_address.to_be_bytes());
    for &w in &encoded.machine_code {
        bytes.extend_from_slice(&w.to_be_bytes());
    }

    // Build address→label map from the assembler's symbol table.
    let syms: HashMap<u16, String> = first
        .symbol_table
        .iter()
        .map(|(label, addr)| (addr, label.to_string()))
        .collect();

    (bytes, syms)
}

/// Parse a .sym file in the format `LABEL=xADDR`.
fn load_sym_file(path: &str) -> HashMap<u16, String> {
    let text = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("warning: cannot read sym file '{}': {e}", path);
        String::new()
    });
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(';') || line.is_empty() {
            continue;
        }
        if let Some((label, addr_str)) = line.split_once('=') {
            let addr_str = addr_str.trim().trim_start_matches('x');
            if let Ok(addr) = u16::from_str_radix(addr_str, 16) {
                map.insert(addr, label.trim().to_string());
            }
        }
    }
    map
}

/// Headless run: execute to HALT and print output to stdout.
fn run_headless(obj_bytes: Vec<u8>) {
    let mut machine = Machine::new();
    machine.headless = true;

    machine.load_obj(&obj_bytes).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    const MAX_STEPS: u64 = 10_000_000;
    loop {
        if machine.step_count >= MAX_STEPS {
            eprintln!("\nwarning: execution limit ({MAX_STEPS} steps) reached");
            break;
        }

        // If machine needs keyboard input in headless mode, read one byte from stdin.
        if machine.waiting_for_input {
            use std::io::Read;
            let mut buf = [0u8; 1];
            match std::io::stdin().read(&mut buf) {
                Ok(1) => machine.input_queue.push_back(buf[0]),
                _ => machine.input_queue.push_back(0),
            }
        }

        match machine.step() {
            StepResult::Halted | StepResult::BreakpointHit(_) => break,
            StepResult::IllegalInstruction(ir) => {
                eprintln!(
                    "\nerror: illegal instruction 0x{:04X} at PC=0x{:04X}",
                    ir,
                    machine.regs.pc.wrapping_sub(1)
                );
                std::process::exit(1);
            }
            StepResult::Ok => {}
        }
    }

    // Flush any partial output line.
    if !machine.output_buf.is_empty() {
        println!("{}", machine.output_buf);
    }
}
