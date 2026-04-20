# LC-3 Simulator — Complete Implementation Plan

This document is the single source of truth for implementing an LC-3 TUI simulator
alongside the existing assembler. Reference this file in a future conversation to
build the entire simulator from scratch.

---

## 1. What Already Exists

The repo is `lc3-assembler` and exposes a public Rust library:

```
src/
  lib.rs          — re-exports all modules below
  lexer/          — tokeniser
  parser/         — token → AST
  first_pass/     — symbol table, address calculation
  encoder/        — AST → 16-bit machine code words
  listing.rs      — listing + .sym file generation
  diagnostic.rs   — rich error display
  preprocessor.rs — .INCLUDE expansion
  macro_expand.rs — .MACRO/.ENDM expansion
  main.rs         — lc3-assembler CLI binary
```

The assembler **library API** the simulator can call:

```rust
use lc3_assembler::{
    lexer::tokenize,
    parser::parse_lines,
    first_pass::first_pass,
    encoder::encode,
};
// Pipeline: tokenize → parse_lines → first_pass → encode
// encode() returns EncodeResult { machine_code: Vec<u16>, orig_address: u16, errors, warnings, line_infos }
```

The `.obj` binary format (what the simulator loads):
- Word 0: origin address (big-endian u16)
- Words 1..N: machine code (big-endian u16 each)

---

## 2. Repo Restructure — Cargo Workspace

Convert the repo into a workspace so `lc3-assembler` and `lc3-sim` are peer crates.

### New directory layout

```
lc3-assembler/               ← repo root (keep name)
├── Cargo.toml               ← workspace manifest (NEW)
├── LC3_SIMULATOR_PLAN.md    ← this file
├── .github/                 ← unchanged
├── assembler/               ← renamed from root-level src/
│   ├── Cargo.toml           ← package manifest (moved + renamed)
│   └── src/                 ← all existing src/ files move here
├── simulator/               ← NEW crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs          ← CLI entry point
│       ├── machine.rs       ← CPU execution engine
│       ├── memory.rs        ← 65 536-word address space + MMIO
│       ├── disasm.rs        ← u16 word → instruction string (for TUI)
│       ├── trap.rs          ← TRAP service routines (GETC, OUT, PUTS…)
│       └── tui/
│           ├── mod.rs       ← TUI entry, event loop
│           ├── app.rs       ← TUI application state
│           └── ui.rs        ← ratatui widget layout + rendering
└── tests/                   ← integration tests (shared, optional)
```

### Root `Cargo.toml` (workspace)

```toml
[workspace]
members = ["assembler", "simulator"]
resolver = "2"
```

### `assembler/Cargo.toml` (existing content, path adjusted)

```toml
[package]
name = "lc3-assembler"
version = "1.0.0"
edition = "2021"
rust-version = "1.70"

[[bin]]
name = "lc3-assembler"
path = "src/main.rs"

[lib]
name = "lc3_assembler"
path = "src/lib.rs"
```

### `simulator/Cargo.toml`

```toml
[package]
name = "lc3-sim"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "lc3-sim"
path = "src/main.rs"

[dependencies]
lc3-assembler = { path = "../assembler" }   # for .asm → .obj pipeline
ratatui = "0.28"
crossterm = "0.28"
```

---

## 3. LC-3 Architecture Reference

Everything the simulator must faithfully implement.

### 3.1 Memory

- 65 536 words of 16-bit memory (`[u16; 65536]`)
- Word-addressed (address 0x0000 – 0xFFFF)
- User programs conventionally start at 0x3000
- System/OS space: 0x0000 – 0x2FFF (trap vectors, interrupt vectors, OS code)
- User space: 0x3000 – 0xFDFF
- Device registers (MMIO): 0xFE00 – 0xFFFF

### 3.2 Registers

```
R0 – R7   general-purpose, 16-bit
PC        program counter, 16-bit
IR        instruction register, 16-bit (internal, not user-visible)
CC        condition codes: N (negative), Z (zero), P (positive) — only one set at a time
```

Rust representation:

```rust
pub struct Registers {
    pub gpr: [u16; 8],   // R0–R7
    pub pc:  u16,
    pub ir:  u16,
    pub cc:  CondCode,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CondCode { N, Z, P }
```

### 3.3 All 15 Opcodes

Bits [15:12] of every instruction word select the opcode.

| Bits  | Mnemonic | Notes |
|-------|----------|-------|
| 0000  | BR       | branch if condition matches CC |
| 0001  | ADD      | register or immediate mode |
| 0010  | LD       | load PC-relative |
| 0011  | ST       | store PC-relative |
| 0100  | JSR/JSRR | bit 11 = 1 → JSR (PC-relative), 0 → JSRR (register) |
| 0101  | AND      | register or immediate mode |
| 0110  | LDR      | load base+offset |
| 0111  | STR      | store base+offset |
| 1000  | RTI      | return from interrupt |
| 1001  | NOT      | bitwise complement |
| 1010  | LDI      | load indirect |
| 1011  | STI      | store indirect |
| 1100  | JMP/RET  | JMP BaseR; RET = JMP R7 |
| 1101  | (reserved) | treat as illegal instruction |
| 1110  | LEA      | load effective address |
| 1111  | TRAP     | system call (trapvect8 in bits [7:0]) |

### 3.4 Instruction Execution — Exact Algorithms

All PC-relative offsets are sign-extended from their field width.
After FETCH: PC has already been incremented (PC = old_PC + 1).

**Sign extension helper:**
```rust
fn sext(value: u16, bits: u8) -> u16 {
    let shift = 16 - bits;
    ((value as i16) << shift >> shift) as u16
}
```

**ADD:**
```
dr  = (ir >> 9) & 0x7
sr1 = (ir >> 6) & 0x7
if bit[5] == 0:
    sr2 = ir & 0x7
    result = R[sr1] + R[sr2]
else:
    imm5 = sext(ir & 0x1F, 5)
    result = R[sr1] + imm5
R[dr] = result; set_cc(result)
```

**AND:**
```
dr  = (ir >> 9) & 0x7
sr1 = (ir >> 6) & 0x7
if bit[5] == 0:
    result = R[sr1] & R[(ir & 0x7)]
else:
    result = R[sr1] & sext(ir & 0x1F, 5)
R[dr] = result; set_cc(result)
```

**NOT:**
```
dr = (ir >> 9) & 0x7
sr = (ir >> 6) & 0x7
R[dr] = !R[sr]; set_cc(R[dr])
```

**BR:**
```
n = (ir >> 11) & 1
z = (ir >> 10) & 1
p = (ir >>  9) & 1
offset9 = sext(ir & 0x1FF, 9)
if (n && CC==N) || (z && CC==Z) || (p && CC==P):
    PC = PC + offset9       // PC already incremented in fetch
```

**LD:**
```
dr      = (ir >> 9) & 0x7
offset9 = sext(ir & 0x1FF, 9)
R[dr] = mem[PC + offset9]; set_cc(R[dr])
```

**LDI:**
```
dr      = (ir >> 9) & 0x7
offset9 = sext(ir & 0x1FF, 9)
ptr = mem[PC + offset9]
R[dr] = mem[ptr]; set_cc(R[dr])
```

**LDR:**
```
dr      = (ir >> 9) & 0x7
base_r  = (ir >> 6) & 0x7
offset6 = sext(ir & 0x3F, 6)
R[dr] = mem[R[base_r] + offset6]; set_cc(R[dr])
```

**LEA:**
```
dr      = (ir >> 9) & 0x7
offset9 = sext(ir & 0x1FF, 9)
R[dr] = PC + offset9       // does NOT set CC
```

**ST:**
```
sr      = (ir >> 9) & 0x7
offset9 = sext(ir & 0x1FF, 9)
mem[PC + offset9] = R[sr]
```

**STI:**
```
sr      = (ir >> 9) & 0x7
offset9 = sext(ir & 0x1FF, 9)
ptr = mem[PC + offset9]
mem[ptr] = R[sr]
```

**STR:**
```
sr      = (ir >> 9) & 0x7
base_r  = (ir >> 6) & 0x7
offset6 = sext(ir & 0x3F, 6)
mem[R[base_r] + offset6] = R[sr]
```

**JMP / RET:**
```
base_r = (ir >> 6) & 0x7
PC = R[base_r]             // RET = JMP R7
```

**JSR:**
```
R[7] = PC                  // save return address
if bit[11] == 1:           // JSR (PC-relative)
    offset11 = sext(ir & 0x7FF, 11)
    PC = PC + offset11
else:                      // JSRR (register)
    base_r = (ir >> 6) & 0x7
    PC = R[base_r]
```

**TRAP:**
```
R[7] = PC
trapvect8 = ir & 0xFF
PC = mem[trapvect8]        // jump into trap handler
// In practice: dispatch to native Rust trap handler (see §3.5)
```

**RTI:**
```
PC = R[7]                  // simplified (no privilege mode in simulator)
```

**set_cc:**
```rust
fn set_cc(regs: &mut Registers, value: u16) {
    regs.cc = match (value as i16).cmp(&0) {
        Ordering::Less    => CondCode::N,
        Ordering::Equal   => CondCode::Z,
        Ordering::Greater => CondCode::P,
    };
}
```

### 3.5 TRAP Service Routines

The simulator intercepts TRAP directly in Rust (no OS in memory).

| Vector | Name  | Behaviour |
|--------|-------|-----------|
| x20    | GETC  | read one char from stdin (blocking); store in R0[7:0]; R0[15:8] = 0 |
| x21    | OUT   | write R0[7:0] as ASCII char to output buffer |
| x22    | PUTS  | write null-terminated string starting at mem[R0] to output buffer |
| x23    | IN    | print "Input a character> " prompt, then GETC |
| x24    | PUTSP | write packed (two chars per word) string at mem[R0] |
| x25    | HALT  | set `halted = true`; stop the run loop |
| other  | —     | log "unimplemented TRAP x{vec:02X}" to output buffer; halt |

Output buffer: a `Vec<String>` displayed in the TUI Output panel, appended per OUT/PUTS call.

For GETC in interactive TUI mode: disable raw-mode temporarily, read one byte from stdin,
re-enable raw-mode.

### 3.6 Memory-Mapped I/O (MMIO)

Handle these addresses specially in `memory.rs` read/write:

| Address | Register | Read behaviour | Write behaviour |
|---------|----------|----------------|-----------------|
| 0xFE00  | KBSR     | bit 15 = 1 always (keyboard always ready) | ignore |
| 0xFE02  | KBDR     | last key pressed (from input queue); clear queue | ignore |
| 0xFE04  | DSR      | bit 15 = 1 always (display always ready) | ignore |
| 0xFE06  | DDR      | undefined | append char to output buffer |
| 0xFFFE  | MCR      | current value | bit 15 = 0 halts machine |

Simplification: In the TUI simulator, KBDR reads block until a key is queued.
Maintain an `input_queue: VecDeque<u8>` in the Machine struct.

---

## 4. Data Structures

### `machine.rs`

```rust
pub struct Machine {
    pub regs:        Registers,
    pub mem:         Memory,
    pub halted:      bool,
    pub output_buf:  Vec<String>,    // lines printed by TRAP OUT/PUTS
    pub input_queue: VecDeque<u8>,   // keys waiting to be consumed by GETC/KBDR
    pub breakpoints: HashSet<u16>,   // addresses where execution pauses
    pub step_count:  u64,            // total instructions executed
}

impl Machine {
    pub fn load_obj(&mut self, data: &[u8]) -> Result<u16, LoadError>
    pub fn step(&mut self) -> StepResult        // execute one instruction
    pub fn run(&mut self) -> StepResult         // run until halt or breakpoint
    pub fn reset(&mut self)
}

pub enum StepResult {
    Ok,
    Halted,
    BreakpointHit(u16),
    IllegalInstruction(u16),
}
```

### `memory.rs`

```rust
pub struct Memory {
    words: [u16; 65536],
}

impl Memory {
    pub fn read(&self, addr: u16) -> u16          // handles MMIO reads
    pub fn write(&mut self, addr: u16, val: u16)  // handles MMIO writes + MCR halt
    pub fn load_image(&mut self, orig: u16, words: &[u16])
}
```

### `disasm.rs`

```rust
/// Disassemble one 16-bit word at `addr` into a human-readable string.
/// `sym_table` is optional — if provided, labels replace raw addresses.
pub fn disassemble(word: u16, addr: u16, sym_table: Option<&HashMap<u16, &str>>) -> String
```

Examples of output strings:
```
ADD R0, R1, R2
ADD R0, R1, #-1
LD  R0, x3010       ; or "LD R0, DATA" if symbol table provided
BRnz x3005
TRAP x25            ; or HALT
.FILL x0041         ; for data words that don't decode as valid instructions
```

---

## 5. TUI Design

### 5.1 Dependencies

```toml
ratatui  = "0.28"
crossterm = "0.28"
```

`crossterm` handles raw-mode terminal I/O; `ratatui` renders the frame.

### 5.2 Layout (80×24 minimum terminal)

```
┌─ LC-3 Simulator ────────────────────────────────────── RUNNING ─┐
│                                                                  │
├─ Registers ──────────┬─ Memory ─────────────────────────────────┤
│ R0  x0041  'A'       │ ► x3000  5021  ADD R0, R0, #1            │
│ R1  x0000            │   x3001  0E02  BRz x3004                 │
│ R2  x0000            │   x3002  1261  ADD R1, R1, #1            │
│ R3  x0000            │   x3003  0FF8  BRnzp x3000               │
│ R4  x0000            │   x3004  F025  HALT                      │
│ R5  x0000            │   x3005  0000  .FILL x0000               │
│ R6  x0000            │   x3006  0000  .FILL x0000               │
│ R7  x0000            │   x3007  0000  .FILL x0000               │
│                      │                                          │
│ PC  x3002            │                                          │
│ CC  P                │                                          │
│                      │                                          │
│ Breaks:              │                                          │
│   x3004              │                                          │
├─ Output ─────────────┴──────────────────────────────────────────┤
│ Hello from LC-3!                                                 │
│                                                                  │
├─ Command ────────────────────────────────────────────────────────┤
│ > _                           [s]tep [c]ont [b]reak [r]eset [q] │
└──────────────────────────────────────────────────────────────────┘
```

**Panel sizes (proportional):**
- Left column (Registers + Breaks): 22 chars wide, top 75% of height
- Right column (Memory): remaining width, same height
- Output: 20% of height, full width
- Command bar: 3 lines, full width (1 input + 1 hint + 1 border)

**Memory panel behaviour:**
- Shows 8–12 disassembled lines centred around PC
- Current PC line is prefixed with `►`
- Breakpoint lines are highlighted (red fg)
- Scroll with `↑`/`↓` when focused

### 5.3 Key Bindings

| Key         | Action |
|-------------|--------|
| `s`         | Single step |
| `c`         | Continue (run until halt/breakpoint) |
| `b <addr>`  | Toggle breakpoint (type in command bar) |
| `r`         | Reset machine (reload original .obj) |
| `g <addr>`  | Go to address (scroll memory panel) |
| `↑` / `↓`  | Scroll memory panel |
| `Tab`       | Switch focus between panels |
| `Esc`       | Cancel command input |
| `q`         | Quit |

### 5.4 TUI App State (`tui/app.rs`)

```rust
pub struct App {
    pub machine:      Machine,
    pub mode:         AppMode,
    pub mem_scroll:   u16,           // top address shown in memory panel
    pub cmd_input:    String,        // current text in command bar
    pub focus:        Panel,
    pub original_obj: Vec<u8>,       // kept for reset
    pub sym_table:    HashMap<u16, String>,  // optional, from .sym file
    pub status:       String,        // status line text (RUNNING/HALTED/BREAK)
}

pub enum AppMode { Normal, CommandInput }
pub enum Panel   { Memory, Output }
```

### 5.5 Event Loop (`tui/mod.rs`)

```rust
pub fn run(app: App) -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(k) => handle_key(&mut app, k),
                _ => {}
            }
        }
    }
}
```

---

## 6. Disassembler (`disasm.rs`)

Needed by the memory-view panel to show human-readable instructions.

```rust
pub fn disassemble(word: u16, addr: u16, syms: Option<&HashMap<u16, String>>) -> String {
    let op = word >> 12;
    match op {
        0b0001 => disasm_add(word),
        0b0101 => disasm_and(word),
        0b1001 => disasm_not(word),
        0b0010 => disasm_pc9("LD",  word, addr, syms),
        0b1010 => disasm_pc9("LDI", word, addr, syms),
        0b1110 => disasm_pc9("LEA", word, addr, syms),
        0b0011 => disasm_pc9("ST",  word, addr, syms),
        0b1011 => disasm_pc9("STI", word, addr, syms),
        0b0110 => disasm_base_off("LDR", word),
        0b0111 => disasm_base_off("STR", word),
        0b0000 => disasm_br(word, addr, syms),
        0b1100 => disasm_jmp(word),
        0b0100 => disasm_jsr(word, addr, syms),
        0b1111 => disasm_trap(word),
        0b1000 => "RTI".into(),
        _      => format!(".FILL x{:04X}", word),
    }
}
```

Each helper (`disasm_add`, etc.) extracts fields and formats a string.
For PC-relative instructions, compute `target = (addr + 1).wrapping_add(sext(offset, 9))`
and look it up in `syms`.

---

## 7. CLI Interface (`simulator/src/main.rs`)

```
USAGE:
  lc3-sim [OPTIONS] <input>

ARGS:
  <input>    Path to .obj file (or .asm — assembles then runs)

OPTIONS:
  -s, --symbols <file>   Load .sym file for label display in TUI
  --run                  Run non-interactively (print output, no TUI)
  --no-color             Disable ANSI colour
  -h, --help
  -V, --version

EXAMPLES:
  lc3-sim program.obj              # Open TUI debugger
  lc3-sim program.obj -s prog.sym  # TUI with symbol labels
  lc3-sim program.asm              # Assemble then open TUI
  lc3-sim program.obj --run        # Run headlessly, print output
```

**`.asm` auto-assembly path** (`main.rs`):

```rust
if path.ends_with(".asm") {
    let source = fs::read_to_string(&path)?;
    let lexed   = lc3_assembler::lexer::tokenize(&source);
    let parsed  = lc3_assembler::parser::parse_lines(&lexed.tokens);
    let first   = lc3_assembler::first_pass::first_pass(parsed.lines);
    let encoded = lc3_assembler::encoder::encode(&first);
    if !encoded.errors.is_empty() { /* print errors, exit(1) */ }
    // serialize encoded.machine_code + orig_address to a temp Vec<u8>
    obj_bytes = serialize_obj(encoded.orig_address, &encoded.machine_code);
} else {
    obj_bytes = fs::read(&path)?;
}
```

---

## 8. Integration with Assembler

The assembler is a **library** (`lc3_assembler`). The simulator references it via the
workspace path dependency. No changes needed to the assembler crate.

To extract the symbol table for the TUI, the user can either:
1. Pass a pre-generated `.sym` file with `-s` (generated by `lc3-assembler -s prog.sym`)
2. When assembling from `.asm`, extract from `first.symbol_table`:

```rust
let sym_table: HashMap<u16, String> = first
    .symbol_table
    .iter()
    .map(|(label, addr)| (addr, label.to_string()))
    .collect();
```

---

## 9. Implementation Phases

### Phase 1 — Headless Simulator (no TUI)

Files: `machine.rs`, `memory.rs`, `trap.rs`, `main.rs` (--run flag only)

Goals:
- Load `.obj` file into memory
- Execute full instruction set
- TRAP routines print to stdout
- `--run` flag runs to HALT and exits

Tests: run existing `tests/test_programs/*.asm`, assert output matches expected.

### Phase 2 — Disassembler

File: `disasm.rs`

Goals:
- Disassemble any `u16` word at any address
- Correct output for all 15 opcodes including edge cases (BRnzp, RET, HALT)

Tests: unit-test every opcode variant against known hex words.

### Phase 3 — TUI Debugger

Files: `tui/mod.rs`, `tui/app.rs`, `tui/ui.rs`

Goals:
- Registers panel, memory panel (disassembled), output panel, command bar
- Step, continue, breakpoints, reset, memory scroll
- `.sym` file support for label display

### Phase 4 — `.asm` Direct Run

File: `main.rs` (detect extension, call assembler pipeline)

Goals:
- `lc3-sim program.asm` assembles in-memory and opens TUI
- Errors from assembler displayed cleanly before TUI starts

---

## 10. Testing Strategy

### Unit tests (in each module)

- `memory.rs`: MMIO read/write, MCR halt, load_image
- `machine.rs`: every opcode, CC behaviour, wrapping arithmetic, all TRAPs
- `disasm.rs`: every opcode round-trip (encode known word → check string)

### Integration tests

Reuse `tests/test_programs/*.asm` — assemble them with the library, load into simulator,
run headlessly, assert `machine.output_buf` matches expected output.

Test programs that exercise:
- All arithmetic + CC transitions
- All load/store forms
- All branch forms
- JSR/RET call chain
- TRAP PUTS string output
- Edge cases: branch not taken, self-modifying memory write, wraparound addresses

### CI

Add `lc3-sim` binary to existing CI matrix — build + test on all three platforms.

---

## 11. Full File Manifest

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root |
| `assembler/Cargo.toml` | Assembler package (existing, path adjusted) |
| `assembler/src/` | All existing assembler source (unchanged) |
| `simulator/Cargo.toml` | Simulator package |
| `simulator/src/main.rs` | CLI: arg parsing, load .obj/.asm, launch TUI or --run |
| `simulator/src/machine.rs` | `Machine` struct, `step()`, `run()`, instruction dispatch |
| `simulator/src/memory.rs` | `Memory` struct, MMIO, `load_image()` |
| `simulator/src/trap.rs` | TRAP dispatch: GETC, OUT, PUTS, IN, PUTSP, HALT |
| `simulator/src/disasm.rs` | `disassemble(word, addr, syms)` → String |
| `simulator/src/tui/mod.rs` | Raw-mode setup, crossterm event loop |
| `simulator/src/tui/app.rs` | `App` state, key handling, command parsing |
| `simulator/src/tui/ui.rs` | ratatui `render()` — all widget layout |

---

## 12. Known Pitfalls

1. **PC increments before execution** — after FETCH, PC = old_PC + 1. All PC-relative
   offsets are computed from this already-incremented value. Easy to get wrong.

2. **GETC in raw-mode TUI** — when the TUI is running, stdin is in raw mode. GETC must
   temporarily restore cooked mode or use a non-blocking read + input queue fed by the
   TUI event loop.

3. **JSR vs JSRR** — bit 11 distinguishes them; the old base register value must be saved
   to a temp before overwriting R7, because JSRR base_r could be R7.

4. **LEA does NOT set CC** — unlike all other load instructions.

5. **RTI privilege** — full LC-3 has supervisor/user mode; for the simulator, implement
   RTI simply as `PC = R7` and skip privilege checks.

6. **Workspace CI** — update `.github/workflows/ci.yml` to build/test both workspace
   members: `cargo test --workspace`.

7. **Cargo.lock** — after workspace conversion, regenerate `Cargo.lock`. The MSRV job
   will need the same `rm -f Cargo.lock` trick for `lc3-sim`'s new dependencies.
