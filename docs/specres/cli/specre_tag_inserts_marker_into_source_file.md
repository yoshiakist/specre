---
id: "01KHB48EYB9686YYQMYFYQ5R1Z"
name: "specre_tag_inserts_marker_into_source_file"
status: "stable"
last_verified: "2026-04-07"
---

## Related Files

- `src/commands/tag.rs`
- `src/scanner.rs` (reuses scanning helpers)
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

The command supports the following file extensions, organized by domain:

**`// @specre <ULID>` — C-family, JVM, modern languages, shaders:**

- General: `.rs`, `.js`, `.ts`, `.jsx`, `.tsx`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`, `.cs`, `.go`, `.swift`, `.kt`, `.kts`, `.scala`, `.groovy`, `.gradle`, `.dart`, `.php`, `.zig`
- Data / schema: `.proto`, `.prisma`, `.jsonc`
- Unity shaders: `.shader`, `.hlsl`, `.cginc`, `.compute`
- Unreal Engine shaders: `.usf`, `.ush`
- Godot shaders: `.gdshader`
- Graphics shaders: `.glsl`, `.vert`, `.frag`, `.geom`, `.wgsl`

**`# @specre <ULID>` — scripting, config, data:**

- General: `.rb`, `.py`, `.sh`, `.bash`, `.zsh`, `.yaml`, `.yml`, `.toml`
- Godot: `.gd` (GDScript)
- Additional languages: `.pl`, `.pm` (Perl), `.r`, `.R` (R), `.ex`, `.exs` (Elixir), `.nim`
- Shell: `.ps1`, `.psm1`, `.psd1` (PowerShell), `.fish`, `.nix`
- Infrastructure: `.tf`, `.tfvars`, `.hcl` (Terraform / HCL), `.cmake`, `.mk`
- Config: `.env`, `.conf`, `.properties`
- Query / schema: `.graphql`, `.gql`
- Unity serialized (YAML): `.unity`, `.prefab`, `.asset`, `.mat`, `.meta`

**`/* @specre <ULID> */` — stylesheets:**

- `.css`, `.scss`, `.sass`, `.less`
- Unity: `.uss` (UI StyleSheet)

**`<!-- @specre <ULID> -->` — markup, SFC:**

- General: `.html`, `.htm`, `.xml`, `.svg`
- Frontend frameworks: `.vue`, `.svelte`, `.astro`
- Unity: `.uxml` (UI XML)
- XML transforms: `.xsl`, `.xslt`

**`-- @specre <ULID>` — SQL, Lua, Haskell:**

- `.sql`, `.lua`, `.hs`

**`; @specre <ULID>` — Godot data, INI:**

- Godot: `.tscn` (scene), `.tres` (resource), `.godot` (project)
- Config: `.ini`, `.cfg`

**`{# @specre <ULID> #}` — Jinja / Twig templates:**

- `.j2`, `.jinja`, `.jinja2` (Jinja2 — Django, Flask, Ansible)
- `.twig` (Twig — Symfony)

**`<%# @specre <ULID> %>` — embedded templates:**

- `.erb` (ERB — Rails)
- `.ejs` (EJS — Express)

**`{{!-- @specre <ULID> --}}` — Handlebars:**

- `.hbs`, `.handlebars`

**`@* @specre <ULID> *@` — Razor:**

- `.cshtml` (ASP.NET Razor)

**`//- @specre <ULID>` — Pug / Jade:**

- `.pug`, `.jade`

**`-# @specre <ULID>` — Haml:**

- `.haml`

**Unsupported extensions:** The command refuses to insert a marker and exits with an error. No fallback.

### Unsupported file extension

1. User runs `specre tag <ULID> data/config.xyz` where `.xyz` is not in the supported list
2. CLI does not modify the file
3. CLI exits with error: `Error: unsupported file extension '.xyz' — comment syntax is unknown`

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
- If the file extension is not in the supported list, CLI exits with error: `Error: unsupported file extension '.<ext>' — comment syntax is unknown`
- If the filesystem is read-only or permissions are insufficient, CLI exits with an error: `Error: Failed to access '<path>': <OS error message>`
