//! # Abstract Syntax Tree (AST)
//!
//! Defines the data structures representing parsed LC-3 assembly code.
//!
//! ## Structure
//!
//! A program is represented as a sequence of `SourceLine` objects, where each line
//! contains:
//! - An optional label
//! - The line content (instruction, directive, or empty)
//! - Source location information for error reporting
//!
//! ## Design Philosophy
//!
//! The AST is designed to be simple and directly map to the LC-3 ISA. Each
//! instruction variant explicitly lists its operands, making the encoder's job
//! straightforward.

use crate::error::Span;
use crate::lexer::token::BrFlags;

/// A single line of LC-3 assembly source code
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLine {
    /// Optional label defined on this line
    pub label: Option<String>,
    /// The actual content of the line
    pub content: LineContent,
    /// Line number in source file (1-indexed)
    pub line_number: usize,
    /// Source span for error reporting
    pub span: Span,
}

/// Content of a source line (instruction, directive, or empty)
#[derive(Debug, Clone, PartialEq)]
pub enum LineContent {
    /// Empty line or comment-only line
    Empty,
    /// .ORIG directive - sets program origin address
    Orig(u16),
    /// .END directive - marks end of program
    End,
    /// .FILL directive with immediate value
    FillImmediate(i32),
    /// .FILL directive with label reference
    FillLabel(String),
    /// .BLKW directive - allocates N words
    Blkw(u16),
    /// .STRINGZ directive - null-terminated string
    Stringz(String),
    /// LC-3 instruction
    Instruction(Instruction),
}

impl LineContent {
    /// Calculate how many words this line content will occupy in memory
    pub fn word_count(&self) -> u32 {
        match self {
            LineContent::Empty => 0,
            LineContent::Orig(_) => 0,
            LineContent::End => 0,
            LineContent::FillImmediate(_) => 1,
            LineContent::FillLabel(_) => 1,
            LineContent::Blkw(n) => *n as u32,
            LineContent::Stringz(s) => (s.len() as u32) + 1, // +1 for null terminator
            LineContent::Instruction(_) => 1,
        }
    }
}

/// LC-3 Instruction
///
/// Each variant explicitly represents an LC-3 instruction with its operands.
/// Register operands are stored as u8 (0-7), immediates as i16.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // === Operate Instructions ===
    /// ADD (register mode): DR = SR1 + SR2
    AddReg { dr: u8, sr1: u8, sr2: u8 },
    /// ADD (immediate mode): DR = SR1 + imm5
    AddImm { dr: u8, sr1: u8, imm5: i16 },
    /// AND (register mode): DR = SR1 & SR2
    AndReg { dr: u8, sr1: u8, sr2: u8 },
    /// AND (immediate mode): DR = SR1 & imm5
    AndImm { dr: u8, sr1: u8, imm5: i16 },
    /// NOT: DR = ~SR (bitwise complement)
    Not { dr: u8, sr: u8 },

    // === Data Movement (PC-relative) ===
    /// LD: Load from PC-relative address
    Ld { dr: u8, label: String },
    /// LDI: Load indirect from PC-relative address
    Ldi { dr: u8, label: String },
    /// LDR: Load from base register + offset
    Ldr { dr: u8, base_r: u8, offset6: i16 },
    /// LEA: Load effective address (PC + offset)
    Lea { dr: u8, label: String },
    /// ST: Store to PC-relative address
    St { sr: u8, label: String },
    /// STI: Store indirect to PC-relative address
    Sti { sr: u8, label: String },
    /// STR: Store to base register + offset
    Str { sr: u8, base_r: u8, offset6: i16 },

    // === Control Flow ===
    /// BR: Conditional branch based on NZP flags
    Br { flags: BrFlags, label: String },
    /// JMP: Jump to address in register (RET is JMP R7)
    Jmp { base_r: u8 },
    /// JSR: Jump to subroutine (PC-relative)
    Jsr { label: String },
    /// JSRR: Jump to subroutine (register)
    Jsrr { base_r: u8 },
    /// RET: Return from subroutine (pseudo-op for JMP R7)
    Ret,
    /// RTI: Return from interrupt
    Rti,

    // === Trap & System ===
    /// TRAP: System call with 8-bit trap vector
    Trap { trapvect8: u8 },
    /// GETC: Read character from keyboard (TRAP x20)
    Getc,
    /// OUT: Write character to console (TRAP x21)
    Out,
    /// PUTS: Write string to console (TRAP x22)
    Puts,
    /// IN: Read character with prompt (TRAP x23)
    In,
    /// PUTSP: Write packed string to console (TRAP x24)
    Putsp,
    /// HALT: Stop execution (TRAP x25)
    Halt,
}
