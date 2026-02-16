//! LC-3 Machine Code Encoder
//!
//! This module converts parsed LC-3 assembly code into binary machine code.
//! It implements the complete LC-3 Instruction Set Architecture (ISA).
//!
//! ## Supported Instructions
//!
//! - **Operate**: ADD, AND, NOT
//! - **Data Movement**: LD, LDI, LDR, LEA, ST, STI, STR
//! - **Control Flow**: BR (with condition codes), JMP, JSR, JSRR, RTI, RET
//! - **Trap**: TRAP, GETC, OUT, PUTS, IN, PUTSP, HALT
//!
//! ## Directives
//!
//! - **.ORIG** - Set origin address
//! - **.FILL** - Fill one word with value or label address
//! - **.BLKW** - Allocate block of words
//! - **.STRINGZ** - Store null-terminated string
//! - **.END** - End of program

use crate::error::{AsmError, ErrorKind, Span};
use crate::first_pass::{symbol_table::SymbolTable, FirstPassResult};
use crate::parser::ast::{Instruction, LineContent, SourceLine};

/// Result of the encoding process
pub struct EncodeResult {
    /// Generated machine code as 16-bit words
    pub machine_code: Vec<u16>,
    /// Origin address where program should be loaded
    pub orig_address: u16,
    /// Errors encountered during encoding
    pub errors: Vec<AsmError>,
}

/// Encode the assembled program into LC-3 machine code
///
/// This function performs the second pass of the assembler, converting
/// the parsed AST and symbol table into binary machine code.
///
/// # Arguments
///
/// * `first_pass` - Result from the first pass containing AST and symbol table
///
/// # Returns
///
/// An `EncodeResult` containing the machine code and any errors encountered
pub fn encode(first_pass: &FirstPassResult) -> EncodeResult {
    let mut encoder = Encoder::new(&first_pass.symbol_table, first_pass.orig_address);

    for line in &first_pass.source_lines {
        encoder.encode_line(line);
    }

    EncodeResult {
        machine_code: encoder.machine_code,
        orig_address: encoder.orig_address,
        errors: encoder.errors,
    }
}

struct Encoder<'a> {
    symbol_table: &'a SymbolTable,
    machine_code: Vec<u16>,
    orig_address: u16,
    current_address: u16,
    errors: Vec<AsmError>,
}

impl<'a> Encoder<'a> {
    fn new(symbol_table: &'a SymbolTable, orig_address: u16) -> Self {
        Self {
            symbol_table,
            machine_code: Vec::new(),
            orig_address,
            current_address: orig_address,
            errors: Vec::new(),
        }
    }

    fn encode_line(&mut self, line: &SourceLine) {
        match &line.content {
            LineContent::Empty => {}
            LineContent::Orig(_) => {} // Already handled in first pass
            LineContent::End => {}     // End of program
            LineContent::FillImmediate(value) => {
                self.emit(*value as u16);
            }
            LineContent::FillLabel(label) => match self.symbol_table.get(label) {
                Some(addr) => self.emit(addr),
                None => {
                    self.errors
                        .push(AsmError::undefined_label(label, line.span));
                    self.emit(0);
                }
            },
            LineContent::Blkw(count) => {
                for _ in 0..*count {
                    self.emit(0);
                }
            }
            LineContent::Stringz(s) => {
                for ch in s.chars() {
                    self.emit(ch as u16);
                }
                self.emit(0); // Null terminator
            }
            LineContent::Instruction(inst) => {
                self.encode_instruction(inst, line.span);
            }
        }
    }

    fn encode_instruction(&mut self, inst: &Instruction, span: Span) {
        let encoded = match inst {
            // Operate instructions
            Instruction::AddReg { dr, sr1, sr2 } => {
                (0b0001 << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (*sr2 as u16)
            }
            Instruction::AddImm { dr, sr1, imm5 } => {
                let imm = sign_extend(*imm5, 5) & 0x1F;
                (0b0001 << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (1 << 5) | imm
            }
            Instruction::AndReg { dr, sr1, sr2 } => {
                (0b0101 << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (*sr2 as u16)
            }
            Instruction::AndImm { dr, sr1, imm5 } => {
                let imm = sign_extend(*imm5, 5) & 0x1F;
                (0b0101 << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (1 << 5) | imm
            }
            Instruction::Not { dr, sr } => {
                (0b1001 << 12) | ((*dr as u16) << 9) | ((*sr as u16) << 6) | 0b111111
            }

            // Data movement with PC offset
            Instruction::Ld { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (0b0010 << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::Ldi { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (0b1010 << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::Lea { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (0b1110 << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::St { sr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (0b0011 << 12) | ((*sr as u16) << 9) | offset
            }
            Instruction::Sti { sr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (0b1011 << 12) | ((*sr as u16) << 9) | offset
            }

            // Data movement with base+offset
            Instruction::Ldr {
                dr,
                base_r,
                offset6,
            } => {
                let offset = sign_extend(*offset6, 6) & 0x3F;
                (0b0110 << 12) | ((*dr as u16) << 9) | ((*base_r as u16) << 6) | offset
            }
            Instruction::Str {
                sr,
                base_r,
                offset6,
            } => {
                let offset = sign_extend(*offset6, 6) & 0x3F;
                (0b0111 << 12) | ((*sr as u16) << 9) | ((*base_r as u16) << 6) | offset
            }

            // Branch
            Instruction::Br { flags, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                let nzp =
                    ((flags.n as u16) << 11) | ((flags.z as u16) << 10) | ((flags.p as u16) << 9);
                nzp | offset
            }

            // Jump
            Instruction::Jmp { base_r } => (0b1100 << 12) | ((*base_r as u16) << 6),
            Instruction::Ret => {
                // RET is encoded as JMP R7
                (0b1100 << 12) | (7 << 6)
            }

            // Subroutine
            Instruction::Jsr { label } => {
                let offset = self.calc_pc_offset(label, 11, span);
                (0b0100 << 12) | (1 << 11) | offset
            }
            Instruction::Jsrr { base_r } => (0b0100 << 12) | ((*base_r as u16) << 6),

            // Trap
            Instruction::Trap { trapvect8 } => (0b1111 << 12) | (*trapvect8 as u16),
            Instruction::Getc => 0xF020,  // TRAP x20
            Instruction::Out => 0xF021,   // TRAP x21
            Instruction::Puts => 0xF022,  // TRAP x22
            Instruction::In => 0xF023,    // TRAP x23
            Instruction::Putsp => 0xF024, // TRAP x24
            Instruction::Halt => 0xF025,  // TRAP x25

            // System
            Instruction::Rti => 0x8000,
        };

        self.emit(encoded);
    }

    /// Calculate PC-relative offset to a label
    ///
    /// PC-relative addressing in LC-3 works as follows:
    /// 1. During execution, PC points to the NEXT instruction (current + 1)
    /// 2. The offset is added to this incremented PC: effective_address = PC + offset
    /// 3. Therefore: offset = target_address - (current_address + 1)
    ///
    /// The offset must fit in the specified number of bits as a signed value.
    /// For example, with 9 bits: range is -256 to +255
    fn calc_pc_offset(&mut self, label: &str, bits: u8, span: Span) -> u16 {
        match self.symbol_table.get(label) {
            Some(target_addr) => {
                // PC will point to next instruction during execution
                let pc = self.current_address.wrapping_add(1);

                // Calculate signed offset from PC to target
                let offset = (target_addr as i32) - (pc as i32);

                // Check if offset fits in the specified number of bits (signed range)
                let max_offset = (1 << (bits - 1)) - 1;
                let min_offset = -(1 << (bits - 1));

                if offset < min_offset || offset > max_offset {
                    self.errors.push(AsmError {
                        kind: ErrorKind::OffsetOutOfRange,
                        message: format!(
                            "PC offset {} to label '{}' exceeds {}-bit range [{}, {}]",
                            offset, label, bits, min_offset, max_offset
                        ),
                        span,
                    });
                    0 // Use 0 on error, but error is recorded
                } else {
                    // Mask to keep only the lower 'bits' bits (preserves two's complement)
                    (offset as u16) & ((1 << bits) - 1)
                }
            }
            None => {
                self.errors.push(AsmError::undefined_label(label, span));
                0
            }
        }
    }

    fn emit(&mut self, word: u16) {
        self.machine_code.push(word);
        self.current_address = self.current_address.wrapping_add(1);
    }
}

/// Truncate a signed value to N bits, preserving two's complement representation
///
/// This function takes a signed i16 value and masks it to fit in the specified
/// number of bits. The two's complement representation is preserved:
/// - Positive values: low bits are kept as-is
/// - Negative values: low bits contain two's complement encoding
///
/// Example: sign_extend(-1, 5) = 0b11111 (5-bit representation of -1)
fn sign_extend(value: i16, bits: u8) -> u16 {
    let mask = (1 << bits) - 1;
    (value as u16) & mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(5, 5), 0b00101);
        assert_eq!(sign_extend(-1, 5), 0b11111);
        assert_eq!(sign_extend(-16, 5), 0b10000);
    }
}
