use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpAdd,
    OpAnd,
    OpNot,
    OpBr(BrFlags),
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

    PseudoRet,
    PseudoGetc,
    PseudoOut,
    PseudoPuts,
    PseudoIn,
    PseudoPutsp,
    PseudoHalt,

    DirOrig,
    DirEnd,
    DirFill,
    DirBlkw,
    DirStringz,

    Register(u8),

    NumDecimal(i32),
    NumHex(i32),
    NumBinary(i32),

    StringLiteral(String),

    Label(String),

    Comma,
    Newline,
    Comment(String),

    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrFlags {
    pub n: bool,
    pub z: bool,
    pub p: bool,
}

impl BrFlags {
    pub fn new(n: bool, z: bool, p: bool) -> Self {
        Self { n, z, p }
    }

    pub fn as_u16(&self) -> u16 {
        ((self.n as u16) << 2) | ((self.z as u16) << 1) | (self.p as u16)
    }
}

impl std::fmt::Display for BrFlags {
    // TODO-MED: Replace loop-based flag formatting with more idiomatic string concatenation
    // TODO-LOW: Use single write! macro instead of multiple writes
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.n {
            write!(f, "n")?;
        }
        if self.z {
            write!(f, "z")?;
        }
        if self.p {
            write!(f, "p")?;
        }
        Ok(())
    }
}

impl TokenKind {
    pub fn is_instruction_or_directive(&self) -> bool {
        // TODO-MED: Replace 25-item matches!() with a more maintainable approach (static set or trait)
        matches!(
            self,
            TokenKind::OpAdd
                | TokenKind::OpAnd
                | TokenKind::OpNot
                | TokenKind::OpBr(_)
                | TokenKind::OpJmp
                | TokenKind::OpJsr
                | TokenKind::OpJsrr
                | TokenKind::OpLd
                | TokenKind::OpLdi
                | TokenKind::OpLdr
                | TokenKind::OpLea
                | TokenKind::OpSt
                | TokenKind::OpSti
                | TokenKind::OpStr
                | TokenKind::OpTrap
                | TokenKind::OpRti
                | TokenKind::PseudoRet
                | TokenKind::PseudoGetc
                | TokenKind::PseudoOut
                | TokenKind::PseudoPuts
                | TokenKind::PseudoIn
                | TokenKind::PseudoPutsp
                | TokenKind::PseudoHalt
                | TokenKind::DirOrig
                | TokenKind::DirEnd
                | TokenKind::DirFill
                | TokenKind::DirBlkw
                | TokenKind::DirStringz
        )
    }
}
