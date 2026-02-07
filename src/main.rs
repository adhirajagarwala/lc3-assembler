use std::env;
use std::fs;

use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::parser::parse_lines;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lc3-assembler <file.asm>");
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("Failed to read file: {}", err);
        std::process::exit(1);
    });

    let lexed = tokenize(&source);
    for err in &lexed.errors {
        eprintln!("{}", err);
    }

    let parsed = parse_lines(&lexed.tokens);
    for err in &parsed.errors {
        eprintln!("{}", err);
    }

    let first = first_pass(&parsed.lines);
    for err in &first.errors {
        eprintln!("{}", err);
    }

    first.symbol_table.print_table();
}
