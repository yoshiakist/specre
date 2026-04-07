---
id: "01KNK6TW0DJQ8TE4FF2ZBC6V67"
name: "specre_reopen_transitions_stable_specre_to_in_development"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/reopen.rs` (Implementation)
- `src/cli.rs` (CLI argument definitions)
- `src/commands/mod.rs` (Module registration)
- `src/main.rs` (Command dispatch)
- `tests/cli_reopen.rs` (Test)

## Functional Overview

`specre reopen` transitions a specre card's status from `stable` back to `in-development` when real drift is found between the specification and its implementation. This is the counterpart to `specre verify` — while `verify` handles false positives by updating `last_verified`, `reopen` handles genuine drift by reverting the specre to an editable lifecycle state.

## Design Intent

After `specre drift` flags specres as potentially out of sync, a human or AI agent reviews the changes and determines that the specification is outdated and needs updating. Rather than manually editing each specre card's front-matter, `specre reopen` provides a command to transition the status back to `in-development`, signaling that the specre card requires revision. The `last_verified` field is preserved — it retains historical value as the date of last verification before the drift was detected.

## Key Members

- `ulid: String` — the ULID of the specre to reopen (positional argument, exactly one)
- `--json` — output results in JSON format (global flag)

## Scenarios

### Reopen a stable specre by ULID

1. A specre with `status: "stable"` and `last_verified: "2026-03-01"` exists
2. User runs `specre reopen <ULID>`
3. The specre card's `status` is changed to `"in-development"` in the YAML front-matter
4. The `last_verified` field is preserved unchanged
5. stdout reports the reopened specre
6. Exit code is 0

### JSON output

1. User runs `specre reopen <ULID> --json`
2. Output is valid JSON:
   ```json
   {
     "reopened": {
       "id": "01HZYPMZRK...",
       "name": "...",
       "path": "docs/specres/.../foo.md",
       "previous_status": "stable",
       "new_status": "in-development",
       "last_verified": "2026-03-01"
     }
   }
   ```
3. Exit code is 0

### Human-readable output (default)

1. User runs `specre reopen <ULID>`
2. Output:
   ```
   Reopened: 01HZYPMZRK...  specre_name  (docs/specres/.../foo.md)
     stable -> in-development
   ```
3. Exit code is 0

### Reopen a non-stable specre returns an error

1. A specre with `status: "draft"` exists
2. User runs `specre reopen <ULID>`
3. stderr shows: `Error: specre '<ULID>' has status 'draft', only 'stable' specres can be reopened`
4. Exit code is 1

### ULID not found

1. User runs `specre reopen <ULID>` with a ULID that does not exist
2. stderr shows: `Error: no specre found with id '<ULID>'`
3. Exit code is 1

### No specre.toml present

1. User runs `specre reopen <ULID>` in a directory without `specre.toml`
2. stderr shows: `Error: specre.toml not found. Run 'specre init' first.`
3. Exit code is 1

### last_verified is preserved after reopen

1. A stable specre has `last_verified: "2026-03-01"`
2. User runs `specre reopen <ULID>`
3. After the operation, `last_verified` remains `"2026-03-01"` in the front-matter
4. Only `status` is changed

## Failures / Exceptions

- If `specre.toml` is missing, exit with error directing user to run `specre init`
- If the ULID does not match any specre card, exit with a clear error message and code 1
- If the specre's status is not `stable`, exit with an error explaining that only stable specres can be reopened
- If writing to the specre file fails (permissions, disk full), report the error on stderr and exit with code 1
