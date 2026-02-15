; Edge Case Test #3: Program Structure Edge Cases
; Tests: Weird but valid structures, empty lines, edge cases

.ORIG x3000

; === Test: Label on same line as instruction ===
LABEL1  ADD R1, R1, #1

; === Test: Label on separate line ===
LABEL2
    ADD R2, R2, #2

; === Test: Multiple empty lines and comments ===


    ; Comment after empty lines

; === Test: All branch variants ===
    BR NEXT       ; BRnzp (unconditional)
    BRn NEG       ; Negative only
    BRz ZERO      ; Zero only
    BRp POS       ; Positive only
    BRnz NEG_ZERO ; Negative or zero
    BRnp NEG_POS  ; Negative or positive
    BRzp POS_ZERO ; Zero or positive
    BRnzp NEXT    ; All flags (same as BR)

NEXT
NEG
ZERO
POS
NEG_ZERO
NEG_POS
POS_ZERO
    ; All labels point here
    NOP           ; Well, ADD R0,R0,#0 acts as NOP

; === Test: String with all escape sequences ===
STRING1 .STRINGZ "Hello\nWorld\r\n"
STRING2 .STRINGZ "Tab:\tQuote:\"Backslash:\\"
STRING3 .STRINGZ "Null:\0After"

; === Test: .BLKW edge cases ===
BLOCK1  .BLKW #1      ; Minimum (1 word)
BLOCK2  .BLKW #100    ; Medium block

; === Test: .FILL with labels ===
PTR1    .FILL LABEL1  ; Fill with label address
PTR2    .FILL STRING1 ; Fill with string address

; === Test: Instructions using all registers ===
    ADD R0, R1, R2
    ADD R3, R4, R5
    ADD R6, R7, R0

; === Test: Maximum negative immediate ===
    ADD R0, R0, #-16  ; Minimum imm5
    LDR R1, R2, #-32  ; Minimum offset6

; === Test: Address overflow boundary ===
; If we're at x3000 and add close to xD000 words, we'd overflow xFFFF
; This should NOT overflow (we're well under)
    .BLKW #1000

FINAL   HALT

.END
