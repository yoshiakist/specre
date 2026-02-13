# specre

**Atomic, living specification cards for AI-agent-friendly development.**

specre is a minimal specification format and toolkit for Spec-Driven Development (SDD). Each specre is a single Markdown file describing exactly one behavior, with machine-readable front-matter for lifecycle tracking and agent navigation.

## Philosophy

Most SDD tools (GitHub Spec Kit, Amazon Kiro, BMAD) treat specifications as large, monolithic documents that feed into linear pipelines. This works for greenfield features but becomes a sledgehammer for everyday development — small fixes, incremental changes, and brownfield evolution.

specre takes the opposite approach:

- **One file, one behavior.** Keep the unit of specification atomic. An AI agent should never need to parse an entire feature to understand a single behavior.
- **Context-window aware.** LLM context is a finite resource. A specre is sized to fit comfortably alongside its test file and implementation in a single agent session.
- **Living documents, not session artifacts.** Each specre carries its own lifecycle status and verification date. Specifications survive beyond the task that created them.
- **Process-agnostic data layer.** specre defines the *format* of specifications, not the *workflow*. Use it with TDD, BDD, or any development process. Pair it with your own workflow commands or CI pipelines.
- **Engine and tool independent.** Plain Markdown with YAML front-matter. No IDE lock-in, no proprietary CLI required.

## Quick Start

### As a Git Submodule

```bash
git submodule add git@github.com:yoshiakist/specre.git specre
```

### Writing Your First specre

Create a Markdown file under your project's specres directory. Organize subdirectories however you like — by domain, module, feature area, or any scheme that fits your project:

```
docs/specres/
  auth/
    001_user_can_sign_up_with_email.md
    002_user_can_reset_password.md
  cart/
    001_items_can_be_added_to_cart.md
    002_cart_total_reflects_quantity_changes.md
```

## specre Card Format

Every specre follows this structure:

```markdown
---
id: "01HZYPMZRK8F9R2DGBGGMM2N8T"
name: "001_user_can_sign_up_with_email"
type: "behavior"
status: "draft"
last_verified: "2026-02-12"
---

## Related Files

- `src/auth/signup_controller.rb`
- `src/auth/email_validator.rb`
- `spec/auth/signup_controller_spec.rb` (Test)

## Functional Overview

Users can create an account by providing a valid email address and password.

## Design Intent

Email signup is the primary onboarding path. We validate email format
client-side and uniqueness server-side to provide fast feedback.

## Key Members

- `email: String` — the user's email address, validated against RFC 5322
- `password_hash: String` — bcrypt hash, never stored in plaintext

## Scenarios

### Successful signup

1. User submits a valid email and a password of 8+ characters
2. System creates the account and emits `account_created` signal
3. User is redirected to the welcome screen

### Duplicate email

1. User submits an email that already exists
2. System displays an error: "This email is already registered"
3. No account is created

### Invalid email format

1. User submits a malformed email (e.g., "foo@")
2. System displays a validation error before submission

## Failures / Exceptions

- If the database is unreachable, display a generic error and log the incident
```

### Front-Matter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | ULID | Yes | Universally unique identifier in [ULID](https://github.com/ulid/spec) format. The single key for bidirectional traceability between specres and source code. |
| `name` | string | Yes | Matches the filename (without `.md`). Human-readable behavior title. |
| `type` | string | No | Free-form classification (e.g., `behavior`, `constraint`, `usecase`, `ui-interaction`). Projects define their own vocabulary. |
| `status` | enum | Yes | `draft` · `in-development` · `stable` · `deprecated` |
| `last_verified` | date | Yes | `YYYY-MM-DD` — when this specre was last confirmed to match the implementation. |

### Status Lifecycle

```
draft ──→ in-development ──→ stable ──→ deprecated
  ↑            │                │
  └────────────┘                │
  (requirements changed)        │
                                ↓
                           (superseded or removed)
```

- **draft**: Behavior is proposed but not yet implemented.
- **in-development**: Implementation or tests are in progress.
- **stable**: Implementation matches the specre. Tests pass. Verified by `last_verified` date.
- **deprecated**: Behavior has been removed or superseded. Kept for historical reference.

### Recommended Sections

| Section | Required | Purpose |
|---------|----------|---------|
| Related Files | Yes | Links specre to source and test files by path (human-readable) |
| Functional Overview | Yes | One-paragraph summary of the behavior |
| Design Intent | No | Explains *why*, not *what* |
| Key Members | No | Important state and parameters in natural language |
| Scenarios | Yes | Step-by-step behavior descriptions |
| Failures / Exceptions | No | Edge cases and error handling |

### Writing Guidelines

- Write scenarios in **natural language**, not code. Avoid copy-pasting implementation details.
- **Do** use exact names for signals, classes, and enums — these are the contract between spec and code.
- Keep each specre **focused on a single behavior**. If you find yourself writing "also, ..." — split it into a separate specre.
- specres can reference each other via relative paths in a "Referenced Specifications" section.

## Bidirectional Traceability

specre uses a single ULID — the specre's `id` — to link specifications and source code in both directions. No intermediate tag layer is needed.

### How It Works

```
┌─────────────────────────────┐
│  specre file (.md)         │  ← Source of truth
│  ┌────────────────────────┐ │
│  │ id: ULID               │ │
│  │ name / type / status   │ │
│  │ last_verified          │ │
│  └────────────────────────┘ │
│  ## Related Files           │  ← Path-based reference (human-readable)
│  ## Scenarios               │
└──────────────┬──────────────┘
               │ id (ULID)
               ▼
┌──────────────────────────────┐
│  source file                 │
│  // @specre <ULID>          │  ← Reverse reference (machine-readable)
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│  index.json                  │  ← Generated cache (CLI-managed)
│  specres[]: front-matter    │
│  source_refs[]: marker index │
└──────────────────────────────┘
```

- **specre → Source**: The "Related Files" section lists file paths. Human-readable, always up to date with the specre.
- **Source → specre**: A `@specre` marker in a source comment references the specre's `id`. Machine-readable, scannable by CLI.
- **index.json**: A derived artifact generated by `specre index`. Can be regenerated at any time. Never edit manually.

### Source File Markers

Place a `@specre` marker in a comment to link a source file back to its specre:

```ruby
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
class CreateQuotation < Usecase
  # ...
end
```

A file may reference multiple specres:

```python
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
# @specre 01HZYQ4N7XW3A8B5C6D9E0F1G2
class QuotationService:
    ...
```

The marker can be placed at the top of a file, above a class or function definition, or at the end — wherever it best communicates the relationship. The CLI detects markers by scanning for the pattern `@specre [0-9A-Z]{26}`, ignoring comment syntax.

**Marker syntax by language:**

| Language | Marker |
|----------|--------|
| Ruby / Python / GDScript / Shell | `# @specre 01HZYPM...` |
| JavaScript / TypeScript / Java / C# / C++ | `// @specre 01HZYPM...` |
| HTML / XML | `<!-- @specre 01HZYPM... -->` |
| CSS | `/* @specre 01HZYPM... */` |
| SQL | `-- @specre 01HZYPM...` |

### Why ULID?

[ULID](https://github.com/ulid/spec) (Universally Unique Lexicographically Sortable Identifier) was chosen over UUID or sequential IDs because:

- **Sortable by creation time**: specres created earlier sort first, providing a natural chronological order without relying on filenames.
- **No coordination required**: ULIDs can be generated independently by any developer or agent without a central registry.
- **Compact**: 26 characters (vs. 36 for UUID), reducing noise in source comments.
- **Monotonic within a millisecond**: Multiple specres created in rapid succession maintain order.

## Index Format

`specre index` scans the specification directory and source tree to generate `index.json` — a machine-readable cache for fast lookups.

```json
{
  "version": 1,
  "generated_at": "2026-03-01T12:00:00Z",
  "specres": [
    {
      "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
      "name": "見積を新規作成できる",
      "type": "usecase",
      "status": "draft",
      "domain": "quotation",
      "path": "docs/specres/quotation/001_見積を新規作成できる.md",
      "last_verified": "2026-03-01"
    }
  ],
  "source_refs": [
    {
      "specre_id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
      "file": "app/usecases/create_quotation.rb",
      "line": 1
    }
  ]
}
```

### `specres` Array

Each entry mirrors the front-matter of a specre file, with two derived fields:

| Field | Source | Description |
|-------|--------|-------------|
| `id` | front-matter | The specre's ULID |
| `name` | front-matter | Human-readable title |
| `type` | front-matter | Classification (free-form string) |
| `status` | front-matter | Current lifecycle status |
| `domain` | directory name | Extracted from the specre's parent directory (e.g., `docs/specres/quotation/` → `"quotation"`) |
| `path` | file system | Relative path from project root to the specre file |
| `last_verified` | front-matter | Date of last verification |

### `source_refs` Array

Each entry records a `@specre` marker found in the source tree:

| Field | Description |
|-------|-------------|
| `specre_id` | The ULID referenced by the marker |
| `file` | Relative path to the source file |
| `line` | Line number where the marker was found |

### Design Principles

- **index.json is a cache, not a source of truth.** If it's missing or stale, run `specre index` to regenerate. Never edit it manually.
- **specre files are the source of truth.** All authoritative data lives in the front-matter and body of each `.md` file.
- **Per-directory INDEX.md** can also be generated for human browsing — a Markdown table summarizing all specres in a subdirectory.

## Comparison with Other Tools

| | specre | GitHub Spec Kit | Amazon Kiro | ADR | Gherkin |
|---|---|---|---|---|---|
| Granularity | One behavior | One feature | One feature | One decision | One feature (multi-scenario) |
| Unique ID | ULID per specre | None | None | Sequential number | None |
| Lifecycle status | Yes (4 states) | No | No | Yes (3-4 states) | No |
| Verification date | Yes | No | No | No | No |
| Code traceability | Bidirectional (ULID) | None | None | None | Step definitions |
| Test integration | By convention | Optional | Optional | No | Executable |
| Process coupling | None | Linear pipeline | Linear pipeline | None | Test runner |
| Files per spec | 1 | 3-4 | 3 | 1 | 1 |
| IDE dependency | None | None | Kiro IDE | None | None |
| Index generation | CLI (JSON + Markdown) | None | Built-in | CLI (optional) | None |

## Roadmap

### v0.1 — Core CLI

- [ ] `specre init` — Scaffold a new specre from a template, auto-generating a ULID for the `id` field
- [ ] `specre index` — Scan specre directory and source tree; generate `index.json` and per-domain `INDEX.md`
- [ ] `specre status` — Report specre counts by status and flag stale `last_verified` dates

### v0.2 — Traceability

- [ ] `specre trace <ULID>` — Given a ULID, show the specre file and all source files referencing it (or vice versa)
- [ ] `specre orphans` — Detect specres with no `@specre` markers in source, or markers with no matching specre
- [ ] `specre tag <ULID> <file>` — Insert a `@specre` marker into a source file at the appropriate location

### v0.3 — Drift Detection

- [ ] `specre drift` — Compare `last_verified` dates against git history of related files; flag specres where source has changed since last verification
- [ ] `specre ci` — Exit with non-zero status if drift or orphans are detected (for CI integration)
- [ ] GitHub Actions workflow template

### v0.4 — Agent Integration

- [ ] Skill / prompt templates for Claude Code, Cursor, and other AI coding assistants
- [ ] `specre search <query>` — Full-text + type/status/domain filtering across all specres, with JSON output for agent consumption
- [ ] Agent-friendly output formats across all commands (`--json`, `--md`)

### Future Considerations

- Mermaid diagram generation from cross-references between specres
- Dependency graph visualization
- Multi-language support for specre content (i18n metadata)
- Plugin system for custom front-matter fields and validation rules

## Contributing

specre is in its early stages. Contributions, feedback, and real-world usage reports are welcome. Please open an issue or pull request on GitHub.

## License

MIT

Atomic, living specification cards for AI-agent-friendly development. Minimal, agnostic, and traceable.
