# ロードマップ

test

## v0.1 — コア CLI ✅

- [x] `specre init` — プロジェクトに specre を初期化し、specre ディレクトリと設定ファイルを作成できる
- [x] `specre new` — テンプレートから新しい specre をスキャフォールドし、`id` フィールドに ULID を自動生成できる
- [x] `specre index` — specre ディレクトリとソースツリーをスキャンし、`index.json` とドメインごとの `_INDEX.md` を生成できる
- [x] `specre status` — ステータスごとの specre 数を報告し、古い `last_verified` の日付をフラグ立てできる

## v0.2 — トレーサビリティ ✅

- [x] `specre trace <ULID>` — ULID を指定して、specre ファイルとそれを参照するすべてのソースファイルを表示（逆方向も可）
- [x] `specre orphans` — ソース内に `@specre` マーカーがない specre、またはマッチする specre がないマーカーを検出できる
- [x] `specre tag <ULID> <file>` — ソースファイルの適切な位置に `@specre` マーカーを挿入できる

## v0.3 — エージェント統合

AIエージェントが specre をファーストクラスのツールとして活用できるようにする。

- [x] `specre coverage` — specre タグでカバーされているソースファイルの割合を報告できる
- [x] `specre health-check` — specre カードがプロジェクトの全体的な振る舞いを適切に記述しているかどうかを判定する包括的なヘルスチェックができる
- [x] `specre search <query>` — すべての specre に対する全文検索 + ステータス/ドメインフィルタリングができる
- [x] すべてのコマンドでAIフレンドリーなフォーマット（`--json`）で出力できる
- [ ] MCP サーバー — specre の機能を [Model Context Protocol](https://modelcontextprotocol.io/) 経由でリソース、ツール、プロンプトとして公開し、Claude Code、Cursor、VS Code Copilot、その他の MCP 対応 AI ツールとの統合を可能にする

### coverage コマンド設計

カバレッジは、`@specre` タグを介して specre カードにリンクされているソースツリーの範囲を測定します。

- **分母:** 設定された `source_dir` 内のファイル総数
- **分子:** `source_dir` 内で少なくとも1つの `@specre` タグを含むファイル数
- 対象ファイル拡張子によるフィルタリングをサポート（例: `--ext rs,ts` で `.rs` と `.ts` ファイルのみをカウント）

### health-check コマンド設計

health-check は、コーディングエージェントがタスク開始前に specre エコシステムが信頼できるかどうかを検証するための単一エントリーポイントです。エージェントがセッション開始時に最初に実行するコマンド、または MCP サーバーのクエリとして設計されています。

カバレッジ、孤立数、インデックスの鮮度を1つのレスポンスに集約することで、コーディングエージェントが specre カードと specre コマンドに依拠できるかどうかを「エージェントが個別に複数コマンドを解釈する必要なく」明確に判定できます。

構造化 JSON を返します：

```json
{
  "healthy": true,
  "coverage": 0.93,
  "orphans": 2,
  "index_age_hours": 3.2,
  "thresholds": { "coverage": 0.90, "orphans": 5, "index_age_hours": 24 }
}
```

- `healthy` はすべてのメトリクスが閾値内の場合に `true`。
- `thresholds` は `specre.toml` で設定可能。上記の値はデフォルト。

### MCP サーバー設計

MCP サーバーは、機能を再実装するのではなく、既存の CLI ロジックを薄いレイヤーとしてラップします。

| MCP プリミティブ | 公開するもの |
|----------------|-------------|
| **Resources** | `specre:///<ULID>` URI として specre カードを公開。エージェントはオンデマンドで個別の specre カードを読み取り可能。 |
| **Tools** | `new`、`search`、`trace`、`orphans`、`status`、`index`、`health-check`、`coverage` — CLI と同じ操作で、構造化 JSON を返す。 |
| **Prompts** | SDD ワークフローテンプレート（例:「specre カードから振る舞いを実装する」）と QA 指向のプロンプト（`review-qa`、`summarize-diff`）で、一貫したエージェント駆動開発を実現。 |

トランスポート: stdio（プライマリ）、将来的にリモートユースケース向けに SSE/HTTP 追加のオプションを検討。

### QA 向け MCP プロンプト

MCP サーバーには QA エンジニア向けに設計されたプロンプトが含まれており、実装コードを読まずに仕様レベルの品質保証に AI を活用できます。

| プロンプト | 目的 |
|----------|------|
| `review-qa` | specre カードを分析し、見落とされた可能性のあるエッジケース、境界条件、Failures / Exceptions を提案。 |
| `summarize-diff` | 前回の stable バージョンと現在の in-development バージョン間の変更を意味的に要約し、リグレッションテストのスコープを提案。 |

これらのプロンプトは specre をエンジン非依存に保ちます — AI の推論はエージェントが接続している LLM によって実行され、specre 自体によるものではありません。

## v0.4 — ドリフト（乖離）検出

- [ ] `specre drift` — 関連ファイルの git 履歴に対して `last_verified` の日付を比較し、最後の検証以降にソースが変更された specre をフラグ付けできる
- [ ] `specre ci` — ドリフトまたは孤立が検出された場合に非ゼロステータスで終了（CI 統合用）
- [ ] GitHub Actions ワークフローテンプレートを利用できる

## v0.5 — QA サポート

QA エンジニアが specre カードと直接作業するための決定論的な CLI コマンド（LLM 不要）

- [ ] `specre impact <ULID>` — 推移的な依存関係と影響分析。関連仕様セクションの相互参照と `@specre` マーカーを走査して依存グラフを構築し、変更によって影響を受ける specre とソースファイルを表示できる。
- [ ] `specre diff [specre-path]` — git 履歴を使用して、最後の `stable` 状態以降の specre カードの変更を表示。`specre drift`（*何か*が変更されたかを検出）を補完し、*何が*変更されたかを表示できる。
- [ ] `specre export [--format <fmt>]` — Scenarios セクションを構造化されたテストケースフォーマット（Markdown チェックリスト、CSV）に変換し、テスト管理ツールへのインポートを可能に。仕様からテストケースへの手動転写を排除できる。

## 将来の検討事項

- カスタム front matter フィールド（`type`、`tags` など）のプラグインシステム（オプションのバリデーション付き） — プロジェクト定義の語彙で `specre search --tag "quotation edit"` のような検索を可能に
- specre 間の相互参照からの Mermaid ダイアグラム生成
- 依存グラフの可視化
- specre コンテンツの多言語サポート（i18n メタデータ） ※ 現在は英語、日本語のみ
