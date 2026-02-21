//! # Cursor
//!
//! Provides byte-by-byte navigation through LC-3 assembly source code with
//! position tracking.
//!
//! LC-3 assembly is strictly ASCII, so the cursor operates on a byte slice
//! (`&[u8]`) rather than `Vec<char>`. This eliminates the per-source
//! allocation that `.chars().collect()` would require. `pos` serves as both
//! the byte offset and the character index — they are identical for ASCII input.

use crate::error::Span;

/// A cursor for iterating through source code bytes
///
/// Tracks position in multiple ways:
/// - Byte position (doubles as character index for ASCII)
/// - Line and column numbers (for error messages)
pub struct Cursor<'a> {
    /// Source bytes (ASCII-only)
    bytes: &'a [u8],
    /// Current byte position (also the byte offset for spans)
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    col: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.bytes.get(self.pos).map(|&b| b as char)
    }

    /// Advance to the next byte and return it as a `char`.
    ///
    /// Updates line/column tracking:
    /// - '\n' increments line, resets column to 1
    /// - Other chars increment column
    pub fn advance(&mut self) -> Option<char> {
        if let Some(&b) = self.bytes.get(self.pos) {
            self.pos += 1;
            if b == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(b as char)
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Returns `(line, col)` of the current cursor position.
    pub fn current_pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    /// Build a `Span` anchored at the given start position.
    pub fn make_span(&self, start_line: usize, start_col: usize) -> Span {
        Span { line: start_line, col: start_col }
    }
}
