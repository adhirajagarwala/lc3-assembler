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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags = format!(
            "{}{}{}",
            if self.n { "n" } else { "" },
            if self.z { "z" } else { "" },
            if self.p { "p" } else { "" }
        );
        write!(f, "{}", flags)
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
