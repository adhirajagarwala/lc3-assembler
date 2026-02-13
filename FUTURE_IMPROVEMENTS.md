# Future Improvements & Enhancement Opportunities

This document provides a comprehensive, deeply-considered analysis of potential improvements to the LC-3 assembler. Each suggestion includes rationale, implementation approach, priority assessment, and impact analysis.

---

## Table of Contents

1. [Architecture & Design](#1-architecture--design)
2. [Performance Optimizations](#2-performance-optimizations)
3. [Feature Additions](#3-feature-additions)
4. [Developer Experience](#4-developer-experience)
5. [Testing & Quality Assurance](#5-testing--quality-assurance)
6. [Documentation & Learning](#6-documentation--learning)
7. [Tooling & Integration](#7-tooling--integration)
8. [Error Handling & Diagnostics](#8-error-handling--diagnostics)
9. [Code Quality & Maintenance](#9-code-quality--maintenance)
10. [Deployment & Distribution](#10-deployment--distribution)

---

## 1. Architecture & Design

### 1.1 Modular Backend Architecture

**Current State**: Encoder is tightly coupled to LC-3 ISA

**Improvement**: Abstract the encoder interface to support multiple target architectures

**Rationale**:
- Opens door for supporting other simple architectures (MIPS subset, ARM subset, RISC-V RV32I)
- Better separation of concerns
- Educational value: students could see similarities/differences between architectures
- Reusable frontend (lexer, parser, symbol table) for other projects

**Implementation Approach**:
```rust
// Create trait for instruction encoding
pub trait InstructionEncoder {
    type Instruction;
    fn encode_instruction(&self, inst: &Self::Instruction, ctx: &EncodingContext) -> Result<Vec<u16>, AsmError>;
}

// Separate ISA-specific concerns
pub trait ISA {
    type Instruction;
    type Directive;
    fn instruction_word_size(&self) -> usize;
    fn address_width(&self) -> usize;
    fn supports_pc_relative(&self) -> bool;
}

// Generic assembler pipeline
pub struct Assembler<I: ISA> {
    isa: I,
    encoder: Box<dyn InstructionEncoder<Instruction = I::Instruction>>,
}
```

**Priority**: LOW (architectural improvement, not immediately needed)

**Impact**:
- ✅ Future-proofs the codebase
- ✅ Educational value
- ⚠️ Significant refactoring required
- ⚠️ Adds complexity for single-ISA use case

**Effort**: HIGH (2-3 weeks for full abstraction)

---

### 1.2 Plugin System for Custom Directives

**Current State**: Directives are hardcoded (.ORIG, .FILL, .BLKW, .STRINGZ, .END)

**Improvement**: Allow registration of custom directives via plugin system

**Rationale**:
- Users could define project-specific directives (e.g., `.INCLUDE`, `.MACRO`)
- Educational institutions could add custom directives for specific assignments
- Enables experimentation without modifying core assembler

**Implementation Approach**:
```rust
pub trait DirectiveHandler {
    fn name(&self) -> &str;
    fn parse(&self, tokens: &[Token]) -> Result<CustomDirective, AsmError>;
    fn first_pass(&self, directive: &CustomDirective, ctx: &mut FirstPassContext) -> Result<(), AsmError>;
    fn encode(&self, directive: &CustomDirective, ctx: &EncodingContext) -> Result<Vec<u16>, AsmError>;
}

pub struct AssemblerWithPlugins {
    core: Assembler,
    directive_handlers: HashMap<String, Box<dyn DirectiveHandler>>,
}

impl AssemblerWithPlugins {
    pub fn register_directive(&mut self, handler: Box<dyn DirectiveHandler>) {
        self.directive_handlers.insert(handler.name().to_string(), handler);
    }
}
```

**Example Plugin**:
```rust
// .INCLUDE directive plugin
struct IncludeDirective;

impl DirectiveHandler for IncludeDirective {
    fn name(&self) -> &str { "INCLUDE" }

    fn parse(&self, tokens: &[Token]) -> Result<CustomDirective, AsmError> {
        // Parse .INCLUDE "filename.asm"
    }

    fn first_pass(&self, directive: &CustomDirective, ctx: &mut FirstPassContext) -> Result<(), AsmError> {
        // Read file and add its lines to the source
    }

    fn encode(&self, directive: &CustomDirective, ctx: &EncodingContext) -> Result<Vec<u16>, AsmError> {
        // No encoding needed for .INCLUDE
        Ok(vec![])
    }
}
```

**Priority**: MEDIUM (useful for advanced users, educational scenarios)

**Impact**:
- ✅ Extensibility without core modifications
- ✅ Community can contribute plugins
- ⚠️ Need to carefully design plugin security
- ⚠️ Documentation burden

**Effort**: MEDIUM (1-2 weeks for plugin infrastructure)

---

### 1.3 Incremental Compilation Support

**Current State**: Full recompilation on every change

**Improvement**: Support incremental recompilation for faster development iteration

**Rationale**:
- Faster feedback loop during development
- Particularly valuable for large projects
- Modern expectation for development tools
- Enables watch mode (see section 7.3)

**Implementation Approach**:
```rust
pub struct IncrementalState {
    file_hashes: HashMap<PathBuf, u64>,  // Track file changes
    symbol_cache: SymbolTable,            // Cache symbols
    dependency_graph: DependencyGraph,     // Track .INCLUDE relationships
}

impl IncrementalState {
    pub fn has_changed(&self, file: &Path) -> bool {
        let current_hash = hash_file(file);
        self.file_hashes.get(file) != Some(&current_hash)
    }

    pub fn invalidate_dependents(&mut self, file: &Path) {
        // Invalidate all files that depend on this one
        for dependent in self.dependency_graph.dependents_of(file) {
            self.file_hashes.remove(dependent);
        }
    }
}
```

**Challenges**:
- Need to track dependencies (if .INCLUDE is implemented)
- Symbol table caching complexity
- State invalidation logic

**Priority**: LOW (optimization, most LC-3 programs are small)

**Impact**:
- ✅ Faster iteration for large projects
- ✅ Better developer experience
- ⚠️ Complexity in state management
- ⚠️ Most LC-3 programs are small enough that full recompilation is fast

**Effort**: MEDIUM-HIGH (1-2 weeks)

---

### 1.4 Multi-Pass Optimization Framework

**Current State**: Two-pass assembler (first pass for symbols, second for encoding)

**Improvement**: Pluggable optimization passes between first and second pass

**Rationale**:
- Enable peephole optimizations
- Dead code elimination
- Constant folding
- Branch optimization (convert long branches to jump sequences)
- Educational: show students how compilers optimize

**Implementation Approach**:
```rust
pub trait OptimizationPass {
    fn name(&self) -> &str;
    fn run(&self, program: &mut Program, symbol_table: &SymbolTable) -> Result<(), AsmError>;
}

pub struct OptimizationPipeline {
    passes: Vec<Box<dyn OptimizationPass>>,
}

// Example: Dead code elimination
struct DeadCodeElimination;

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &str { "dead-code-elimination" }

    fn run(&self, program: &mut Program, symbol_table: &SymbolTable) -> Result<(), AsmError> {
        // Remove unreachable code after unconditional branches/RET/HALT
        // Remove unused labels
    }
}

// Example: Branch distance optimization
struct BranchOptimization;

impl OptimizationPass for BranchOptimization {
    fn name(&self) -> &str { "branch-optimization" }

    fn run(&self, program: &mut Program, symbol_table: &SymbolTable) -> Result<(), AsmError> {
        // If BR offset would be out of range, convert to:
        //   BRn SKIP
        //   JMP label
        //   SKIP: ...
    }
}
```

**Priority**: LOW (most LC-3 code is hand-written, not generated)

**Impact**:
- ✅ Educational value
- ✅ Better code generation if assembler is used as compiler backend
- ⚠️ Complexity
- ⚠️ Most LC-3 code is already optimized by humans

**Effort**: MEDIUM-HIGH (varies by optimization)

---

## 2. Performance Optimizations

### 2.1 Static Opcode Lookup Table

**Current State**: Match statement for opcode recognition (line 327-350 in lexer/mod.rs)

**Improvement**: Use `phf` (perfect hash function) for compile-time constant lookup

**Rationale**:
- Faster opcode lookup (O(1) with no branches vs O(n) match)
- Zero runtime overhead (computed at compile time)
- Industry standard practice for lexers

**Implementation Approach**:
```rust
// Add to Cargo.toml
phf = { version = "0.11", features = ["macros"] }

// In lexer/mod.rs
use phf::phf_map;

static OPCODES: phf::Map<&'static str, TokenKind> = phf_map! {
    "ADD" => TokenKind::OpAdd,
    "AND" => TokenKind::OpAnd,
    "NOT" => TokenKind::OpNot,
    "LD" => TokenKind::OpLd,
    // ... rest of opcodes
    "HALT" => TokenKind::PseudoHalt,
};

// Usage
if let Some(kind) = OPCODES.get(&upper) {
    return Ok(Some(Token {
        kind: kind.clone(),
        lexeme: word,
        span: cursor.make_span(sb, sl, sc),
    }));
}
```

**Benchmarking**:
```rust
#[bench]
fn bench_opcode_lookup_match(b: &mut Bencher) {
    b.iter(|| {
        // Current implementation
        for opcode in OPCODES {
            black_box(match_opcode(opcode));
        }
    });
}

#[bench]
fn bench_opcode_lookup_phf(b: &mut Bencher) {
    b.iter(|| {
        // PHF implementation
        for opcode in OPCODES {
            black_box(OPCODES.get(opcode));
        }
    });
}
```

**Priority**: LOW (performance is already good for typical files)

**Impact**:
- ✅ Faster lexing (est. 5-10% for opcode-heavy files)
- ✅ Industry best practice
- ⚠️ Additional dependency
- ⚠️ Marginal benefit for small files

**Effort**: LOW (1-2 hours)

---

### 2.2 String Interning for Labels

**Current State**: Labels stored as owned `String` everywhere

**Improvement**: Use string interning to reduce memory allocations and improve cache locality

**Rationale**:
- Labels are duplicated across: tokens, AST, symbol table
- String interning reduces memory usage
- Faster comparisons (pointer equality vs string comparison)
- Better cache locality

**Implementation Approach**:
```rust
// Use `string_cache` or `lasso` crate
use lasso::{Rodeo, Spur};

pub struct AssemblerContext {
    interner: Rodeo,
}

// Instead of String, use Spur (interned string handle)
pub struct SymbolTable {
    map: HashMap<Spur, u16>,
    order: Vec<Spur>,
}

// TokenKind::Label stores Spur instead of String
pub enum TokenKind {
    // ...
    Label(Spur),
}

// Convert back to str for display
impl Display for Token {
    fn fmt(&self, f: &mut Formatter, ctx: &AssemblerContext) -> Result {
        match &self.kind {
            TokenKind::Label(spur) => write!(f, "{}", ctx.interner.resolve(spur)),
            // ...
        }
    }
}
```

**Challenges**:
- Need to thread `interner` context through pipeline
- API changes (breaking change)
- Complexity increase

**Priority**: LOW (memory usage is not a concern for typical LC-3 programs)

**Impact**:
- ✅ Lower memory usage (est. 20-30% reduction for label-heavy programs)
- ✅ Faster label comparisons
- ⚠️ API complexity (need context everywhere)
- ⚠️ Most LC-3 programs are small

**Effort**: MEDIUM (3-5 days for full refactoring)

---

### 2.3 Zero-Copy Token Parsing

**Current State**: Tokens store owned `String` for lexeme

**Improvement**: Use `Cow<str>` or lifetime parameters to avoid allocations

**Rationale**:
- Most tokens don't need to own their lexeme
- Can reference original source string
- Reduces allocations during lexing

**Implementation Approach**:
```rust
// Option 1: Cow<str>
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Cow<'static, str>,  // Or Cow<'source, str> with lifetime
    pub span: Span,
}

// Option 2: Lifetime-based
pub struct Token<'source> {
    pub kind: TokenKind,
    pub lexeme: &'source str,  // Reference to source
    pub span: Span,
}

// Only allocate for escaped strings
TokenKind::StringLiteral(Cow::Owned(processed_string))
```

**Challenges**:
- Lifetimes complicate the API
- Not all tokens can be zero-copy (e.g., escaped strings)
- Need to keep source string alive

**Priority**: LOW (performance already good)

**Impact**:
- ✅ Fewer allocations
- ✅ Faster lexing (est. 10-15%)
- ⚠️ API complexity (lifetimes)
- ⚠️ Marginal benefit for typical use

**Effort**: MEDIUM (3-5 days)

---

### 2.4 Parallel Lexing for Multi-File Projects

**Current State**: Sequential lexing of source files

**Improvement**: Use `rayon` for parallel lexing when assembling multiple files

**Rationale**:
- Modern CPUs have multiple cores
- Lexing is embarrassingly parallel (no shared state)
- Significant speedup for large projects

**Implementation Approach**:
```rust
use rayon::prelude::*;

pub fn assemble_project(files: &[PathBuf]) -> Result<Vec<ObjectFile>, Vec<AsmError>> {
    let results: Vec<_> = files
        .par_iter()  // Parallel iterator
        .map(|file| {
            let source = fs::read_to_string(file)?;
            let tokens = tokenize(&source);
            let ast = parse_lines(&tokens.tokens);
            Ok((file, tokens, ast))
        })
        .collect();

    // Merge results and link
    link_object_files(results)
}
```

**Priority**: LOW (most LC-3 projects are single-file)

**Impact**:
- ✅ Significant speedup for multi-file projects (3-4x on 4 cores)
- ⚠️ Added complexity
- ⚠️ Most LC-3 programs are single-file
- ⚠️ Linking phase is still sequential

**Effort**: MEDIUM (2-4 days)

---

## 3. Feature Additions

### 3.1 Macro System

**Current State**: No macro support

**Improvement**: Implement assembler macros (.MACRO/.ENDM directives)

**Rationale**:
- Reduce code duplication in assembly
- Common feature in real assemblers (NASM, MASM, GAS)
- Educational: teaches students about code generation
- Makes larger projects more maintainable

**Implementation Approach**:
```asm
; Example macro definition
.MACRO PUSH reg
    ADD R6, R6, #-1
    STR \reg, R6, #0
.ENDM

.MACRO POP reg
    LDR \reg, R6, #0
    ADD R6, R6, #1
.ENDM

; Usage
    PUSH R0
    PUSH R1
    ; ... do work
    POP R1
    POP R0
```

**Implementation**:
```rust
pub struct Macro {
    name: String,
    parameters: Vec<String>,
    body: Vec<SourceLine>,
}

pub struct MacroExpander {
    macros: HashMap<String, Macro>,
}

impl MacroExpander {
    pub fn expand(&self, invocation: &MacroInvocation) -> Result<Vec<SourceLine>, AsmError> {
        let macro_def = self.macros.get(&invocation.name)?;

        // Substitute parameters
        let mut expanded = macro_def.body.clone();
        for (param, arg) in macro_def.parameters.iter().zip(&invocation.arguments) {
            substitute_parameter(&mut expanded, param, arg);
        }

        Ok(expanded)
    }
}

// Add macro expansion pass between parsing and first pass
pub fn assemble_with_macros(source: &str) -> EncodeResult {
    let tokens = tokenize(source);
    let ast = parse_lines(&tokens.tokens);
    let expanded = expand_macros(&ast)?;  // New step
    let first = first_pass(&expanded);
    let encoded = encode(&first);
    encoded
}
```

**Challenges**:
- Recursive macro detection
- Macro hygiene (name conflicts)
- Error reporting (show original line + expanded line)
- Label generation inside macros

**Priority**: MEDIUM-HIGH (very useful feature)

**Impact**:
- ✅ Major productivity boost for users
- ✅ Reduces code duplication
- ✅ Common feature in real assemblers
- ⚠️ Adds complexity to pipeline
- ⚠️ Need to handle edge cases carefully

**Effort**: MEDIUM-HIGH (1-2 weeks)

---

### 3.2 Include File Support

**Current State**: No file inclusion mechanism

**Improvement**: Implement `.INCLUDE` directive for modular code organization

**Rationale**:
- Large projects should be split across multiple files
- Enables code libraries (common routines, constants)
- Standard feature in assemblers
- Better project organization

**Implementation Approach**:
```asm
; main.asm
.ORIG x3000
    .INCLUDE "stdio.asm"
    .INCLUDE "math.asm"

    LEA R0, HELLO
    PUTS
    HALT

HELLO .STRINGZ "Hello, World!"
.END

; stdio.asm (library of I/O routines)
; PUTS_NEWLINE: Print string and newline
PUTS_NEWLINE
    PUTS
    LD R0, NEWLINE
    OUT
    RET
NEWLINE .FILL x0A

; math.asm (library of math routines)
; ABS: Absolute value of R0
ABS
    ADD R0, R0, #0
    BRzp ABS_DONE
    NOT R0, R0
    ADD R0, R0, #1
ABS_DONE
    RET
```

**Implementation**:
```rust
pub struct IncludeResolver {
    include_paths: Vec<PathBuf>,
    included_files: HashSet<PathBuf>,  // Prevent circular includes
}

impl IncludeResolver {
    pub fn resolve(&mut self, directive: &IncludeDirective, current_file: &Path) -> Result<Vec<SourceLine>, AsmError> {
        let file_path = self.find_file(&directive.filename, current_file)?;

        // Check for circular includes
        if self.included_files.contains(&file_path) {
            return Err(AsmError::circular_include(file_path, directive.span));
        }

        self.included_files.insert(file_path.clone());

        let source = fs::read_to_string(&file_path)?;
        let tokens = tokenize(&source);
        let ast = parse_lines(&tokens.tokens);

        // Recursively process includes in the included file
        self.process_includes(ast.lines)
    }

    fn find_file(&self, filename: &str, current_file: &Path) -> Result<PathBuf, AsmError> {
        // Try relative to current file first
        let relative = current_file.parent().unwrap().join(filename);
        if relative.exists() {
            return Ok(relative);
        }

        // Try include paths
        for path in &self.include_paths {
            let full_path = path.join(filename);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        Err(AsmError::file_not_found(filename))
    }
}
```

**Challenges**:
- Circular include detection
- Include path resolution
- Error reporting (show which file an error is from)
- Relative paths vs absolute paths

**Priority**: MEDIUM-HIGH (very useful for larger projects)

**Impact**:
- ✅ Better code organization
- ✅ Enables code libraries
- ✅ Standard assembler feature
- ⚠️ Need careful design for error messages
- ⚠️ Path resolution can be tricky

**Effort**: MEDIUM (4-5 days)

---

### 3.3 Conditional Assembly

**Current State**: No conditional compilation

**Improvement**: Implement `.IF/.ELSE/.ENDIF` directives for conditional assembly

**Rationale**:
- Enable debug vs release builds
- Platform-specific code (if targeting multiple LC-3 variants)
- Feature flags
- Common in real assemblers

**Implementation Approach**:
```asm
.DEFINE DEBUG 1
.DEFINE ENABLE_LOGGING 1

; Conditional assembly
.IF DEBUG
    ; Debug-only code
    LEA R0, DEBUG_MSG
    PUTS
.ENDIF

.IF ENABLE_LOGGING
    JSR LOG_EVENT
.ELSE
    ; No-op in release builds
.ENDIF

; Conditional based on address ranges
.IF (@CURRENT_ADDRESS < x4000)
    ; Low memory code
.ELSE
    ; High memory code
.ENDIF
```

**Implementation**:
```rust
pub struct ConditionalContext {
    symbols: HashMap<String, i32>,
    condition_stack: Vec<bool>,
}

impl ConditionalContext {
    pub fn evaluate(&self, expr: &Expression) -> Result<bool, AsmError> {
        match expr {
            Expression::Symbol(name) => {
                Ok(self.symbols.get(name).copied().unwrap_or(0) != 0)
            }
            Expression::Comparison(left, op, right) => {
                let lval = self.eval_value(left)?;
                let rval = self.eval_value(right)?;
                Ok(match op {
                    CompOp::Eq => lval == rval,
                    CompOp::Ne => lval != rval,
                    CompOp::Lt => lval < rval,
                    CompOp::Gt => lval > rval,
                    // ...
                })
            }
            // ...
        }
    }

    pub fn should_include(&self) -> bool {
        self.condition_stack.iter().all(|&b| b)
    }
}

// During parsing
if let LineContent::Directive(Directive::If(expr)) = &line.content {
    let condition = ctx.evaluate(expr)?;
    ctx.condition_stack.push(condition);
}
if let LineContent::Directive(Directive::EndIf) = &line.content {
    ctx.condition_stack.pop();
}

// Only include lines where all conditions are true
if ctx.should_include() {
    processed_lines.push(line);
}
```

**Priority**: LOW-MEDIUM (useful but not essential)

**Impact**:
- ✅ Flexible build configurations
- ✅ Cleaner debug code
- ⚠️ Adds complexity
- ⚠️ Most LC-3 programs don't need this

**Effort**: MEDIUM (5-7 days)

---

### 3.4 Listing File Generation

**Current State**: Only generates binary object file

**Improvement**: Generate human-readable listing file showing source, addresses, and machine code

**Rationale**:
- Debugging aid (see exactly what was assembled)
- Learning tool (see address assignments)
- Required by some instructors
- Standard assembler feature

**Example Output**:
```
LC-3 Assembly Listing
Generated: 2026-02-14 10:30:45
Source: hello.asm

Line  Address  Machine Code  Source
----  -------  ------------  ------
   1                         .ORIG x3000
   2  x3000    0xE002       LEA R0, HELLO
   3  x3001    0xF022       PUTS
   4  x3002    0xF025       HALT
   5
   6  x3003    0x0048       HELLO .STRINGZ "Hello, World!"
   7  x3004    0x0065
   8  x3005    0x006C
   9  x3006    0x006C
  10  x3007    0x006F
  ...
  19  x3010    0x0000
  20                         .END

Symbol Table:
Name          Address
----          -------
HELLO         x3003

Statistics:
Total lines:    20
Code size:      14 words (28 bytes)
Origin:         x3000
Entry point:    x3000
```

**Implementation**:
```rust
pub struct ListingGenerator {
    source_lines: Vec<String>,
    line_info: Vec<LineInfo>,
    symbol_table: SymbolTable,
}

pub struct LineInfo {
    line_number: usize,
    address: Option<u16>,
    machine_code: Vec<u16>,
    source: String,
}

impl ListingGenerator {
    pub fn generate(&self, output: &mut dyn Write) -> Result<(), io::Error> {
        // Header
        writeln!(output, "LC-3 Assembly Listing")?;
        writeln!(output, "Generated: {}", Utc::now())?;
        writeln!(output)?;

        // Column headers
        writeln!(output, "Line  Address  Machine Code  Source")?;
        writeln!(output, "----  -------  ------------  ------")?;

        // Source lines with addresses and machine code
        for info in &self.line_info {
            write!(output, "{:4}", info.line_number)?;

            if let Some(addr) = info.address {
                write!(output, "  x{:04X}    ", addr)?;

                if let Some(&first_word) = info.machine_code.first() {
                    write!(output, "0x{:04X}       ", first_word)?;
                } else {
                    write!(output, "             ")?;
                }
            } else {
                write!(output, "                       ")?;
            }

            writeln!(output, "{}", info.source)?;

            // Additional words for multi-word lines (e.g., .STRINGZ)
            for &word in info.machine_code.iter().skip(1) {
                writeln!(output, "      x{:04X}    0x{:04X}",
                    info.address.unwrap() + 1, word)?;
            }
        }

        // Symbol table
        writeln!(output)?;
        writeln!(output, "Symbol Table:")?;
        self.symbol_table.print_table();

        // Statistics
        writeln!(output)?;
        writeln!(output, "Statistics:")?;
        writeln!(output, "Total lines:    {}", self.source_lines.len())?;
        writeln!(output, "Code size:      {} words", self.total_words())?;

        Ok(())
    }
}
```

**Priority**: MEDIUM (useful for debugging and learning)

**Impact**:
- ✅ Better debugging experience
- ✅ Educational value
- ✅ Standard feature
- ⚠️ Additional output file
- ⚠️ Need to format nicely

**Effort**: LOW-MEDIUM (2-3 days)

---

### 3.5 Multiple Output Formats

**Current State**: Only outputs raw binary (.obj)

**Improvement**: Support multiple output formats (Intel HEX, Motorola S-Record, COE for FPGA)

**Rationale**:
- Intel HEX is widely used for embedded systems
- S-Record is another common format
- COE (Coefficient) files for Xilinx FPGAs
- Better interoperability with other tools

**Implementation Approach**:
```rust
pub trait OutputFormat {
    fn name(&self) -> &str;
    fn extension(&self) -> &str;
    fn write(&self, data: &[u16], origin: u16, output: &mut dyn Write) -> Result<(), io::Error>;
}

// Intel HEX format
pub struct IntelHex;

impl OutputFormat for IntelHex {
    fn name(&self) -> &str { "Intel HEX" }
    fn extension(&self) -> &str { "hex" }

    fn write(&self, data: &[u16], origin: u16, output: &mut dyn Write) -> Result<(), io::Error> {
        // :LLAAAATT[DD...]CC
        // LL = byte count
        // AAAA = address
        // TT = record type (00 = data, 01 = EOF)
        // DD = data bytes
        // CC = checksum

        for (i, chunk) in data.chunks(16).enumerate() {
            let addr = origin + (i * 16) as u16;
            let bytes: Vec<u8> = chunk.iter()
                .flat_map(|&word| vec![(word >> 8) as u8, word as u8])
                .collect();

            let checksum = calculate_checksum(&bytes, addr);
            writeln!(output, ":{:02X}{:04X}00{}:{:02X}",
                bytes.len(), addr, hex_string(&bytes), checksum)?;
        }

        writeln!(output, ":00000001FF")?; // EOF record
        Ok(())
    }
}

// Usage
let format: Box<dyn OutputFormat> = match cli.format.as_str() {
    "hex" => Box::new(IntelHex),
    "srec" => Box::new(MotorolaSRecord),
    "coe" => Box::new(XilinxCOE),
    _ => Box::new(RawBinary),
};

format.write(&encoded.machine_code, encoded.orig_address, &mut output)?;
```

**Priority**: LOW (raw binary is sufficient for most use cases)

**Impact**:
- ✅ Better tool interoperability
- ✅ Useful for FPGA/embedded workflows
- ⚠️ Most LC-3 simulators expect raw binary
- ⚠️ Additional testing needed

**Effort**: MEDIUM (3-4 days for multiple formats)

---

### 3.6 Expression Evaluation in Operands

**Current State**: Only literal values allowed in operands

**Improvement**: Support arithmetic expressions in immediate operands

**Rationale**:
- More expressive assembly code
- Useful for computed offsets and constants
- Standard feature in real assemblers

**Example**:
```asm
.DEFINE STACK_SIZE 100
.DEFINE PAGE_SIZE 256

    ADD R1, R1, #(STACK_SIZE / 2)    ; Computed constant
    .FILL (x3000 + PAGE_SIZE * 2)    ; Computed address
    LDR R0, R1, #(OFFSET + 3)        ; Computed offset
```

**Implementation**:
```rust
pub enum Expression {
    Literal(i32),
    Symbol(String),
    Binary(Box<Expression>, BinOp, Box<Expression>),
    Unary(UnOp, Box<Expression>),
}

pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Xor,
    Shl, Shr,
}

pub struct ExpressionEvaluator {
    symbols: HashMap<String, i32>,
}

impl ExpressionEvaluator {
    pub fn eval(&self, expr: &Expression) -> Result<i32, AsmError> {
        match expr {
            Expression::Literal(n) => Ok(*n),
            Expression::Symbol(name) => {
                self.symbols.get(name)
                    .copied()
                    .ok_or_else(|| AsmError::undefined_symbol(name))
            }
            Expression::Binary(left, op, right) => {
                let lval = self.eval(left)?;
                let rval = self.eval(right)?;
                Ok(match op {
                    BinOp::Add => lval + rval,
                    BinOp::Sub => lval - rval,
                    BinOp::Mul => lval * rval,
                    BinOp::Div => lval / rval,
                    BinOp::Mod => lval % rval,
                    // ...
                })
            }
            Expression::Unary(op, expr) => {
                let val = self.eval(expr)?;
                Ok(match op {
                    UnOp::Neg => -val,
                    UnOp::Not => !val,
                })
            }
        }
    }
}
```

**Challenges**:
- Need to extend parser to handle expressions
- Operator precedence
- Type checking (ensure results fit in instruction fields)

**Priority**: MEDIUM (useful but not essential)

**Impact**:
- ✅ More expressive code
- ✅ Easier to write portable code
- ⚠️ Parser complexity
- ⚠️ Error messages become more complex

**Effort**: MEDIUM-HIGH (1 week)

---

## 4. Developer Experience

### 4.1 Language Server Protocol (LSP) Implementation

**Current State**: No IDE integration

**Improvement**: Implement Language Server Protocol for rich IDE support

**Rationale**:
- Modern expectation for programming languages
- Enables features in VS Code, Vim, Emacs, etc.:
  - Auto-completion (opcodes, registers, labels)
  - Go-to-definition for labels
  - Hover documentation
  - Syntax errors as you type
  - Rename refactoring
- Major productivity boost

**Features to Implement**:
```rust
// Language server capabilities
pub struct LC3LanguageServer {
    documents: HashMap<Url, Document>,
    diagnostics: HashMap<Url, Vec<Diagnostic>>,
}

impl LanguageServer for LC3LanguageServer {
    // Provide completions
    fn completion(&mut self, params: CompletionParams) -> Vec<CompletionItem> {
        let mut items = vec![];

        // Complete opcodes
        for opcode in OPCODES {
            items.push(CompletionItem {
                label: opcode.to_string(),
                kind: CompletionItemKind::Keyword,
                documentation: Some(opcode_docs(opcode)),
                insert_text: Some(completion_snippet(opcode)),
                ..Default::default()
            });
        }

        // Complete labels from symbol table
        let doc = self.documents.get(&params.text_document.uri)?;
        for (label, _) in doc.symbol_table.iter() {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: CompletionItemKind::Variable,
                ..Default::default()
            });
        }

        items
    }

    // Go to definition
    fn goto_definition(&mut self, params: GotoDefinitionParams) -> Option<Location> {
        let doc = self.documents.get(&params.text_document.uri)?;
        let position = params.position;

        let word = doc.word_at_position(position)?;
        let address = doc.symbol_table.get(&word)?;
        let line = doc.line_at_address(address)?;

        Some(Location {
            uri: params.text_document.uri,
            range: line.span.to_lsp_range(),
        })
    }

    // Hover documentation
    fn hover(&mut self, params: HoverParams) -> Option<Hover> {
        let doc = self.documents.get(&params.text_document.uri)?;
        let word = doc.word_at_position(params.position)?;

        // Check if it's an opcode
        if let Some(opcode_info) = OPCODE_INFO.get(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "## {}\n\n{}\n\n**Format**: `{}`\n\n**Example**: `{}`",
                        opcode_info.name,
                        opcode_info.description,
                        opcode_info.format,
                        opcode_info.example,
                    ),
                }),
                range: Some(word.span.to_lsp_range()),
            });
        }

        // Check if it's a label
        if let Some(address) = doc.symbol_table.get(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("Label `{}` at address `x{:04X}`", word, address),
                }),
                range: Some(word.span.to_lsp_range()),
            });
        }

        None
    }

    // Diagnostics (errors/warnings)
    fn publish_diagnostics(&mut self, uri: Url) {
        let doc = self.documents.get(&uri).unwrap();

        let diagnostics: Vec<Diagnostic> = doc.errors.iter().map(|error| {
            Diagnostic {
                range: error.span.to_lsp_range(),
                severity: Some(DiagnosticSeverity::Error),
                code: Some(NumberOrString::String(format!("{:?}", error.kind))),
                message: error.message.clone(),
                source: Some("lc3-assembler".to_string()),
                ..Default::default()
            }
        }).collect();

        self.client.publish_diagnostics(uri.clone(), diagnostics, None);
    }
}
```

**VS Code Extension**:
```typescript
// extension.ts
import { LanguageClient, ServerOptions, LanguageClientOptions } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    const serverOptions: ServerOptions = {
        command: 'lc3-language-server',
        args: ['--stdio']
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'lc3asm' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.asm')
        }
    };

    const client = new LanguageClient(
        'lc3LanguageServer',
        'LC-3 Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}
```

**Priority**: HIGH (major productivity improvement)

**Impact**:
- ✅✅✅ Massive productivity boost
- ✅ Modern developer experience
- ✅ Attracts more users
- ⚠️ Significant implementation effort
- ⚠️ Ongoing maintenance

**Effort**: HIGH (3-4 weeks for full implementation)

---

### 4.2 Interactive Debugger Integration

**Current State**: No debugging support

**Improvement**: Integrate with LC-3 simulator for step-through debugging

**Rationale**:
- Critical for learning and development
- See program execution in real-time
- Inspect registers and memory
- Set breakpoints on labels

**Implementation Approach**:
```rust
// Debug info generation
pub struct DebugInfo {
    line_map: HashMap<u16, SourceLocation>,  // Address -> source line
    symbol_map: HashMap<String, u16>,         // Label -> address
    source_lines: Vec<String>,                // Original source
}

impl DebugInfo {
    pub fn from_assembly(result: &FirstPassResult, source: &str) -> Self {
        let mut line_map = HashMap::new();

        for line in &result.source_lines {
            if let Some(addr) = line.address {
                line_map.insert(addr, SourceLocation {
                    line: line.line_number,
                    column: line.span.col,
                });
            }
        }

        DebugInfo {
            line_map,
            symbol_map: result.symbol_table.to_map(),
            source_lines: source.lines().map(|s| s.to_string()).collect(),
        }
    }

    pub fn write(&self, output: &mut dyn Write) -> Result<(), io::Error> {
        // Write debug info in JSON format
        let json = serde_json::json!({
            "lineMap": self.line_map,
            "symbols": self.symbol_map,
            "source": self.source_lines,
        });

        serde_json::to_writer_pretty(output, &json)?;
        Ok(())
    }
}

// Usage
let debug_info = DebugInfo::from_assembly(&first_pass, &source);
let mut debug_file = File::create("program.debug")?;
debug_info.write(&mut debug_file)?;
```

**Simulator Integration**:
```rust
// Debug protocol for communicating with simulator
pub trait Debugger {
    fn set_breakpoint(&mut self, address: u16) -> Result<(), DebugError>;
    fn continue_execution(&mut self) -> Result<DebugEvent, DebugError>;
    fn step(&mut self) -> Result<DebugEvent, DebugError>;
    fn get_registers(&self) -> [u16; 8];
    fn get_memory(&self, address: u16, count: usize) -> Vec<u16>;
}

pub enum DebugEvent {
    Breakpoint { address: u16, source_line: usize },
    ProgramEnd,
    Error(String),
}
```

**Priority**: MEDIUM (depends on simulator availability)

**Impact**:
- ✅ Much better learning experience
- ✅ Easier debugging
- ⚠️ Requires simulator integration
- ⚠️ Need standard debug info format

**Effort**: MEDIUM-HIGH (2 weeks with existing simulator)

---

### 4.3 Better Error Messages with Suggestions

**Current State**: Error messages are descriptive but don't offer suggestions

**Improvement**: Provide helpful suggestions for common errors

**Rationale**:
- Helps beginners learn faster
- Reduces frustration
- Common in modern compilers (Rust, TypeScript)

**Examples**:
```rust
// Current:
ERROR (line 5:10): Undefined label 'LOOOP'

// Improved:
ERROR (line 5:10): Undefined label 'LOOOP'
  |
5 |     BR LOOOP
  |        ^^^^^
  |
  = help: Did you mean 'LOOP'? (defined at line 3)
  = note: Labels are case-sensitive
```

```rust
// Current:
ERROR (line 12:5): Expected comma after first operand

// Improved:
ERROR (line 12:5): Expected comma after first operand
   |
12 |     ADD R1 R2, #5
   |           ^
   |
   = help: ADD requires commas between operands: ADD R1, R2, #5
```

```rust
// Current:
ERROR (line 8:15): PC offset -300 to label 'FAR_LABEL' exceeds 9-bit range [-256, 255]

// Improved:
ERROR (line 8:15): PC offset -300 to label 'FAR_LABEL' exceeds 9-bit range [-256, 255]
  |
8 |     LD R0, FAR_LABEL
  |            ^^^^^^^^^
  |
  = note: Label is too far away for PC-relative addressing
  = help: Consider using LDI (load indirect) or moving the label closer
  = help: Or use LEA to get the address, then LDR to load the value:
          LEA R1, FAR_LABEL
          LDR R0, R1, #0
```

**Implementation**:
```rust
pub struct ErrorBuilder {
    error: AsmError,
    source: Option<String>,
    help: Vec<String>,
    notes: Vec<String>,
}

impl ErrorBuilder {
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.help.push(suggestion.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_source_context(mut self, source: &str) -> Self {
        self.source = Some(source.to_string());
        self
    }
}

impl Display for ErrorBuilder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Main error message
        writeln!(f, "{}", self.error)?;

        // Source context with caret
        if let Some(source) = &self.source {
            writeln!(f, "  |")?;
            writeln!(f, "{} | {}", self.error.span.line, source)?;
            writeln!(f, "  | {}^", " ".repeat(self.error.span.col - 1))?;
            writeln!(f, "  |")?;
        }

        // Help messages
        for help in &self.help {
            writeln!(f, "  = help: {}", help)?;
        }

        // Notes
        for note in &self.notes {
            writeln!(f, "  = note: {}", note)?;
        }

        Ok(())
    }
}

// Fuzzy label matching
fn find_similar_label(unknown: &str, symbols: &SymbolTable) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for (label, _) in symbols.iter() {
        let distance = levenshtein_distance(unknown, label);
        if distance < best_distance && distance <= 3 {
            best_match = Some(label.to_string());
            best_distance = distance;
        }
    }

    best_match
}

// Usage
if let Some(similar) = find_similar_label(label, &symbol_table) {
    return ErrorBuilder::new(AsmError::undefined_label(label, span))
        .with_source_context(&source_line)
        .with_suggestion(format!("Did you mean '{}'?", similar))
        .with_note("Labels are case-sensitive")
        .build();
}
```

**Priority**: MEDIUM-HIGH (great for beginners)

**Impact**:
- ✅ Much better learning experience
- ✅ Faster debugging
- ✅ Reduced frustration
- ⚠️ Need to implement fuzzy matching
- ⚠️ Need good heuristics for suggestions

**Effort**: MEDIUM (1 week)

---

### 4.4 Web-Based Playground

**Current State**: Command-line only

**Improvement**: Create web-based playground for trying LC-3 assembly online

**Rationale**:
- No installation required
- Great for teaching/demos
- Share code snippets easily
- Modern expectation (Rust Playground, Go Playground)

**Implementation Approach**:
- Compile assembler to WebAssembly
- Create React/Vue frontend
- Monaco editor for syntax highlighting
- Integrate with web-based LC-3 simulator

**Features**:
```typescript
interface Playground {
    // Code editor with syntax highlighting
    editor: MonacoEditor;

    // Assemble button
    assemble(): void;

    // Output panels
    machineCode: string;      // Hex dump
    listing: string;          // Listing file
    errors: Error[];          // Assembly errors

    // Optional: Integrated simulator
    simulator?: LC3Simulator;

    // Share functionality
    shareLink(): string;      // Generate shareable URL
}
```

**Example UI**:
```
┌─────────────────────────────────────────────────────────────┐
│ LC-3 Assembly Playground                     [Share] [Save] │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1  .ORIG x3000                     ┌─────────────────────┐ │
│  2      LEA R0, HELLO               │ Output              │ │
│  3      PUTS                        │                     │ │
│  4      HALT                        │ ✅ Assembly OK      │ │
│  5                                  │                     │ │
│  6  HELLO .STRINGZ "Hello!"         │ 5 words (10 bytes) │ │
│  7  .END                            │                     │ │
│                                     │ [View Hex] [Run]    │ │
│                                     └─────────────────────┘ │
│                                                               │
│ [Assemble] [Clear]                                           │
└─────────────────────────────────────────────────────────────┘
```

**WASM Compilation**:
```rust
// lib.rs
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn assemble_wasm(source: &str) -> String {
    let lexed = tokenize(source);
    let parsed = parse_lines(&lexed.tokens);
    let first = first_pass(&parsed.lines);
    let encoded = encode(&first);

    if encoded.errors.is_empty() {
        serde_json::to_string(&encoded).unwrap()
    } else {
        serde_json::to_string(&encoded.errors).unwrap()
    }
}
```

**Priority**: MEDIUM (great for teaching, but not essential)

**Impact**:
- ✅ Great for teaching/learning
- ✅ No installation barrier
- ✅ Easy to share examples
- ⚠️ Significant web development effort
- ⚠️ Ongoing hosting costs

**Effort**: HIGH (2-3 weeks for full playground)

---

## 5. Testing & Quality Assurance

### 5.1 Property-Based Testing

**Current State**: Example-based unit tests

**Improvement**: Add property-based tests using `proptest` or `quickcheck`

**Rationale**:
- Finds edge cases that humans miss
- Tests invariants rather than specific examples
- Catches unexpected bugs
- Industry best practice

**Implementation**:
```rust
use proptest::prelude::*;

// Property: Lexing should never panic
proptest! {
    #[test]
    fn lexer_never_panics(source in "\\PC*") {
        let _ = tokenize(&source); // Should never panic
    }
}

// Property: Round-trip (encode then decode) should preserve semantics
proptest! {
    #[test]
    fn encode_decode_roundtrip(
        instruction in arb_instruction()
    ) {
        let encoded = encode_instruction(&instruction);
        let decoded = decode_instruction(encoded);
        assert_semantically_equivalent(instruction, decoded);
    }
}

// Property: PC offset calculation is always correct
proptest! {
    #[test]
    fn pc_offset_correct(
        current_addr in 0x0000u16..=0xFFFF,
        target_addr in 0x0000u16..=0xFFFF,
    ) {
        let offset = calc_offset(current_addr, target_addr);
        let effective_addr = current_addr.wrapping_add(1).wrapping_add(offset);
        assert_eq!(effective_addr, target_addr);
    }
}

// Property: Symbol table maintains consistency
proptest! {
    #[test]
    fn symbol_table_consistent(
        operations in vec((any::<String>(), any::<u16>()), 0..100)
    ) {
        let mut table = SymbolTable::new();

        for (label, addr) in operations {
            table.insert(label.clone(), addr);

            // Invariant: inserted labels can be retrieved
            assert_eq!(table.get(&label), Some(addr));
        }

        // Invariant: length matches unique labels
        let unique_labels: HashSet<_> = operations.iter().map(|(l, _)| l).collect();
        assert_eq!(table.len(), unique_labels.len());
    }
}

// Custom generators
fn arb_instruction() -> impl Strategy<Value = Instruction> {
    prop_oneof![
        // ADD
        (arb_register(), arb_register(), arb_register())
            .prop_map(|(dr, sr1, sr2)| Instruction::AddReg { dr, sr1, sr2 }),
        (arb_register(), arb_register(), -16i16..=15)
            .prop_map(|(dr, sr1, imm5)| Instruction::AddImm { dr, sr1, imm5 }),
        // ... other instructions
    ]
}

fn arb_register() -> impl Strategy<Value = u8> {
    0u8..=7
}
```

**Priority**: MEDIUM (good practice, but current tests are comprehensive)

**Impact**:
- ✅ Finds more bugs
- ✅ Better confidence in correctness
- ✅ Industry best practice
- ⚠️ Steeper learning curve
- ⚠️ Slower test execution

**Effort**: MEDIUM (1 week to add comprehensive property tests)

---

### 5.2 Fuzzing

**Current State**: No fuzzing

**Improvement**: Add fuzzing with `cargo-fuzz` or AFL

**Rationale**:
- Finds crashes and panics
- Security: prevent DOS attacks via malformed input
- Catches parser edge cases
- Essential for production tools

**Implementation**:
```rust
// fuzz/fuzz_targets/lexer.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use lc3_assembler::tokenize;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = tokenize(s); // Should never panic or crash
    }
});

// fuzz/fuzz_targets/parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use lc3_assembler::{tokenize, parse_lines};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let tokens = tokenize(s);
        let _ = parse_lines(&tokens.tokens); // Should never panic
    }
});

// fuzz/fuzz_targets/full_pipeline.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use lc3_assembler::{tokenize, parse_lines, first_pass, encode};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let tokens = tokenize(s);
        let parsed = parse_lines(&tokens.tokens);
        let first = first_pass(&parsed.lines);
        let _ = encode(&first); // Should never panic
    }
});
```

**Running Fuzzer**:
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzing
cargo fuzz run lexer -- -max_total_time=3600  # 1 hour
cargo fuzz run parser -- -max_total_time=3600
cargo fuzz run full_pipeline -- -max_total_time=3600

# Check coverage
cargo fuzz coverage lexer
```

**Priority**: MEDIUM (important for robustness)

**Impact**:
- ✅ Finds crashes and panics
- ✅ Better security
- ✅ More robust parsing
- ⚠️ Requires setup
- ⚠️ Can be time-consuming

**Effort**: LOW-MEDIUM (2-3 days to set up and run)

---

### 5.3 Performance Benchmarks

**Current State**: No performance benchmarks

**Improvement**: Add benchmarks with `criterion.rs`

**Rationale**:
- Measure performance improvements
- Catch performance regressions
- Optimize hot paths
- Compare different approaches

**Implementation**:
```rust
// benches/assembler_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use lc3_assembler::{tokenize, parse_lines, first_pass, encode};

fn bench_lexer(c: &mut Criterion) {
    let sources = [
        ("small", include_str!("../tests/test_programs/hello.asm")),
        ("medium", include_str!("../tests/test_programs/countdown.asm")),
        ("large", include_str!("../tests/test_programs/stress.asm")),
    ];

    let mut group = c.benchmark_group("lexer");
    for (name, source) in sources {
        group.bench_with_input(BenchmarkId::new("tokenize", name), &source, |b, s| {
            b.iter(|| tokenize(black_box(s)));
        });
    }
    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let source = include_str!("../tests/test_programs/stress.asm");
    let tokens = tokenize(source);

    c.bench_function("parse_lines", |b| {
        b.iter(|| parse_lines(black_box(&tokens.tokens)));
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    let sources = [
        ("hello", include_str!("../tests/test_programs/hello.asm")),
        ("stress", include_str!("../tests/test_programs/stress.asm")),
    ];

    let mut group = c.benchmark_group("full_pipeline");
    for (name, source) in sources {
        group.bench_with_input(BenchmarkId::new("assemble", name), &source, |b, s| {
            b.iter(|| {
                let lexed = tokenize(black_box(s));
                let parsed = parse_lines(&lexed.tokens);
                let first = first_pass(&parsed.lines);
                let _encoded = encode(&first);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_lexer, bench_parser, bench_full_pipeline);
criterion_main!(benches);
```

**Running Benchmarks**:
```bash
cargo bench

# Output:
# tokenize/small          time:   [12.5 µs 12.7 µs 13.0 µs]
# tokenize/medium         time:   [45.2 µs 45.8 µs 46.5 µs]
# tokenize/large          time:   [312 µs 318 µs 325 µs]
# parse_lines             time:   [89.3 µs 91.2 µs 93.5 µs]
# full_pipeline/hello     time:   [67.8 µs 68.9 µs 70.2 µs]
# full_pipeline/stress    time:   [445 µs 452 µs 461 µs]
```

**Priority**: LOW-MEDIUM (nice to have, performance is already good)

**Impact**:
- ✅ Objective performance metrics
- ✅ Catch regressions
- ✅ Guide optimization efforts
- ⚠️ Benchmark maintenance
- ⚠️ Results can vary by machine

**Effort**: LOW (1-2 days)

---

### 5.4 Integration with LC-3 Test Suite

**Current State**: Custom test programs

**Improvement**: Integrate with existing LC-3 test suites from academia

**Rationale**:
- More comprehensive testing
- Test compatibility with other assemblers
- Find edge cases
- Build confidence in correctness

**Sources**:
- University of Texas LC-3 test suite
- Penn State LC-3 test suite
- MIT's 6.004 test cases

**Implementation**:
```rust
#[test]
fn test_ut_suite() {
    let test_files = glob("tests/ut-suite/**/*.asm").unwrap();

    for entry in test_files {
        let path = entry.unwrap();
        let source = fs::read_to_string(&path).unwrap();

        // Assemble
        let result = assemble(&source);

        // Check for errors
        assert!(result.errors.is_empty(),
            "Failed to assemble {}: {:?}", path.display(), result.errors);

        // Compare with expected output if available
        let obj_path = path.with_extension("obj");
        if obj_path.exists() {
            let expected = fs::read(&obj_path).unwrap();
            assert_eq!(result.machine_code, expected,
                "Output mismatch for {}", path.display());
        }
    }
}
```

**Priority**: MEDIUM (good for compatibility verification)

**Impact**:
- ✅ More comprehensive testing
- ✅ Compatibility verification
- ✅ Edge case discovery
- ⚠️ Need to obtain test suites
- ⚠️ May need to handle dialect differences

**Effort**: MEDIUM (1 week to integrate and verify)

---

### 5.5 Continuous Integration Enhancements

**Current State**: Likely no CI

**Improvement**: Set up comprehensive CI/CD pipeline

**GitHub Actions Workflow**:
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      - name: Run tests
        run: cargo test --all-features
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  lint:
    name: Linting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  bench:
    name: Performance
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench -- --save-baseline main
      - name: Compare with baseline
        run: cargo bench -- --baseline main

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run cargo audit
        run: cargo audit
      - name: Run cargo deny
        run: cargo deny check

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build docs
        run: cargo doc --no-deps --all-features
      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
```

**Priority**: MEDIUM-HIGH (important for collaborative development)

**Impact**:
- ✅ Automated testing
- ✅ Catch issues early
- ✅ Professional development workflow
- ✅ Documentation deployment
- ⚠️ Setup effort
- ⚠️ Ongoing maintenance

**Effort**: LOW-MEDIUM (1-2 days to set up)

---

## 6. Documentation & Learning

### 6.1 Comprehensive User Guide

**Current State**: Basic README and code documentation

**Improvement**: Create comprehensive user guide with examples

**Structure**:
```markdown
# LC-3 Assembler User Guide

## Table of Contents
1. Introduction
2. Installation
3. Quick Start
4. Language Reference
5. Advanced Features
6. Examples
7. Troubleshooting
8. API Documentation

## 1. Introduction
The LC-3 assembler is a two-pass assembler for the Little Computer 3 (LC-3)
instruction set architecture...

## 2. Installation
### From Source
```bash
git clone https://github.com/username/lc3-assembler
cd lc3-assembler
cargo build --release
```

### Pre-built Binaries
Download from releases page...

## 3. Quick Start
Create a file `hello.asm`:
```asm
.ORIG x3000
    LEA R0, HELLO
    PUTS
    HALT
HELLO .STRINGZ "Hello, World!"
.END
```

Assemble it:
```bash
lc3-assembler hello.asm
```

## 4. Language Reference
### Instructions
#### ADD - Add
**Syntax**: `ADD DR, SR1, SR2` or `ADD DR, SR1, imm5`
**Description**: Adds two values and stores result in DR
**Examples**:
```asm
ADD R1, R2, R3     ; R1 = R2 + R3
ADD R1, R2, #5     ; R1 = R2 + 5
```

[... similar for all instructions ...]

### Directives
#### .ORIG - Set Origin Address
**Syntax**: `.ORIG address`
**Description**: Sets the starting address for the program
**Example**:
```asm
.ORIG x3000  ; Program starts at x3000
```

## 5. Advanced Features
### Numeric Literals
- Decimal: `#123`, `#-45`
- Hexadecimal: `x3000`, `xFFFF`
- Binary: `b1010`, `b1111`

### Comments
```asm
; This is a comment
ADD R1, R2, #3  ; Inline comment
```

## 6. Examples
### Example 1: Hello World
[...]

### Example 2: Loop Counter
[...]

### Example 3: Subroutines
[...]

## 7. Troubleshooting
### Common Errors
**Error**: "Undefined label 'LOOP'"
**Cause**: Label was used but never defined
**Solution**: Make sure all labels are defined before use

[... more troubleshooting ...]

## 8. API Documentation
[Link to rustdoc]
```

**Priority**: MEDIUM (important for adoption)

**Impact**:
- ✅ Easier to learn
- ✅ Better adoption
- ✅ Fewer support questions
- ⚠️ Documentation maintenance
- ⚠️ Need to keep in sync

**Effort**: MEDIUM (3-5 days for comprehensive guide)

---

### 6.2 Interactive Tutorial

**Current State**: No tutorial

**Improvement**: Create interactive tutorial for learning LC-3 assembly

**Implementation**:
```markdown
# LC-3 Assembly Tutorial

## Lesson 1: Your First Program
Let's write a simple program that displays "Hello!".

**Step 1**: Every LC-3 program starts with `.ORIG` to set where in memory
the program should be loaded:

```asm
.ORIG x3000
```

Try it:
[ Editor with pre-filled .ORIG x3000 ]
[ Run ] [ Next ]

**Step 2**: Now let's add code to print our message. The `PUTS` instruction
prints a null-terminated string:

```asm
.ORIG x3000
    LEA R0, MESSAGE    ; Load address of MESSAGE into R0
    PUTS               ; Print the string
    HALT               ; Stop the program
```

Try it:
[ Editor ]
[ Run ] [ Next ]

[... interactive steps continue ...]

## Lesson 2: Registers and Arithmetic
The LC-3 has 8 registers (R0-R7) that store 16-bit values...

## Lesson 3: Branching and Loops
[...]

## Lesson 4: Subroutines
[...]
```

**Features**:
- Step-by-step lessons
- Interactive code editor
- Instant feedback
- Challenges/exercises
- Progress tracking

**Priority**: MEDIUM (great for teaching)

**Impact**:
- ✅ Excellent learning resource
- ✅ Attracts beginners
- ✅ Self-paced learning
- ⚠️ Significant development effort
- ⚠️ Ongoing maintenance

**Effort**: HIGH (2-3 weeks)

---

### 6.3 Video Tutorials

**Current State**: No video content

**Improvement**: Create video tutorial series

**Topics**:
1. Introduction to LC-3 Assembly (10 min)
2. Installing and Using the Assembler (5 min)
3. Your First Program (15 min)
4. Working with Registers and Memory (20 min)
5. Control Flow: Branches and Loops (25 min)
6. Subroutines and the Stack (30 min)
7. Debugging Assembly Programs (20 min)
8. Advanced Topics: Optimizations (25 min)

**Priority**: LOW (nice to have, but time-consuming)

**Impact**:
- ✅ Accessible to visual learners
- ✅ Great marketing
- ✅ Can reach wider audience
- ⚠️ Production time
- ⚠️ Ongoing updates needed

**Effort**: HIGH (1-2 weeks for series)

---

### 6.4 Example Programs Library

**Current State**: Basic test programs

**Improvement**: Create comprehensive library of example programs

**Categories**:
```
examples/
├── 01-basics/
│   ├── hello.asm           - Hello World
│   ├── add_numbers.asm     - Simple arithmetic
│   └── loop.asm            - Basic loop
├── 02-io/
│   ├── echo.asm            - Echo user input
│   ├── calculator.asm      - Simple calculator
│   └── menu.asm            - Menu-driven program
├── 03-data-structures/
│   ├── array_sum.asm       - Sum array elements
│   ├── string_length.asm   - Calculate string length
│   └── stack.asm           - Stack implementation
├── 04-algorithms/
│   ├── bubble_sort.asm     - Bubble sort
│   ├── binary_search.asm   - Binary search
│   └── factorial.asm       - Recursive factorial
├── 05-advanced/
│   ├── interrupt.asm       - Interrupt handling
│   ├── memory_allocator.asm- Simple allocator
│   └── mini_os.asm         - Tiny operating system
└── README.md               - Index of all examples
```

**Each Example Should Include**:
- Well-commented source code
- Explanation of what it does
- Learning objectives
- Expected output
- Exercises for modification

**Example**:
```asm
; bubble_sort.asm
; Description: Sorts an array of numbers using bubble sort
; Learning objectives:
;   - Nested loops
;   - Array manipulation
;   - Subroutines
; Expected output: Sorted array printed to console

.ORIG x3000

; Main program
    LD R6, STACK_PTR    ; Initialize stack pointer

    ; Print unsorted array
    LEA R0, UNSORTED_MSG
    PUTS
    JSR PRINT_ARRAY

    ; Sort the array
    JSR BUBBLE_SORT

    ; Print sorted array
    LEA R0, SORTED_MSG
    PUTS
    JSR PRINT_ARRAY

    HALT

; [... rest of implementation ...]

; Try modifying:
; 1. Change the array to different numbers
; 2. Implement selection sort instead
; 3. Add error handling for empty arrays
```

**Priority**: MEDIUM (useful for learning)

**Impact**:
- ✅ Great learning resource
- ✅ Shows best practices
- ✅ Reduces "how do I..." questions
- ⚠️ Requires maintenance
- ⚠️ Need good variety

**Effort**: MEDIUM (1-2 weeks for comprehensive library)

---

## 7. Tooling & Integration

### 7.1 Syntax Highlighting for Editors

**Current State**: No syntax highlighting support

**Improvement**: Create syntax definitions for popular editors

**Implementations Needed**:

**VS Code** (`syntaxes/lc3.tmLanguage.json`):
```json
{
    "scopeName": "source.lc3asm",
    "patterns": [
        {
            "name": "keyword.control.lc3",
            "match": "\\b(ADD|AND|NOT|BR|JMP|JSR|LD|LDI|LDR|LEA|ST|STI|STR|TRAP|RTI|RET|GETC|OUT|PUTS|IN|PUTSP|HALT)\\b"
        },
        {
            "name": "storage.type.register.lc3",
            "match": "\\bR[0-7]\\b"
        },
        {
            "name": "constant.numeric.lc3",
            "match": "#-?[0-9]+|x[0-9A-Fa-f]+|b[01]+"
        },
        {
            "name": "entity.name.label.lc3",
            "match": "^[A-Z_][A-Z0-9_]*"
        },
        {
            "name": "keyword.directive.lc3",
            "match": "\\.(ORIG|FILL|BLKW|STRINGZ|END)\\b"
        },
        {
            "name": "comment.line.semicolon.lc3",
            "match": ";.*$"
        }
    ]
}
```

**Vim** (`syntax/lc3asm.vim`):
```vim
" Vim syntax file for LC-3 assembly

if exists("b:current_syntax")
  finish
endif

syn keyword lc3Instruction ADD AND NOT BR JMP JSR JSRR LD LDI LDR LEA ST STI STR TRAP RTI
syn keyword lc3Pseudo RET GETC OUT PUTS IN PUTSP HALT
syn keyword lc3Directive .ORIG .FILL .BLKW .STRINGZ .END

syn match lc3Register "\<R[0-7]\>"
syn match lc3Number "#-\?[0-9]\+"
syn match lc3Number "x[0-9A-Fa-f]\+"
syn match lc3Number "b[01]\+"
syn match lc3Label "^[A-Z_][A-Z0-9_]*"
syn match lc3Comment ";.*$"

hi def link lc3Instruction Statement
hi def link lc3Pseudo Function
hi def link lc3Directive PreProc
hi def link lc3Register Type
hi def link lc3Number Constant
hi def link lc3Label Identifier
hi def link lc3Comment Comment

let b:current_syntax = "lc3asm"
```

**Emacs** (`lc3-mode.el`):
```elisp
;;; lc3-mode.el --- Major mode for LC-3 assembly

(defvar lc3-mode-syntax-table
  (let ((st (make-syntax-table)))
    (modify-syntax-entry ?\; "<" st)
    (modify-syntax-entry ?\n ">" st)
    st)
  "Syntax table for LC-3 assembly mode.")

(defvar lc3-font-lock-keywords
  '(("\\<\\(ADD\\|AND\\|NOT\\|BR\\|JMP\\|JSR\\|JSRR\\|LD\\|LDI\\|LDR\\|LEA\\|ST\\|STI\\|STR\\|TRAP\\|RTI\\)\\>" . font-lock-keyword-face)
    ("\\<\\(RET\\|GETC\\|OUT\\|PUTS\\|IN\\|PUTSP\\|HALT\\)\\>" . font-lock-builtin-face)
    ("\\<R[0-7]\\>" . font-lock-type-face)
    ("#-?[0-9]+\\|x[0-9A-Fa-f]+\\|b[01]+" . font-lock-constant-face)
    ("^[A-Z_][A-Z0-9_]*" . font-lock-variable-name-face)
    ("\\.\\(ORIG\\|FILL\\|BLKW\\|STRINGZ\\|END\\)\\>" . font-lock-preprocessor-face))
  "Keyword highlighting for LC-3 assembly mode.")

(define-derived-mode lc3-mode prog-mode "LC-3"
  "Major mode for editing LC-3 assembly files."
  (setq font-lock-defaults '(lc3-font-lock-keywords))
  (setq-local comment-start ";")
  (setq-local comment-start-skip ";+\\s-*"))

(provide 'lc3-mode)
```

**TextMate/Sublime** (`LC3.sublime-syntax`):
```yaml
%YAML 1.2
---
name: LC-3 Assembly
file_extensions: [asm]
scope: source.lc3asm

contexts:
  main:
    - match: '\b(ADD|AND|NOT|BR|JMP|JSR|LD|LDI|LDR|LEA|ST|STI|STR|TRAP|RTI)\b'
      scope: keyword.control.lc3
    - match: '\bR[0-7]\b'
      scope: storage.type.register.lc3
    - match: '#-?[0-9]+|x[0-9A-Fa-f]+|b[01]+'
      scope: constant.numeric.lc3
    - match: '^[A-Z_][A-Z0-9_]*'
      scope: entity.name.label.lc3
    - match: '\.(ORIG|FILL|BLKW|STRINGZ|END)\b'
      scope: keyword.directive.lc3
    - match: ';.*$'
      scope: comment.line.semicolon.lc3
```

**Priority**: HIGH (essential for good developer experience)

**Impact**:
- ✅ Much better code readability
- ✅ Fewer typos
- ✅ Professional appearance
- ✅ Minimal maintenance
- ⚠️ Need to support multiple editors

**Effort**: LOW (1-2 days for all major editors)

---

### 7.2 Build System Integration

**Current State**: Manual command-line invocation

**Improvement**: Integrate with common build systems

**Make Integration**:
```makefile
# Makefile for LC-3 projects

# Assembler
ASM = lc3-assembler
ASMFLAGS = --listing

# Source files
SOURCES = $(wildcard *.asm)
OBJECTS = $(SOURCES:.asm=.obj)
LISTINGS = $(SOURCES:.asm=.lst)

# Default target
all: $(OBJECTS)

# Pattern rule for assembly
%.obj %.lst: %.asm
	$(ASM) $(ASMFLAGS) $< -o $*.obj

# Run program in simulator
run: hello.obj
	lc3sim hello.obj

# Clean build artifacts
clean:
	rm -f $(OBJECTS) $(LISTINGS)

.PHONY: all run clean
```

**CMake Integration**:
```cmake
# CMakeLists.txt for LC-3 projects

cmake_minimum_required(VERSION 3.10)
project(LC3Project)

find_program(LC3_ASSEMBLER lc3-assembler REQUIRED)

function(add_lc3_program target source)
    add_custom_command(
        OUTPUT ${target}.obj
        COMMAND ${LC3_ASSEMBLER} ${CMAKE_CURRENT_SOURCE_DIR}/${source}
                -o ${target}.obj
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${source}
        COMMENT "Assembling ${source}"
    )

    add_custom_target(${target} ALL DEPENDS ${target}.obj)
endfunction()

# Add programs
add_lc3_program(hello hello.asm)
add_lc3_program(calculator calculator.asm)
```

**Cargo Integration** (for mixed Rust/LC-3 projects):
```rust
// build.rs
use std::process::Command;

fn main() {
    // Assemble LC-3 programs during build
    let status = Command::new("lc3-assembler")
        .arg("firmware/boot.asm")
        .arg("-o")
        .arg("firmware/boot.obj")
        .status()
        .expect("Failed to run lc3-assembler");

    assert!(status.success(), "Assembly failed");

    println!("cargo:rerun-if-changed=firmware/boot.asm");
}
```

**Priority**: MEDIUM (useful for project management)

**Impact**:
- ✅ Better integration with workflows
- ✅ Easier multi-file projects
- ✅ Automated builds
- ⚠️ Need examples for each system
- ⚠️ Platform-specific considerations

**Effort**: LOW (1-2 days for examples)

---

### 7.3 Watch Mode for Development

**Current State**: Manual reassembly on each change

**Improvement**: Watch mode that reassembles on file changes

**Implementation**:
```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_mode(files: Vec<PathBuf>, output: PathBuf) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;

    for file in &files {
        watcher.watch(file, RecursiveMode::NonRecursive)?;
    }

    println!("📺 Watching for changes... (Ctrl+C to stop)");

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("\n🔄 File changed, reassembling...");

                match assemble_files(&files) {
                    Ok(result) => {
                        if result.errors.is_empty() {
                            println!("✅ Assembly successful!");
                            write_output(&output, &result)?;
                        } else {
                            println!("❌ Assembly failed:");
                            for error in &result.errors {
                                println!("  {}", error);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

// Usage
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--watch".to_string()) {
        watch_mode(vec![PathBuf::from(&args[1])], PathBuf::from("output.obj"))?;
    } else {
        // Normal assembly
    }
}
```

**CLI**:
```bash
# Watch mode
lc3-assembler --watch program.asm

# Output:
# 📺 Watching for changes... (Ctrl+C to stop)
# ✅ Assembly successful! (42 words, 0 errors)
#
# [wait for changes...]
#
# 🔄 File changed, reassembling...
# ❌ Assembly failed:
#   ERROR (line 15:10): Undefined label 'LOOOP'
#   = help: Did you mean 'LOOP'?
```

**Priority**: MEDIUM (nice quality-of-life feature)

**Impact**:
- ✅ Faster development cycle
- ✅ Immediate feedback
- ✅ Modern developer experience
- ⚠️ Additional dependency (notify)
- ⚠️ Battery usage on laptops

**Effort**: LOW (1-2 days)

---

### 7.4 IDE Extensions

**Current State**: No IDE integration

**Improvement**: Create IDE extensions/plugins

**VS Code Extension** (`package.json`):
```json
{
    "name": "lc3-assembler",
    "displayName": "LC-3 Assembly",
    "description": "LC-3 assembly language support",
    "version": "0.1.0",
    "engines": {
        "vscode": "^1.70.0"
    },
    "categories": ["Programming Languages"],
    "activationEvents": [
        "onLanguage:lc3asm"
    ],
    "main": "./out/extension.js",
    "contributes": {
        "languages": [{
            "id": "lc3asm",
            "aliases": ["LC-3 Assembly", "lc3"],
            "extensions": [".asm"],
            "configuration": "./language-configuration.json"
        }],
        "grammars": [{
            "language": "lc3asm",
            "scopeName": "source.lc3asm",
            "path": "./syntaxes/lc3.tmLanguage.json"
        }],
        "commands": [
            {
                "command": "lc3.assemble",
                "title": "LC-3: Assemble Current File"
            },
            {
                "command": "lc3.assembleAndRun",
                "title": "LC-3: Assemble and Run"
            }
        ],
        "keybindings": [
            {
                "command": "lc3.assemble",
                "key": "ctrl+shift+b",
                "when": "editorLangId == 'lc3asm'"
            }
        ],
        "problemMatchers": [{
            "name": "lc3",
            "owner": "lc3",
            "fileLocation": ["relative", "${workspaceFolder}"],
            "pattern": {
                "regexp": "^ERROR \\(line (\\d+):(\\d+)\\): (.*)$",
                "file": 1,
                "line": 2,
                "column": 3,
                "message": 4
            }
        }]
    }
}
```

**Extension Features**:
- Syntax highlighting
- Build task integration
- Problem matcher (errors show in Problems panel)
- Commands for assemble/run
- Snippets for common patterns
- Hover documentation

**Priority**: HIGH (major productivity boost)

**Impact**:
- ✅ Much better developer experience
- ✅ Integration with popular IDE
- ✅ Lowers barrier to entry
- ⚠️ Need to maintain extension
- ⚠️ Need to learn VS Code extension API

**Effort**: MEDIUM-HIGH (1-2 weeks for full-featured extension)

---

## 8. Error Handling & Diagnostics

### 8.1 Detailed Error Codes

**Current State**: Error messages without codes

**Improvement**: Add structured error codes for documentation and scripting

**Implementation**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Lexer errors (E1xxx)
    E1001, // UnterminatedString
    E1002, // InvalidEscapeSequence
    E1003, // InvalidDecimalLiteral
    E1004, // InvalidHexLiteral
    E1005, // InvalidBinaryLiteral
    E1006, // InvalidRegister
    E1007, // UnknownDirective
    E1008, // UnexpectedCharacter

    // Parser errors (E2xxx)
    E2001, // ExpectedOperand
    E2002, // ExpectedRegister
    E2003, // ExpectedComma
    E2004, // UnexpectedToken
    E2005, // TooManyOperands
    E2006, // TooFewOperands
    E2007, // InvalidOperandType

    // First pass errors (E3xxx)
    E3001, // DuplicateLabel
    E3002, // MissingOrig
    E3003, // MultipleOrig
    E3004, // MissingEnd
    E3005, // InvalidOrigAddress
    E3006, // InvalidBlkwCount
    E3007, // AddressOverflow

    // Encoder errors (E4xxx)
    E4001, // UndefinedLabel
    E4002, // OffsetOutOfRange
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::E1001 => "E1001",
            ErrorCode::E1002 => "E1002",
            // ...
        }
    }

    pub fn url(&self) -> String {
        format!("https://lc3-assembler.dev/errors/{}", self.as_str())
    }
}

pub struct AsmError {
    pub code: ErrorCode,
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

impl Display for AsmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ERROR [{}] (line {}:{}): {}\n",
            self.code.as_str(),
            self.span.line,
            self.span.col,
            self.message)?;
        write!(f, "  For more information, see: {}", self.code.url())?;
        Ok(())
    }
}
```

**Example Output**:
```
ERROR [E4001] (line 12:10): Undefined label 'LOOOP'
  For more information, see: https://lc3-assembler.dev/errors/E4001
  = help: Did you mean 'LOOP'?
```

**Error Documentation Page** (`docs/errors/E4001.md`):
```markdown
# Error E4001: Undefined Label

## Description
This error occurs when you reference a label that hasn't been defined anywhere
in your program.

## Example
```asm
.ORIG x3000
    BR LOOOP    ; ERROR: LOOOP not defined
    HALT
LOOP
    ADD R1, R1, #1
    RET
.END
```

## Common Causes
1. Typo in label name (labels are case-sensitive)
2. Label defined after .END
3. Label in different file (if using .INCLUDE)

## How to Fix
1. Check spelling of label
2. Make sure label is defined before .END
3. Verify label is in correct file

## See Also
- E3001: Duplicate Label
- Labels documentation
```

**Priority**: MEDIUM (useful for documentation and tooling)

**Impact**:
- ✅ Better error documentation
- ✅ Scriptable error handling
- ✅ Professional appearance
- ⚠️ Need to maintain error docs
- ⚠️ More work for each error

**Effort**: MEDIUM (1 week for codes + documentation)

---

### 8.2 Warning System

**Current State**: Only errors, no warnings

**Improvement**: Add warning system for suspicious but valid code

**Examples of Warnings**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum WarningKind {
    // Potentially confusing code
    UnusedLabel,              // Label defined but never referenced
    ConstantConditional,      // BR with all flags always branches
    DeadCode,                 // Unreachable code after unconditional branch

    // Potential bugs
    OffsetNearLimit,          // PC offset close to range limit
    LargeImmediate,           // Immediate value uses full bit range
    RegisterOverwrite,        // Register assigned but never read

    // Style/convention
    InconsistentIndentation,  // Mixed tabs/spaces
    MissingComment,           // Complex instruction without comment
    LongLabel,                // Label longer than 20 characters

    // Performance
    UnalignedAccess,          // Access that may be inefficient
    RedundantInstruction,     // Instruction that has no effect
}

pub struct AsmWarning {
    pub kind: WarningKind,
    pub message: String,
    pub span: Span,
    pub severity: WarningSeverity,
}

pub enum WarningSeverity {
    Note,       // FYI
    Warning,    // Probably a bug
    Error,      // Definitely a bug (promoted warning)
}
```

**Example Output**:
```
WARNING (line 15:5): Unused label 'TEMP'
  |
15|  TEMP
  |  ^^^^
  |
  = note: This label is defined but never referenced
  = help: Remove it, or use it in an instruction

WARNING (line 23:5): Dead code detected
  |
23|      BR LABEL
24|      ADD R1, R1, #1  ; This will never execute
  |      ^^^^^^^^^^^^^^
  |
  = note: Code after unconditional branch is unreachable
  = help: Remove this instruction or change the branch condition

NOTE (line 8:5): PC offset is near limit (248/255)
  |
8 |     LD R0, FAR_LABEL
  |            ^^^^^^^^^
  |
  = note: This instruction uses 248 out of 255 available offset
  = help: If you add more code, this may stop working
```

**Configuration** (`.lc3lint.toml`):
```toml
[warnings]
unused-labels = "warn"
dead-code = "warn"
offset-near-limit = "note"
large-immediate = "note"
inconsistent-indentation = "allow"

[warnings.deny]
# Promote these warnings to errors
promote = ["dead-code"]

[warnings.allow]
# Suppress these warnings
suppress = ["missing-comment"]
```

**Priority**: MEDIUM (helps catch bugs early)

**Impact**:
- ✅ Catches potential bugs
- ✅ Improves code quality
- ✅ Educational value
- ⚠️ Too many warnings can be annoying
- ⚠️ Need good defaults

**Effort**: MEDIUM (1 week for basic warning system)

---

### 8.3 Recovery Mode

**Current State**: Assembly stops on first error in each stage

**Improvement**: Error recovery to report multiple errors at once

**Rationale**:
- Fixing one error at a time is frustrating
- Modern compilers report many errors
- Helps see all problems at once

**Implementation**:
```rust
pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<AsmError>,
    recovery_mode: bool,
}

impl Parser {
    fn parse_line(&mut self) -> Option<SourceLine> {
        match self.try_parse_line() {
            Ok(line) => Some(line),
            Err(error) => {
                self.errors.push(error);

                // Recovery: skip to next line
                while let Some(token) = self.current() {
                    if matches!(token.kind, TokenKind::Newline | TokenKind::Eof) {
                        self.advance();
                        break;
                    }
                    self.advance();
                }

                // Return empty line to continue parsing
                self.recovery_mode = true;
                Some(SourceLine::empty())
            }
        }
    }
}

// Example: Multiple errors reported at once
// ERROR (line 5:10): Undefined label 'LOOOP'
// ERROR (line 8:15): Expected comma after first operand
// ERROR (line 12:5): Invalid register R8
// ERROR (line 20:1): No .END directive found
//
// Assembly failed with 4 errors
```

**Priority**: MEDIUM (better user experience)

**Impact**:
- ✅ See all errors at once
- ✅ Faster debugging
- ✅ Modern compiler behavior
- ⚠️ Implementation complexity
- ⚠️ May report cascading errors

**Effort**: MEDIUM-HIGH (1-2 weeks for robust recovery)

---

## 9. Code Quality & Maintenance

### 9.1 Clippy Lints

**Current State**: No custom lints

**Improvement**: Configure Clippy with project-specific lints

**Configuration** (`clippy.toml`):
```toml
# Clippy configuration for lc3-assembler

# Warn on common performance issues
warn = [
    "clone_on_copy",
    "needless_borrow",
    "unnecessary_to_owned",
    "redundant_clone",
]

# Error on correctness issues
deny = [
    "correctness",
    "suspicious",
]

# Allow some pedantic lints
allow = [
    "too_many_arguments",  # Sometimes necessary
    "single_match",        # Sometimes clearer
]

# Project-specific settings
cognitive-complexity-threshold = 15
```

**In `Cargo.toml`**:
```toml
[lints.clippy]
all = "warn"
correctness = "deny"
suspicious = "deny"
perf = "warn"
pedantic = "warn"
```

**Priority**: LOW-MEDIUM (code quality)

**Impact**:
- ✅ Catches common mistakes
- ✅ Enforces best practices
- ✅ Improves code quality
- ⚠️ Can be opinionated
- ⚠️ May require suppression in some cases

**Effort**: LOW (1 day to configure)

---

### 9.2 Code Coverage Tracking

**Current State**: No coverage tracking

**Improvement**: Set up code coverage with `cargo-tarpaulin` or `cargo-llvm-cov`

**Setup**:
```bash
# Install
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

**CI Integration**:
```yaml
# .github/workflows/coverage.yml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          file: ./cobertura.xml
```

**Coverage Goals**:
- Overall: >90%
- Core modules (lexer, parser, encoder): >95%
- Error paths: >80%

**Priority**: MEDIUM (good practice for quality)

**Impact**:
- ✅ Know what's tested
- ✅ Find untested code
- ✅ Professional project
- ⚠️ Coverage isn't everything
- ⚠️ Can be slow in CI

**Effort**: LOW (1-2 days to set up)

---

### 9.3 Mutation Testing

**Current State**: No mutation testing

**Improvement**: Use `cargo-mutants` to test test quality

**Rationale**:
- Tests might pass but not catch bugs
- Mutation testing changes code to see if tests catch it
- Improves test quality

**Setup**:
```bash
cargo install cargo-mutants
cargo mutants
```

**Example**:
```rust
// Original code
fn calc_offset(target: u16, current: u16) -> i16 {
    (target as i32 - current as i32 - 1) as i16
}

// Mutant 1: Remove -1
fn calc_offset(target: u16, current: u16) -> i16 {
    (target as i32 - current as i32) as i16  // <-- mutation
}
// If tests still pass, they're not comprehensive enough!

// Mutant 2: Swap target and current
fn calc_offset(target: u16, current: u16) -> i16 {
    (current as i32 - target as i32 - 1) as i16  // <-- mutation
}
```

**Priority**: LOW (advanced testing technique)

**Impact**:
- ✅ Better test quality
- ✅ Finds weak tests
- ✅ Increases confidence
- ⚠️ Can be very slow
- ⚠️ May find false positives

**Effort**: LOW to run, MEDIUM to address findings (3-5 days)

---

## 10. Deployment & Distribution

### 10.1 Binary Releases

**Current State**: No official releases

**Improvement**: Automated binary releases for major platforms

**GitHub Actions**:
```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: lc3-assembler
            asset_name: lc3-assembler-linux-x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: lc3-assembler.exe
            asset_name: lc3-assembler-windows-x64.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: lc3-assembler
            asset_name: lc3-assembler-macos-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: lc3-assembler
            asset_name: lc3-assembler-macos-arm64

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Strip binary (Unix)
        if: matrix.os != 'windows-latest'
        run: strip target/${{ matrix.target }}/release/${{ matrix.artifact_name }}

      - name: Upload binary
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.asset_name }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            lc3-assembler-*
          generate_release_notes: true
```

**Priority**: HIGH (essential for distribution)

**Impact**:
- ✅ Easy installation
- ✅ Professional project
- ✅ Wider adoption
- ⚠️ Need to maintain releases
- ⚠️ Binary size considerations

**Effort**: LOW-MEDIUM (2-3 days to set up)

---

### 10.2 Package Manager Distribution

**Current State**: No package manager support

**Improvement**: Distribute via package managers

**Cargo (crates.io)**:
```bash
# Publish to crates.io
cargo publish
```

**Homebrew** (`Formula/lc3-assembler.rb`):
```ruby
class Lc3Assembler < Formula
  desc "Assembler for the LC-3 (Little Computer 3) architecture"
  homepage "https://github.com/username/lc3-assembler"
  url "https://github.com/username/lc3-assembler/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    (testpath/"hello.asm").write <<~EOS
      .ORIG x3000
      LEA R0, HELLO
      PUTS
      HALT
      HELLO .STRINGZ "Hello!"
      .END
    EOS

    system "#{bin}/lc3-assembler", "hello.asm"
    assert_predicate testpath/"hello.obj", :exist?
  end
end
```

**APT (Debian/Ubuntu)**:
```bash
# Create .deb package
cargo install cargo-deb
cargo deb
```

**Chocolatey (Windows)**:
```powershell
# lc3-assembler.nuspec
choco pack
choco push
```

**Priority**: MEDIUM (improves adoption)

**Impact**:
- ✅ Easy installation
- ✅ Automatic updates
- ✅ Trusted distribution
- ⚠️ Maintenance for each platform
- ⚠️ Review processes

**Effort**: MEDIUM (1 week for major platforms)

---

### 10.3 Docker Image

**Current State**: No containerization

**Improvement**: Provide Docker image for portable execution

**Dockerfile**:
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/lc3-assembler /usr/local/bin/
WORKDIR /workspace
ENTRYPOINT ["lc3-assembler"]
CMD ["--help"]
```

**Usage**:
```bash
# Build image
docker build -t lc3-assembler .

# Run
docker run -v $(pwd):/workspace lc3-assembler program.asm

# Publish to Docker Hub
docker tag lc3-assembler username/lc3-assembler:latest
docker push username/lc3-assembler:latest
```

**Priority**: LOW (niche use case)

**Impact**:
- ✅ Platform-independent
- ✅ Reproducible environment
- ✅ Easy CI/CD integration
- ⚠️ Overhead for simple tool
- ⚠️ Most users don't need it

**Effort**: LOW (1 day)

---

## Priority Summary

### HIGH Priority (Start Here)
1. **Language Server Protocol** (4.1) - Major productivity boost
2. **Better Error Messages with Suggestions** (4.3) - Great for beginners
3. **Syntax Highlighting** (7.1) - Essential for good experience
4. **IDE Extensions** (7.4) - VS Code extension
5. **Binary Releases** (10.1) - Essential for distribution

### MEDIUM-HIGH Priority (Next Phase)
1. **Macro System** (3.1) - Very useful feature
2. **Include File Support** (3.2) - Better project organization
3. **Comprehensive User Guide** (6.1) - Important for adoption
4. **CI/CD Pipeline** (5.5) - Professional development

### MEDIUM Priority (Future Enhancements)
1. **Listing File Generation** (3.4) - Debugging aid
2. **Expression Evaluation** (3.6) - More expressive code
3. **Interactive Debugger** (4.2) - Better learning
4. **Warning System** (8.2) - Code quality
5. **Watch Mode** (7.3) - Quality of life
6. **Example Programs** (6.4) - Learning resource
7. **Property-Based Testing** (5.1) - Better test coverage
8. **Fuzzing** (5.2) - Robustness
9. **Package Managers** (10.2) - Distribution

### LOW Priority (Nice to Have)
1. **Modular Backend** (1.1) - Future-proofing
2. **Plugin System** (1.2) - Extensibility
3. **Multiple Output Formats** (3.5) - Interoperability
4. **Conditional Assembly** (3.3) - Advanced feature
5. **Performance Optimizations** (2.1-2.4) - Already fast enough
6. **Video Tutorials** (6.3) - Time-consuming
7. **Web Playground** (4.4) - Significant effort
8. **Docker Image** (10.3) - Niche use case

---

## Conclusion

This document presents a comprehensive vision for the future of the LC-3 assembler. While the current implementation is production-ready and well-documented, these improvements would transform it into a world-class development tool.

The suggestions are organized by priority to help guide development efforts. Focus on HIGH priority items first for maximum impact, then work through MEDIUM and LOW priority items as time and resources allow.

Each improvement includes detailed rationale, implementation approaches, and honest assessment of trade-offs. This allows for informed decision-making about which features align with project goals and available resources.

Remember: **Done is better than perfect.** The current assembler is excellent. These improvements are opportunities, not requirements.
