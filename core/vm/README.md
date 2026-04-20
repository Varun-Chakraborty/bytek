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
cargo run -p assembler programs/index.asm --out=kernel.bin
```

Then run:

```bash
cargo run -p vm
```

## Runtime Model

`MyVM` owns:

- `registers`: general registers, flags, program counter, and execution metadata.
- `memory`: 256 cells of 8-bit VM memory.
- `opt_spec`: the opcode table from `isa`.
- `logger`: debug logging support.

## Machine Architecture

The VM is an 8-bit machine with a compact bit-level instruction encoding.

| Part | Shape | Notes |
| --- | --- | --- |
| Word size | 8 bits | Registers and memory cells store `u8` values. Some operations interpret those bytes as signed `i8` values. |
| General registers | `R0` through `R2` | The register index is encoded in 2 bits, while the current machine exposes 3 registers. |
| Memory | 256 cells | Data memory is addressed by 8-bit operands, so data addresses range from `0` to `255`. |
| Program counter | Bit address | `pc` points to the next bit to decode, not just the next byte. |
| EOF | Bit address | Loaded programs carry their effective bit length, and execution stops when `pc` reaches `eof`. |
| Stack pointer | Memory cell address | `sp` starts at the top of memory and grows downward for `PUSH`, `POP`, `CALL`, and `RET`. |
| Flags | `zero`, `sign`, `overflow`, `carry` | Arithmetic and movement instructions update condition flags used by conditional jumps. |

Instruction decoding is driven by the shared `isa` crate:

- Every instruction starts with a 6-bit opcode.
- Register operands are 2 bits.
- Data-memory operands are 8 bits.
- Code-address operands are 10 bits because they target bit positions in program memory.
- Immediate operands are 8 bits.

The VM stores program bytes in the same memory structure it uses for data. `Instruction::new` reads bits from memory starting at `pc`, consumes the opcode and operands according to the ISA table, and advances `pc` as it decodes. Jump and call instructions then overwrite `pc` with a code-address operand.

Assembled binaries end with a 4-byte big-endian EOF marker. `load_kernel()` strips those final 4 bytes, stores the decoded bit length in `registers.eof`, loads the remaining bytes into memory, resets `pc` to `0`, and starts execution.

Execution follows the usual fetch-decode-execute loop:

1. Read an instruction from memory at the program counter.
2. Decode the opcode and operands through `isa::OptSpec`.
3. Dispatch to the matching handler.
4. Update registers, flags, memory, and the program counter.
5. Stop when the program counter reaches EOF.

## Public API

- `MyVM::new()` creates a VM with empty memory and registers.
- `load_kernel()` loads `kernel.bin` and runs it.
- `start()` currently calls `load_kernel()`.
- `step()` executes a single instruction and returns an `ExecutionStep`.
- `run()` executes until the loaded program halts or reaches EOF.
- `reset()` clears registers and memory.
- `get_state()` returns a borrowed snapshot of registers and memory.

## Implemented Instructions

The VM currently dispatches:

- Halt and I/O: `HALT`, `IN`, `OUT`, `OUT_16`, `OUT_CHAR`
- Movement: `MOVER`, `MOVEM`, `MOVEI`
- Arithmetic: `ADD`, `SUB`, `MULT`, `ADDI`, `SUBI`, `MULTI`
- Carry arithmetic: `ADC`, `SBC`, `ADCI`, `SBCI`
- 16-bit multiply helpers: `MULT_16`, `MULTI_16`
- Control flow: `JMP`, `JZ`, `JNZ`, `CALL`, `RET`
- Stack: `PUSH`, `POP`

Instructions present in `isa` should only be considered executable once they have a matching VM handler.
