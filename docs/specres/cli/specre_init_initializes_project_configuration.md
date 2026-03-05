---
id: "01KHAGG8NQQ7RSNYZ6SWBCYH3N"
name: "specre_init_initializes_project_configuration"
status: "stable"
last_verified: "2026-03-05"
---

## Related Files

- `src/commands/init.rs`
- `src/config.rs`
- `tests/cli_init.rs` (Test)

## Functional Overview

`specre init` initializes specre in a project by creating the specre directory, a `specre.toml` configuration file, and a `glossary.toml` sample vocabulary file. It is the first command a user runs when adopting specre.

## Design Intent

The init command establishes the minimal scaffolding needed for all other specre commands to work with sensible defaults. By writing a `specre.toml` in the project root, it removes the need to pass directory paths to every subsequent command.

The `glossary.toml` file is a sample vocabulary list that `specre search` uses to provide refinement suggestions when queries return zero or too many results. It is generated with generic placeholder terms that the team is expected to customize with project-specific domain vocabulary. The glossary is optional — deleting it has no effect on core specre functionality.

The primary consumer of this command is expected to be a human developer setting up specre for the first time. Unlike `specre new`, this command is typically run once per project.

## Key Members

- `specre_dir: String` — directory where specre cards are stored (default: `docs/specres`)
- `source_dirs: Vec<String>` — directories to scan for `@specre` markers (default: `["src"]`)
- `ext: Option<Vec<String>>` — target file extensions for source scanning (e.g., `["rs", "ts"]`). When omitted, all files are scanned.
- `health_check` — optional thresholds for `specre health-check` (coverage, orphans, index_age_hours)
- `exclude_patterns: Option<Vec<String>>` — glob/substring patterns to exclude from source scanning
- `language: Option<String>` — language code for specre card generation (default: `en`)

## Scenarios

### Basic invocation in a project root

1. User runs `specre init` in their project root
2. CLI creates `docs/specres/` directory
3. CLI writes `specre.toml` with default values and commented-out optional settings:
   ```toml
   specre_dir = "docs/specres"
   source_dirs = ["src"]

   # target_extensions = ["rb", "js", "ts"]
   # exclude_patterns = [".stories.tsx", "**/dist"]
   # language = "ja"

   # [health_check]
   # coverage = 0.30
   # orphans = 10
   # index_age_hours = 48
   ```
4. CLI writes `glossary.toml` with sample terms:
   ```toml
   # specre glossary — Project vocabulary for search suggestions
   #
   # Add terms that describe your project's domains, operations, and concepts.
   # `specre search` uses this glossary to suggest query refinements when
   # results are empty or too broad.
   #
   # Customize this list with your project's domain-specific vocabulary:
   #   - Domain nouns: "user", "order", "payment", "notification"
   #   - Operations: "create", "update", "delete", "approve", "reject"
   #   - Technical concepts: "authentication", "authorization", "validation"

   terms = [
     "user",
     "system",
     "create",
     "update",
     "delete",
     "find",
     "list",
   ]
   ```
5. CLI prints a summary to stdout:
   ```
   Created docs/specres/
   Created specre.toml
   Created glossary.toml
   ```

### Custom specre directory

1. User runs `specre init --specre-dir specs/specres`
2. CLI creates `specs/specres/` directory
3. CLI writes `specre.toml` with `specre_dir = "specs/specres"`
4. CLI prints the summary to stdout

### Custom source directories

1. User runs `specre init --source-dirs src,lib`
2. CLI creates `docs/specres/` directory
3. CLI writes `specre.toml` with `source_dirs = ["src", "lib"]`
4. CLI prints the summary to stdout

### Custom target extensions

1. User runs `specre init --ext rs,ts`
2. CLI creates `docs/specres/` directory
3. CLI writes `specre.toml` with `target_extensions = ["rs", "ts"]` in addition to `specre_dir` and `source_dirs`; the `# target_extensions` comment line is omitted since the field is already active
4. CLI prints the summary to stdout

### specre.toml already exists

1. User runs `specre init` in a project that already has `specre.toml`
2. CLI exits with an error: `Error: specre.toml already exists. This project is already initialized.`
3. No files or directories are modified

### specre directory already exists

1. User runs `specre init` in a project where `docs/specres/` already exists but `specre.toml` does not
2. CLI keeps the existing directory as-is (does not overwrite or delete contents)
3. CLI writes `specre.toml` and `glossary.toml`
4. CLI prints the summary, noting the directory already existed

### glossary.toml already exists

1. User runs `specre init` in a project where `glossary.toml` already exists but `specre.toml` does not
2. CLI keeps the existing `glossary.toml` as-is (does not overwrite)
3. CLI writes `specre.toml` and creates the specre directory
4. CLI prints the summary, noting that the glossary already existed:
   ```
   Created docs/specres/
   Created specre.toml
   Exists  glossary.toml
   ```

### Special characters in arguments

1. User runs `specre init --specre-dir 'path with "quotes"'`
2. CLI writes `specre.toml` with properly escaped TOML values (e.g., `specre_dir = "path with \"quotes\""`)
3. The generated `specre.toml` is always valid TOML regardless of user input

## Implementation Notes

- Config generation uses `toml::to_string()` for serialization to prevent TOML injection via special characters in user-supplied values (quotes, backslashes, etc.)
- After the serialized active config, commented-out examples of optional settings are appended as a static string; a comment for a field is omitted when the user already supplied that field via CLI arguments (e.g., `--ext` suppresses the `# target_extensions` comment, `--language` suppresses the `# language` comment)
- `exclude_patterns` and `[health_check]` comments are always appended because there are no CLI arguments to set them directly via `specre init`

## Failures / Exceptions

- If the filesystem is read-only or permissions are insufficient (e.g., a parent path component is a file, not a directory), CLI exits with an error: `Error: Failed to access '<path>': <OS error message>`
