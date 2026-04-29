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
| `REG_COUNT` | `5` | Number of general-purpose registers exposed by the machine. |
| `MEM_BYTES` | `256` | Number of addressable memory cells. |
| `MEM_BITS` | `2048` | Total memory capacity in bits. |
| `WORD_SIZE` | `8` | Word size in bits. |
| `MODE_BIT_COUNT` | `3` | Width of the encoded addressing-mode tag. |

## Operand Types

`AddressingMode::bit_count()` determines operand payload width from the selected mode.

| Type | Width | Meaning |
| --- | ---: | --- |
| `Register` | 3 bits | General register operand. |
| `DirectCode` | 11 bits | Bit address into program memory. |
| `DirectData` | 8 bits | Data-memory address. |
| `Indirect` | 8 bits | Memory address whose contents are dereferenced. |
| `IndirectRegister` | 3 bits | Register-selected memory indirection. |
| `Immediate` | 8 bits | Literal 8-bit value. |

## Opcode Table

Opcodes are indexes in the operation table.

| Opcode | Mnemonic | Operands |
| ---: | --- | --- |
| 0 | `HALT` | none |
| 1 | `IN` | register |
| 2 | `OUT` | register |
| 3 | `OUT_16` | none |
| 4 | `OUT_CHAR` | register |
| 5 | `MOVER` | register, non-register value |
| 6 | `MOVEM` | register, non-register value |
| 7 | `ADD` | register, register, immediate or register |
| 8 | `SUB` | register, register, immediate or register |
| 9 | `MULT` | register, register, immediate or register |
| 10 | `ADC` | register, register, immediate or register |
| 11 | `SBC` | register, register, immediate or register |
| 12 | `MULT_16` | immediate or register |
| 13 | `JMP` | direct code |
| 14 | `JZ` | direct code |
| 15 | `JNZ` | direct code |
| 16 | `PUSH` | register |
| 17 | `POP` | register |
| 18 | `CALL` | direct code |
| 19 | `RET` | none |
| 20 | `CMP` | register, immediate or register |

## API Notes

- Use `OptSpec::clone()` to create the current operation table.
- Use `get_by_opcode` when decoding bytecode in the VM.
- Use `get_by_operation_name` when converting assembly mnemonics into opcodes.
- The operand groups used in `OptSpec::clone()` currently distinguish between:
  - register-only operands,
  - direct code addresses,
  - non-register values,
  - and general values that may also be registers.
