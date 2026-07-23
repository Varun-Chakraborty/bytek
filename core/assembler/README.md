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

Arithmetic and bitwise instructions such as `ADD`, `SUB`, `ADC`, `SBC`, `AND`, `OR`, and `XOR` are encoded as three-operand instructions. As a convenience, the semantic parser accepts two-operand forms and rewrites them so the first operand is also the destination:

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
| `.include` | one double-quoted `.asm` file path | Replaces the statement with the contents of `programs/<path>`. Included files are preprocessed recursively, so an included file can itself include other files. |

For example, [`programs/kernel.asm`](../../programs/kernel.asm) can pull in the Bytek standard library:

```asm
.include "stdlib.asm"
```

Include paths must be double-quoted and end in `.asm`. The current implementation resolves them from the workspace `programs` directory, so assembler commands should be run from the repository root when using includes.

Programs that include the standard library before their entrypoint should jump over it first, because execution starts at the first encoded instruction:

```asm
JMP START

.include "stdlib.asm"

START:
    ; program code
```

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

## Standard Library

[`programs/stdlib.asm`](../../programs/stdlib.asm) is the Bytek assembly standard library. It is split into three modules, all included by `stdlib.asm`:

```asm
.include "math_stdlib.asm"
.include "string_stdlib.asm"
.include "print_stdlib.asm"
```

Individual modules can also be included directly when only a subset of routines is needed.

### Math (`math_stdlib.asm`)

| Routine | Inputs | Outputs | Preserved | Clobbers |
| --- | --- | --- | --- | --- |
| `MULT` | `R2`: first operand, `R3`: second operand | `R0`: high byte, `R1`: low byte of 16-bit product | `R2`, `R3`, `R4` | flags |
| `DIV` | `R2`: numerator, `R3`: denominator | `R0`: quotient, `R1`: remainder | `R2`, `R3`, `R4` | flags |

`MULT` performs 8×8→16-bit multiplication via shift-and-add. `DIV` performs repeated-subtraction division and also produces the remainder in `R1`.

### Strings (`string_stdlib.asm`)

| Routine | Inputs | Outputs | Preserved | Clobbers |
| --- | --- | --- | --- | --- |
| `COMPARE_STRINGS` | `R1`: first string address, `R2`: second string address | `R0`: `0` if equal, `1` otherwise | `R1`, `R2`, `R3`, `R4` | flags |
| `STRLEN` | `R1`: address of a null-terminated string | `R0`: string length in bytes, excluding `\0` | `R1`, `R3` | flags |

All string routines expect null-terminated strings.

### Print (`print_stdlib.asm`)

| Routine | Inputs | Outputs | Preserved | Clobbers |
| --- | --- | --- | --- | --- |
| `PRINT_STRING` | `R1`: address of a null-terminated string | none | `R1`, `R3` | flags |
| `PRINT_INT` | `R1`: unsigned byte value to print in decimal | none | `R1`, `R2`, `R3`, `R4` | flags |
| `PRINTLN` | none | none | `R3` | flags |

`PRINT_INT` prints the byte value in `R1`, so values are limited to `0..255`.

## Conventions

### Comments

Comments begin with `;` and extend to the end of the line. They are stripped during lexing and produce no output:

```asm
; This is a comment
ADD R0, R1  ; Inline comment
```

### Labels

Labels are identifiers followed by a colon. They must start with a letter and contain only letters, digits, and underscores (`^[A-Za-z][A-Za-z0-9_]*$`). Labels can appear on their own line or before an instruction on the same line:

```asm
LOOP:
    ADD R0, #1
    JNZ LOOP

NEXT: HALT
```

Labels resolve to **bit offsets** for code addressing and **byte offsets** for data addressing. Forward references are supported.

### Addressing Mode Syntax

| Mode | Syntax | Example | Notes |
| --- | --- | --- | --- |
| Register | bare identifier | `R0` | Must match `R[0-4]`. |
| Immediate | `#` prefix | `#42`, `#PROMPT` | `#` followed by a number or label. Label resolves to a byte address. |
| DirectData | bare identifier | `DATA`, `0` | Used by `MOVER`/`MOVEM`. Resolves to a byte address. |
| DirectCode | bare identifier | `START` | Used by `JMP`/`JZ`/`JNZ`/`CALL`. Resolves to a bit address. |
| Indirect | `[...]` | `[0]`, `[DATA]` | Brackets around a data address. |
| IndirectRegister | `[Rn]` | `[R0]` | Brackets around a register. |

### Strings

Strings are delimited by double quotes. Supported escape sequences: `\n` (newline), `\t` (tab), `\0` (null). Other escape sequences are rejected. Strings are used with the `.ascii` directive, not as instruction operands.

All stdlib string routines (`PRINT_STRING`, `COMPARE_STRINGS`, `STRLEN`) expect **null-terminated** strings — always end strings with `\0`:

```asm
MSG:
.ascii "Hello World\n\0"
```

### Program Entry Point

Data directives (`.ascii`, `.byte`, `.align`) and `.include` files are placed inline in the code stream. Since execution starts at the first encoded instruction, programs that place data or includes before their entry point must jump over them:

```asm
JMP START

.align

PROMPT:
.ascii "Enter a number: \0"
.include "stdlib.asm"

START:
    ; actual program code
```

### Subroutine Structure

Every subroutine saves registers it will modify on entry and restores them before returning:

```asm
FUNC:
    PUSH R2
    PUSH R3
    ...            ; body
    POP R3
    POP R2
    RET
```

Registers are restored in the reverse order they were pushed. Only callee-saved registers (`R2`, `R3`, `R4`) need this treatment; `R0` and `R1` are caller-saved and do not need preservation.

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
