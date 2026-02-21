//! # Token Types
//!
//! Defines all token types for the LC-3 assembly language.
//!
//! ## Token Structure
//!
//! Each token contains:
//! - `kind`: The token type (opcode, register, literal, etc.)
//! - `lexeme`: The original text from source code
//! - `span`: Location in source for error reporting

use crate::error::Span;

/// A single lexical token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

/// Token types for LC-3 assembly language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // === LC-3 Opcodes ===
    OpAdd,
    OpAnd,
    OpNot,
    OpBr(BrFlags), // Branch with condition flags
    OpJmp,
    OpJsr,
    OpJsrr,
    OpLd,
    OpLdi,
    OpLdr,
    OpLea,
    OpSt,
    OpSti,
    OpStr,
    OpTrap,
    OpRti,

    // === Pseudo-ops ===
    PseudoRet,   // JMP R7 (return from subroutine) — NOT a TRAP instruction
    PseudoGetc,  // TRAP x20
    PseudoOut,   // TRAP x21
    PseudoPuts,  // TRAP x22
    PseudoIn,    // TRAP x23
    PseudoPutsp, // TRAP x24
    PseudoHalt,  // TRAP x25

    // === Assembler Directives ===
    DirOrig,    // .ORIG
    DirEnd,     // .END
    DirFill,    // .FILL
    DirBlkw,    // .BLKW
    DirStringz, // .STRINGZ

    // === Operands ===
    Register(u8), // R0-R7

    NumDecimal(i32), // #123 or #-45
    NumHex(i32),     // x3000 (two's complement signed)
    NumBinary(i32),  // b1010 (two's complement signed)

    StringLiteral(String), // "hello\n"

    Label(String), // Identifier (uppercase)

    // === Punctuation & Structural ===
    Comma,
    Newline,
    Comment(String),

    Eof,
}

/// Branch condition flags for BR instruction
///
/// The LC-3 BR instruction can branch based on the condition codes:
/// - N (negative): branch if result was negative
/// - Z (zero): branch if result was zero
/// - P (positive): branch if result was positive
///
/// Multiple flags can be combined: BRnz, BRnp, BRzp, BRnzp
/// BR with no flags is equivalent to BRnzp (unconditional branch)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrFlags {
    pub n: bool, // Negative flag
    pub z: bool, // Zero flag
    pub p: bool, // Positive flag
}

impl BrFlags {
    pub fn new(n: bool, z: bool, p: bool) -> Self {
        Self { n, z, p }
    }

    /// Parse BrFlags from a string like "BR", "BRN", "BRNZP", etc.
    pub fn parse(s: &str) -> Option<Self> {
        if !s.starts_with("BR") {
            return None;
        }

        let flags_part = &s[2..];
        if flags_part.is_empty() {
            // BR with no flags = branch always (NZP)
            return Some(Self::new(true, true, true));
        }

        let mut n = false;
        let mut z = false;
        let mut p = false;

        for ch in flags_part.chars() {
            match ch {
                'N' => n = true,
                'Z' => z = true,
                'P' => p = true,
                _ => return None, // unknown char → not a valid BR variant
            }
        }
        // Note: the guard "!n && !z && !p" was removed because it is unreachable:
        // • empty flags_part returns early above (BRnzp)
        // • non-empty flags_part either sets ≥1 flag or the `_ => return None` fires first
        Some(Self::new(n, z, p))
    }

    /// Convert flags to 3-bit encoding: [N][Z][P]
    ///
    /// Used in machine code generation: bits [11:9] of BR instruction
    pub fn as_u16(&self) -> u16 {
        ((self.n as u16) << 2) | ((self.z as u16) << 1) | (self.p as u16)
    }
}

impl std::fmt::Display for BrFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.n {
            f.write_str("n")?;
        }
        if self.z {
            f.write_str("z")?;
        }
        if self.p {
            f.write_str("p")?;
        }
        Ok(())
    }
}

impl TokenKind {
    pub fn is_instruction_or_directive(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            // Operate instructions
            OpAdd | OpAnd | OpNot | OpBr(_) |
            // Control flow
            OpJmp | OpJsr | OpJsrr | OpRti |
            // Data movement
            OpLd | OpLdi | OpLdr | OpLea | OpSt | OpSti | OpStr |
            // Trap & pseudos
            OpTrap | PseudoRet | PseudoGetc | PseudoOut | PseudoPuts | PseudoIn | PseudoPutsp | PseudoHalt |
            // Directives
            DirOrig | DirEnd | DirFill | DirBlkw | DirStringz
        )
    }
}
