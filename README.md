# Bytek
![Rust](https://img.shields.io/badge/Rust-1.89.0-orange)
![MIT](https://img.shields.io/badge/License-MIT-green)
![GitHub release (latest by date)](https://img.shields.io/github/v/release/Varun-Chakraborty/compiler)
[![Release](https://github.com/Varun-Chakraborty/compiler/actions/workflows/release.yml/badge.svg)](https://github.com/Varun-Chakraborty/compiler/actions/workflows/release.yml)

Bytek is a Rust workspace for experimenting with a small custom instruction set, an assembler, a virtual machine, and the early pieces of a higher-level compiler frontend.

The repository started as a Java VM simulator and assembler. That version is archived in the [`java-archive`](https://github.com/Varun-Chakraborty/compiler/tree/java-archive) tag; active development now happens in Rust.

## Workspace

The root README stays intentionally high level. Crate-specific behavior, command usage, and implementation notes live beside each crate:

| Crate | Path | Purpose |
| --- | --- | --- |
| `isa` | [`core/isa`](./core/isa/README.md) | Shared instruction set, operand specs, and machine constants. |
| `assembler` | [`core/assembler`](./core/assembler/README.md) | Converts `.asm` source into bytecode for the VM. |
| `vm` | [`core/vm`](./core/vm/README.md) | Executes bytecode using the shared ISA. |
| `compiler` | [`core/compiler`](./core/compiler/README.md) | Early compiler frontend crate with a scaffold lexer and compile entrypoint. |
| `args` | [`infra/args`](./infra/args/README.md) | Shared command-line flag parser. |
| `logger` | [`infra/logger`](./infra/logger/README.md) | Lightweight console/file logging helper. |

## Quick Start

Install Rust, then build the full workspace:

```bash
cargo build --workspace
```

Assemble a program:

```bash
cargo run -p assembler programs/index.asm --debug --pretty --out=kernel.bin
```

Run the VM from the repository root:

```bash
cargo run -p vm
```

The VM currently loads `kernel.bin` from the current working directory and has no separate CLI flags yet.

## Repository Layout

- [`core`](./core): core system crates.
- [`infra`](./infra): shared support crates used by command-line tools.
- [`programs`](./programs): sample assembly programs.
- [`scripts`](./scripts): helper scripts.

## How The Pieces Fit

1. Assembly source in [`programs`](./programs) is passed to the `assembler`.
2. The `assembler` uses the shared `isa` crate to validate operation names and operands.
3. The `assembler` writes bytecode, normally as `output.bin` or a path passed with `--out=...`.
4. The `vm` loads `kernel.bin`, decodes bytes using the same `isa` crate, and executes instructions step by step.
5. The `compiler` crate is currently a library-only work area. Its `compile()` path only lexes input and prints the token stream scaffold, but that crate is where higher-level language work is headed.

## Development

Run all workspace tests:

```bash
cargo test --workspace
```

Build release binaries:

```bash
cargo build --workspace --release
```

## Motivation

"Feels good to write 0s and 1s and see them do something."

This project is a practical step toward understanding system software, the fetch-decode-execute cycle, and the bridge between source text and machine behavior.

## License

The project is released under the [MIT License](./LICENSE).

## Contributing

Contributions are welcome. Fork the repository and open a pull request.
