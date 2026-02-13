use crate::error::Span;
use crate::lexer::token::BrFlags;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLine {
    pub label: Option<String>,
    pub content: LineContent,
    pub line_number: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineContent {
    Empty,
    Orig(u16),
    End,
    FillImmediate(i32),
    FillLabel(String),
    Blkw(u16),
    Stringz(String),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    AddReg { dr: u8, sr1: u8, sr2: u8 },
    AddImm { dr: u8, sr1: u8, imm5: i16 },
    AndReg { dr: u8, sr1: u8, sr2: u8 },
    AndImm { dr: u8, sr1: u8, imm5: i16 },
    Not { dr: u8, sr: u8 },

    Ld { dr: u8, label: String },
    Ldi { dr: u8, label: String },
    Ldr { dr: u8, base_r: u8, offset6: i16 },
    Lea { dr: u8, label: String },
    St { sr: u8, label: String },
    Sti { sr: u8, label: String },
    Str { sr: u8, base_r: u8, offset6: i16 },

    Br { flags: BrFlags, label: String },
    Jmp { base_r: u8 },
    Jsr { label: String },
    Jsrr { base_r: u8 },
    Ret,
    Rti,

    Trap { trapvect8: u8 },
    Getc,
    Out,
    Puts,
    In,
    Putsp,
    Halt,
}
