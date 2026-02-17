---
id: "01KHMEB8WF7BFZASE8SQHF5PR2"
name: "specre_error_provides_contextual_diagnostics"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/error.rs`
- `src/main.rs` (error dispatch)

## Functional Overview

`SpecreError` is the unified error type for the specre CLI. Every command returns `Result<(), SpecreError>`, and `main()` dispatches errors to stderr with an `Error: ` prefix and exit code 1. Each variant carries enough context for the user to diagnose the problem — notably, `Io` includes the file path that failed. `NonZeroExit` is the exception: it signals a non-zero exit without printing any message, used by commands like `orphans` and `health-check` that report findings via stdout but still need to indicate failure to CI pipelines.

## Design Intent

A single error enum ensures consistent formatting across all commands. The `Display` implementation is the user-facing contract: every error message printed to stderr originates from `SpecreError::fmt()`. By embedding the file path in `Io` errors, users see which file caused the failure without needing to cross-reference log output. The `std::error::Error::source()` chain preserves the original error for programmatic consumers.

## Key Members

- `AlreadyInitialized` — `specre init` called twice
- `ConfigNotFound` — `specre.toml` missing
- `ConfigParse(toml::de::Error)` — `specre.toml` syntax error
- `Io { path: PathBuf, source: std::io::Error }` — filesystem failure with path context
- `Serialize(serde_json::Error)` — JSON serialization failure
- `InvalidArgument(String)` — caller-provided message for bad CLI input
- `NonZeroExit` — silent exit code 1 (no stderr output)
- `ConfigSerialize(toml::ser::Error)` — TOML serialization failure (specre.toml generation)
- `TokioRuntime(std::io::Error)` — tokio runtime creation failure
- `McpInit(String)` — MCP server initialization failure
- `McpTask(String)` — MCP server runtime task error

## Scenarios

### Each variant formats a human-readable message

1. `AlreadyInitialized` displays `"specre.toml already exists. This project is already initialized."`
2. `ConfigNotFound` displays `"specre.toml not found. Run 'specre init' first."`
3. `ConfigParse(e)` displays `"Failed to parse specre.toml: <inner message>"`
4. `Io { path, source }` displays `"Failed to access '<path>': <inner message>"`
5. `Serialize(e)` displays `"Failed to serialize: <inner message>"`
6. `InvalidArgument(msg)` displays the message as-is (no prefix)
7. `ConfigSerialize(e)` displays `"Failed to serialize specre.toml: <inner message>"`
8. `TokioRuntime(e)` displays `"Failed to create tokio runtime: <inner message>"`
9. `McpInit(e)` displays `"Failed to initialize MCP server: <inner message>"`
10. `McpTask(e)` displays `"MCP server error: <inner message>"`

### NonZeroExit produces an empty display string

1. `NonZeroExit.to_string()` returns `""`
2. In `main()`, `NonZeroExit` triggers `process::exit(1)` without printing to stderr

### source() preserves the error chain

1. `ConfigParse(e)` returns `Some(e)` from `source()`
2. `Io { source, .. }` returns `Some(source)` from `source()`
3. `Serialize(e)` returns `Some(e)` from `source()`
4. `ConfigSerialize(e)` returns `Some(e)` from `source()`
5. `TokioRuntime(e)` returns `Some(e)` from `source()`
6. `AlreadyInitialized`, `ConfigNotFound`, `InvalidArgument`, `NonZeroExit`, `McpInit`, `McpTask` return `None` from `source()`

### From<serde_json::Error> converts to Serialize variant

1. A `serde_json::Error` is converted to `SpecreError::Serialize` via `From`
2. The original error is preserved inside the variant

### main() dispatches errors to stderr with "Error: " prefix

1. When a command returns `Err(SpecreError::NonZeroExit)`, `main()` calls `process::exit(1)` without printing
2. When a command returns any other `Err(e)`, `main()` prints `"Error: {e}"` to stderr and calls `process::exit(1)`
3. When a command returns `Ok(())`, `main()` exits normally (exit code 0)

## Failures / Exceptions

- `SpecreError` implements `std::error::Error` and `Display`; it does not implement `Clone` or `PartialEq`
- `McpInit` and `McpTask` store stringified messages because rmcp's error types (`ServerInitializeError`, `tokio::task::JoinError`) are not stored directly to avoid tight coupling with rmcp internals
