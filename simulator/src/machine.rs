use std::collections::{HashSet, VecDeque};

use crate::memory::Memory;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CondCode {
    N,
    Z,
    P,
}

impl std::fmt::Display for CondCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CondCode::N => write!(f, "N"),
            CondCode::Z => write!(f, "Z"),
            CondCode::P => write!(f, "P"),
        }
    }
}

pub struct Registers {
    pub gpr: [u16; 8],
    pub pc: u16,
    pub cc: CondCode,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            gpr: [0u16; 8],
            pc: 0x3000,
            cc: CondCode::Z,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StepResult {
    Ok,
    Halted,
    BreakpointHit(u16),
    IllegalInstruction(u16),
}

pub struct Machine {
    pub regs: Registers,
    pub mem: Memory,
    pub halted: bool,
    /// Characters buffered into the current output line.
    pub output_buf: String,
    /// Completed output lines (each TRAP PUTS/OUT call appends here on newline).
    pub output_lines: Vec<String>,
    /// Pending keyboard input consumed by GETC.
    pub input_queue: VecDeque<u8>,
    /// Set when GETC finds `input_queue` empty; cleared once a char arrives.
    pub waiting_for_input: bool,
    pub breakpoints: HashSet<u16>,
    pub step_count: u64,
    /// When true, OUT/PUTS write directly to stdout instead of output_lines.
    pub headless: bool,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            mem: Memory::new(),
            halted: false,
            output_buf: String::new(),
            output_lines: Vec::new(),
            input_queue: VecDeque::new(),
            waiting_for_input: false,
            breakpoints: HashSet::new(),
            step_count: 0,
            headless: false,
        }
    }

    /// Load an LC-3 object file (big-endian: origin word + code words).
    /// Returns the origin address on success.
    pub fn load_obj(&mut self, data: &[u8]) -> Result<u16, String> {
        if data.len() < 2 {
            return Err("object file too short (missing origin word)".into());
        }
        let orig = u16::from_be_bytes([data[0], data[1]]);
        let words: Vec<u16> = data[2..]
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
            .collect();
        self.mem.load(orig, &words);
        self.regs.pc = orig;
        Ok(orig)
    }

    /// Execute one instruction. Returns immediately if already halted.
    pub fn step(&mut self) -> StepResult {
        if self.halted {
            return StepResult::Halted;
        }
        if self.waiting_for_input {
            return StepResult::Ok; // caller must feed input_queue first
        }
        if !self.mem.clock_enabled() {
            self.halted = true;
            return StepResult::Halted;
        }

        let pc = self.regs.pc;
        let ir = self.mem.read(pc);
        self.regs.pc = pc.wrapping_add(1);
        self.step_count += 1;

        let result = self.execute(ir);

        // Check breakpoints on the *next* PC (where execution will land).
        if result == StepResult::Ok && self.breakpoints.contains(&self.regs.pc) {
            return StepResult::BreakpointHit(self.regs.pc);
        }
        result
    }

    /// Run up to `max` steps, stopping at halt, breakpoint, or input wait.
    pub fn run_steps(&mut self, max: u64) -> StepResult {
        for _ in 0..max {
            match self.step() {
                StepResult::Ok => {}
                other => return other,
            }
            if self.waiting_for_input {
                return StepResult::Ok;
            }
        }
        StepResult::Ok
    }

    /// Push a character to the output, handling newlines as line boundaries.
    pub fn push_char(&mut self, ch: char) {
        if self.headless {
            print!("{ch}");
        } else if ch == '\n' {
            let line = std::mem::take(&mut self.output_buf);
            self.output_lines.push(line);
        } else {
            self.output_buf.push(ch);
        }
    }

    /// Push a whole string, splitting on newlines.
    pub fn push_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.push_char(ch);
        }
    }

    // ── Instruction execution ─────────────────────────────────────────────────

    fn execute(&mut self, ir: u16) -> StepResult {
        match ir >> 12 {
            // ADD
            0b0001 => {
                let dr = ((ir >> 9) & 7) as usize;
                let sr1 = ((ir >> 6) & 7) as usize;
                let result = if (ir >> 5) & 1 == 0 {
                    self.regs.gpr[sr1].wrapping_add(self.regs.gpr[(ir & 7) as usize])
                } else {
                    self.regs.gpr[sr1].wrapping_add(sext(ir & 0x1F, 5))
                };
                self.regs.gpr[dr] = result;
                set_cc(&mut self.regs, result);
            }
            // AND
            0b0101 => {
                let dr = ((ir >> 9) & 7) as usize;
                let sr1 = ((ir >> 6) & 7) as usize;
                let result = if (ir >> 5) & 1 == 0 {
                    self.regs.gpr[sr1] & self.regs.gpr[(ir & 7) as usize]
                } else {
                    self.regs.gpr[sr1] & sext(ir & 0x1F, 5)
                };
                self.regs.gpr[dr] = result;
                set_cc(&mut self.regs, result);
            }
            // NOT
            0b1001 => {
                let dr = ((ir >> 9) & 7) as usize;
                let sr = ((ir >> 6) & 7) as usize;
                let result = !self.regs.gpr[sr];
                self.regs.gpr[dr] = result;
                set_cc(&mut self.regs, result);
            }
            // LD
            0b0010 => {
                let dr = ((ir >> 9) & 7) as usize;
                let addr = self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9));
                let val = self.mem.read(addr);
                self.regs.gpr[dr] = val;
                set_cc(&mut self.regs, val);
            }
            // LDI
            0b1010 => {
                let dr = ((ir >> 9) & 7) as usize;
                let ptr = self
                    .mem
                    .read(self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9)));
                let val = self.mem.read(ptr);
                self.regs.gpr[dr] = val;
                set_cc(&mut self.regs, val);
            }
            // LDR
            0b0110 => {
                let dr = ((ir >> 9) & 7) as usize;
                let base = self.regs.gpr[((ir >> 6) & 7) as usize];
                let addr = base.wrapping_add(sext(ir & 0x3F, 6));
                let val = self.mem.read(addr);
                self.regs.gpr[dr] = val;
                set_cc(&mut self.regs, val);
            }
            // LEA  (does NOT set CC)
            0b1110 => {
                let dr = ((ir >> 9) & 7) as usize;
                self.regs.gpr[dr] = self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9));
            }
            // ST
            0b0011 => {
                let sr = ((ir >> 9) & 7) as usize;
                let addr = self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9));
                self.mem.write(addr, self.regs.gpr[sr]);
            }
            // STI
            0b1011 => {
                let sr = ((ir >> 9) & 7) as usize;
                let ptr = self
                    .mem
                    .read(self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9)));
                self.mem.write(ptr, self.regs.gpr[sr]);
            }
            // STR
            0b0111 => {
                let sr = ((ir >> 9) & 7) as usize;
                let base = self.regs.gpr[((ir >> 6) & 7) as usize];
                let addr = base.wrapping_add(sext(ir & 0x3F, 6));
                self.mem.write(addr, self.regs.gpr[sr]);
            }
            // BR
            0b0000 => {
                let n = (ir >> 11) & 1 != 0;
                let z = (ir >> 10) & 1 != 0;
                let p = (ir >> 9) & 1 != 0;
                let taken = (n && self.regs.cc == CondCode::N)
                    || (z && self.regs.cc == CondCode::Z)
                    || (p && self.regs.cc == CondCode::P);
                if taken {
                    self.regs.pc = self.regs.pc.wrapping_add(sext(ir & 0x1FF, 9));
                }
            }
            // JMP / RET
            0b1100 => {
                self.regs.pc = self.regs.gpr[((ir >> 6) & 7) as usize];
            }
            // JSR / JSRR
            0b0100 => {
                let saved = self.regs.pc;
                if (ir >> 11) & 1 == 1 {
                    // JSR: PC-relative
                    self.regs.pc = saved.wrapping_add(sext(ir & 0x7FF, 11));
                } else {
                    // JSRR: register — save base before overwriting R7
                    let base = self.regs.gpr[((ir >> 6) & 7) as usize];
                    self.regs.pc = base;
                }
                self.regs.gpr[7] = saved;
            }
            // RTI (simplified: restore PC from R7, no privilege logic)
            0b1000 => {
                self.regs.pc = self.regs.gpr[7];
            }
            // TRAP
            0b1111 => {
                // Save return address
                self.regs.gpr[7] = self.regs.pc;
                let vect = (ir & 0xFF) as u8;
                return self.dispatch_trap(vect);
            }
            // Illegal / reserved
            _ => {
                return StepResult::IllegalInstruction(ir);
            }
        }
        StepResult::Ok
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Sign-extend `value` from `bits` wide to 16 bits.
#[inline]
pub(crate) fn sext(value: u16, bits: u8) -> u16 {
    let shift = 16u8 - bits;
    (((value as i16) << shift) >> shift) as u16
}

/// Set condition codes from a 16-bit result (interpreted as signed).
fn set_cc(regs: &mut Registers, value: u16) {
    regs.cc = match (value as i16).cmp(&0) {
        std::cmp::Ordering::Less => CondCode::N,
        std::cmp::Ordering::Equal => CondCode::Z,
        std::cmp::Ordering::Greater => CondCode::P,
    };
}
