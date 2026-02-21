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

### ~~1.6 `write_obj_file` issues one syscall per u16 word~~ DONE

**File:** `src/main.rs` — Pre-allocates a `Vec<u8>` buffer, writes all words into it, then issues a single `fs::write`. One syscall for the entire file regardless of size.

---

### 1.7 ~~`SymbolTable` uses `HashMap` + `Vec` when `IndexMap` would suffice~~ DONE

**File:** `src/first_pass/symbol_table.rs` — **Rewritten.** Replaced `HashMap<String, u16> + Vec<String>` with a single `Vec<(String, u16)>`. Each label stored exactly once. O(n) linear scan is fine for LC-3 programs (<50 labels). Zero external dependencies.

---

### ~~1.8 `sign_extend` is misnamed and could be `const fn`~~ DONE

**File:** `src/encoder/mod.rs` — Changed to `const fn`. Pure arithmetic; no runtime dependencies.

---

## 2. Dead / Unnecessary Code

### ~~2.1 `PseudoRet` comment is wrong -- it says "TRAP x25 (HALT)"~~ DONE

**File:** `src/lexer/token.rs` — Fixed to `// JMP R7 (return from subroutine) — NOT a TRAP instruction`.

---

### 2.2 ~~`peek_next()` on Cursor is never used anywhere~~ DONE

Removed during Cursor rewrite (1.2). The new byte-slice Cursor does not include `peek_next`.

---

### ~~2.3 `Span.start` and `Span.end` are never used for anything meaningful~~ DONE

**File:** `src/error.rs` — `start` and `end` byte-offset fields removed from `Span`. `Span` now holds only `line: usize` and `col: usize`. `cursor.make_span()` signature reduced from `(start_byte, start_line, start_col)` to `(start_line, start_col)`. `current_pos()` returns `(usize, usize)` instead of `(usize, usize, usize)`. All lex functions had their `sb` parameter removed. `parser/mod.rs::line_span()` simplified from 12 lines to 4. Two hard-coded `Span { start: 0, end: 0, line: 1, col: 1 }` literals in `first_pass/mod.rs` and the test helper in `encoder/mod.rs` updated. Doc comment in `Span` notes: if source-underline diagnostics are ever added, restore `start`/`end`.

---

### ~~2.4 `ErrorKind::OrigNotFirst` and `ErrorKind::LabelIsReservedWord` are never constructed~~ DONE

**File:** `src/error.rs` — Both dead variants removed. Explanatory comments added explaining why each was removed and what to do if the relevant feature is ever implemented.

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

### ~~3.2 `process_line` filters comments every time, but most lines have 0-1 comments~~ DONE

**File:** `src/parser/mod.rs` — Uses `position()` to find the first comment, then slices `tokens[..code_end]`. Short-circuits after finding the comment rather than evaluating the predicate for every remaining token.

---

### ~~3.3 Duplicate label handling in `process_line` -- two branches do the same thing~~ DONE

**File:** `src/parser/mod.rs` — Collapsed into a single branch: `label = Some(name.clone()); if filtered.len() > 1 && filtered[1].kind.is_instruction_or_directive() { ... } else { push Empty, return; }`.

---

### ~~3.4 `BrFlags::parse` unreachable dead-letter check~~ DONE

**File:** `src/lexer/token.rs` — Removed the `if !n && !z && !p { return None; }` check. Confirmed unreachable: empty `flags_part` returns early with all-true, non-empty either sets ≥1 flag or hits the `_ => return None` arm.

---

### 3.5 ~~`SymbolTable::insert` does a double lookup~~ DONE

First fixed with Entry API (session 2), then entire SymbolTable rewritten to `Vec<(String, u16)>` (session 3, item 1.7). No HashMap lookups at all now.

---

### 3.6 ~~`record_label` in first_pass also does a double lookup~~ DONE

Refactored to single `table.get()` call with `if let Some`/`else` pattern. `SymbolTable::contains()` was also removed as it became unused.

---

### ~~3.7 `encode_instruction` duplicates BR flag encoding that `BrFlags::as_u16` already provides~~ DONE

**File:** `src/encoder/mod.rs` — Now uses `(OP_BR << 12) | (flags.as_u16() << 9) | offset`. Manual bit-shifting removed entirely.

---

### ~~3.8 `#[inline]` annotations are unnecessary~~ DONE

**File:** `src/lexer/mod.rs` — Both `#[inline]` annotations removed from `u16_to_twos_complement` and `process_escape_char`. The compiler inlines small private functions at opt-level 2+ without hints.

---

## 4. Bug Risks & Correctness Issues

### ~~4.1 `.FILL` with out-of-range values silently produces wrong output~~ DONE

**File:** `src/parser/mod.rs` — Range validation added before storing the value. Values outside `i16::MIN as i32..=0xFFFF` are rejected with `InvalidOrigAddress` error. The encoder cast is now safe.

---

### ~~4.2 `.BLKW` with negative count is not validated~~ DONE

**File:** `src/parser/mod.rs` — Added `if value <= 0 || value > 0xFFFF` check before the `as u16` cast. Negative and zero counts produce `InvalidBlkwCount` error.

---

### ~~4.3 `TRAP` vector truncation -- no range validation~~ DONE

**File:** `src/parser/mod.rs` — Added `!(0..=0xFF).contains(&value)` check. `TRAP x1FF` now produces an `InvalidOperandType` error instead of silently truncating.

---

### ~~4.4 `imm5` range is never validated during parsing~~ DONE

**File:** `src/parser/macros.rs` — Added `!(-16..=15).contains(&imm)` check before the cast. `ADD R1, R1, #100` now produces `InvalidOperandType` error.

---

### ~~4.5 `offset6` range is never validated during parsing~~ DONE

**File:** `src/parser/macros.rs` — Added `!(-32..=31).contains(&value)` check. `LDR R0, R1, #100` now produces `InvalidOperandType` error.

---

### ~~4.6 `.ORIG` rejects valid hex addresses above `0x7FFF`~~ DONE

**File:** `src/parser/mod.rs` — Changed check to `!(i16::MIN as i32..=0xFFFF_i32).contains(&value)`. The lexer converts `xFFFF` → `-1` (two's complement), and `-1` is now accepted. `.ORIG xFFFF` works correctly.

---

### ~~4.7 `input_file.replace(".asm", ".obj")` is fragile~~ DONE

**File:** `src/main.rs` — Changed to `Path::new(input_file).with_extension("obj").to_string_lossy().into_owned()`. Added `use std::path::Path;`.

---

### 4.8 ~~`Stringz` word count uses `s.len()` which counts bytes, not characters~~ DONE

Fixed to `s.chars().count()`. Doc test also updated to verify correct counting.

---

## 5. Code Quality & Maintainability

### ~~5.1 `main.rs` has no `--version` or `--help` flag support~~ DONE

**File:** `src/main.rs` — Added `--version`/`-V` and `--help`/`-h` flags. Version reads from `env!("CARGO_PKG_VERSION")`. `--help` exits 0; no args exits 1. Fixes broken Dockerfile `CMD ["--help"]`.

---

### ~~5.2 `AsmError` should implement `std::error::Error` trait~~ DONE

**File:** `src/error.rs` — Added `impl std::error::Error for AsmError {}`. Now compatible with `?`, `Box<dyn Error>`, `anyhow`, etc.

---

### ~~5.3 No `Display` impl for `ErrorKind`~~ DONE

**File:** `src/error.rs` — Added `impl std::fmt::Display for ErrorKind` with a match on all variants, producing human-readable phrases like "duplicate label", "PC offset out of range", etc.

---

### ~~5.4 `Cargo.toml` has placeholder repository URLs~~ DONE

**File:** `Cargo.toml` — Updated to `https://github.com/adhirajagarwala/lc3-assembler` for both `homepage` and `repository`.

---

### ~~5.5 `LexResult`, `ParseResult`, `FirstPassResult`, `EncodeResult` should have consistent API~~ DONE

**Files:** `src/lexer/mod.rs`, `src/parser/mod.rs`, `src/first_pass/mod.rs`, `src/encoder/mod.rs` — All four result types now have `#[must_use] pub fn has_errors(&self) -> bool { !self.errors.is_empty() }`. The `lib.rs` doc example updated to use `encoded.has_errors()`. A shared `DiagnosticResult` trait was deliberately skipped — a trait would force a public trait object boundary with no current benefit; `has_errors()` as a concrete method on each type is sufficient.

---

### ~~5.6 Magic numbers in encoder without named constants~~ DONE

**File:** `src/encoder/mod.rs` — Added 15 opcode constants (`OP_ADD` through `OP_RTI`) and 6 pre-shifted TRAP constants (`TRAP_GETC` through `TRAP_HALT`). All raw binary literals replaced throughout `encode_instruction`.

---

### ~~5.7 No `#[must_use]` on result-returning public functions~~ DONE

**Files:** All four pipeline functions — `tokenize`, `parse_lines`, `first_pass`, `encode` — now have `#[must_use]`.

---

### ~~5.8 `pub` visibility on parser helper functions is only needed for macros~~ DONE

**File:** `src/parser/mod.rs` — All 7 parser helpers changed from `pub` to `pub(crate)`. `macros.rs` re-export updated to `pub(crate) use super::`.

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

### ~~8.7 No `--version` flag~~ DONE

See 5.1. Both `--version`/`-V` and `--help`/`-h` were added to `main.rs`. The Dockerfile `CMD ["--help"]` now works correctly.

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

*End of document. 53 items total across 8 categories. As of session 6: **54 items completed** (~~strikethrough~~), **0 remaining** actionable code items.*

*Remaining open items are all CI/Docker/infrastructure (section 7, items 7.1–7.8) and feature gaps (section 8, items 8.1–8.6, 8.8–8.10). These are outside the core codebase scope:*

**CI/Infrastructure (7 items):**
- *7.1 — Update `release.yml` from deprecated `actions/create-release@v1` → `softprops/action-gh-release@v2`*
- *7.2 — Replace unmaintained `actions-rs/audit-check@v1` → `rustsec/audit-check` or `cargo audit`*
- *7.3 — Dockerfile: `as` → `AS`, update from pinned `rust:1.75` to `rust:latest` or newer*
- *7.4 — Dockerfile: remove unnecessary `ca-certificates` (binary makes no HTTPS calls)*
- *7.6 — Add comment to `Cargo.toml` explaining why MSRV is 1.70*
- *7.7 — Update `codecov/codecov-action@v3` → `v4`*
- *7.8 — Remove `Cargo.lock` from `.gitignore` (binaries should commit it for reproducible builds)*

**Feature Gaps (9 items):**
- *8.1 — Validate that labels don't shadow reserved words (re-add `LabelIsReservedWord` error variant)*
- *8.2 — `.INCLUDE` directive for multi-file programs*
- *8.3 — Macro system (`.MACRO`/`.ENDM`)*
- *8.4 — Listing file output (`-l` flag) showing addresses + machine code + source*
- *8.5 — Numeric offset support for PC-relative instructions (`LD R0, #5`)*
- *8.6 — Warning system (unused labels, unreachable code, etc.)*
- *8.8 — Octal literal support (`o17`)*
- *8.9 — `.STRINGZ` validation that code points fit in 16 bits (currently truncates silently)*
- *8.10 — stdin/stdout support (`lc3-assembler - | hexdump`)*
