# ロードマップ

## v0.5 — ドリフト（乖離）検出

specre カードと実装コードの乖離を検知し、検知後の対応パスを提供する。検知だけでなく、偽陽性の効率的な処理と真のドリフトへの対応手段をセットで設計する。

- [x] `specre drift` — 関連ファイルの git 履歴に対して `last_verified` の日付を比較し、最後の検証以降にソースが変更された specre を報告できる
- [x] `specre verify` — 確認済みの specre の `last_verified` を今日の日付に一括更新できる（偽陽性の高速パス）
- [ ] `specre reopen` — 真のドリフトが見つかった specre を `stable` から `in-development` に戻せる
- [ ] `specre ci` — ドリフトまたは孤立が検出された場合に非ゼロステータスで終了（CI 統合用）
- [ ] `specre init` 拡張 — 生成される `specre.toml` に `[drift] grace_days = 7` をコメントアウトで含める
- [ ] `specre health-check` 拡張 — `drifted` 数をヘルスチェック報告に追加（grace 適用済み）
- [ ] GitHub Actions ワークフローテンプレートを利用できる

### drift コマンド設計

#### 設計原則: 報告のみ、front-matter は書き換えない

ドリフトは `last_verified` と git 履歴から常に導出可能な状態であり、specre カード自身の属性ではない。`index.json` が「導出キャッシュ」であるのと同様に、ドリフト情報はコマンド実行時に算出して報告する。front-matter に `drifted: true` のようなフラグを書き込まない。

#### 検知ロジック

```
drifted = file_last_modified > (last_verified + grace_period)
```

- specre カードの `last_verified` と、Related Files および `@specre` マーカーで紐づくソースファイルの git 上の最終変更日を比較する
- grace 期間内の変更はドリフトとして報告しない（後述）
- 対象はデフォルトで `stable` ステータスの specre のみ。`draft` や `in-development` は実装と一致していないことが前提であり、ドリフトという概念が当てはまらない

#### インターフェース

```
specre drift [target] [--grace <duration>] [--status <status>] [--domain <name>] [--json]
```

| 引数/オプション | 説明 |
|----------------|------|
| （なし） | プロジェクト全体のドリフトを報告 |
| `<ULID>` | 単一 specre のドリフトを確認 |
| `<path>` | 指定パス配下の specre のみを対象 |
| `--domain <name>` | ドメイン名でフィルタ |
| `--status <status>` | 対象ステータスを指定（デフォルト: `stable`） |
| `--grace <duration>` | バッファ期間（例: `3d`, `7d`, `30d`）。デフォルトは `specre.toml` の設定値、未設定なら `0`（即時検知） |
| `--json` | JSON 形式で出力 |

#### grace（猶予期間）

翌日に行った変更は同一作業者によるものであることが多く、仕様書も実態に合わせて修正されている可能性が高い。一方、1ヶ月以上後に行った変更は、仕様を意識せずに行われドリフトを招いている可能性が高い。grace はこの現実を反映するバッファである。

`specre.toml` でプロジェクトごとのデフォルトを設定可能:

```toml
[drift]
grace_days = 7
```

`--grace` オプションが明示された場合はそちらが優先される。用途別の目安:

| 場面 | grace |
|------|-------|
| CI（厳格） | `0` または `1d` |
| 開発中のチェック | `3d`〜`7d` |
| 長期メンテナンス棚卸し | `30d`（明らかなドリフトのみ） |
| AI エージェントのプリフライト | `7d` |

#### JSON 出力

```json
{
  "drifted": [
    {
      "id": "01HZYPMZRK...",
      "name": "user_can_sign_up_with_email",
      "path": "docs/specres/auth/user_can_sign_up_with_email.md",
      "domain": "auth",
      "last_verified": "2026-03-01",
      "changed_files": [
        { "file": "src/auth/signup.rs", "last_modified": "2026-04-05", "diff_stat": "+12 -3" }
      ]
    }
  ],
  "clean": 42,
  "total": 45,
  "grace_days": 7
}
```

- `changed_files` には `diff_stat`（`+N -M`）を含める。変更内容の詳細判断はエージェントが `git diff` を読んで行う
- `changed_functions` のような言語依存のヒューリスティックは含めない（言語非依存の原則）

#### 集約ファイル問題と対応パス

意図的な集約パターン（`mod.rs`、`router.rs` など）のソースファイルには10個以上の `@specre` タグが付与されることがある。このようなファイルに1行でも変更が入ると、紐づくすべての specre がドリフトとして報告される。大半は偽陽性である。

この問題に対して、検知精度を上げるのではなく、**検知後の対応コストを下げる**アプローチを取る。具体的には `specre verify`（偽陽性の一括処理）と `specre reopen`（真のドリフトへの対応）を提供する。

| 検知結果 | 対応 | コマンド |
|---------|------|---------|
| ドリフトなし（偽陽性） | `last_verified` を更新するだけ | `specre verify` |
| ドリフトあり、仕様は正しい | 実装を仕様に合わせる | 既存の SDD ワークフロー |
| ドリフトあり、仕様が古い | specre を `in-development` に戻して仕様を更新 | `specre reopen` |

### verify コマンド設計

ドリフトとして報告されたが、確認の結果仕様と実装に乖離がない場合に、`last_verified` を今日の日付に更新する。

```
specre verify <ULID>...                    # 複数 ULID を一括指定
specre verify --domain <name>              # ドメイン単位
specre verify --file <source-file-path>    # このファイルに紐づく全 specre を一括更新
```

`--file` オプションは集約ファイル問題への直接的な対策。`src/commands/mod.rs` の変更で12個の specre がフラグされた場合、確認後に `specre verify --file src/commands/mod.rs` で一括更新できる。

### reopen コマンド設計

真のドリフトが見つかった場合に、specre のステータスを `stable` から `in-development` に戻す。

```
specre reopen <ULID>
```

- `status` を `in-development` に変更し、`last_verified` はそのまま残す（最後に検証した日付としての履歴的価値がある）
- `stable` 以外のステータスに対して実行した場合はエラー

### health-check 拡張

`drifted` 数をヘルスチェック報告に追加する。grace 期間を適用した上でカウントする。

```json
{
  "healthy": false,
  "coverage": 0.93,
  "orphans": 2,
  "drifted": 3,
  "index_age_hours": 1.2,
  "thresholds": { "coverage": 0.90, "orphans": 5, "index_age_hours": 24, "drifted": 0 }
}
```

### AI エージェント向けトリアージワークフロー

drift 検知後の AI エージェントによるトリアージの想定フロー:

1. `specre drift --json` でドリフト一覧を取得
2. drifted ごとに:
   - `git diff <last_verified>..HEAD -- <changed_files>` で変更内容を確認
   - specre カードの Scenarios を読む
   - 判定: 偽陽性 / 仕様が正しい（実装バグ） / 仕様が古い
3. 偽陽性 → `specre verify <ULID>`
4. 仕様が正しい → 実装を修正（既存 SDD ワークフロー）
5. 仕様が古い → `specre reopen <ULID>` → カードを更新 → 実装を確認

## v0.6 — QA サポート

QA エンジニアが specre カードと直接作業するための決定論的な CLI コマンド（LLM 不要）

- [ ] `specre impact <ULID>` — 推移的な依存関係と影響分析。関連仕様セクションの相互参照と `@specre` マーカーを走査して依存グラフを構築し、変更によって影響を受ける specre とソースファイルを表示できる。
- [ ] `specre diff [specre-path]` — git 履歴を使用して、最後の `stable` 状態以降の specre カードの変更を表示。`specre drift`（*何か*が変更されたかを検出）を補完し、*何が*変更されたかを表示できる。
- [ ] `specre export [--format <fmt>]` — Scenarios セクションを構造化されたテストケースフォーマット（Markdown チェックリスト、CSV）に変換し、テスト管理ツールへのインポートを可能に。仕様からテストケースへの手動転写を排除できる。

## v0.7 — マルチリポジトリ・トレーサビリティ

リポジトリ境界を越えた specre のトレーサビリティを実現する。ポリレポ構成のマイクロサービス、フロントエンド/バックエンド分離、イベント駆動アーキテクチャに対応。

### 設計原則: Provider が所有し、Consumer が外部参照する

サービス境界をまたぐ振る舞い（API コントラクト、イベントスキーマ、共有 DTO）には自然な所有権モデルがある。**提供側（Provider）が specre カードを所有し、消費側（Consumer）がそれを外部参照する。** これにより重複を避け、各コントラクトの唯一の情報源を確立する。

```
orders-service (Provider)              frontend-app (Consumer)
┌─────────────────────────────────┐   ┌──────────────────────────────────┐
│ docs/specres/api/               │   │ src/api/orders.ts                │
│   order_api_returns_order_dto.md│   │   // @specre-ext 01XYZ... orders │
│   (id: 01XYZ..., status: stable)│   │   interface OrderDto { ... }     │
│                                 │   │                                  │
│ src/handlers/orders.rs          │   │                                  │
│   // @specre 01XYZ...           │   │                                  │
└─────────────────────────────────┘   └──────────────────────────────────┘
```

### 設計原則: 宣言と解決を分離する

マルチリポジトリ設定には、混同してはならない2つの関心事がある:

- **宣言**（どのリモートが存在するか） — git にコミットし、チーム全体で共有
- **解決**（リモートがこのマシンのどこにあるか） — ローカル、個人管理、git で追跡しない

この分離は実際のチーム開発で極めて重要である。20人規模のプロダクトチームが4つのスクラムチームで運営される場合、サービス間連携に携わるのは通常1チームのみ。他のチームは関連リポジトリをチェックアウトすらしていないことがある。未解決の外部参照がプロジェクトを「unhealthy」にしてしまうと、大半の開発者が常に赤いステータスを見ることになり、health-check の存在意義が根本から崩れる。

**`specre.toml`（コミット対象）:**

```toml
[remotes.orders-api]
git = "https://github.com/org/orders-service.git"
specre_dir = "docs/specres"
```

**`.specre.local.toml`（`.gitignore` 対象、個人管理）:**

```toml
[remotes.orders-api]
path = "../orders-service"
```

### 設計原則: 未解決の外部参照は unhealthy ではない

`specre health-check` は**ローカル**の specre エコシステムのみを判定する。外部参照の解決状況は別セクションで報告されるが、`healthy` フラグには影響しない。

```
Local ecosystem:
  healthy: true

External references:
  @specre-ext markers: 3
  resolved: 1 / unresolved: 2
```

### 計画

- [ ] `@specre-ext <ULID> [origin]` マーカー — `@specre` とは明確に区別される新しいマーカー型。参照先の仕様が別プロジェクトに存在することを示す。origin ヒントは省略可能だが、解決速度の向上とドキュメントとしての価値がある
- [ ] `specre.toml [remotes]` セクション — 正規の git URL でリモート specre ソースを宣言。ローカルパスはここに書かない（`.specre.local.toml` に記述）
- [ ] `.specre.local.toml` サポート — 個人管理の gitignore 対象ファイル。リモートのローカルパス解決を提供
- [ ] `specre trace` 拡張 — 設定済みリモート経由で `@specre-ext` マーカーを解決し、出力に `(ext)` 注釈を表示
- [ ] `specre orphans` 拡張 — リモート未設定の `@specre-ext` マーカーは情報表示扱い（エラーではない）
- [ ] `specre coverage` 拡張 — `@specre-ext` マーカーもカバレッジとしてカウント
- [ ] `specre health-check` 拡張 — 外部参照を別セクションで報告。未解決の外部参照は `healthy` に影響しない

## v0.8 — リモート解決とサービス境界管理

v0.6 のマルチリポジトリ基盤の上に、ネットワーク経由の解決とクロスリポジトリのコントラクト管理ツールを構築する。

- [ ] `specre fetch [remote-name | --all]` — リポジトリ全体をクローンせずにリモートの specre ディレクトリを取得（sparse-checkout または GitHub API）。`.specre-cache/`（gitignore 対象、ローカル）に保存
- [ ] `specre fetch --status` — 全宣言済みリモートのキャッシュ鮮度を報告
- [ ] `specre boundary` — プロジェクト全体の外部参照一覧、解決状況、依存するリモートを表示
- [ ] `specre boundary --check` — クロスリポジトリのコントラクト健全性の明示的な検証（インテグレーションチームおよび CI 向け）
- [ ] `[remotes]` git 解決 — `.specre.local.toml` のパスが未設定の場合、git URL から直接リモート specre カードを解決（`.specre-cache/` を使用）
- [ ] クロスリポジトリ `specre drift` — ローカルコードが依存するリモート specre カードの `last_verified` 鮮度を検証

## 将来の検討事項

- カスタム front matter フィールド（`type`、`tags` など）のプラグインシステム（オプションのバリデーション付き） — プロジェクト定義の語彙で `specre search --tag "quotation edit"` のような検索を可能に
- specre 間の相互参照からの Mermaid ダイアグラム生成
- 依存グラフの可視化
- specre コンテンツの多言語サポート（i18n メタデータ） ※ 現在は英語、日本語のみ

---

<details>
<summary>完了済みバージョン (v0.1〜v0.4)</summary>

## v0.1 — コア CLI ✅

- [x] `specre init` — プロジェクトに specre を初期化し、specre ディレクトリと設定ファイルを作成できる
- [x] `specre new` — テンプレートから新しい specre をスキャフォールドし、`id` フィールドに ULID を自動生成できる
- [x] `specre index` — specre ディレクトリとソースツリーをスキャンし、`index.json` とドメインごとの `_INDEX.md` を生成できる
- [x] `specre status` — ステータスごとの specre 数を報告し、古い `last_verified` の日付をフラグ立てできる

## v0.2 — トレーサビリティ ✅

- [x] `specre trace <ULID>` — ULID を指定して、specre ファイルとそれを参照するすべてのソースファイルを表示（逆方向も可）
- [x] `specre orphans` — ソース内に `@specre` マーカーがない specre、またはマッチする specre がないマーカーを検出できる
- [x] `specre tag <ULID> <file>` — ソースファイルの適切な位置に `@specre` マーカーを挿入できる

## v0.3 — エージェント統合 ✅

AIエージェントが specre をファーストクラスのツールとして活用できるようにする。

- [x] `specre coverage` — specre タグでカバーされているソースファイルの割合を報告できる
- [x] `specre health-check` — specre カードがプロジェクトの全体的な振る舞いを適切に記述しているかどうかを判定する包括的なヘルスチェックができる
- [x] `specre search <query>` — すべての specre に対する全文検索 + ステータス/ドメインフィルタリングができる
- [x] すべてのコマンドでAIフレンドリーなフォーマット（`--json`）で出力できる
- [x] MCP サーバー — specre の機能を [Model Context Protocol](https://modelcontextprotocol.io/) 経由でリソース、ツール、プロンプトとして公開し、Claude Code、Cursor、VS Code Copilot、その他の MCP 対応 AI ツールとの統合を可能にする

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

## v0.4 — コマンド利便性向上

既存コマンドの使い勝手を改善するパッチ群（v0.3.x として出荷済み）。

- [x] `specre destroy` — ソースファイルから `@specre` マーカーを除去し、specre 削除時のトレーサビリティリンクを綺麗に掃除できる
- [x] `specre init` — 生成される `specre.toml` にデフォルトオプション（`exclude_patterns`、health-check 閾値など）をコメントアウトで含めることで、設定項目の発見コストを下げられる
- [x] `specre.toml` の `exclude_patterns` — `vendor/` や生成ファイルなどをソーススキャンから除外できる

</details>
