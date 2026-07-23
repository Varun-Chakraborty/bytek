MULT:
    PUSH R2
    PUSH R3
    PUSH R4
    MOVER R0, #0
    MOVER R1, #0
    MOVER R4, R3
    MOVER R3, R2
    MOVER R2, #0
LOOP6:
    CMP R4, #0
    JZ EXIT_MULT
    PUSH R4
    AND R4, #1
    JZ ZERO
ADD:
    ADD R1, R3
    ADC R0, R2
ZERO:
    MOVER R4, #0
    SHL R3
    ADC R4, #0
    SHL R2
    ADD R2, R4
    POP R4
    SHR R4
    JMP LOOP6
EXIT_MULT:
    POP R4
    POP R3
    POP R2
    RET

DIV:
    PUSH R2
    PUSH R3
    PUSH R4
    MOVER R0, #0
    MOVER R1, R2
LOOP7:
    MOVER R4, #0
    CMP R1, R3
    ADC R4, #0
    CMP R4, #1
    JZ EXIT_DIV
    SUB R1, R3
    ADD R0, #1
    JMP LOOP7
EXIT_DIV:
    POP R4
    POP R3
    POP R2
    RET
