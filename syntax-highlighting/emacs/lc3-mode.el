;;; lc3-mode.el --- Major mode for LC-3 assembly language -*- lexical-binding: t; -*-

;; Copyright (C) 2026 LC-3 Assembler Project

;; Author: LC-3 Assembler Project
;; Version: 1.0
;; Package-Requires: ((emacs "24.3"))
;; Keywords: languages, assembly, lc3
;; URL: https://github.com/your-repo/lc3-assembler

;;; Commentary:

;; Major mode for editing LC-3 (Little Computer 3) assembly language files.
;; Provides syntax highlighting, indentation, and commenting support.

;;; Installation:

;; 1. Copy this file to your Emacs load-path:
;;    - Unix/Linux: ~/.emacs.d/lisp/lc3-mode.el
;;    - Windows: ~/AppData/Roaming/.emacs.d/lisp/lc3-mode.el
;;
;; 2. Add to your init.el or .emacs:
;;    (add-to-list 'load-path "~/.emacs.d/lisp")
;;    (require 'lc3-mode)
;;    (add-to-list 'auto-mode-alist '("\\.asm\\'" . lc3-mode))

;;; Code:

(defgroup lc3 nil
  "Major mode for editing LC-3 assembly language."
  :prefix "lc3-"
  :group 'languages)

(defcustom lc3-tab-width 8
  "Tab width for LC-3 assembly code."
  :type 'integer
  :group 'lc3)

(defcustom lc3-basic-offset 4
  "Basic indentation offset for LC-3 assembly code."
  :type 'integer
  :group 'lc3)

;;; Syntax Highlighting

(defconst lc3-font-lock-keywords
  (list
   ;; Comments
   '(";.*$" . font-lock-comment-face)

   ;; Directives
   '("\\.\\(ORIG\\|END\\|FILL\\|BLKW\\|STRINGZ\\)\\>" . font-lock-preprocessor-face)

   ;; Operate instructions
   '("\\<\\(ADD\\|AND\\|NOT\\)\\>" . font-lock-keyword-face)

   ;; Branch instructions (all variants)
   '("\\<\\(BR\\|BRn\\|BRz\\|BRp\\|BRnz\\|BRnp\\|BRzp\\|BRnzp\\)\\>" . font-lock-keyword-face)

   ;; Jump and control flow
   '("\\<\\(JMP\\|JSR\\|JSRR\\)\\>" . font-lock-keyword-face)

   ;; Load instructions
   '("\\<\\(LD\\|LDI\\|LDR\\|LEA\\)\\>" . font-lock-keyword-face)

   ;; Store instructions
   '("\\<\\(ST\\|STI\\|STR\\)\\>" . font-lock-keyword-face)

   ;; System instructions
   '("\\<\\(TRAP\\|RTI\\)\\>" . font-lock-builtin-face)

   ;; Pseudo-operations
   '("\\<\\(RET\\|GETC\\|OUT\\|PUTS\\|IN\\|PUTSP\\|HALT\\)\\>" . font-lock-builtin-face)

   ;; Registers (R0-R7)
   '("\\<R[0-7]\\>" . font-lock-type-face)

   ;; Decimal numbers (#123 or #-45)
   '("#-?[0-9]+" . font-lock-constant-face)

   ;; Hexadecimal numbers (x3000)
   '("\\<x[0-9A-Fa-f]+\\>" . font-lock-constant-face)

   ;; Binary numbers (b1010)
   '("\\<b[01]+\\>" . font-lock-constant-face)

   ;; String literals
   '("\"\\([^\"\\]\\|\\\\.\\)*\"" . font-lock-string-face)

   ;; Label definitions (at start of line)
   '("^[ \t]*\\([A-Z_][A-Z0-9_]*\\)\\>" 1 font-lock-function-name-face)

   ;; Label references
   '("\\<\\([A-Z_][A-Z0-9_]*\\)\\>" . font-lock-variable-name-face))
  "Syntax highlighting rules for LC-3 assembly language.")

;;; Indentation

(defun lc3-indent-line ()
  "Indent current line as LC-3 assembly code."
  (interactive)
  (let ((indent-column 0))
    (save-excursion
      (beginning-of-line)
      (cond
       ;; Labels at column 0
       ((looking-at "^[A-Z_][A-Z0-9_]*")
        (setq indent-column 0))
       ;; Directives indented
       ((looking-at "^[ \t]*\\.")
        (setq indent-column lc3-basic-offset))
       ;; Instructions indented
       ((looking-at "^[ \t]*\\(ADD\\|AND\\|NOT\\|BR\\|JMP\\|JSR\\|LD\\|ST\\|TRAP\\|RET\\|HALT\\)")
        (setq indent-column lc3-basic-offset))
       ;; Comments at current position
       ((looking-at "^[ \t]*;")
        (setq indent-column (current-indentation)))
       ;; Default: indent
       (t
        (setq indent-column lc3-basic-offset))))
    (indent-line-to indent-column)))

;;; Syntax Table

(defvar lc3-mode-syntax-table
  (let ((table (make-syntax-table)))
    ;; Comments start with semicolon
    (modify-syntax-entry ?\; "<" table)
    (modify-syntax-entry ?\n ">" table)
    ;; Strings
    (modify-syntax-entry ?\" "\"" table)
    ;; Labels can have underscores
    (modify-syntax-entry ?_ "w" table)
    table)
  "Syntax table for `lc3-mode'.")

;;; Keymap

(defvar lc3-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "RET") 'newline-and-indent)
    (define-key map (kbd "TAB") 'lc3-indent-line)
    map)
  "Keymap for `lc3-mode'.")

;;; Mode Definition

;;;###autoload
(define-derived-mode lc3-mode prog-mode "LC-3"
  "Major mode for editing LC-3 assembly language.

Key bindings:
\\{lc3-mode-map}"
  :syntax-table lc3-mode-syntax-table

  ;; Comments
  (setq-local comment-start ";")
  (setq-local comment-end "")
  (setq-local comment-start-skip ";+\\s-*")

  ;; Font lock
  (setq-local font-lock-defaults '(lc3-font-lock-keywords nil t))

  ;; Indentation
  (setq-local indent-line-function 'lc3-indent-line)
  (setq-local tab-width lc3-tab-width)

  ;; Case-insensitive for instructions
  (setq-local font-lock-keywords-case-fold-search t))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.asm\\'" . lc3-mode))
(add-to-list 'auto-mode-alist '("\\.lc3\\'" . lc3-mode))

(provide 'lc3-mode)

;;; lc3-mode.el ends here
