; Countdown program
.ORIG x3000
AND R1, R1, #0
ADD R1, R1, #5
LOOP ADD R1, R1, #-1
BRp LOOP
HALT
.END
