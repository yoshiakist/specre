---
id: "01KNK4YMXKJ2B395E8NPQ4DV2Q"
name: "specre_verify_updates_last_verified_for_confirmed_specres"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/verify.rs` (Implementation)
- `src/cli.rs` (CLI argument definitions)
- `src/commands/mod.rs` (Module registration)
- `src/main.rs` (Command dispatch)
- `tests/cli_verify.rs` (Test)

## Functional Overview

`specre verify` bulk-updates `last_verified` to today's date for specre cards that have been confirmed to match their implementation. This is the fast path for resolving false positives after `specre drift` reports specres as drifted. It supports targeting by ULID(s), domain, or source file path.

## Design Intent

After `specre drift` flags specres as potentially out of sync, a human or AI agent reviews the changes and determines that no actual divergence exists. Rather than manually editing each specre card's front-matter, `specre verify` provides a batch operation to update `last_verified` dates efficiently. The `--file` option directly addresses the aggregation file problem: when a change to a file like `mod.rs` flags many specres, all can be verified at once after review.

## Key Members

- `ulids: Vec<String>` -- zero or more ULIDs to verify (positional arguments)
- `--domain <name>` -- verify all specres in a domain
- `--file <source-file-path>` -- verify all specres linked to a source file (via Related Files and `@specre` markers)
- `--json` -- output results in JSON format

## Scenarios

### Verify single specre by ULID

1. A stable specre with `last_verified: "2026-03-01"` exists
2. User runs `specre verify <ULID>`
3. The specre card's `last_verified` is updated to today's date in the YAML front-matter
4. stdout reports the updated specre
5. Exit code is 0

### Verify multiple specres by ULID

1. Two stable specres exist with old `last_verified` dates
2. User runs `specre verify <ULID1> <ULID2>`
3. Both specre cards' `last_verified` fields are updated to today's date
4. stdout reports both updated specres
5. Exit code is 0

### Verify by domain

1. Multiple stable specres exist in the `cli` domain
2. User runs `specre verify --domain cli`
3. All specre cards in the `cli` domain have their `last_verified` updated to today's date
4. stdout reports all updated specres
5. Exit code is 0

### Verify by file (aggregation file problem)

1. A source file `src/commands/mod.rs` contains `@specre` markers for 3 specres
2. User runs `specre verify --file src/commands/mod.rs`
3. All 3 specre cards linked to that file (via `@specre` markers or Related Files references) have their `last_verified` updated
4. stdout reports all updated specres
5. Exit code is 0

### JSON output

1. User runs `specre verify <ULID> --json`
2. Output is valid JSON:
   ```json
   {
     "verified": [
       {
         "id": "01HZYPMZRK...",
         "name": "...",
         "path": "docs/specres/.../foo.md",
         "last_verified": "2026-04-07"
       }
     ],
     "count": 1
   }
   ```
3. Exit code is 0

### Human-readable output (default)

1. User runs `specre verify <ULID>`
2. Output:
   ```
   Verified 1 specre(s):
     01HZYPMZRK...  specre_name  (docs/specres/.../foo.md)
   ```
3. Exit code is 0

### No matching specres found

1. User runs `specre verify <ULID>` with a ULID that does not exist
2. stderr shows: `Error: no specre found with id '<ULID>'`
3. Exit code is 1

### No arguments provided

1. User runs `specre verify` with no ULIDs and no `--domain` or `--file` option
2. stderr shows: `Error: specify at least one ULID, --domain, or --file`
3. Exit code is 1

### ULID not found among multiple

1. User runs `specre verify <ULID1> <ULID_NONEXISTENT>`
2. `<ULID1>` is updated successfully
3. stderr warns about the ULID that was not found
4. Exit code is 1 (partial failure)

### Specre without last_verified field

1. A stable specre has no `last_verified` field in its front-matter
2. User runs `specre verify <ULID>`
3. The `last_verified` field is added with today's date
4. Exit code is 0

### No specre.toml present

1. User runs `specre verify <ULID>` in a directory without `specre.toml`
2. stderr shows: `Error: specre.toml not found. Run 'specre init' first.`
3. Exit code is 1

## Failures / Exceptions

- If `specre.toml` is missing, exit with error directing user to run `specre init`
- If no arguments are provided (no ULIDs, no `--domain`, no `--file`), exit with a clear error message
- If a ULID does not match any specre card, warn on stderr and continue processing remaining ULIDs; exit with code 1 after processing all
- If writing to a specre file fails (permissions, disk full), report the error on stderr and continue with remaining files; exit with code 1
- The `--domain` and `--file` options are mutually exclusive with each other and with positional ULIDs; if combined, exit with an error
