# compiler

`compiler` is the beginning of a higher-level frontend for Bytek.

Right now the crate exposes a library API centered on `MyCompiler`:

- `MyCompiler::new()` constructs the compiler state.
- `MyCompiler::compile(program)` runs the current frontend pipeline.

The current pipeline is intentionally small:

- it creates a compiler-local lexer,
- splits the input into source lines,
- returns an empty placeholder `TokenStream` internally,
- prints that token stream for inspection,
- and returns `Ok(())` from `compile()` when lexing succeeds.

There is no parser, IR, code generator, or binary CLI entrypoint yet.

## Development Notes

- Keep frontend experimentation here, even if the implementation is still skeletal.
- Keep instruction-set details in [`../isa`](../isa/README.md).
- Keep assembly text generation or bytecode encoding in [`../assembler`](../assembler/README.md) unless the compiler needs a higher-level abstraction.
- Add compiler behavior here only when it is separate from assembling existing `.asm` programs.
