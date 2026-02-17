---
id: "01KHB48EES4FR5TFV6ZP2W3MGT"
name: "specre_orphans_detects_unlinked_specres_and_markers"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/commands/orphans.rs`
- `src/scanner.rs` (reuses scanning helpers)
- `src/config.rs`
- `tests/cli_orphans.rs` (Test)

## Functional Overview

`specre orphans` detects specres that have no `@specre` markers in any source file (orphan specres) and `@specre` markers in source files that do not match any specre's `id` (dangling markers). It reports both categories so developers can restore traceability or clean up stale references. Source file scanning respects the `target_extensions` setting in `specre.toml` when configured.

## Design Intent

Bidirectional traceability only works when both sides of the link exist. Over time, specres can be created without corresponding markers, or markers can survive after a specre is deleted. The orphans command surfaces these gaps so they can be addressed before they compound into untraceable code.

The primary consumers are CI pipelines (where a non-zero exit code can block merges) and human developers running periodic hygiene checks. Output is structured for both human readability and scripted parsing.

## Key Members

- Orphan specre: a specre file whose `id` does not appear as an `@specre` marker in any scanned source file (filtered by `target_extensions` if set)
- Dangling marker: a `@specre <ULID>` marker in a scanned source file where no specre file has that ULID as its `id`

## Scenarios

### No orphans or dangling markers

1. User runs `specre orphans` in a project where every specre has at least one source marker and every marker matches a specre
2. CLI prints:
   ```
   No orphans or dangling markers found.
   ```
3. CLI exits with exit code 0

### Orphan specres detected

1. User runs `specre orphans` in a project where two specres have no `@specre` markers in source
2. CLI prints:
   ```
   Orphan specres (no source markers):
     docs/specres/auth/user_can_sign_up_with_email.md
     docs/specres/cart/user_can_add_item_to_cart.md
   ```
3. CLI exits with exit code 1

### Dangling markers detected

1. User runs `specre orphans` where source files contain `@specre` markers that do not match any specre `id`
2. CLI prints:
   ```
   Dangling markers (no matching specre):
     src/example.rs:3  01HZYPMZRK8F9R2DGBGGMM2N8T
     src/other.rs:10   01HZYQ4N7XW3A8B5C6D9E0F1G2
   ```
3. CLI exits with exit code 1

### Both orphans and dangling markers

1. User runs `specre orphans` where both orphan specres and dangling markers exist
2. CLI prints both sections:
   ```
   Orphan specres (no source markers):
     docs/specres/auth/user_can_sign_up_with_email.md

   Dangling markers (no matching specre):
     src/example.rs:3  01HZYPMZRK8F9R2DGBGGMM2N8T
   ```
3. CLI exits with exit code 1

### Deprecated specres are excluded from orphan detection

1. User runs `specre orphans` where a specre has `status: "deprecated"` and no source markers
2. The deprecated specre is not reported as an orphan (deprecated specres are expected to have no markers)

### Paths use forward slashes

1. On all platforms, output paths use forward slashes (`/`), not backslashes

### specre.toml does not exist

1. User runs `specre orphans` in a directory without `specre.toml`
2. CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`

### Empty project

1. User runs `specre orphans` with a valid `specre.toml` but no specre files and no source files
2. CLI prints:
   ```
   No orphans or dangling markers found.
   ```
3. CLI exits with exit code 0

## Failures / Exceptions

- If `specre.toml` is missing, CLI exits with error: `Error: specre.toml not found. Run 'specre init' first.`
- If `specre_dir` does not exist, CLI treats it as no specres found
- If a `source_dirs` entry does not exist, CLI skips it silently
- Specre files with malformed front-matter are skipped with a warning to stderr
- If a specre or source file cannot be read (IO error), CLI prints a warning to stderr (`Warning: failed to read '<path>': <reason>`) and skips that file
