use std::env;
use std::fs;
use std::io::Write;

use lc3_assembler::encoder::encode;
use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::parser::parse_lines;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("LC-3 Assembler");
        eprintln!("Usage: lc3-assembler <input.asm> [-o output.obj]");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  lc3-assembler program.asm           # Creates program.obj");
        eprintln!("  lc3-assembler program.asm -o out.obj # Creates out.obj");
        std::process::exit(1);
    }

    let input_file = &args[1];
    let source = fs::read_to_string(input_file).unwrap_or_else(|err| {
        eprintln!("Error: Failed to read '{}': {}", input_file, err);
        std::process::exit(1);
    });

    let lexed = tokenize(&source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(&parsed.lines);
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
        eprintln!("{}", err);
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

    // Determine output file name
    let output_file = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        input_file.replace(".asm", ".obj")
    };

    // Write binary output (LC-3 object file format)
    match write_obj_file(&output_file, encoded.orig_address, &encoded.machine_code) {
        Ok(_) => {
            println!("\n\u{2705} Assembly successful!");
            println!("   Input:  {}", input_file);
            println!("   Output: {}", output_file);
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

fn write_obj_file(path: &str, orig: u16, code: &[u16]) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;

    // Write origin address (big-endian)
    file.write_all(&orig.to_be_bytes())?;

    // Write machine code (big-endian)
    for &word in code {
        file.write_all(&word.to_be_bytes())?;
    }

    Ok(())
}
