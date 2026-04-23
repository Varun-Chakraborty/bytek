# assembler

`assembler` converts Bytek assembly source into VM bytecode.

The crate owns the source-to-bytecode pipeline:

- Lex assembly text into tokens.
- Preprocess source before semantic analysis.
- Parse tokens into statements.
- Resolve operation names and operands through the `isa` crate.
- Encode instructions and raw data into bytes.
- Write binary output, plus optional debug output.

## Usage

From the workspace root:

```bash
cargo run -p assembler programs/index.asm
```

By default this writes `output.bin`.

To choose the output path:

```bash
cargo run -p assembler programs/index.asm --out=kernel.bin
```

To also write an ASCII bit dump:

```bash
cargo run -p assembler programs/index.asm --debug
```

To prettify the debug dump with instruction delimiters:

```bash
cargo run -p assembler programs/index.asm --debug --pretty
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
The parser also supports assembler directives that become raw binary data in the encoded stream.

Example:

```asm
MOVE: MOVER R0, 0
```

## Output

The writer creates:

- `output.bin` by default, or the path passed through `--out=...`.
- `debug.txt` when `--debug` is enabled.
- A 4-byte big-endian EOF marker is appended to the binary so the VM knows the program's effective bit length.

The VM currently looks for `kernel.bin`, so use `--out=kernel.bin` when preparing a program for `cargo run -p vm`.

## Internal Structure

- `lexer`: tokenizes source text and tracks source locations.
- `preprocessor`: handles preprocessing before parse.
- `parser`: converts tokens into validated semantic nodes.
- `encoder`: turns semantic nodes into bytes.
- `writer`: writes binary output and an optional ASCII/debug bit view.

## Relationship To Other Crates

- Depends on `isa` for operation names, opcodes, and operand widths.
- Depends on `args` for shared CLI flag parsing.
- Depends on `logger` as shared infrastructure, although the current assembler binary does not actively emit structured logs through it.
