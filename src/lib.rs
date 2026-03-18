//! # LC-3 Assembler
//!
//! A complete assembler for the LC-3 (Little Computer 3) architecture.
//!
//! ## Pipeline
//!
//! Source code passes through five stages before machine code is produced:
//!
//! 1. **Preprocessor** — Expands `.INCLUDE "file"` directives recursively,
//!    with cycle detection (see [`preprocessor`]).
//! 2. **Macro Expander** — Expands `.MACRO`/`.ENDM` definitions and their
//!    invocations using text substitution (see [`macro_expand`]).
//! 3. **Lexer** — Tokenises the expanded source into a flat token stream
//!    (see [`lexer`]).
//! 4. **Parser** — Converts tokens into a line-oriented AST, validating
//!    operand types and immediate ranges (see [`parser`]).
//! 5. **First Pass** — Builds the symbol table, computes addresses, and
//!    validates program structure (see [`first_pass`]).
//! 6. **Encoder** — Converts the AST + symbol table into 16-bit machine-code
//!    words, resolving PC-relative offsets (see [`encoder`]).
//!
//! Errors are accumulated at every stage rather than halting on the first
//! failure, so a single assembly run reports as many problems as possible.
//!
//! ## Example
//!
//! ```rust,no_run
//! use lc3_assembler::{lexer::tokenize, parser::parse_lines, first_pass::first_pass, encoder::encode};
//!
//! let source = std::fs::read_to_string("program.asm").unwrap();
//! let lexed = tokenize(&source);
//! let parsed = parse_lines(&lexed.tokens);
//! let first = first_pass(parsed.lines);  // takes ownership of parsed lines
//! let encoded = encode(&first);
//!
//! // Check for errors at each stage
//! if !encoded.has_errors() {
//!     println!("Success! Generated {} words of machine code", encoded.machine_code.len());
//! }
//!
//! // Print any warnings (unused labels, unreachable code, etc.)
//! for w in first.warnings.iter().chain(encoded.warnings.iter()) {
//!     eprintln!("{w}");
//! }
//! ```

pub mod diagnostic;
pub mod encoder;
pub mod error;
pub mod first_pass;
pub mod lexer;
pub mod listing;
pub mod macro_expand;
pub mod parser;
pub mod preprocessor;
pub mod warning;
