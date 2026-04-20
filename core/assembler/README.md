# assembler

`assembler` converts Bytek assembly source into VM bytecode.

The crate owns the source-to-bytecode pipeline:

- Lex assembly text into tokens.
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

## Accepted Input

The command-line binary accepts files ending in `.asm`.

Assembly statements generally look like:

```asm
LABEL: OPCODE OPERAND, OPERAND, OPERAND
```

Labels are optional. Operands are interpreted using the operand specs from [`../isa`](../isa).

Example:

```asm
MOVE: MOVER R0, 0
```

## Output

The writer creates:

- `output.bin` by default, or the path passed through `--out=...`.
- `debug.txt` when `--debug` is enabled.

The VM currently looks for `kernel.bin`, so use `--out=kernel.bin` when preparing a program for `cargo run -p vm`.

## Internal Structure

- `lexer`: tokenizes source text and tracks source locations.
- `preprocessor`: handles preprocessing before parse.
- `parser`: converts tokens into validated semantic nodes.
- `encoder`: turns semantic nodes into bytes.
- `writer`: writes binary and optional debug output.

## Relationship To Other Crates

- Depends on `isa` for operation names, opcodes, and operand widths.
- Depends on `args` for shared CLI flag parsing.
- Depends on `logger` as shared infrastructure.
