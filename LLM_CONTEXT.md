# LC-3 Assembler — LLM Context Document

> **Purpose:** This document gives an LLM (or any new contributor) everything needed to understand, modify, extend, or optimise the LC-3 assembler codebase. Read this before touching any code.

---

## Table of Contents

1. [What This Project Is](#1-what-this-project-is)
2. [LC-3 Architecture Primer](#2-lc-3-architecture-primer)
3. [Pipeline Overview](#3-pipeline-overview)
4. [File Map](#4-file-map)
5. [Module Deep Dives](#5-module-deep-dives)
6. [Key Design Decisions](#6-key-design-decisions)
7. [Code Conventions](#7-code-conventions)
8. [Remaining TODOs](#8-remaining-todos)
9. [How to Extend the Codebase](#9-how-to-extend-the-codebase)
10. [Test Strategy](#10-test-strategy)
11. [Error Model](#11-error-model)

---

## 1. What This Project Is

A **two-pass assembler** for the LC-3 educational computer architecture, written in Rust. It reads `.asm` source files and produces `.obj` binary files in the LC-3 object file format (big-endian, origin address first, then machine code words).

**Binary name:** `lc3-assembler`
**Crate name:** `lc3_assembler` (lib + bin in the same crate)
**MSRV:** Rust 1.70 (reason not documented — a future TODO)
**Repo:** `https://github.com/adhirajagarwala/lc3-assembler`

### Entry Points

- **Binary:** `src/main.rs` — CLI wrapper, parses args, runs pipeline, writes `.obj`
- **Library:** `src/lib.rs` — re-exports the four pipeline modules for embedding

---

## 2. LC-3 Architecture Primer

The LC-3 is a 16-bit educational CPU with:
- **8 general-purpose registers:** R0–R7 (R7 is the link register by convention)
- **16-bit address space:** 0x0000–0xFFFF
- **Big-endian** instruction encoding
- **Condition codes:** N (negative), Z (zero), P (positive) — set by any instruction that loads or computes
- **Trap mechanism:** `TRAP x20`–`TRAP x25` for OS services (GETC, OUT, PUTS, IN, PUTSP, HALT)

### Instruction Format (16 bits)

```
Bits 15:12  — Opcode (4 bits)
Bits 11:0   — Operand fields (vary by instruction)
```

### Object File Format

```
[2 bytes] Origin address (big-endian u16)
[2 bytes] First instruction word (big-endian u16)
[2 bytes] Second instruction word
...
```

### Key Addressing Modes

- **PC-relative (9-bit offset):** LD, LDI, LEA, ST, STI, BR — offset sign-extended to 16 bits, added to PC+1
- **Base+offset (6-bit offset):** LDR, STR — base register + sign-extended offset
- **JSR (11-bit offset):** PC-relative subroutine call
- **JSRR / JMP:** Register-indirect

### Assembler Directives

| Directive | Meaning |
|-----------|---------|
| `.ORIG addr` | Set origin address (must be first) |
| `.END` | Mark end of program (must be last) |
| `.FILL value/label` | Emit one word |
| `.BLKW n` | Allocate n zero-filled words |
| `.STRINGZ "text"` | Emit null-terminated string (one char per word) |

---

## 3. Pipeline Overview

```
Source (.asm)
    │
    ▼
┌─────────────┐
│   Lexer     │  tokenize(&source) → LexResult { tokens, errors }
│ src/lexer/  │  Bytes → stream of Token { kind, lexeme, span }
└──────┬──────┘
       │ tokens: Vec<Token>
       ▼
┌─────────────┐
│   Parser    │  parse_lines(&tokens) → ParseResult { lines, errors }
│ src/parser/ │  Tokens → Vec<SourceLine> (one per source line)
└──────┬──────┘
       │ lines: Vec<SourceLine>   (ownership transferred)
       ▼
┌──────────────┐
│  First Pass  │  first_pass(lines) → FirstPassResult { symbol_table, source_lines, orig_address, errors }
│ src/first_   │  Builds symbol table, validates .ORIG/.END, tracks location counter
│ pass/        │
└──────┬───────┘
       │ &FirstPassResult
       ▼
┌─────────────┐
│   Encoder   │  encode(&first_pass) → EncodeResult { machine_code, orig_address, errors }
│ src/encoder/│  AST + symbol table → Vec<u16> machine code words
└──────┬──────┘
       │
       ▼
Output (.obj)  — big-endian binary
```

**All errors accumulate** — the pipeline always runs all four stages. All four error vecs are collected and printed together. This means you get ALL errors in one run rather than failing at the first stage.

---

## 4. File Map

```
lc3-assembler/
├── src/
│   ├── lib.rs                      # Crate root: re-exports all 5 modules
│   ├── main.rs                     # CLI entry point
│   ├── error.rs                    # Span, AsmError, ErrorKind types
│   ├── lexer/
│   │   ├── mod.rs                  # tokenize() + LexResult + all lex_* functions
│   │   ├── cursor.rs               # Cursor<'a>: byte-slice iteration with line/col tracking
│   │   ├── token.rs                # Token, TokenKind, BrFlags
│   │   └── tests.rs                # Lexer unit tests
│   ├── parser/
│   │   ├── mod.rs                  # parse_lines() + ParseResult + all parse_* functions
│   │   ├── ast.rs                  # SourceLine, LineContent, Instruction enums
│   │   ├── macros.rs               # Declarative macros for parse_reg_reg_or_imm! etc.
│   │   └── tests.rs                # Parser unit tests
│   ├── first_pass/
│   │   ├── mod.rs                  # first_pass() + FirstPassResult + state machine
│   │   ├── symbol_table.rs         # SymbolTable (Vec<(String, u16)>)
│   │   └── tests.rs                # First-pass unit tests
│   └── encoder/
│       └── mod.rs                  # encode() + EncodeResult + Encoder struct + unit tests
├── tests/
│   └── integration_tests.rs        # End-to-end pipeline tests
├── tests/programs/
│   ├── hello_world.asm             # Full hello world program
│   ├── loop.asm                    # Loop example
│   └── errors.asm                  # Program designed to produce errors
├── Cargo.toml
├── Dockerfile
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── changes to be made.md           # Full audit document (53 items, 54 done)
└── LLM_CONTEXT.md                  # ← this file
```

---

## 5. Module Deep Dives

### 5.1 `src/error.rs` — Error Types

**Key types:**

```rust
pub struct Span {
    pub line: usize,  // 1-indexed
    pub col: usize,   // 1-indexed
}
// NOTE: byte-offset fields (start, end) were intentionally removed — they were
// computed but never read. If you add source-underline diagnostics, add them back.

pub struct AsmError {
    pub kind: ErrorKind,
    pub message: String,  // Human-readable description
    pub span: Span,
}

pub enum ErrorKind {
    // Lexer errors
    UnterminatedString, InvalidEscapeSequence, InvalidDecimalLiteral,
    InvalidHexLiteral, InvalidBinaryLiteral, InvalidRegister,
    UnknownDirective, UnexpectedCharacter,
    // Parser errors
    ExpectedOperand, ExpectedRegister, ExpectedComma,
    UnexpectedToken, TooManyOperands, TooFewOperands, InvalidOperandType,
    // First-pass errors
    DuplicateLabel, MissingOrig, MultipleOrig, MissingEnd,
    InvalidOrigAddress, InvalidBlkwCount, AddressOverflow,
    // Encoder errors
    UndefinedLabel, OffsetOutOfRange,
}
```

`AsmError` implements `Display` (prints `ERROR (line L:C): message`), `std::error::Error`, and has builder constructors for common patterns (`AsmError::undefined_label()`, `AsmError::duplicate_label()`, etc.).

`ErrorKind` implements `Display` with human-readable lowercase phrases (e.g. `"PC offset out of range"`).

---

### 5.2 `src/lexer/cursor.rs` — Cursor

```rust
pub struct Cursor<'a> {
    bytes: &'a [u8],  // Source as ASCII bytes (lifetime tied to source string)
    pos: usize,        // Current byte position (== char index for ASCII)
    line: usize,       // 1-indexed current line
    col: usize,        // 1-indexed current column
}
```

**Key methods:**
- `peek() -> Option<char>` — look without consuming
- `advance() -> Option<char>` — consume, update line/col tracking
- `is_at_end() -> bool`
- `current_pos() -> (usize, usize)` — returns `(line, col)` before consuming the next char
- `make_span(start_line, start_col) -> Span` — creates a Span anchored at saved position

**Usage pattern in lexer:**
```rust
let (sl, sc) = cursor.current_pos();   // save position BEFORE consuming token
// ... consume chars ...
span: cursor.make_span(sl, sc)         // span anchored at token's first char
```

---

### 5.3 `src/lexer/mod.rs` — Lexer

**Public API:**
```rust
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<AsmError>,
}
impl LexResult {
    pub fn has_errors(&self) -> bool { ... }
}

#[must_use]
pub fn tokenize(source: &str) -> LexResult
```

**How it works:**
- `tokenize()` creates a `Cursor`, runs `lex_token()` in a loop until EOF
- `lex_token()` dispatches on the current character to the appropriate `lex_*` function
- All `lex_*` functions take `(cursor: &mut Cursor, sl: usize, sc: usize)` — position already captured

**Token dispatch:**
| Input | Function |
|-------|----------|
| `;` | `lex_comment` |
| `"` | `lex_string` |
| `#` | `lex_decimal` |
| `x`/`X` | hex path in `lex_word` |
| `b`/`B` | binary path in `lex_word` |
| `.` | `lex_directive` |
| `\n` | `lex_newline` |
| `R`/`r` + digit | register in `lex_word` |
| letter | `lex_word` → keyword or Label |
| `,` | `Token { Comma }` |
| whitespace | skip |
| other | `UnexpectedCharacter` error |

**Numeric literal encoding:**
- Decimal `#n` → `NumDecimal(i32)` — parsed directly, range unchecked at lex time
- Hex `xNNNN` → `NumHex(i32)` — converted via `u16_to_twos_complement`: values > 0x7FFF become negative (`xFFFF` → `-1`)
- Binary `bNNNN` → `NumBinary(i32)` — same two's complement conversion

This means **all three numeric types carry `i32`**, and the sign may be negative for bit patterns that would be negative in 16-bit two's complement. Parser range checks must account for this.

**`lex_decimal` optimisation (no extra allocation):**
```rust
// sign is a String ("" or "-"), digits is a &str
sign.push_str(&digits);
let value = sign.parse::<i32>()...
// reuses the sign String's allocation rather than format!("{}{}", sign, digits)
```

---

### 5.4 `src/lexer/token.rs` — Tokens

```rust
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,  // Original source text
    pub span: Span,
}

pub enum TokenKind {
    // Opcodes: OpAdd, OpAnd, OpNot, OpBr(BrFlags), OpJmp, OpJsr, OpJsrr,
    //          OpLd, OpLdi, OpLdr, OpLea, OpSt, OpSti, OpStr, OpTrap, OpRti
    // Pseudo-ops: PseudoRet (JMP R7), PseudoGetc/Out/Puts/In/Putsp/Halt
    // Directives: DirOrig, DirEnd, DirFill, DirBlkw, DirStringz
    // Operands: Register(u8), NumDecimal(i32), NumHex(i32), NumBinary(i32)
    //           StringLiteral(String), Label(String)
    // Structural: Comma, Newline, Comment(String), Eof
}
```

**`BrFlags`:**
```rust
pub struct BrFlags { pub n: bool, pub z: bool, pub p: bool }
impl BrFlags {
    pub fn parse(s: &str) -> Option<Self>  // "BR" → all true, "BRnz" → n+z true
    pub fn as_u16(&self) -> u16            // 3-bit encoding: [N][Z][P]
}
```

`TokenKind::is_instruction_or_directive()` — used by the parser to check if a token after a label is a valid instruction start.

---

### 5.5 `src/parser/mod.rs` — Parser

**Public API:**
```rust
pub struct ParseResult {
    pub lines: Vec<SourceLine>,
    pub errors: Vec<AsmError>,
}
impl ParseResult {
    pub fn has_errors(&self) -> bool { ... }
}

#[must_use]
pub fn parse_lines(tokens: &[Token]) -> ParseResult
```

**How it works:**
- `parse_lines()` scans for `Newline`/`Eof` tokens to identify line boundaries, then calls `process_line(&tokens[line_start..i], ...)` for each slice — **zero cloning** of tokens
- `process_line()` strips the trailing `Comment` token using `position()` (short-circuits), then dispatches on the first non-label token
- `parse_content()` dispatches on `TokenKind` to the appropriate parser function

**Line structure rules:**
1. `[Label] [Instruction or Directive] [Operands...]`
2. `[Label]` alone → `LineContent::Empty` with label
3. No label, no instruction → `UnexpectedToken` error

**Helper functions (pub(crate) for macros):**
- `ensure_no_extra(tokens, expected_len)` — rejects extra tokens
- `expect_comma(tokens, idx, msg)` — asserts token[idx] is Comma
- `expect_register(tokens, idx, msg)` — asserts token[idx] is Register
- `expect_label(tokens, idx, msg)` — asserts token[idx] is Label
- `token_to_i32(token)` — extracts numeric value from any of the three numeric kinds
- `token_to_register(token)` — extracts `u8` register number
- `token_to_label(token)` — extracts `String` label name

---

### 5.6 `src/parser/macros.rs` — Parser Macros

Four declarative macros generate parser closures inline, massively reducing boilerplate:

| Macro | Instructions | Pattern |
|-------|-------------|---------|
| `parse_reg_reg_or_imm!` | ADD, AND | `OP DR, SR1, SR2` or `OP DR, SR1, #imm5` |
| `parse_reg_label!` | LD, LDI, LEA, ST, STI | `OP REG, LABEL` |
| `parse_reg_reg_imm!` | LDR, STR | `OP REG, BASE, #offset6` |
| `parse_single_reg!` | JMP, JSRR | `OP BASE_R` |
| `parse_single_label!` | JSR | `OP LABEL` |
| `parse_no_operands!` | RET, HALT, GETC, etc. | `OP` |

**Validation in macros:**
- `parse_reg_reg_or_imm!` validates `imm5` range: `!(-16..=15).contains(&imm)` → `InvalidOperandType`
- `parse_reg_reg_imm!` validates `offset6` range: `!(-32..=31).contains(&value)` → `InvalidOperandType`

**Note on format strings in macros:** `$name` is a `literal` metavariable so `{$name}` doesn't work inside format strings — these correctly use `"{} ...", $name` style.

---

### 5.7 `src/parser/ast.rs` — AST

```rust
pub struct SourceLine {
    pub label: Option<String>,   // Label defined on this line
    pub content: LineContent,
    pub line_number: usize,      // 1-indexed
    pub span: Span,              // Span of first token on the line
}

pub enum LineContent {
    Empty,
    Orig(u16),
    End,
    FillImmediate(i32),   // .FILL with numeric value (may be negative from two's complement)
    FillLabel(String),    // .FILL with label reference
    Blkw(u16),            // .BLKW count (always positive after parser validation)
    Stringz(String),      // .STRINGZ text (escape sequences already resolved)
    Instruction(Instruction),
}

impl LineContent {
    pub fn word_count(&self) -> u32  // How many 16-bit words this emits
}
```

`Instruction` enum has 27 variants covering all LC-3 instructions. Each variant explicitly names its operands:
- Registers: `u8` (0–7)
- Immediates: `i16`
- Labels: `String` (resolved to offsets in encoder)
- Trap vectors: `u8`

---

### 5.8 `src/first_pass/mod.rs` — First Pass

**Public API:**
```rust
pub struct FirstPassResult {
    pub symbol_table: SymbolTable,
    pub source_lines: Vec<SourceLine>,  // owned — no clone needed
    pub orig_address: u16,
    pub errors: Vec<AsmError>,
}
impl FirstPassResult {
    pub fn has_errors(&self) -> bool { ... }
}

#[must_use]
pub fn first_pass(lines: Vec<SourceLine>) -> FirstPassResult
// Takes OWNERSHIP to avoid cloning the AST
```

**State machine (3 states):**
```
WaitingForOrig ──(.ORIG found)──► Processing ──(.END found)──► AfterEnd
     │                                │
     │ (non-empty line)                └── any other line: record labels,
     └── MissingOrig error                  advance location counter
```

**What it does per line (in Processing state):**
1. If line has a label → call `record_label()` → check for duplicates
2. Match on `line.content`:
   - `Orig(_)` → `MultipleOrig` error
   - `End` → transition to `AfterEnd`
   - `Blkw(0)` → `InvalidBlkwCount` error (belt-and-suspenders, parser also catches this)
   - Everything else → no special handling
3. `location_counter += line.content.word_count()` — check for 16-bit overflow

After loop: emit `MissingOrig` if still in `WaitingForOrig`, `MissingEnd` if not in `AfterEnd`.

**`record_label` — single lookup:**
```rust
if let Some(first_addr) = table.get(label) {
    errors.push(AsmError::duplicate_label(...));
} else {
    table.insert(label.to_string(), address);
}
```

---

### 5.9 `src/first_pass/symbol_table.rs` — Symbol Table

```rust
pub struct SymbolTable {
    entries: Vec<(String, u16)>,  // insertion-ordered label→address pairs
}
```

**Design rationale:** Old implementation used `HashMap<String, u16>` + `Vec<String>` for ordering, storing each label name twice. New implementation uses a single `Vec<(String, u16)>`. O(n) lookup is fine for LC-3 (typically < 50 labels). No external dependencies.

**Methods:**
- `new() / Default`
- `insert(label, addr)` — updates if exists, appends if new
- `get(label) -> Option<u16>` — linear scan
- `len() / is_empty()`
- `iter() -> impl Iterator<Item = (&str, u16)>` — used by encoder
- `print_table()` — prints a formatted table to stdout (used in CLI success output)

---

### 5.10 `src/encoder/mod.rs` — Encoder

**Public API:**
```rust
pub struct EncodeResult {
    pub machine_code: Vec<u16>,
    pub orig_address: u16,
    pub errors: Vec<AsmError>,
}
impl EncodeResult {
    pub fn has_errors(&self) -> bool { ... }
}

#[must_use]
pub fn encode(first_pass: &FirstPassResult) -> EncodeResult
```

**Opcode constants (bits 15:12):**
```rust
const OP_ADD: u16 = 0b0001;   const OP_AND: u16 = 0b0101;
const OP_NOT: u16 = 0b1001;   const OP_LD:  u16 = 0b0010;
const OP_LDI: u16 = 0b1010;   const OP_LEA: u16 = 0b1110;
const OP_ST:  u16 = 0b0011;   const OP_STI: u16 = 0b1011;
const OP_LDR: u16 = 0b0110;   const OP_STR: u16 = 0b0111;
const OP_BR:  u16 = 0b0000;   const OP_JMP: u16 = 0b1100;
const OP_JSR: u16 = 0b0100;   const OP_TRAP:u16 = 0b1111;
const OP_RTI: u16 = 0b1000;
// Pre-shifted TRAP words:
const TRAP_GETC: u16 = (OP_TRAP << 12) | 0x20;  // etc through TRAP_HALT
```

**Encoder struct (private):**
```rust
struct Encoder<'a> {
    symbol_table: &'a SymbolTable,
    machine_code: Vec<u16>,
    orig_address: u16,
    current_address: u16,   // tracks where we are for PC-relative offset calculation
    errors: Vec<AsmError>,
}
```

**PC-relative offset calculation:**
```rust
fn resolve_label(&mut self, label: &str, bits: u32, span: Span) -> u16 {
    let target = self.symbol_table.get(label)
        .ok_or_else(/* UndefinedLabel error */)?;
    let offset = (target as i32) - (self.current_address as i32 + 1);
    // +1 because PC has already advanced past the current instruction
    let (min_offset, max_offset) = (-(1 << (bits-1)), (1 << (bits-1)) - 1);
    // Range check → OffsetOutOfRange error if out of range
    sign_extend(offset as u16, bits)
}
```

**`sign_extend` is `const fn`:**
```rust
const fn sign_extend(value: u16, bits: u32) -> u16 {
    let shift = 16 - bits;
    ((value << shift) as i16 >> shift) as u16
}
```

---

### 5.11 `src/main.rs` — CLI

```
lc3-assembler <input.asm> [-o output.obj]
              --help / -h
              --version / -V
```

Flow:
1. Parse args
2. `fs::read_to_string(input_file)` — bail on IO error
3. Run all four pipeline stages
4. Collect errors from all four results, print all of them
5. Exit 1 if any errors
6. Print symbol table, write `.obj` file
7. Print success summary (origin, size, filenames)

**`write_obj_file`** pre-allocates a `Vec<u8>` buffer for `(1 + code.len()) * 2` bytes and issues a single `fs::write` syscall. Origin address is written first as big-endian u16, then all machine code words as big-endian u16.

---

## 6. Key Design Decisions

### 6.1 Why a library + binary in one crate?

`src/lib.rs` exposes all pipeline modules publicly so the assembler can be embedded in other tools (simulators, IDEs, etc.) without shell-out. The binary is just a thin CLI wrapper.

### 6.2 Why accumulate errors instead of failing fast?

Users benefit from seeing all errors in one pass (like modern compilers). All four stages always run. This means downstream stages might see partially-valid input (e.g., the encoder may encounter undefined labels that were missing because the first pass errored). Each stage handles missing data gracefully by emitting a sentinel value (usually `0`) and recording an error.

### 6.3 Why `Vec<(String, u16)>` instead of `HashMap` for the symbol table?

- Eliminates string duplication (old code stored each label name in both HashMap keys and a separate `Vec<String>` for ordering)
- O(n) lookup is negligible for LC-3 programs (< 50 labels is typical)
- Preserves insertion order without extra bookkeeping
- Zero external dependencies

### 6.4 Why does the parser take `&[Token]` but first_pass takes `Vec<SourceLine>`?

The parser borrows the token list (it doesn't need to own it after producing the AST). The first pass takes **ownership** of the `Vec<SourceLine>` and stores it in `FirstPassResult.source_lines` so the encoder can iterate it without cloning.

### 6.5 Why no `start`/`end` byte offsets in `Span`?

They were computed on every token but never read anywhere. Removing them eliminated wasted computation and storage in every `make_span` call. If source-underline diagnostics are added later (e.g., `^^^^` under the bad token), `start`/`end` should be restored to `Span` and `current_pos()` should return a 3-tuple again.

### 6.6 Why `DuplicateLabel` on re-insertion vs. silent update?

The `SymbolTable::insert` silently updates on re-insert (the Vec-based implementation does this). But `record_label` in the first pass catches duplicates by checking `table.get()` first. The separation exists because `insert` is also used for legitimate updates (e.g., if the assembler is ever extended to allow label reassignment), while `record_label` enforces the "label defined only once" invariant.

### 6.7 Why declarative macros for the parser?

The parser originally had 606 lines with many identical pattern repetitions for LD/LDI/LEA/ST/STI (all `REG, LABEL`) and ADD/AND (both `REG, REG, REG/IMM`). The macros cut this to ~450 lines with no runtime overhead. The generated closures are monomorphised identically to handwritten code.

---

## 7. Code Conventions

### Format strings

Use captured variable syntax throughout (Rust 1.58+):
```rust
format!("Error at {line}:{col}: {msg}")  // ✅
format!("Error at {}:{}: {}", line, col, msg)  // ❌ old style
```

Exception: `env!("CARGO_PKG_VERSION")` and `$name` macro metavariables still require `{}` style since they're not variable identifiers.

### `#[must_use]`

All four pipeline functions (`tokenize`, `parse_lines`, `first_pass`, `encode`) and all four `has_errors()` methods carry `#[must_use]`.

### Module visibility

- Parser helpers: `pub(crate)` — accessible from `macros.rs` but not the public API
- Everything else in `src/`: `pub` or private as appropriate
- No `pub(super)` patterns

### Error construction

Prefer the builder methods on `AsmError`:
```rust
AsmError::undefined_label(label, span)         // ✅
AsmError::new(ErrorKind::UndefinedLabel, ..., span)  // ✅ for custom messages
AsmError { kind: ..., message: ..., span }     // ⚠️ struct literal — only if no builder exists
```

### No `unwrap()` in production code

The pipeline never panics on bad input. All error cases record an `AsmError` and emit a sentinel value (typically `0`). The only `unwrap_or_else` is in `main.rs` for `fs::read_to_string` which terminates the process with a user-facing message.

### Numeric literals

Use hex for addresses (`0x3000`), binary for bit patterns (`0b0001_0000`), decimal for counts. Named constants (`OP_ADD`, `TRAP_HALT`) instead of raw numbers in encoder.

---

## 8. Remaining TODOs

### 8a. CI / Infrastructure (not core code changes)

| Item | Description |
|------|-------------|
| **7.1** | `release.yml`: Replace deprecated `actions/create-release@v1` + `actions/upload-release-asset@v1` with `softprops/action-gh-release@v2` |
| **7.2** | `ci.yml`: Replace unmaintained `actions-rs/audit-check@v1` with `cargo install cargo-audit && cargo audit` or `rustsec/audit-check` |
| **7.3** | `Dockerfile`: Change `as` → `AS`; update from pinned `rust:1.75` to `rust:latest` or `rust:1.82` |
| **7.4** | `Dockerfile`: Remove `ca-certificates` (binary never makes HTTPS calls) |
| **7.6** | `Cargo.toml`: Add comment explaining why MSRV is 1.70 (what feature requires it?) |
| **7.7** | `ci.yml`: Update `codecov/codecov-action@v3` → `v4` |
| **7.8** | `.gitignore`: Remove `Cargo.lock` from gitignore (binaries should commit it) |

### 8b. Feature Gaps (require new code)

| Item | Description | Effort |
|------|-------------|--------|
| **8.1** | Label shadowing reserved words: add `LabelIsReservedWord` back to `ErrorKind`; check in `record_label()` that label name is not a keyword | Low |
| **8.4** | Listing file (`-l` flag): after encoding, emit a `.lst` file with columns: `address | machine_code | source` | Medium |
| **8.5** | Numeric offsets for PC-relative: `LD R0, #5` should mean `offset = 5`. Requires parser to accept `NumDecimal`/`NumHex` where a label is expected | Low |
| **8.6** | Warning system: add `AsmWarning` type, a `warnings: Vec<AsmWarning>` field to result types; implement unused-label and unreachable-code warnings | Medium |
| **8.9** | `.STRINGZ` Unicode guard: in encoder, add `if ch as u32 > 0xFFFF` check → error | Low |
| **8.10** | stdin/stdout: accept `-` as input filename, pipe output to stdout with `-` as output | Low |
| **8.2** | `.INCLUDE` directive: requires a pre-processing pass before lexing | High |
| **8.3** | Macro system (`.MACRO`/`.ENDM`): significant new module | High |
| **8.8** | Octal literals (`o17`): add a `NumOctal(i32)` token variant, lex path similar to hex | Low |

---

## 9. How to Extend the Codebase

### Adding a new instruction

1. **`src/lexer/token.rs`:** Add a new `TokenKind` variant (e.g., `OpFoo`)
2. **`src/lexer/mod.rs`:** Add a match arm in `lex_word()` mapping the uppercase string to the new variant
3. **`src/parser/ast.rs`:** Add a new `Instruction` variant with operand fields
4. **`src/parser/mod.rs`:** Add a match arm in `parse_content()`. If it fits a macro pattern, use the macro; otherwise write a `parse_foo()` helper
5. **`src/encoder/mod.rs`:** Add match arm in `encode_instruction()`, use the named opcode constants
6. **Tests:** Add unit tests in `parser/tests.rs` and `encoder/mod.rs` test module; add integration test if it represents a significant scenario

### Adding a new directive

Same flow as above, but:
- Token: add `DirFoo` to `TokenKind`
- Lexer: add match arm in `lex_directive()` for `.FOO` (case-insensitive via `to_uppercase`)
- AST: add `Foo(...)` variant to `LineContent`; update `word_count()` if it emits words
- Parser: add `parse_foo()` called from `parse_content()`
- First pass: add any special handling needed in `first_pass()` (e.g., if the directive changes the location counter in a non-standard way)
- Encoder: add match arm in `encode_line()`

### Adding a new error kind

1. Add a variant to `ErrorKind` in `error.rs`
2. Add a `Display` arm in the `match` block in `impl Display for ErrorKind`
3. Optionally add a builder method on `AsmError` if it's used in many places

### Adding a new result field

Add the field to the relevant `*Result` struct. All four result types are concrete structs with `pub` fields — there's no trait barrier to adding fields.

### Source-underline diagnostics (restoring `Span.start/end`)

1. Add `pub start: usize, pub end: usize` back to `Span`
2. Change `Cursor::current_pos()` to return `(usize, usize, usize)` → `(byte_pos, line, col)`
3. Change `Cursor::make_span(start_byte, start_line, start_col) -> Span`
4. Update all `lex_*` functions to capture and pass `sb` (start byte) again
5. Update `AsmError::fmt` to also print the source line with `^` underline — this requires the original source to be available at display time, which means either storing it in `AsmError` or passing it separately

---

## 10. Test Strategy

### Test counts (as of session 6)

| Location | Count | Coverage |
|----------|-------|---------|
| `src/lexer/tests.rs` | ~20 | Numeric literals, strings, opcodes, directives, edge cases |
| `src/parser/tests.rs` | 32 | All instruction forms, all directives, error paths |
| `src/first_pass/tests.rs` | 14 | Symbol table, .ORIG/.END, duplicate labels, overflow, address tracking |
| `src/encoder/mod.rs` (inline) | 43 | All instructions, directives, PC offsets, errors, edge cases |
| `tests/integration_tests.rs` | ~25 | Full pipeline, error accumulation, test programs |
| **Total** | **~134** | |

### Test helper pattern (integration tests)

```rust
fn run_pipeline(source: &str) -> EncodeResult {
    let lexed = tokenize(source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(parsed.lines);
    encode(&first)
}

fn run_full_pipeline(source: &str) -> (LexResult, ParseResult, FirstPassResult, EncodeResult) {
    // wraps run_pipeline, returns all four results for error inspection
}
```

### Encoder test helper

```rust
fn build_first_pass(orig: u16, contents: Vec<LineContent>, symbols: SymbolTable) -> FirstPassResult
// Creates a minimal FirstPassResult with dummy spans for encoder unit tests
```

### Assertion patterns

```rust
assert!(result.errors.is_empty());                          // expect success
assert!(result.errors.iter().any(|e| e.kind == ErrorKind::Foo));  // expect specific error
assert_eq!(result.machine_code[0], 0x1234);                // expect specific encoding
```

---

## 11. Error Model

### Error accumulation

The pipeline never aborts. Downstream stages receive potentially invalid data and handle it gracefully:
- Parser encounters broken tokens → records error, emits `Empty` line, continues
- Encoder encounters undefined label → records `UndefinedLabel` error, emits `0x0000`, continues

### Error display format

```
ERROR (line 5:3): Duplicate label 'LOOP' (first defined at x3002)
```

(`line:col` is 1-indexed, pointing to the start of the offending token)

### When the binary exits non-zero

```rust
if !all_errors.is_empty() {
    eprintln!("\n❌ Assembly failed with {} error(s)", all_errors.len());
    std::process::exit(1);
}
```

Errors from all four stages are concatenated in pipeline order before printing.

---

*Generated after session 6. Covers all changes through the complete audit + optimisation pass.*
*Cross-reference: `changes to be made.md` has the 53-item audit history with before/after for each change.*
