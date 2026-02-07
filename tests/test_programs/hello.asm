; Hello world sample
.ORIG x3000
LEA R0, MSG
PUTS
HALT
MSG .STRINGZ "Hello"
.END
