pub mod symbol_table;

use crate::error::{AsmError, ErrorKind, Span};
use crate::parser::ast::{LineContent, SourceLine};
use symbol_table::SymbolTable;

pub struct FirstPassResult {
    pub symbol_table: SymbolTable,
    pub source_lines: Vec<SourceLine>,
    pub orig_address: u16,
    pub errors: Vec<AsmError>,
}

pub fn first_pass(lines: &[SourceLine]) -> FirstPassResult {
    // TODO-MED: Extract repetitive error construction (Err(AsmError { kind, message, span }))
    // into helper function to reduce boilerplate throughout this module
    let mut symbol_table = SymbolTable::new();
    let mut errors = Vec::new();
    let mut location_counter: Option<u16> = None;
    let mut orig_address: u16 = 0;
    let mut found_orig = false;
    let mut found_end = false;

    for line in lines {
        // TODO-MED: Replace boolean state flags (found_orig, found_end) with enum state machine
        if !found_orig {
            match &line.content {
                LineContent::Orig(addr) => {
                    found_orig = true;
                    orig_address = *addr;
                    location_counter = Some(*addr);
                    if let Some(ref label) = line.label {
                        record_label(&mut symbol_table, label, *addr, line.span, &mut errors);
                    }
                    continue;
                }
                LineContent::Empty => continue,
                _ => {
                    errors.push(AsmError {
                        kind: ErrorKind::MissingOrig,
                        message: "Expected .ORIG before any instructions".into(),
                        span: line.span,
                    });
                    found_orig = true;
                    orig_address = 0x3000;
                    location_counter = Some(0x3000);
                }
            }
        }

        if found_end {
            continue;
        }

        let lc = location_counter.unwrap();

        if let Some(ref label) = line.label {
            record_label(&mut symbol_table, label, lc, line.span, &mut errors);
        }

        let words: u32 = match &line.content {
            LineContent::Empty => 0,
            LineContent::Orig(_) => {
                errors.push(AsmError {
                    kind: ErrorKind::MultipleOrig,
                    message: "Multiple .ORIG directives are not supported".into(),
                    span: line.span,
                });
                0
            }
            LineContent::End => {
                found_end = true;
                0
            }
            LineContent::FillImmediate(_) => 1,
            LineContent::FillLabel(_) => 1,
            LineContent::Blkw(n) => {
                if *n == 0 {
                    errors.push(AsmError {
                        kind: ErrorKind::InvalidBlkwCount,
                        message: ".BLKW count must be positive".into(),
                        span: line.span,
                    });
                }
                *n as u32
            }
            LineContent::Stringz(s) => (s.len() as u32) + 1,
            LineContent::Instruction(_) => 1,
        };

        let new_lc = (lc as u32) + words;
        if new_lc > 0x10000 {
            errors.push(AsmError {
                kind: ErrorKind::AddressOverflow,
                message: format!(
                    "Address overflow: location counter would exceed 0xFFFF (at x{:04X} + {} words)",
                    lc, words
                ),
                span: line.span,
            });
            location_counter = Some(0xFFFF);
        } else {
            location_counter = Some(new_lc as u16);
        }
    }

    if !found_orig {
        errors.push(AsmError {
            kind: ErrorKind::MissingOrig,
            message: "No .ORIG directive found".into(),
            span: Span {
                start: 0,
                end: 0,
                line: 1,
                col: 1,
            },
        });
    }

    if !found_end {
        errors.push(AsmError {
            kind: ErrorKind::MissingEnd,
            message: "No .END directive found".into(),
            span: Span {
                start: 0,
                end: 0,
                line: 1,
                col: 1,
            },
        });
    }

    FirstPassResult {
        symbol_table,
        source_lines: lines.to_vec(),
        orig_address,
        errors,
    }
}

fn record_label(
    table: &mut SymbolTable,
    label: &str,
    address: u16,
    span: Span,
    errors: &mut Vec<AsmError>,
) {
    if table.contains(label) {
        errors.push(AsmError {
            kind: ErrorKind::DuplicateLabel,
            message: format!(
                "Duplicate label '{}' (first defined at x{:04X})",
                label,
                table.get(label).unwrap()
            ),
            span,
        });
    } else {
        table.insert(label.to_string(), address);
    }
}
