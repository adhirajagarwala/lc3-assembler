/// TRAP service routine dispatch.
///
/// All six standard LC-3 traps are implemented natively in Rust.
/// The machine's `R7` is already set to the return address by `machine.rs`
/// before this function is called.
use crate::machine::{Machine, StepResult};

impl Machine {
    pub(crate) fn dispatch_trap(&mut self, vect: u8) -> StepResult {
        match vect {
            0x20 => self.trap_getc(),
            0x21 => self.trap_out(),
            0x22 => self.trap_puts(),
            0x23 => self.trap_in(),
            0x24 => self.trap_putsp(),
            0x25 => self.trap_halt(),
            v => {
                self.push_str(&format!("[TRAP x{v:02X}: unimplemented]\n"));
                self.halted = true;
                StepResult::Halted
            }
        }
    }

    /// GETC — read one character from the keyboard into R0[7:0].
    ///
    /// If the input queue is empty the machine stalls: PC is wound back one
    /// word so the TRAP instruction re-executes on the next step, and
    /// `waiting_for_input` is set so the caller knows to feed the queue.
    fn trap_getc(&mut self) -> StepResult {
        if let Some(ch) = self.input_queue.pop_front() {
            self.regs.gpr[0] = ch as u16;
            self.waiting_for_input = false;
        } else {
            // Rewind PC so this TRAP re-fires once a char is available.
            self.regs.pc = self.regs.pc.wrapping_sub(1);
            self.waiting_for_input = true;
        }
        StepResult::Ok
    }

    /// OUT — write R0[7:0] as an ASCII character.
    fn trap_out(&mut self) -> StepResult {
        let ch = (self.regs.gpr[0] & 0xFF) as u8 as char;
        self.push_char(ch);
        StepResult::Ok
    }

    /// PUTS — write a null-terminated string starting at mem[R0].
    fn trap_puts(&mut self) -> StepResult {
        let mut addr = self.regs.gpr[0];
        loop {
            let word = self.mem.read(addr);
            if word == 0 {
                break;
            }
            self.push_char((word & 0xFF) as u8 as char);
            addr = addr.wrapping_add(1);
        }
        StepResult::Ok
    }

    /// IN — print a prompt then read one character (same stall logic as GETC).
    fn trap_in(&mut self) -> StepResult {
        if self.input_queue.is_empty() {
            self.push_str("Input a character> ");
        }
        self.trap_getc()
    }

    /// PUTSP — write a packed (two chars per word) null-terminated string at mem[R0].
    fn trap_putsp(&mut self) -> StepResult {
        let mut addr = self.regs.gpr[0];
        loop {
            let word = self.mem.read(addr);
            if word == 0 {
                break;
            }
            let lo = (word & 0xFF) as u8;
            let hi = ((word >> 8) & 0xFF) as u8;
            if lo == 0 {
                break;
            }
            self.push_char(lo as char);
            if hi != 0 {
                self.push_char(hi as char);
            }
            addr = addr.wrapping_add(1);
        }
        StepResult::Ok
    }

    /// HALT — stop the machine.
    fn trap_halt(&mut self) -> StepResult {
        self.push_str("\n--- HALT ---\n");
        self.halted = true;
        StepResult::Halted
    }
}
