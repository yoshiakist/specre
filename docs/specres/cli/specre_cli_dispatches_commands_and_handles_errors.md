---
id: "01KHFFCX8BCDAYP8YHG0J65H0E"
name: "specre_cli_dispatches_commands_and_handles_errors"
status: "stable"
last_verified: "2026-02-15"
---

## Related Files

- `src/main.rs`
- `src/cli.rs`
- `src/commands/mod.rs`
- `tests/cli_dispatch.rs` (Test)

## Functional Overview

The `specre` binary parses command-line arguments via clap, dispatches each subcommand to its corresponding handler, and provides uniform error handling. When any handler returns an error, the binary prints the message to stderr with an `Error:` prefix and exits with code 1.

## Design Intent

The dispatch layer exists to provide a consistent interface and error behavior across all specre subcommands. By centralizing argument parsing (`cli.rs`), module registration (`commands/mod.rs`), and error handling (`main.rs`) in three thin files, each individual command handler can focus purely on its domain logic and return `Result` without worrying about exit codes or output formatting.

## Key Members

- `Cli` — top-level clap struct with a single `command: Commands` field
- `Commands` — enum of all subcommands (`Init`, `New`, `Index`, `Status`, `Trace`, `Orphans`, `Tag`, `Coverage`), each variant holding its respective `Args` struct or no data
- `main()` — parses `Cli`, matches on `Commands`, calls the handler's `execute()`, and converts `Err` to stderr + exit code 1

## Scenarios

### Dispatches a valid subcommand to its handler

1. User runs `specre init` (or any other valid subcommand)
2. CLI parses the arguments and identifies the subcommand
3. CLI calls the corresponding handler's `execute()` function
4. The handler runs successfully and the process exits with code 0

### Exits with error when handler fails

1. User runs a subcommand that triggers an error in its handler (e.g., `specre status` without `specre.toml`)
2. The handler returns `Err` with an error message
3. CLI prints `Error: <message>` to stderr
4. CLI exits with code 1

## Failures / Exceptions

- If a handler returns `Err`, the error message is printed to stderr with `Error:` prefix and the process exits with code 1
