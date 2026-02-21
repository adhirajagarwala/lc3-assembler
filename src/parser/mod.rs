//! # LC-3 Parser
//!
//! Parses tokenized LC-3 assembly code into an Abstract Syntax Tree (AST).
//!
//! ## Architecture
//!
//! The parser processes tokens line by line, handling:
//! - Optional labels at the start of lines
//! - Instructions with their operands
//! - Assembler directives (.ORIG, .FILL, etc.)
//! - Comments (filtered out during parsing)
//!
//! ## Macro-Based Parsing
//!
//! To eliminate code duplication, the parser uses declarative macros to generate
//! parsing functions for similar instruction patterns:
//! - `parse_reg_reg_or_imm!` - ADD, AND (register or immediate mode)
//! - `parse_reg_label!` - LD, LDI, LEA, ST, STI (PC-relative addressing)
//! - `parse_reg_reg_imm!` - LDR, STR (base+offset addressing)
//! - `parse_single_reg!` - JMP, JSRR (single register operand)
//! - `parse_single_label!` - JSR (single label operand)
//! - `parse_no_operands!` - RET, HALT, etc. (no operands)
//!
//! This reduced the parser from 606 to 450 lines (-26% code reduction).

#[macro_use]
mod macros;
pub mod ast;

#[cfg(test)]
mod tests;

use crate::error::{AsmError, ErrorKind, Span};
use crate::lexer::token::{Token, TokenKind};
use ast::{Instruction, LineContent, SourceLine};

pub struct ParseResult {
    pub lines: Vec<SourceLine>,
    pub errors: Vec<AsmError>,
}

#[must_use]
pub fn parse_lines(tokens: &[Token]) -> ParseResult {
    let mut lines = Vec::new();
    let mut errors = Vec::new();
    let mut line_start = 0;
    let mut line_number = 1;

    for (i, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::Newline => {
                process_line(&tokens[line_start..i], line_number, &mut lines, &mut errors);
                line_start = i + 1;
                line_number += 1;
            }
            TokenKind::Eof => {
                process_line(&tokens[line_start..i], line_number, &mut lines, &mut errors);
                break;
            }
            _ => {}
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
    // Strip any trailing comment token.  `position` short-circuits after finding the
    // first comment rather than evaluating the predicate for every token afterward.
    let code_end = tokens
        .iter()
        .position(|t| matches!(t.kind, TokenKind::Comment(_)))
        .unwrap_or(tokens.len());
    let filtered: Vec<&Token> = tokens[..code_end].iter().collect();

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

    match &first.kind {
        TokenKind::Label(name) => {
            label = Some(name.clone());
            if filtered.len() > 1 && filtered[1].kind.is_instruction_or_directive() {
                content_tokens = &filtered[1..];
            } else {
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
    let first = tokens[0];

    // Use macros to generate parsers inline
    match &first.kind {
        // Operate instructions - consolidated with macros
        TokenKind::OpAdd => parse_reg_reg_or_imm!(
            "ADD",
            |dr, sr1, sr2| Instruction::AddReg { dr, sr1, sr2 },
            |dr, sr1, imm5| Instruction::AddImm { dr, sr1, imm5 }
        )(tokens),

        TokenKind::OpAnd => parse_reg_reg_or_imm!(
            "AND",
            |dr, sr1, sr2| Instruction::AndReg { dr, sr1, sr2 },
            |dr, sr1, imm5| Instruction::AndImm { dr, sr1, imm5 }
        )(tokens),

        TokenKind::OpNot => parse_not(tokens),
        TokenKind::OpBr(flags) => parse_br(tokens, *flags),

        // Data movement - PC offset (consolidated with macros)
        TokenKind::OpLd => {
            parse_reg_label!("LD", |dr, label| Instruction::Ld { dr, label })(tokens)
        }
        TokenKind::OpLdi => {
            parse_reg_label!("LDI", |dr, label| Instruction::Ldi { dr, label })(tokens)
        }
        TokenKind::OpLea => {
            parse_reg_label!("LEA", |dr, label| Instruction::Lea { dr, label })(tokens)
        }
        TokenKind::OpSt => {
            parse_reg_label!("ST", |sr, label| Instruction::St { sr, label })(tokens)
        }
        TokenKind::OpSti => {
            parse_reg_label!("STI", |sr, label| Instruction::Sti { sr, label })(tokens)
        }

        // Data movement - base+offset (consolidated with macros)
        TokenKind::OpLdr => parse_reg_reg_imm!("LDR", |dr, base_r, offset6| Instruction::Ldr {
            dr,
            base_r,
            offset6
        })(tokens),
        TokenKind::OpStr => parse_reg_reg_imm!("STR", |sr, base_r, offset6| Instruction::Str {
            sr,
            base_r,
            offset6
        })(tokens),

        // Control flow (consolidated with macros)
        TokenKind::OpJmp => parse_single_reg!("JMP", |base_r| Instruction::Jmp { base_r })(tokens),
        TokenKind::OpJsr => parse_single_label!("JSR", |label| Instruction::Jsr { label })(tokens),
        TokenKind::OpJsrr => {
            parse_single_reg!("JSRR", |base_r| Instruction::Jsrr { base_r })(tokens)
        }

        // Trap
        TokenKind::OpTrap => parse_trap(tokens),

        // No-operand instructions (consolidated with macros)
        TokenKind::OpRti => parse_no_operands!("RTI", Instruction::Rti)(tokens),
        TokenKind::PseudoRet => parse_no_operands!("RET", Instruction::Ret)(tokens),
        TokenKind::PseudoGetc => parse_no_operands!("GETC", Instruction::Getc)(tokens),
        TokenKind::PseudoOut => parse_no_operands!("OUT", Instruction::Out)(tokens),
        TokenKind::PseudoPuts => parse_no_operands!("PUTS", Instruction::Puts)(tokens),
        TokenKind::PseudoIn => parse_no_operands!("IN", Instruction::In)(tokens),
        TokenKind::PseudoPutsp => parse_no_operands!("PUTSP", Instruction::Putsp)(tokens),
        TokenKind::PseudoHalt => parse_no_operands!("HALT", Instruction::Halt)(tokens),

        // Directives
        TokenKind::DirOrig => parse_orig(tokens),
        TokenKind::DirEnd => parse_end(tokens),
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

// Only need custom parsing for unique instructions

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

fn parse_br(
    tokens: &[&Token],
    flags: crate::lexer::token::BrFlags,
) -> Result<LineContent, AsmError> {
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
    // Trap vector must fit in 8 bits (0x00–0xFF). Without this check,
    // `TRAP x1FF` silently truncates to `TRAP xFF`, producing wrong machine code.
    if !(0..=0xFF).contains(&value) {
        return Err(AsmError {
            kind: ErrorKind::InvalidOperandType,
            message: format!("TRAP vector {} is out of range (must be 0x00-0xFF)", value),
            span: tokens[1].span,
        });
    }
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
    // Accept any value whose 16-bit representation is valid (0x0000–0xFFFF).
    // Hex/binary literals above 0x7FFF arrive as negative i32 values due to the
    // two's complement conversion in the lexer (e.g. xFFFF → -1, x8000 → -32768).
    // Decimal literals arrive as positive i32 (e.g. #65535 → 65535).
    // Both representations must be accepted for the full 16-bit address space.
    if !(i16::MIN as i32..=0xFFFF_i32).contains(&value) {
        return Err(AsmError {
            kind: ErrorKind::InvalidOrigAddress,
            message: ".ORIG address must be 0x0000-0xFFFF".into(),
            span: tokens[1].span,
        });
    }
    ensure_no_extra(tokens, 2)?;
    Ok(LineContent::Orig(value as u16))
}

fn parse_end(tokens: &[&Token]) -> Result<LineContent, AsmError> {
    if tokens.len() > 1 {
        return Err(AsmError {
            kind: ErrorKind::TooManyOperands,
            message: ".END takes no operands".into(),
            span: tokens[1].span,
        });
    }
    Ok(LineContent::End)
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
        // Validate the value fits in a 16-bit slot.
        // Hex/binary > 0x7FFF arrive as negative i32 (e.g. xFFFF → -1), so we
        // accept -32768..=65535 to cover the full unsigned 16-bit range.
        if !(i16::MIN as i32..=0xFFFF_i32).contains(&value) {
            return Err(AsmError {
                kind: ErrorKind::InvalidOperandType,
                message: format!(
                    ".FILL value {} is out of 16-bit range (-32768 to 65535)",
                    value
                ),
                span: tokens[1].span,
            });
        }
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
    // Reject negative or zero counts now, in the parser, with a clear error.
    // Without this check, `.BLKW #-1` silently casts to 65535 (u16::MAX) and
    // allocates a 65535-word block, producing silently wrong output.
    if value <= 0 || value > 0xFFFF {
        return Err(AsmError {
            kind: ErrorKind::InvalidBlkwCount,
            message: format!(".BLKW count {} is out of range (must be 1-65535)", value),
            span: tokens[1].span,
        });
    }
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

// Helper functions — pub(crate) so macros in macros.rs can call them via
// `$crate::parser::macros::*` without exposing them in the public library API.
pub(crate) fn ensure_no_extra(tokens: &[&Token], expected_len: usize) -> Result<(), AsmError> {
    if tokens.len() > expected_len {
        return Err(AsmError {
            kind: ErrorKind::UnexpectedToken,
            message: "Unexpected token after instruction".into(),
            span: tokens[expected_len].span,
        });
    }
    Ok(())
}

pub(crate) fn expect_comma(tokens: &[&Token], idx: usize, message: &str) -> Result<(), AsmError> {
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

pub(crate) fn expect_register(
    tokens: &[&Token],
    idx: usize,
    message: &str,
) -> Result<u8, AsmError> {
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

pub(crate) fn expect_label(
    tokens: &[&Token],
    idx: usize,
    message: &str,
) -> Result<String, AsmError> {
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

pub(crate) fn token_to_i32(token: &Token) -> Option<i32> {
    match &token.kind {
        TokenKind::NumDecimal(v) => Some(*v),
        TokenKind::NumHex(v) => Some(*v),
        TokenKind::NumBinary(v) => Some(*v),
        _ => None,
    }
}

pub(crate) fn token_to_register(token: &Token) -> Option<u8> {
    match &token.kind {
        TokenKind::Register(r) => Some(*r),
        _ => None,
    }
}

pub(crate) fn token_to_label(token: &Token) -> Option<String> {
    match &token.kind {
        TokenKind::Label(s) => Some(s.clone()),
        _ => None,
    }
}
