pub mod ast;

use crate::error::{AsmError, ErrorKind, Span};
use crate::lexer::token::{Token, TokenKind};
use ast::{Instruction, LineContent, SourceLine};

// TODO-HIGH: Refactor parse_content() match statement (30+ arms) into a dispatch table or macro system

pub struct ParseResult {
    pub lines: Vec<SourceLine>,
    pub errors: Vec<AsmError>,
}

pub fn parse_lines(tokens: &[Token]) -> ParseResult {
    let mut lines = Vec::new();
    let mut errors = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let mut line_number = 1;

    for token in tokens {
        match token.kind {
            TokenKind::Newline => {
                process_line(&current, line_number, &mut lines, &mut errors);
                current.clear();
                line_number += 1;
            }
            TokenKind::Eof => {
                process_line(&current, line_number, &mut lines, &mut errors);
                break;
            }
            _ => current.push(token.clone()),
        }
    }

    ParseResult { lines, errors }
}

fn process_line(
    tokens: &[Token],
    line_number: usize,
    lines: &mut Vec<SourceLine>,
    errors: &mut Vec<AsmError>,
) {
    let span = line_span(tokens, line_number);
    let filtered: Vec<&Token> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
        .collect();

    if filtered.is_empty() {
        lines.push(SourceLine {
            label: None,
            content: LineContent::Empty,
            line_number,
            span,
        });
        return;
    }

    let first = filtered[0];
    let mut label: Option<String> = None;
    let content_tokens: &[&Token];

    // TODO-MED: Consolidate two early returns with identical SourceLine Empty construction
    match &first.kind {
        TokenKind::Label(name) => {
            if filtered.len() == 1 {
                label = Some(name.clone());
                lines.push(SourceLine {
                    label,
                    content: LineContent::Empty,
                    line_number,
                    span,
                });
                return;
            }

            if filtered[1].kind.is_instruction_or_directive() {
                label = Some(name.clone());
                content_tokens = &filtered[1..];
            } else {
                label = Some(name.clone());
                lines.push(SourceLine {
                    label,
                    content: LineContent::Empty,
                    line_number,
                    span,
                });
                return;
            }
        }
        kind if kind.is_instruction_or_directive() => {
            content_tokens = &filtered[..];
        }
        _ => {
            errors.push(AsmError {
                kind: ErrorKind::UnexpectedToken,
                message: "Unexpected token at start of line".into(),
                span: first.span,
            });
            lines.push(SourceLine {
                label: None,
                content: LineContent::Empty,
                line_number,
                span,
            });
            return;
        }
    }

    match parse_content(content_tokens) {
        Ok(content) => lines.push(SourceLine {
            label,
            content,
            line_number,
            span,
        }),
        Err(err) => {
            errors.push(err);
            lines.push(SourceLine {
                label,
                content: LineContent::Empty,
                line_number,
                span,
            });
        }
    }
}

fn line_span(tokens: &[Token], line_number: usize) -> Span {
    if let (Some(first), Some(last)) = (tokens.first(), tokens.last()) {
        Span {
            start: first.span.start,
            end: last.span.end,
            line: first.span.line,
            col: first.span.col,
        }
    } else {
        Span {
            start: 0,
            end: 0,
            line: line_number,
            col: 1,
        }
    }
}

fn parse_content(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    // TODO-HIGH: Replace all 30+ match arms below with macro-generated dispatch table
    let first = tokens[0];
    match &first.kind {
        TokenKind::OpAdd => parse_add(tokens),
        TokenKind::OpAnd => parse_and(tokens),
        TokenKind::OpNot => parse_not(tokens),
        TokenKind::OpBr(flags) => parse_br(tokens, *flags),
        TokenKind::OpLd => parse_ld(tokens),
        TokenKind::OpLdi => parse_ldi(tokens),
        TokenKind::OpLdr => parse_ldr(tokens),
        TokenKind::OpLea => parse_lea(tokens),
        TokenKind::OpSt => parse_st(tokens),
        TokenKind::OpSti => parse_sti(tokens),
        TokenKind::OpStr => parse_str(tokens),
        TokenKind::OpJmp => parse_jmp(tokens),
        TokenKind::OpJsr => parse_jsr(tokens),
        TokenKind::OpJsrr => parse_jsrr(tokens),
        TokenKind::OpTrap => parse_trap(tokens),
        // TODO-LOW: Group RTI/RET/GETC/OUT/PUTS/IN/PUTSP/HALT into single handler
        TokenKind::OpRti => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Rti), "RTI"),
        TokenKind::PseudoRet => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Ret), "RET"),
        TokenKind::PseudoGetc => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Getc), "GETC"),
        TokenKind::PseudoOut => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Out), "OUT"),
        TokenKind::PseudoPuts => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Puts), "PUTS"),
        TokenKind::PseudoIn => ensure_no_operands(tokens, LineContent::Instruction(Instruction::In), "IN"),
        TokenKind::PseudoPutsp => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Putsp), "PUTSP"),
        TokenKind::PseudoHalt => ensure_no_operands(tokens, LineContent::Instruction(Instruction::Halt), "HALT"),
        TokenKind::DirOrig => parse_orig(tokens),
        TokenKind::DirEnd => ensure_no_operands(tokens, LineContent::End, ".END"),
        TokenKind::DirFill => parse_fill(tokens),
        TokenKind::DirBlkw => parse_blkw(tokens),
        TokenKind::DirStringz => parse_stringz(tokens),
        _ => Err(AsmError {
            kind: ErrorKind::UnexpectedToken,
            message: "Unexpected token in line".into(),
            span: first.span,
        }),
    }
}

// TODO-HIGH: Consolidate parse_add, parse_and into single macro for reg-reg-or-imm instructions
// TODO-HIGH: Consolidate parse_ld, parse_ldi, parse_lea, parse_st, parse_sti into parse_reg_label macro
// TODO-HIGH: Consolidate parse_ldr, parse_str into parse_reg_reg_imm macro
fn parse_add(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    // TODO-HIGH: Extract repetitive error construction (Err(AsmError { kind, message, span }))
    // into a helper function or builder to reduce ~500 lines of boilerplate across parser
    if tokens.len() < 6 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "ADD requires 3 operands: ADD DR, SR1, SR2/imm5".into(),
            span: tokens[0].span,
        });
    }
    expect_comma(tokens, 2, "Expected comma after first operand")?;
    expect_comma(tokens, 4, "Expected comma after second operand")?;
    let dr = expect_register(tokens, 1, "ADD first operand must be a register (R0-R7)")?;
    let sr1 = expect_register(tokens, 3, "ADD second operand must be a register (R0-R7)")?;
    if let Some(sr2) = token_to_register(tokens[5]) {
        ensure_no_extra(tokens, 6)?;
        Ok(LineContent::Instruction(Instruction::AddReg { dr, sr1, sr2 }))
    } else if let Some(imm) = token_to_i32(tokens[5]) {
        ensure_no_extra(tokens, 6)?;
        Ok(LineContent::Instruction(Instruction::AddImm {
            dr,
            sr1,
            imm5: imm as i16,
        }))
    } else {
        Err(AsmError {
            kind: ErrorKind::InvalidOperandType,
            message: "ADD third operand must be a register (R0-R7) or immediate (#n)".into(),
            span: tokens[5].span,
        })
    }
}

fn parse_and(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 6 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "AND requires 3 operands: AND DR, SR1, SR2/imm5".into(),
            span: tokens[0].span,
        });
    }
    expect_comma(tokens, 2, "Expected comma after first operand")?;
    expect_comma(tokens, 4, "Expected comma after second operand")?;
    let dr = expect_register(tokens, 1, "AND first operand must be a register (R0-R7)")?;
    let sr1 = expect_register(tokens, 3, "AND second operand must be a register (R0-R7)")?;
    if let Some(sr2) = token_to_register(tokens[5]) {
        ensure_no_extra(tokens, 6)?;
        Ok(LineContent::Instruction(Instruction::AndReg { dr, sr1, sr2 }))
    } else if let Some(imm) = token_to_i32(tokens[5]) {
        ensure_no_extra(tokens, 6)?;
        Ok(LineContent::Instruction(Instruction::AndImm {
            dr,
            sr1,
            imm5: imm as i16,
        }))
    } else {
        Err(AsmError {
            kind: ErrorKind::InvalidOperandType,
            message: "AND third operand must be a register (R0-R7) or immediate (#n)".into(),
            span: tokens[5].span,
        })
    }
}

fn parse_not(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 4 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "NOT requires 2 operands: NOT DR, SR".into(),
            span: tokens[0].span,
        });
    }
    expect_comma(tokens, 2, "Expected comma after first operand")?;
    let dr = expect_register(tokens, 1, "NOT first operand must be a register (R0-R7)")?;
    let sr = expect_register(tokens, 3, "NOT second operand must be a register (R0-R7)")?;
    ensure_no_extra(tokens, 4)?;
    Ok(LineContent::Instruction(Instruction::Not { dr, sr }))
}

fn parse_br(tokens: &[&Token], flags: crate::lexer::token::BrFlags) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "BR requires a label operand".into(),
            span: tokens[0].span,
        });
    }
    let label = expect_label(tokens, 1, "BR requires a label operand")?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Instruction(Instruction::Br { flags, label }))
}

fn parse_ld(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_label(tokens, "LD", |dr, label| {
        LineContent::Instruction(Instruction::Ld { dr, label })
    })
}

fn parse_ldi(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_label(tokens, "LDI", |dr, label| {
        LineContent::Instruction(Instruction::Ldi { dr, label })
    })
}

fn parse_lea(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_label(tokens, "LEA", |dr, label| {
        LineContent::Instruction(Instruction::Lea { dr, label })
    })
}

fn parse_st(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_label(tokens, "ST", |sr, label| {
        LineContent::Instruction(Instruction::St { sr, label })
    })
}

fn parse_sti(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_label(tokens, "STI", |sr, label| {
        LineContent::Instruction(Instruction::Sti { sr, label })
    })
}

fn parse_ldr(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_reg_imm(tokens, "LDR", |dr, base_r, offset6| {
        LineContent::Instruction(Instruction::Ldr { dr, base_r, offset6 })
    })
}

fn parse_str(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    parse_reg_reg_imm(tokens, "STR", |sr, base_r, offset6| {
        LineContent::Instruction(Instruction::Str { sr, base_r, offset6 })
    })
}

fn parse_jmp(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "JMP requires 1 operand: JMP BaseR".into(),
            span: tokens[0].span,
        });
    }
    let base_r = expect_register(tokens, 1, "JMP operand must be a register (R0-R7)")?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Instruction(Instruction::Jmp { base_r }))
}

fn parse_jsr(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "JSR requires 1 operand: JSR LABEL".into(),
            span: tokens[0].span,
        });
    }
    let label = expect_label(tokens, 1, "JSR requires a label operand")?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Instruction(Instruction::Jsr { label }))
}

fn parse_jsrr(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "JSRR requires 1 operand: JSRR BaseR".into(),
            span: tokens[0].span,
        });
    }
    let base_r = expect_register(tokens, 1, "JSRR operand must be a register (R0-R7)")?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Instruction(Instruction::Jsrr { base_r }))
}

fn parse_trap(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: "TRAP requires a numeric trap vector (e.g., TRAP x25)".into(),
            span: tokens[0].span,
        });
    }
    let value = token_to_i32(tokens[1]).ok_or_else(|| AsmError {
        kind: ErrorKind::InvalidOperandType,
        message: "TRAP requires a numeric trap vector (e.g., TRAP x25)".into(),
        span: tokens[1].span,
    })?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Instruction(Instruction::Trap {
        trapvect8: value as u8,
    }))
}

fn parse_orig(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: ".ORIG requires a numeric operand".into(),
            span: tokens[0].span,
        });
    }
    let value = token_to_i32(tokens[1]).ok_or_else(|| AsmError {
        kind: ErrorKind::InvalidOperandType,
        message: ".ORIG requires a numeric operand".into(),
        span: tokens[1].span,
    })?;
    if value < 0 || value > 0xFFFF {
        return Err(AsmError {
            kind: ErrorKind::InvalidOrigAddress,
            message: ".ORIG address must be 0x0000-0xFFFF".into(),
            span: tokens[1].span,
        });
    }
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Orig(value as u16))
}

fn parse_fill(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: ".FILL requires a numeric or label operand".into(),
            span: tokens[0].span,
        });
    }
    if let Some(value) = token_to_i32(tokens[1]) {
        ensure_no_extra(tokens, 2)?;
        Ok(LineContent::FillImmediate(value))
    } else if let Some(label) = token_to_label(tokens[1]) {
        ensure_no_extra(tokens, 2)?;
        Ok(LineContent::FillLabel(label))
    } else {
        Err(AsmError {
            kind: ErrorKind::InvalidOperandType,
            message: ".FILL requires a numeric or label operand".into(),
            span: tokens[1].span,
        })
    }
}

fn parse_blkw(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: ".BLKW requires a numeric operand".into(),
            span: tokens[0].span,
        });
    }
    let value = token_to_i32(tokens[1]).ok_or_else(|| AsmError {
        kind: ErrorKind::InvalidOperandType,
        message: ".BLKW requires a numeric operand".into(),
        span: tokens[1].span,
    })?;
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Blkw(value as u16))
}

fn parse_stringz(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() < 2 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: ".STRINGZ requires a string literal operand".into(),
            span: tokens[0].span,
        });
    }
    match &tokens[1].kind {
        TokenKind::StringLiteral(s) => {
            ensure_no_extra(tokens, 2)?;
            Ok(LineContent::Stringz(s.clone()))
        }
        _ => Err(AsmError {
            kind: ErrorKind::InvalidOperandType,
            message: ".STRINGZ requires a string literal operand".into(),
            span: tokens[1].span,
        }),
    }
}

fn parse_reg_label<F>(tokens: &[&Token], name: &str, f: F) -> Result<LineContent, AsmError>
where
    F: Fn(u8, String) -> LineContent,
{
    // TODO-MED: Consider merging parse_reg_label and parse_reg_reg_imm into unified generic helper
    if tokens.len() < 4 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: format!("{} requires 2 operands: {} DR, LABEL", name, name),
            span: tokens[0].span,
        });
    }
    expect_comma(tokens, 2, "Expected comma after first operand")?;
    let reg = expect_register(tokens, 1, &format!("{} first operand must be a register (R0-R7)", name))?;
    let label = expect_label(tokens, 3, &format!("{} requires a label operand", name))?;
    ensure_no_extra(tokens, 4)?;
    Ok(f(reg, label))
}

fn parse_reg_reg_imm<F>(tokens: &[&Token], name: &str, f: F) -> Result<LineContent, AsmError>
where
    F: Fn(u8, u8, i16) -> LineContent,
{
    if tokens.len() < 6 {
        return Err(AsmError {
            kind: ErrorKind::TooFewOperands,
            message: format!("{} requires 3 operands: {} DR, BaseR, #offset6", name, name),
            span: tokens[0].span,
        });
    }
    expect_comma(tokens, 2, "Expected comma after first operand")?;
    expect_comma(tokens, 4, "Expected comma after second operand")?;
    let r1 = expect_register(tokens, 1, &format!("{} first operand must be a register (R0-R7)", name))?;
    let r2 = expect_register(tokens, 3, &format!("{} second operand must be a register (R0-R7)", name))?;
    let value = token_to_i32(tokens[5]).ok_or_else(|| AsmError {
        kind: ErrorKind::InvalidOperandType,
        message: format!("{} third operand must be an immediate (#n)", name),
        span: tokens[5].span,
    })?;
    ensure_no_extra(tokens, 6)?;
    Ok(f(r1, r2, value as i16))
}

fn ensure_no_operands(tokens: &[&Token], content: LineContent, name: &str) -> Result<LineContent, AsmError> {
    if tokens.len() > 1 {
        return Err(AsmError {
            kind: ErrorKind::TooManyOperands,
            message: format!("{} takes no operands", name),
            span: tokens[1].span,
        });
    }
    Ok(content)
}

// TODO-MED: Consolidate validation helpers with a builder pattern or macro
// TODO-MED: These expect_* and ensure_* helpers could be consolidated with a validator trait/macro
fn ensure_no_extra(tokens: &[&Token], expected_len: usize) -> Result<(), AsmError> {
    if tokens.len() > expected_len {
        return Err(AsmError {
            kind: ErrorKind::UnexpectedToken,
            message: "Unexpected token after instruction".into(),
            span: tokens[expected_len].span,
        });
    }
    Ok(())
}

fn expect_comma(tokens: &[&Token], idx: usize, message: &str) -> Result<(), AsmError> {
    if tokens.len() <= idx {
        return Err(AsmError {
            kind: ErrorKind::ExpectedComma,
            message: message.into(),
            span: tokens[0].span,
        });
    }
    match tokens[idx].kind {
        TokenKind::Comma => Ok(()),
        _ => Err(AsmError {
            kind: ErrorKind::ExpectedComma,
            message: message.into(),
            span: tokens[idx].span,
        }),
    }
}

fn expect_register(tokens: &[&Token], idx: usize, message: &str) -> Result<u8, AsmError> {
    if tokens.len() <= idx {
        return Err(AsmError {
            kind: ErrorKind::ExpectedRegister,
            message: message.into(),
            span: tokens[0].span,
        });
    }
    token_to_register(tokens[idx]).ok_or_else(|| AsmError {
        kind: ErrorKind::ExpectedRegister,
        message: message.into(),
        span: tokens[idx].span,
    })
}

fn expect_label(tokens: &[&Token], idx: usize, message: &str) -> Result<String, AsmError> {
    if tokens.len() <= idx {
        return Err(AsmError {
            kind: ErrorKind::ExpectedOperand,
            message: message.into(),
            span: tokens[0].span,
        });
    }
    token_to_label(tokens[idx]).ok_or_else(|| AsmError {
        kind: ErrorKind::ExpectedOperand,
        message: message.into(),
        span: tokens[idx].span,
    })
}

// TODO-MED: Replace these token conversion helpers with a trait or macro to reduce duplication
fn token_to_i32(token: &Token) -> Option<i32> {
    match &token.kind {
        TokenKind::NumDecimal(v) => Some(*v),
        TokenKind::NumHex(v) => Some(*v),
        TokenKind::NumBinary(v) => Some(*v),
        _ => None,
    }
}

fn token_to_register(token: &Token) -> Option<u8> {
    match &token.kind {
        TokenKind::Register(r) => Some(*r),
        _ => None,
    }
}

fn token_to_label(token: &Token) -> Option<String> {
    match &token.kind {
        TokenKind::Label(s) => Some(s.clone()),
        _ => None,
    }
}
