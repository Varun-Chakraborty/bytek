# logger

`logger` provides a small logging helper for Bytek crates.

It can write messages to:

- Standard output through `LogTo::Console`.
- A file through `LogTo::File`.

## API

```rust
use logger::{LogTo, Logger};

let mut logger = Logger::new(
    String::from("vm.txt"),
    String::from("/logs/"),
    LogTo::Console,
)?;

logger.log(String::from("Starting execution"))?;
```

## Notes

- `Logger::new` creates a file handle when `LogTo::File` is selected.
- `Logger::log` writes one message at a time.
- Callers own decisions about when logging is enabled and where logs should be written.
