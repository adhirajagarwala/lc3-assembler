; Subroutine example
.ORIG x3000
JSR SUB
HALT
SUB ADD R1, R1, #1
RET
.END
