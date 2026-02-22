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

// LC-3 opcode constants — bits 15:12 of every instruction word.
const OP_ADD: u16 = 0b0001;
const OP_AND: u16 = 0b0101;
const OP_NOT: u16 = 0b1001;
const OP_LD: u16 = 0b0010;
const OP_LDI: u16 = 0b1010;
const OP_LEA: u16 = 0b1110;
const OP_ST: u16 = 0b0011;
const OP_STI: u16 = 0b1011;
const OP_LDR: u16 = 0b0110;
const OP_STR: u16 = 0b0111;
const OP_BR: u16 = 0b0000;
const OP_JMP: u16 = 0b1100;
const OP_JSR: u16 = 0b0100;
const OP_TRAP: u16 = 0b1111;
const OP_RTI: u16 = 0b1000;

// Full TRAP instruction words (opcode pre-shifted into bits 15:12).
const TRAP_GETC: u16 = (OP_TRAP << 12) | 0x20;
const TRAP_OUT: u16 = (OP_TRAP << 12) | 0x21;
const TRAP_PUTS: u16 = (OP_TRAP << 12) | 0x22;
const TRAP_IN: u16 = (OP_TRAP << 12) | 0x23;
const TRAP_PUTSP: u16 = (OP_TRAP << 12) | 0x24;
const TRAP_HALT: u16 = (OP_TRAP << 12) | 0x25;

/// Result of the encoding process
pub struct EncodeResult {
    /// Generated machine code as 16-bit words
    pub machine_code: Vec<u16>,
    /// Origin address where program should be loaded
    pub orig_address: u16,
    /// Errors encountered during encoding
    pub errors: Vec<AsmError>,
}

impl EncodeResult {
    /// Returns `true` if any encoding errors were recorded.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
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
#[must_use]
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
                    if !ch.is_ascii() {
                        self.errors
                            .push(AsmError::non_ascii_in_stringz(ch, line.span));
                        // Continue encoding so downstream errors are still caught.
                        // Emit the low byte to keep the word count predictable.
                        self.emit(ch as u8 as u16);
                    } else {
                        self.emit(ch as u16);
                    }
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
                (OP_ADD << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (*sr2 as u16)
            }
            Instruction::AddImm { dr, sr1, imm5 } => {
                let imm = sign_extend(*imm5, 5) & 0x1F;
                (OP_ADD << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (1 << 5) | imm
            }
            Instruction::AndReg { dr, sr1, sr2 } => {
                (OP_AND << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (*sr2 as u16)
            }
            Instruction::AndImm { dr, sr1, imm5 } => {
                let imm = sign_extend(*imm5, 5) & 0x1F;
                (OP_AND << 12) | ((*dr as u16) << 9) | ((*sr1 as u16) << 6) | (1 << 5) | imm
            }
            Instruction::Not { dr, sr } => {
                (OP_NOT << 12) | ((*dr as u16) << 9) | ((*sr as u16) << 6) | 0b111111
            }

            // Data movement with PC offset
            Instruction::Ld { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (OP_LD << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::Ldi { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (OP_LDI << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::Lea { dr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (OP_LEA << 12) | ((*dr as u16) << 9) | offset
            }
            Instruction::St { sr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (OP_ST << 12) | ((*sr as u16) << 9) | offset
            }
            Instruction::Sti { sr, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                (OP_STI << 12) | ((*sr as u16) << 9) | offset
            }

            // Data movement with base+offset
            Instruction::Ldr {
                dr,
                base_r,
                offset6,
            } => {
                let offset = sign_extend(*offset6, 6) & 0x3F;
                (OP_LDR << 12) | ((*dr as u16) << 9) | ((*base_r as u16) << 6) | offset
            }
            Instruction::Str {
                sr,
                base_r,
                offset6,
            } => {
                let offset = sign_extend(*offset6, 6) & 0x3F;
                (OP_STR << 12) | ((*sr as u16) << 9) | ((*base_r as u16) << 6) | offset
            }

            // Branch (opcode 0000 — zero, so no shift needed; flags occupy bits 11:9)
            Instruction::Br { flags, label } => {
                let offset = self.calc_pc_offset(label, 9, span);
                // BrFlags::as_u16() encodes [N][Z][P] as a 3-bit value.
                // Shifting left by 9 places n→bit11, z→bit10, p→bit9.
                (OP_BR << 12) | (flags.as_u16() << 9) | offset
            }

            // Jump
            Instruction::Jmp { base_r } => (OP_JMP << 12) | ((*base_r as u16) << 6),
            Instruction::Ret => {
                // RET is encoded as JMP R7
                (OP_JMP << 12) | (7 << 6)
            }

            // Subroutine
            Instruction::Jsr { label } => {
                let offset = self.calc_pc_offset(label, 11, span);
                (OP_JSR << 12) | (1 << 11) | offset
            }
            Instruction::Jsrr { base_r } => (OP_JSR << 12) | ((*base_r as u16) << 6),

            // Trap
            Instruction::Trap { trapvect8 } => (OP_TRAP << 12) | (*trapvect8 as u16),
            Instruction::Getc => TRAP_GETC,
            Instruction::Out => TRAP_OUT,
            Instruction::Puts => TRAP_PUTS,
            Instruction::In => TRAP_IN,
            Instruction::Putsp => TRAP_PUTSP,
            Instruction::Halt => TRAP_HALT,

            // System
            Instruction::Rti => OP_RTI << 12,
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
                        message: format!("PC offset {offset} to label '{label}' exceeds {bits}-bit range [{min_offset}, {max_offset}]"),
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
const fn sign_extend(value: i16, bits: u8) -> u16 {
    let mask = (1 << bits) - 1;
    (value as u16) & mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::first_pass::symbol_table::SymbolTable;
    use crate::lexer::token::BrFlags;

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

    const DUMMY_SPAN: Span = Span { line: 1, col: 1 };

    /// Build a minimal `FirstPassResult` from an origin address, a list of
    /// `LineContent` items, and a symbol table.  Every line gets a dummy span
    /// and no label — the encoder doesn't inspect those fields.
    fn build_first_pass(
        orig: u16,
        contents: Vec<LineContent>,
        symbols: SymbolTable,
    ) -> FirstPassResult {
        let mut lines = vec![SourceLine {
            label: None,
            content: LineContent::Orig(orig),
            line_number: 1,
            span: DUMMY_SPAN,
        }];
        for (i, c) in contents.into_iter().enumerate() {
            lines.push(SourceLine {
                label: None,
                content: c,
                line_number: i + 2,
                span: DUMMY_SPAN,
            });
        }
        FirstPassResult {
            symbol_table: symbols,
            source_lines: lines,
            orig_address: orig,
            errors: Vec::new(),
        }
    }

    /// Encode a single instruction at origin 0x3000 with no symbols needed.
    fn encode_single(inst: Instruction) -> u16 {
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(inst)],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(
            result.errors.is_empty(),
            "unexpected errors: {:?}",
            result.errors
        );
        assert_eq!(result.machine_code.len(), 1);
        result.machine_code[0]
    }

    fn symbols_with_target(label: &str, addr: u16) -> SymbolTable {
        let mut st = SymbolTable::new();
        st.insert(label.to_string(), addr);
        st
    }

    // ---------------------------------------------------------------
    // sign_extend
    // ---------------------------------------------------------------

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(5, 5), 0b00101);
        assert_eq!(sign_extend(-1, 5), 0b11111);
        assert_eq!(sign_extend(-16, 5), 0b10000);
    }

    #[test]
    fn test_sign_extend_6bit() {
        assert_eq!(sign_extend(31, 6), 0b011111);
        assert_eq!(sign_extend(-32, 6), 0b100000);
        assert_eq!(sign_extend(-1, 6), 0b111111);
    }

    // ---------------------------------------------------------------
    // Operate instructions: ADD, AND, NOT
    // ---------------------------------------------------------------

    #[test]
    fn encode_add_reg() {
        // ADD R2, R3, R4  →  0001 010 011 000 100 = 0x14C4
        let word = encode_single(Instruction::AddReg {
            dr: 2,
            sr1: 3,
            sr2: 4,
        });
        assert_eq!(word, 0x14C4);
    }

    #[test]
    fn encode_add_imm_positive() {
        // ADD R1, R1, #5  →  0001 001 001 1 00101 = 0x1265
        let word = encode_single(Instruction::AddImm {
            dr: 1,
            sr1: 1,
            imm5: 5,
        });
        assert_eq!(word, 0x1265);
    }

    #[test]
    fn encode_add_imm_negative() {
        // ADD R0, R0, #-1  →  0001 000 000 1 11111 = 0x103F
        let word = encode_single(Instruction::AddImm {
            dr: 0,
            sr1: 0,
            imm5: -1,
        });
        assert_eq!(word, 0x103F);
    }

    #[test]
    fn encode_and_reg() {
        // AND R5, R6, R7  →  0101 101 110 000 111 = 0x5B87
        let word = encode_single(Instruction::AndReg {
            dr: 5,
            sr1: 6,
            sr2: 7,
        });
        assert_eq!(word, 0x5B87);
    }

    #[test]
    fn encode_and_imm() {
        // AND R0, R0, #0  →  0101 000 000 1 00000 = 0x5020
        let word = encode_single(Instruction::AndImm {
            dr: 0,
            sr1: 0,
            imm5: 0,
        });
        assert_eq!(word, 0x5020);
    }

    #[test]
    fn encode_not() {
        // NOT R4, R5  →  1001 100 101 111111 = 0x997F
        let word = encode_single(Instruction::Not { dr: 4, sr: 5 });
        assert_eq!(word, 0x997F);
    }

    // ---------------------------------------------------------------
    // Data movement with base+offset: LDR, STR
    // ---------------------------------------------------------------

    #[test]
    fn encode_ldr() {
        // LDR R2, R3, #5  →  0110 010 011 000101 = 0x64C5
        let word = encode_single(Instruction::Ldr {
            dr: 2,
            base_r: 3,
            offset6: 5,
        });
        assert_eq!(word, 0x64C5);
    }

    #[test]
    fn encode_ldr_negative_offset() {
        // LDR R0, R1, #-1  →  0110 000 001 111111 = 0x607F
        let word = encode_single(Instruction::Ldr {
            dr: 0,
            base_r: 1,
            offset6: -1,
        });
        assert_eq!(word, 0x607F);
    }

    #[test]
    fn encode_str() {
        // STR R7, R6, #0  →  0111 111 110 000000 = 0x7F80
        let word = encode_single(Instruction::Str {
            sr: 7,
            base_r: 6,
            offset6: 0,
        });
        assert_eq!(word, 0x7F80);
    }

    // ---------------------------------------------------------------
    // Data movement with PC offset: LD, LDI, LEA, ST, STI
    // ---------------------------------------------------------------

    #[test]
    fn encode_ld() {
        // LD R3, TARGET (TARGET=0x3005, inst at 0x3000, PC=0x3001, offset=4)
        // 0010 011 000000100 = 0x2604
        let st = symbols_with_target("TARGET", 0x3005);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 3,
                label: "TARGET".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x2604);
    }

    #[test]
    fn encode_ldi() {
        // LDI R0, PTR (PTR=0x3003, inst at 0x3000, offset=2)
        // 1010 000 000000010 = 0xA002
        let st = symbols_with_target("PTR", 0x3003);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Ldi {
                dr: 0,
                label: "PTR".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0xA002);
    }

    #[test]
    fn encode_lea() {
        // LEA R7, MSG (MSG=0x3002, inst at 0x3000, offset=1)
        // 1110 111 000000001 = 0xEE01
        let st = symbols_with_target("MSG", 0x3002);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Lea {
                dr: 7,
                label: "MSG".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0xEE01);
    }

    #[test]
    fn encode_st() {
        // ST R2, DATA (DATA=0x3004, inst at 0x3000, offset=3)
        // 0011 010 000000011 = 0x3403
        let st = symbols_with_target("DATA", 0x3004);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::St {
                sr: 2,
                label: "DATA".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x3403);
    }

    #[test]
    fn encode_sti() {
        // STI R1, PTR (PTR=0x3002, inst at 0x3000, offset=1)
        // 1011 001 000000001 = 0xB201
        let st = symbols_with_target("PTR", 0x3002);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Sti {
                sr: 1,
                label: "PTR".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0xB201);
    }

    // ---------------------------------------------------------------
    // Control flow: BR, JMP, RET, JSR, JSRR, RTI
    // ---------------------------------------------------------------

    #[test]
    fn encode_br_nzp() {
        // BRnzp LOOP (LOOP=0x3000, inst at 0x3000, offset=-1)
        // 0000 111 111111111 = 0x0FFF
        let st = symbols_with_target("LOOP", 0x3000);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Br {
                flags: BrFlags::new(true, true, true),
                label: "LOOP".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x0FFF);
    }

    #[test]
    fn encode_brn() {
        // BRn BACK (BACK=0x2FFF, inst at 0x3000, offset=-2)
        // 0000 100 111111110 = 0x09FE
        let st = symbols_with_target("BACK", 0x2FFF);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Br {
                flags: BrFlags::new(true, false, false),
                label: "BACK".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x09FE);
    }

    #[test]
    fn encode_brzp() {
        // BRzp FWD (FWD=0x3005, inst at 0x3000, offset=4)
        // 0000 011 000000100 = 0x0604
        let st = symbols_with_target("FWD", 0x3005);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Br {
                flags: BrFlags::new(false, true, true),
                label: "FWD".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x0604);
    }

    #[test]
    fn encode_jmp() {
        // JMP R3  →  1100 000 011 000000 = 0xC0C0
        let word = encode_single(Instruction::Jmp { base_r: 3 });
        assert_eq!(word, 0xC0C0);
    }

    #[test]
    fn encode_ret() {
        // RET = JMP R7  →  1100 000 111 000000 = 0xC1C0
        let word = encode_single(Instruction::Ret);
        assert_eq!(word, 0xC1C0);
    }

    #[test]
    fn encode_jsr() {
        // JSR SUB (SUB=0x3100, inst at 0x3000, PC=0x3001, offset=0xFF)
        // 0100 1 00011111111 = 0x48FF
        let st = symbols_with_target("SUB", 0x3100);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Jsr {
                label: "SUB".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x48FF);
    }

    #[test]
    fn encode_jsrr() {
        // JSRR R4  →  0100 0 00 100 000000 = 0x4100
        let word = encode_single(Instruction::Jsrr { base_r: 4 });
        assert_eq!(word, 0x4100);
    }

    #[test]
    fn encode_rti() {
        // RTI  →  1000 000000000000 = 0x8000
        let word = encode_single(Instruction::Rti);
        assert_eq!(word, 0x8000);
    }

    // ---------------------------------------------------------------
    // Trap instructions and aliases
    // ---------------------------------------------------------------

    #[test]
    fn encode_trap() {
        // TRAP x25  →  1111 0000 00100101 = 0xF025
        let word = encode_single(Instruction::Trap { trapvect8: 0x25 });
        assert_eq!(word, 0xF025);
    }

    #[test]
    fn encode_trap_aliases() {
        assert_eq!(encode_single(Instruction::Getc), 0xF020);
        assert_eq!(encode_single(Instruction::Out), 0xF021);
        assert_eq!(encode_single(Instruction::Puts), 0xF022);
        assert_eq!(encode_single(Instruction::In), 0xF023);
        assert_eq!(encode_single(Instruction::Putsp), 0xF024);
        assert_eq!(encode_single(Instruction::Halt), 0xF025);
    }

    // ---------------------------------------------------------------
    // Directives: .FILL, .BLKW, .STRINGZ
    // ---------------------------------------------------------------

    #[test]
    fn encode_fill_immediate() {
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::FillImmediate(42)],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![42]);
    }

    #[test]
    fn encode_fill_negative() {
        // .FILL #-1  →  stored as 0xFFFF (i32 -1 cast to u16)
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::FillImmediate(-1)],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![0xFFFF]);
    }

    #[test]
    fn encode_fill_label() {
        let st = symbols_with_target("DATA", 0x4000);
        let fp = build_first_pass(0x3000, vec![LineContent::FillLabel("DATA".into())], st);
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![0x4000]);
    }

    #[test]
    fn encode_blkw() {
        let fp = build_first_pass(0x3000, vec![LineContent::Blkw(5)], SymbolTable::new());
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn encode_stringz() {
        // .STRINGZ "Hi" → 'H'=0x48, 'i'=0x69, null=0x00
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Stringz("Hi".into())],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![0x48, 0x69, 0x00]);
    }

    #[test]
    fn encode_stringz_empty() {
        // .STRINGZ "" → just null terminator
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Stringz(String::new())],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code, vec![0x00]);
    }

    #[test]
    fn encode_stringz_non_ascii_errors() {
        // .STRINGZ "café" → 'c', 'a', 'f' are ASCII; 'é' (U+00E9) is not
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Stringz("café".into())],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, crate::error::ErrorKind::NonAsciiInStringz);
        // Word count is still 5: 'c','a','f','é'(low byte),'\\0'
        assert_eq!(result.machine_code.len(), 5);
        assert_eq!(result.machine_code[0], b'c' as u16);
        assert_eq!(result.machine_code[3], 0x00E9u32 as u8 as u16); // low byte of é
        assert_eq!(*result.machine_code.last().unwrap(), 0x0000); // null terminator
    }

    #[test]
    fn encode_stringz_all_ascii_no_error() {
        // Full printable ASCII range should produce zero errors
        let s: String = (0x20u8..=0x7Eu8).map(|b| b as char).collect();
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Stringz(s.clone())],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty(), "pure ASCII should not error");
        assert_eq!(result.machine_code.len(), s.len() + 1); // chars + null
    }

    // ---------------------------------------------------------------
    // PC offset edge cases
    // ---------------------------------------------------------------

    #[test]
    fn pc_offset_max_positive_9bit() {
        // 9-bit signed max = +255. inst at 0x3000, PC=0x3001, target=0x3100
        // offset = 0x3100 - 0x3001 = 0xFF = 255
        let st = symbols_with_target("FAR", 0x3100);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "FAR".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x20FF);
    }

    #[test]
    fn pc_offset_max_negative_9bit() {
        // 9-bit signed min = -256. inst at 0x3100, PC=0x3101, target=0x3001
        // offset = 0x3001 - 0x3101 = -256
        let st = symbols_with_target("BACK", 0x3001);
        let fp = build_first_pass(
            0x3100,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "BACK".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x2100);
    }

    #[test]
    fn pc_offset_out_of_range_positive() {
        // 9-bit max is +255. target at +256 → error.
        let st = symbols_with_target("TOO_FAR", 0x3101);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "TOO_FAR".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::OffsetOutOfRange);
    }

    #[test]
    fn pc_offset_out_of_range_negative() {
        // 9-bit min is -256. target at -257 → error.
        let st = symbols_with_target("TOO_FAR_BACK", 0x3000);
        let fp = build_first_pass(
            0x3100,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "TOO_FAR_BACK".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::OffsetOutOfRange);
    }

    #[test]
    fn pc_offset_jsr_11bit_max() {
        // JSR 11-bit max = +1023. inst at 0x3000, PC=0x3001, target=0x3400
        let st = symbols_with_target("FUNC", 0x3400);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Jsr {
                label: "FUNC".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code[0], 0x4BFF);
    }

    #[test]
    fn pc_offset_jsr_out_of_range() {
        // JSR 11-bit max is +1023. target=0x3401 → offset=1024 → error
        let st = symbols_with_target("FUNC", 0x3401);
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Jsr {
                label: "FUNC".into(),
            })],
            st,
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::OffsetOutOfRange);
    }

    // ---------------------------------------------------------------
    // Error paths
    // ---------------------------------------------------------------

    #[test]
    fn undefined_label_in_instruction() {
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::Instruction(Instruction::Ld {
                dr: 0,
                label: "MISSING".into(),
            })],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::UndefinedLabel);
        assert_eq!(result.machine_code.len(), 1); // still emits a word (0)
    }

    #[test]
    fn undefined_label_in_fill() {
        let fp = build_first_pass(
            0x3000,
            vec![LineContent::FillLabel("NOPE".into())],
            SymbolTable::new(),
        );
        let result = encode(&fp);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].kind, ErrorKind::UndefinedLabel);
        assert_eq!(result.machine_code, vec![0]);
    }

    // ---------------------------------------------------------------
    // Address tracking (emit advances current_address correctly)
    // ---------------------------------------------------------------

    #[test]
    fn address_tracking_multi_instruction() {
        // inst0=0x3000, inst1=0x3001, inst2=0x3002 (BR back to TOP=0x3000)
        // BR PC=0x3003, offset = 0x3000 - 0x3003 = -3
        let st = symbols_with_target("TOP", 0x3000);
        let fp = build_first_pass(
            0x3000,
            vec![
                LineContent::Instruction(Instruction::AddImm {
                    dr: 0,
                    sr1: 0,
                    imm5: 1,
                }),
                LineContent::Instruction(Instruction::AddImm {
                    dr: 1,
                    sr1: 1,
                    imm5: -1,
                }),
                LineContent::Instruction(Instruction::Br {
                    flags: BrFlags::new(false, false, true),
                    label: "TOP".into(),
                }),
            ],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code.len(), 3);
        // BRp offset=-3 → 0000 001 111111101 = 0x03FD
        assert_eq!(result.machine_code[2], 0x03FD);
    }

    #[test]
    fn address_tracking_blkw_affects_offset() {
        // .BLKW 10 at 0x3000 → 10 words, then LD at 0x300A
        // TARGET=0x3005, PC=0x300B, offset = 0x3005 - 0x300B = -6
        let st = symbols_with_target("TARGET", 0x3005);
        let fp = build_first_pass(
            0x3000,
            vec![
                LineContent::Blkw(10),
                LineContent::Instruction(Instruction::Ld {
                    dr: 0,
                    label: "TARGET".into(),
                }),
            ],
            st,
        );
        let result = encode(&fp);
        assert!(result.errors.is_empty());
        assert_eq!(result.machine_code.len(), 11); // 10 zeros + 1 instruction
                                                   // LD R0, offset=-6 → 0010 000 111111010 = 0x21FA
        assert_eq!(result.machine_code[10], 0x21FA);
    }

    // ---------------------------------------------------------------
    // orig_address propagation
    // ---------------------------------------------------------------

    #[test]
    fn orig_address_propagated() {
        let fp = build_first_pass(0x4000, vec![], SymbolTable::new());
        let result = encode(&fp);
        assert_eq!(result.orig_address, 0x4000);
        assert!(result.machine_code.is_empty());
    }
}
