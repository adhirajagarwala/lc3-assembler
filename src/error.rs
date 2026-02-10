#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsmError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

// TODO-MED: Add impl for AsmError with builder/constructor methods to reduce boilerplate throughout codebase

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnterminatedString,
    InvalidEscapeSequence,
    InvalidDecimalLiteral,
    InvalidHexLiteral,
    InvalidBinaryLiteral,
    InvalidRegister,
    UnknownDirective,
    UnexpectedCharacter,

    ExpectedOperand,
    ExpectedRegister,
    ExpectedComma,
    UnexpectedToken,
    TooManyOperands,
    TooFewOperands,
    InvalidOperandType,

    DuplicateLabel,
    MissingOrig,
    MultipleOrig,
    OrigNotFirst,
    MissingEnd,
    InvalidOrigAddress,
    InvalidBlkwCount,
    AddressOverflow,
    LabelIsReservedWord,
}

impl std::fmt::Display for AsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ERROR (line {}:{}): {}",
            self.span.line, self.span.col, self.message
        )
    }
}
