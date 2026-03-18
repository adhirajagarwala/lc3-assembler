//! # Listing File Generator
//!
//! Produces a human-readable `.lst` file that shows, for every source line:
//! - The LC-3 address where the line was assembled
//! - The machine-code word(s) produced
//! - The original source text
//!
//! ## Format
//!
//! ```text
//! LC-3 Assembler Listing — program.asm
//! ─────────────────────────────────────────────────────────────────
//!  Addr   Code     Line  Source
//! ─────────────────────────────────────────────────────────────────
//! (3000)  5020        1  MAIN    ADD R0, R1, R0
//! (3001)  0E01        2          BRz DONE
//! (3002)  2004        3          LD  R0, DATA
//! (3003)  F025        4          HALT
//! ────── ──────  ──────
//! (3004)  000A        6  DATA    .FILL #10
//!         ----        7  DONE    .END
//! ```

use crate::encoder::EncodeResult;
use crate::first_pass::FirstPassResult;

/// Generate a listing string from the assembled program.
///
/// `source`      — original source text (used to extract source lines by number)
/// `first`       — first-pass result (has source_lines with spans)
/// `encoded`     — encoder result (has machine_code and per-line word map)
/// `filename`    — display name for the header
#[must_use]
pub fn generate(
    source: &str,
    first: &FirstPassResult,
    encoded: &EncodeResult,
    filename: &str,
) -> String {
    let raw_lines: Vec<&str> = source.lines().collect();

    let sep = "─".repeat(66);
    let mut out = String::new();

    // Header
    out.push_str(&format!("LC-3 Assembler Listing — {filename}\n"));
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!("{:>6}  {:>4}  {:>6}  {}\n", "Addr", "Code", "Line", "Source"));
    out.push_str(&sep);
    out.push('\n');

    // Walk the encoded per-line info
    for info in &encoded.line_infos {
        let sl = &first.source_lines[info.source_line_idx];
        let line_num = sl.span.line;
        let src_text = raw_lines.get(line_num.saturating_sub(1)).copied().unwrap_or("");

        match info.words.as_slice() {
            // Lines that produce no words (Empty, .END, labels-only)
            [] => {
                out.push_str(&format!(
                    "{:>8}  {:>4}  {:>6}  {}\n",
                    "", "----", line_num, src_text,
                ));
            }
            // Single-word lines (most instructions and directives)
            [word] => {
                out.push_str(&format!(
                    "({:04X})  {:04X}  {:>6}  {}\n",
                    info.address, word, line_num, src_text,
                ));
            }
            // Multi-word lines (.BLKW, .STRINGZ)
            words => {
                // First word: print address + code + line + source
                out.push_str(&format!(
                    "({:04X})  {:04X}  {:>6}  {}\n",
                    info.address, words[0], line_num, src_text,
                ));
                // Remaining words: continuation rows (address increments, no source)
                for (i, &word) in words[1..].iter().enumerate() {
                    out.push_str(&format!(
                        "({:04X})  {:04X}\n",
                        info.address + 1 + i as u16,
                        word,
                    ));
                }
            }
        }
    }

    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "  Origin: x{:04X}   Words: {}   Bytes: {}\n",
        encoded.orig_address,
        encoded.machine_code.len(),
        encoded.machine_code.len() * 2,
    ));

    out
}
