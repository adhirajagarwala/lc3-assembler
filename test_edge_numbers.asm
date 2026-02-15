; Edge Case Test #2: Numeric Literal Boundaries
; Tests: Min/max values for different bit widths

.ORIG x3000

    ; === 5-bit immediate (imm5): -16 to +15 ===
    ADD R0, R0, #15      ; Max positive (01111)
    ADD R0, R0, #-16     ; Min negative (10000)
    ; ADD R0, R0, #16    ; Would be ERROR: out of range

    ; === 6-bit offset: -32 to +31 ===
    LDR R1, R2, #31      ; Max positive
    LDR R1, R2, #-32     ; Min negative
    ; LDR R1, R2, #32    ; Would be ERROR: out of range

    ; === 16-bit values (.FILL) ===
    ; Testing two's complement edge cases

POSITIVE_MAX .FILL x7FFF    ; +32767 (max positive)
NEGATIVE_MIN .FILL x8000    ; -32768 (min negative)
ZERO         .FILL x0000    ; 0
ALL_ONES     .FILL xFFFF    ; -1 in two's complement

    ; === Hex literal boundaries ===
    LD R0, HEX_MAX
    LD R1, HEX_HIGH      ; High bit set = negative

HEX_MAX  .FILL xFFFF        ; Should be -1
HEX_HIGH .FILL x8001        ; Should be -32767

    ; === Binary literals ===
BIN_VAL  .FILL b1111111111111111    ; 16 ones = -1
BIN_HIGH .FILL b1000000000000000    ; High bit = -32768

    ; === Decimal edge cases ===
DEC_POS  .FILL #32767
DEC_NEG  .FILL #-32768
DEC_ZERO .FILL #0

    HALT

.END
