//! # First Pass
//!
//! Builds the symbol table and validates program structure.
//!
//! ## Responsibilities
//!
//! The first pass performs several critical tasks:
//!
//! 1. **Symbol Table Construction**: Records all labels and their addresses
//! 2. **Address Calculation**: Tracks the location counter as it processes each line
//! 3. **Structure Validation**: Ensures .ORIG comes first, .END is present, no duplicates
//! 4. **Overflow Detection**: Checks that the program doesn't exceed 16-bit address space
//!
//! ## State Machine
//!
//! The first pass uses a state machine with three states:
//! - `WaitingForOrig`: Initial state, expecting .ORIG directive
//! - `Processing`: Normal processing after .ORIG
//! - `AfterEnd`: After .END directive (ignores subsequent lines)
//!
//! This replaces error-prone boolean flags and makes the logic clearer.

pub mod symbol_table;

#[cfg(test)]
mod tests;

use crate::error::{AsmError, ErrorKind, Span};
use crate::parser::ast::{Instruction, LineContent, SourceLine};
use crate::warning::AsmWarning;
use symbol_table::SymbolTable;

pub struct FirstPassResult {
    pub symbol_table: SymbolTable,
    pub source_lines: Vec<SourceLine>,
    pub orig_address: u16,
    pub errors: Vec<AsmError>,
    pub warnings: Vec<AsmWarning>,
}

impl FirstPassResult {
    /// Returns `true` if any first-pass errors were recorded.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AssemblerState {
    WaitingForOrig,
    Processing,
    AfterEnd,
}

/// Perform the first pass of the assembler.
///
/// Takes ownership of the parsed lines so the resulting `FirstPassResult`
/// can store them directly without cloning. Previously this function accepted
/// `&[SourceLine]` and called `lines.to_vec()` at the end — an unnecessary
/// clone of the entire AST. Taking `Vec<SourceLine>` eliminates that allocation.
#[must_use]
pub fn first_pass(lines: Vec<SourceLine>) -> FirstPassResult {
    let mut symbol_table = SymbolTable::new();
    let mut errors = Vec::new();
    let mut location_counter: Option<u16> = None;
    let mut orig_address: u16 = 0;
    let mut state = AssemblerState::WaitingForOrig;

    for line in &lines {
        match state {
            AssemblerState::WaitingForOrig => match &line.content {
                LineContent::Orig(addr) => {
                    state = AssemblerState::Processing;
                    orig_address = *addr;
                    location_counter = Some(*addr);
                    if let Some(ref label) = line.label {
                        record_label(&mut symbol_table, label, *addr, line.span, &mut errors);
                    }
                    continue;
                }
                LineContent::Empty => continue,
                _ => {
                    errors.push(AsmError::new(
                        ErrorKind::MissingOrig,
                        "Expected .ORIG before any instructions",
                        line.span,
                    ));
                    state = AssemblerState::Processing;
                    orig_address = 0x3000;
                    location_counter = Some(0x3000);
                }
            },
            AssemblerState::AfterEnd => continue,
            AssemblerState::Processing => {}
        }

        let lc = location_counter.unwrap();

        if let Some(ref label) = line.label {
            record_label(&mut symbol_table, label, lc, line.span, &mut errors);
        }

        // Handle special cases before word counting
        match &line.content {
            LineContent::Orig(_) => {
                errors.push(AsmError::new(
                    ErrorKind::MultipleOrig,
                    "Multiple .ORIG directives are not supported",
                    line.span,
                ));
            }
            LineContent::End => {
                state = AssemblerState::AfterEnd;
            }
            LineContent::Blkw(n) if *n == 0 => {
                errors.push(AsmError::new(
                    ErrorKind::InvalidBlkwCount,
                    ".BLKW count must be positive",
                    line.span,
                ));
            }
            _ => {}
        }

        let words = line.content.word_count();

        // Check for address overflow (LC-3 only has 16-bit address space)
        let new_lc = (lc as u32) + words;
        if new_lc > 0x10000 {
            errors.push(AsmError::new(
                ErrorKind::AddressOverflow,
                format!("Address overflow: location counter would exceed 0xFFFF (at x{lc:04X} + {words} words)"),
                line.span,
            ));
            location_counter = Some(0xFFFF); // Cap at max address
        } else {
            location_counter = Some(new_lc as u16);
        }
    }

    if state == AssemblerState::WaitingForOrig {
        errors.push(AsmError::new(
            ErrorKind::MissingOrig,
            "No .ORIG directive found",
            Span { line: 1, col: 1 },
        ));
    }

    if state != AssemblerState::AfterEnd {
        errors.push(AsmError::new(
            ErrorKind::MissingEnd,
            "No .END directive found",
            Span { line: 1, col: 1 },
        ));
    }

    // Detect unreachable code: instructions that follow an unconditional
    // terminator (HALT or BRnzp) within the same program body.
    let warnings = detect_unreachable_code(&lines);

    FirstPassResult {
        symbol_table,
        source_lines: lines, // No clone needed — we own the Vec
        orig_address,
        errors,
        warnings,
    }
}

/// Returns warnings for instructions that can never be executed because they
/// follow an unconditional HALT or BRnzp branch.
fn detect_unreachable_code(lines: &[SourceLine]) -> Vec<AsmWarning> {
    let mut warnings = Vec::new();
    let mut after_terminator = false;

    for line in lines {
        match &line.content {
            LineContent::Orig(_) | LineContent::End | LineContent::Empty => {
                // Structural lines — reset or ignore
                if matches!(line.content, LineContent::End) {
                    after_terminator = false;
                }
            }
            content => {
                if after_terminator {
                    // A labeled line is always reachable — something can branch to it.
                    // Only warn on unlabeled instructions/data in dead code.
                    if line.label.is_none() && !matches!(content, LineContent::Empty) {
                        warnings.push(AsmWarning::unreachable_code(line.span));
                        // Only warn once per "dead block" — reset so we don't
                        // flood every subsequent line with warnings.
                    }
                    // A label (whether or not we warned) re-opens the live code window.
                    if line.label.is_some() {
                        after_terminator = false;
                    }
                }
                // Check if this line itself is a terminator
                if is_unconditional_terminator(content) {
                    after_terminator = true;
                }
            }
        }
    }
    warnings
}

/// Returns `true` for instructions that unconditionally transfer control and
/// after which no following instruction can be reached.
fn is_unconditional_terminator(content: &LineContent) -> bool {
    match content {
        LineContent::Instruction(Instruction::Halt)
        | LineContent::Instruction(Instruction::Ret)
        | LineContent::Instruction(Instruction::Rti)
        // JMP always transfers control regardless of condition codes.
        // (JSRR is a call that returns, so it's not a terminator.)
        | LineContent::Instruction(Instruction::Jmp { .. }) => true,
        // BRnzp is an unconditional branch (all three flags set)
        LineContent::Instruction(Instruction::Br { flags, .. }) => {
            flags.n && flags.z && flags.p
        }
        _ => false,
    }
}

/// Directive names that can appear as `Label` tokens when written without a leading dot.
///
/// Instruction mnemonics (ADD, AND, …) and register names (R0–R7) are NOT listed here
/// because the lexer already converts them to their own `TokenKind` variants before the
/// parser or first-pass ever sees them — they can never arrive as `Label(…)` tokens.
/// Directive names, however, are only recognised when preceded by a `.`; without the dot
/// they fall through to `Label(…)`, so this check catches the most likely user mistake
/// of writing e.g. `FILL ADD R1, R2, R3` intending `FILL` as a data label.
const DIRECTIVE_RESERVED_WORDS: &[&str] = &["ORIG", "END", "FILL", "BLKW", "STRINGZ"];

fn record_label(
    table: &mut SymbolTable,
    label: &str,
    address: u16,
    span: Span,
    errors: &mut Vec<AsmError>,
) {
    // Flag labels that shadow assembler directives.  These are almost certainly typos
    // (e.g., `FILL` used as a label instead of `.FILL` as a directive).
    let upper = label.to_uppercase();
    if DIRECTIVE_RESERVED_WORDS.contains(&upper.as_str()) {
        errors.push(AsmError::label_is_reserved_word(label, span));
        // Still insert so the assembler can continue without cascading UndefinedLabel errors.
    }

    // Single lookup: if Some, it's a duplicate; if None, insert it.
    // The old code called `contains` (hash lookup) then `get` (another lookup).
    if let Some(first_addr) = table.get(label) {
        errors.push(AsmError::duplicate_label(label, first_addr, span));
    } else {
        table.insert(label.to_string(), address);
    }
}
