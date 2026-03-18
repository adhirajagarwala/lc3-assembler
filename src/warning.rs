//! # Assembler Warnings
//!
//! Non-fatal diagnostic messages emitted when the assembler detects code that
//! is syntactically valid but is almost certainly a mistake.
//!
//! Warnings do **not** prevent a successful assembly; the `.obj` file is still
//! written.  They are printed to stderr before the success banner.
//!
//! ## Current warnings
//!
//! | Kind | Trigger |
//! |------|---------|
//! | `UnusedLabel` | A label was defined but never referenced by any instruction or directive |
//! | `UnreachableCode` | An instruction or directive follows an unconditional halt or branch |

use crate::error::Span;

// ── Warning kind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum WarnKind {
    /// Label defined but never referenced.
    UnusedLabel,
    /// Instruction follows unconditional HALT / BRnzp (dead code).
    UnreachableCode,
}

impl std::fmt::Display for WarnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnusedLabel     => write!(f, "unused label"),
            Self::UnreachableCode => write!(f, "unreachable code"),
        }
    }
}

// ── Warning value ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AsmWarning {
    pub kind:    WarnKind,
    pub message: String,
    pub span:    Span,
}

impl AsmWarning {
    pub fn new(kind: WarnKind, message: impl Into<String>, span: Span) -> Self {
        Self { kind, message: message.into(), span }
    }

    pub fn unused_label(name: &str, span: Span) -> Self {
        Self::new(
            WarnKind::UnusedLabel,
            format!("label '{name}' is defined but never referenced"),
            span,
        )
    }

    pub fn unreachable_code(span: Span) -> Self {
        Self::new(
            WarnKind::UnreachableCode,
            "this code is unreachable (follows an unconditional HALT or BRnzp)".to_string(),
            span,
        )
    }
}

impl std::fmt::Display for AsmWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WARNING (line {}:{}): {}",
            self.span.line, self.span.col, self.message
        )
    }
}
