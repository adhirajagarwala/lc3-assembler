# Roadmap

Actionable next steps for the LC-3 assembler. Items are roughly ordered by bang-for-buck.

---

## CI / Infrastructure

These are quick fixes — nothing requires changing the Rust source.

| Item | File | Action |
|------|------|--------|
| Deprecated release actions | `.github/workflows/release.yml` | Replace `actions/create-release@v1` + `actions/upload-release-asset@v1` → `softprops/action-gh-release@v2` |
| Deprecated audit action | `.github/workflows/ci.yml` | Replace `actions-rs/audit-check@v1` → `cargo install cargo-audit && cargo audit` |
| Outdated Codecov action | `.github/workflows/ci.yml` | `codecov/codecov-action@v3` → `v4` |
| Old Rust version in Docker | `Dockerfile` | `rust:1.75` → `rust:1.83` (or latest stable) |
| Unnecessary apt package | `Dockerfile` | Remove `ca-certificates` (binary makes no HTTP calls) |
| Commit Cargo.lock | `.gitignore` | Remove `Cargo.lock` — binaries should commit it for reproducible builds |

---

## Feature Gaps (small → large)

### Small (< 1 day each)

**8.9 `.STRINGZ` Unicode guard** — `src/encoder/mod.rs`
Characters with code points > 0xFFFF silently truncate when cast to `u16`. Add a check in `encode_line` for `Stringz`:
```rust
for ch in s.chars() {
    if ch as u32 > 0xFFFF {
        self.errors.push(AsmError::new(ErrorKind::InvalidOperandType,
            format!("Character '{}' (U+{:04X}) cannot fit in 16-bit LC-3 word", ch, ch as u32),
            line.span));
    }
    self.emit(ch as u16);
}
```

**8.8 Octal literals** — `src/lexer/mod.rs`
Add `o` prefix support in `lex_word`: parse `oNNN` identically to hex but in base 8. Add `NumOctal(i32)` to `TokenKind` and handle in `token_to_i32`.

**8.1 Label shadows reserved word** — `src/error.rs` + `src/first_pass/mod.rs`
Re-add `LabelIsReservedWord` to `ErrorKind`. In `record_label()`, check that the label name is not a keyword (ADD, AND, LD, etc.) before inserting it.

**8.5 Numeric PC-relative offsets** — `src/parser/mod.rs`
`LD R0, #5` should be valid (offset = 5, relative to PC). In `expect_label()`, also accept `NumDecimal`/`NumHex` tokens and wrap them in a new `LabelOrOffset` enum.

---

### Medium (1–3 days each)

**8.10 stdin/stdout support** — `src/main.rs`
Accept `-` as the input filename to read from `stdin`. Accept `-` or no `-o` flag to write to `stdout`. Useful for piping: `lc3-assembler - | hexdump`.

**8.4 Listing file** — `src/main.rs` + new `src/listing.rs`
Add `-l [file.lst]` flag. After encoding, produce a human-readable listing:
```
Addr  Machine  Source
3000  5020     ADD R0, R0, #0
3001  0E01     BRz DONE
```
The encoder already tracks `current_address` per line; just needs a parallel output path.

**8.6 Warning system** — `src/error.rs` + all pipeline stages
Add `AsmWarning { kind: WarnKind, message: String, span: Span }`. Add `warnings: Vec<AsmWarning>` to all four result types. Initial warnings:
- Unused labels (defined but never referenced)
- Unreachable code after unconditional `BRnzp` or `HALT`
- Label name shadows a pseudo-op or directive (softer than 8.1)

---

### Large (1+ week each)

**8.2 `.INCLUDE` directive**
Requires a pre-processing stage before lexing: resolve `include` paths, read files, splice their tokens into the stream. Key challenge: Span line numbers must be remapped per-file.

**8.3 Macro system (`.MACRO` / `.ENDM`)**
Define named instruction sequences with parameters. Requires a macro expansion pass between the lexer and parser. New module `src/macro_expand/`.

---

## Architecture Ideas (long-term)

These are not near-term TODOs but worth knowing:

- **Source-underline diagnostics**: Restore `start: usize, end: usize` to `Span` and change `AsmError::fmt` to print the source line with `^` underlines. Requires the original source string to be accessible at error-display time.
- **LSP server**: Wrap the library API in a Language Server Protocol server for IDE integration (auto-complete, hover docs, inline errors).
- **Disassembler**: Add a `lc3-disassembler` binary in the same workspace that reads `.obj` files and produces `.asm`. The encoder's opcode constants and Span-free Instruction enum would be directly reusable.
