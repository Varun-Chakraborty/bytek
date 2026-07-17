JMP START

.align

PROMPT:
.ascii "Enter a number: \0"
.include "stdlib.asm"

START:
    MOVER R1, #PROMPT
    CALL PRINT_STRING
    IN R2
    SUB R2, #48         ; convert ASCII to decimal
    MOVER R0, #1
    CMP R2, #0
    JZ DONE
LOOP:
    MULT R0, R2
    SUB R2, #1
    JNZ LOOP
DONE:
    ADD R1, R0, #0
    CALL PRINT_INT
    CALL PRINTLN
    HALT
