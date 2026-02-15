You are executing the release workflow for specre. The user may optionally provide a version as $ARGUMENTS (e.g., `0.2.4`). If not provided, increment the patch version from the current `Cargo.toml`.

Human checkpoints are limited to PR merge only — the agent drives everything else autonomously.

Follow these phases strictly.

---

## Phase 1: Version Bump PR

1. Ensure you are on `main` and up to date: `git checkout main && git pull origin main`
2. Read `Cargo.toml` to determine the current version
3. Calculate the new version (use $ARGUMENTS if provided, otherwise increment the patch version)
4. Create a branch: `chore/bump-version-v<NEW_VERSION>`
5. Update `version` in `Cargo.toml`
6. Run `cargo build` to update `Cargo.lock`
7. Commit `Cargo.toml` and `Cargo.lock`
8. Push the branch and create a PR using `gh pr create`

## --- CHECKPOINT: PR merge ---

Stop here and present the PR URL. Tell the user to merge. Wait for the user to confirm.

## Phase 2: Release Tag & CI

1. `git checkout main && git pull origin main`
2. Create and push the release tag: `git tag v<NEW_VERSION> && git push origin v<NEW_VERSION>`
3. Tell the user that the release CI has been triggered
4. Wait 5 minutes (`sleep 300`), then check CI status: `gh run list --limit 1`
5. If CI is still running, wait another 2 minutes and check again. Repeat until completed or failed.
6. If CI failed, report the failure and stop. Do not proceed to Phase 3.
7. If CI succeeded, verify the release artifacts: `gh release view v<NEW_VERSION> --json tagName,assets -q '{tag: .tagName, assets: [.assets[].name]}'`
8. Generate release notes and update the release:
   1. Get the previous release tag: `git tag --sort=-v:refname | sed -n '2p'`
   2. Get commit log since the previous tag: `git log <PREV_TAG>..v<NEW_VERSION> --oneline`
   3. Compose release notes in the following format and update the release using `gh release edit v<NEW_VERSION> --notes "..."`:

```
## What's New

<Summarize new features, improvements, and bug fixes based on the commit log. Group by category if needed (e.g., Features, Fixes). Write concise, user-facing descriptions — not raw commit messages.>

## Install

### cargo (recommended)

\`\`\`
cargo install specre
\`\`\`

### Download prebuilt binaries

Prebuilt binaries are available from the **Assets** section below for the following platforms:

<List each artifact from step 7 as a bullet point with platform description, e.g.:>
- **Linux (x86_64):** `specre-x86_64-unknown-linux-gnu.tar.gz`
- **macOS (Apple Silicon):** `specre-aarch64-apple-darwin.tar.gz`
- **macOS (Intel):** `specre-x86_64-apple-darwin.tar.gz`
- **Windows (x86_64):** `specre-x86_64-pc-windows-msvc.zip`
```

## Phase 3: Publish to crates.io

1. Run `cargo publish --dry-run` to verify the package
2. Run `cargo publish` to publish to crates.io
3. Report the final result with links:
   - GitHub Release URL
   - crates.io URL
