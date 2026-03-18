//! # Diagnostic Formatter
//!
//! Renders assembler errors and warnings in Rust-compiler-style output:
//! bold labels, source-line context with column-pointer carets, and
//! optional ANSI colours (auto-detected from the terminal type).
//!
//! ## Example output
//!
//! ```text
//! error: TRAP vector 0x200 is out of range (must be 0x00–0xFF)
//!  --> program.asm:5:6
//!   |
//! 5 | TRAP x200
//!   |      ^
//!
//! warning: label 'LOOP' is defined but never referenced
//!  --> program.asm:3:1
//!   |
//! 3 | LOOP  ADD R1, R1, #-1
//!   | ^
//! ```

use crate::error::AsmError;
use crate::warning::AsmWarning;
use std::io::IsTerminal as _;

// ── ANSI escape codes ────────────────────────────────────────────────────────

const BOLD:   &str = "\x1b[1m";
const DIM:    &str = "\x1b[2m";
const RED:    &str = "\x1b[31;1m";    // bold red
const YELLOW: &str = "\x1b[33;1m";   // bold yellow
const CYAN:   &str = "\x1b[36m";
const RESET:  &str = "\x1b[0m";

// ── Public types ─────────────────────────────────────────────────────────────

/// Formats and prints diagnostics with source-line context.
///
/// Construct once per assembler invocation, then call `emit_*` for each
/// error or warning collected from the pipeline stages.
pub struct Diagnostics<'src> {
    lines:    Vec<&'src str>,
    filename: String,
    color:    bool,
}

impl<'src> Diagnostics<'src> {
    /// Build a formatter from the full source text and a display filename.
    ///
    /// Colour defaults to `stderr.is_terminal()`.
    #[must_use]
    pub fn new(source: &'src str, filename: impl Into<String>) -> Self {
        Self {
            lines:    source.lines().collect(),
            filename: filename.into(),
            color:    std::io::stderr().is_terminal(),
        }
    }

    /// Override automatic terminal-detection for colour output.
    #[must_use]
    pub fn with_color(mut self, on: bool) -> Self {
        self.color = on;
        self
    }

    // ── Errors ───────────────────────────────────────────────────────────────

    /// Print a single error.
    pub fn emit_error(&self, err: &AsmError) {
        self.emit(Level::Error, &err.message, err.span.line, err.span.col);
    }

    /// Print a slice of errors then a final summary.
    pub fn emit_all_errors(&self, errors: &[&AsmError]) {
        for e in errors {
            self.emit_error(e);
        }
        if !errors.is_empty() {
            self.emit_error_summary(errors.len());
        }
    }

    /// Print the "aborting due to N previous error(s)" banner.
    ///
    /// Factored out so `RichDiagnostics` can reuse it without duplicating the
    /// format string.
    pub(crate) fn emit_error_summary(&self, n: usize) {
        eprintln!(
            "{red}error{reset}{bold}: aborting due to {n} previous error{s}{reset}",
            red   = self.c(RED),
            bold  = self.c(BOLD),
            reset = self.c(RESET),
            s     = if n == 1 { "" } else { "s" },
        );
    }

    // ── Warnings ─────────────────────────────────────────────────────────────

    /// Print a single warning.
    pub fn emit_warning(&self, warn: &AsmWarning) {
        self.emit(Level::Warning, &warn.message, warn.span.line, warn.span.col);
    }

    /// Print all warnings, then a summary count.
    pub fn emit_all_warnings(&self, warnings: &[AsmWarning]) {
        for w in warnings {
            self.emit_warning(w);
        }
        if !warnings.is_empty() {
            let n = warnings.len();
            eprintln!(
                "{yellow}warning{reset}{bold}: {n} warning{s} emitted{reset}",
                yellow = self.c(YELLOW),
                bold   = self.c(BOLD),
                reset  = self.c(RESET),
                s      = if n == 1 { "" } else { "s" },
            );
            eprintln!();
        }
    }

    // ── Private ──────────────────────────────────────────────────────────────

    /// Return the ANSI code if colour is enabled, empty string otherwise.
    #[inline]
    fn c(&self, code: &'static str) -> &'static str {
        if self.color { code } else { "" }
    }

    fn emit(&self, level: Level, message: &str, line: usize, col: usize) {
        let (label, color) = match level {
            Level::Error   => ("error",   self.c(RED)),
            Level::Warning => ("warning", self.c(YELLOW)),
        };

        // ── header line: error: message ──────────────────────────────────
        eprintln!(
            "{color}{label}{reset}{bold}: {message}{reset}",
            color  = color,
            label  = label,
            bold   = self.c(BOLD),
            reset  = self.c(RESET),
        );

        // ──  --> file:line:col ────────────────────────────────────────────
        eprintln!(
            " {dim}-->{reset} {file}:{line}:{col}",
            dim   = self.c(DIM),
            reset = self.c(RESET),
            file  = self.filename,
        );

        // ── source excerpt ────────────────────────────────────────────────
        if line == 0 {
            eprintln!();
            return;
        }

        let gutter_w = digits(line);
        let pad      = " ".repeat(gutter_w);

        // blank gutter line before source
        eprintln!(
            " {dim}{pad} |{reset}",
            dim   = self.c(DIM),
            reset = self.c(RESET),
        );

        // the source line itself (lines are 1-indexed)
        if let Some(&src) = self.lines.get(line - 1) {
            eprintln!(
                " {dim}{line:>gutter_w$} |{reset} {src}",
                dim   = self.c(DIM),
                reset = self.c(RESET),
            );

            // caret row: indent by (col - 1) spaces, then ^
            let indent = col.saturating_sub(1);
            eprintln!(
                " {dim}{pad} |{reset} {spaces}{color}{bold}^{reset}",
                dim    = self.c(DIM),
                reset  = self.c(RESET),
                color  = color,
                bold   = self.c(BOLD),
                spaces = " ".repeat(indent),
            );
        }

        eprintln!();
    }
}

enum Level { Error, Warning }

/// Number of decimal digits in `n` (for gutter width).
fn digits(n: usize) -> usize {
    if n == 0 { return 1; }
    let mut d = 0;
    let mut v = n;
    while v > 0 { d += 1; v /= 10; }
    d
}

// ── Suggestion helpers ────────────────────────────────────────────────────────
// These turn common ErrorKind variants into human-readable hints.

use crate::error::ErrorKind;

/// Return a short suggestion string for a known error kind, if one exists.
pub fn suggestion_for(kind: &ErrorKind) -> Option<&'static str> {
    match kind {
        ErrorKind::TooFewOperands =>
            Some("hint: check the instruction's required operand count"),
        ErrorKind::TooManyOperands =>
            Some("hint: remove the extra operand"),
        ErrorKind::ExpectedComma =>
            Some("hint: LC-3 operands are separated by commas (e.g. ADD R0, R1, R2)"),
        ErrorKind::ExpectedRegister =>
            Some("hint: registers are R0–R7 (e.g. R0, R3)"),
        ErrorKind::InvalidHexLiteral =>
            Some("hint: hex literals use the 'x' prefix without '0x' (e.g. x3000)"),
        ErrorKind::InvalidDecimalLiteral =>
            Some("hint: decimal literals require a '#' prefix (e.g. #10 or #-5)"),
        ErrorKind::InvalidBinaryLiteral =>
            Some("hint: binary literals use the 'b' prefix (e.g. b1010)"),
        ErrorKind::InvalidOctalLiteral =>
            Some("hint: octal literals use the '0o' prefix (e.g. 0o777)"),
        ErrorKind::UndefinedLabel =>
            Some("hint: check spelling — labels are case-insensitive"),
        ErrorKind::MissingOrig =>
            Some("hint: every LC-3 program must begin with .ORIG (e.g. .ORIG x3000)"),
        ErrorKind::MissingEnd =>
            Some("hint: every LC-3 program must end with .END"),
        ErrorKind::OffsetOutOfRange =>
            Some("hint: move the label closer, or use indirect addressing (LDI/STI)"),
        ErrorKind::NonAsciiInStringz =>
            Some("hint: LC-3 strings are ASCII only (code points 0x00–0x7F)"),
        ErrorKind::LabelIsReservedWord =>
            Some("hint: rename the label or use a dotted form (e.g. .FILL instead of FILL as a label)"),
        _ => None,
    }
}

/// Enhanced error emitter that also prints a suggestion when one is available.
pub struct RichDiagnostics<'src>(pub Diagnostics<'src>);

impl<'src> RichDiagnostics<'src> {
    #[must_use]
    pub fn new(source: &'src str, filename: impl Into<String>) -> Self {
        Self(Diagnostics::new(source, filename))
    }

    #[must_use]
    pub fn with_color(mut self, on: bool) -> Self {
        self.0 = self.0.with_color(on);
        self
    }

    pub fn emit_error(&self, err: &AsmError) {
        self.0.emit_error(err);
        if let Some(hint) = suggestion_for(&err.kind) {
            eprintln!(
                "  {cyan}={reset} {hint}",
                cyan  = self.0.c(CYAN),
                reset = self.0.c(RESET),
            );
            eprintln!();
        }
    }

    pub fn emit_all_errors(&self, errors: &[&AsmError]) {
        for e in errors {
            self.emit_error(e);
        }
        if !errors.is_empty() {
            self.0.emit_error_summary(errors.len());
        }
    }

    pub fn emit_warning(&self, warn: &AsmWarning) {
        self.0.emit_warning(warn);
    }

    pub fn emit_all_warnings(&self, warnings: &[AsmWarning]) {
        self.0.emit_all_warnings(warnings);
    }
}
