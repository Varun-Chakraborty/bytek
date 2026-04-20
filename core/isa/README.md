# isa

`isa` defines the shared instruction set used by the assembler and VM.

It is the source of truth for:

- Machine constants such as register count, memory size, and word size.
- Operand categories and bit widths.
- Opcode lookup by numeric opcode.
- Operation lookup by mnemonic.

## Machine Constants

| Constant | Value | Meaning |
| --- | ---: | --- |
| `REG_COUNT` | `3` | Number of general-purpose registers exposed by the machine. |
| `MEM_SIZE` | `256` | Memory size in bits. |
| `WORD_SIZE` | `8` | Word size in bits. |

## Operand Types

| Type | Width | Meaning |
| --- | ---: | --- |
| `Register` | 2 bits | General register operand. |
| `MemoryAddress(Data)` | 8 bits | Data-memory address. |
| `MemoryAddress(Code)` | 10 bits | Code-memory address. |
| `Constant` | 8 bits | Immediate value. |

## Opcode Table

Opcodes are indexes in the operation table.

| Opcode | Mnemonic | Operands |
| ---: | --- | --- |
| 0 | `HALT` | none |
| 1 | `IN` | register |
| 2 | `OUT` | register |
| 3 | `OUT_16` | none |
| 4 | `OUT_CHAR` | register |
| 5 | `MOVER` | register, data address |
| 6 | `MOVEM` | register, data address |
| 7 | `MOVEI` | register, constant |
| 8 | `ADD` | register, register, register |
| 9 | `SUB` | register, register, register |
| 10 | `MULT` | register, register, register |
| 11 | `ADDI` | register, register, constant |
| 12 | `SUBI` | register, register, constant |
| 13 | `MULTI` | register, register, constant |
| 14 | `ADC` | register, register, register |
| 15 | `SBC` | register, register, register |
| 16 | `ADCI` | register, register, constant |
| 17 | `SBCI` | register, register, constant |
| 18 | `MULT_16` | register |
| 19 | `MULTI_16` | constant |
| 20 | `JMP` | code address |
| 21 | `JZ` | code address |
| 22 | `JNZ` | code address |
| 23 | `PUSH` | register |
| 24 | `POP` | register |
| 25 | `CALL` | code address |
| 26 | `RET` | none |

## API Notes

- Use `OptSpec::clone()` to create the current operation table.
- Use `get_by_opcode` when decoding bytecode in the VM.
- Use `get_by_operation_name` when converting assembly mnemonics into opcodes.
