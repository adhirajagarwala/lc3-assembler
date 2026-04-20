//! # Macro Expander
//!
//! Implements a simple text-substitution macro system for LC-3 assembly.
//!
//! ## Syntax
//!
//! ```text
//! .MACRO  PUSH_R0
//!     STR  R0, R6, #0
//!     ADD  R6, R6, #-1
//! .ENDM
//!
//!     PUSH_R0          ; expands to the two STR/ADD lines above
//! ```
//!
//! ### Parameterised macros
//!
//! ```text
//! .MACRO  PUSH  %REG
//!     STR  %REG, R6, #0
//!     ADD  R6, R6, #-1
//! .ENDM
//!
//!     PUSH  R0         ; %REG → R0
//!     PUSH  R1         ; %REG → R1
//! ```
//!
//! - Parameter names start with `%` and are replaced textually.
//! - Multiple parameters are comma-separated in the `.MACRO` header.
//! - Macro names are case-insensitive; parameter names are case-sensitive.
//!
//! ## Processing order
//!
//! Macro expansion runs **after** `.INCLUDE` preprocessing and **before**
//! lexing.  The output is a flat source string handed directly to `tokenize()`.
//!
//! ## Restrictions
//!
//! - Macros may not be defined inside other macros.
//! - Recursive macro invocations are detected and produce an error.
//! - Macro names must not shadow LC-3 instruction mnemonics or directives
//!   (the assembler will catch any such errors in the subsequent parse stage).

use std::collections::HashMap;

// ── Public types ──────────────────────────────────────────────────────────────

/// A defined macro: its parameter list and body lines.
#[derive(Debug, Clone)]
pub struct MacroDef {
    pub name: String,
    /// Parameter names in declaration order, **without** the `%` prefix.
    pub params: Vec<String>,
    /// Body lines exactly as written (with `%PARAM` placeholders intact).
    pub body: Vec<String>,
}

/// An error encountered during macro expansion.
#[derive(Debug, Clone)]
pub struct MacroError {
    pub message: String,
    /// 1-based source line that triggered the error.
    pub line: usize,
}

impl std::fmt::Display for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "macro error (line {}): {}", self.line, self.message)
    }
}

/// Result of macro expansion.
pub struct MacroResult {
    /// Fully-expanded source text.
    pub source: String,
    /// Errors encountered during expansion.
    pub errors: Vec<MacroError>,
}

impl MacroResult {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Expand all macro definitions and invocations in `source`.
///
/// `source` should be the output of the `.INCLUDE` preprocessor.
#[must_use]
pub fn expand(source: &str) -> MacroResult {
    let mut macros: HashMap<String, MacroDef> = HashMap::new();
    let mut errors: Vec<MacroError> = Vec::new();
    let mut output: Vec<String> = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_num = i + 1;

        // ── .MACRO definition ────────────────────────────────────────────────
        if let Some(def) = parse_macro_header(line) {
            // Collect body until .ENDM
            let start_line = line_num;
            i += 1;
            let mut body: Vec<String> = Vec::new();
            let mut found_endm = false;
            while i < lines.len() {
                let body_line = lines[i];
                if is_endm(body_line) {
                    found_endm = true;
                    i += 1;
                    break;
                }
                // Nested .MACRO is an error; skip the header line entirely so it
                // doesn't leak into the outer macro's body and confuse later passes.
                if parse_macro_header(body_line).is_some() {
                    errors.push(MacroError {
                        message: "nested .MACRO definitions are not allowed".to_string(),
                        line: i + 1,
                    });
                    i += 1;
                    continue;
                }
                body.push(body_line.to_string());
                i += 1;
            }
            if !found_endm {
                errors.push(MacroError {
                    message: format!(
                        "macro '{}' opened at line {start_line} has no matching .ENDM",
                        def.name
                    ),
                    line: start_line,
                });
            }
            // Register macro (last definition wins)
            let key = def.name.to_uppercase();
            macros.insert(key, MacroDef { body, ..def });
            // .MACRO/.ENDM block itself produces no output
            continue;
        }

        // ── .ENDM without matching .MACRO ────────────────────────────────────
        if is_endm(line) {
            errors.push(MacroError {
                message: ".ENDM without a preceding .MACRO".to_string(),
                line: line_num,
            });
            i += 1;
            continue;
        }

        // ── Possible macro invocation ────────────────────────────────────────
        if let Some((name, call_args)) = parse_macro_call(line) {
            let key = name.to_uppercase();
            if let Some(def) = macros.get(&key).cloned() {
                if def.params.len() != call_args.len() {
                    errors.push(MacroError {
                        message: format!(
                            "macro '{}' expects {} argument{} but got {}",
                            def.name,
                            def.params.len(),
                            if def.params.len() == 1 { "" } else { "s" },
                            call_args.len()
                        ),
                        line: line_num,
                    });
                    // Emit blank lines to preserve line numbering
                    for _ in &def.body {
                        output.push(String::new());
                    }
                } else {
                    // Detect direct recursive self-invocation inside the body.
                    // The expander is single-pass, so recursive calls would just
                    // emit the invocation text verbatim and produce confusing parse
                    // errors downstream. Catch them here with a clear diagnostic.
                    let recursive = def.body.iter().any(|body_line| {
                        parse_macro_call(body_line)
                            .is_some_and(|(call, _)| call.to_uppercase() == key)
                    });
                    if recursive {
                        errors.push(MacroError {
                            message: format!(
                                "macro '{}' recursively invokes itself, which is not allowed",
                                def.name
                            ),
                            line: line_num,
                        });
                        for _ in &def.body {
                            output.push(String::new());
                        }
                    } else {
                        // Substitute parameters and emit body lines
                        for body_line in &def.body {
                            let expanded = substitute_params(body_line, &def.params, &call_args);
                            output.push(expanded);
                        }
                    }
                }
                i += 1;
                continue;
            }
        }

        // ── Normal line — pass through unchanged ─────────────────────────────
        output.push(line.to_string());
        i += 1;
    }

    MacroResult {
        source: output.join("\n") + "\n",
        errors,
    }
}

// ── Parsing helpers ───────────────────────────────────────────────────────────

/// Parse a `.MACRO name [%P1, %P2, ...]` header.
/// Returns `Some(MacroDef)` (with empty body) if this is a macro header.
fn parse_macro_header(line: &str) -> Option<MacroDef> {
    let trimmed = line.trim();
    if !trimmed.to_uppercase().starts_with(".MACRO") {
        return None;
    }
    let rest = trimmed[".MACRO".len()..].trim();
    if rest.is_empty() {
        return None; // .MACRO with no name is not a valid header
    }

    // Split rest into name and optional parameter list
    let mut parts = rest.splitn(2, |c: char| c.is_whitespace() || c == ',');
    let name = parts.next()?.trim().to_string();
    if name.is_empty() {
        return None;
    }

    // Collect comma-separated parameters (strip leading %)
    let params: Vec<String> = if let Some(param_str) = parts.next() {
        param_str
            .split(',')
            .map(|p| p.trim().trim_start_matches('%').to_string())
            .filter(|p| !p.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    Some(MacroDef {
        name,
        params,
        body: Vec::new(),
    })
}

/// Returns `true` if `line` is a `.ENDM` directive.
fn is_endm(line: &str) -> bool {
    let t = line.trim();
    t.eq_ignore_ascii_case(".ENDM")
}

/// Try to parse `line` as a macro invocation.
///
/// Returns `Some((macro_name, args))` when the first token on the line is
/// *not* a directive or comment and matches a pattern we could dispatch.
/// The caller checks whether `macro_name` is actually in the macro table.
fn parse_macro_call(line: &str) -> Option<(String, Vec<String>)> {
    let trimmed = line.trim();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with(';') {
        return None;
    }

    // Skip lines that are clearly directives or labels+directives
    // (they can't be macro invocations)
    if trimmed.starts_with('.') {
        return None;
    }

    // Extract the first token
    let (first_token, rest) = split_first_token(trimmed);

    // Skip if it looks like a label definition (followed by another token
    // that starts with a dot or is an opcode).  We rely on the fact that
    // label-only lines won't match any macro name, and label+instr lines
    // will have the instruction after the label — those are handled as
    // normal lines and won't break anything even if we check them here,
    // because the macro table won't contain standard opcodes.
    //
    // We simply return the first token and let the caller decide.
    let args: Vec<String> = if rest.trim().is_empty() {
        Vec::new()
    } else {
        rest.split(',').map(|a| a.trim().to_string()).collect()
    };

    if first_token.is_empty() {
        None
    } else {
        Some((first_token.to_string(), args))
    }
}

/// Split a line into its first whitespace-delimited token and the remainder.
fn split_first_token(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    let end = s.find(|c: char| c.is_whitespace()).unwrap_or(s.len());
    (&s[..end], s[end..].trim_start())
}

/// Replace `%PARAM` placeholders in `template` with the corresponding `args`.
fn substitute_params(template: &str, params: &[String], args: &[String]) -> String {
    let mut result = template.to_string();
    for (param, arg) in params.iter().zip(args.iter()) {
        // Replace both `%PARAM` and `%param` (case-sensitive match as declared)
        let placeholder = format!("%{param}");
        result = result.replace(&placeholder, arg);
    }
    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn expand_str(s: &str) -> MacroResult {
        expand(s)
    }

    // ── Definition parsing ────────────────────────────────────────────────────

    #[test]
    fn parse_macro_header_simple() {
        let def = parse_macro_header(".MACRO PUSH_R0").unwrap();
        assert_eq!(def.name, "PUSH_R0");
        assert!(def.params.is_empty());
    }

    #[test]
    fn parse_macro_header_with_params() {
        let def = parse_macro_header(".MACRO PUSH %REG").unwrap();
        assert_eq!(def.name, "PUSH");
        assert_eq!(def.params, vec!["REG"]);
    }

    #[test]
    fn parse_macro_header_multi_param() {
        let def = parse_macro_header(".MACRO COPY %SRC, %DST").unwrap();
        assert_eq!(def.params, vec!["SRC", "DST"]);
    }

    #[test]
    fn parse_macro_header_case_insensitive() {
        assert!(parse_macro_header(".macro zero_r0").is_some());
        assert!(parse_macro_header(".MACRO  THING").is_some());
    }

    #[test]
    fn parse_macro_header_not_macro() {
        assert!(parse_macro_header("ADD R0, R1, R2").is_none());
        assert!(parse_macro_header("; .MACRO FOO").is_none());
        assert!(parse_macro_header(".ORIG x3000").is_none());
    }

    // ── No-op (no macros) ─────────────────────────────────────────────────────

    #[test]
    fn expand_no_macros_passthrough() {
        let src = ".ORIG x3000\nHALT\n.END\n";
        let r = expand_str(src);
        assert!(!r.has_errors());
        // Source is unchanged (modulo final newline normalisation)
        assert!(r.source.contains("HALT"));
        assert!(r.source.contains(".ORIG x3000"));
    }

    // ── Parameterless macro ───────────────────────────────────────────────────

    #[test]
    fn expand_no_param_macro() {
        let src = "\
.MACRO HALT_NOW
    HALT
.ENDM
.ORIG x3000
HALT_NOW
.END
";
        let r = expand_str(src);
        assert!(!r.has_errors(), "errors: {:?}", r.errors);
        assert!(
            r.source.contains("HALT"),
            "body should appear: {}",
            r.source
        );
        assert!(
            !r.source.to_uppercase().contains(".MACRO"),
            "definition should be stripped"
        );
        assert!(
            !r.source.to_uppercase().contains(".ENDM"),
            "endm should be stripped"
        );
    }

    // ── Parameterised macro ───────────────────────────────────────────────────

    #[test]
    fn expand_param_macro() {
        let src = "\
.MACRO CLR %R
    AND %R, %R, #0
.ENDM
.ORIG x3000
CLR R0
CLR R1
.END
";
        let r = expand_str(src);
        assert!(!r.has_errors(), "errors: {:?}", r.errors);
        assert!(
            r.source.contains("AND R0, R0, #0"),
            "R0 substitution: {}",
            r.source
        );
        assert!(
            r.source.contains("AND R1, R1, #0"),
            "R1 substitution: {}",
            r.source
        );
    }

    // ── Wrong argument count ──────────────────────────────────────────────────

    #[test]
    fn expand_wrong_arg_count_is_error() {
        let src = "\
.MACRO CLR %R
    AND %R, %R, #0
.ENDM
.ORIG x3000
CLR
.END
";
        let r = expand_str(src);
        assert!(r.has_errors());
        assert!(r.errors[0].message.contains("expects 1 argument"));
    }

    // ── Missing .ENDM ─────────────────────────────────────────────────────────

    #[test]
    fn expand_missing_endm_is_error() {
        let src = "\
.MACRO FOO
    HALT
.ORIG x3000
.END
";
        let r = expand_str(src);
        assert!(r.has_errors());
        assert!(r.errors[0].message.contains("no matching .ENDM"));
    }

    // ── Stray .ENDM ───────────────────────────────────────────────────────────

    #[test]
    fn expand_stray_endm_is_error() {
        let src = ".ORIG x3000\n.ENDM\nHALT\n.END\n";
        let r = expand_str(src);
        assert!(r.has_errors());
        assert!(r.errors[0]
            .message
            .contains(".ENDM without a preceding .MACRO"));
    }

    // ── Recursive self-invocation ─────────────────────────────────────────────

    #[test]
    fn expand_recursive_macro_is_error() {
        let src = "\
.MACRO FOREVER
    FOREVER
.ENDM
.ORIG x3000
FOREVER
.END
";
        let r = expand_str(src);
        assert!(r.has_errors());
        assert!(r.errors[0].message.contains("recursively invokes itself"));
    }

    // ── Nested macro definition ───────────────────────────────────────────────

    #[test]
    fn expand_nested_macro_def_is_error() {
        let src = "\
.MACRO OUTER
.MACRO INNER
    HALT
.ENDM
.ENDM
.ORIG x3000
HALT
.END
";
        let r = expand_str(src);
        // The nested .MACRO header should produce an error
        assert!(r.has_errors());
        assert!(r.errors[0].message.contains("nested .MACRO"));
        // The nested .MACRO line must NOT appear in the expanded output
        assert!(
            !r.source.to_uppercase().contains(".MACRO INNER"),
            "nested .MACRO header leaked into output: {}",
            r.source
        );
    }

    // ── substitute_params ─────────────────────────────────────────────────────

    #[test]
    fn substitute_single_param() {
        let result = substitute_params("ADD %R, %R, #0", &["R".to_string()], &["R5".to_string()]);
        assert_eq!(result, "ADD R5, R5, #0");
    }

    #[test]
    fn substitute_two_params() {
        let result = substitute_params(
            "STR %SRC, %BASE, #0",
            &["SRC".to_string(), "BASE".to_string()],
            &["R1".to_string(), "R6".to_string()],
        );
        assert_eq!(result, "STR R1, R6, #0");
    }
}
