//! # Parser Macros
//!
//! Declarative macros for generating instruction parsing functions.
//!
//! These macros eliminate ~150 lines of repetitive parsing code by generating
//! parsers for similar instruction patterns. Each macro validates operand counts,
//! checks for proper comma placement, and constructs the appropriate AST node.
//!
//! ## Benefits
//!
//! - **Consistency**: All instructions with the same pattern are parsed identically
//! - **Maintainability**: Fixing a bug in the pattern fixes it for all instructions
//! - **Readability**: The main parser file is much cleaner and easier to understand
//!
//! ## Error Messages
//!
//! All macro parameters use `$name:literal` so that `concat!` can produce
//! `&'static str` error messages at compile time, avoiding per-call heap
//! allocation on the error path. Only messages that interpolate runtime values
//! (e.g. the actual out-of-range immediate) still use `format!`.

/// Macro to generate parsers for reg-reg-or-imm instructions (ADD, AND)
///
/// These instructions can operate in two modes:
/// - Register mode: `ADD R1, R2, R3` (add R2 and R3, store in R1)
/// - Immediate mode: `ADD R1, R2, #5` (add R2 and 5, store in R1)
macro_rules! parse_reg_reg_or_imm {
    ($name:literal, $reg_variant:expr, $imm_variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 6 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: concat!($name, " requires 3 operands: ", $name, " DR, SR1, SR2/imm5").into(),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            $crate::parser::macros::expect_comma(tokens, 4, "Expected comma after second operand")?;
            let dr = $crate::parser::macros::expect_register(tokens, 1, concat!($name, " first operand must be a register (R0-R7)"))?;
            let sr1 = $crate::parser::macros::expect_register(tokens, 3, concat!($name, " second operand must be a register (R0-R7)"))?;

            if let Some(sr2) = $crate::parser::macros::token_to_register(tokens[5]) {
                $crate::parser::macros::ensure_no_extra(tokens, 6)?;
                Ok(LineContent::Instruction($reg_variant(dr, sr1, sr2)))
            } else if let Some(imm) = $crate::parser::macros::token_to_i32(tokens[5]) {
                // Validate the 5-bit signed immediate range (-16..=15).
                // Without this, `ADD R1, R1, #100` silently truncates to a wrong value.
                if !(-16..=15).contains(&imm) {
                    return Err(AsmError {
                        kind: ErrorKind::InvalidOperandType,
                        message: format!(
                            "{} immediate value {} is out of 5-bit signed range (-16 to 15)",
                            $name, imm
                        ),
                        span: tokens[5].span,
                    });
                }
                $crate::parser::macros::ensure_no_extra(tokens, 6)?;
                Ok(LineContent::Instruction($imm_variant(dr, sr1, imm as i16)))
            } else {
                Err(AsmError {
                    kind: ErrorKind::InvalidOperandType,
                    message: concat!($name, " third operand must be a register (R0-R7) or immediate (#n)").into(),
                    span: tokens[5].span,
                })
            }
        }
    };
}

/// Macro to generate parsers for reg-label instructions (LD, LDI, LEA, ST, STI)
macro_rules! parse_reg_label {
    ($name:literal, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 4 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: concat!($name, " requires 2 operands: ", $name, " DR, LABEL").into(),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            let reg = $crate::parser::macros::expect_register(tokens, 1, concat!($name, " first operand must be a register (R0-R7)"))?;
            let label = $crate::parser::macros::expect_label(tokens, 3, concat!($name, " requires a label operand"))?;
            $crate::parser::macros::ensure_no_extra(tokens, 4)?;
            Ok(LineContent::Instruction($variant(reg, label)))
        }
    };
}

/// Macro to generate parsers for reg-reg-imm instructions (LDR, STR)
macro_rules! parse_reg_reg_imm {
    ($name:literal, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 6 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: concat!($name, " requires 3 operands: ", $name, " DR, BaseR, #offset6").into(),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            $crate::parser::macros::expect_comma(tokens, 4, "Expected comma after second operand")?;
            let r1 = $crate::parser::macros::expect_register(tokens, 1, concat!($name, " first operand must be a register (R0-R7)"))?;
            let r2 = $crate::parser::macros::expect_register(tokens, 3, concat!($name, " second operand must be a register (R0-R7)"))?;
            let value = $crate::parser::macros::token_to_i32(tokens[5]).ok_or_else(|| AsmError {
                kind: ErrorKind::InvalidOperandType,
                message: concat!($name, " third operand must be an immediate (#n)").into(),
                span: tokens[5].span,
            })?;
            // Validate the 6-bit signed offset range (-32..=31).
            // Without this, `LDR R0, R1, #100` silently truncates to a wrong offset.
            if !(-32..=31).contains(&value) {
                return Err(AsmError {
                    kind: ErrorKind::InvalidOperandType,
                    message: format!(
                        "{} offset value {} is out of 6-bit signed range (-32 to 31)",
                        $name, value
                    ),
                    span: tokens[5].span,
                });
            }
            $crate::parser::macros::ensure_no_extra(tokens, 6)?;
            Ok(LineContent::Instruction($variant(r1, r2, value as i16)))
        }
    };
}

/// Macro to generate parsers for single-register instructions (JMP, JSRR)
macro_rules! parse_single_reg {
    ($name:literal, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 2 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: concat!($name, " requires 1 operand: ", $name, " BaseR").into(),
                    span: tokens[0].span,
                });
            }
            let base_r = $crate::parser::macros::expect_register(tokens, 1, concat!($name, " operand must be a register (R0-R7)"))?;
            $crate::parser::macros::ensure_no_extra(tokens, 2)?;
            Ok(LineContent::Instruction($variant(base_r)))
        }
    };
}

/// Macro to generate parsers for single-label instructions (JSR)
macro_rules! parse_single_label {
    ($name:literal, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 2 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: concat!($name, " requires 1 operand: ", $name, " LABEL").into(),
                    span: tokens[0].span,
                });
            }
            let label = $crate::parser::macros::expect_label(tokens, 1, concat!($name, " requires a label operand"))?;
            $crate::parser::macros::ensure_no_extra(tokens, 2)?;
            Ok(LineContent::Instruction($variant(label)))
        }
    };
}

/// Macro to generate parsers for no-operand instructions (RTI, RET, GETC, etc.)
macro_rules! parse_no_operands {
    ($name:literal, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() > 1 {
                return Err(AsmError {
                    kind: ErrorKind::TooManyOperands,
                    message: concat!($name, " takes no operands").into(),
                    span: tokens[1].span,
                });
            }
            Ok(LineContent::Instruction($variant))
        }
    };
}

// Re-export helpers at parser::macros so macro expansions can reach them via
// `$crate::parser::macros::*`. Using pub(crate) keeps them out of the public API.
pub(crate) use super::{
    ensure_no_extra, expect_comma, expect_label, expect_register, token_to_i32, token_to_register,
};
