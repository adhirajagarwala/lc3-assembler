//! # LC-3 Assembler
//!
//! A complete two-pass assembler for the LC-3 (Little Computer 3) architecture.
//!
//! ## Pipeline
//!
//! The assembler processes LC-3 assembly code through four stages:
//!
//! 1. **Lexer** - Tokenizes source code into tokens
//! 2. **Parser** - Parses tokens into an Abstract Syntax Tree (AST)
//! 3. **First Pass** - Builds symbol table and validates structure
//! 4. **Encoder** - Generates binary machine code
//!
//! ## Example
//!
//! ```rust,no_run
//! use lc3_assembler::{lexer::tokenize, parser::parse_lines, first_pass::first_pass, encoder::encode};
//!
//! let source = std::fs::read_to_string("program.asm").unwrap();
//! let lexed = tokenize(&source);
//! let parsed = parse_lines(&lexed.tokens);
//! let first = first_pass(&parsed.lines);
//! let encoded = encode(&first);
//!
//! // Check for errors at each stage
//! if encoded.errors.is_empty() {
//!     println!("Success! Generated {} words of machine code", encoded.machine_code.len());
//! }
//! ```

pub mod encoder;
pub mod error;
pub mod first_pass;
pub mod lexer;
pub mod parser;
