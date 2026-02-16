# Syntax Highlighting Changelog

All notable changes to the LC-3 Assembly syntax highlighting will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-02-15

### Added

#### VS Code Extension
- Complete TextMate grammar (`lc3asm.tmLanguage.json`) with full LC-3 ISA support
- Language configuration with bracket matching, auto-closing pairs, and indentation rules
- Code snippets for common patterns:
  - `lc3prog` - Basic program structure
  - `lc3sub` - Subroutine template with stack management
  - `push` / `pop` - Stack operations
  - `lc3string` - String output template
  - `lc3loop` - Loop structure with counter
  - `lc3if` - Conditional branch structure
  - `trap` - TRAP system call with auto-completion
  - `lc3data` - Data section template
  - `lc3header` - Section header comment block
- Package manifest for distribution via VS Code Marketplace
- Support for code folding with region markers
- Comment toggling support

#### Vim Syntax File
- Complete syntax highlighting (`lc3asm.vim`) with proper color groups
- Case-insensitive instruction matching
- Case-sensitive label handling
- TODO/FIXME/XXX highlighting in comments
- String escape sequence highlighting
- Error highlighting for invalid escape sequences
- Filetype detection for `.asm` and `.lc3` files

#### Sublime Text Syntax
- YAML-based syntax definition (`LC3.sublime-syntax`)
- Full pattern matching for all language features
- Proper scope naming for theme compatibility
- String context with escape sequence handling
- Error detection for unterminated strings

#### Emacs Major Mode
- Complete major mode (`lc3-mode.el`) with font-lock support
- Automatic indentation (labels at column 0, instructions indented)
- Comment handling with proper syntax table
- Customizable indentation settings
- Case-insensitive instruction matching
- Auto-mode-alist registration for `.asm` and `.lc3` files

#### Language Coverage
- **All Opcodes**: ADD, AND, NOT, BR (all variants), JMP, JSR, JSRR, LD, LDI, LDR, LEA, ST, STI, STR, TRAP, RTI
- **All Pseudo-ops**: RET, GETC, OUT, PUTS, IN, PUTSP, HALT
- **All Directives**: .ORIG, .END, .FILL, .BLKW, .STRINGZ
- **Branch Variants**: BR, BRn, BRz, BRp, BRnz, BRnp, BRzp, BRnzp
- **Numeric Literals**: Decimal (#123), hexadecimal (x3000), binary (b1010)
- **String Literals**: Full escape sequence support (\n, \t, \r, \\, \", \0)
- **Registers**: R0-R7 with distinct highlighting
- **Labels**: Case-sensitive identifier support
- **Comments**: Semicolon-style line comments

#### Documentation
- Comprehensive README with installation instructions for all editors
- Language reference with syntax elements table
- Example code demonstrating all features
- Troubleshooting guide for common issues
- Color customization guide
- Contributing guidelines

#### Installation Scripts
- Unix/Linux/macOS installation script (`install.sh`)
- Windows batch installation script (`install.bat`)
- Automated installation for all supported editors
- Individual editor installation support

#### Testing
- Complete test file (`test.asm`) covering all language features
- Organized into 13 sections testing different aspects
- Real-world examples (Fibonacci subroutine, string printing)
- All numeric literal formats
- All escape sequences
- Complex nested structures

### Technical Details

#### Scope Naming Convention
Follows TextMate and Sublime Text conventions:
- `source.lc3asm` - Root scope
- `comment.line.semicolon.lc3asm` - Comments
- `keyword.control.directive.lc3asm` - Directives
- `keyword.operator.instruction.*.lc3asm` - Instructions
- `keyword.control.branch.lc3asm` - Branch instructions
- `keyword.control.pseudo.lc3asm` - Pseudo-operations
- `variable.language.register.lc3asm` - Registers
- `constant.numeric.*.lc3asm` - Numeric literals
- `string.quoted.double.lc3asm` - String literals
- `constant.character.escape.lc3asm` - Escape sequences
- `entity.name.label.lc3asm` - Label definitions
- `variable.other.label.lc3asm` - Label references

#### Color Mapping
- Directives → Preprocessor (purple/pink)
- Instructions → Keywords (blue/cyan)
- Branches → Control flow (purple)
- Pseudo-ops → Functions (yellow)
- Registers → Types (green/teal)
- Numbers → Constants (orange/brown)
- Strings → Strings (orange/red)
- Labels → Variables (light blue)
- Comments → Comments (gray/green)

#### File Size
- VS Code grammar: ~6 KB
- Vim syntax: ~5 KB
- Sublime syntax: ~5 KB
- Emacs mode: ~7 KB
- Total package: ~30 KB (excluding documentation)

### Design Decisions

1. **Case Sensitivity**: Instructions are case-insensitive (can be ADD, add, or Add), but labels are case-sensitive (LOOP ≠ loop) to match LC-3 assembler behavior.

2. **Label Detection**: Labels at the start of a line are highlighted as definitions, while labels used as operands are highlighted as references.

3. **Branch Variants**: All 8 branch variants (BR, BRn, BRz, BRp, BRnz, BRnp, BRzp, BRnzp) are explicitly listed for proper highlighting.

4. **Numeric Literals**: Three formats supported with proper two's complement interpretation:
   - Decimal: `#` prefix, signed values (-32768 to 32767)
   - Hexadecimal: `x` prefix, 16-bit values (0x0000 to 0xFFFF)
   - Binary: `b` prefix, 16-bit values

5. **Escape Sequences**: Support for standard C-style escapes: \n, \r, \t, \\, \", \0. Invalid escapes are highlighted as errors.

6. **Editor Features**:
   - VS Code: Focus on IntelliSense and productivity (snippets, auto-completion)
   - Vim: Focus on precise syntax highlighting and traditional editing
   - Sublime: Focus on visual clarity and theme compatibility
   - Emacs: Focus on integration with Emacs workflow (indentation, major mode)

### Quality Assurance

- ✅ Tested with popular color themes (Dark+, Monokai, Solarized)
- ✅ Verified on multiple editors (VS Code, Vim, Sublime, Emacs)
- ✅ Comprehensive test file covering all features
- ✅ Cross-platform installation scripts (Unix, Windows)
- ✅ Documentation reviewed for clarity and completeness

### Future Enhancements (Not in v1.0)

- Language Server Protocol (LSP) for advanced features
- Semantic highlighting for labels (distinguish between jumps and data)
- Integration with LC-3 assembler for real-time error checking
- Symbol navigation (go-to-definition for labels)
- Hover documentation for instructions
- Inline assembly documentation
- Debugger integration markers
- More advanced snippets (interrupt handlers, complex data structures)

---

## Version History

### [1.0.0] - 2026-02-15
Initial release with full editor support (VS Code, Vim, Sublime Text, Emacs)

---

## Contributing

See [README.md](README.md) for contribution guidelines.

## License

MIT License - See project root for details
