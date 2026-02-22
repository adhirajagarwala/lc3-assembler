use crate::first_pass::first_pass;
use crate::lexer::tokenize;
use crate::parser::parse_lines;

fn run_first_pass(input: &str) -> crate::first_pass::FirstPassResult {
    let lexed = tokenize(input);
    assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
    let parsed = parse_lines(&lexed.tokens);
    assert!(
        parsed.errors.is_empty(),
        "Parser errors: {:?}",
        parsed.errors
    );
    first_pass(parsed.lines)
}

#[test]
fn simple_symbol_table() {
    let result = run_first_pass(".ORIG x3000\nLOOP ADD R1, R1, #-1\nBRp LOOP\nHALT\n.END\n");
    assert_eq!(result.symbol_table.get("LOOP"), Some(0x3000));
}

#[test]
fn multiple_labels() {
    let result = run_first_pass(
        ".ORIG x3000\nLOOP ADD R1, R1, #-1\nDATA .FILL #0\nMSG .STRINGZ \"Hi\"\n.END\n",
    );
    assert_eq!(result.symbol_table.get("LOOP"), Some(0x3000));
    assert_eq!(result.symbol_table.get("DATA"), Some(0x3001));
    assert_eq!(result.symbol_table.get("MSG"), Some(0x3002));
}

#[test]
fn label_only_line() {
    let result = run_first_pass(".ORIG x3000\nLOOP\nADD R1, R1, #-1\n.END\n");
    assert_eq!(result.symbol_table.get("LOOP"), Some(0x3000));
}

#[test]
fn blkw_advances_correctly() {
    let result = run_first_pass(".ORIG x3000\n.BLKW #10\nNEXT ADD R1, R1, #1\n.END\n");
    assert_eq!(result.symbol_table.get("NEXT"), Some(0x300A));
}

#[test]
fn stringz_advances_correctly() {
    let result = run_first_pass(".ORIG x3000\n.STRINGZ \"Hello\"\nNEXT ADD R1, R1, #1\n.END\n");
    assert_eq!(result.symbol_table.get("NEXT"), Some(0x3006));
}

#[test]
fn empty_stringz() {
    let result = run_first_pass(".ORIG x3000\n.STRINGZ \"\"\nNEXT ADD R1, R1, #1\n.END\n");
    assert_eq!(result.symbol_table.get("NEXT"), Some(0x3001));
}

#[test]
fn duplicate_label_error() {
    let result = run_first_pass(".ORIG x3000\nLOOP ADD R1, R1, #1\nLOOP ADD R1, R1, #1\n.END\n");
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e.kind, crate::error::ErrorKind::DuplicateLabel)));
}

#[test]
fn missing_orig_error() {
    let result = run_first_pass("ADD R1, R2, R3\n.END\n");
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e.kind, crate::error::ErrorKind::MissingOrig)));
}

#[test]
fn missing_end_error() {
    let result = run_first_pass(".ORIG x3000\nADD R1, R2, R3\n");
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e.kind, crate::error::ErrorKind::MissingEnd)));
}

#[test]
fn content_after_end_ignored() {
    let result =
        run_first_pass(".ORIG x3000\nHALT\n.END\nADD R1, R2, R3\nEXTRA_LABEL ADD R1, R1, #1\n");
    assert!(result.symbol_table.get("EXTRA_LABEL").is_none());
}

#[test]
fn fill_with_label_still_advances() {
    let result = run_first_pass(".ORIG x3000\nDATA .FILL SOMEVAR\nNEXT ADD R1, R1, #1\n.END\n");
    assert_eq!(result.symbol_table.get("DATA"), Some(0x3000));
    assert_eq!(result.symbol_table.get("NEXT"), Some(0x3001));
}

#[test]
fn address_overflow_error() {
    // .ORIG xFFF0 + .BLKW #100 would push the location counter past 0xFFFF.
    let result = run_first_pass(".ORIG xFFF0\n.BLKW #100\n.END\n");
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e.kind, crate::error::ErrorKind::AddressOverflow)));
}

#[test]
fn orig_with_label() {
    // A label on the .ORIG line should record the label at the origin address.
    let result = run_first_pass("START .ORIG x3000\nADD R1, R1, #1\n.END\n");
    assert_eq!(result.symbol_table.get("START"), Some(0x3000));
}

// Instruction mnemonics (ADD, AND, …) and register names (R0–R7) are tokenised by the
// lexer as their own TokenKind variants, so they can never appear as Label tokens and
// therefore never reach record_label.  Only bare directive names (without a leading dot)
// fall through to Label and are checked for reserved-word collisions.

#[test]
fn label_shadows_directive_fill() {
    // `FILL` without a dot is a Label token → should be flagged as a reserved word.
    let result = run_first_pass(".ORIG x3000\nFILL .FILL #0\n.END\n");
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e.kind, crate::error::ErrorKind::LabelIsReservedWord)),
        "Expected LabelIsReservedWord for bare FILL label, got: {:?}",
        result.errors
    );
}

#[test]
fn label_shadows_directive_blkw() {
    let result = run_first_pass(".ORIG x3000\nBLKW .BLKW #3\n.END\n");
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e.kind, crate::error::ErrorKind::LabelIsReservedWord)),
        "Expected LabelIsReservedWord for bare BLKW label, got: {:?}",
        result.errors
    );
}

#[test]
fn label_shadows_directive_stringz() {
    let result = run_first_pass(".ORIG x3000\nSTRINGZ .STRINGZ \"hi\"\n.END\n");
    assert!(
        result
            .errors
            .iter()
            .any(|e| matches!(e.kind, crate::error::ErrorKind::LabelIsReservedWord)),
        "Expected LabelIsReservedWord for bare STRINGZ label, got: {:?}",
        result.errors
    );
}

#[test]
fn normal_label_no_reserved_word_error() {
    // A non-reserved label must not trigger the check.
    let result = run_first_pass(".ORIG x3000\nLOOP ADD R1, R1, #-1\n.END\n");
    assert!(
        !result
            .errors
            .iter()
            .any(|e| matches!(e.kind, crate::error::ErrorKind::LabelIsReservedWord)),
        "LOOP should not be considered a reserved word"
    );
}
