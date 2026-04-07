# CI インテグレーションガイド

specre の health-check を CI パイプラインに組み込み、仕様のドリフト、カバレッジ低下、孤立カードをメインブランチへのマージ前に検知する方法を説明します。

## なぜ CI に組み込むのか？

CI による強制がなければ、specre エコシステムは静かに劣化します：

- 開発者が新しいソースファイルを追加したが `@specre` マーカーを忘れた — カバレッジが低下する
- リファクタリングでコードを移動したが specre カードの Related Files を更新しなかった — オーファンが発生する
- 機能変更で振る舞いが変わったが `last_verified` が更新されなかった — ドリフトが蓄積する

`specre health-check` は、これらすべてを1コマンドで検出します。設定された閾値を下回ると終了コード 1 を返すため、CI ゲートとして最適です。

## 前提条件

- `specre.toml` が設定済みのプロジェクト（[specre を始める](START-SPECRE.ja.md)を参照）
- CI 環境で Git 履歴が利用可能であること（ドリフト検出に必要）

---

## GitHub Actions

### 基本: Health-Check ゲート

最もシンプルな統合 — プルリクエスト時に `specre health-check` を必須チェックとして実行します。

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
          fetch-depth: 0  # ドリフト検出にフル履歴が必要

      - name: Install specre
        run: curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

      - name: Regenerate index
        run: specre index

      - name: Run health-check
        run: specre health-check
```

**ポイント:**

- `fetch-depth: 0` — ドリフト検出には完全な Git 履歴が必要です。これがないと、`specre drift` は `last_verified` の日付とファイルの変更履歴を比較できません。
- `health-check` の前に `specre index` — インデックスが最新でないと正確な結果が得られません。CI で再生成することで `index_age_hours` チェックが常に通過します。
- 失敗時の終了コード 1 — `specre health-check` はいずれかのメトリクスが不健全なときにゼロ以外の終了コードを返し、GitHub Actions のステップを自動的に失敗させます。

### 応用: ドリフトレポートを PR にコメント

ドリフトの詳細を PR コメントとして投稿するステップを追加し、レビュアーがどの specre カードに注意が必要かを簡単に確認できるようにします。

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

            // 既存コメントを更新、なければ新規作成
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

### 警告のみモード（非推奨）

チームがまだ specre の health-check をハードゲートとして強制する準備ができていない場合、失敗ではなく警告として扱うこともできます。ワークフローは実行され結果をレポートしますが、プルリクエストのマージはブロックしません。

> **これは推奨しません。** 強制がなければ、カバレッジ低下やドリフトを検知するフィードバックループが存在しないため、specre エコシステムは時間とともに劣化します。specre 導入のランプアップ期間中の過渡的な措置としてのみ使用し、強制ゲートに切り替える具体的な期限を設定してください。

これを行うには、health-check ステップに `continue-on-error: true` を追加します：

```yaml
      - name: Run health-check
        run: specre health-check
        continue-on-error: true
```

`ci-gate` パターン（このプロジェクトで使用しているもの）では、specre ジョブを失敗条件から除外します：

```yaml
  ci-gate:
    needs: [test, clippy, fmt, specre]
    # ...
    steps:
      - name: Check results
        run: |
          # specre は意図的に除外 — 警告のみ
          if [[ "${{ needs.test.result }}" == "failure" || \
                "${{ needs.clippy.result }}" == "failure" || \
                "${{ needs.fmt.result }}" == "failure" ]]; then
            echo "CI failed"
            exit 1
          fi
```

specre ジョブは PR のチェックリストに赤または緑のステータスで表示されるため、ブロックせずに可視性を確保できます。

---

## CI 用の閾値設定

CI の閾値は `specre.toml` で設定します。プロジェクトの成熟度に合わせて調整してください：

```toml
[health_check]
coverage = 0.30        # 低く始め、カバレッジの成長に応じて引き上げる
orphans = 10           # 許容するオーファンの数
index_age_hours = 48   # CI では重要ではない（インデックスは再生成される）
drifts = 0             # 許容するドリフト数（grace 適用後）

[drift]
grace_days = 7         # この期間内の変更はドリフトとしてカウントしない
```

### 推奨される段階的な引き上げ

| フェーズ | Coverage | Orphans | Drifts | Grace |
|---------|----------|---------|--------|-------|
| 導入初期 | `0.30` | `10` | `3` | `7` |
| エコシステム成長期 | `0.60` | `5` | `0` | `7` |
| 成熟期 | `0.80` | `0` | `0` | `3` |

緩い閾値から始め、specre エコシステムの成熟に応じて引き締めてください。最初から厳しい閾値を設定すると摩擦が生まれ、導入を妨げます。

---

## 他の CI プラットフォーム

プラットフォームに関係なくパターンは同じです — specre をインストールし、インデックスを再生成し、health-check を実行します。

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

## トラブルシューティング

### `specre: command not found`

インストーラはバイナリを `~/.cargo/bin/` に配置します。CI 環境がこのパスを `$PATH` に含んでいない場合、明示的に追加してください：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### ドリフト検出で結果が出ない

ドリフト検出には完全な Git 履歴が必要です。チェックアウトステップで完全な履歴を取得していることを確認してください：

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
```

### インデックスの age チェックが常に失敗する

`index.json` を git にコミットしていない場合（推奨）、CI ではインデックスが存在しません。`specre health-check` の前に必ず `specre index` を実行してください。

### ローカルでは通るが CI で失敗する

通常、`index.json` がコミットされていて古くなっていることが原因です。以下のいずれかを検討してください：
- `index.json` を `.gitignore` に追加し、CI で再生成する（推奨）
- コミット前にローカルで `specre index` を実行する
