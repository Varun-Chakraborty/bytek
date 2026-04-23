IN R2               ; Input the number
MOVER R0, #1        ; Move 1 to R0
LOOP: MULT_16 R2    ; Support of labels; Multiply value at R0 (default 1 for the first iteration) with input
SUB R2, #1         ; Subtract 1 from input
JNZ LOOP            ; Jump to loop if input is not 0
OUT_16              ; Output the result
HALT                ; END of program
