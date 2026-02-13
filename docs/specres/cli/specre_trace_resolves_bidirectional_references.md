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

`specre trace <ULID>` performs a bidirectional traceability lookup. Given a ULID, it shows the specre file that owns it and all source files containing `@specre` markers referencing it.

## Design Intent

Traceability is a core promise of specre. The trace command makes it trivial to answer "where is this behavior specified?" and "where is this behavior implemented?" in a single invocation. The output is designed for both human developers navigating a codebase and AI agents resolving references during code generation.

## Key Members

- `ulid: String` — the 26-character ULID to look up (positional argument)

## Scenarios

### Basic invocation shows specre and source references

1. User runs `specre trace 01HZYPMZRK8F9R2DGBGGMM2N8T` in a project with `specre.toml`
2. CLI reads `specre.toml` to determine `specre_dir` and `source_dirs`
3. CLI scans specre files for a matching `id` in front-matter
4. CLI scans source files for `@specre 01HZYPMZRK8F9R2DGBGGMM2N8T` markers
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

### Invalid ULID format

1. User runs `specre trace abc123` (not a valid 26-character uppercase alphanumeric ULID)
2. CLI exits with error: `Error: invalid ULID format. Expected 26 uppercase alphanumeric characters.`

### specre.toml does not exist

1. User runs `specre trace <ULID>` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

### Paths use forward slashes

1. On all platforms, output paths use forward slashes (`/`), not backslashes

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If `specre_dir` does not exist, CLI treats it as no specre found
- If a `source_dirs` entry does not exist, CLI skips it silently
- If ULID format is invalid, CLI exits with error before scanning
