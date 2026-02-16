; ============================================================================
; LC-3 Syntax Highlighting Test File
; ============================================================================
; This file tests all language features for syntax highlighting verification
; Author: LC-3 Assembler Project
; Date: 2026-02-15
; ============================================================================

.ORIG x3000

; ============================================================================
; SECTION 1: Labels and Comments
; ============================================================================

START           ; Label at start of line
MAIN_LOOP       ; Another label
MY_CONSTANT     ; TODO: Initialize this constant
BUFFER_START    ; FIXME: Check buffer size

; ============================================================================
; SECTION 2: Arithmetic and Logic Operations
; ============================================================================

    ; ADD instruction variants
    ADD R1, R2, R3      ; Register mode: R1 = R2 + R3
    ADD R1, R2, #5      ; Immediate mode: R1 = R2 + 5
    ADD R0, R0, #-1     ; Negative immediate: R0 = R0 - 1

    ; AND instruction variants
    AND R0, R0, #0      ; Clear register: R0 = 0
    AND R1, R2, R3      ; Register mode: R1 = R2 & R3
    AND R7, R7, #15     ; Mask lower bits

    ; NOT instruction
    NOT R1, R2          ; Bitwise NOT: R1 = ~R2
    NOT R0, R0          ; Complement R0

; ============================================================================
; SECTION 3: Branch Instructions (All Variants)
; ============================================================================

    ; Unconditional branches
    BR ALWAYS           ; Branch always (same as BRnzp)
    BRnzp ALWAYS        ; Branch always (explicit)

    ; Single condition branches
    BRn NEGATIVE        ; Branch if negative (N=1)
    BRz ZERO            ; Branch if zero (Z=1)
    BRp POSITIVE        ; Branch if positive (P=1)

    ; Two condition branches
    BRnz NEG_OR_ZERO    ; Branch if N=1 or Z=1
    BRnp NOT_ZERO       ; Branch if N=1 or P=1 (not zero)
    BRzp NON_NEGATIVE   ; Branch if Z=1 or P=1 (non-negative)

; ============================================================================
; SECTION 4: Control Flow (Jumps and Subroutines)
; ============================================================================

    ; Jump instructions
    JMP R3              ; Jump to address in R3
    JSRR R4             ; Jump to subroutine via register R4

    ; Subroutine call and return
    JSR FIBONACCI       ; Call subroutine at label
    JSR PRINT_STRING    ; Another subroutine call
    RET                 ; Return from subroutine

; ============================================================================
; SECTION 5: Memory Operations (Load/Store)
; ============================================================================

    ; Load instructions
    LD R0, DATA_VALUE   ; Load from PC-relative address
    LDI R1, POINTER     ; Load indirect (pointer dereference)
    LDR R2, R3, #5      ; Load base+offset: R2 = MEM[R3 + 5]
    LEA R0, STRING_MSG  ; Load effective address

    ; Store instructions
    ST R0, RESULT       ; Store to PC-relative address
    STI R1, OUTPUT_PTR  ; Store indirect
    STR R2, R3, #-5     ; Store base+offset: MEM[R3 - 5] = R2

; ============================================================================
; SECTION 6: TRAP Instructions and Pseudo-Operations
; ============================================================================

    ; Generic TRAP
    TRAP x20            ; Generic trap call
    TRAP x25            ; HALT trap

    ; TRAP pseudo-operations (shortcuts)
    GETC                ; Read character from keyboard (x20)
    OUT                 ; Output character in R0 (x21)
    PUTS                ; Output null-terminated string (x22)
    IN                  ; Input with prompt and echo (x23)
    PUTSP               ; Output packed string (x24)
    HALT                ; Stop execution (x25)

    ; System instruction
    RTI                 ; Return from interrupt

; ============================================================================
; SECTION 7: Numeric Literals (All Formats)
; ============================================================================

    ; Decimal literals
    ADD R0, R0, #10     ; Positive decimal
    ADD R1, R1, #-5     ; Negative decimal
    ADD R2, R2, #255    ; Maximum 8-bit value
    ADD R3, R3, #0      ; Zero

    .FILL #32767        ; Maximum positive 16-bit
    .FILL #-32768       ; Minimum negative 16-bit
    .FILL #100          ; Arbitrary positive
    .FILL #-1           ; Negative one

    ; Hexadecimal literals
    .FILL x0000         ; Zero in hex
    .FILL x3000         ; Common origin address
    .FILL xFFFF         ; All ones (65535 or -1)
    .FILL xABCD         ; Mixed case hex
    .FILL x7FFF         ; Maximum positive (32767)
    .FILL x8000         ; Minimum negative (-32768)
    .FILL xCAFE         ; Arbitrary hex value
    .FILL xBeEf         ; Case insensitive

    ; Binary literals
    .FILL b0000         ; Zero in binary
    .FILL b1111         ; Fifteen
    .FILL b1010         ; Ten (0xA)
    .FILL b0101         ; Five
    .FILL b1111111111111111  ; All ones (16-bit)
    .FILL b1000000000000000  ; Minimum negative
    .FILL b0111111111111111  ; Maximum positive

; ============================================================================
; SECTION 8: String Literals and Escape Sequences
; ============================================================================

HELLO_WORLD .STRINGZ "Hello, World!"
NEWLINE_TEST .STRINGZ "Line 1\nLine 2\nLine 3"
TAB_TEST .STRINGZ "Column1\tColumn2\tColumn3"
MIXED_ESCAPES .STRINGZ "Path: C:\\Users\\Name\n\tIndented"
QUOTE_TEST .STRINGZ "He said \"Hello\" to me"
NULL_TEST .STRINGZ "Text before null\0Text after null"
CARRIAGE_RET .STRINGZ "Old style\r\nLine ending"
BACKSLASH .STRINGZ "Backslash: \\ works"
EMPTY_STRING .STRINGZ ""
LONG_STRING .STRINGZ "This is a longer string to test how the syntax highlighter handles extended text that spans what would be multiple columns in a typical editor window."

; ============================================================================
; SECTION 9: Data Directives
; ============================================================================

; .FILL directive (single word)
CONSTANT .FILL #42
ADDRESS .FILL x4000
FLAGS .FILL b1010

; .BLKW directive (block of words)
SMALL_BUFFER .BLKW #10      ; 10-word buffer
LARGE_BUFFER .BLKW #100     ; 100-word buffer
ARRAY .BLKW #50             ; Array of 50 words

; .STRINGZ directive (null-terminated strings)
PROMPT .STRINGZ "Enter a character: "
ERROR_MSG .STRINGZ "Error: Invalid input!"
SUCCESS_MSG .STRINGZ "Operation successful."

; ============================================================================
; SECTION 10: Complex Example - Fibonacci Subroutine
; ============================================================================

; Calculate Fibonacci number
; Input: R0 = n (which Fibonacci number to calculate)
; Output: R1 = fib(n)
; Uses: R2, R3 (saved on stack)
FIBONACCI
    ; Save return address and registers
    ADD R6, R6, #-1     ; Decrement stack pointer
    STR R7, R6, #0      ; Save R7
    ADD R6, R6, #-1
    STR R2, R6, #0      ; Save R2
    ADD R6, R6, #-1
    STR R3, R6, #0      ; Save R3

    ; Base case: if n == 0, return 0
    ADD R0, R0, #0      ; Test if R0 is zero
    BRz FIB_ZERO

    ; Base case: if n == 1, return 1
    ADD R0, R0, #-1     ; Test if R0 is one
    BRz FIB_ONE

    ; Recursive case: fib(n) = fib(n-1) + fib(n-2)
    ; Calculate fib(n-1)
    ADD R0, R0, #-1     ; n = n - 1
    JSR FIBONACCI       ; Call fib(n-1)
    ADD R2, R1, #0      ; Save result in R2

    ; Calculate fib(n-2)
    ADD R0, R0, #-1     ; n = n - 2
    JSR FIBONACCI       ; Call fib(n-2)
    ADD R3, R1, #0      ; Save result in R3

    ; Sum the results
    ADD R1, R2, R3      ; R1 = fib(n-1) + fib(n-2)
    BR FIB_END

FIB_ZERO
    AND R1, R1, #0      ; Return 0
    BR FIB_END

FIB_ONE
    AND R1, R1, #0
    ADD R1, R1, #1      ; Return 1

FIB_END
    ; Restore registers and return
    LDR R3, R6, #0      ; Restore R3
    ADD R6, R6, #1
    LDR R2, R6, #0      ; Restore R2
    ADD R6, R6, #1
    LDR R7, R6, #0      ; Restore R7
    ADD R6, R6, #1
    RET

; ============================================================================
; SECTION 11: Complex Example - String Print Loop
; ============================================================================

PRINT_STRING
    ; Print null-terminated string pointed to by R0
    ; Preserves: R1, R2, R3, R4, R5, R6, R7
    ; Uses: R0 (character to print)

    ; Save registers
    ADD R6, R6, #-2     ; Make space for 2 registers
    STR R1, R6, #0      ; Save R1
    STR R2, R6, #1      ; Save R2

    ADD R1, R0, #0      ; R1 = string pointer

PRINT_LOOP
    LDR R0, R1, #0      ; Load character
    BRz PRINT_DONE      ; If null, we're done
    OUT                 ; Print character
    ADD R1, R1, #1      ; Advance pointer
    BR PRINT_LOOP       ; Continue loop

PRINT_DONE
    ; Restore registers
    LDR R2, R6, #1      ; Restore R2
    LDR R1, R6, #0      ; Restore R1
    ADD R6, R6, #2      ; Pop stack
    RET

; ============================================================================
; SECTION 12: Label Definitions and Forward References
; ============================================================================

NEGATIVE            ; Label targets from branches
ZERO
POSITIVE
NEG_OR_ZERO
NOT_ZERO
NON_NEGATIVE
ALWAYS
DATA_VALUE
POINTER
OUTPUT_PTR
RESULT
STRING_MSG

; ============================================================================
; SECTION 13: Data Section
; ============================================================================

; Stack initialization
STACK_BASE .FILL xFE00      ; Base of stack
STACK_PTR .FILL xFDFF       ; Initial stack pointer

; Constants
ZERO_CONST .FILL #0
ONE_CONST .FILL #1
NEG_ONE .FILL #-1
MAX_INT .FILL x7FFF
MIN_INT .FILL x8000

; Messages
WELCOME_MSG .STRINGZ "Welcome to LC-3!\n"
GOODBYE_MSG .STRINGZ "Goodbye!\n"

; Buffers
INPUT_BUFFER .BLKW #80      ; 80-character input buffer
TEMP_STORAGE .BLKW #16      ; Temporary storage

.END

; ============================================================================
; End of test file
; ============================================================================
