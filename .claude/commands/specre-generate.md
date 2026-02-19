---
description: "Generate specre cards for uncovered source files in a domain"
---

You are executing the specre-generate workflow. The user has optionally provided a domain or subdirectory as `$ARGUMENTS`.

Your goal: create specre specification cards for source files that currently lack specre coverage, working through them **one card at a time** in a structured sequence.

**Critical constraint — sequential generation:** Do NOT attempt to draft or hold multiple card contents in your context simultaneously. Each card must be fully created, written to disk, and tagged before you begin analyzing the next behavior. This prevents context-window saturation from degrading the quality of later cards.

## Phase 0: Setup

1. Read `specre.toml` to determine `specre_dir` and `source_dirs`.
2. If `$ARGUMENTS` is empty, ask the user which domain or subdirectory to target.
3. Confirm the target domain with the user before proceeding.

## Phase 1: Discovery

1. Run `specre coverage` to identify uncovered source files.
2. Filter the **Uncovered files** list to only those belonging to the target domain.
   - "Domain" means the top-level functional directory within each `source_dirs` entry (e.g., `src/auth/`, `src/cart/`).
   - Exclude test files from the generation targets. Test files are used as evidence for status determination (Phase 3), not as specre subjects.
3. Identify the project's test file convention by examining the directory structure (e.g., `tests/<domain>/cli_*.rs`, `src/**/*.test.ts`, `spec/**/*_spec.rb`). Record this pattern for reuse in Phase 3b so that test discovery does not need to be re-explored for each card.
4. Present the filtered file list to the user:
   > "Found N uncovered source files in the `<domain>` domain. Shall I proceed with analyzing these files and generating specre cards?"

## Phase 2: Behavior Classification

**Purpose:** Build a complete picture of the domain's behaviors BEFORE creating any cards. This prevents duplicate cards, identifies cross-file behaviors, and establishes a logical generation order.

> **Large domain shortcut:** If the domain contains more than 15 uncovered files, use the Task tool with `subagent_type=Explore` to read and classify files in batches. This protects the main context window from saturation while still building a complete catalog.

For each uncovered file in the list:

1. **Read the source file** and identify its primary behaviors.
2. **Identify tightly coupled files** that participate in the same behavior:
   - Base classes or traits that the file extends/implements
   - Types that the file instantiates or depends on directly
   - Files where the file's public interface is consumed extensively
3. **Search for existing specre cards** that may already cover this behavior:
   ```
   specre search "<subject> <action_verb>"
   ```
   Use AND-keyword queries combining the behavior's subject (noun) and action (verb). For example: `specre search "order approve"`, `specre search "token validate"`.

After analyzing all files, produce a **behavior catalog** — a numbered list of proposed specre cards:

```
Behavior Catalog for domain: <domain>

 1. [subject]_[predicate]
    Source files: src/domain/file_a.ext, src/domain/file_b.ext
    Action: NEW — no existing specre covers this behavior

 2. [subject]_[predicate]
    Source files: src/domain/file_c.ext
    Action: EXTEND — add to existing specre [ULID] ([existing_name])

 3. ...
```

Each entry must specify:
- **Proposed name**: subject + predicate sentence form (see specre-author skill for naming rules)
- **Source files**: which files this card will cover
- **Action**: `NEW` (create a new card) or `EXTEND` (tag source file to an existing card and update its Related Files)

**Classify by subject first, then by behavior.** Group related behaviors by their actor/subject (e.g., all `user_can_*` behaviors together, all `system_rejects_*` together). This produces a natural reading order and makes it easy to spot missing behaviors.

Present the catalog to the user for approval. The user may:
- Approve as-is
- Request splitting a behavior into multiple cards
- Request merging multiple entries into one card
- Rename proposed behaviors
- Remove entries they don't want specre cards for

**Do not proceed to Phase 3 until the user approves the catalog.**

After approval, write the finalized catalog to `<specre_dir>/<domain>/_GENERATION_PLAN.md`. This file serves as a persistent reference during Phase 3 — if context compression causes the catalog to be summarized or lost, re-read this file to recover the full plan. This file is deleted in Phase 4 after all entries are processed.

## Phase 3: Sequential Card Generation

Before starting, create a TodoWrite entry for each catalog item (use the catalog entry name as the task content). Mark each entry as `in_progress` when you begin processing it, and `completed` when the card is fully written, tagged, and status-determined. If context compression occurs, re-read `<specre_dir>/<domain>/_GENERATION_PLAN.md` and the current TodoWrite state to recover your position.

Process the approved catalog entries **one at a time, in order**. For each entry:

### Step 3a: Create or Extend

**If action is `NEW`:**

1. Run `specre new <specre_dir>/<domain> --name "<behavior_name>"` to scaffold the card.
2. Fill in the card content following these authoring rules:
   - **Related Files**: The source files listed in the catalog entry, plus any tightly coupled files identified in Phase 2. Use project-root-relative paths. Suffix test files with `(Test)`.
   - **Functional Overview**: A one-paragraph summary of the behavior, derived from the source code.
   - **Scenarios**: Step-by-step behavior descriptions in **natural language**. Do NOT copy-paste code into scenarios. Exception: use exact names for signals/events, class/type names, enum values, and API endpoints. Aim for 2–5 scenarios per card — fewer suggests the card is a fragment of a larger behavior, more suggests it conflates multiple behaviors.
   - **Design Intent**: Include if the reasoning is apparent from the code. Omit if unclear — do not fabricate rationale.
   - **Key Members**: Include if there are important state variables or parameters. Omit otherwise.
   - **Failures / Exceptions**: Include if the code has explicit error handling paths. Omit otherwise.
3. Run `specre tag <ULID> <source_file>` for each source file in the entry.

**If action is `EXTEND`:**

1. Add the source file path to the existing specre card's "Related Files" section.
2. Run `specre tag <existing_ULID> <source_file>` to insert the marker.
3. Mark this TodoWrite entry as `completed` and move to the next catalog entry. **Skip Steps 3b and 3c for EXTEND actions.**

### Step 3b: Test Discovery and Status Determination (NEW actions only)

1. Search for test files corresponding to the source files in this entry, using the test convention identified in Phase 1 step 3. Apply the recorded glob pattern within the target domain's test directory.
2. **If matching tests exist:**
   - Add the test file paths to the card's "Related Files" section with a `(Test)` suffix.
   - Compare the test assertions against the card's scenarios.
     - **If they align**: Set `status` to `stable` and `last_verified` to today's date. Mark this card internally as **auto-stabilized** for the review prompt in Phase 4.
     - **If they diverge**: Keep `status` as `draft`.
3. **If no matching tests exist:**
   - Keep `status` as `draft`.
   - Do NOT create test files. This workflow only generates specre cards.

### Step 3c: Size Check

After writing the card, check its length. If the card body (excluding front-matter) exceeds roughly 120 lines, it likely covers more than one behavior. Split it into separate cards and re-run Steps 3a–3b for each.

**Confirm completion of this entry before moving to the next one.**

## Phase 4: Validation and Review

After all catalog entries are processed:

1. Delete `<specre_dir>/<domain>/_GENERATION_PLAN.md` (the generation plan is no longer needed).
2. Run `specre index` to regenerate the index.
3. Run `specre orphans` to verify there are no unlinked cards or dangling markers.
4. Run `specre coverage` and report the coverage change (before vs. after).
5. Present a summary to the user:

```
Generation complete.

Created: N new specre cards
Extended: M existing specre cards
Coverage: X% → Y%

Cards marked stable (auto-stabilized from tests):
  - docs/specres/domain/behavior_a.md
  - docs/specres/domain/behavior_b.md

Cards remaining as draft:
  - docs/specres/domain/behavior_c.md (no matching tests)
  - docs/specres/domain/behavior_d.md (test scenarios diverge)
```

6. **If any cards were auto-stabilized**, append this notice:

> Some cards were automatically set to `stable` because matching tests were found and their assertions align with the documented scenarios. We recommend reviewing these cards to confirm that the specification accurately reflects the intended behavior, not just the current implementation.

## Rules

- **Autonomous within phases.** Do not pause for user input between individual card generations in Phase 3. The user approval point is the behavior catalog in Phase 2.
- **Write in natural language.** Scenarios must be written in natural language, not in code. See the specre-author skill for the code-independence principle and its exceptions.
- **Never create test files.** This workflow generates specre cards only.
- **Never modify source files** beyond inserting `@specre` markers via `specre tag`.
