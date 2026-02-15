---
id: "01KHB48DYZDN8GHXPX7MSYJ1NZ"
name: "specre_trace_resolves_bidirectional_references"
status: "stable"
last_verified: "2026-02-13"
---

## Related Files

- `src/commands/trace.rs`
- `src/commands/index.rs` (reuses scanning helpers)
- `src/config.rs`
- `tests/cli_trace.rs` (Test)

## Functional Overview

`specre trace <query>` performs a bidirectional traceability lookup. The argument is auto-detected: if it matches a 26-character uppercase alphanumeric ULID, the command shows the specre file and all source files referencing it. If it is a file path, the command reads the file for `@specre` markers and shows all linked specre cards.

## Design Intent

Traceability is a core promise of specre. The trace command makes it trivial to answer "where is this behavior specified?" and "where is this behavior implemented?" in a single invocation. By accepting both ULIDs and file paths, it supports both directions of the traceability link without requiring a separate command.

The output is designed for both human developers navigating a codebase and AI agents resolving references during code generation.

## Key Members

- `query: String` — either a 26-character ULID or a file path (positional argument, auto-detected)

## Scenarios

### Basic ULID invocation shows specre and source references

1. User runs `specre trace 01HZYPMZRK8F9R2DGBGGMM2N8T` in a project with `specre.toml`
2. CLI reads `specre.toml` to determine `specre_dir` and `source_dirs`
3. CLI scans specre files for a matching `id` in front-matter
4. CLI scans source files for `@specre 01HZYPMZRK8F9R2DGBGGMM2N8T` markers (filtered by `target_extensions` if set)
5. CLI prints:
   ```
   Specre:
     docs/specres/quotation/user_can_create_new_quotation.md

   Source references:
     src/usecases/create_quotation.rs:1
     src/controllers/quotation_controller.rs:5
   ```

### ULID found in specre but no source references

1. User runs `specre trace <ULID>` where the ULID matches a specre but no source files reference it
2. CLI prints:
   ```
   Specre:
     docs/specres/auth/user_can_sign_up_with_email.md

   Source references:
     (none)
   ```

### ULID found in source but no matching specre

1. User runs `specre trace <ULID>` where source files contain `@specre <ULID>` but no specre file has that `id`
2. CLI prints:
   ```
   Specre:
     (not found)

   Source references:
     src/example.rs:3
   ```

### ULID not found anywhere

1. User runs `specre trace <ULID>` where neither specre files nor source files reference the ULID
2. CLI prints:
   ```
   Specre:
     (not found)

   Source references:
     (none)
   ```
3. CLI exits with exit code 1

### File path invocation shows linked specres

1. User runs `specre trace src/config.rs` where the file contains multiple `@specre` markers
2. CLI reads the file directly and extracts all `@specre <ULID>` markers (using the same detection logic as index, ignoring string literals). The `target_extensions` filter does not apply to the explicitly specified file.
3. CLI reads `specre.toml` to determine `specre_dir`, then scans specre files to resolve each ULID to its specre card path
4. CLI prints:
   ```
   File: src/config.rs

   Specres:
     01KHAGG8NQQ7RSNYZ6SWBCYH3N  docs/specres/cli/specre_init_initializes_project_configuration.md
     01KHAKAYN5WPTDVR99D5Q5TMJE  docs/specres/cli/specre_index_generates_project_index.md
   ```

### File path with a ULID that has no matching specre

1. User runs `specre trace src/example.rs` where the file contains `@specre <ULID>` but no specre has that `id`
2. CLI prints the ULID with `(not found)` instead of a path:
   ```
   File: src/example.rs

   Specres:
     01ZZZZZZZZZZZZZZZZZZZZZZZZ  (not found)
   ```

### File path with no markers

1. User runs `specre trace src/utils.rs` where the file contains no `@specre` markers
2. CLI prints:
   ```
   File: src/utils.rs

   Specres:
     (none)
   ```
3. CLI exits with exit code 1

### File does not exist

1. User runs `specre trace nonexistent.rs`
2. CLI exits with error: `Error: file not found: nonexistent.rs`

### specre.toml does not exist

1. User runs `specre trace <ULID>` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

### Paths use forward slashes

1. On all platforms, output paths use forward slashes (`/`), not backslashes
2. Input file paths containing backslashes are normalized to forward slashes before processing (e.g., `src\config.rs` → `src/config.rs`)

### Argument auto-detection

1. If the argument is exactly 26 characters and all uppercase alphanumeric → treated as ULID
2. Otherwise → treated as a file path

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If `specre_dir` does not exist, CLI treats it as no specre found
- If a `source_dirs` entry does not exist, CLI skips it silently
- If the file path does not exist, CLI exits with error: `Error: file not found: <path>`
