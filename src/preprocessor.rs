//! # Preprocessor
//!
//! Handles source-level transformations before lexing:
//!
//! - **`.INCLUDE "file"`** — Recursively inserts the contents of `file` at
//!   the point of the directive, replacing the `.INCLUDE` line itself.
//!   Cycle detection prevents infinite recursion.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use lc3_assembler::preprocessor::preprocess;
//!
//! let result = preprocess("program.asm", None);
//! if result.has_errors() {
//!     for e in &result.errors { eprintln!("{e}"); }
//! } else {
//!     println!("{}", result.source);  // fully expanded source
//! }
//! ```
//!
//! ## `.INCLUDE` syntax
//!
//! The directive is case-insensitive and the path must be a double-quoted
//! string literal on the same line:
//!
//! ```text
//! .INCLUDE "macros.asm"
//! .include "defs/constants.asm"
//! ```
//!
//! Relative paths are resolved relative to the **directory of the file that
//! contains the `.INCLUDE`** directive, mirroring C preprocessor behaviour.
//!
//! ## Line-number mapping
//!
//! After expansion every source line carries a `#line N "file"` marker
//! comment on the same row.  The lexer/parser strip these; the diagnostic
//! formatter uses them so errors always point back to the *original* file and
//! line number rather than a position in the flattened output.
//!
//! > **Note**: line markers are not yet consumed by the lexer.  For now the
//! > fully-expanded source is the single string fed to `tokenize()`; the
//! > file/line annotations appear only in `IncludedLine` metadata.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Public types ──────────────────────────────────────────────────────────────

/// A single line in the fully-expanded source, annotated with its origin.
#[derive(Debug, Clone)]
pub struct IncludedLine {
    /// The text of the line (without the trailing newline).
    pub text: String,
    /// 1-based line number in `file`.
    pub line: usize,
    /// The file this line came from (absolute path or `"<stdin>"`).
    pub file: String,
}

/// Result of preprocessing a source file.
pub struct PreprocessResult {
    /// Fully-expanded source text ready to feed to `tokenize()`.
    pub source: String,
    /// Per-line origin data (same length as `source.lines()`).
    pub lines: Vec<IncludedLine>,
    /// Non-fatal warnings (e.g. included file not found when permissive mode
    /// is used — currently all errors are hard errors so this is always empty).
    pub warnings: Vec<String>,
    /// Errors encountered during preprocessing.
    pub errors: Vec<PreprocessError>,
}

impl PreprocessResult {
    /// Returns `true` if any preprocessing errors were recorded.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// An error that occurred while expanding `.INCLUDE` directives.
#[derive(Debug, Clone)]
pub struct PreprocessError {
    pub message: String,
    /// File that contained the offending `.INCLUDE` directive.
    pub file: String,
    /// 1-based line number of the `.INCLUDE` directive.
    pub line: usize,
}

impl std::fmt::Display for PreprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error in {}:{}: {}", self.file, self.line, self.message)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Preprocess an LC-3 assembly source file.
///
/// * `path` — path to the root source file, or `"-"` / `"<stdin>"` if the
///   source was supplied via `raw_source`.
/// * `raw_source` — if `Some`, use this string as the source for `path`
///   instead of reading from disk (used when `path == "-"`).
#[must_use]
pub fn preprocess(path: &str, raw_source: Option<&str>) -> PreprocessResult {
    let mut result_lines: Vec<IncludedLine> = Vec::new();
    let mut errors: Vec<PreprocessError> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Canonicalize the root file path so cycle detection works even with `..`.
    let root_path = if path == "-" || path == "<stdin>" {
        "<stdin>".to_string()
    } else {
        std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string())
    };

    expand_file(
        &root_path,
        raw_source,
        &mut result_lines,
        &mut errors,
        &mut seen,
        0,
    );

    // Build the flat source string from the expanded lines
    let mut source = String::new();
    for line in &result_lines {
        source.push_str(&line.text);
        source.push('\n');
    }

    PreprocessResult {
        source,
        lines: result_lines,
        warnings: Vec::new(),
        errors,
    }
}

// ── Recursive expansion ───────────────────────────────────────────────────────

/// Maximum nesting depth for `.INCLUDE` directives.
const MAX_INCLUDE_DEPTH: usize = 64;

fn expand_file(
    path: &str,
    raw_source: Option<&str>,
    out: &mut Vec<IncludedLine>,
    errors: &mut Vec<PreprocessError>,
    seen: &mut HashSet<String>,
    depth: usize,
) {
    if depth > MAX_INCLUDE_DEPTH {
        errors.push(PreprocessError {
            message: format!("`.INCLUDE` nesting too deep (max {MAX_INCLUDE_DEPTH})"),
            file: path.to_string(),
            line: 0,
        });
        return;
    }

    // Cycle / duplicate detection
    if path != "<stdin>" {
        if seen.contains(path) {
            errors.push(PreprocessError {
                message: format!("circular `.INCLUDE`: '{path}' is already being processed"),
                file: path.to_string(),
                line: 0,
            });
            return;
        }
        seen.insert(path.to_string());
    }

    // Read source text
    let source_text: String = if let Some(src) = raw_source {
        src.to_string()
    } else {
        match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(PreprocessError {
                    message: format!("cannot open '{path}': {e}"),
                    file: path.to_string(),
                    line: 0,
                });
                if path != "<stdin>" {
                    seen.remove(path);
                }
                return;
            }
        }
    };

    // Directory of this file (for resolving relative include paths)
    let base_dir: PathBuf = if path == "<stdin>" {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        Path::new(path)
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    };

    for (idx, line_text) in source_text.lines().enumerate() {
        let line_num = idx + 1;

        if let Some(include_path) = parse_include_directive(line_text) {
            // Resolve path relative to the current file's directory
            let resolved = resolve_path(&base_dir, &include_path);
            let canonical = std::fs::canonicalize(&resolved)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| resolved.to_string_lossy().into_owned());

            // Recursively expand — no raw_source (always read from disk)
            expand_file(&canonical, None, out, errors, seen, depth + 1);
        } else {
            out.push(IncludedLine {
                text: line_text.to_string(),
                line: line_num,
                file: path.to_string(),
            });
        }
    }

    if path != "<stdin>" {
        seen.remove(path);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// If `line` is a `.INCLUDE "path"` directive, return the path string.
/// Returns `None` for all other lines.
fn parse_include_directive(line: &str) -> Option<String> {
    // Strip leading whitespace and optional label
    let trimmed = line.trim_start();

    // Skip comments and empty lines quickly
    if trimmed.is_empty() || trimmed.starts_with(';') {
        return None;
    }

    // A label may precede the directive: `LABEL .INCLUDE "file"`
    // We scan past a label token (word ending before whitespace or '.')
    let directive_start = if trimmed.starts_with('.') {
        trimmed
    } else {
        // Skip label token
        let after_label = trimmed.trim_start_matches(|c: char| c.is_alphanumeric() || c == '_');
        after_label.trim_start()
    };

    if !directive_start.to_uppercase().starts_with(".INCLUDE") {
        return None;
    }

    // After `.INCLUDE`, find the quoted path
    let after_kw = directive_start[".INCLUDE".len()..].trim_start();
    if !after_kw.starts_with('"') {
        return None;
    }
    let inner = &after_kw[1..]; // skip opening quote
    let end = inner.find('"')?;
    let path = &inner[..end];
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn resolve_path(base_dir: &Path, include_path: &str) -> PathBuf {
    let p = Path::new(include_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_include_simple() {
        assert_eq!(
            parse_include_directive(".INCLUDE \"defs.asm\""),
            Some("defs.asm".to_string())
        );
    }

    #[test]
    fn parse_include_case_insensitive() {
        assert_eq!(
            parse_include_directive("  .include \"macros/lib.asm\""),
            Some("macros/lib.asm".to_string())
        );
    }

    #[test]
    fn parse_include_with_label() {
        // Labels before .INCLUDE are unusual but syntactically allowed
        assert_eq!(
            parse_include_directive("LABEL .INCLUDE \"file.asm\""),
            Some("file.asm".to_string())
        );
    }

    #[test]
    fn parse_include_not_a_directive() {
        assert_eq!(parse_include_directive("ADD R0, R1, R2"), None);
        assert_eq!(parse_include_directive("; .INCLUDE \"foo.asm\""), None);
        assert_eq!(parse_include_directive(""), None);
    }

    #[test]
    fn parse_include_empty_path_returns_none() {
        assert_eq!(parse_include_directive(".INCLUDE \"\""), None);
    }

    #[test]
    fn preprocess_no_includes() {
        let src = ".ORIG x3000\nHALT\n.END\n";
        let result = preprocess("<stdin>", Some(src));
        assert!(!result.has_errors());
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.source, src);
    }

    #[test]
    fn preprocess_include_file_not_found_is_error() {
        let src = ".INCLUDE \"does_not_exist_xyz.asm\"\n";
        let result = preprocess("<stdin>", Some(src));
        assert!(result.has_errors());
        assert!(result.errors[0].message.contains("does_not_exist_xyz.asm"));
    }
}
