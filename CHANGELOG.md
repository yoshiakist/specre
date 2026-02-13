# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-02-13

### Added

- `specre init` — Initialize a project with `specre.toml` and a specre directory
- `specre new <dir> --name <name>` — Scaffold a new specre card with auto-generated ULID
- `specre index` — Scan specre directory and source tree; generate `index.json` and per-domain `INDEX.md`
- `specre status` — Report specre counts by status and flag stale `last_verified` dates
- Bidirectional traceability via `@specre <ULID>` source markers
- specre card format with YAML front-matter (`id`, `name`, `status`, `last_verified`)

[0.1.0]: https://github.com/yoshiakist/specre/releases/tag/v0.1.0
