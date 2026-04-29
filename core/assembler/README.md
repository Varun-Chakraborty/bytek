# assembler

`assembler` converts Bytek assembly source into VM bytecode.

The crate owns the source-to-bytecode pipeline:

- Preprocess source before semantic analysis.
- Lex assembly text into tokens.
- Parse tokens into statements.
- Resolve operation names and operands through the `isa` crate.
- Encode instructions, raw numeric data, and string data into bytes.
- Write binary output, plus optional debug output.

## Usage

From the workspace root:

```bash
cargo run -p assembler programs/kernel.asm
```

By default this writes `output.bin`.

To choose the output path:

```bash
cargo run -p assembler programs/kernel.asm --out=kernel.bin
```

To also write an ASCII bit dump:

```bash
cargo run -p assembler programs/kernel.asm --debug
```

To prettify the debug dump with instruction delimiters:

```bash
cargo run -p assembler programs/kernel.asm --debug --pretty
```

The binary currently accepts these parsed flags from `infra/args`:

- first positional argument: input `.asm` path
- `--debug`
- `--pretty`
- `--out=...`

Other shared logging flags are parsed by `args`, but the assembler binary does not currently wire them into runtime behavior.

## Accepted Input

The command-line binary accepts files ending in `.asm`.

Assembly statements generally look like:

```asm
LABEL: OPCODE OPERAND, OPERAND, OPERAND
```

Labels are optional. Operands are interpreted using the operand specs from [`../isa`](../isa).
The preprocessor runs before lexing, and the parser supports assembler directives that become raw binary data in the encoded stream.

Example:

```asm
MOVE: MOVER R0, 0
```

Arithmetic instructions such as `ADD`, `SUB`, `ADC`, `SBC`, and `MULT` are encoded as three-operand instructions. As a convenience, the semantic parser accepts two-operand forms and rewrites them so the first operand is also the destination:

```asm
ADD R0, #1
```

is normalized as:

```asm
ADD R0, R0, #1
```

Preprocessor statements currently supported:

| Statement | Operand | Behavior |
| --- | --- | --- |
| `.include` | one double-quoted `.asm` file path | Replaces the statement with the contents of `programs/<path>`. |

For example, [`programs/kernel.asm`](../../programs/kernel.asm) can pull in shared assembly routines:

```asm
.include "stdlib.asm"
```

Include paths must be double-quoted and end in `.asm`. The current implementation resolves them from the workspace `programs` directory, so assembler commands should be run from the repository root when using includes.

Directives currently supported by the semantic parser:

| Directive | Operands | Encoded output |
| --- | --- | --- |
| `.byte` | one integer constant | One 8-bit byte. |
| `.ascii` | one quoted string | One 8-bit byte per character whose scalar value fits in 8 bits. |
| `.align` | none | Zero bits until the next byte boundary. |

String operands are accepted for directives such as `.ascii`:

```asm
.ascii "Hello\n"
.ascii "Tabbed\ttext"
.ascii "Null\0terminated"
```

The lexer decodes `\n`, `\t`, and `\0` inside strings. Other escape sequences are rejected. Because `.ascii` emits 8-bit values, characters with scalar values above `255` fail during encoding. String operands are not valid instruction operands unless a future ISA operation explicitly supports them.

## Output

The writer creates:

- `output.bin` by default, or the path passed through `--out=...`.
- `debug.txt` when `--debug` is enabled.
- A 4-byte big-endian EOF marker is appended to the binary so the VM knows the program's effective bit length.

The VM currently looks for `kernel.bin`, so use `--out=kernel.bin` when preparing a program for `cargo run -p vm`.

## Internal Structure

- `preprocessor`: expands supported source-level statements such as `.include`.
- `lexer`: tokenizes source text and tracks source locations.
- `parser`: converts tokens into validated semantic nodes.
- `encoder`: turns semantic nodes into bytes.
- `writer`: writes binary output and an optional ASCII/debug bit view.

## Relationship To Other Crates

- Depends on `isa` for operation names, opcodes, and operand widths.
- Depends on `args` for shared CLI flag parsing.
- Depends on `logger` as shared infrastructure, although the current assembler binary does not actively emit structured logs through it.
