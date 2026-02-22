/// Source location used for error reporting.
///
/// Stores the 1-based line and column of the first character of a token.
/// Byte offsets were removed because they were computed but never read —
/// if source-underline diagnostics are added later, add `start`/`end` back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsmError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

impl AsmError {
    /// Create a new AsmError
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    /// Builder-style constructor for common error patterns
    pub fn too_few_operands(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::TooFewOperands, message, span)
    }

    pub fn too_many_operands(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::TooManyOperands, message, span)
    }

    pub fn invalid_operand_type(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::InvalidOperandType, message, span)
    }

    pub fn expected_register(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedRegister, message, span)
    }

    pub fn expected_comma(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedComma, message, span)
    }

    pub fn expected_operand(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::ExpectedOperand, message, span)
    }

    pub fn unexpected_token(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::UnexpectedToken, message, span)
    }

    pub fn undefined_label(label: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::UndefinedLabel,
            format!("Undefined label '{label}'"),
            span,
        )
    }

    pub fn duplicate_label(label: &str, first_addr: u16, span: Span) -> Self {
        Self::new(
            ErrorKind::DuplicateLabel,
            format!("Duplicate label '{label}' (first defined at x{first_addr:04X})"),
            span,
        )
    }

    pub fn non_ascii_in_stringz(ch: char, span: Span) -> Self {
        Self::new(
            ErrorKind::NonAsciiInStringz,
            format!(
                "Character '{ch}' (U+{:04X}) in .STRINGZ is not ASCII; \
                 LC-3 only supports characters 0x00–0x7F",
                ch as u32
            ),
            span,
        )
    }

    pub fn label_is_reserved_word(label: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::LabelIsReservedWord,
            format!("Label '{label}' shadows an assembler directive name; prefix it with '.' or rename it"),
            span,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnterminatedString,
    InvalidEscapeSequence,
    InvalidDecimalLiteral,
    InvalidHexLiteral,
    InvalidBinaryLiteral,
    InvalidOctalLiteral,
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
    // OrigNotFirst was removed — the first pass handles this via MissingOrig instead,
    // and this variant was never constructed anywhere in the codebase.
    MissingEnd,
    InvalidOrigAddress,
    InvalidBlkwCount,
    AddressOverflow,
    UndefinedLabel,
    OffsetOutOfRange,
    NonAsciiInStringz,   // feature 8.9: non-ASCII char in .STRINGZ
    LabelIsReservedWord, // feature 8.1: label shadows a directive name
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnterminatedString => "unterminated string",
            Self::InvalidEscapeSequence => "invalid escape sequence",
            Self::InvalidDecimalLiteral => "invalid decimal literal",
            Self::InvalidHexLiteral => "invalid hex literal",
            Self::InvalidBinaryLiteral => "invalid binary literal",
            Self::InvalidOctalLiteral => "invalid octal literal",
            Self::InvalidRegister => "invalid register",
            Self::UnknownDirective => "unknown directive",
            Self::UnexpectedCharacter => "unexpected character",
            Self::ExpectedOperand => "expected operand",
            Self::ExpectedRegister => "expected register",
            Self::ExpectedComma => "expected comma",
            Self::UnexpectedToken => "unexpected token",
            Self::TooManyOperands => "too many operands",
            Self::TooFewOperands => "too few operands",
            Self::InvalidOperandType => "invalid operand type",
            Self::DuplicateLabel => "duplicate label",
            Self::MissingOrig => "missing .ORIG directive",
            Self::MultipleOrig => "multiple .ORIG directives",
            Self::MissingEnd => "missing .END directive",
            Self::InvalidOrigAddress => "invalid .ORIG address",
            Self::InvalidBlkwCount => "invalid .BLKW count",
            Self::AddressOverflow => "address overflow",
            Self::UndefinedLabel => "undefined label",
            Self::OffsetOutOfRange => "PC offset out of range",
            Self::NonAsciiInStringz => "non-ASCII character in .STRINGZ",
            Self::LabelIsReservedWord => "label shadows a reserved word",
        };
        f.write_str(s)
    }
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

/// Make AsmError compatible with the standard Rust error-handling ecosystem.
/// This allows it to be used with `?`, `Box<dyn Error>`, `anyhow`, etc.
impl std::error::Error for AsmError {}
