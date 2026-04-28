# AGENTS.md

## Git Identity — Non-Negotiable

Every commit and push in this repo must be authored as:
  Name:  Adhiraj Agarwala
  Email: adhirajagarwala2007@gmail.com

Before the first commit in any session, run:
  git config user.name "Adhiraj Agarwala"
  git config user.email "adhirajagarwala2007@gmail.com"

Rules:
- Never commit unless git config user.name is exactly Adhiraj Agarwala.
- Never commit unless git config user.email is exactly adhirajagarwala2007@gmail.com.
- Never add Co-Authored-By lines.
- Never add Signed-off-by lines.
- Never add AI/tool/assistant contributor metadata.
- The commit author must be always and only Adhiraj Agarwala.

## Project

This repository is `lc3-assembler`: a production-quality two-pass assembler for
the LC-3 educational computer architecture, written in Rust.

It reads `.asm` source files and produces `.obj` binary files in the LC-3 object
file format (big-endian, origin address first, then machine code words).

The workspace also contains `simulator/` — an LC-3 TUI debugger that uses the
assembler library as a dependency.

## Architecture

Four-stage pipeline: Lexer → Parser → First Pass → Encoder.
All stages accumulate errors rather than failing fast.

- `src/lexer/`      — tokenizer, byte-slice cursor (zero allocation)
- `src/parser/`     — token → AST (SourceLine, Instruction)
- `src/first_pass/` — symbol table, location counter, .ORIG/.END validation
- `src/encoder/`    — AST + symbol table → Vec<u16> machine code
- `simulator/`      — TUI debugger (ratatui + crossterm)

## Quality Standards

- MSRV: Rust 1.70 (assembler crate only; simulator has higher MSRV via ratatui)
- `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` must pass
- `Cargo.lock` is committed (binary crate policy)
- No `unwrap()` in production paths — all errors accumulate into result types
