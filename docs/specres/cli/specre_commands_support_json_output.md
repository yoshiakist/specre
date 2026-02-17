---
id: "01KHG0A2V4YXE918WMJCY7WFE8"
name: "specre_commands_support_json_output"
status: "stable"
last_verified: "2026-02-17"
---

## Related Files

- `src/cli.rs`
- `src/main.rs`
- `src/commands/mod.rs`
- `src/commands/status.rs`
- `src/commands/trace.rs`
- `src/commands/orphans.rs`
- `src/commands/coverage.rs`
- `src/commands/init.rs`
- `src/commands/new.rs`
- `src/commands/tag.rs`
- `src/commands/index.rs`
- `tests/cli_json_output.rs` (Test)

## Functional Overview

All specre CLI commands support a `--json` flag that switches output from human-readable text to structured JSON on stdout. This enables AI agents and scripts to consume specre output programmatically without parsing free-form text. Commands that already output JSON (`search`, `health-check`) accept the flag but their output is unchanged. The flag is defined as a global option on the top-level CLI struct and propagated to each subcommand handler.

## Design Intent

The v0.3 roadmap targets agent integration. Agents need machine-readable output to make decisions. While `search` and `health-check` were designed JSON-first, the remaining commands (`status`, `trace`, `orphans`, `coverage`, `init`, `new`, `tag`, `index`) output human-readable text that is fragile to parse. Adding `--json` across all commands provides a uniform, predictable interface for any consumer — whether an MCP server, a CI script, or a coding agent.

## Key Members

- `Cli.json: bool` — global flag (`--json`) on the top-level clap struct, defaults to `false`
- Each command handler receives the `json` flag and branches output accordingly
- All JSON output uses `serde_json::to_string_pretty` for consistency with existing commands

## Scenarios

### status --json outputs structured JSON

1. Project has specre cards in various statuses, some stale
2. User runs `specre status --json`
3. CLI outputs JSON to stdout:
   ```json
   {
     "summary": {
       "draft": 2,
       "in_development": 1,
       "stable": 3,
       "deprecated": 0,
       "total": 6
     },
     "stale": [
       {
         "name": "user_can_reset_password",
         "path": "docs/specres/auth/user_can_reset_password.md",
         "reason": "45 days"
       }
     ]
   }
   ```
4. CLI exits with exit code 0

### status without --json is unchanged

1. User runs `specre status` (no `--json` flag)
2. CLI outputs the same human-readable text format as before
3. Backward compatibility is preserved

### trace --json outputs structured JSON (ULID lookup)

1. User runs `specre trace --json 01HZYPMZRK8F9R2DGBGGMM2N8T`
2. CLI outputs JSON to stdout:
   ```json
   {
     "specre": "docs/specres/cli/user_can_sign_up_with_email.md",
     "source_refs": [
       { "file": "src/auth/signup.rs", "line": 1 }
     ]
   }
   ```
3. CLI exits with exit code 0

### trace --json outputs structured JSON (file lookup)

1. User runs `specre trace --json src/auth/signup.rs`
2. CLI outputs JSON to stdout:
   ```json
   {
     "file": "src/auth/signup.rs",
     "specres": [
       { "id": "01HZYPMZRK8F9R2DGBGGMM2N8T", "path": "docs/specres/cli/user_can_sign_up_with_email.md" }
     ]
   }
   ```
3. CLI exits with exit code 0

### trace --json with unknown ULID

1. User runs `specre trace --json 01ZZZZZZZZZZZZZZZZZZZZZZZZ`
2. CLI outputs JSON with `"specre": null` and empty `source_refs`
3. CLI exits with exit code 1 (same error semantics as text mode)

### orphans --json outputs structured JSON

1. User runs `specre orphans --json`
2. CLI outputs JSON to stdout:
   ```json
   {
     "orphan_specres": [
       "docs/specres/auth/user_can_reset_password.md"
     ],
     "dangling_markers": [
       { "file": "src/old_module.rs", "line": 5, "id": "01ZZZZZZZZZZZZZZZZZZZZZZZZ" }
     ]
   }
   ```
3. CLI exits with exit code 1 (same error semantics: non-zero when orphans exist)

### orphans --json with no orphans

1. User runs `specre orphans --json` in a clean project
2. CLI outputs JSON:
   ```json
   {
     "orphan_specres": [],
     "dangling_markers": []
   }
   ```
3. CLI exits with exit code 0

### coverage --json outputs structured JSON

1. User runs `specre coverage --json`
2. CLI outputs JSON to stdout:
   ```json
   {
     "total": 10,
     "tagged": 8,
     "coverage": 0.8,
     "uncovered": [
       "src/utils/helper.rs",
       "src/utils/format.rs"
     ]
   }
   ```
3. CLI exits with exit code 0

### coverage --json with --ext filter

1. User runs `specre coverage --json --ext rs`
2. CLI applies the extension filter and outputs the same JSON structure, filtered to `.rs` files only

### init --json outputs structured JSON

1. User runs `specre init --json`
2. CLI outputs JSON to stdout:
   ```json
   {
     "specre_dir": "docs/specres",
     "config_file": "specre.toml"
   }
   ```
3. CLI exits with exit code 0

### new --json outputs structured JSON

1. User runs `specre new docs/specres/cli --name user_can_do_thing --json`
2. CLI outputs JSON to stdout:
   ```json
   {
     "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
     "path": "docs/specres/cli/user_can_do_thing.md"
   }
   ```
3. CLI exits with exit code 0

### tag --json outputs structured JSON

1. User runs `specre tag --json 01HZYPMZRK8F9R2DGBGGMM2N8T src/main.rs`
2. CLI outputs JSON to stdout:
   ```json
   {
     "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
     "file": "src/main.rs",
     "line": 1
   }
   ```
3. CLI exits with exit code 0

### index --json outputs structured JSON

1. User runs `specre index --json`
2. CLI outputs JSON to stdout:
   ```json
   {
     "index_file": "index.json",
     "specre_count": 12,
     "source_ref_count": 8,
     "index_md_files": [
       "docs/specres/cli/INDEX.md",
       "docs/specres/auth/INDEX.md"
     ]
   }
   ```
3. CLI exits with exit code 0

### search --json is accepted but output is unchanged

1. `search` already outputs JSON by default
2. User runs `specre search --json "password"`
3. Output is identical to `specre search "password"`
4. The flag is accepted without error

### health-check --json is accepted but output is unchanged

1. `health-check` already outputs JSON by default
2. User runs `specre health-check --json`
3. Output is identical to `specre health-check`
4. The flag is accepted without error

### --json flag placement is flexible

1. The `--json` flag is a global option on the top-level `Cli` struct
2. It can appear before or after the subcommand: `specre --json status` and `specre status --json` both work
3. This is the standard clap behavior for global options propagated to subcommands

### Error output goes to stderr regardless of --json

1. User runs `specre status --json` without `specre.toml`
2. CLI outputs `Error: specre.toml not found. Run 'specre init' first.` to stderr (not JSON)
3. CLI exits with exit code 1
4. Error messages are never wrapped in JSON — stderr is always plain text

### specre.toml does not exist

1. User runs any command with `--json` in a directory without `specre.toml`
2. CLI outputs the same error to stderr as without `--json`
3. CLI exits with exit code 1

## Failures / Exceptions

- Error messages always go to stderr as plain text, regardless of `--json`. JSON is only for successful structured output on stdout.
- If serialization fails (should not happen in practice), CLI outputs `Error: Failed to serialize: <details>` to stderr and exits with exit code 1.
- Commands that already output JSON (`search`, `health-check`) are unaffected by the flag — their output format does not change.
