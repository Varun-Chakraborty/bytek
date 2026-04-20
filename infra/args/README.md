# args

`args` is a small shared command-line parser used by Bytek command-line crates.

It collects common flags into the `Args` struct:

| Field | Source | Meaning |
| --- | --- | --- |
| `input_filepath` | first positional argument | Input file path. |
| `debug` | `--debug` | Enables debug behavior in callers. |
| `pretty` | `--pretty` | Enables pretty debug formatting in callers. |
| `log_to` | `--log=...` | Caller-defined logging target. |
| `log_file_path` | `--log_path=...` | Caller-defined log directory. |
| `log_filename` | `--log_filename=...` | Caller-defined log file name. |
| `output_filepath` | `--out=...` | Caller-defined output path. |

## API

```rust
let args = args::Args::parse()?;
```

`Args::parse()` reads `std::env::args()` and returns an `ArgsError::InvalidFlag` only for parser-level flag errors.

Each binary is responsible for deciding which parsed fields it supports.
