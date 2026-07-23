JMP MAIN

.include "stdlib.asm"

MAIN:
    MOVER R1, #2
    MOVER R2, #3
    ADD R0, R1, R2
    ADD R0, R0
    MOVER R1, R0
    CALL PRINT_INT
    CALL PRINTLN
    HALT
