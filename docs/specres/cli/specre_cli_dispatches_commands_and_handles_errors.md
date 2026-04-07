---
id: "01KHFFCX8BCDAYP8YHG0J65H0E"
name: "specre_cli_dispatches_commands_and_handles_errors"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/main.rs`
- `src/cli.rs`
- `src/commands/mod.rs`
- `tests/cli_dispatch.rs` (Test)
- `tests/common/mod.rs` (Test helper)

## Functional Overview

The `specre` binary parses command-line arguments via clap, dispatches each subcommand to its corresponding handler, and provides uniform error handling. A global `--json` flag is propagated to each handler that supports it. When a handler returns an error, the binary prints the message to stderr with an `Error:` prefix and exits with code 1 — except for `NonZeroExit`, which exits with code 1 silently (no message).

## Design Intent

The dispatch layer exists to provide a consistent interface and error behavior across all specre subcommands. By centralizing argument parsing (`cli.rs`), module registration (`commands/mod.rs`), and error handling (`main.rs`) in three thin files, each individual command handler can focus purely on its domain logic and return `Result` without worrying about exit codes or output formatting.

## Key Members

- `Cli` — top-level clap struct with `json: bool` (global `--json` flag) and `command: Commands`
- `Commands` — enum of all subcommands (`Init`, `New`, `Index`, `Status`, `Trace`, `Orphans`, `Tag`, `Coverage`, `HealthCheck`, `Search`, `Mcp`, `Destroy`, `Drift`), each variant holding its respective `Args` struct or no data
- `main()` — parses `Cli`, extracts the `json` flag, matches on `Commands`, calls the handler's `execute()` (passing `json` where supported), and handles the result with a three-way match:
  - `Ok(())` → exit code 0
  - `Err(SpecreError::NonZeroExit)` → exit code 1, no message (used by commands like `orphans` and `drift` to signal a non-zero result without an error message)
  - `Err(e)` → prints `Error: {e}` to stderr, exit code 1

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

### Exits silently with code 1 for NonZeroExit

1. User runs a subcommand that returns `Err(SpecreError::NonZeroExit)` (e.g., `specre orphans` when orphans are found, or `specre drift` when drift is detected)
2. The handler has already printed its own output to stdout
3. CLI exits with code 1 without printing any additional error message to stderr

## Failures / Exceptions

- If a handler returns `Err(SpecreError::NonZeroExit)`, the process exits with code 1 silently — no message is printed. This is used by commands whose non-zero exit code signals a meaningful result (e.g., orphans found, drift detected) rather than an error.
- If a handler returns any other `Err`, the error message is printed to stderr with `Error:` prefix and the process exits with code 1.
