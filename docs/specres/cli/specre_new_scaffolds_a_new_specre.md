---
id: "01JMBJK7QRVX3N4P5G6H8W9Y0Z"
name: "specre_new_scaffolds_a_new_specre"
status: "stable"
last_verified: "2026-02-16"
---

## Related Files

- `src/commands/new.rs`
- `src/ulid.rs`
- `src/template.rs`
- `tests/commands/new_test.rs` (Test)

## Functional Overview

`specre new` creates a new specre Markdown file from a template, auto-generating a ULID for the `id` field. It accepts a target directory and a name, then writes a ready-to-edit specre to disk.

## Design Intent

The new command lowers the barrier to creating specres. By auto-generating the ULID and populating boilerplate sections, developers can focus on writing scenarios rather than remembering the front-matter schema. The command should complete in under a second and require no network access.

The primary consumer of this command is expected to be AI coding agents, which generate specres as part of their development workflow. The CLI interface is designed to be equally usable by humans and agents — no interactive prompts, deterministic output, machine-parseable stdout.

## Key Members

- `target_dir: String` — the directory where the new specre will be created (e.g., `docs/specres/auth/`)
- `name: String` — specre name describing the behavior (e.g., `user_can_sign_up_with_email`)

## Scenarios

### Basic invocation with a name

1. User runs `specre new docs/specres/auth --name user_can_sign_up_with_email`
2. CLI generates a new ULID
3. CLI writes `docs/specres/auth/user_can_sign_up_with_email.md` with:
   - `id`: the generated ULID
   - `name`: `"user_can_sign_up_with_email"`
   - `status`: `"draft"`
   - All recommended sections as empty placeholders
4. CLI prints the path of the created file to stdout

### Invocation without a name

1. User runs `specre new docs/specres/auth`
2. CLI generates a ULID
3. CLI writes `docs/specres/auth/untitled.md` with `name: "untitled"`
4. CLI prints the path to stdout
5. The user renames the file and updates the `name` field manually

### Target directory does not exist

1. User runs `specre new docs/specres/new_domain --name some_behavior`
2. CLI creates the directory `docs/specres/new_domain/` recursively
3. File is created as `docs/specres/new_domain/some_behavior.md`

## Generated Template

The output file contains the following content:

```markdown
---
id: "<generated ULID>"
name: "<provided or 'untitled'>"
status: "draft"
---

## Related Files

-

## Functional Overview



## Scenarios

###

1.
```

The template includes only the required sections with minimal placeholders. Optional sections (Design Intent, Key Members, Failures / Exceptions) are omitted to avoid clutter — users add them when needed.

## Failures / Exceptions

- If `target_dir` points to a path where a file (not a directory) already exists, CLI exits with an error: `Error: '<path>' is a file, not a directory`
- If a file with the exact same name already exists, CLI exits with an error: `Error: '<path>' already exists`
- If the filesystem is read-only or permissions are insufficient (e.g., a parent path component is a file, not a directory), CLI exits with an error: `Error: Failed to access '<path>': <OS error message>`
