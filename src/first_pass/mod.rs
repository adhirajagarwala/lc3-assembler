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

#[derive(Debug, Clone, Copy, PartialEq)]
enum AssemblerState {
    WaitingForOrig,
    Processing,
    AfterEnd,
}

pub fn first_pass(lines: &[SourceLine]) -> FirstPassResult {
    let mut symbol_table = SymbolTable::new();
    let mut errors = Vec::new();
    let mut location_counter: Option<u16> = None;
    let mut orig_address: u16 = 0;
    let mut state = AssemblerState::WaitingForOrig;

    for line in lines {
        match state {
            AssemblerState::WaitingForOrig => {
                match &line.content {
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
                }
            }
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

    if state == AssemblerState::WaitingForOrig {
        errors.push(AsmError::new(
            ErrorKind::MissingOrig,
            "No .ORIG directive found",
            Span { start: 0, end: 0, line: 1, col: 1 },
        ));
    }

    if state != AssemblerState::AfterEnd {
        errors.push(AsmError::new(
            ErrorKind::MissingEnd,
            "No .END directive found",
            Span { start: 0, end: 0, line: 1, col: 1 },
        ));
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
        let first_addr = table.get(label).unwrap();
        errors.push(AsmError::duplicate_label(label, first_addr, span));
    } else {
        table.insert(label.to_string(), address);
    }
}
