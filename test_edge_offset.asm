; Edge Case Test #1: PC Offset Limits
; Tests: Maximum and minimum PC-relative offsets

.ORIG x3000

START
    ; This should work: offset = +255 (maximum for 9-bit)
    BR FAR_FORWARD

    ; Fill 254 words to make FAR_FORWARD exactly +255 away
    .BLKW #254

FAR_FORWARD
    ; This should work: going back exactly -256 (minimum for 9-bit)
    LD R0, FAR_BACK
    HALT

FAR_BACK .FILL #42

    ; Now test a jump that's TOO FAR (should fail)
    .BLKW #300    ; Create a big gap

TOO_FAR
    BR START      ; ERROR: This will be > 256 words away!

.END
