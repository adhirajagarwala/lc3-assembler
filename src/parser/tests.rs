#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use crate::parser::ast::{Instruction, LineContent};
    use crate::parser::parse_lines;

    fn parse_ok(input: &str) -> Vec<crate::parser::ast::SourceLine> {
        let lexed = tokenize(input);
        assert!(lexed.errors.is_empty(), "Lexer errors: {:?}", lexed.errors);
        let parsed = parse_lines(&lexed.tokens);
        assert!(
            parsed.errors.is_empty(),
            "Parser errors: {:?}",
            parsed.errors
        );
        parsed.lines
    }

    fn parse_errors(input: &str) -> Vec<crate::error::ErrorKind> {
        let lexed = tokenize(input);
        let parsed = parse_lines(&lexed.tokens);
        parsed.errors.into_iter().map(|e| e.kind).collect()
    }

    #[test]
    fn parse_add_register() {
        let lines = parse_ok("ADD R1, R2, R3");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::AddReg {
                dr: 1,
                sr1: 2,
                sr2: 3
            })
        );
    }

    #[test]
    fn parse_add_immediate() {
        let lines = parse_ok("ADD R1, R2, #5");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::AddImm {
                dr: 1,
                sr1: 2,
                imm5: 5
            })
        );
    }

    #[test]
    fn parse_and_register() {
        let lines = parse_ok("AND R1, R2, R3");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::AndReg {
                dr: 1,
                sr1: 2,
                sr2: 3
            })
        );
    }

    #[test]
    fn parse_and_immediate() {
        let lines = parse_ok("AND R1, R2, #-1");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::AndImm {
                dr: 1,
                sr1: 2,
                imm5: -1
            })
        );
    }

    #[test]
    fn parse_not() {
        let lines = parse_ok("NOT R1, R2");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Not { dr: 1, sr: 2 })
        );
    }

    #[test]
    fn parse_ld() {
        let lines = parse_ok("LD R0, DATA");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "DATA".into()
            })
        );
    }

    #[test]
    fn parse_ldi() {
        let lines = parse_ok("LDI R0, PTR");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Ldi {
                dr: 0,
                label: "PTR".into()
            })
        );
    }

    #[test]
    fn parse_ldr() {
        let lines = parse_ok("LDR R0, R1, #5");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Ldr {
                dr: 0,
                base_r: 1,
                offset6: 5
            })
        );
    }

    #[test]
    fn parse_lea() {
        let lines = parse_ok("LEA R0, MSG");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Lea {
                dr: 0,
                label: "MSG".into()
            })
        );
    }

    #[test]
    fn parse_st() {
        let lines = parse_ok("ST R0, RESULT");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::St {
                sr: 0,
                label: "RESULT".into()
            })
        );
    }

    #[test]
    fn parse_sti() {
        let lines = parse_ok("STI R0, PTR");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Sti {
                sr: 0,
                label: "PTR".into()
            })
        );
    }

    #[test]
    fn parse_str() {
        let lines = parse_ok("STR R0, R1, #0");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Str {
                sr: 0,
                base_r: 1,
                offset6: 0
            })
        );
    }

    #[test]
    fn parse_br_with_flags() {
        let lines = parse_ok("BRnz LOOP");
        if let LineContent::Instruction(Instruction::Br { flags, label }) = &lines[0].content {
            assert!(flags.n && flags.z && !flags.p);
            assert_eq!(label, "LOOP");
        } else {
            panic!("Expected BR instruction");
        }
    }

    #[test]
    fn parse_jmp() {
        let lines = parse_ok("JMP R3");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Jmp { base_r: 3 })
        );
    }

    #[test]
    fn parse_jsr() {
        let lines = parse_ok("JSR SUB");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Jsr {
                label: "SUB".into()
            })
        );
    }

    #[test]
    fn parse_jsrr() {
        let lines = parse_ok("JSRR R3");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Jsrr { base_r: 3 })
        );
    }

    #[test]
    fn parse_trap() {
        let lines = parse_ok("TRAP x25");
        assert_eq!(
            lines[0].content,
            LineContent::Instruction(Instruction::Trap { trapvect8: 0x25 })
        );
    }

    #[test]
    fn parse_rti() {
        let lines = parse_ok("RTI");
        assert_eq!(lines[0].content, LineContent::Instruction(Instruction::Rti));
    }

    #[test]
    fn parse_ret() {
        let lines = parse_ok("RET");
        assert_eq!(lines[0].content, LineContent::Instruction(Instruction::Ret));
    }

    #[test]
    fn parse_halt() {
        let lines = parse_ok("HALT");
        assert_eq!(lines[0].content, LineContent::Instruction(Instruction::Halt));
    }

    #[test]
    fn parse_orig() {
        let lines = parse_ok(".ORIG x3000");
        assert_eq!(lines[0].content, LineContent::Orig(0x3000));
    }

    #[test]
    fn parse_end() {
        let lines = parse_ok(".END");
        assert_eq!(lines[0].content, LineContent::End);
    }

    #[test]
    fn parse_fill_number() {
        let lines = parse_ok(".FILL #42");
        assert_eq!(lines[0].content, LineContent::FillImmediate(42));
    }

    #[test]
    fn parse_fill_hex() {
        let lines = parse_ok(".FILL xBEEF");
        assert_eq!(lines[0].content, LineContent::FillImmediate(0xBEEF));
    }

    #[test]
    fn parse_fill_label() {
        let lines = parse_ok(".FILL MYVAR");
        assert_eq!(
            lines[0].content,
            LineContent::FillLabel("MYVAR".into())
        );
    }

    #[test]
    fn parse_blkw() {
        let lines = parse_ok(".BLKW #5");
        assert_eq!(lines[0].content, LineContent::Blkw(5));
    }

    #[test]
    fn parse_stringz() {
        let lines = parse_ok(".STRINGZ \"Hello\"");
        assert_eq!(lines[0].content, LineContent::Stringz("Hello".into()));
    }

    #[test]
    fn parse_label_only_line() {
        let lines = parse_ok("LOOP\n");
        assert_eq!(lines[0].label, Some("LOOP".into()));
        assert_eq!(lines[0].content, LineContent::Empty);
    }

    #[test]
    fn parse_label_with_instr() {
        let lines = parse_ok("LOOP ADD R1, R1, #1");
        assert_eq!(lines[0].label, Some("LOOP".into()));
        assert!(matches!(
            lines[0].content,
            LineContent::Instruction(Instruction::AddImm { .. })
        ));
    }

    #[test]
    fn parse_label_with_dir() {
        let lines = parse_ok("DATA .FILL #0");
        assert_eq!(lines[0].label, Some("DATA".into()));
        assert_eq!(lines[0].content, LineContent::FillImmediate(0));
    }

    #[test]
    fn parse_missing_operand() {
        let errors = parse_errors("ADD R1, R2");
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_extra_operand() {
        let errors = parse_errors("NOT R1, R2, R3");
        assert!(!errors.is_empty());
    }

    #[test]
    fn parse_missing_comma() {
        let errors = parse_errors("ADD R1 R2 R3");
        assert!(!errors.is_empty());
    }
}
