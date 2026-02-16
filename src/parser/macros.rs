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

/// Macro to generate parsers for reg-reg-or-imm instructions (ADD, AND)
///
/// These instructions can operate in two modes:
/// - Register mode: `ADD R1, R2, R3` (add R2 and R3, store in R1)
/// - Immediate mode: `ADD R1, R2, #5` (add R2 and 5, store in R1)
macro_rules! parse_reg_reg_or_imm {
    ($name:expr, $reg_variant:expr, $imm_variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 6 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: format!("{} requires 3 operands: {} DR, SR1, SR2/imm5", $name, $name),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            $crate::parser::macros::expect_comma(tokens, 4, "Expected comma after second operand")?;
            let dr = $crate::parser::macros::expect_register(tokens, 1, &format!("{} first operand must be a register (R0-R7)", $name))?;
            let sr1 = $crate::parser::macros::expect_register(tokens, 3, &format!("{} second operand must be a register (R0-R7)", $name))?;

            if let Some(sr2) = $crate::parser::macros::token_to_register(tokens[5]) {
                $crate::parser::macros::ensure_no_extra(tokens, 6)?;
                Ok(LineContent::Instruction($reg_variant(dr, sr1, sr2)))
            } else if let Some(imm) = $crate::parser::macros::token_to_i32(tokens[5]) {
                $crate::parser::macros::ensure_no_extra(tokens, 6)?;
                Ok(LineContent::Instruction($imm_variant(dr, sr1, imm as i16)))
            } else {
                Err(AsmError {
                    kind: ErrorKind::InvalidOperandType,
                    message: format!("{} third operand must be a register (R0-R7) or immediate (#n)", $name),
                    span: tokens[5].span,
                })
            }
        }
    };
}

/// Macro to generate parsers for reg-label instructions (LD, LDI, LEA, ST, STI)
macro_rules! parse_reg_label {
    ($name:expr, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 4 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: format!("{} requires 2 operands: {} DR, LABEL", $name, $name),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            let reg = $crate::parser::macros::expect_register(tokens, 1, &format!("{} first operand must be a register (R0-R7)", $name))?;
            let label = $crate::parser::macros::expect_label(tokens, 3, &format!("{} requires a label operand", $name))?;
            $crate::parser::macros::ensure_no_extra(tokens, 4)?;
            Ok(LineContent::Instruction($variant(reg, label)))
        }
    };
}

/// Macro to generate parsers for reg-reg-imm instructions (LDR, STR)
macro_rules! parse_reg_reg_imm {
    ($name:expr, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 6 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: format!("{} requires 3 operands: {} DR, BaseR, #offset6", $name, $name),
                    span: tokens[0].span,
                });
            }
            $crate::parser::macros::expect_comma(tokens, 2, "Expected comma after first operand")?;
            $crate::parser::macros::expect_comma(tokens, 4, "Expected comma after second operand")?;
            let r1 = $crate::parser::macros::expect_register(tokens, 1, &format!("{} first operand must be a register (R0-R7)", $name))?;
            let r2 = $crate::parser::macros::expect_register(tokens, 3, &format!("{} second operand must be a register (R0-R7)", $name))?;
            let value = $crate::parser::macros::token_to_i32(tokens[5]).ok_or_else(|| AsmError {
                kind: ErrorKind::InvalidOperandType,
                message: format!("{} third operand must be an immediate (#n)", $name),
                span: tokens[5].span,
            })?;
            $crate::parser::macros::ensure_no_extra(tokens, 6)?;
            Ok(LineContent::Instruction($variant(r1, r2, value as i16)))
        }
    };
}

/// Macro to generate parsers for single-register instructions (JMP, JSRR)
macro_rules! parse_single_reg {
    ($name:expr, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 2 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: format!("{} requires 1 operand: {} BaseR", $name, $name),
                    span: tokens[0].span,
                });
            }
            let base_r = $crate::parser::macros::expect_register(tokens, 1, &format!("{} operand must be a register (R0-R7)", $name))?;
            $crate::parser::macros::ensure_no_extra(tokens, 2)?;
            Ok(LineContent::Instruction($variant(base_r)))
        }
    };
}

/// Macro to generate parsers for single-label instructions (JSR)
macro_rules! parse_single_label {
    ($name:expr, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() < 2 {
                return Err(AsmError {
                    kind: ErrorKind::TooFewOperands,
                    message: format!("{} requires 1 operand: {} LABEL", $name, $name),
                    span: tokens[0].span,
                });
            }
            let label = $crate::parser::macros::expect_label(tokens, 1, &format!("{} requires a label operand", $name))?;
            $crate::parser::macros::ensure_no_extra(tokens, 2)?;
            Ok(LineContent::Instruction($variant(label)))
        }
    };
}

/// Macro to generate parsers for no-operand instructions (RTI, RET, GETC, etc.)
macro_rules! parse_no_operands {
    ($name:expr, $variant:expr) => {
        |tokens: &[&$crate::lexer::token::Token]| -> Result<$crate::parser::ast::LineContent, $crate::error::AsmError> {
            use $crate::error::{AsmError, ErrorKind};
            use $crate::parser::ast::LineContent;

            if tokens.len() > 1 {
                return Err(AsmError {
                    kind: ErrorKind::TooManyOperands,
                    message: format!("{} takes no operands", $name),
                    span: tokens[1].span,
                });
            }
            Ok(LineContent::Instruction($variant))
        }
    };
}

// Helper functions used by macros (must be public for macro access)
pub use super::{
    ensure_no_extra, expect_comma, expect_label, expect_register, token_to_i32, token_to_register,
};
