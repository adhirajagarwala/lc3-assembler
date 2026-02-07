#[cfg(test)]
mod tests {
    use crate::first_pass::first_pass;
    use crate::lexer::tokenize;
    use crate::parser::parse_lines;

    fn run_first_pass(input: &str) -> crate::first_pass::FirstPassResult {
        let lexed = tokenize(input);
        assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
        let parsed = parse_lines(&lexed.tokens);
        assert!(parsed.errors.is_empty(), "Parser errors: {:?}", parsed.errors);
        first_pass(&parsed.lines)
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
        assert!(result.errors.iter().any(|e| matches!(e.kind, crate::error::ErrorKind::DuplicateLabel)));
    }

    #[test]
    fn missing_orig_error() {
        let result = run_first_pass("ADD R1, R2, R3\n.END\n");
        assert!(result.errors.iter().any(|e| matches!(e.kind, crate::error::ErrorKind::MissingOrig)));
    }

    #[test]
    fn missing_end_error() {
        let result = run_first_pass(".ORIG x3000\nADD R1, R2, R3\n");
        assert!(result.errors.iter().any(|e| matches!(e.kind, crate::error::ErrorKind::MissingEnd)));
    }

    #[test]
    fn content_after_end_ignored() {
        let result = run_first_pass(
            ".ORIG x3000\nHALT\n.END\nADD R1, R2, R3\nEXTRA_LABEL ADD R1, R1, #1\n",
        );
        assert!(result.symbol_table.get("EXTRA_LABEL").is_none());
    }

    #[test]
    fn fill_with_label_still_advances() {
        let result = run_first_pass(".ORIG x3000\nDATA .FILL SOMEVAR\nNEXT ADD R1, R1, #1\n.END\n");
        assert_eq!(result.symbol_table.get("DATA"), Some(0x3000));
        assert_eq!(result.symbol_table.get("NEXT"), Some(0x3001));
    }
}
