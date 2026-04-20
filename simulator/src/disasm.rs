use std::collections::HashMap;

use crate::machine::sext;

/// Disassemble one 16-bit word at `addr` into a human-readable string.
///
/// `addr` is the word address of `word` in memory.  PC-relative targets are
/// computed as `addr + 1 + offset` (matching LC-3 execution semantics).
/// If `syms` is provided, target addresses are replaced with label names.
pub fn disassemble(word: u16, addr: u16, syms: Option<&HashMap<u16, String>>) -> String {
    // PC seen during execution of this instruction = addr + 1.
    let pc = addr.wrapping_add(1);

    match word >> 12 {
        // ADD
        0b0001 => {
            let dr = (word >> 9) & 7;
            let sr1 = (word >> 6) & 7;
            if (word >> 5) & 1 == 0 {
                format!("ADD R{dr}, R{sr1}, R{}", word & 7)
            } else {
                format!("ADD R{dr}, R{sr1}, #{}", sext_signed(word & 0x1F, 5))
            }
        }
        // AND
        0b0101 => {
            let dr = (word >> 9) & 7;
            let sr1 = (word >> 6) & 7;
            if (word >> 5) & 1 == 0 {
                format!("AND R{dr}, R{sr1}, R{}", word & 7)
            } else {
                format!("AND R{dr}, R{sr1}, #{}", sext_signed(word & 0x1F, 5))
            }
        }
        // NOT
        0b1001 => {
            let dr = (word >> 9) & 7;
            let sr = (word >> 6) & 7;
            format!("NOT R{dr}, R{sr}")
        }
        // LD
        0b0010 => reg_pc9("LD", word, pc, syms),
        // LDI
        0b1010 => reg_pc9("LDI", word, pc, syms),
        // LEA
        0b1110 => reg_pc9("LEA", word, pc, syms),
        // ST
        0b0011 => sr_pc9("ST", word, pc, syms),
        // STI
        0b1011 => sr_pc9("STI", word, pc, syms),
        // LDR
        0b0110 => {
            let dr = (word >> 9) & 7;
            let br = (word >> 6) & 7;
            format!("LDR R{dr}, R{br}, #{}", sext_signed(word & 0x3F, 6))
        }
        // STR
        0b0111 => {
            let sr = (word >> 9) & 7;
            let br = (word >> 6) & 7;
            format!("STR R{sr}, R{br}, #{}", sext_signed(word & 0x3F, 6))
        }
        // BR
        0b0000 => {
            let n = (word >> 11) & 1 != 0;
            let z = (word >> 10) & 1 != 0;
            let p = (word >> 9) & 1 != 0;
            let offset = sext(word & 0x1FF, 9);
            let target = pc.wrapping_add(offset);
            let label = resolve(target, syms);
            if !n && !z && !p {
                "NOP".into()
            } else if n && z && p {
                format!("BR {label}")
            } else {
                let mut m = String::from("BR");
                if n {
                    m.push('n');
                }
                if z {
                    m.push('z');
                }
                if p {
                    m.push('p');
                }
                format!("{m} {label}")
            }
        }
        // JMP / RET
        0b1100 => {
            let base = (word >> 6) & 7;
            if base == 7 {
                "RET".into()
            } else {
                format!("JMP R{base}")
            }
        }
        // JSR / JSRR
        0b0100 => {
            if (word >> 11) & 1 == 1 {
                let offset = sext(word & 0x7FF, 11);
                let target = pc.wrapping_add(offset);
                format!("JSR {}", resolve(target, syms))
            } else {
                format!("JSRR R{}", (word >> 6) & 7)
            }
        }
        // RTI
        0b1000 => "RTI".into(),
        // TRAP
        0b1111 => match word & 0xFF {
            0x20 => "GETC".into(),
            0x21 => "OUT".into(),
            0x22 => "PUTS".into(),
            0x23 => "IN".into(),
            0x24 => "PUTSP".into(),
            0x25 => "HALT".into(),
            v => format!("TRAP x{v:02X}"),
        },
        // Data / unknown opcode
        _ => format!(".FILL x{word:04X}"),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Sign-extended value as a signed i32 (for display with `#` prefix).
fn sext_signed(value: u16, bits: u8) -> i32 {
    sext(value, bits) as i16 as i32
}

/// Resolve an address to a label name or `xADDR` fallback.
fn resolve(addr: u16, syms: Option<&HashMap<u16, String>>) -> String {
    syms.and_then(|m| m.get(&addr))
        .cloned()
        .unwrap_or_else(|| format!("x{addr:04X}"))
}

/// Disassemble a DR-based PC+offset9 instruction (LD, LDI, LEA).
fn reg_pc9(mnem: &str, word: u16, pc: u16, syms: Option<&HashMap<u16, String>>) -> String {
    let dr = (word >> 9) & 7;
    let target = pc.wrapping_add(sext(word & 0x1FF, 9));
    format!("{mnem} R{dr}, {}", resolve(target, syms))
}

/// Disassemble a SR-based PC+offset9 instruction (ST, STI).
fn sr_pc9(mnem: &str, word: u16, pc: u16, syms: Option<&HashMap<u16, String>>) -> String {
    let sr = (word >> 9) & 7;
    let target = pc.wrapping_add(sext(word & 0x1FF, 9));
    format!("{mnem} R{sr}, {}", resolve(target, syms))
}
