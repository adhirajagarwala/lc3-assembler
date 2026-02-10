use std::fs;

use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::parser::parse_lines;

// TODO-HIGH: Add integration tests for encoder once second_pass is implemented
fn run_pipeline(path: &str) -> lc3_assembler::first_pass::FirstPassResult {
    let source = fs::read_to_string(path).expect("Failed to read test program");
    let lexed = tokenize(&source);
    assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
    let parsed = parse_lines(&lexed.tokens);
    assert!(parsed.errors.is_empty(), "Parser errors: {:?}", parsed.errors);
    let result = first_pass(&parsed.lines);
    assert!(result.errors.is_empty(), "First pass errors: {:?}", result.errors);
    result
}

// TODO-LOW: Consider parameterized test macro to reduce duplication across integration tests
#[test]
fn hello_program() {
    let result = run_pipeline("tests/test_programs/hello.asm");
    assert_eq!(result.symbol_table.get("MSG"), Some(0x3003));
}

#[test]
fn countdown_program() {
    let result = run_pipeline("tests/test_programs/countdown.asm");
    assert_eq!(result.symbol_table.get("LOOP"), Some(0x3002));
}

#[test]
fn all_instructions_program() {
    let result = run_pipeline("tests/test_programs/all_instructions.asm");
    assert_eq!(result.symbol_table.get("TARGET"), Some(0x3010));
}

#[test]
fn all_directives_program() {
    let result = run_pipeline("tests/test_programs/all_directives.asm");
    assert_eq!(result.symbol_table.get("DATA"), Some(0x3000));
    assert_eq!(result.symbol_table.get("BUFFER"), Some(0x3001));
    assert_eq!(result.symbol_table.get("MSG"), Some(0x3006));
}

#[test]
fn edge_cases_program() {
    let result = run_pipeline("tests/test_programs/edge_cases.asm");
    assert_eq!(result.symbol_table.get("START"), Some(0x3000));
}

#[test]
fn subroutine_program() {
    let result = run_pipeline("tests/test_programs/subroutine.asm");
    assert_eq!(result.symbol_table.get("SUB"), Some(0x3002));
}

#[test]
fn trap_aliases_program() {
    let result = run_pipeline("tests/test_programs/trap_aliases.asm");
    assert_eq!(result.symbol_table.get("MSG"), Some(0x3006));
}

#[test]
fn large_blkw_program() {
    let result = run_pipeline("tests/test_programs/large_blkw.asm");
    assert_eq!(result.symbol_table.get("AFTER"), Some(0x3014));
}

#[test]
fn multiple_labels_program() {
    let result = run_pipeline("tests/test_programs/multiple_labels.asm");
    assert_eq!(result.symbol_table.get("FIRST"), Some(0x3000));
    assert_eq!(result.symbol_table.get("SECOND"), Some(0x3001));
    assert_eq!(result.symbol_table.get("THIRD"), Some(0x3002));
}

#[test]
fn stress_program() {
    let result = run_pipeline("tests/test_programs/stress.asm");
    assert_eq!(result.symbol_table.get("ENTRY"), Some(0x3000));
    assert_eq!(result.symbol_table.get("DONE"), Some(0x3012));
}
