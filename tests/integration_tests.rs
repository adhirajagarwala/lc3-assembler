use std::fs;

use lc3_assembler::encoder::encode;
use lc3_assembler::error::ErrorKind;
use lc3_assembler::first_pass::first_pass;
use lc3_assembler::lexer::tokenize;
use lc3_assembler::parser::parse_lines;

/// Run lexer → parser → first pass, asserting no errors at any stage.
fn run_pipeline(path: &str) -> lc3_assembler::first_pass::FirstPassResult {
    let source = fs::read_to_string(path).expect("Failed to read test program");
    let lexed = tokenize(&source);
    assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
    let parsed = parse_lines(&lexed.tokens);
    assert!(
        parsed.errors.is_empty(),
        "Parser errors: {:?}",
        parsed.errors
    );
    let result = first_pass(parsed.lines);
    assert!(
        result.errors.is_empty(),
        "First pass errors: {:?}",
        result.errors
    );
    result
}

/// Run the full pipeline (lexer → parser → first pass → encoder), asserting no errors.
fn run_full_pipeline(path: &str) -> lc3_assembler::encoder::EncodeResult {
    let source = fs::read_to_string(path).expect("Failed to read test program");
    let lexed = tokenize(&source);
    assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
    let parsed = parse_lines(&lexed.tokens);
    assert!(
        parsed.errors.is_empty(),
        "Parser errors: {:?}",
        parsed.errors
    );
    let first = first_pass(parsed.lines);
    assert!(
        first.errors.is_empty(),
        "First pass errors: {:?}",
        first.errors
    );
    let encoded = encode(&first);
    assert!(
        encoded.errors.is_empty(),
        "Encoder errors: {:?}",
        encoded.errors
    );
    encoded
}

/// Run the pipeline on a source string and collect all errors from every stage.
fn collect_all_errors(source: &str) -> Vec<ErrorKind> {
    let mut kinds = Vec::new();
    let lexed = tokenize(source);
    kinds.extend(lexed.errors.iter().map(|e| e.kind.clone()));
    let parsed = parse_lines(&lexed.tokens);
    kinds.extend(parsed.errors.iter().map(|e| e.kind.clone()));
    let first = first_pass(parsed.lines);
    kinds.extend(first.errors.iter().map(|e| e.kind.clone()));
    let encoded = encode(&first);
    kinds.extend(encoded.errors.iter().map(|e| e.kind.clone()));
    kinds
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

// ========== ENCODER INTEGRATION TESTS ==========

#[test]
fn encode_hello_program() {
    let encoded = run_full_pipeline("tests/test_programs/hello.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    assert!(
        !encoded.machine_code.is_empty(),
        "Should generate machine code"
    );
    // LEA R0, MSG (0xE002) + PUTS (0xF022) + HALT (0xF025) + "Hello" + null
    assert_eq!(encoded.machine_code[0], 0xE002); // LEA R0, offset=2
    assert_eq!(encoded.machine_code[1], 0xF022); // PUTS
    assert_eq!(encoded.machine_code[2], 0xF025); // HALT
    assert_eq!(encoded.machine_code[3], 'H' as u16);
    assert_eq!(encoded.machine_code[4], 'e' as u16);
    assert_eq!(encoded.machine_code[5], 'l' as u16);
    assert_eq!(encoded.machine_code[6], 'l' as u16);
    assert_eq!(encoded.machine_code[7], 'o' as u16);
    assert_eq!(encoded.machine_code[8], 0x0000); // null terminator
}

#[test]
fn encode_all_instructions() {
    let encoded = run_full_pipeline("tests/test_programs/all_instructions.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    // Verify we have machine code for all instructions
    assert!(
        encoded.machine_code.len() >= 18,
        "Should have at least 18 words"
    );

    // Verify opcode nibbles are correct for first few instructions
    assert_eq!(
        encoded.machine_code[0] >> 12,
        0x1,
        "First instruction should be ADD (0001)"
    );
    assert_eq!(
        encoded.machine_code[1] >> 12,
        0x5,
        "Second instruction should be AND (0101)"
    );
    assert_eq!(
        encoded.machine_code[2] >> 12,
        0x9,
        "Third instruction should be NOT (1001)"
    );
}

#[test]
fn encode_trap_aliases() {
    let encoded = run_full_pipeline("tests/test_programs/trap_aliases.asm");
    assert_eq!(encoded.orig_address, 0x3000);

    // Check that trap aliases are properly encoded
    let has_getc = encoded.machine_code.contains(&0xF020);
    let has_out = encoded.machine_code.contains(&0xF021);
    let has_puts = encoded.machine_code.contains(&0xF022);
    let has_halt = encoded.machine_code.contains(&0xF025);

    assert!(has_getc || has_out || has_puts, "Should have TRAP aliases");
    assert!(has_halt, "Should have HALT");
}

#[test]
fn encode_blkw_directive() {
    let encoded = run_full_pipeline("tests/test_programs/large_blkw.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    // Should have instructions + 20 zeros from .BLKW
    assert!(encoded.machine_code.len() >= 20, "Should allocate 20 words");
}

#[test]
fn encode_fill_directive() {
    let encoded = run_full_pipeline("tests/test_programs/all_directives.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    // Should contain .FILL values
    assert!(!encoded.machine_code.is_empty());
}

#[test]
fn encode_stringz_directive() {
    let encoded = run_full_pipeline("tests/test_programs/hello.asm");
    // Check for string data with null terminator
    let has_null = encoded.machine_code.contains(&0x0000);
    assert!(has_null, "STRINGZ should end with null terminator");
}

#[test]
fn encode_pc_offset_calculation() {
    let encoded = run_full_pipeline("tests/test_programs/countdown.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    // Program has BR instruction that should have valid PC offset
    assert!(
        !encoded.machine_code.is_empty(),
        "Should encode branch instructions"
    );
}

#[test]
fn encode_preserves_orig_address() {
    let encoded = run_full_pipeline("tests/test_programs/hello.asm");
    assert_eq!(encoded.orig_address, 0x3000);

    let encoded2 = run_full_pipeline("tests/test_programs/all_instructions.asm");
    assert_eq!(encoded2.orig_address, 0x3000);
}

// ========== LOOP PROGRAM ==========

#[test]
fn loop_program() {
    let result = run_pipeline("tests/test_programs/loop.asm");
    assert_eq!(result.symbol_table.get("LOOP"), Some(0x3002));
}

#[test]
fn encode_loop_program() {
    let encoded = run_full_pipeline("tests/test_programs/loop.asm");
    assert_eq!(encoded.orig_address, 0x3000);
    // AND R1, R1, #0 → clear R1
    assert_eq!(encoded.machine_code[0] >> 12, 0x5, "First should be AND");
    // ADD R1, R1, #5
    assert_eq!(encoded.machine_code[1] >> 12, 0x1, "Second should be ADD");
    // ADD R1, R1, #-1  (loop body)
    assert_eq!(encoded.machine_code[2] >> 12, 0x1, "Third should be ADD");
    // BRp LOOP — branch back
    assert_eq!(encoded.machine_code[3] >> 12, 0x0, "Fourth should be BR");
    // HALT
    assert_eq!(encoded.machine_code[4], 0xF025);
}

// ========== ERROR-PATH TESTS ==========

#[test]
fn error_undefined_label() {
    let source = ".ORIG x3000\nLD R0, NOWHERE\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::UndefinedLabel),
        "Expected UndefinedLabel error, got: {:?}",
        errors
    );
}

#[test]
fn error_duplicate_label() {
    let source = ".ORIG x3000\nFOO ADD R0, R0, #1\nFOO ADD R1, R1, #2\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::DuplicateLabel),
        "Expected DuplicateLabel error, got: {:?}",
        errors
    );
}

#[test]
fn error_missing_orig() {
    let source = "ADD R0, R0, #1\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::MissingOrig),
        "Expected MissingOrig error, got: {:?}",
        errors
    );
}

#[test]
fn error_imm5_out_of_range() {
    let source = ".ORIG x3000\nADD R1, R1, #100\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidOperandType),
        "Expected InvalidOperandType for imm5 out of range, got: {:?}",
        errors
    );
}

#[test]
fn error_offset6_out_of_range() {
    let source = ".ORIG x3000\nLDR R0, R1, #100\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidOperandType),
        "Expected InvalidOperandType for offset6 out of range, got: {:?}",
        errors
    );
}

#[test]
fn error_too_few_operands() {
    let source = ".ORIG x3000\nADD R1, R2\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::TooFewOperands),
        "Expected TooFewOperands error, got: {:?}",
        errors
    );
}

#[test]
fn error_invalid_orig_address() {
    // x10000 overflows the lexer's 16-bit hex parser, so it's caught as InvalidHexLiteral
    // before the parser ever sees it. This is correct: the lexer rejects it first.
    let source = ".ORIG x10000\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidHexLiteral),
        "Expected InvalidHexLiteral for oversized hex literal, got: {:?}",
        errors
    );
}

#[test]
fn error_invalid_orig_decimal() {
    // Decimal 70000 is parseable but exceeds the 16-bit .ORIG range.
    let source = ".ORIG #70000\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidOrigAddress),
        "Expected InvalidOrigAddress for decimal out of range, got: {:?}",
        errors
    );
}

#[test]
fn error_invalid_blkw_count() {
    let source = ".ORIG x3000\n.BLKW #0\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidBlkwCount),
        "Expected InvalidBlkwCount error, got: {:?}",
        errors
    );
}

#[test]
fn error_trap_vector_out_of_range() {
    let source = ".ORIG x3000\nTRAP x1FF\n.END\n";
    let errors = collect_all_errors(source);
    assert!(
        errors.contains(&ErrorKind::InvalidOperandType),
        "Expected InvalidOperandType for TRAP vector out of range, got: {:?}",
        errors
    );
}

#[test]
fn errors_asm_file_produces_errors() {
    // The errors.asm test program has intentional errors — verify the pipeline catches them.
    let source =
        fs::read_to_string("tests/test_programs/errors.asm").expect("Failed to read errors.asm");
    let errors = collect_all_errors(&source);
    assert!(
        !errors.is_empty(),
        "errors.asm should produce at least one error"
    );
}
