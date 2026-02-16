" Vim syntax file
" Language:     LC-3 Assembly
" Maintainer:   LC-3 Assembler Project
" Last Change:  2026-02-15
" Version:      1.0
" Description:  Syntax highlighting for LC-3 (Little Computer 3) assembly language
"
" Installation:
"   1. Copy this file to ~/.vim/syntax/lc3asm.vim
"   2. Add to ~/.vim/ftdetect/lc3asm.vim:
"      au BufRead,BufNewFile *.asm set filetype=lc3asm
"
" Or add to your vimrc:
"   au BufRead,BufNewFile *.asm set filetype=lc3asm

if exists("b:current_syntax")
  finish
endif

" Case-insensitive for instructions but case-sensitive for labels
syn case ignore

" ============================================================================
" Comments
" ============================================================================
syn match lc3Comment ";.*$" contains=lc3Todo
syn keyword lc3Todo TODO FIXME XXX NOTE contained

" ============================================================================
" Directives (case-insensitive)
" ============================================================================
syn match lc3Directive "\.\(ORIG\|END\|FILL\|BLKW\|STRINGZ\)\>"

" ============================================================================
" Opcodes - Operate Instructions
" ============================================================================
syn keyword lc3Opcode ADD AND NOT

" ============================================================================
" Opcodes - Branch Instructions (all variants)
" ============================================================================
syn keyword lc3Branch BR BRn BRz BRp BRnz BRnp BRzp BRnzp

" ============================================================================
" Opcodes - Control Flow
" ============================================================================
syn keyword lc3Jump JMP JSR JSRR
syn keyword lc3System TRAP RTI

" ============================================================================
" Opcodes - Data Movement (Load/Store)
" ============================================================================
syn keyword lc3Load LD LDI LDR LEA
syn keyword lc3Store ST STI STR

" ============================================================================
" Pseudo-operations (TRAP shortcuts)
" ============================================================================
syn keyword lc3Pseudo RET GETC OUT PUTS IN PUTSP HALT

" ============================================================================
" Registers (R0-R7)
" ============================================================================
syn match lc3Register "\<R[0-7]\>"

" ============================================================================
" Numeric Literals
" ============================================================================
" Decimal with # prefix
syn match lc3Number "#-\?\d\+"

" Hexadecimal with x prefix
syn match lc3Number "\<x[0-9A-Fa-f]\+\>"

" Binary with b prefix
syn match lc3Number "\<b[01]\+\>"

" ============================================================================
" String Literals
" ============================================================================
syn region lc3String start=/"/ skip=/\\"/ end=/"/ contains=lc3Escape,lc3EscapeError
syn match lc3Escape "\\[nrt\\\"]" contained
syn match lc3Escape "\\0" contained
syn match lc3EscapeError "\\." contained

" ============================================================================
" Labels (case-sensitive, must start with letter or underscore)
" ============================================================================
syn case match
" Label definition at start of line
syn match lc3Label "^\s*[A-Z_][A-Z0-9_]*\>"

" Label reference (used as operand)
syn match lc3LabelRef "\<[A-Z_][A-Z0-9_]*\>" contained

" Reset to case-insensitive
syn case ignore

" ============================================================================
" Punctuation
" ============================================================================
syn match lc3Comma ","

" ============================================================================
" Highlighting Groups
" ============================================================================

" Comments
hi def link lc3Comment      Comment
hi def link lc3Todo         Todo

" Directives
hi def link lc3Directive    PreProc

" Instructions
hi def link lc3Opcode       Keyword
hi def link lc3Branch       Conditional
hi def link lc3Jump         Conditional
hi def link lc3System       Special
hi def link lc3Load         Keyword
hi def link lc3Store        Keyword
hi def link lc3Pseudo       Function

" Operands
hi def link lc3Register     Type
hi def link lc3Number       Number
hi def link lc3String       String
hi def link lc3Escape       SpecialChar
hi def link lc3EscapeError  Error

" Labels
hi def link lc3Label        Identifier
hi def link lc3LabelRef     Identifier

" Punctuation
hi def link lc3Comma        Delimiter

" ============================================================================
" Syntax Name
" ============================================================================
let b:current_syntax = "lc3asm"

" vim: ts=2 sw=2 et
