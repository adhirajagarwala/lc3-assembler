# LC-3 Assembly Syntax Highlighting

Professional syntax highlighting for LC-3 (Little Computer 3) assembly language across multiple editors and IDEs.

## Features

✨ **Comprehensive Language Support**
- All LC-3 opcodes (ADD, AND, NOT, BR, JMP, JSR, LD, LDI, LDR, LEA, ST, STI, STR, TRAP, RTI)
- All pseudo-operations (RET, GETC, OUT, PUTS, IN, PUTSP, HALT)
- All directives (.ORIG, .END, .FILL, .BLKW, .STRINGZ)
- Branch variants (BR, BRn, BRz, BRp, BRnz, BRnp, BRzp, BRnzp)
- Numeric literals (decimal `#123`, hexadecimal `x3000`, binary `b1010`)
- String literals with escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`, `\0`)
- Registers (R0-R7)
- Labels (uppercase identifiers)
- Comments (semicolon-style)

🎨 **Editor Support**
- **VS Code**: Full extension with IntelliSense, snippets, and bracket matching
- **Vim**: Complete syntax highlighting with proper colorization
- **Sublime Text**: YAML-based syntax definition
- **Emacs**: Major mode with indentation and font-lock support

---

## Installation

### Visual Studio Code

#### Method 1: Manual Installation (Recommended for Development)

1. **Copy the extension files:**
   ```bash
   mkdir -p ~/.vscode/extensions/lc3-assembly-1.0.0
   cp vscode/* ~/.vscode/extensions/lc3-assembly-1.0.0/
   ```

2. **Reload VS Code:**
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "Reload Window" and press Enter

3. **Open any `.asm` file** and the syntax highlighting will activate automatically

#### Method 2: Package as VSIX (For Distribution)

1. **Install vsce (VS Code Extension Manager):**
   ```bash
   npm install -g @vscode/vsce
   ```

2. **Package the extension:**
   ```bash
   cd vscode
   vsce package
   ```

3. **Install the generated `.vsix` file:**
   - In VS Code: Extensions view → `...` menu → "Install from VSIX"
   - Or: `code --install-extension lc3-assembly-1.0.0.vsix`

#### Features in VS Code:
- ✅ Syntax highlighting with semantic colors
- ✅ Code snippets (`lc3prog`, `lc3sub`, `push`, `pop`, `lc3loop`, etc.)
- ✅ Auto-closing brackets and quotes
- ✅ Comment toggling (`Ctrl+/`)
- ✅ Code folding with region markers
- ✅ Bracket matching
- ✅ IntelliSense-like features

**Available Snippets:**
- `lc3prog` - Basic program structure
- `lc3sub` - Subroutine template with stack management
- `push` - Push register to stack
- `pop` - Pop register from stack
- `lc3string` - String output template
- `lc3loop` - Loop structure with counter
- `lc3if` - Conditional branch (if-else)
- `trap` - TRAP system call
- `lc3data` - Data section template
- `lc3header` - Section header comment block

---

### Vim

1. **Create syntax directory if it doesn't exist:**
   ```bash
   mkdir -p ~/.vim/syntax
   mkdir -p ~/.vim/ftdetect
   ```

2. **Copy the syntax file:**
   ```bash
   cp vim/lc3asm.vim ~/.vim/syntax/
   ```

3. **Create filetype detection:**
   ```bash
   echo 'au BufRead,BufNewFile *.asm set filetype=lc3asm' > ~/.vim/ftdetect/lc3asm.vim
   ```

4. **Reload Vim or restart**

**Optional:** Add to your `.vimrc` for additional customization:
```vim
" LC-3 Assembly settings
autocmd FileType lc3asm setlocal commentstring=;\ %s
autocmd FileType lc3asm setlocal tabstop=8 shiftwidth=4 expandtab
```

#### Features in Vim:
- ✅ Full syntax highlighting
- ✅ Comment support
- ✅ Proper case handling (case-insensitive instructions, case-sensitive labels)
- ✅ TODO/FIXME/XXX highlighting in comments
- ✅ String escape sequence highlighting
- ✅ Error highlighting for invalid escape sequences

---

### Sublime Text

1. **Locate your Packages directory:**
   - **Windows:** `%APPDATA%\Sublime Text\Packages\User`
   - **macOS:** `~/Library/Application Support/Sublime Text/Packages/User`
   - **Linux:** `~/.config/sublime-text/Packages/User`

2. **Copy the syntax file:**
   ```bash
   cp sublime/LC3.sublime-syntax "/path/to/Packages/User/"
   ```

3. **Restart Sublime Text**

4. **Open a `.asm` file and select:**
   - View → Syntax → LC-3 Assembly

**Tip:** Sublime Text will remember the syntax choice for `.asm` files.

#### Features in Sublime Text:
- ✅ Full syntax highlighting
- ✅ Proper scope naming for theme compatibility
- ✅ String escape sequence handling
- ✅ Error detection for unterminated strings

---

### Emacs

1. **Create or locate your Emacs lisp directory:**
   ```bash
   mkdir -p ~/.emacs.d/lisp
   ```

2. **Copy the mode file:**
   ```bash
   cp emacs/lc3-mode.el ~/.emacs.d/lisp/
   ```

3. **Add to your `init.el` or `.emacs`:**
   ```elisp
   ;; Add lisp directory to load path
   (add-to-list 'load-path "~/.emacs.d/lisp")

   ;; Load LC-3 mode
   (require 'lc3-mode)

   ;; Associate .asm files with LC-3 mode
   (add-to-list 'auto-mode-alist '("\\.asm\\'" . lc3-mode))
   (add-to-list 'auto-mode-alist '("\\.lc3\\'" . lc3-mode))
   ```

4. **Restart Emacs or evaluate the configuration:**
   - `M-x eval-buffer` in your init file

#### Features in Emacs:
- ✅ Full syntax highlighting (font-lock mode)
- ✅ Automatic indentation
- ✅ Comment handling (`M-;`)
- ✅ Case-insensitive instruction matching
- ✅ Customizable indentation settings
- ✅ Labels at column 0, instructions indented

**Customization:**
```elisp
;; Customize indentation
(setq lc3-basic-offset 4)  ; Default: 4 spaces
(setq lc3-tab-width 8)     ; Default: 8
```

---

## Language Reference

### Syntax Elements

| Element | Syntax | Examples | Scope Name |
|---------|--------|----------|------------|
| **Comments** | `;comment` | `; This is a comment` | `comment.line` |
| **Directives** | `.DIRECTIVE` | `.ORIG x3000`, `.FILL #10`, `.STRINGZ "Hi"` | `keyword.control.directive` |
| **Opcodes** | `INSTRUCTION` | `ADD`, `AND`, `NOT`, `LD`, `ST` | `keyword.operator.instruction` |
| **Branch** | `BR[nzp]*` | `BR`, `BRnz`, `BRnzp` | `keyword.control.branch` |
| **Pseudo-ops** | `PSEUDO` | `HALT`, `PUTS`, `RET` | `keyword.control.pseudo` |
| **Registers** | `R[0-7]` | `R0`, `R1`, `R7` | `variable.language.register` |
| **Decimal** | `#[-]?[0-9]+` | `#10`, `#-5`, `#255` | `constant.numeric.decimal` |
| **Hexadecimal** | `x[0-9A-Fa-f]+` | `x3000`, `xFFFF`, `xAbCd` | `constant.numeric.hex` |
| **Binary** | `b[01]+` | `b1010`, `b1111` | `constant.numeric.binary` |
| **Strings** | `"..."` | `"Hello\n"`, `"Text\t"` | `string.quoted.double` |
| **Escapes** | `\n \r \t \\ \" \0` | Inside strings | `constant.character.escape` |
| **Labels** | `[A-Z_][A-Z0-9_]*` | `LOOP`, `START`, `MY_VAR` | `entity.name.label` |
| **Comma** | `,` | `ADD R1, R2, #5` | `punctuation.separator` |

### Example Code with Highlighting

```asm
; Hello World Program
.ORIG x3000

    LEA R0, HELLO       ; Load address of string
    PUTS                ; Print string
    HALT                ; Stop execution

HELLO .STRINGZ "Hello, World!\n"

.END
```

```asm
; Subroutine Example with Stack
.ORIG x3000

    LD R6, STACK_PTR    ; Initialize stack pointer
    JSR FIBONACCI       ; Call subroutine
    HALT

; Calculate Fibonacci number
; Input: R0 = n
; Output: R1 = fib(n)
FIBONACCI
    ; Save registers
    ADD R6, R6, #-1
    STR R7, R6, #0

    ; Base case
    ADD R0, R0, #0
    BRz FIB_ZERO
    ADD R0, R0, #-1
    BRz FIB_ONE

    ; Recursive case
    ; ... implementation ...

FIB_ZERO
    AND R1, R1, #0      ; Return 0
    BR FIB_END

FIB_ONE
    AND R1, R1, #0
    ADD R1, R1, #1      ; Return 1

FIB_END
    ; Restore registers
    LDR R7, R6, #0
    ADD R6, R6, #1
    RET

STACK_PTR .FILL xFE00

.END
```

---

## Color Customization

### VS Code Themes

The syntax highlighting works with all VS Code themes. Colors are mapped to semantic tokens:

- **Directives** → Preprocessor (purple/pink)
- **Instructions** → Keywords (blue/cyan)
- **Branches/Jumps** → Control flow (purple)
- **Pseudo-ops** → Functions (yellow)
- **Registers** → Types (green/teal)
- **Numbers** → Constants (orange/brown)
- **Strings** → Strings (orange/red)
- **Labels** → Variables/identifiers (light blue)
- **Comments** → Comments (gray/green)

### Custom VS Code Colors

Add to your `settings.json` for custom colors:

```json
{
  "editor.tokenColorCustomizations": {
    "textMateRules": [
      {
        "scope": "keyword.control.directive.lc3asm",
        "settings": {
          "foreground": "#C586C0",
          "fontStyle": "bold"
        }
      },
      {
        "scope": "variable.language.register.lc3asm",
        "settings": {
          "foreground": "#4EC9B0",
          "fontStyle": "bold"
        }
      }
    ]
  }
}
```

---

## Testing

### Test File

Create a test file `test.asm`:

```asm
; LC-3 Syntax Test File
; Tests all language features

.ORIG x3000

; === Labels ===
START
LOOP
MY_CONSTANT

; === Instructions ===
    ADD R1, R2, R3      ; Register mode
    ADD R1, R2, #5      ; Immediate mode
    AND R0, R0, #0      ; Clear register
    NOT R1, R2          ; Bitwise NOT

; === Branches ===
    BR START            ; Unconditional
    BRn NEGATIVE        ; Negative only
    BRz ZERO            ; Zero only
    BRp POSITIVE        ; Positive only
    BRnz NEG_ZERO       ; Negative or zero
    BRnp NOT_ZERO       ; Not zero
    BRzp NON_NEG        ; Non-negative
    BRnzp ALWAYS        ; Always (same as BR)

; === Jumps ===
    JMP R3              ; Jump to address in R3
    JSR SUBROUTINE      ; Jump to subroutine
    JSRR R4             ; JSR via register
    RET                 ; Return

; === Memory Operations ===
    LD R0, CONSTANT     ; Load
    LDI R1, POINTER     ; Load indirect
    LDR R2, R3, #5      ; Load base+offset
    LEA R0, STRING      ; Load effective address
    ST R0, RESULT       ; Store
    STI R1, OUTPUT      ; Store indirect
    STR R2, R3, #-5     ; Store base+offset

; === System Operations ===
    TRAP x25            ; Generic trap
    GETC                ; Get character
    OUT                 ; Output character
    PUTS                ; Output string
    IN                  ; Input with echo
    PUTSP               ; Output packed string
    HALT                ; Stop execution
    RTI                 ; Return from interrupt

; === Numbers ===
    .FILL #10           ; Decimal positive
    .FILL #-5           ; Decimal negative
    .FILL x3000         ; Hexadecimal
    .FILL xFFFF         ; Hex max
    .FILL b1010         ; Binary
    .FILL b1111111111111111  ; Binary 16-bit

; === Strings ===
STRING .STRINGZ "Hello, World!"
ESCAPE .STRINGZ "Line1\nLine2\tTabbed"
QUOTES .STRINGZ "He said \"Hi\""
PATH   .STRINGZ "C:\\path\\to\\file"
NULL   .STRINGZ "Null\0Here"

; === Directives ===
CONSTANT .FILL #100
BUFFER   .BLKW #50
MESSAGE  .STRINGZ "Test message"

NEGATIVE
ZERO
POSITIVE
NEG_ZERO
NOT_ZERO
NON_NEG
ALWAYS
SUBROUTINE
POINTER
RESULT
OUTPUT

.END
```

Open this file in your editor to verify all syntax elements are highlighted correctly.

---

## Troubleshooting

### VS Code

**Problem:** Syntax highlighting not working
- **Solution:** Check that the file extension is `.asm` or `.lc3`
- **Solution:** Press `Ctrl+K M` and manually select "LC-3 Assembly"
- **Solution:** Reload the window (`Ctrl+Shift+P` → "Reload Window")

**Problem:** Snippets not appearing
- **Solution:** Make sure you're typing the prefix and pressing `Tab` or `Ctrl+Space`
- **Solution:** Check that IntelliSense is enabled in settings

### Vim

**Problem:** Syntax not detected
- **Solution:** Manually set filetype: `:set filetype=lc3asm`
- **Solution:** Check that ftdetect file exists: `~/.vim/ftdetect/lc3asm.vim`
- **Solution:** Reload Vim: `:source ~/.vimrc`

**Problem:** Colors look wrong
- **Solution:** Check your colorscheme supports syntax highlighting
- **Solution:** Ensure `syntax on` is in your `.vimrc`

### Sublime Text

**Problem:** Syntax not auto-detected
- **Solution:** Manually select: View → Syntax → LC-3 Assembly
- **Solution:** Make sure file is saved with `.asm` extension

### Emacs

**Problem:** Mode not loading
- **Solution:** Check load-path: `M-x describe-variable RET load-path`
- **Solution:** Verify file exists: `ls ~/.emacs.d/lisp/lc3-mode.el`
- **Solution:** Manually load: `M-x load-file RET ~/.emacs.d/lisp/lc3-mode.el`

**Problem:** Syntax highlighting not working
- **Solution:** Enable font-lock: `M-x font-lock-mode`
- **Solution:** Check mode is active: `M-x describe-mode`

---

## Contributing

Contributions are welcome! Areas for improvement:

1. **Additional Editor Support**
   - Atom
   - IntelliJ IDEA / CLion
   - Kate / KWrite
   - Nano
   - Notepad++

2. **Enhanced Features**
   - Language server protocol (LSP) for advanced IntelliSense
   - Semantic highlighting
   - Error checking as-you-type
   - Symbol navigation (go-to-definition)
   - Hover documentation

3. **Theme Compatibility**
   - Test with popular themes
   - Create LC-3-specific color schemes

---

## License

MIT License - See project root for details

---

## Credits

Created for the LC-3 Assembler project. Based on the LC-3 ISA specification and informed by the assembler's lexer implementation.

**References:**
- [LC-3 ISA Specification](https://www.jmeiners.com/lc3-vm/)
- [TextMate Language Grammars](https://macromates.com/manual/en/language_grammars)
- [VS Code Language Extensions](https://code.visualstudio.com/api/language-extensions/overview)
- [Vim Syntax Highlighting](https://vimdoc.sourceforge.net/htmldoc/syntax.html)
