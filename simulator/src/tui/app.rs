use std::collections::HashMap;

use crate::machine::{Machine, StepResult};

#[derive(Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    CommandInput,
}

pub struct App {
    pub machine: Machine,
    pub mode: AppMode,
    /// Current text typed in the command bar.
    pub cmd_input: String,
    /// Top address shown in the memory panel.
    pub mem_scroll: u16,
    /// When true the machine runs automatically each frame.
    pub running: bool,
    /// Status line text shown in the header.
    pub status: String,
    /// Original raw .obj bytes kept for the reset command.
    pub original_obj: Vec<u8>,
    /// address → label name (loaded from .sym file or assembler pass).
    pub sym_table: HashMap<u16, String>,
    /// Whether the user has quit.
    pub should_quit: bool,
}

impl App {
    pub fn new(obj: Vec<u8>, sym_table: HashMap<u16, String>) -> Result<Self, String> {
        let mut machine = Machine::new();
        machine.load_obj(&obj)?;
        let pc = machine.regs.pc;
        Ok(Self {
            machine,
            mode: AppMode::Normal,
            cmd_input: String::new(),
            mem_scroll: pc,
            running: false,
            status: "Ready".into(),
            original_obj: obj,
            sym_table,
            should_quit: false,
        })
    }

    // ── Per-frame tick ────────────────────────────────────────────────────────

    /// Called once per render frame.  If `running`, execute up to 5 000 steps.
    pub fn tick(&mut self) {
        if !self.running || self.machine.halted || self.machine.waiting_for_input {
            return;
        }
        match self.machine.run_steps(5_000) {
            StepResult::Halted => {
                self.running = false;
                self.status = "HALTED".into();
                self.sync_scroll();
            }
            StepResult::BreakpointHit(addr) => {
                self.running = false;
                self.status = format!("BREAK @ x{addr:04X}");
                self.sync_scroll();
            }
            StepResult::IllegalInstruction(ir) => {
                self.running = false;
                self.status = format!("ILLEGAL x{ir:04X}");
                self.sync_scroll();
            }
            StepResult::Ok => {
                self.status = "RUNNING".into();
            }
        }
    }

    // ── Key handling ──────────────────────────────────────────────────────────

    /// Returns `false` when the user requests quit.
    pub fn handle_char(&mut self, ch: char) {
        match self.mode {
            AppMode::CommandInput => {
                self.cmd_input.push(ch);
            }
            AppMode::Normal => {
                if self.machine.waiting_for_input {
                    // Feed the character to the machine's keyboard queue.
                    self.machine.input_queue.push_back(ch as u8);
                    return;
                }
                match ch {
                    's' | 'S' => self.do_step(),
                    'c' | 'C' => self.do_continue(),
                    'p' | 'P' => self.do_pause(),
                    'r' | 'R' => self.do_reset(),
                    'b' | 'B' => self.enter_cmd("b "),
                    'g' | 'G' => self.enter_cmd("g "),
                    'q' | 'Q' => self.should_quit = true,
                    _ => {}
                }
            }
        }
    }

    pub fn handle_enter(&mut self) {
        match self.mode {
            AppMode::CommandInput => {
                let cmd = std::mem::take(&mut self.cmd_input);
                self.mode = AppMode::Normal;
                self.execute_command(cmd.trim().to_string());
            }
            AppMode::Normal => {
                if self.machine.waiting_for_input {
                    self.machine.input_queue.push_back(b'\n');
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.mode == AppMode::CommandInput {
            self.cmd_input.pop();
        }
    }

    pub fn handle_escape(&mut self) {
        if self.mode == AppMode::CommandInput {
            self.cmd_input.clear();
            self.mode = AppMode::Normal;
        }
    }

    pub fn scroll_up(&mut self) {
        self.mem_scroll = self.mem_scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.mem_scroll = self.mem_scroll.wrapping_add(1);
    }

    // ── Actions ───────────────────────────────────────────────────────────────

    fn do_step(&mut self) {
        if self.machine.halted {
            self.status = "HALTED — press R to reset".into();
            return;
        }
        match self.machine.step() {
            StepResult::Ok => {
                self.status = format!("Stepped  PC=x{:04X}", self.machine.regs.pc);
            }
            StepResult::Halted => {
                self.status = "HALTED".into();
            }
            StepResult::BreakpointHit(a) => {
                self.status = format!("BREAK @ x{a:04X}");
            }
            StepResult::IllegalInstruction(ir) => {
                self.status = format!("ILLEGAL x{ir:04X}");
            }
        }
        self.sync_scroll();
    }

    fn do_continue(&mut self) {
        if self.machine.halted {
            self.status = "HALTED — press R to reset".into();
            return;
        }
        self.running = true;
        self.status = "RUNNING".into();
    }

    fn do_pause(&mut self) {
        self.running = false;
        self.status = format!("PAUSED  PC=x{:04X}", self.machine.regs.pc);
    }

    fn do_reset(&mut self) {
        let obj = self.original_obj.clone();
        let syms = self.sym_table.clone();
        match App::new(obj, syms) {
            Ok(fresh) => {
                let scroll = fresh.machine.regs.pc;
                *self = fresh;
                self.mem_scroll = scroll;
                self.status = "RESET".into();
            }
            Err(e) => {
                self.status = format!("Reset failed: {e}");
            }
        }
    }

    fn enter_cmd(&mut self, prefix: &str) {
        self.cmd_input = prefix.into();
        self.mode = AppMode::CommandInput;
    }

    fn execute_command(&mut self, cmd: String) {
        if let Some(rest) = cmd.strip_prefix("b ").or_else(|| cmd.strip_prefix("b")) {
            match parse_addr(rest.trim()) {
                Some(addr) => {
                    if self.machine.breakpoints.contains(&addr) {
                        self.machine.breakpoints.remove(&addr);
                        self.status = format!("Breakpoint removed: x{addr:04X}");
                    } else {
                        self.machine.breakpoints.insert(addr);
                        self.status = format!("Breakpoint set: x{addr:04X}");
                    }
                }
                None => self.status = format!("Bad address: '{}'", cmd),
            }
        } else if let Some(rest) = cmd.strip_prefix("g ").or_else(|| cmd.strip_prefix("g")) {
            match parse_addr(rest.trim()) {
                Some(addr) => {
                    self.mem_scroll = addr;
                    self.status = format!("Scrolled to x{addr:04X}");
                }
                None => self.status = format!("Bad address: '{}'", cmd),
            }
        }
    }

    /// Keep the memory panel centred a few lines before PC.
    fn sync_scroll(&mut self) {
        let pc = self.machine.regs.pc;
        self.mem_scroll = pc.saturating_sub(3);
    }
}

fn parse_addr(s: &str) -> Option<u16> {
    if s.is_empty() {
        return None;
    }
    // Accept: x3000  0x3000  3000 (pure hex)  12345 (decimal)
    if let Some(hex) = s.strip_prefix('x').or_else(|| s.strip_prefix("0x")) {
        u16::from_str_radix(hex, 16).ok()
    } else if s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() == 4 {
        u16::from_str_radix(s, 16).ok()
    } else {
        s.parse::<u16>().ok()
    }
}
