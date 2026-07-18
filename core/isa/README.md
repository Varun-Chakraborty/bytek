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
| `MEM_BYTES` | `65536` | Number of addressable memory cells. |
| `MEM_BITS` | `524288` | Total memory capacity in bits. |
| `DATA_BITS` | `8` | Width of data that can be stored in one memory cell/register. |
| `MODE_BIT_COUNT` | `3` | Width of the encoded addressing-mode tag. |

## Operand Types

`AddressingMode::bit_count()` determines operand payload width from the selected mode.

| Type | Width | Meaning |
| --- | ---: | --- |
| `Register` | 3 bits | General register operand. |
| `DirectCode` | 19 bits | Bit address into program memory. |
| `DirectData` | 16 bits | Data-memory address. |
| `Indirect` | 16 bits | Memory address whose contents are dereferenced. |
| `IndirectRegister` | 3 bits | Register-selected memory indirection. |
| `Immediate` | 8 bits | Literal 8-bit value. |

## Opcode Table

Opcodes are indexes in the operation table.

| Opcode | Mnemonic | Operands |
| ---: | --- | --- |
| 0 | `HALT` | none |
| 1 | `IN` | register |
| 2 | `OUT` | register |
| 3 | `MOVER` | register, value |
| 4 | `MOVEM` | register, memory value |
| 5 | `ADD` | register, register, immediate or register |
| 6 | `SUB` | register, register, immediate or register |
| 7 | `ADC` | register, register, immediate or register |
| 8 | `SBC` | register, register, immediate or register |
| 9 | `JMP` | direct code |
| 10 | `JZ` | direct code |
| 11 | `JNZ` | direct code |
| 12 | `PUSH` | register |
| 13 | `POP` | register |
| 14 | `CALL` | direct code |
| 15 | `RET` | none |
| 16 | `CMP` | register, immediate or register |
| 17 | `SHL` | register |
| 18 | `SHR` | register |
| 19 | `AND` | register, register, immediate or register |
| 20 | `OR` | register, register, immediate or register |
| 21 | `XOR` | register, register, immediate or register |

## API Notes

- Use `OptSpec::clone()` to create the current operation table.
- Use `get_by_opcode` when decoding bytecode in the VM.
- Use `get_by_operation_name` when converting assembly mnemonics into opcodes.
- The operand groups used in `OptSpec::clone()` currently distinguish between:
  - register-only operands,
  - direct code addresses,
  - general values (register, direct data, indirect, indirect register, immediate),
  - memory values (direct data, indirect, indirect register — no immediate),
  - and immediate-or-register operands.
