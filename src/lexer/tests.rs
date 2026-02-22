#[cfg(test)]
mod lexer_tests {
    use crate::error::ErrorKind;
    use crate::lexer::token::{BrFlags, TokenKind};
    use crate::lexer::tokenize;

    fn lex_ok(input: &str) -> Vec<TokenKind> {
        let result = tokenize(input);
        assert!(
            result.errors.is_empty(),
            "Unexpected errors: {:?}",
            result.errors
        );
        result.tokens.into_iter().map(|t| t.kind).collect()
    }

    fn lex_errors(input: &str) -> Vec<ErrorKind> {
        let result = tokenize(input);
        result.errors.into_iter().map(|e| e.kind).collect()
    }

    #[test]
    fn empty_input() {
        assert_eq!(lex_ok(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn blank_lines() {
        assert_eq!(
            lex_ok("\n\n"),
            vec![TokenKind::Newline, TokenKind::Newline, TokenKind::Eof]
        );
    }

    #[test]
    fn comment_only() {
        assert_eq!(
            lex_ok("; hello\n"),
            vec![
                TokenKind::Comment(" hello".into()),
                TokenKind::Newline,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn all_opcodes() {
        let kinds = lex_ok("ADD AND NOT LD LDI LDR LEA ST STI STR JMP JSR JSRR TRAP RTI");
        assert_eq!(
            kinds,
            vec![
                TokenKind::OpAdd,
                TokenKind::OpAnd,
                TokenKind::OpNot,
                TokenKind::OpLd,
                TokenKind::OpLdi,
                TokenKind::OpLdr,
                TokenKind::OpLea,
                TokenKind::OpSt,
                TokenKind::OpSti,
                TokenKind::OpStr,
                TokenKind::OpJmp,
                TokenKind::OpJsr,
                TokenKind::OpJsrr,
                TokenKind::OpTrap,
                TokenKind::OpRti,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn opcodes_case_insens() {
        let kinds = lex_ok("add Add ADD");
        assert_eq!(
            kinds,
            vec![
                TokenKind::OpAdd,
                TokenKind::OpAdd,
                TokenKind::OpAdd,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn all_br_variants() {
        let kinds = lex_ok("BR BRn BRz BRp BRnz BRnp BRzp BRnzp");
        assert_eq!(
            kinds,
            vec![
                TokenKind::OpBr(BrFlags::new(true, true, true)),
                TokenKind::OpBr(BrFlags::new(true, false, false)),
                TokenKind::OpBr(BrFlags::new(false, true, false)),
                TokenKind::OpBr(BrFlags::new(false, false, true)),
                TokenKind::OpBr(BrFlags::new(true, true, false)),
                TokenKind::OpBr(BrFlags::new(true, false, true)),
                TokenKind::OpBr(BrFlags::new(false, true, true)),
                TokenKind::OpBr(BrFlags::new(true, true, true)),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn br_case_insensitive() {
        let kinds = lex_ok("brn Brn BRN bRn");
        assert_eq!(
            kinds,
            vec![
                TokenKind::OpBr(BrFlags::new(true, false, false)),
                TokenKind::OpBr(BrFlags::new(true, false, false)),
                TokenKind::OpBr(BrFlags::new(true, false, false)),
                TokenKind::OpBr(BrFlags::new(true, false, false)),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn all_registers() {
        let kinds = lex_ok("R0 R1 R2 R3 R4 R5 R6 R7");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Register(0),
                TokenKind::Register(1),
                TokenKind::Register(2),
                TokenKind::Register(3),
                TokenKind::Register(4),
                TokenKind::Register(5),
                TokenKind::Register(6),
                TokenKind::Register(7),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn register_case() {
        let kinds = lex_ok("r0 R0");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Register(0),
                TokenKind::Register(0),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn invalid_register() {
        let errors = lex_errors("R8");
        assert_eq!(errors, vec![ErrorKind::InvalidRegister]);
    }

    #[test]
    fn all_directives() {
        let kinds = lex_ok(".ORIG .END .FILL .BLKW .STRINGZ");
        assert_eq!(
            kinds,
            vec![
                TokenKind::DirOrig,
                TokenKind::DirEnd,
                TokenKind::DirFill,
                TokenKind::DirBlkw,
                TokenKind::DirStringz,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn directives_case() {
        let kinds = lex_ok(".orig .Orig");
        assert_eq!(
            kinds,
            vec![TokenKind::DirOrig, TokenKind::DirOrig, TokenKind::Eof]
        );
    }

    #[test]
    fn unknown_directive() {
        let errors = lex_errors(".FOOBAR");
        assert_eq!(errors, vec![ErrorKind::UnknownDirective]);
    }

    #[test]
    fn all_pseudos() {
        let kinds = lex_ok("RET GETC OUT PUTS IN PUTSP HALT");
        assert_eq!(
            kinds,
            vec![
                TokenKind::PseudoRet,
                TokenKind::PseudoGetc,
                TokenKind::PseudoOut,
                TokenKind::PseudoPuts,
                TokenKind::PseudoIn,
                TokenKind::PseudoPutsp,
                TokenKind::PseudoHalt,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn decimal_positive() {
        assert_eq!(
            lex_ok("#10"),
            vec![TokenKind::NumDecimal(10), TokenKind::Eof]
        );
    }

    #[test]
    fn decimal_negative() {
        assert_eq!(
            lex_ok("#-5"),
            vec![TokenKind::NumDecimal(-5), TokenKind::Eof]
        );
    }

    #[test]
    fn decimal_zero() {
        assert_eq!(lex_ok("#0"), vec![TokenKind::NumDecimal(0), TokenKind::Eof]);
    }

    #[test]
    fn decimal_with_plus() {
        assert_eq!(
            lex_ok("#+3"),
            vec![TokenKind::NumDecimal(3), TokenKind::Eof]
        );
    }

    #[test]
    fn hex_literal() {
        assert_eq!(
            lex_ok("x3000"),
            vec![TokenKind::NumHex(0x3000), TokenKind::Eof]
        );
    }

    #[test]
    fn hex_case() {
        // 0xABCD > 0x7FFF, so it becomes negative in 16-bit two's complement
        assert_eq!(
            lex_ok("xAbCd"),
            vec![TokenKind::NumHex(-21555), TokenKind::Eof]
        );
        assert_eq!(
            lex_ok("XABCD"),
            vec![TokenKind::NumHex(-21555), TokenKind::Eof]
        );
    }

    #[test]
    fn binary_literal() {
        assert_eq!(
            lex_ok("b1010"),
            vec![TokenKind::NumBinary(10), TokenKind::Eof]
        );
    }

    #[test]
    fn octal_literal() {
        // 0o17 = 8+7 = 15
        assert_eq!(
            lex_ok("0o17"),
            vec![TokenKind::NumOctal(15), TokenKind::Eof]
        );
    }

    #[test]
    fn octal_literal_uppercase_prefix() {
        // Lexer uppercases everything, so 0O17 is the same as 0o17
        assert_eq!(
            lex_ok("0O17"),
            vec![TokenKind::NumOctal(15), TokenKind::Eof]
        );
    }

    #[test]
    fn octal_literal_zero() {
        assert_eq!(lex_ok("0o0"), vec![TokenKind::NumOctal(0), TokenKind::Eof]);
    }

    #[test]
    fn octal_literal_twos_complement() {
        // 0o177777 = 65535 = 0xFFFF → two's complement → -1
        assert_eq!(
            lex_ok("0o177777"),
            vec![TokenKind::NumOctal(-1), TokenKind::Eof]
        );
    }

    #[test]
    fn octal_literal_overflow() {
        // 0o200000 = 65536 → exceeds 16 bits
        let errors = lex_errors("0o200000");
        assert_eq!(errors, vec![ErrorKind::InvalidOctalLiteral]);
    }

    #[test]
    fn invalid_decimal() {
        let errors = lex_errors("#abc");
        assert_eq!(errors, vec![ErrorKind::InvalidDecimalLiteral]);
    }

    #[test]
    fn bare_hash() {
        let errors = lex_errors("#");
        assert_eq!(errors, vec![ErrorKind::InvalidDecimalLiteral]);
    }

    #[test]
    fn simple_string() {
        assert_eq!(
            lex_ok("\"Hello\""),
            vec![TokenKind::StringLiteral("Hello".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn string_escapes() {
        assert_eq!(
            lex_ok("\"Hi\\n\""),
            vec![TokenKind::StringLiteral("Hi\n".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn string_escaped_quote() {
        assert_eq!(
            lex_ok("\"say \\\"hi\\\"\""),
            vec![
                TokenKind::StringLiteral("say \"hi\"".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn string_backslash() {
        assert_eq!(
            lex_ok("\"a\\\\b\""),
            vec![TokenKind::StringLiteral("a\\b".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn string_null_char() {
        assert_eq!(
            lex_ok("\"a\\0b\""),
            vec![TokenKind::StringLiteral("a\0b".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn unterminated_string() {
        let errors = lex_errors("\"oops");
        assert_eq!(errors, vec![ErrorKind::UnterminatedString]);
    }

    #[test]
    fn empty_string() {
        assert_eq!(
            lex_ok("\"\""),
            vec![TokenKind::StringLiteral("".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn bad_escape() {
        let errors = lex_errors("\"bad\\q\"");
        // First error is invalid escape, second is unterminated string (error recovery)
        assert!(!errors.is_empty());
        assert_eq!(errors[0], ErrorKind::InvalidEscapeSequence);
    }

    #[test]
    fn simple_label() {
        assert_eq!(
            lex_ok("LOOP"),
            vec![TokenKind::Label("LOOP".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn label_underscore() {
        assert_eq!(
            lex_ok("my_data"),
            vec![TokenKind::Label("MY_DATA".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn label_uppercase_stored() {
        assert_eq!(
            lex_ok("myLabel"),
            vec![TokenKind::Label("MYLABEL".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn b_prefix_ambiguity() {
        assert_eq!(
            lex_ok("B0110"),
            vec![TokenKind::NumBinary(6), TokenKind::Eof]
        );
    }

    #[test]
    fn x_not_hex() {
        assert_eq!(
            lex_ok("xGHIJ"),
            vec![TokenKind::Label("XGHIJ".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn x_alone() {
        assert_eq!(
            lex_ok("x"),
            vec![TokenKind::Label("X".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn result_is_label() {
        assert_eq!(
            lex_ok("RESULT"),
            vec![TokenKind::Label("RESULT".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn in_is_pseudo() {
        assert_eq!(lex_ok("IN"), vec![TokenKind::PseudoIn, TokenKind::Eof]);
    }

    #[test]
    fn unexpected_char() {
        let errors = lex_errors("@");
        assert_eq!(errors, vec![ErrorKind::UnexpectedCharacter]);
    }

    #[test]
    fn full_add_line() {
        assert_eq!(
            lex_ok("ADD R1, R2, #5"),
            vec![
                TokenKind::OpAdd,
                TokenKind::Register(1),
                TokenKind::Comma,
                TokenKind::Register(2),
                TokenKind::Comma,
                TokenKind::NumDecimal(5),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn label_with_instr() {
        assert_eq!(
            lex_ok("LOOP ADD R1, R1, #-1"),
            vec![
                TokenKind::Label("LOOP".into()),
                TokenKind::OpAdd,
                TokenKind::Register(1),
                TokenKind::Comma,
                TokenKind::Register(1),
                TokenKind::Comma,
                TokenKind::NumDecimal(-1),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn line_with_comment() {
        assert_eq!(
            lex_ok("HALT ; done"),
            vec![
                TokenKind::PseudoHalt,
                TokenKind::Comment(" done".into()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn complete_program() {
        let kinds = lex_ok(".ORIG x3000\nADD R1, R1, #1\n.END\n");
        assert!(kinds.contains(&TokenKind::DirOrig));
        assert!(kinds.contains(&TokenKind::OpAdd));
        assert!(kinds.contains(&TokenKind::DirEnd));
        assert_eq!(kinds.last().unwrap(), &TokenKind::Eof);
    }

    #[test]
    fn multiple_errors() {
        let errors = lex_errors("@ # x-3 \"bad\\q\"");
        assert!(errors.len() >= 3);
    }

    #[test]
    fn spans_line_numbers() {
        let result = tokenize("ADD\nAND");
        assert_eq!(result.tokens[0].span.line, 1);
        assert_eq!(result.tokens[2].span.line, 2);
    }

    #[test]
    fn spans_column_numbers() {
        let result = tokenize("  ADD");
        assert_eq!(result.tokens[0].span.col, 3);
    }

    // ── 16-bit two's complement tests ───────────────────────────

    #[test]
    fn hex_full_range() {
        // xFFFF should parse as -1 (16-bit two's complement)
        assert_eq!(lex_ok("xFFFF")[0], TokenKind::NumHex(-1));
        // x8000 should parse as -32768
        assert_eq!(lex_ok("x8000")[0], TokenKind::NumHex(-32768));
        // x7FFF should stay positive
        assert_eq!(lex_ok("x7FFF")[0], TokenKind::NumHex(32767));
    }

    #[test]
    fn hex_overflow() {
        // Values exceeding 16 bits should error
        let errs = lex_errors("x10000");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], ErrorKind::InvalidHexLiteral);
    }

    #[test]
    fn binary_full_range() {
        // 16 ones should parse as -1
        assert_eq!(lex_ok("b1111111111111111")[0], TokenKind::NumBinary(-1));
        // MSB set should be -32768
        assert_eq!(lex_ok("b1000000000000000")[0], TokenKind::NumBinary(-32768));
    }

    #[test]
    fn binary_overflow() {
        // 17 bits should error
        let errs = lex_errors("b10000000000000000");
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], ErrorKind::InvalidBinaryLiteral);
    }

    #[test]
    fn string_carriage_return() {
        assert_eq!(
            lex_ok("\"a\\rb\"")[0],
            TokenKind::StringLiteral("a\rb".into())
        );
    }
}
