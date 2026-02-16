//! # Cursor
//!
//! Provides character-by-character navigation through source code with position tracking.
//!
//! The cursor maintains both character position and byte offset to support proper
//! Unicode handling and accurate error reporting.

use crate::error::Span;

/// A cursor for iterating through source code characters
///
/// Tracks position in multiple ways:
/// - Character index (for Unicode correctness)
/// - Byte offset (for span creation)
/// - Line and column numbers (for error messages)
pub struct Cursor {
    /// All characters in the source
    chars: Vec<char>,
    /// Current character index
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    col: usize,
    /// Current byte offset in original string
    byte_offset: usize,
}

impl Cursor {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            byte_offset: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    /// Advance to the next character and return it
    ///
    /// Updates line/column tracking:
    /// - '\n' increments line, resets column to 1
    /// - Other chars increment column
    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.get(self.pos).copied() {
            self.pos += 1;
            self.byte_offset += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    pub fn current_pos(&self) -> (usize, usize, usize) {
        (self.byte_offset, self.line, self.col)
    }

    pub fn make_span(&self, start_byte: usize, start_line: usize, start_col: usize) -> Span {
        Span {
            start: start_byte,
            end: self.byte_offset,
            line: start_line,
            col: start_col,
        }
    }
}
