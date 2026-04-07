---
description: "Triage drifted specre cards: subagent reviews each drift, then auto-updates last_verified for false positives"
---

You are executing the specre drift triage workflow. The user may provide filters as `$ARGUMENTS` (e.g., a domain name, a ULID, or `--grace 0`).

Your goal: process every drifted specre card **one at a time**, delegating the spec-vs-implementation comparison to a subagent, and updating `last_verified` for confirmed false positives.

**Critical constraint — sequential processing with subagent isolation:** Do NOT read spec cards or their related source files yourself. For each drifted item, spawn a subagent that reads the spec card and its changed files, then returns a structured verdict. This keeps your context window clean and prevents quality degradation across many items.

---

## Phase 0: Setup

1. Read `specre.toml` and check for the `language` field.
   - If `language` is set, all user-facing output MUST be written in that language. Tool calls, file paths, and code remain untranslated.
   - If `language` is not set, default to English.

2. Run `specre drift --json` (append any `$ARGUMENTS` the user provided).

3. Parse the JSON output. If `drifted` is empty:
   > No drifted specre cards found. All specifications are up to date.

   Stop here.

4. Present a summary to the user:
   > Found N drifted specre cards (M clean, grace: G days). Starting triage.

   List the drifted card names briefly (id + name, one per line) before beginning Phase 1.

---

## Phase 1: Sequential Triage

Process each drifted item **one at a time, in order**. For each item:

### Step 1.1: Spawn Subagent

Launch an Agent (subagent_type: general-purpose) with the following information:

- The specre card path (from the drift JSON `path` field)
- The list of changed files with their `last_modified` dates and `diff_stat`
- The `last_verified` date from the drift JSON

The subagent's task prompt must instruct it to:

1. **Read the specre card** — focus on Functional Overview, Scenarios, and Failures / Exceptions.
2. **For each changed file**, run `git diff` from `last_verified` to HEAD for that file, and read the current source to understand the change in context:
   ```
   git log --after="<last_verified>" --format="%h %s" -- <file>
   git diff $(git log --after="<last_verified>" --format="%H" -- <file> | tail -1)^..HEAD -- <file>
   ```
3. **Compare** the spec's described behavior against the actual code changes.
4. **Return a structured verdict** as the final output, using exactly this format:

```
VERDICT: <no_drift | drift_spec_stale | drift_impl_wrong | uncertain>
CONFIDENCE: <high | medium | low>
SUMMARY: <1-2 sentence explanation>
CHANGED_BEHAVIORS: <comma-separated list of affected scenarios, or "none">
```

Verdict definitions:
- `no_drift` — The code changes do not affect the behavior described in the specre card. The spec remains accurate. (Common causes: refactoring, formatting, changes to unrelated parts of a shared file, new features added alongside existing behavior.)
- `drift_spec_stale` — The implementation has intentionally changed behavior, and the specre card no longer accurately describes the current implementation. The spec needs updating.
- `drift_impl_wrong` — The implementation appears to have diverged from the spec in a way that looks unintentional (e.g., a regression, a missing edge case). The implementation may need fixing.
- `uncertain` — Cannot determine with confidence. The change is ambiguous or touches areas not well-described by the spec.

### Step 1.2: Process Verdict

Based on the subagent's returned verdict:

**`no_drift`:**

Record the ULID for bulk verification later (Phase 2). Report to the user:
> [check] `<name>` — false positive. Will update `last_verified` in bulk.
> Reason: <subagent's SUMMARY>

**`drift_spec_stale`:**

Do NOT update anything. Report to the user:
> [drift] `<name>` — spec is stale. The specification needs updating.
> Detail: <subagent's SUMMARY>
> Affected scenarios: <CHANGED_BEHAVIORS>
> Suggested action: Run `/specre-sdd-fix` to update this specre card, or manually edit `<path>`.

**`drift_impl_wrong`:**

Do NOT update anything. Report to the user:
> [drift] `<name>` — implementation may have regressed.
> Detail: <subagent's SUMMARY>
> Affected scenarios: <CHANGED_BEHAVIORS>
> Suggested action: Review the implementation against the specre card's scenarios.

**`uncertain`:**

Do NOT update anything. Report to the user:
> [?] `<name>` — requires manual review.
> Detail: <subagent's SUMMARY>
> Suggested action: Read the specre card at `<path>` and the changed files, then decide whether to update `last_verified` or revise the spec.

**After processing each item, proceed immediately to the next.** Do not wait for user input between items.

---

## Phase 2: Summary

After all drifted items have been processed, present a final summary:

```
────────────────────────────────────────
Triage Complete
────────────────────────────────────────
Total drifted:     N
False positives:   X (last_verified updated)
Spec stale:        Y (needs /specre-sdd-fix)
Impl regression:   Z (needs code review)
Uncertain:         W (needs manual review)
────────────────────────────────────────
```

If there are any `no_drift` items collected during Phase 1, run `specre verify` with all their ULIDs in a single command:
```
specre verify <ULID1> <ULID2> <ULID3> ...
```
Report the result to the user.

If any items were NOT auto-resolved (spec stale, impl regression, or uncertain), list them with their suggested actions as a concise action list.

If ALL items were false positives:
> All drifted specre cards were false positives. The ecosystem is now up to date.

Run `specre index` to regenerate the index after any `last_verified` updates.

---

## Rules

- **Subagent isolation is mandatory.** Never read specre cards or their related source files in the parent agent's context. All spec-vs-code comparison happens in subagents.
- **One item at a time.** Do not batch multiple drifted items into a single subagent. Each specre card gets its own subagent with a fresh context.
- **Only update `last_verified` for `no_drift` verdicts.** Never update for other verdicts.
- **Use `specre verify` for updates.** Do not edit specre card front-matter directly. Collect all `no_drift` ULIDs during Phase 1, then run a single `specre verify <ULID>...` command in Phase 2.
- **Do not modify source code or specre card content.** This workflow only updates `last_verified` via the `specre verify` command.
- **Respect the `language` setting.** All user-facing prose follows `specre.toml`'s `language` field. Technical tokens remain untranslated.
