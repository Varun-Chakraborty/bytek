MOVEMI 1, 0         ; Constants; declare a constant of value 1 at memory location 1, so zero flag is false
JZ HALT             ; Jump to HALT
JNZ HALT            ; Jump to HALT
HALT: HALT          ; END of program
