JMP START

.include "stdlib.asm"

START:
    MOVER R0, #1
    MOVER R2, #5
    CALL FACT
    ADD R1, R0, #0
    CALL PRINT_INT
    CALL PRINTLN
    HALT

FACT:
    MULT R0, R2
    SUB R2, #1
    JNZ FACT
RET: RET
