---
id: "01KHAGG8NQQ7RSNYZ6SWBCYH3N"
name: "specre_init_initializes_project_configuration"
status: "draft"
---

## Related Files

- `src/commands/init.rs`
- `src/config.rs`
- `tests/cli_init.rs` (Test)

## Functional Overview

`specre init` initializes specre in a project by creating the specre directory and a `specre.toml` configuration file. It is the first command a user runs when adopting specre.

## Design Intent

The init command establishes the minimal scaffolding needed for all other specre commands to work with sensible defaults. By writing a `specre.toml` in the project root, it removes the need to pass directory paths to every subsequent command.

The primary consumer of this command is expected to be a human developer setting up specre for the first time. Unlike `specre new`, this command is typically run once per project.

## Key Members

- `specre_dir: String` — directory where specre cards are stored (default: `docs/specres`)
- `source_dirs: Vec<String>` — directories to scan for `@specre` markers (default: `["src"]`)

## Scenarios

### Basic invocation in a project root

1. User runs `specre init` in their project root
2. CLI creates `docs/specres/` directory
3. CLI writes `specre.toml` with default values:
   ```toml
   specre_dir = "docs/specres"
   source_dirs = ["src"]
   ```
4. CLI prints a summary to stdout:
   ```
   Created docs/specres/
   Created specre.toml
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

### specre.toml already exists

1. User runs `specre init` in a project that already has `specre.toml`
2. CLI exits with an error: `Error: specre.toml already exists. This project is already initialized.`
3. No files or directories are modified

### specre directory already exists

1. User runs `specre init` in a project where `docs/specres/` already exists but `specre.toml` does not
2. CLI keeps the existing directory as-is (does not overwrite or delete contents)
3. CLI writes `specre.toml`
4. CLI prints the summary, noting the directory already existed

## Failures / Exceptions

- If the filesystem is read-only or permissions are insufficient, CLI exits with the OS-level error message
