use crate::error::Span;

pub struct Cursor {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
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

    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.get(self.pos).copied() {
            self.pos += 1;
            self.byte_offset += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else if ch == '\r' {
                self.col += 1;
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
