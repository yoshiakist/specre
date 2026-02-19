[English](README.md) | [日本語](README.ja.md)

# specre

**AIフレンドリーな開発のための、生きている小さな仕様カード**

specre（スペクレ）は、仕様駆動開発（SDD: Spec-Driven Development）のための軽量な仕様フォーマットおよびツールキットです。各 specre は1つの振る舞いを記述する単一の Markdown ファイルであり、ライフサイクル管理およびエージェントの素早い検索のための機械可読な front matter を備えています。

## 課題

仕様書は、開発の意図を可視化し追跡可能にするために不可欠です。しかし実際には、仕様は腐敗します：

- **仕様とコードは乖離していく**: 実装が仕様から逸脱しても誰も気づきません。次の開発者（またはAI）が古い前提の上に構築するとき初めて問題になります。
- **巨大な仕様はAIのコンテキストを浪費する**: 大きな仕様書は、エージェントに単一の振る舞いを理解するためだけに機能全体を解析させ、今本当に関心のあるコードやテストに充てるべき有限のコンテキストウィンドウを消費します。
- **小さな変更は仕様化されない** 仕様を書くコストが高いと、新機能だけが文書化されます。バグ修正、リファクタリング、段階的な変更は仕様書に残りません。

specre は、仕様を書くコストを限りなくゼロに近づけることで、これらの課題を解決します。

## 設計思想

多くのSDDツール（GitHub Spec Kit、Amazon Kiro、BMAD）は、仕様を線形パイプラインに投入する大きな一枚岩のドキュメントとして扱います。一方、specre は正反対のアプローチを取ります：

- **1ファイル、1つの振る舞い**: 仕様の単位を極めて小さく保つ。AIエージェントが単一の振る舞いを理解するために機能全体を解析する必要があってはなりません。
- **コンテキストウィンドウを意識した設計**: LLMのコンテキストは有限のリソースです。specre は、テストファイルと実装と共に1回のセッションに快適に収まるサイズに設計されています。
- **セッション成果物ではなく、生きたドキュメント**: 各 specre は独自のライフサイクルステータスと最終検証日を持ちます。仕様はそれを作成したプロジェクト期間を超えて存続します。
- **プロセスに依存しないデータレイヤー**: specre は仕様の「フォーマット」を定義するものであり、「ワークフロー」を定義するものではありません。TDD、BDD、その他あらゆる開発プロセスと組み合わせて使用できます。
- **エンジン・ツールに依存しない**: YAML front matter 付きのプレーンな Markdown。IDE やモデルのロックインも、専用 CLI ツールもありません。

*Specificatio credibilis crescere potest.* が本プロジェクトのクレドです。

## クイックスタート

### ビルド済みバイナリのインストール

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.ps1 | iex"
```

### Cargo 経由のインストール（Rust ユーザー向け）

```bash
cargo install specre
```

[crates.io](https://crates.io/crates/specre) から `specre` CLI をインストールします。Rust 1.85+ が必要です。

### Git サブモジュールとして

```bash
git submodule add git@github.com:yoshiakist/specre.git specre
```

### 既存コードベースへの specre 導入

既にコードとテストが存在するプロジェクトに specre を導入する場合は、**[導入戦略ガイド](docs/guides/adoption-strategy.ja.md)** を参照してください。テストランドスケープの評価と適切な戦略の選択（テスト由来の抽出、コード振る舞い分析、トップダウンドメイン分解）について説明しています。

### 最初の specre を書く

プロジェクトの specres ディレクトリ配下に Markdown ファイルを作成します。ディレクトリ構成は自由なので、ドメイン、モジュール、機能領域など、プロジェクトに合った体系で整理可能です：

```
docs/specres/
  auth/
    signup/
      user_can_sign_up_with_email.md
      system_sends_verification_email_on_signup.md
    password/
      user_can_reset_password.md
  cart/
    user_can_add_item_to_cart.md
    cart_total_reflects_quantity_changes.md
```

ドメイン内のサブディレクトリは任意です。関連する振る舞いをグループ化するのに有用な場合に使い、1階層で十分な場合はフラットに保ちましょう。

## specre カードフォーマット

すべての specre は以下の構造に従います：

```markdown
---
id: "01HZYPMZRK8F9R2DGBGGMM2N8T"
name: "ユーザーはemailアドレスでサインアップできる"
status: "draft"
---

## 関連ファイル

- `src/auth/signup_controller.rb`
- `src/auth/email_validator.rb`
- `spec/auth/signup_controller_spec.rb` (Test)

## 機能概要

ユーザーが有効なメールアドレスとパスワードを入力してアカウントを作成できる。

## 意図

メールサインアップは主要なオンボーディングパスとなる。クライアント側でメール形式を、サーバー側で一意性を検証することで、素早いフィードバックを提供する。

## 主要なメンバー

- `email: String` — ユーザーのメールアドレス、RFC 5322 に準拠して検証
- `password_hash: String` — bcrypt ハッシュ、平文で保存されることはない

## シナリオ

### サインアップ成功

1. ユーザーが有効なメールアドレスと8文字以上のパスワードを送信する
2. システムがアカウントを作成し、`account_created` シグナルを発行する
3. ユーザーがウェルカム画面にリダイレクトされる

### 重複メールアドレス

1. ユーザーが既に存在するメールアドレスを送信する
2. システムがエラーを表示する: "既に登録されているメールアドレスです"
3. アカウントは作成されない

### 不正なメール形式

1. ユーザーが不正な形式のメール（例: "foo@"）を送信する
2. システムが送信前にバリデーションエラーを表示する
```

### front matter フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `id` | ULID | Yes | [ULID](https://github.com/ulid/spec) 形式の普遍的一意識別子。specre とソースコード間の双方向トレーサビリティのための単一キー。 |
| `name` | string | Yes | ファイル名（`.md` なし）と一致する。振る舞いを記述する明確な主語と述語を持つ文（例: `user_can_sign_up_with_email`、`system_rejects_invalid_email`）。`create_quotation` のような関数スタイルの名詞は避ける。 |
| `status` | enum | Yes | `draft` · `in-development` · `stable` · `deprecated` |
| `last_verified` | date | No | `YYYY-MM-DD` — この specre が実装と一致していることが最後に確認された日付。`stable` な specre に適用。`draft` や `in-development` では不要。 |

### ステータスライフサイクル

```
draft ──→ in-development ──→ stable ──→ deprecated
  ↑            │                │
  └────────────┘                │
  (要件変更)                     │
                                ↓
                           (置換または削除)
```

- **draft**: 振る舞いが提案されたが、まだ実装されていない。
- **in-development**: 実装またはテストが進行中。
- **stable**: 実装が specre と一致する。テストが通る。`last_verified` の日付で検証済み。
- **deprecated**: 振る舞いが削除されたか置き換えられた。履歴参照用に保持。

ステータスは specre の現在の状態を記録するものであり、このワークフローに従う必要はありません。チームは状態をスキップしたり、後戻りしたり、サブセットのみを採用することができます。つまり、specre は遷移を強制しません。

### 推奨セクション

| セクション | 必須 | 目的 |
|-----------|------|------|
| Related Files | Yes | specre をソースファイルとテストファイルにパスでリンクする（人間可読） |
| Functional Overview | Yes | 振る舞いの1段落要約 |
| Design Intent | No | *何を*ではなく*なぜ*を説明する |
| Key Members | No | 重要な状態とパラメータを自然言語で記述 |
| Scenarios | Yes | ステップバイステップの振る舞い記述 |
| Failures / Exceptions | No | エッジケースとエラーハンドリング |

### 命名規則

- **文として命名する。** すべての specre 名は、振る舞いを記述する明確な主語と述語を持たせることを強く推奨します： 
  - OK: `ユーザーはパスワードをリセットできる`、`システムは期限切れのトークンを拒否する`、`カート合計は数量変更を反映する`。
  - NG:`パスワードのリセット` や `期限切れトークンの判定` のような関数スタイルの名詞は避けてください。
- **連番を付けない。**
   - ファイル名に `001_`、`002_` などの接頭辞を付けないでください。specre の ULID が既に時系列順序を提供します。
   - 連番は管理オーバーヘッドを生み、コーディングエージェントとの摩擦を生じさせます。
- **グループ化にはサブディレクトリを使用する。**
  - ドメインに多くの関連する振る舞いが含まれる場合、番号付けの代わりにサブディレクトリでグループ化します。
  - 例: `見積/201_マネジャーは見積を承認できる.md` よりも `見積/承認/マネジャーは見積を承認できる.md` を推奨します。1階層で十分な場合はフラットに保ちましょう。

### 記述ガイドライン

- シナリオは**自然言語**で記述し、コードの模倣は避けます。実装の詳細をコピー＆ペーストしないでください。
- クラス、enum、シグナル名の正確な名前は **使用する** のがベストです。これらは仕様とコード間の契約です。
- 各 specre は**単一の振る舞いに焦点を当てて**ください。「また、...」と書いていることに気づいたら、別の specre に分割しましょう。
- specre は「Referenced Specifications」セクションで相対パスを使って相互参照できます。 ※ 機械可読な実装を検討中

## 双方向トレーサビリティ

specre は単一の ULID（specre の `id`） を使用して、仕様とソースコードを双方向にリンクします。中間的なタグレイヤーは不要です。

### 仕組み

```
┌─────────────────────────────┐
│  specre ファイル (.md)       │  ← 信頼できる唯一の情報源
│  ┌────────────────────────┐ │
│  │ id: ULID               │ │
│  │ name / status          │ │
│  │ last_verified          │ │
│  └────────────────────────┘ │
│  ## Related Files           │  ← パスベースの参照（人間可読）
│  ## Scenarios               │
└──────────────┬──────────────┘
               │ id (ULID)
               ▼
┌──────────────────────────────┐
│  ソースファイル               │
│  // @specre <ULID>           │  ← 逆参照（機械可読）
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│  index.json                  │  ← 生成キャッシュ（CLI管理）
│  specres[]: フロントマター    │
│  source_refs[]: マーカー索引  │
└──────────────────────────────┘
```

- **specre → ソース**: 「Related Files」セクションにファイルパスを記載。人間可読で、常に specre と同期。
- **ソース → specre**: ソースコメント内の `@specre` マーカーが specre の `id` を参照。機械可読で、CLI がスキャン可能。
- **index.json**: `specre index` コマンドで生成される派生成果物。いつでも再生成可能。手動編集は禁止。

### ソースファイルマーカー

ソースファイルを対応する specre にリンクするために、コメント内に `@specre` マーカーを配置します：

```ruby
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
class CreateQuotation < Usecase
  # ...
end
```

1つのファイルで複数の specre を参照できます：

```python
# @specre 01HZYPMZRK8F9R2DGBGGMM2N8T
# @specre 01HZYQ4N7XW3A8B5C6D9E0F1G2
class QuotationService:
    ...
```

マーカーはファイルの先頭、クラスや関数定義の上、または末尾など、関係性を最も分かりやすく伝える場所に配置できます。CLI はコメント構文を無視して `@specre [0-9A-Z]{26}` パターンをスキャンしてマーカーを検出します。

**言語別マーカー構文:**

| 言語 | マーカー |
|------|---------|
| Ruby / Python / GDScript / Shell | `# @specre 01HZYPM...` |
| JavaScript / TypeScript / Java / C# / C++ | `// @specre 01HZYPM...` |
| HTML / XML | `<!-- @specre 01HZYPM... -->` |
| CSS | `/* @specre 01HZYPM... */` |
| SQL | `-- @specre 01HZYPM...` |

※ 各種フレームワークやゲームエンジンなど、80を超える拡張子に対応済み

### なぜ ULID か

[ULID](https://github.com/ulid/spec)（Universally Unique Lexicographically Sortable Identifier）が UUID や連番の代わりに選ばれた理由：

- **作成時刻でソート可能**: 先に作成された specre が先にソートされ、ファイル名に依存しない自然な時系列順序を提供。
- **調整不要**: ULID は中央レジストリなしに、任意の開発者やエージェントが独立して生成可能。
- **コンパクト**: 26文字（UUID の36文字と比較）で、ソースコメント内のノイズを軽減。
- **ミリ秒内で単調増加**: 急速に連続作成された複数の specre の順序を維持。

## インデックスフォーマット

`specre index` コマンドは仕様ディレクトリとソースツリーをスキャンし、`specre_dir` 内に `index.json`（高速検索のための機械可読キャッシュ）を生成します。

```json
{
  "version": 1,
  "generated_at": "2026-03-01T12:00:00Z",
  "specres": [
    {
      "id": "01HZYPMZRK8F9R2DGBGGMM2N8T",
      "name": "ユーザーは見積を作成できる",
      "status": "draft",
      "domain": "見積",
      "path": "docs/specres/見積/作成/ユーザーは見積を作成できる.md",
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

### `specres` 配列

各エントリは specre ファイルの front matter をミラーし、2つの派生フィールドを追加します：

| フィールド | ソース | 説明 |
|-----------|--------|------|
| `id` | front matter | specre の ULID |
| `name` | front matter | 人間可読なタイトル |
| `status` | front matter | 現在のライフサイクルステータス |
| `domain` | ディレクトリ名 | specres ルート配下のトップレベルディレクトリから抽出（例: `docs/specres/見積/作成/ユーザーは見積を作成できる.md` → `"見積"`）。ドメイン内のサブディレクトリはドメイン値に影響しません。 |
| `path` | ファイルシステム | プロジェクトルートから specre ファイルへの相対パス |
| `last_verified` | front matter | 最終検証日 |

### `source_refs` 配列

各エントリはソースツリーで見つかった `@specre` マーカーを記録します：

| フィールド | 説明 |
|-----------|------|
| `specre_id` | マーカーが参照する ULID |
| `file` | ソースファイルへの相対パス |
| `line` | マーカーが見つかった行番号 |

### 設計原則

- **index.json はキャッシュであり、信頼できる情報源ではない。** 欠損や古い場合は `specre index` を実行して再生成。手動編集はしない。
- **specre ファイルが信頼できる情報源。** すべての正式なデータは各 `.md` ファイルのフロントマターと本文に存在します。
- **ディレクトリごとの _INDEX.md** も人間がブラウジングするために生成可能 — サブディレクトリ内のすべての specre を要約する Markdown テーブル。

## 他ツールとの比較

| | specre | プレーン Markdown | GitHub Spec Kit | Amazon Kiro | Gherkin |
|---|---|---|---|---|---|
| 粒度 | 1つの振る舞い | 様々 | 1つの機能 | 1つの機能 | 1つの機能（複数シナリオ） |
| 一意ID | specre ごとに ULID | なし | なし | なし | なし |
| ライフサイクルステータス | あり（4状態） | なし | なし | なし | なし |
| 検証日 | あり | なし | なし | なし | なし |
| コードトレーサビリティ | 双方向（ULID） | なし | なし | なし | ステップ定義 |
| テスト統合 | 慣例による | なし | 任意 | 任意 | 実行可能 |
| プロセス結合 | なし | なし | 線形パイプライン | 線形パイプライン | テストランナー |
| 仕様あたりのファイル数 | 1 | 様々 | 3-4 | 3 | 1 |
| IDE 依存 | なし | なし | なし | Kiro IDE | なし |
| インデックス生成 | CLI（JSON + Markdown） | なし | なし | 組み込み | なし |

## ロードマップ

> 詳細: [docs/project/ROADMAP.ja.md](docs/project/ROADMAP.ja.md)

- **v0.1 — コア CLI** ✅ `init`, `new`, `index`, `status`
- **v0.2 — トレーサビリティ** ✅ `trace`, `orphans`, `tag`
- **v0.3 — エージェント統合** ✅ `coverage`, `health-check`, `search`, `--json` 出力, MCP サーバー
- **v0.4 — ドリフト検出** — `drift`, `ci`, GitHub Actions テンプレート
- **v0.5 — QA サポート** — `impact`, `diff`, `export`

## コントリビューション

specre は初期段階にあります。コントリビューション、フィードバック、実際の使用レポートを歓迎します。GitHub で Issue または Pull Request を開いてください。

Rust コードをコントリビュートする場合は、Pull Request を提出する前に **[Rust Conventions](docs/guides/RUST-CONVENTIONS.md)** をお読みください。

## ライセンス

MIT

AIフレンドリーな開発のための、生きている小さな仕様カード。軽量で、依存無く、追跡可能です。
