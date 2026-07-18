# vm

`vm` executes Bytek bytecode.

The VM uses the shared [`isa`](../isa/README.md) crate to decode instructions, then applies each instruction to registers and memory.

## Usage

From the workspace root:

```bash
cargo run -p vm
```

The current binary entrypoint loads `kernel.bin` from the current working directory. To assemble a program into that path:

```bash
cargo run -p assembler programs/kernel.asm --out=kernel.bin
```

Then run:

```bash
cargo run -p vm
```

## Runtime Model

`MyVM` owns:

- `device`: the byte-oriented input/output device used by `IN` and `OUT`.
- `registers`: general registers, flags, program counter, and execution metadata.
- `memory`: 65,536 cells of 8-bit VM memory.
- `opt_spec`: the opcode table from `isa`.
- `logger`: debug logging support.

## Machine Architecture

The VM is an 8-bit machine with a compact bit-level instruction encoding.

| Part | Shape | Notes |
| --- | --- | --- |
| Word size | 8 bits | Registers and memory cells store `u8` values. Some operations interpret those bytes as signed `i8` values. |
| General registers | `R0` through `R4` | The register index is encoded in 3 bits, while the current machine exposes 5 registers. |
| Memory | 65,536 cells | Data memory is addressed by 16-bit operands, so data addresses range from `0` to `65535`. |
| Program counter | Bit address | `pc` points to the next bit to decode, not just the next byte. |
| Addressing-mode tag | 3 bits | Every encoded operand starts with a 3-bit addressing-mode field. |
| EOF | Bit address | Loaded programs carry their effective bit length, and execution stops when `pc` reaches `eof`. |
| Stack pointer | Memory cell address | `sp` starts at `MEM_BYTES`, one past the last valid memory cell, and grows downward for `PUSH`, `POP`, `CALL`, and `RET`. The first pushed value lands at address `65535`. |
| Flags | `zero`, `sign`, `overflow`, `carry` | Arithmetic, comparison, bitwise, and shift instructions update condition flags used by conditional jumps. |

Instruction decoding is driven by the shared `isa` crate:

- Every instruction starts with a 6-bit opcode.
- Non-empty operands begin with a 3-bit addressing-mode tag.
- Register operands are 3 bits.
- Data-memory operands are 16 bits.
- Code-address operands are 19 bits because they target bit positions in the 65,536-byte program-memory space.
- Immediate operands are 8 bits.

The VM stores program bytes in the same fixed-size memory structure it uses for data. Memory writes overwrite an existing cell instead of inserting a new one, so addresses stay stable from `0` through `65535`. `Instruction::new` reads bits from memory starting at `pc`, records that starting `pc` for debug output, consumes the opcode and operands according to the ISA table, and advances `pc` as it decodes. Jump and call instructions then overwrite `pc` with a code-address operand.

`CALL` stores the decoded return address on the stack before jumping. Return addresses currently use three bytes: the low byte is pushed first, then the middle byte, then the high byte. `RET` reads the high byte, middle byte, and low byte, rebuilds the bit address, advances `sp` past all three bytes, and resumes execution at that address.

Assembled binaries end with a 4-byte big-endian EOF marker. `load_kernel()` strips those final 4 bytes, stores the decoded bit length in `registers.eof`, loads the remaining bytes into memory, resets `pc` to `0`, and starts execution.

Execution follows the usual fetch-decode-execute loop:

1. Read an instruction from memory at the program counter.
2. Decode the opcode and operands through `isa::OptSpec`.
3. Dispatch to the matching handler.
4. Update registers, flags, memory, and the program counter.
5. Stop when the program counter reaches EOF.

`IN` and `OUT` are byte-oriented. The default binary constructs `MyVM` with `ConsoleDevice`, which reads one byte from standard input and writes output bytes as characters to standard output.

## Devices

VM I/O is routed through the `Device` trait:

```rust
pub trait Device {
    fn read_byte(&mut self) -> Result<u8, std::io::Error>;
    fn write_byte(&mut self, byte: u8) -> Result<(), std::io::Error>;
}
```

This keeps instruction execution independent from a specific terminal or host I/O implementation. `IN Rn` calls `read_byte()` and stores the returned byte in the target register. `OUT Rn` reads the target register and passes that byte to `write_byte()`.

`ConsoleDevice` is the default device used by the VM binary. It reads one raw byte from standard input and writes bytes to standard output as characters. Tests or alternate frontends can provide a different `Device` implementation to capture output, feed scripted input, or connect the VM to another host interface.

## Public API

- `MyVM::new(device)` creates a VM with empty memory, registers, and the provided I/O device.
- `load_kernel()` loads `kernel.bin` and runs it.
- `start()` currently calls `load_kernel()`.
- `step()` executes a single instruction and returns an `ExecutionStep` with the decoded instruction text, resulting program-counter address, and halted status.
- `run()` executes until the loaded program halts or reaches EOF.
- `reset()` clears registers and memory.
- `get_state()` returns a borrowed snapshot of registers and memory.

When `debug` is enabled, each decoded instruction is logged with the bit-address `PC` where decoding began, followed by the opcode, operation name, and decoded operands.

## Implemented Instructions

The VM currently dispatches:

- Halt and I/O: `HALT`, `IN`, `OUT`
- Movement: `MOVER`, `MOVEM`
- Arithmetic: `ADD`, `SUB`
- Carry arithmetic: `ADC`, `SBC`
- Comparison: `CMP`
- Bitwise: `AND`, `OR`, `XOR`
- Shifts: `SHL`, `SHR`
- Control flow: `JMP`, `JZ`, `JNZ`, `CALL`, `RET`
- Stack: `PUSH`, `POP`

Instructions present in `isa` should only be considered executable once they have a matching VM handler.
