# CI Integration Guide

How to integrate specre's health-check into your CI pipeline to catch specification drift, coverage drops, and orphaned cards before they reach the main branch.

## Why CI Integration?

Without CI enforcement, specre ecosystems degrade silently:

- A developer adds a new source file but forgets the `@specre` marker — coverage drops
- A refactor moves code but doesn't update the specre card's Related Files — orphans appear
- A feature change modifies behavior but `last_verified` is never updated — drift accumulates

`specre health-check` catches all of these in a single command. It exits with code 1 when any metric falls below the configured thresholds, making it a natural fit for CI gates.

## Prerequisites

- A project with `specre.toml` already configured (see [Getting Started](START-SPECRE.md))
- Git history available in CI (needed for drift detection)

---

## GitHub Actions

### Basic: Health-Check Gate

The simplest integration — run `specre health-check` as a required check on pull requests.

```yaml
name: specre

on:
  pull_request:
    branches: [main]

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history needed for drift detection

      - name: Install specre
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

      - name: Regenerate index
        run: specre index

      - name: Run health-check
        run: specre health-check
```

**Key points:**

- `fetch-depth: 0` — Full git history is required for drift detection. Without it, `specre drift` cannot compare `last_verified` dates against file modification history.
- `specre index` before `health-check` — The index must be fresh for accurate results. Regenerating it in CI ensures the `index_age_hours` check always passes.
- Exit code 1 on failure — `specre health-check` returns a non-zero exit code when any metric is unhealthy, which automatically fails the GitHub Actions step.

### Advanced: With Drift Report on PR

Add a step that posts drift details as a PR comment, making it easy for reviewers to see which specre cards need attention.

```yaml
name: specre

on:
  pull_request:
    branches: [main]

permissions:
  pull-requests: write

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install specre
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

      - name: Regenerate index
        run: specre index

      - name: Run health-check
        id: health
        run: |
          specre health-check | tee health.json
        continue-on-error: true

      - name: Run drift check
        id: drift
        if: steps.health.outcome == 'failure'
        run: |
          specre drift --json | tee drift.json || true

      - name: Comment on PR
        if: steps.health.outcome == 'failure'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const health = JSON.parse(fs.readFileSync('health.json', 'utf8'));
            const hasDrift = fs.existsSync('drift.json');
            const drift = hasDrift ? JSON.parse(fs.readFileSync('drift.json', 'utf8')) : null;

            let body = `## specre Health Check Failed\n\n`;
            body += `| Metric | Value | Threshold | Status |\n`;
            body += `|--------|-------|-----------|--------|\n`;
            body += `| Coverage | ${health.coverage} | ≥ ${health.thresholds.coverage} | ${health.coverage >= health.thresholds.coverage ? '✅' : '❌'} |\n`;
            body += `| Orphans | ${health.orphans} | ≤ ${health.thresholds.orphans} | ${health.orphans <= health.thresholds.orphans ? '✅' : '❌'} |\n`;
            if (health.drifts !== null) {
              body += `| Drifts | ${health.drifts} | ≤ ${health.thresholds.drifts} | ${health.drifts <= health.thresholds.drifts ? '✅' : '❌'} |\n`;
            }
            body += `| Index Age (hours) | ${health.index_age_hours ?? 'N/A'} | ≤ ${health.thresholds.index_age_hours} | ✅ |\n`;

            if (drift && drift.drifted && drift.drifted.length > 0) {
              body += `\n### Drifted Specre Cards\n\n`;
              for (const d of drift.drifted) {
                body += `- **${d.name}** (\`${d.id}\`) — last verified: ${d.last_verified}\n`;
                for (const f of d.changed_files) {
                  body += `  - \`${f.file}\` (modified: ${f.last_modified}, ${f.diff_stat})\n`;
                }
              }
            }

            // Find and update existing comment, or create new one
            const marker = '## specre Health Check Failed';
            const { data: comments } = await github.rest.issues.listComments({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
            });
            const existing = comments.find(c => c.body.startsWith(marker));
            if (existing) {
              await github.rest.issues.updateComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                comment_id: existing.id,
                body,
              });
            } else {
              await github.rest.issues.createComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: context.issue.number,
                body,
              });
            }

      - name: Fail if unhealthy
        if: steps.health.outcome == 'failure'
        run: exit 1
```

### Warning-Only Mode (Not Recommended)

If your team is not yet ready to enforce specre health as a hard gate, you can treat the health-check as a warning instead of a failure. The workflow will still run and report results, but it will not block the pull request from being merged.

> **This is not recommended.** Without enforcement, specre ecosystems degrade over time because there is no feedback loop to catch coverage drops or drift. Use this mode only as a transitional step while ramping up specre adoption, and set a concrete deadline for switching to the enforced gate.

To do this, add `continue-on-error: true` to the health-check step:

```yaml
      - name: Run health-check
        run: specre health-check
        continue-on-error: true
```

In a `ci-gate` pattern (like the one used in this project), exclude the specre job from the failure condition:

```yaml
  ci-gate:
    needs: [test, clippy, fmt, specre]
    # ...
    steps:
      - name: Check results
        run: |
          # specre is intentionally excluded — warning only
          if [[ "${{ needs.test.result }}" == "failure" || \
                "${{ needs.clippy.result }}" == "failure" || \
                "${{ needs.fmt.result }}" == "failure" ]]; then
            echo "CI failed"
            exit 1
          fi
```

The specre job will still appear in the PR checks list with a red or green status, giving visibility without blocking.

---

## Configuring Thresholds for CI

CI thresholds are configured in `specre.toml`. Adjust these values to match your project's maturity:

```toml
[health_check]
coverage = 0.30        # Start low, ratchet up as coverage grows
orphans = 10           # Allowed orphaned specre cards
index_age_hours = 48   # Not critical in CI (index is regenerated)
drifts = 0             # Allowed drifted specre cards (with grace applied)

[drift]
grace_days = 7         # Changes within this window are not counted as drift
```

### Recommended progression

| Phase | Coverage | Orphans | Drifts | Grace |
|-------|----------|---------|--------|-------|
| Initial adoption | `0.30` | `10` | `3` | `7` |
| Growing ecosystem | `0.60` | `5` | `0` | `7` |
| Mature ecosystem | `0.80` | `0` | `0` | `3` |

Start with lenient thresholds and tighten them as your specre ecosystem matures. Jumping to strict thresholds too early creates friction and discourages adoption.

---

## Other CI Platforms

The pattern is the same regardless of platform — install specre, regenerate the index, and run health-check.

### GitLab CI

```yaml
specre:
  image: ubuntu:latest
  script:
    - curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh
    - export PATH="$HOME/.cargo/bin:$PATH"
    - specre index
    - specre health-check
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

### CircleCI

```yaml
version: 2.1
jobs:
  specre:
    docker:
      - image: cimg/base:current
    steps:
      - checkout
      - run:
          name: Install specre
          command: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh
      - run:
          name: Health check
          command: |
            specre index
            specre health-check
```

---

## Troubleshooting

### `specre: command not found`

The installer places the binary in `~/.cargo/bin/`. If your CI environment does not include this in `$PATH`, add it explicitly:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Drift detection shows no results

Drift detection requires full git history. Ensure your checkout step fetches the complete history:

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
```

### Index age always fails

If you are not committing `index.json` to git (recommended), the index won't exist in CI until you generate it. Always run `specre index` before `specre health-check`.

### Health-check passes locally but fails in CI

This usually means `index.json` is committed and stale. Either:
- Add `index.json` to `.gitignore` and regenerate in CI (recommended)
- Run `specre index` locally before committing
