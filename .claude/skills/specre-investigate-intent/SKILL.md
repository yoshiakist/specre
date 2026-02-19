---
name: specre-investigate-intent
description: This skill should be used when the user asks "what is the intent of this code", "explain the specification behind this", "what does this command do", "why does this behavior exist", "what specre governs this file", or when a coding agent needs to understand the specification and design intent of a piece of code, a CLI command, a term, or a behavior in a specre-enabled project. It leverages specre's bidirectional traceability (search, trace, card reading) to locate and present the governing specification card.
---

# Investigate Specification Intent

Locate and present the specre card that governs a given piece of code, command, term, or behavior. Use specre's bidirectional traceability when the ecosystem is healthy; fall back to direct code reading when it is not.

## Phase 0: Determine Tool Availability

Check your available tool list and follow the **first matching** path:

### Path A: specre MCP tools are available

**Detection:** A tool named `mcp__specre__health-check` (or similar `mcp__specre__*` prefix) exists in your tool list.

Run `health-check` immediately.

- **healthy = true** → The specre ecosystem is trustworthy. Use MCP tools (`search`, `trace`, card resources) for all exploration.
- **healthy = false** → Do not rely on specre tools for navigation. Skip to the Fallback phase.

### Path B: specre MCP tools are NOT available, but the `specre` CLI is installed

**Detection:** No `mcp__specre__*` tools exist in your tool list. Run `which specre` to confirm the CLI is on PATH.

Use the companion script bundled with this skill. The script is located relative to the project root:

```bash
.claude/skills/specre-investigate-spec-intent/scripts/specre-investigate.sh \
    [--file <path>] [--query "<terms>"] [--ulid <ULID>]
```

The script wraps `specre health-check`, `specre search`, and `specre trace`. Interpret its structured output and exit code:

- **Exit 0** — at least one specre card was found. Parse the output.
- **Exit 1** — no card found. Proceed to Fallback.
- **Exit 2** — `specre` CLI not available or ecosystem unhealthy. Proceed to Fallback.

### Path C: Neither MCP tools nor CLI are available

**Detection:** No `mcp__specre__*` tools exist, and `which specre` fails.

Skip directly to the Fallback phase.

## Phase 1: Classify the Input

Determine what context is available before choosing a path:

| Input type | Detection |
|---|---|
| **File path** | User referenced a specific file, or IDE context includes an opened file |
| **ULID** | A 26-character uppercase alphanumeric string is present |
| **Code with `@specre` tags** | A file in context already contains `@specre <ULID>` markers |
| **Named subject** | A recognizable function name, command name, or domain term |
| **Plain description or empty** | Natural-language phrase or no explicit subject — derive terms from conversation context |

## Phase 2A: File or Code Provided

1. Read the file if not already in context.
2. Scan for `@specre <ULID>` markers (comment syntax varies by language).

**Markers found:**

- Extract each ULID.
- Run `specre trace <ULID>` for each.
- Read the governing specre card(s) in full. When MCP tools are available, use `ReadMcpResourceTool` to fetch the card directly as a resource with URI `specre:///<ULID>` (e.g. `specre:///01JABCDEFGHJKMNPQRSTVWXYZ0`).
- Proceed to Output.

**No markers found:**

- Derive search terms from: (a) user's description, (b) the file name and directory path, (c) primary function, type, or module names in the file.
- Proceed to Phase 2B.

## Phase 2B: Search Mode

Construct a query using `"<noun> <verb>"` pattern — the subject first, the action second:

- `"tag insert"` → tag marker insertion specre
- `"trace bidirectional"` → traceability lookup specre
- `"coverage report"` → coverage specre

Conduct up to **3 search rounds**, escalating breadth on each miss:

| Round | Strategy | Command |
|---|---|---|
| 1 | AND query — most precise | `specre search "<noun> <verb>"` |
| 2 | Single keyword or rephrased pair | `specre search "<noun>"` or `"<alt_noun> <alt_verb>"` |
| 3 | OR query — widest net | `specre search "<noun> <verb>" --or` |

**Card found** → read the most relevant card in full. Proceed to Output.

**No card found after 3 rounds** → proceed to Fallback.

## Fallback: Direct Code Exploration

When specre search fails to locate a relevant card, or when the ecosystem is unhealthy:

1. Use grep to search for `@specre` markers in files near the subject.
2. Use glob to find related files by name pattern (e.g., `src/commands/<name>.rs`, `tests/cli_<name>.rs`).
3. Read source files and test files directly.
4. Synthesize intent from code structure, function signatures, inline comments, and test scenarios.

Clearly note in the output that no governing specre card was found.

## Output Format

### When specre card(s) are found

```
### Specre Card: <name>
**Status:** <status> | **Last Verified:** <date>
**ID:** <ULID>

**Functional Overview:**
<one-paragraph summary>

**Key Scenarios:**
<numbered list>

**Related Files:**
<list>

**Specification:**
<markdown link to the specre card file, using the `path` field from the search result or card metadata>
```

Present each card in sequence when multiple cards govern the subject.

**Only if coding agent is working on VSCode based IDE**:

- At the end of your response, open each consulted specre card file directly in the editor using the `code -r` command via the Bash tool.
- The `-r` flag reuses the current window and opens the file as a new tab instead of a new window. Use the absolute path constructed from the workspace root and the `path` field returned by the search result or card metadata. Example:

```bash
code -r "<full_path>/name_of_specification.md"
```

If multiple cards were consulted, open each file with a separate `code -r` command. Do NOT output markdown file links — open the files directly instead.

### When no specre card is found

```
### Analysis (no specre card found)
**Source:** direct code reading — <file path(s)>

**Inferred Intent:**
<analysis synthesized from code>
```

## Additional Resources

### Scripts

- **`scripts/specre-investigate.sh`** — CLI-based investigation helper for environments where MCP tools are unavailable. Accepts `--file`, `--query`, and `--ulid` flags. Runs health-check, searches, and traces automatically with structured output.
