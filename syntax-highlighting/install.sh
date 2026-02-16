#!/bin/bash
# ==============================================================================
# LC-3 Assembly Syntax Highlighting Installer
# ==============================================================================
# Installs syntax highlighting for various editors
# Usage: ./install.sh [editor]
#   editor: vscode, vim, sublime, emacs, all (default: all)
# ==============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Print colored message
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ==============================================================================
# VS Code Installation
# ==============================================================================
install_vscode() {
    print_info "Installing VS Code syntax highlighting..."

    # Determine VS Code extensions directory based on OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        VSCODE_EXT_DIR="$HOME/.vscode/extensions"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        VSCODE_EXT_DIR="$HOME/.vscode/extensions"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        VSCODE_EXT_DIR="$HOME/.vscode/extensions"
    else
        print_error "Unsupported OS for VS Code installation"
        return 1
    fi

    EXT_DIR="$VSCODE_EXT_DIR/lc3-assembly-1.0.0"

    # Create extension directory
    mkdir -p "$EXT_DIR"

    # Copy extension files
    cp "$SCRIPT_DIR/vscode/lc3asm.tmLanguage.json" "$EXT_DIR/"
    cp "$SCRIPT_DIR/vscode/language-configuration.json" "$EXT_DIR/"
    cp "$SCRIPT_DIR/vscode/package.json" "$EXT_DIR/"
    cp "$SCRIPT_DIR/vscode/snippets.json" "$EXT_DIR/"

    print_success "VS Code extension installed to: $EXT_DIR"
    print_info "Please reload VS Code (Ctrl+Shift+P -> 'Reload Window')"
}

# ==============================================================================
# Vim Installation
# ==============================================================================
install_vim() {
    print_info "Installing Vim syntax highlighting..."

    # Create directories
    mkdir -p "$HOME/.vim/syntax"
    mkdir -p "$HOME/.vim/ftdetect"

    # Copy syntax file
    cp "$SCRIPT_DIR/vim/lc3asm.vim" "$HOME/.vim/syntax/"

    # Create or update ftdetect file
    FTDETECT_FILE="$HOME/.vim/ftdetect/lc3asm.vim"
    echo "au BufRead,BufNewFile *.asm set filetype=lc3asm" > "$FTDETECT_FILE"

    print_success "Vim syntax file installed"
    print_info "Filetype detection: $FTDETECT_FILE"
}

# ==============================================================================
# Sublime Text Installation
# ==============================================================================
install_sublime() {
    print_info "Installing Sublime Text syntax highlighting..."

    # Determine Sublime Text packages directory based on OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        SUBLIME_DIR="$HOME/Library/Application Support/Sublime Text/Packages/User"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Try Sublime Text 3 first, then Sublime Text 4
        if [ -d "$HOME/.config/sublime-text-3" ]; then
            SUBLIME_DIR="$HOME/.config/sublime-text-3/Packages/User"
        elif [ -d "$HOME/.config/sublime-text" ]; then
            SUBLIME_DIR="$HOME/.config/sublime-text/Packages/User"
        else
            print_error "Sublime Text directory not found"
            return 1
        fi
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        SUBLIME_DIR="$APPDATA/Sublime Text/Packages/User"
    else
        print_error "Unsupported OS for Sublime Text installation"
        return 1
    fi

    if [ ! -d "$SUBLIME_DIR" ]; then
        print_warning "Sublime Text directory not found: $SUBLIME_DIR"
        print_info "Please install Sublime Text first or create the directory manually"
        return 1
    fi

    # Copy syntax file
    cp "$SCRIPT_DIR/sublime/LC3.sublime-syntax" "$SUBLIME_DIR/"

    print_success "Sublime Text syntax file installed to: $SUBLIME_DIR"
}

# ==============================================================================
# Emacs Installation
# ==============================================================================
install_emacs() {
    print_info "Installing Emacs mode..."

    # Create lisp directory
    mkdir -p "$HOME/.emacs.d/lisp"

    # Copy mode file
    cp "$SCRIPT_DIR/emacs/lc3-mode.el" "$HOME/.emacs.d/lisp/"

    print_success "Emacs mode installed to: $HOME/.emacs.d/lisp/lc3-mode.el"
    print_info ""
    print_info "Add the following to your init.el or .emacs:"
    print_info ""
    echo -e "${YELLOW}"
    echo "(add-to-list 'load-path \"~/.emacs.d/lisp\")"
    echo "(require 'lc3-mode)"
    echo "(add-to-list 'auto-mode-alist '(\"\\.asm\\\\'\" . lc3-mode))"
    echo -e "${NC}"
}

# ==============================================================================
# Main Installation Logic
# ==============================================================================

print_info "========================================"
print_info "LC-3 Syntax Highlighting Installer"
print_info "========================================"
print_info ""

# Determine what to install
INSTALL_TARGET="${1:-all}"

case "$INSTALL_TARGET" in
    vscode)
        install_vscode
        ;;
    vim)
        install_vim
        ;;
    sublime)
        install_sublime
        ;;
    emacs)
        install_emacs
        ;;
    all)
        print_info "Installing for all supported editors..."
        print_info ""

        install_vscode
        echo ""

        install_vim
        echo ""

        install_sublime
        echo ""

        install_emacs
        echo ""

        print_success "Installation complete for all editors!"
        ;;
    *)
        print_error "Unknown editor: $INSTALL_TARGET"
        print_info "Usage: $0 [vscode|vim|sublime|emacs|all]"
        exit 1
        ;;
esac

print_info ""
print_success "Installation complete!"
print_info "Open a .asm file to see syntax highlighting in action."
print_info "Test file available at: $SCRIPT_DIR/test.asm"
