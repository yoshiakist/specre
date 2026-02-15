# Roadmap

## v0.1 — Core CLI ✅

- [x] `specre init` — Initialize specre in a project, creating the specre directory and configuration file
- [x] `specre new` — Scaffold a new specre from a template, auto-generating a ULID for the `id` field
- [x] `specre index` — Scan specre directory and source tree; generate `index.json` and per-domain `_INDEX.md`
- [x] `specre status` — Report specre counts by status and flag stale `last_verified` dates

## v0.2 — Traceability ✅

- [x] `specre trace <ULID>` — Given a ULID, show the specre file and all source files referencing it (or vice versa)
- [x] `specre orphans` — Detect specres with no `@specre` markers in source, or markers with no matching specre
- [x] `specre tag <ULID> <file>` — Insert a `@specre` marker into a source file at the appropriate location

## v0.3 — Agent Integration

Enable AI agents to utilize specre as a first-class tool.

- [x] `specre coverage` — Report the percentage of source files covered by specre tags
- [x] `specre health-check` — Comprehensive health check to determine whether specre cards adequately describe the project's overall behavior
- [x] `specre search <query>` — Full-text + status/domain filtering across all specres, with JSON output for agent consumption
- [x] Agent-friendly output formats across all commands (`--json`)
- [ ] MCP server — Expose specre capabilities as Resources, Tools, and Prompts via the [Model Context Protocol](https://modelcontextprotocol.io/), enabling integration with Claude Code, Cursor, VS Code Copilot, and other MCP-compatible AI tools

### Coverage command design

Coverage measures how much of the source tree is linked to specre cards via `@specre` tags.

- **Denominator:** Total number of files in the configured `source_dir`
- **Numerator:** Number of files in `source_dir` that contain at least one `@specre` tag
- Supports filtering by target file extensions (e.g., `--ext rs,ts` to only count `.rs` and `.ts` files)

### Health-check command design

Health-check is a single entry point for coding agents to verify that the specre ecosystem is trustworthy before starting a task. It is designed as the first command an agent runs at the start of a session, or as an MCP server query.

By aggregating coverage, orphan count, and index freshness into one response, it enables coding agents to unambiguously determine whether specre cards and specre commands can be relied upon — without the agent needing to interpret multiple commands individually.

Returns structured JSON:

```json
{
  "healthy": true,
  "coverage": 0.93,
  "orphans": 2,
  "index_age_hours": 3.2,
  "thresholds": { "coverage": 0.90, "orphans": 5, "index_age_hours": 24 }
}
```

- `healthy` is `true` when all metrics are within their thresholds.
- `thresholds` are configurable via `specre.toml`. The values above are defaults.

### MCP server design

The MCP server wraps existing CLI logic as a thin layer, rather than reimplementing functionality.

| MCP primitive | What it exposes |
|---------------|-----------------|
| **Resources** | specre cards as `specre:///<ULID>` URIs. Agents can read individual specre cards on demand. |
| **Tools** | `search`, `trace`, `orphans`, `status`, `index` — the same operations available via CLI, returning structured JSON. |
| **Prompts** | SDD workflow templates (e.g., "implement a behavior from a specre card") and QA-oriented prompts (`review-qa`, `summarize-diff`) for consistent agent-driven development. |

Transport: stdio (primary), with the option to add SSE/HTTP for remote use cases in the future.

### MCP Prompts for QA

The MCP server includes prompts designed for QA engineers, enabling them to leverage AI for specification-level quality assurance without reading implementation code.

| Prompt | Purpose |
|--------|---------|
| `review-qa` | Analyze a specre card and suggest missing edge cases, boundary conditions, and Failures / Exceptions that may have been overlooked. |
| `summarize-diff` | Semantically summarize changes between the previous stable version and the current in-development version of a specre, and suggest regression test scope. |

These prompts keep specre engine-independent — the AI reasoning is performed by whichever LLM the agent is connected to, not by specre itself.

## v0.4 — Drift Detection

- [ ] `specre drift` — Compare `last_verified` dates against git history of related files; flag specres where source has changed since last verification
- [ ] `specre ci` — Exit with non-zero status if drift or orphans are detected (for CI integration)
- [ ] GitHub Actions workflow template

## v0.5 — QA Support

Deterministic CLI commands that help QA engineers work directly with specre cards — no LLM required.

- [ ] `specre impact <ULID>` — Transitive dependency and impact analysis. Traverse cross-references in Referenced Specifications sections and `@specre` markers to build a dependency graph, showing which specres and source files are affected by a change.
- [ ] `specre diff [specre-path]` — Show how a specre card has changed since its last `stable` state, using git history. Complements `specre drift` (which detects *whether* something changed) by showing *what* changed.
- [ ] `specre export [--format <fmt>]` — Convert Scenarios sections into structured test case formats (Markdown checklist, CSV) for import into test management tools. Eliminates the manual transcription of specifications into test cases.

## Future Considerations

- Plugin system for custom front-matter fields (`type`, `tags`, etc.) with optional validation — enabling searches like `specre search --tag "quotation edit"` across project-defined vocabularies
- Mermaid diagram generation from cross-references between specres
- Dependency graph visualization
- Multi-language support for specre content (i18n metadata)
