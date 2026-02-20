//! # LC-3 Lexer
//!
//! Tokenizes LC-3 assembly source code into a stream of tokens.
//!
//! ## Features
//!
//! - **Numeric Literals**: Supports decimal (#10, #-5), hexadecimal (x3000, xFFFF),
//!   and binary (b1010, b1111) notation
//! - **String Literals**: Handles escape sequences (\n, \r, \t, \\, \", \0)
//! - **Comments**: Line comments starting with semicolon
//! - **Instructions**: All LC-3 opcodes and pseudo-ops
//! - **Directives**: .ORIG, .FILL, .BLKW, .STRINGZ, .END
//! - **Branch Variants**: Dynamic parsing of BR, BRn, BRz, BRp, BRnz, BRnp, etc.
//!
//! ## Two's Complement Handling
//!
//! Hexadecimal and binary literals are interpreted as 16-bit values and converted
//! to signed integers using two's complement:
//! - `x0000` to `x7FFF` → 0 to 32767 (positive)
//! - `x8000` to `xFFFF` → -32768 to -1 (negative)

pub mod cursor;
pub mod token;

#[cfg(test)]
mod tests;

use crate::error::{AsmError, ErrorKind, Span};
use cursor::Cursor;
use token::{BrFlags, Token, TokenKind};

pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<AsmError>,
}

/// Convert a 16-bit unsigned value to i32 using two's complement representation
#[inline]
fn u16_to_twos_complement(v: u32) -> i32 {
    if v > 0x7FFF {
        (v as i32) - 0x10000
    } else {
        v as i32
    }
}

/// Process an escape sequence character and return the actual character
#[inline]
fn process_escape_char(esc: char) -> Option<char> {
    match esc {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '\\' => Some('\\'),
        '"' => Some('"'),
        '0' => Some('\0'),
        _ => None,
    }
}

#[must_use]
pub fn tokenize(source: &str) -> LexResult {
    // TODO-MED: Consider builder pattern for Token creation to avoid manual Span construction
    let mut cursor = Cursor::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    while !cursor.is_at_end() {
        match lex_token(&mut cursor) {
            Ok(Some(token)) => tokens.push(token),
            Ok(None) => {}
            Err(err) => errors.push(err),
        }
    }

    let (b, l, c) = cursor.current_pos();
    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: String::new(),
        span: Span {
            start: b,
            end: b,
            line: l,
            col: c,
        },
    });

    LexResult { tokens, errors }
}

fn lex_token(cursor: &mut Cursor) -> Result<Option<Token>, AsmError> {
    // Skip whitespace (inlined)
    while matches!(cursor.peek(), Some(' ' | '\t')) {
        cursor.advance();
    }

    if cursor.is_at_end() {
        return Ok(None);
    }

    let (sb, sl, sc) = cursor.current_pos();
    let ch = cursor.peek().unwrap();

    match ch {
        '\n' | '\r' => lex_newline(cursor, sb, sl, sc),
        ';' => lex_comment(cursor, sb, sl, sc),
        ',' => {
            cursor.advance();
            Ok(Some(Token {
                kind: TokenKind::Comma,
                lexeme: ",".into(),
                span: cursor.make_span(sb, sl, sc),
            }))
        }
        '"' => lex_string(cursor, sb, sl, sc),
        '#' => lex_decimal(cursor, sb, sl, sc),
        '.' => lex_directive(cursor, sb, sl, sc),
        c if c.is_ascii_alphabetic() || c == '_' => lex_word(cursor, sb, sl, sc),
        _ => {
            cursor.advance();
            Err(AsmError {
                kind: ErrorKind::UnexpectedCharacter,
                message: format!("Unexpected character: '{}'", ch),
                span: cursor.make_span(sb, sl, sc),
            })
        }
    }
}

fn lex_newline(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    if cursor.peek() == Some('\r') {
        cursor.advance();
        if cursor.peek() == Some('\n') {
            cursor.advance();
        }
    } else {
        cursor.advance();
    }

    Ok(Some(Token {
        kind: TokenKind::Newline,
        lexeme: "\n".into(),
        span: cursor.make_span(sb, sl, sc),
    }))
}

fn lex_comment(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    cursor.advance();
    let mut text = String::new();
    while let Some(ch) = cursor.peek() {
        if ch == '\n' || ch == '\r' {
            break;
        }
        cursor.advance();
        text.push(ch);
    }

    Ok(Some(Token {
        kind: TokenKind::Comment(text.clone()),
        lexeme: format!(";{}", text),
        span: cursor.make_span(sb, sl, sc),
    }))
}

fn lex_string(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    cursor.advance();
    let mut processed = String::new();
    let mut raw = String::from("\"");

    loop {
        if cursor.is_at_end() {
            return Err(AsmError {
                kind: ErrorKind::UnterminatedString,
                message: "Unterminated string literal".into(),
                span: cursor.make_span(sb, sl, sc),
            });
        }

        let ch = cursor.peek().unwrap();
        if ch == '\n' || ch == '\r' {
            return Err(AsmError {
                kind: ErrorKind::UnterminatedString,
                message: "Unterminated string literal".into(),
                span: cursor.make_span(sb, sl, sc),
            });
        }

        if ch == '"' {
            cursor.advance();
            raw.push('"');
            break;
        }

        if ch == '\\' {
            cursor.advance();
            raw.push('\\');
            if cursor.is_at_end() {
                return Err(AsmError {
                    kind: ErrorKind::UnterminatedString,
                    message: "Unterminated string literal".into(),
                    span: cursor.make_span(sb, sl, sc),
                });
            }

            let esc = cursor.peek().unwrap();
            match process_escape_char(esc) {
                Some(ch) => {
                    processed.push(ch);
                    cursor.advance();
                    raw.push(esc);
                }
                None => {
                    return Err(AsmError {
                        kind: ErrorKind::InvalidEscapeSequence,
                        message: format!("Invalid escape sequence: \\{}", esc),
                        span: cursor.make_span(sb, sl, sc),
                    });
                }
            }
        } else {
            let c = cursor.advance().unwrap();
            processed.push(c);
            raw.push(c);
        }
    }

    Ok(Some(Token {
        kind: TokenKind::StringLiteral(processed),
        lexeme: raw,
        span: cursor.make_span(sb, sl, sc),
    }))
}

fn lex_decimal(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    cursor.advance();
    let mut raw = String::from("#");
    let mut sign = String::new();

    if matches!(cursor.peek(), Some('-' | '+')) {
        let ch = cursor.advance().unwrap();
        raw.push(ch);
        sign.push(ch);
    }

    let mut digits = String::new();
    while matches!(cursor.peek(), Some(c) if c.is_ascii_digit()) {
        let ch = cursor.advance().unwrap();
        digits.push(ch);
        raw.push(ch);
    }

    if digits.is_empty() {
        return Err(AsmError {
            kind: ErrorKind::InvalidDecimalLiteral,
            message: "Expected digits after #".into(),
            span: cursor.make_span(sb, sl, sc),
        });
    }

    let value_str = format!("{}{}", sign, digits);
    let value = value_str.parse::<i32>().map_err(|_| AsmError {
        kind: ErrorKind::InvalidDecimalLiteral,
        message: format!("Invalid decimal literal: {}", raw),
        span: cursor.make_span(sb, sl, sc),
    })?;

    Ok(Some(Token {
        kind: TokenKind::NumDecimal(value),
        lexeme: raw,
        span: cursor.make_span(sb, sl, sc),
    }))
}

fn lex_directive(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    cursor.advance();
    let mut raw = String::from(".");
    let mut word = String::new();

    while matches!(cursor.peek(), Some(c) if c.is_ascii_alphabetic()) {
        let ch = cursor.advance().unwrap();
        word.push(ch);
        raw.push(ch);
    }

    let upper = word.to_ascii_uppercase();
    let kind = match upper.as_str() {
        "ORIG" => TokenKind::DirOrig,
        "END" => TokenKind::DirEnd,
        "FILL" => TokenKind::DirFill,
        "BLKW" => TokenKind::DirBlkw,
        "STRINGZ" => TokenKind::DirStringz,
        _ => {
            return Err(AsmError {
                kind: ErrorKind::UnknownDirective,
                message: format!("Unknown directive .{}", upper),
                span: cursor.make_span(sb, sl, sc),
            })
        }
    };

    Ok(Some(Token {
        kind,
        lexeme: raw,
        span: cursor.make_span(sb, sl, sc),
    }))
}

fn lex_word(
    cursor: &mut Cursor,
    sb: usize,
    sl: usize,
    sc: usize,
) -> Result<Option<Token>, AsmError> {
    let mut word = String::new();
    while matches!(cursor.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
        let ch = cursor.advance().unwrap();
        word.push(ch);
    }

    let upper = word.to_ascii_uppercase();

    if upper.len() == 2 && upper.starts_with('R') {
        let digit = upper.chars().nth(1).unwrap();
        if digit.is_ascii_digit() {
            let reg = digit.to_digit(10).unwrap() as u8;
            if reg <= 7 {
                return Ok(Some(Token {
                    kind: TokenKind::Register(reg),
                    lexeme: word,
                    span: cursor.make_span(sb, sl, sc),
                }));
            }
            if reg == 8 || reg == 9 {
                return Err(AsmError {
                    kind: ErrorKind::InvalidRegister,
                    message: format!("Invalid register R{} (must be R0-R7)", reg),
                    span: cursor.make_span(sb, sl, sc),
                });
            }
        }
    }

    // TODO-LOW: Consider using static HashMap or phf_map for opcode lookup instead of match
    let kind = match upper.as_str() {
        "ADD" => TokenKind::OpAdd,
        "AND" => TokenKind::OpAnd,
        "NOT" => TokenKind::OpNot,
        "LD" => TokenKind::OpLd,
        "LDI" => TokenKind::OpLdi,
        "LDR" => TokenKind::OpLdr,
        "LEA" => TokenKind::OpLea,
        "ST" => TokenKind::OpSt,
        "STI" => TokenKind::OpSti,
        "STR" => TokenKind::OpStr,
        "JMP" => TokenKind::OpJmp,
        "JSR" => TokenKind::OpJsr,
        "JSRR" => TokenKind::OpJsrr,
        "TRAP" => TokenKind::OpTrap,
        "RTI" => TokenKind::OpRti,
        "RET" => TokenKind::PseudoRet,
        "GETC" => TokenKind::PseudoGetc,
        "OUT" => TokenKind::PseudoOut,
        "PUTS" => TokenKind::PseudoPuts,
        "IN" => TokenKind::PseudoIn,
        "PUTSP" => TokenKind::PseudoPutsp,
        "HALT" => TokenKind::PseudoHalt,
        _ => {
            // Try to parse as BR instruction with flags
            if let Some(flags) = BrFlags::parse(&upper) {
                return Ok(Some(Token {
                    kind: TokenKind::OpBr(flags),
                    lexeme: word,
                    span: cursor.make_span(sb, sl, sc),
                }));
            }
            // HEX LITERAL: Parse as u32 first, then handle 16-bit two's complement
            if upper.starts_with('X')
                && upper.len() > 1
                && upper[1..].chars().all(|c| c.is_ascii_hexdigit())
            {
                let hex_part = &upper[1..];
                match u32::from_str_radix(hex_part, 16) {
                    Ok(v) if v <= 0xFFFF => {
                        let value = u16_to_twos_complement(v);
                        return Ok(Some(Token {
                            kind: TokenKind::NumHex(value),
                            lexeme: word,
                            span: cursor.make_span(sb, sl, sc),
                        }));
                    }
                    Ok(_) => {
                        return Err(AsmError {
                            kind: ErrorKind::InvalidHexLiteral,
                            message: format!("Hex literal {} exceeds 16 bits", word),
                            span: cursor.make_span(sb, sl, sc),
                        });
                    }
                    Err(_) => {
                        return Err(AsmError {
                            kind: ErrorKind::InvalidHexLiteral,
                            message: format!("Invalid hex literal: {}", word),
                            span: cursor.make_span(sb, sl, sc),
                        });
                    }
                }
            }

            // BINARY LITERAL: Same treatment
            if upper.starts_with('B')
                && upper.len() > 1
                && upper[1..].chars().all(|c| c == '0' || c == '1')
            {
                let bin_part = &upper[1..];
                match u32::from_str_radix(bin_part, 2) {
                    Ok(v) if v <= 0xFFFF => {
                        let value = u16_to_twos_complement(v);
                        return Ok(Some(Token {
                            kind: TokenKind::NumBinary(value),
                            lexeme: word,
                            span: cursor.make_span(sb, sl, sc),
                        }));
                    }
                    Ok(_) => {
                        return Err(AsmError {
                            kind: ErrorKind::InvalidBinaryLiteral,
                            message: format!("Binary literal {} exceeds 16 bits", word),
                            span: cursor.make_span(sb, sl, sc),
                        });
                    }
                    Err(_) => {
                        return Err(AsmError {
                            kind: ErrorKind::InvalidBinaryLiteral,
                            message: format!("Invalid binary literal: {}", word),
                            span: cursor.make_span(sb, sl, sc),
                        });
                    }
                }
            }

            TokenKind::Label(upper.clone())
        }
    };

    Ok(Some(Token {
        kind,
        lexeme: word,
        span: cursor.make_span(sb, sl, sc),
    }))
}
