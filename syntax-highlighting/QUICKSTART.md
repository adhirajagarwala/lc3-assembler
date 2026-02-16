# Quick Start Guide

Get LC-3 syntax highlighting in under 60 seconds!

## One-Command Installation

### Unix / Linux / macOS
```bash
cd syntax-highlighting
./install.sh all
```

### Windows
```cmd
cd syntax-highlighting
install.bat all
```

## Per-Editor Installation

### VS Code
```bash
./install.sh vscode   # Unix/Linux/macOS
install.bat vscode    # Windows
```
Then: `Ctrl+Shift+P` → "Reload Window"

### Vim
```bash
./install.sh vim      # Unix/Linux/macOS
install.bat vim       # Windows
```

### Sublime Text
```bash
./install.sh sublime  # Unix/Linux/macOS
install.bat sublime   # Windows
```

### Emacs
```bash
./install.sh emacs    # Unix/Linux/macOS
```
Then add to your `init.el`:
```elisp
(add-to-list 'load-path "~/.emacs.d/lisp")
(require 'lc3-mode)
```

## Verify Installation

1. Open `test.asm` in your editor
2. Verify you see colored syntax
3. Start coding!

## VS Code Snippets

Type these prefixes and press `Tab`:

- `lc3prog` → Basic program template
- `lc3sub` → Subroutine with stack
- `push` → Push register to stack
- `pop` → Pop register from stack
- `lc3loop` → Loop structure
- `lc3if` → If-else branch
- `lc3string` → String output
- `trap` → TRAP system call

## Need Help?

See [README.md](README.md) for full documentation.

## What You Get

✨ **All LC-3 features highlighted:**
- Instructions (ADD, LD, BR, etc.)
- Registers (R0-R7)
- Numbers (#10, x3000, b1010)
- Strings with escapes
- Labels
- Comments
- Directives (.ORIG, .FILL, etc.)

🎨 **Works with your favorite theme**

📝 **10 productivity snippets** (VS Code)

---

**Installation time:** < 60 seconds
**Effort:** One command
**Editors supported:** 4
**Maintenance:** Zero

Happy coding! 🚀
