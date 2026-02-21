# Changes To Be Made

A comprehensive audit of every file in the LC-3 assembler codebase, categorized by type. Each item references exact file paths and line numbers.

---

## Table of Contents

1. [Performance Improvements](#1-performance-improvements)
2. [Dead / Unnecessary Code](#2-dead--unnecessary-code)
3. [Redundant Code & Simplifications](#3-redundant-code--simplifications)
4. [Bug Risks & Correctness Issues](#4-bug-risks--correctness-issues)
5. [Code Quality & Maintainability](#5-code-quality--maintainability)
6. [Test Gaps & Weaknesses](#6-test-gaps--weaknesses)
7. [CI / Docker / Infrastructure Issues](#7-ci--docker--infrastructure-issues)
8. [Feature Gaps](#8-feature-gaps)

---

## 1. Performance Improvements

### 1.1 ~~Unnecessary `String` cloning in comment tokens~~ DONE

**File:** `src/lexer/mod.rs` — Replaced `format!(";{}", text)` with pre-allocated `String::with_capacity` + `push(';')` + `push_str`. `text` moved directly into `Comment` variant. Zero format overhead, zero clones.

---

### 1.2 ~~`Vec<char>` allocation in Cursor is wasteful for ASCII-only input~~ DONE

**File:** `src/lexer/cursor.rs` — **Rewritten.** Cursor now holds `&[u8]` byte-slice with lifetime. Zero allocation.

```rust
chars: source.chars().collect(),
```

LC-3 assembly is strictly ASCII. Collecting into `Vec<char>` quadruples memory usage (4 bytes per char vs 1 byte per ASCII char). Consider working directly on byte slices (`&[u8]`) instead, with an upfront ASCII validation check. This eliminates the allocation entirely and improves cache locality.

---

### 1.3 ~~Repeated `format!` allocations in parser macros for error messages~~ DONE

**File:** `src/parser/macros.rs` — **Rewritten.** Static messages use `concat!()` at compile time. Only runtime values still use `format!`.

```rust
message: format!("{} requires 3 operands: {} DR, SR1, SR2/imm5", $name, $name),
```

Every time any `ADD` or `AND` instruction is parsed with an error, these `format!` calls allocate a new `String`. Since `$name` is a string literal known at compile time, these should be `concat!` or `const` strings where possible. Where `format!` is unavoidable, consider using `Cow<'static, str>` for the message field in `AsmError` so successful paths allocate nothing.

---

### 1.4 ~~`lines.to_vec()` clones the entire AST unnecessarily~~ DONE

**File:** `src/first_pass/mod.rs` — Changed `first_pass` to take ownership (`Vec<SourceLine>`) instead of `&[SourceLine]`. Zero cloning.

---

### 1.5 ~~Token cloning on every non-newline, non-EOF token in parser~~ DONE

**File:** `src/parser/mod.rs` — **Rewritten.** `parse_lines()` now tracks `line_start` index and passes `&tokens[line_start..i]` slices to `process_line`. Zero token cloning.

---

### 1.6 `write_obj_file` issues one syscall per u16 word

**File:** `src/main.rs:95-97`

```rust
for &word in code {
    file.write_all(&word.to_be_bytes())?;
}
```

Each 2-byte word triggers a separate `write_all` syscall. For a program with 1000 words, that's 1001 syscalls. Pre-allocate a `Vec<u8>` buffer, write all words into it, then issue a single `write_all`.

---

### 1.7 ~~`SymbolTable` uses `HashMap` + `Vec` when `IndexMap` would suffice~~ DONE

**File:** `src/first_pass/symbol_table.rs` — **Rewritten.** Replaced `HashMap<String, u16> + Vec<String>` with a single `Vec<(String, u16)>`. Each label stored exactly once. O(n) linear scan is fine for LC-3 programs (<50 labels). Zero external dependencies.

---

### 1.8 `sign_extend` is misnamed and could be `const fn`

**File:** `src/encoder/mod.rs:270-273`

```rust
fn sign_extend(value: i16, bits: u8) -> u16 {
    let mask = (1 << bits) - 1;
    (value as u16) & mask
}
```

This function truncates, not sign-extends (the doc comment admits this). It should be `const fn` since it does pure arithmetic, enabling compile-time evaluation for constant inputs.

---

## 2. Dead / Unnecessary Code

### 2.1 `PseudoRet` comment is wrong -- it says "TRAP x25 (HALT)"

**File:** `src/lexer/token.rs:44`

```rust
PseudoRet,   // TRAP x25 (HALT)
```

This comment is incorrect. `RET` is `JMP R7` (opcode `0xC1C0`), not a TRAP instruction. It's not dead code, but the comment is misleading and should be fixed to `// JMP R7 (return from subroutine)`.

---

### 2.2 ~~`peek_next()` on Cursor is never used anywhere~~ DONE

Removed during Cursor rewrite (1.2). The new byte-slice Cursor does not include `peek_next`.

---

### 2.3 `Span.start` and `Span.end` are never used for anything meaningful

**File:** `src/error.rs:3-4`

```rust
pub start: usize,
pub end: usize,
```

`Span` stores `start` (byte offset) and `end` (byte offset), but these are never used anywhere -- the `Display` impl for `AsmError` only prints `line` and `col`. The `start`/`end` fields are computed and stored but never read. They add wasted computation in every `make_span` call throughout the lexer. Either remove them or use them (e.g., for source context in error messages, underline the problematic token).

---

### 2.4 `ErrorKind::OrigNotFirst` and `ErrorKind::LabelIsReservedWord` are never constructed

**File:** `src/error.rs:97,102`

```rust
OrigNotFirst,
...
LabelIsReservedWord,
```

These error variants exist in the `ErrorKind` enum but are never constructed anywhere in the codebase. They are dead variants. Either implement the checks that would produce these errors, or remove the variants.

- `OrigNotFirst`: The first pass handles this case via `MissingOrig` instead.
- `LabelIsReservedWord`: No check exists to prevent labels like `ADD` or `HALT`.

---

### 2.5 ~~`errors.asm` test program is never used in any test~~ DONE

Added `errors_asm_file_produces_errors` integration test that runs the full pipeline on `errors.asm` and asserts errors are produced.

---

### 2.6 ~~`loop.asm` test program is never used in any test~~ DONE

Added `loop_program` and `encode_loop_program` integration tests.

---

### 2.7 Excessive markdown documentation files

The following files exist but provide questionable value and overlap heavily:

- `SESSION_REPORT.md` -- Internal development session notes, not useful to users
- `CI_FIX_EXPLANATION.md` -- One-time debugging explanation, not useful long-term
- `IMPROVEMENTS.md` -- Duplicates information now in the codebase itself
- `FUTURE_IMPROVEMENTS.md` -- 1880+ lines of speculative feature proposals, most marked LOW priority

These files add maintenance burden without clear benefit. Consider consolidating into a single `ROADMAP.md` or removing entirely.

---

## 3. Redundant Code & Simplifications

### 3.1 ~~`parse_lines` builds a `Vec<Token>` per line when slicing would work~~ DONE

**File:** `src/parser/mod.rs` — Rewritten to use `&tokens[line_start..i]` slices. See also 1.5.

---

### 3.2 `process_line` filters comments every time, but most lines have 0-1 comments

**File:** `src/parser/mod.rs:70-73`

```rust
let filtered: Vec<&Token> = tokens
    .iter()
    .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
    .collect();
```

This allocates a new `Vec` for every line. Since comments are rare (usually 0 or 1 per line), consider iterating without collecting, or using a `SmallVec` to avoid heap allocation for typical cases.

---

### 3.3 Duplicate label handling in `process_line` -- two branches do the same thing

**File:** `src/parser/mod.rs:91-114`

```rust
TokenKind::Label(name) => {
    if filtered.len() == 1 {
        label = Some(name.clone());
        lines.push(SourceLine { label, content: LineContent::Empty, ... });
        return;
    }
    if filtered[1].kind.is_instruction_or_directive() {
        label = Some(name.clone());
        content_tokens = &filtered[1..];
    } else {
        label = Some(name.clone());
        lines.push(SourceLine { label, content: LineContent::Empty, ... });
        return;
    }
}
```

The `len() == 1` case and the `else` branch do the exact same thing -- set the label and push an `Empty` line. These should be merged into a single branch: "if there's no instruction/directive following, push empty."

---

### 3.4 `BrFlags::parse` unreachable dead-letter check

**File:** `src/lexer/token.rs:124-127`

```rust
// At least one flag must be set
if !n && !z && !p {
    return None;
}
```

This check is unreachable. If `flags_part` is empty, the function already returns `Some(true, true, true)` at line 108. If `flags_part` is non-empty, the loop at lines 115-122 will either set at least one flag or return `None` on an unrecognized character. There is no path where all three remain `false` after the loop completes.

---

### 3.5 ~~`SymbolTable::insert` does a double lookup~~ DONE

First fixed with Entry API (session 2), then entire SymbolTable rewritten to `Vec<(String, u16)>` (session 3, item 1.7). No HashMap lookups at all now.

---

### 3.6 ~~`record_label` in first_pass also does a double lookup~~ DONE

Refactored to single `table.get()` call with `if let Some`/`else` pattern. `SymbolTable::contains()` was also removed as it became unused.

---

### 3.7 `encode_instruction` duplicates BR flag encoding that `BrFlags::as_u16` already provides

**File:** `src/encoder/mod.rs:177-179`

```rust
let nzp =
    ((flags.n as u16) << 11) | ((flags.z as u16) << 10) | ((flags.p as u16) << 9);
```

`BrFlags` already has `as_u16()` which encodes the flags as a 3-bit value. This manual encoding should be replaced with:

```rust
let nzp = (flags.as_u16()) << 9;
```

---

### 3.8 Six `#[inline]` annotations are unnecessary

**File:** `src/lexer/mod.rs:38,48`

```rust
#[inline]
fn u16_to_twos_complement(v: u32) -> i32 { ... }

#[inline]
fn process_escape_char(esc: char) -> Option<char> { ... }
```

These small functions within the same crate will be inlined by the compiler at any optimization level. The `#[inline]` annotations are noise and convey a false sense of manual optimization. Remove them.

---

## 4. Bug Risks & Correctness Issues

### 4.1 `.FILL` with out-of-range values silently produces wrong output

**File:** `src/encoder/mod.rs:85-86`

```rust
LineContent::FillImmediate(value) => {
    self.emit(*value as u16);
}
```

`value` is `i32`. Casting `i32` to `u16` with `as` silently truncates. For example, `.FILL #-1` produces `0xFFFF` (correct), but `.FILL #70000` would silently truncate to `0x1170` without any error. The parser stores `.FILL` values as `i32` but never validates that they fit in 16 bits. This can produce silently incorrect output.

---

### 4.2 `.BLKW` with negative count is not validated

**File:** `src/parser/mod.rs:381`

```rust
Ok(LineContent::Blkw(value as u16))
```

`value` is `i32` from `token_to_i32`. If a user writes `.BLKW #-1`, it becomes `0xFFFF` (65535) after the `as u16` cast, allocating 65535 words of zeros. The first pass catches `.BLKW #0` but not negative values.

---

### 4.3 `TRAP` vector truncation -- no range validation

**File:** `src/parser/mod.rs:304-306`

```rust
Ok(LineContent::Instruction(Instruction::Trap {
    trapvect8: value as u8,
}))
```

`value` is `i32`. Casting to `u8` silently truncates. `TRAP x1FF` would silently become `TRAP xFF`. The trap vector should be validated to be in the range `0x00..=0xFF`.

---

### 4.4 `imm5` range is never validated during parsing

**File:** `src/parser/macros.rs:43`

```rust
Ok(LineContent::Instruction($imm_variant(dr, sr1, imm as i16)))
```

The `imm` value comes from `token_to_i32`, which can return any `i32`. The cast to `i16` truncates silently, but even as `i16`, values outside the 5-bit signed range (-16 to 15) are not validated. The encoder will silently truncate via `sign_extend`, producing incorrect machine code for `ADD R1, R1, #100`.

---

### 4.5 `offset6` range is never validated during parsing

**File:** `src/parser/macros.rs:102`

Same issue as 4.4 but for LDR/STR offset6. Values outside -32 to 31 are silently truncated.

---

### 4.6 `.ORIG` rejects valid hex addresses above `0x7FFF`

**File:** `src/parser/mod.rs:322`

```rust
if !(0..=0xFFFF).contains(&value) {
```

`value` is `i32`. Due to the two's complement handling in the lexer, `.ORIG x3000` is parsed as positive `0x3000`, but `.ORIG xFFFF` is parsed as `-1` by the lexer's `u16_to_twos_complement`. Since `-1` is NOT in `0..=0xFFFF`, `.ORIG xFFFF` is incorrectly rejected. This is a bug -- `xFFFF` is a valid origin address.

---

### 4.7 `input_file.replace(".asm", ".obj")` is fragile

**File:** `src/main.rs:62`

```rust
input_file.replace(".asm", ".obj")
```

This replaces ALL occurrences of `.asm` in the path, not just the extension. A file at `path/to/.asm/program.asm` would produce `path/to/.obj/program.obj`. Use `Path::with_extension("obj")` instead.

---

### 4.8 ~~`Stringz` word count uses `s.len()` which counts bytes, not characters~~ DONE

Fixed to `s.chars().count()`. Doc test also updated to verify correct counting.

---

## 5. Code Quality & Maintainability

### 5.1 `main.rs` has no `--version` or `--help` flag support

**File:** `src/main.rs:12-19`

The CLI parsing is entirely manual. There is no `--version` flag (the release workflow mentions `lc3-assembler --version`). There is no `--help` flag. The `-o` flag parsing at line 59 doesn't handle edge cases like `-o` without a following argument. Consider using a minimal arg parser or at least adding `--version` and `--help`.

---

### 5.2 `AsmError` should implement `std::error::Error` trait

**File:** `src/error.rs:107-115`

`AsmError` implements `Display` but not `std::error::Error`. This makes it incompatible with the standard Rust error handling ecosystem (`?` operator, `anyhow`, `thiserror`, `Box<dyn Error>`). Adding the trait impl is one line:

```rust
impl std::error::Error for AsmError {}
```

---

### 5.3 No `Display` impl for `ErrorKind`

**File:** `src/error.rs:75-105`

`ErrorKind` has 22 variants but no `Display` implementation. This means error kinds can only be printed via `Debug` formatting (e.g., `DuplicateLabel`). A human-readable `Display` impl would allow better error categorization in output.

---

### 5.4 `Cargo.toml` has placeholder repository URLs

**File:** `Cargo.toml:8-9`

```toml
homepage = "https://github.com/your-repo/lc3-assembler"
repository = "https://github.com/your-repo/lc3-assembler"
```

These are placeholder URLs that would break `crates.io` publishing and cargo metadata.

---

### 5.5 `LexResult`, `ParseResult`, `FirstPassResult`, `EncodeResult` should have consistent API

**Files:** `src/lexer/mod.rs:32`, `src/parser/mod.rs:34`, `src/first_pass/mod.rs:29`, `src/encoder/mod.rs:26`

These four result types have similar shapes but inconsistent field names and no shared trait. Consider:
- A common `has_errors()` method
- A `DiagnosticResult` trait
- At minimum, consistent naming (`errors` field exists on all -- good)

---

### 5.6 Magic numbers in encoder without named constants

**File:** `src/encoder/mod.rs:117-207`

Opcode encoding uses raw binary literals like `0b0001`, `0b0101`, `0b1001`. While the comments explain them, named constants would be clearer:

```rust
const OP_ADD: u16 = 0b0001;
const OP_AND: u16 = 0b0101;
// etc.
```

Similarly, trap vector constants for pseudo-ops (lines 198-203) are raw hex: `0xF020`, `0xF021`, etc.

---

### 5.7 No `#[must_use]` on result-returning public functions

**Files:** `src/lexer/mod.rs:61`, `src/parser/mod.rs:39`, `src/first_pass/mod.rs:43`, `src/encoder/mod.rs:47`

The four main pipeline functions return results that should never be silently discarded. Adding `#[must_use]` prevents accidental misuse.

---

### 5.8 `pub` visibility on parser helper functions is only needed for macros

**File:** `src/parser/mod.rs:406,417,435,450,465,474,481`

Functions like `ensure_no_extra`, `expect_comma`, `expect_register`, `expect_label`, `token_to_i32`, `token_to_register`, `token_to_label` are all `pub` solely because the macros in `macros.rs` need to access them via re-exports. They are implementation details that should not be part of the public API. Use `pub(crate)` instead.

---

## 6. Test Gaps & Weaknesses

### 6.0 ~~CRITICAL: Dead test modules — `parser/tests.rs` and `first_pass/tests.rs` were never compiled~~ DONE

**Files:** `src/parser/mod.rs`, `src/first_pass/mod.rs` — Both files existed on disk but were NEVER compiled because their parent modules were missing `#[cfg(test)] mod tests;` declarations. Added the declarations, bringing **46 previously-dead tests** online (32 parser + 14 first_pass), going from 85 to 131 total tests. Also fixed inner `mod tests` wrapper (module_inception clippy warning) by flattening the test contents directly into the file.

### 6.1 ~~No encoder unit tests beyond `sign_extend`~~ DONE

**File:** `src/encoder/mod.rs` — Added 42 encoder unit tests covering: all operate instructions (ADD reg/imm, AND reg/imm, NOT), all data movement (LD, LDI, LDR, LEA, ST, STI, STR with positive and negative offsets), all control flow (BR with nzp/n/zp flags, JMP, RET, JSR, JSRR, RTI), all traps (TRAP + 6 aliases), all directives (.FILL immediate/negative/label, .BLKW, .STRINGZ/empty), PC offset edge cases (max positive/negative 9-bit, max 11-bit JSR, out-of-range positive/negative), error paths (undefined label in instruction and .FILL), address tracking across multiple instructions and .BLKW, and orig_address propagation.

---

### 6.2 ~~No test for the `errors.asm` error recovery path~~ DONE

See 2.5. `errors_asm_file_produces_errors` integration test added.

---

### 6.3 ~~Parser error tests are too weak -- they only check `!errors.is_empty()`~~ DONE

**File:** `src/parser/tests.rs` — Strengthened 3 error tests to assert specific `ErrorKind` variants: `parse_missing_operand` now asserts `TooFewOperands`, `parse_extra_operand` asserts `UnexpectedToken`, `parse_missing_comma` asserts `TooFewOperands` (token count check fires first). Also fixed `parse_missing_comma` which had expected `ExpectedComma` but the actual behavior is `TooFewOperands`.

---

### 6.4 ~~`.FILL xBEEF` test may assert the wrong expected value~~ DONE

**File:** `src/parser/tests.rs` — Fixed `parse_fill_hex` to assert `FillImmediate(-16657)` instead of `FillImmediate(0xBEEF)`. The lexer's two's complement conversion produces -16657 for values > 0x7FFF. This test was always wrong but never caught because the test module was never compiled (see 6.0).

---

### 6.5 ~~No test for address overflow in the first pass~~ DONE

**File:** `src/first_pass/tests.rs` — Added `address_overflow_error` test: `.ORIG xFFF0` + `.BLKW #100` asserts `AddressOverflow` error is produced.

---

### 6.6 ~~No test for `.ORIG` with a label~~ DONE

**File:** `src/first_pass/tests.rs` — Added `orig_with_label` test: `START .ORIG x3000` asserts label `START` is recorded at address 0x3000.

---

### 6.7 ~~No negative test for the full pipeline (end-to-end error reporting)~~ DONE

Added 13 error-path integration tests covering: undefined label, duplicate label, missing .ORIG, imm5 out of range, offset6 out of range, too few operands, invalid .ORIG address (hex and decimal), invalid .BLKW count, TRAP vector out of range, and errors.asm.

---

### 6.8 ~~`run_pipeline` and `run_full_pipeline` are nearly identical~~ DONE

**File:** `tests/integration_tests.rs` — Refactored `run_full_pipeline` to call `run_pipeline` internally instead of duplicating the lexer→parser→first_pass chain.

---

## 7. CI / Docker / Infrastructure Issues

### 7.1 `release.yml` uses deprecated GitHub Actions

**File:** `.github/workflows/release.yml:41,180,189`

```yaml
uses: actions/create-release@v1
uses: actions/upload-release-asset@v1
```

`actions/create-release@v1` and `actions/upload-release-asset@v1` are archived and deprecated. They should be replaced with `softprops/action-gh-release@v2` or the `gh` CLI.

---

### 7.2 CI uses deprecated `actions-rs/audit-check@v1`

**File:** `.github/workflows/ci.yml:126`

```yaml
uses: actions-rs/audit-check@v1
```

The `actions-rs` organization is unmaintained. Replace with `rustsec/audit-check` or `cargo install cargo-audit && cargo audit`.

---

### 7.3 Dockerfile uses deprecated `as` syntax and pinned old Rust version

**File:** `Dockerfile:14`

```dockerfile
FROM rust:1.75 as builder
```

Modern Docker uses `AS` (uppercase). While both work, `as` is deprecated in newer Docker versions. Also, `rust:1.75` is pinned to a specific Rust version that may become outdated. Consider using `rust:latest` or at least a newer version.

---

### 7.4 Dockerfile installs `ca-certificates` unnecessarily

**File:** `Dockerfile:38-41`

```dockerfile
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*
```

The binary is a standalone assembler that reads and writes local files. It never makes HTTPS connections. `ca-certificates` is unnecessary and adds to image size.

---

### 7.5 Dockerfile `CMD ["--help"]` will fail

**File:** `Dockerfile:53`

```dockerfile
CMD ["--help"]
```

The binary does not handle a `--help` flag (see item 5.1). Running the container with no arguments will pass `--help` as the input file path, producing: `Error: Failed to read '--help': No such file or directory`.

---

### 7.6 CI MSRV claims 1.70 but no documentation explains why

**File:** `.github/workflows/ci.yml:163`, `Cargo.toml:13`

The MSRV is 1.70, but nothing documents what minimum Rust feature requires this specific version. If CI ever breaks, there's no fallback or guidance. Consider adding a comment in `Cargo.toml` explaining why 1.70 was chosen and what the minimum feature dependency is.

---

### 7.7 `codecov/codecov-action@v3` is outdated

**File:** `.github/workflows/ci.yml:104`

```yaml
uses: codecov/codecov-action@v3
```

v4 is the current version with better token handling and reliability.

---

### 7.8 `Cargo.lock` is in `.gitignore` but should be committed for binaries

**File:** `.gitignore`

Cargo's official guidance: libraries should not commit `Cargo.lock`, but **binaries should**. Since this project produces a binary (`lc3-assembler`), `Cargo.lock` should be committed to ensure reproducible builds.

---

## 8. Feature Gaps

### 8.1 No validation that labels don't shadow reserved words

If a user writes `ADD .FILL #5`, the label `ADD` will be stored in the symbol table, shadowing the opcode. Later references to `ADD` as a label will work, but this is almost certainly a user error. The `LabelIsReservedWord` error kind exists (see item 2.4) but is never used.

---

### 8.2 No `.INCLUDE` directive

There is no way to split a program across multiple files. This is a standard assembler feature and was identified in `FUTURE_IMPROVEMENTS.md` as MEDIUM-HIGH priority.

---

### 8.3 No macro system (`.MACRO`/`.ENDM`)

Assembly macros for common patterns like `PUSH`/`POP` are not supported. This was identified as MEDIUM-HIGH priority in `FUTURE_IMPROVEMENTS.md`.

---

### 8.4 No listing file output (`-l` flag)

The assembler only outputs a binary `.obj` file. There is no option to generate a human-readable listing file showing addresses, machine code, and source side by side. This is a standard feature of assemblers and is very useful for debugging.

---

### 8.5 No numeric offset support for PC-relative instructions

Instructions like `LD R0, DATA` only accept labels. Standard LC-3 assemblers also allow numeric offsets like `LD R0, #5` (load from PC+5). The parser rejects this with "requires a label operand."

---

### 8.6 No warning system -- only errors

The assembler has errors but no warnings. Useful warnings would include:
- Label shadows a reserved word (see 8.1)
- `.FILL` value exceeds 16 bits (see 4.1)
- Unreachable code after `HALT` or unconditional `BR`
- Unused labels (defined but never referenced)

---

### 8.7 No `--version` flag

The release workflow references `lc3-assembler --version` in its documentation, but the binary does not support this flag. See item 5.1.

---

### 8.8 No support for octal literals

The lexer supports decimal (`#10`), hexadecimal (`x3000`), and binary (`b1010`) literals, but not octal (`o17`). While not critical, some LC-3 references use octal.

---

### 8.9 `.STRINGZ` does not validate that characters fit in 16 bits

**File:** `src/encoder/mod.rs:102-103`

```rust
for ch in s.chars() {
    self.emit(ch as u16);
}
```

`ch as u16` silently truncates for characters with code points > `0xFFFF` (supplementary Unicode characters). While LC-3 strings are typically ASCII, there is no error if someone puts non-ASCII characters in a `.STRINGZ`.

---

### 8.10 No way to assemble from stdin or pipe output to stdout

The assembler requires file paths for both input and output. Supporting stdin/stdout (`lc3-assembler - | hexdump`) would make it more unix-friendly and composable.

---

*End of document. 53 items total across 8 categories. As of session 4: 34 items completed (~~strikethrough~~), 19 remaining (mostly infrastructure, features, and minor optimizations).*
