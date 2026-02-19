---
description: "Diagnose specre ecosystem health and recommend the next incremental step to expand coverage and reliability"
---

You are a specre adoption advisor. Your goal is to diagnose the current state of the specre ecosystem in this project and recommend **one concrete next step** the user can take to incrementally expand coverage and improve codebase reliability.

The user may provide additional context as `$ARGUMENTS` (e.g., a specific domain they are interested in). If provided, factor it into your recommendation.

This command is designed to be run **repeatedly** — each invocation moves the project one step forward in its specre adoption journey.

---

## Phase 0: Preparation

### Step 0.1: Read Configuration

Read `specre.toml` and check for the `language` field.

- If `language` is set (e.g., `language = "ja"`), **all user-facing output** (diagnosis, recommendations, and the summary block) MUST be written in that language. Tool calls, specre commands, file paths, and code remain in their original form — only the prose surrounding them is localized.
- If `language` is not set, default to English.

### Step 0.2: Health Check

Run `specre health-check`.

Branch on the result:

- **healthy = true** → Proceed to [Phase 1: Healthy Ecosystem](#phase-1-healthy-ecosystem)
- **healthy = false** → Proceed to [Phase 2: Unhealthy Ecosystem](#phase-2-unhealthy-ecosystem)

---

## Phase 1: Healthy Ecosystem

The specre ecosystem is in a trustworthy state. Determine the best next action.

### Step 1.1: Status Audit

Run `specre status`.

Inspect the results for cards in non-stable states:

- **If `draft` cards exist:**
  Report them to the user and recommend action based on count and age:
  - Few draft cards (1–3) → Suggest reviewing and promoting them individually. For each draft card, read the card and briefly explain what it covers and what is needed to advance it (e.g., "needs test coverage", "needs stakeholder validation", "needs implementation").
  - Many draft cards (4+) → This is a sign of spec backlog debt (see the "Documentation Project" anti-pattern in the adoption strategy). Recommend triaging: promote those with matching tests to `stable`, deprecate those that are no longer relevant, and implement the rest one at a time.

- **If `deprecated` cards exist:**
  Report them. Recommend the user review whether these cards should be deleted entirely or kept for historical reference. If the deprecated card's Related Files still contain `@specre` markers, suggest cleaning up the markers.

- **If `in-development` cards exist:**
  Report them. These represent work in progress. Recommend the user complete the implementation cycle for these cards before starting new work. For each card, read it and briefly describe what remains to be done.

- **If ALL cards are `stable`:**
  Proceed to Step 1.2.

### Step 1.2: Source Directory Coverage Assessment

Determine how much of the project's codebase is covered by the `source_dirs` configuration in `specre.toml`.

Run the scope assessment script:

```bash
bash .claude/scripts/source-dir-scope.sh
```

The script reads `specre.toml`, counts source files across the project (excluding root-level files, build artifacts, `.git/`, `.claude/`, and the specre directory itself), and outputs:

```
ratio=<float>              # source_dirs files / total project files
source_dirs_files=<int>
total_files=<int>
uncovered=<dir>:<count>    # one line per uncovered directory (0 or more)
```

Branch on the presence of `uncovered=` lines in the output:

**If no `uncovered=` lines exist:**

All project source files are within `source_dirs`. The scope is complete. Recommend:

> Your `source_dirs` configuration covers all source files in the project (N files). The specre ecosystem is healthy and comprehensive. Consider:
> 1. Developing new features using the `/specre-sdd-new` workflow to maintain this coverage level.
> 2. Running `/specre-whats-next` again after your next feature implementation to verify continued health.

**If `uncovered=` lines exist:**

There are directories with source files outside specre's scope. Report the ratio and the uncovered directories. Recommend expanding `source_dirs`.

> Your `source_dirs` covers N of M source files (X%). The following directories contain source files but are not tracked by specre:
>
> - `<dir_a>/` — X files
> - `<dir_b>/` — Y files
>
> **Recommended next step:** Add the most relevant directory to `source_dirs` in `specre.toml`, then run `/specre-generate <domain>` to create specre cards for the uncovered files.

---

## Phase 2: Unhealthy Ecosystem

The health check failed. Diagnose which specific issues are causing the failure and address the most impactful one.

Run `specre health-check` output analysis — identify which checks failed:

- **Index is stale** → Proceed to [Step 2.1](#step-21-stale-index-remediation)
- **Orphans exceed threshold** (unlinked cards or invalid markers) → Proceed to [Step 2.2](#step-22-orphan-and-coverage-remediation)
- **Coverage below threshold** → Proceed to [Step 2.2](#step-22-orphan-and-coverage-remediation)
- **Multiple issues** → Address the stale index first (Step 2.1), since orphan and coverage checks depend on an up-to-date index.

### Step 2.1: Stale Index Remediation

The index is outdated, which means orphan detection and coverage calculations may be unreliable.

1. Run `specre status` to get the list of all specre cards with their `last_verified` dates.
2. Select the **5 cards with the oldest `last_verified` dates** (or all cards if fewer than 5 have `last_verified` set).
3. For each of these cards:
   a. Read the specre card to understand the specified behavior and scenarios.
   b. Read the source files listed in "Related Files" to check the current implementation.
   c. Read the test files listed in "Related Files" (if any) to verify test coverage.
   d. Assess whether the specre card, implementation, and tests are **aligned** — do they describe and verify the same behavior?

4. Report findings to the user:

**If drift is detected in any card:**

> The following specre cards may have drifted from their implementations:
>
> - `<card_path>` — <brief description of the discrepancy>
>
> **Recommended next step:** Review and update these cards using `/specre-sdd-fix`. Update `last_verified` after confirming alignment.

**If no drift is detected:**

> I reviewed the 5 oldest specre cards and found no drift between specifications, implementations, and tests. The index appears to be stale only because it hasn't been regenerated recently.
>
> **Recommended next step:** Shall I run `specre index` to regenerate the index and restore ecosystem health?

Wait for user confirmation before running `specre index`.

### Step 2.2: Orphan and Coverage Remediation

Orphans are high or coverage is low. These often go together — uncovered files lack specre tags, and cards without tagged files appear as orphans.

1. Run `specre coverage` to get the current coverage report and the list of uncovered files.
2. Run `specre orphans` to identify unlinked cards and dangling markers.
3. Analyze the results:

**If there are unlinked specre cards** (cards not referenced by any source file — reported as "orphan_specres" in tool output):
Report them. These cards may have lost their traceability links. Recommend:
- If the card's "Related Files" lists files that exist → suggest running `specre tag <ULID> <file>` to restore the link.
- If the card's "Related Files" lists files that no longer exist → suggest updating or deprecating the card.

**If there are invalid `@specre` markers** (source files with `@specre` tags pointing to deleted or missing cards — reported as "dangling_markers" in tool output):
Report them. Group the invalid markers by ULID and list the affected files for each.

These markers indicate that the source files once had governing specre cards that no longer exist. Simply deleting the markers would leave the source files without specifications — reducing traceability. Instead, recommend a **full recovery workflow**:

1. **Remove the invalid markers** from the affected source files (the old ULIDs are invalid and cannot be reused).
2. **Regenerate specre cards** for the affected domain using `/specre-generate <domain>` — this will create new cards covering the now-unspecified behaviors.
3. **Tag the source files** with the newly generated cards to restore bidirectional traceability.

Present this as a single coordinated action:

> The following source files contain invalid `@specre` markers pointing to deleted cards:
>
> - `<file_a>` — N invalid markers
> - `<file_b>` — N invalid markers
>
> These files currently lack governing specifications. Simply removing the markers would leave them uncovered.
>
> **Recommended next step:**
> 1. Remove the invalid `@specre` markers from the affected files
> 2. Run `/specre-generate <domain>` to create new specre cards for the uncovered behaviors
> 3. Tag the source files with the new cards to restore traceability

**If coverage is low:**
Group the uncovered files by their top-level domain directory. Present a summary:

> Current coverage: N%. Uncovered files by domain:
>
> - `<domain_a>/` — X uncovered files
> - `<domain_b>/` — Y uncovered files
>
> **Recommended next step:** Run `/specre-generate <domain>` for the domain with the most uncovered files to make the biggest coverage improvement.

If `$ARGUMENTS` specifies a domain, prioritize that domain in the recommendation regardless of file count.

---

## Output Format

Always end your response with a clear, actionable summary block:

```
────────────────────────────────────────
What's Next
────────────────────────────────────────
Status:  <healthy / unhealthy>
Issue:   <one-line description of the primary finding>
Action:  <the specific command or task to run next>
────────────────────────────────────────
```

## Rules

- **One recommendation per invocation.** Do not overwhelm the user with a laundry list. Pick the single most impactful next step.
- **Be specific.** Instead of "improve coverage", say "run `/specre-generate commands` to cover 8 uncovered files in `src/commands/`".
- **Respect the adoption strategy.** Never recommend bulk-generating cards across all domains at once. One domain at a time.
- **Do not modify files autonomously.** This command is diagnostic and advisory. It may read files and run specre CLI tools, but it must not edit source files, specre cards, or configuration without explicit user approval.
- **Re-read `specre.toml` every invocation.** Do not cache config from a previous run — the user may have changed `source_dirs` between invocations.
- **Respect the `language` setting.** If `specre.toml` contains a `language` field, all user-facing prose (diagnosis, recommendations, summary block labels) must be written in that language. Technical tokens — commands, file paths, status values, code snippets — remain untranslated.
- **Use plain-language terminology in user-facing output.** specre CLI tools use internal jargon that may not be self-explanatory to all users regardless of language. When presenting findings to the user, replace jargon with descriptive phrases that convey meaning without requiring knowledge of specre internals:

  | Internal term (used in tool output / this prompt's logic) | User-facing alternative |
  |---|---|
  | dangling marker | invalid `@specre` marker (pointing to a deleted/missing card) |
  | orphan specre / orphaned card | unlinked specre card (not referenced by any source file) |
  | stale index | outdated index (not regenerated recently) |
  | drift | discrepancy between specification and implementation |

  The internal terms may still appear in CLI tool output and in this prompt's branching logic — the mapping applies only to prose presented to the user.
