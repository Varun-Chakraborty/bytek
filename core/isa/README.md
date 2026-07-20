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

## Register Usage

The five general-purpose registers (`R0`-`R4`) have de facto conventions established by the standard library:

| Register | Role |
| --- | --- |
| `R0` | Return value register. Callee writes results here. For 16-bit results, `R0` holds the high byte and `R1` the low byte. |
| `R1` | Primary argument / input register. First argument to stdlib routines. |
| `R2` | Second argument. Callee-saved. |
| `R3` | Third argument / scratch. Callee-saved. |
| `R4` | Scratch / loop counter. Callee-saved. |

These are conventions only, not enforced by hardware. Any register can be used for any purpose.

## Flag Behavior

Four condition flags (`zero`, `sign`, `overflow`, `carry`) are updated by arithmetic, comparison, bitwise, and shift instructions. Movement, control flow, and stack instructions do not modify flags.

| Instruction | zero | sign | carry | overflow |
| --- | ---: | ---: | ---: | ---: |
| `ADD`, `ADC` | result == 0 | MSB set | unsigned sum > 255 | signed overflow |
| `SUB`, `SBC` | result == 0 | MSB set | num1 < num2 (borrow) | signed overflow |
| `CMP` | result == 0 | MSB set | num1 < num2 | signed overflow |
| `SHL` | result == 0 | MSB set | old MSB | cleared |
| `SHR` | result == 0 | MSB set | old LSB | cleared |
| `AND`, `OR`, `XOR` | result == 0 | MSB set | cleared | cleared |

`CMP` performs the same subtraction and flag computation as `SUB` but does not write the result back. `ADC` and `SBC` read the current carry flag, enabling multi-precision arithmetic.

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
