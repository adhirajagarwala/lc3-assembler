use std::env;
use std::fs;
use std::path::Path;

use lc3_assembler::encoder::encode;
use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::parser::parse_lines;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().skip(1).any(|a| a == "--version" || a == "-V") {
        println!("lc3-assembler {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.len() < 2 || args.iter().skip(1).any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let input_file = &args[1];
    let source = fs::read_to_string(input_file).unwrap_or_else(|err| {
        eprintln!("Error: Failed to read '{input_file}': {err}");
        std::process::exit(1);
    });

    let lexed = tokenize(&source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(parsed.lines); // moves lines into first_pass; parsed.errors still accessible
    let encoded = encode(&first);

    // Collect and print all errors
    let all_errors: Vec<_> = lexed
        .errors
        .iter()
        .chain(parsed.errors.iter())
        .chain(first.errors.iter())
        .chain(encoded.errors.iter())
        .collect();

    for err in &all_errors {
        eprintln!("{err}");
    }

    if !all_errors.is_empty() {
        eprintln!(
            "\n\u{274c} Assembly failed with {} error(s)",
            all_errors.len()
        );
        std::process::exit(1);
    }

    // Print symbol table
    println!("\n\u{1f4cb} Symbol Table:");
    first.symbol_table.print_table();

    // Determine output file name.
    // Use Path::with_extension so that only the file extension is replaced.
    // The old `replace(".asm", ".obj")` would corrupt paths like `/my.asm/prog.asm`.
    let output_file = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        Path::new(input_file)
            .with_extension("obj")
            .to_string_lossy()
            .into_owned()
    };

    // Write binary output (LC-3 object file format)
    match write_obj_file(&output_file, encoded.orig_address, &encoded.machine_code) {
        Ok(_) => {
            println!("\n\u{2705} Assembly successful!");
            println!("   Input:  {input_file}");
            println!("   Output: {output_file}");
            println!("   Origin: 0x{:04X}", encoded.orig_address);
            println!(
                "   Size:   {} words ({} bytes)",
                encoded.machine_code.len(),
                encoded.machine_code.len() * 2
            );
        }
        Err(err) => {
            eprintln!(
                "\n\u{274c} Error: Failed to write '{}': {}",
                output_file, err
            );
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("lc3-assembler {}", env!("CARGO_PKG_VERSION"));
    println!("A production-ready assembler for the LC-3 educational computer architecture");
    println!();
    println!("USAGE:");
    println!("  lc3-assembler <input.asm> [-o output.obj]");
    println!();
    println!("OPTIONS:");
    println!(
        "  -o <output.obj>   Write machine code to this file (default: replaces .asm with .obj)"
    );
    println!("  -h, --help        Print this help message");
    println!("  -V, --version     Print version information");
    println!();
    println!("EXAMPLES:");
    println!("  lc3-assembler program.asm             # Creates program.obj");
    println!("  lc3-assembler program.asm -o out.obj  # Creates out.obj");
}

fn write_obj_file(path: &str, orig: u16, code: &[u16]) -> std::io::Result<()> {
    // Pre-allocate the full output buffer and issue a single write_all.
    // The old per-word approach issued one syscall per 2-byte word, meaning a
    // 1000-word program made 1001 write syscalls. Now it's always exactly 1.
    let total_words = 1 + code.len(); // origin word + machine code words
    let mut buf = Vec::with_capacity(total_words * 2);

    // Write origin address (big-endian) into buffer
    buf.extend_from_slice(&orig.to_be_bytes());

    // Write all machine code words (big-endian) into buffer
    for &word in code {
        buf.extend_from_slice(&word.to_be_bytes());
    }

    // Single write_all — one syscall for the entire file
    fs::write(path, &buf)
}
