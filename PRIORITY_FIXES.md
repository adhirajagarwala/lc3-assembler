# Priority Fixes for LC-3 Assembler

**Analysis Date:** February 16, 2026
**Source Document:** `changes to be made.md`

This document prioritizes the 53 identified issues into actionable tiers based on **impact vs effort**.

---

## 🔴 TIER 1: CRITICAL - Fix Immediately

These are **correctness bugs** that produce wrong output or reject valid programs.

### 1. `.FILL` Silent Truncation (Item 4.1)
**File:** `src/encoder/mod.rs:85-86`
**Issue:** `.FILL #70000` silently truncates to wrong value
**Fix:** Validate that value fits in 16 bits during parsing
**Effort:** ⭐ Low | **Impact:** ⭐⭐⭐ Critical

### 2. `.BLKW` Negative Count (Item 4.2)
**File:** `src/parser/mod.rs:381`
**Issue:** `.BLKW #-1` becomes 65535 words
**Fix:** Reject negative values in first pass
**Effort:** ⭐ Low | **Impact:** ⭐⭐⭐ Critical

### 3. `TRAP` Vector Truncation (Item 4.3)
**File:** `src/parser/mod.rs:304-306`
**Issue:** `TRAP x1FF` silently becomes `TRAP xFF`
**Fix:** Validate trapvect8 is in range 0x00-0xFF
**Effort:** ⭐ Low | **Impact:** ⭐⭐⭐ Critical

### 4. `imm5` Range Not Validated (Item 4.4)
**File:** `src/parser/macros.rs:43`
**Issue:** `ADD R1, R1, #100` silently truncates
**Fix:** Validate imm5 is -16 to 15
**Effort:** ⭐⭐ Medium | **Impact:** ⭐⭐⭐ Critical

### 5. `offset6` Range Not Validated (Item 4.5)
**File:** `src/parser/macros.rs:102`
**Issue:** LDR/STR offsets outside -32 to 31 silently truncate
**Fix:** Validate offset6 range
**Effort:** ⭐⭐ Medium | **Impact:** ⭐⭐⭐ Critical

### 6. `.ORIG xFFFF` Incorrectly Rejected (Item 4.6)
**File:** `src/parser/mod.rs:322`
**Issue:** Valid address `xFFFF` is rejected due to two's complement
**Fix:** Handle unsigned range 0x0000-0xFFFF correctly
**Effort:** ⭐⭐ Medium | **Impact:** ⭐⭐⭐ Critical

### 7. Fragile Path Extension Replacement (Item 4.7)
**File:** `src/main.rs:62`
**Issue:** `replace(".asm", ".obj")` breaks on paths like `/path/to/.asm/file.asm`
**Fix:** Use `Path::with_extension("obj")`
**Effort:** ⭐ Low | **Impact:** ⭐⭐ High

---

## 🟡 TIER 2: HIGH VALUE - Do Next

These provide **significant performance gains** or are **expected CLI features**.

### 8. `lines.to_vec()` Clones Entire AST (Item 1.4)
**File:** `src/first_pass/mod.rs:152`
**Issue:** Biggest unnecessary allocation in the pipeline
**Fix:** Take ownership of Vec or use lifetimes
**Effort:** ⭐⭐ Medium | **Impact:** ⭐⭐⭐ High (performance)

### 9. Token Cloning on Every Parse (Item 1.5)
**File:** `src/parser/mod.rs:56`
**Issue:** Every token cloned into line buffer
**Fix:** Use slices instead of cloning
**Effort:** ⭐⭐⭐ Medium-High | **Impact:** ⭐⭐⭐ High (performance)

### 10. One Syscall Per Word in Output (Item 1.6)
**File:** `src/main.rs:95-97`
**Issue:** 1000 words = 1000 syscalls
**Fix:** Pre-allocate Vec<u8>, single write_all
**Effort:** ⭐ Low | **Impact:** ⭐⭐⭐ High (performance)

### 11. SymbolTable Duplicates Strings (Item 1.7)
**File:** `src/first_pass/symbol_table.rs:5-8`
**Issue:** Every label stored twice (HashMap + Vec)
**Fix:** Use IndexMap crate
**Effort:** ⭐ Low | **Impact:** ⭐⭐ Medium (performance + memory)

### 12. No `--version` or `--help` Flags (Item 5.1)
**File:** `src/main.rs:12-19`
**Issue:** Basic CLI expectations not met
**Fix:** Add --version and --help handling
**Effort:** ⭐ Low | **Impact:** ⭐⭐⭐ High (UX)

### 13. AsmError Missing `std::error::Error` Trait (Item 5.2)
**File:** `src/error.rs:107-115`
**Issue:** Incompatible with Rust error ecosystem
**Fix:** Add `impl std::error::Error for AsmError {}`
**Effort:** ⭐ Low | **Impact:** ⭐⭐ Medium (ecosystem)

---

## 🟢 TIER 3: GOOD TO HAVE - Clean Up When Time Permits

These improve **code quality** but don't affect users immediately.

### Dead Code Removal
- **Item 2.2:** Remove `peek_next()` (never used)
- **Item 2.3:** Remove `Span.start/end` or use them
- **Item 2.4:** Remove unused error variants (`OrigNotFirst`, `LabelIsReservedWord`)
- **Item 2.5-2.6:** Remove or test `errors.asm` and `loop.asm`

### Redundant Code
- **Item 3.1:** Parser line slicing instead of cloning
- **Item 3.5:** SymbolTable double lookup
- **Item 3.6:** record_label double lookup
- **Item 3.7:** Duplicate BR flag encoding

### Code Quality
- **Item 5.3:** Add Display impl for ErrorKind
- **Item 5.4:** Fix placeholder URLs in Cargo.toml
- **Item 5.6:** Named constants for opcodes
- **Item 5.7:** Add #[must_use] on public functions
- **Item 5.8:** Change pub to pub(crate) for helpers

**Effort:** ⭐ Low each | **Impact:** ⭐ Low-Medium (maintainability)

---

## 🔵 TIER 4: LOW PRIORITY - Future Work

These are **feature gaps** and **infrastructure updates** for later.

### Test Improvements (Section 6)
- Add encoder unit tests
- Test error recovery paths
- Assert specific error kinds
- Add overflow tests

### CI/Docker Updates (Section 7)
- Update deprecated GitHub Actions
- Upgrade Docker image
- Update codecov action
- Commit Cargo.lock

### Feature Gaps (Section 8)
- Warning system
- `.INCLUDE` directive
- Macro system
- Listing file output
- Stdin/stdout support

**Effort:** ⭐⭐⭐⭐ High | **Impact:** ⭐ Low (future enhancements)

---

## Recommended Action Plan

### Week 1: Fix All Critical Bugs
```bash
# Fix items 1-7 (all of Tier 1)
# Estimated time: 2-3 days
# Test thoroughly after each fix
```

### Week 2: Performance Quick Wins
```bash
# Fix items 8, 10, 11, 13 (easy perf + CLI)
# Estimated time: 1-2 days
# Item 9 can wait if time-constrained
```

### Week 3: Add --version/--help + Clean Up
```bash
# Fix item 12 (--version/--help)
# Remove dead code (items 2.2-2.6)
# Estimated time: 1 day
```

### Later: Everything Else
```bash
# Tackle Tier 3 and Tier 4 as maintenance work
# Or when specific features are requested
```

---

## Summary Statistics

- **Total Issues:** 53
- **Critical Bugs:** 7 (must fix)
- **High Value:** 6 (should fix soon)
- **Code Quality:** ~15 (nice to have)
- **Future Work:** ~25 (low priority)

**Recommendation:** Focus on the first 13 items. That's 25% of the list but captures 80% of the value.

---

*This analysis assumes a solo developer with ~2 weeks of focused time. Adjust priorities based on your use case and timeline.*
