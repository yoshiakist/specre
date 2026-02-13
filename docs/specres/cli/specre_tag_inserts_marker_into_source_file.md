---
id: "01KHB48EYB9686YYQMYFYQ5R1Z"
name: "specre_tag_inserts_marker_into_source_file"
status: "stable"
last_verified: "2026-02-13"
---

## Related Files

- `src/commands/tag.rs`
- `src/commands/index.rs` (reuses scanning helpers)
- `src/config.rs`
- `tests/cli_tag.rs` (Test)

## Functional Overview

`specre tag <ULID> <file>` inserts a `@specre <ULID>` marker comment at line 1 of the specified source file, using the appropriate comment syntax for the file's language. If the file already contains a marker for that ULID, the command does nothing and reports that the marker already exists.

## Design Intent

Manually typing `@specre` markers is error-prone — ULIDs are long and comment syntax varies by language. The tag command automates this by inserting a correctly formatted marker, reducing friction for both human developers and AI agents that need to link source files to specres.

Inserting at line 1 is the simplest unambiguous strategy. Developers can reposition the marker manually if a different location is preferred.

## Key Members

- `ulid: String` — the 26-character ULID to insert as a marker (positional argument)
- `file: String` — path to the source file where the marker will be inserted (positional argument)

## Scenarios

### Basic invocation inserts marker at line 1

1. User runs `specre tag 01HZYPMZRK8F9R2DGBGGMM2N8T src/example.rs`
2. CLI detects the file extension `.rs` and selects `//` as the comment prefix
3. CLI prepends `// @specre 01HZYPMZRK8F9R2DGBGGMM2N8T\n` to the beginning of the file
4. CLI prints: `Tagged src/example.rs with 01HZYPMZRK8F9R2DGBGGMM2N8T`

### Comment syntax varies by language

1. For `.rs`, `.js`, `.ts`, `.java`, `.c`, `.cpp`, `.cs`, `.go`, `.swift` files: `// @specre <ULID>`
2. For `.rb`, `.py`, `.sh`, `.yaml`, `.yml`, `.toml` files: `# @specre <ULID>`
3. For `.css`, `.scss` files: `/* @specre <ULID> */`
4. For `.html`, `.xml`, `.svg` files: `<!-- @specre <ULID> -->`
5. For `.sql` files: `-- @specre <ULID>`
6. For unrecognized extensions: `// @specre <ULID>` (default)

### Marker already exists in file

1. User runs `specre tag <ULID> src/example.rs` where the file already contains `// @specre <ULID>`
2. CLI does not modify the file
3. CLI prints: `Marker already exists in src/example.rs`
4. CLI exits with exit code 0

### File does not exist

1. User runs `specre tag <ULID> src/nonexistent.rs`
2. CLI exits with error: `Error: file not found: src/nonexistent.rs`

### Invalid ULID format

1. User runs `specre tag abc123 src/example.rs`
2. CLI exits with error: `Error: invalid ULID format. Expected 26 uppercase alphanumeric characters.`

### ULID does not match any specre (warning only)

1. User runs `specre tag <ULID> src/example.rs` where no specre file has that ULID
2. CLI inserts the marker anyway (the specre may be created later)
3. CLI prints a warning to stderr: `Warning: no specre found with id <ULID>`
4. CLI prints the success message to stdout and exits with exit code 0

### Preserves existing file content

1. User runs `specre tag <ULID> src/example.rs` on a file with existing content
2. The marker line is inserted at line 1, followed by the original content unchanged
3. No trailing newlines are added or removed

### Paths use forward slashes in output

1. On all platforms, output paths use forward slashes (`/`), not backslashes

## Failures / Exceptions

- If the target file does not exist, CLI exits with error: `Error: file not found: <path>`
- If the target path is a directory, CLI exits with error: `Error: '<path>' is a directory, not a file`
- If ULID format is invalid, CLI exits with error before any file operations
- If the filesystem is read-only or permissions are insufficient, CLI exits with the OS-level error message
