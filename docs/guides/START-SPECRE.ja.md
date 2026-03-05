# specre を始める

specre をプロジェクトに導入し、AI エージェントと共に仕様駆動開発（SDD）を始めるためのステップバイステップガイドです。

このガイドを最後まで進めると、以下の状態になります：

- specre CLI がインストールされ、使える状態
- プロジェクトに specre が初期化され、設定が完了
- `glossary.toml` がプロジェクトのドメイン語彙に最適化されている
- AI エディタから MCP 経由で specre にアクセスし、エージェントがコード探索の第一手段として specre を活用する状態
- 基底プロンプト（CLAUDE.md 等）にエージェントの行動指針が記載されている
- `/specre-whats-next` コマンドで、次にやるべきことを AI が提案してくれる状態

## 前提条件

- Git 管理されたプロジェクトがあること
- AI コーディングエディタ（Claude Code、Cursor、Windsurf 等）を使用していること

---

## ステップ 1: specre のインストール

### ビルド済みバイナリ（推奨）

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/yoshiakist/specre/releases/latest/download/specre-installer.ps1 | iex"
```

### Cargo 経由（Rust ユーザー向け）

```bash
cargo install specre
```

Rust 1.85 以上が必要です。

### インストールの確認

```bash
specre --version
```

バージョン番号が表示されれば成功です。`command not found` と表示される場合は、次のステップで PATH を確認してください。

---

## ステップ 2: PATH の確認

`specre --version` が正常に動作した場合、このステップはスキップしてください。

### Linux / macOS

ビルド済みバイナリのインストーラを使用した場合、PATH は自動設定されます。

Cargo 経由でインストールした場合、`~/.cargo/bin` が PATH に含まれている必要があります：

```bash
echo $PATH | tr ':' '\n' | grep cargo
```

表示がない場合、シェルの設定ファイルに追加します：

```bash
# ~/.bashrc, ~/.zshrc, または ~/.profile に追加
export PATH="$HOME/.cargo/bin:$PATH"
```

変更後、シェルを再起動するか `source ~/.bashrc`（または該当ファイル）で反映してください。

### Windows

ビルド済みバイナリのインストーラを使用した場合、PATH は自動設定されます。

Cargo 経由でインストールした場合、`%USERPROFILE%\.cargo\bin` が PATH に含まれている必要があります：

1. 「システム環境変数の編集」を開く
2. 「環境変数」→ ユーザー変数の「Path」→「編集」
3. `%USERPROFILE%\.cargo\bin` が含まれていなければ「新規」で追加
4. ターミナルを再起動

---

## ステップ 3: プロジェクトの初期化

プロジェクトのルートディレクトリで `specre init` を実行します：

```bash
specre init
```

以下のファイルとディレクトリが作成されます：

| ファイル | 説明 |
|---------|------|
| `specre.toml` | specre の設定ファイル |
| `glossary.toml` | 検索ヒント用のプロジェクト語彙定義 |
| `docs/specres/` | specre カードを格納するディレクトリ |

この時点では設定はデフォルト値のままです。次のステップで手動調整します。

---

## ステップ 4: specre.toml の設定

`specre init` で生成された `specre.toml` を開き、プロジェクトに合わせて手動で編集します。

```toml
# specre カードの格納場所
specre_dir = "docs/specres"

# @specre マーカーをスキャンするディレクトリ
# ⚠ 最初は1つのドメインに絞ること（理由は後述）
source_dirs = ["src/auth", "tests/auth"]

# 対象ファイル拡張子（省略時は一般的な拡張子を自動検出）
ext = ["rb", "js", "ts"]

# スキャンから除外するファイルパターン（任意）
# exclude_patterns = [".stories.tsx", "**/dist"]

# specre カードの言語（省略時は "en"）
# 現在は "en" と "ja" に対応
language = "ja"

# health-check の閾値
[health_check]
coverage = 0.30        # カバレッジ閾値（0.0〜1.0）
orphans = 10           # 許容する「ソースと紐づかない仕様カード」の数
index_age_hours = 48   # インデックスの有効期間（h）
```

### 各設定項目の解説

#### `source_dirs` — スコープの定義

specre が管理対象とするソースファイルの範囲です。ここで指定したディレクトリ内のファイルに対して、カバレッジ計算やマーカースキャンが行われます。

**最初は必ず1つのドメインに限定してください。** これには2つの重要な理由があります：

1. **進捗の実感**: スコープが広すぎるとカバレッジが常に低く、progress の実感が得られず、導入が頓挫しやすくなります。
2. **エージェントの信頼判定**: カバレッジが低いと `health-check` が `unhealthy` を返し、コーディングエージェントは **specre エコシステムを信頼しなくなります**。その結果、ターゲットドメインであっても `specre search` や `specre trace` による仕様と意図の効率的な逆引きを行わず、従来通りの grep やファイル探索にフォールバックしてしまいます。せっかく specre カードを書いても、エージェントがそれを活用しないのでは意味がありません。

1つのドメインのカバレッジを高めてから、段階的にスコープを拡大してください。

```toml
# 良い例：1つのドメインに集中
source_dirs = ["src/auth", "tests/auth"]

# 避けるべき例：最初からプロジェクト全体
source_dirs = ["src", "tests"]
```

#### `ext` — 対象ファイル拡張子

省略すると specre が一般的なプログラミング言語の拡張子を自動検出します。明示的に指定することで、画像ファイルや設定ファイルなど不要なファイルのスキャンを避けられます。

```toml
# Rust プロジェクト
ext = ["rs"]

# Web アプリケーション
ext = ["ts", "tsx", "js", "jsx"]

# Ruby on Rails
ext = ["rb", "erb"]
```

#### `exclude_patterns` — スキャン除外パターン

ソーススキャンから特定ファイルを除外するパターンです。テストフィクスチャ、生成ファイル、specre で追跡したくないファイルを除外する際に使います。

各パターンはファイルパスの部分文字列として照合されます。`*` を含む場合はグロブパターンとして扱われます：

```toml
# Storybook のストーリーファイルとビルド出力を除外
exclude_patterns = [".stories.tsx", "**/dist"]
```

省略すると、`source_dirs` 内の対象拡張子に一致するすべてのファイルがスキャンされます。

#### `language` — 言語設定

specre カードテンプレートのセクション見出しの言語を設定します。

- `"en"`（デフォルト）: `## Related Files`, `## Functional Overview`, `## Scenarios`
- `"ja"`: `## 関連ファイル`, `## 機能概要`, `## シナリオ`

また、`/specre-whats-next` の診断結果やレコメンデーションもこの設定に従って日本語で出力されます。

#### `[health_check]` — 健全性判定の閾値

`specre health-check` コマンドがエコシステムの健全性を判定する際の閾値です。

| 項目 | 説明 | 導入初期の推奨値 |
|------|------|-----------------|
| `coverage` | specre カバレッジの最低ライン。`0.3` = 30% | `0.30` |
| `orphans` | 許容する孤立した specre カード（リンク切れカードや id 無し）の数 | `10` |
| `index_age_hours` | `index.json` を「古い」と判定するまでの時間 | `48` |

導入初期は閾値を低めに設定し、specre カードの蓄積に合わせて段階的に引き上げていきます。最終的には `coverage = 0.8` 以上、`orphans = 0` を目指すのが理想です。

---

## ステップ 5: AI ワークフローコマンドとスキルの導入

specre は、AI コーディングエージェントと連携するためのワークフローコマンドとスキルを提供しています。これらを自分のプロジェクトにコピーすることで、`/specre-whats-next` や `/specre-generate` などのスラッシュコマンドが使えるようになります。

### コピー元

[specre リポジトリ](https://github.com/yoshiakist/specre) の `.claude/` ディレクトリに格納されています。

### コピーするファイル

**コマンド（スラッシュコマンド）** — AI に特定のワークフローを実行させるプロンプトです：

| ファイル | 用途 |
|---------|------|
| `specre-whats-next.md` | エコシステムの診断と次のアクション提案 |
| `specre-generate.md` | ドメインの未カバーファイルに specre カードを一括生成 |
| `specre-sdd-new.md` | SDD ワークフローで新機能を実装 |
| `specre-sdd-fix.md` | SDD ワークフローで既存機能を修正 |
| `specre-sdd-quality-improvement.md` | SDD ワークフローでコード品質を改善 |
| `specre-refine-glossary.md` | glossary.toml を整理して検索ヒント品質を向上 |
| `scripts/source-dir-scope.sh` | `specre-whats-next` が内部で使用するヘルパースクリプト |

**スキル** — AI が状況に応じて自動的に参照するガイドラインです：

| ディレクトリ | 用途 |
|------------|------|
| `specre-investigate-intent/` | コードの仕様意図を specre カードから調査する |
| `specre-author/` | specre カードの作成・編集ガイドライン |

### Claude Code の場合

```bash
# specre リポジトリをクローン（一時的に）
git clone --depth 1 https://github.com/yoshiakist/specre.git /tmp/specre

# コマンドをコピー
mkdir -p .claude/commands/scripts
cp /tmp/specre/.claude/commands/specre-*.md .claude/commands/
cp /tmp/specre/.claude/commands/scripts/source-dir-scope.sh .claude/commands/scripts/

# スキルをコピー
mkdir -p .claude/skills
cp -r /tmp/specre/.claude/skills/specre-* .claude/skills/

# クリーンアップ
rm -rf /tmp/specre
```

### 他のエディタの場合

コマンドとスキルの配置先は、使用するエディタによって異なります：

| エディタ | コマンド配置先 | スキル配置先 |
|---------|-------------|------------|
| Claude Code | `.claude/commands/` | `.claude/skills/` |
| Cursor | `.cursor/rules/` 等 | エディタの設定に依存 |
| Windsurf | `.windsurf/rules/` 等 | エディタの設定に依存 |
| Gemini | `.gemini/` 等 | エディタの設定に依存 |

> **注意**: コマンドとスキルの形式は Claude Code に最適化されています。他のエディタを使用する場合、各エディタのカスタムルール/カスタムコマンドの仕様に合わせて、ファイルの配置先やフォーマットを調整してください。

---

## ステップ 6: MCP サーバの設定

specre MCP サーバを設定すると、AI エージェントが specre の検索、トレース、カバレッジ確認などの機能に直接アクセスできるようになります。これは単なる「機能の追加」ではありません。MCP サーバが有効になり、エコシステムが健全な状態であれば、コーディングエージェントがファイル検索やプロジェクト探索を行う際に、従来の grep やファイルツリー走査ではなく **specre search や specre trace を第一候補として選択する**ようになります。結果として、エージェントは1〜2回のツール呼び出しで目的の仕様・ソース・テストにたどり着けるようになり、コンテキストウィンドウの節約とタスク精度の向上が期待できます。

MCP サーバは `specre mcp` コマンドで起動され、stdio トランスポートで通信します。

### Claude Code

`.claude/settings.json` に以下を追加します（ファイルが存在しない場合は新規作成）：

```json
{
  "mcpServers": {
    "specre": {
      "command": "specre",
      "args": ["mcp"]
    }
  }
}
```

### Cursor

`.cursor/mcp.json` に以下を追加します：

```json
{
  "mcpServers": {
    "specre": {
      "command": "specre",
      "args": ["mcp"]
    }
  }
}
```

### VS Code (GitHub Copilot)

`.vscode/mcp.json` に以下を追加します：

```json
{
  "servers": {
    "specre": {
      "type": "stdio",
      "command": "specre",
      "args": ["mcp"]
    }
  }
}
```

### Windsurf

`~/.codeium/windsurf/mcp_config.json` に以下を追加します：

```json
{
  "mcpServers": {
    "specre": {
      "command": "specre",
      "args": ["mcp"]
    }
  }
}
```

### 動作確認

エディタを再起動（または MCP サーバの再読み込み）した後、AI エージェントに以下を依頼してください：

```
specre の health-check を実行して
```

エージェントが Bash などのターミナルを経由せずに specre の health-check ツールを呼び出して結果を返せば、MCP サーバが正常に動作しています。

> **トラブルシューティング**: MCP サーバが起動しない場合、以下を確認してください：
>
> - `specre --version` がターミナルで正常に動作するか（PATH の問題）
> - `specre.toml` がプロジェクトルートに存在するか（`specre init` 実行済みか）
> - エディタの設定ファイルの JSON 構文が正しいか

※ MCP 導入は強く推奨されますが、もし MCP が動作しなくても Bash から specre コマンドを発行するようにフォールバックする設計になっています。

---

## ステップ 7: 基底プロンプトへの行動指針の記載

specre MCP サーバを設定しただけでは、コーディングエージェントは specre をいつ・どう使うべきか判断できません。プロジェクトの基底プロンプト（CLAUDE.md、GEMINI.md、CLINE.md、AGENT.md 等）に、**health-check の結果に基づく行動指針**を明記する必要があります。

この記載により、エージェントはセッション開始時に自動的に health-check を実行し、エコシステムが健全であれば specre search や specre trace を積極的に活用してコード探索を行うようになります。

以下のテンプレートをプロジェクトの基底プロンプトファイルにコピーしてください：

```markdown
## specre: 仕様駆動のコード探索

specre MCP ツールが利用可能な場合、**他のいかなる specre 操作よりも先に `health-check` を実行すること。**
この1回の呼び出しで、セッション全体のコード探索戦略が決まる。

### healthy = true の場合

specre エコシステム（仕様カード、トレーサビリティリンク、インデックス、カバレッジ）は
信頼できる状態にある。積極的に活用すること：

- **`specre search`** で関連する specre カードを検索する。
  最も効果的なパターンは **名詞**（対象）と **操作**（何をするか）を組み合わせた AND クエリ：
  - `specre search "認証 ログイン"` — 認証関連の specre カードを検索
  - `specre search "注文 検証"` — 注文検証の specre カードを検索
  - AND クエリが厳しすぎる場合のみ `--or` を追加
- **`specre trace`** で specre カード、ソースファイル、テストファイル間を往来する：
  - `specre trace <ULID>` — specre カードからリンク先の全ソース/テストファイルへ
  - `specre trace <ファイルパス>` — ソースファイルから管轄する specre カードへ
- specre カードを**各振る舞いの正式な記述**として信頼する。
  コードを変更する前に、カードの Scenarios と Related Files を読むこと。

### healthy = false の場合

specre エコシステムにギャップがある。specre ツールに依存せず、以下にフォールバックすること：

- `grep` / `glob` でコードを探索
- ソースファイルとテストを直接読む
- specre カードは参考資料として扱い、唯一の情報源としない
```

### エディタ別の基底プロンプトファイル

| エディタ | ファイル |
|---------|---------|
| Claude Code | `CLAUDE.md` |
| Cursor | `.cursorrules` または `.cursor/rules/` |
| Windsurf | `.windsurfrules` |
| Cline | `.clinerules` |
| Gemini | `GEMINI.md` |
| エディタ非依存 | `AGENT.md` |

> **なぜこれが重要か**: 健全な specre エコシステムとは、すべての振る舞いに仕様があり、すべてのソースファイルがトレースされ、インデックスが最新であることを意味します。この状態をエージェントに伝えることで、エージェントは広範なコードベース検索の代わりに 1〜2 回のツール呼び出しで目的のファイルを見つけられます。この指針がなければ、エージェントは MCP ツールの存在を知っていても、それをいつ使うべきか判断できません。

---

## ステップ 8: glossary.toml の最適化

`specre init` で生成された `glossary.toml` には、specre 自体の汎用的な語彙がサンプルとして含まれています。しかし、あなたのプロジェクト固有のドメイン語彙（例: 「認証」「注文」「決済」など）は含まれていません。

`glossary.toml` は `specre search` のヒント品質を直接左右します。エージェントが specre search を使用した際に、検索結果が0件だったり多すぎたりする場合、glossary.toml の語彙を元にクエリの改善候補を提案します。**プロジェクト固有の語彙が glossary に登録されていなければ、エージェントは最短経路で目的の仕様カードにたどり着けません。**

AI エージェントに以下を依頼してください：

```
/specre-refine-glossary
```

このコマンドは以下を自動的に行います：

1. プロジェクトの README やソースコードからドメイン語彙を分析
2. 多様な検索パターンで現在のヒント品質をテスト
3. 不要な汎用語の削除と、プロジェクト固有の語彙の追加を提案・実行
4. 改善前後のヒント品質を比較して報告

> **このステップは specre カードを作り始める前に必ず実行してください。** glossary の最適化は specre エコシステム全体の検索効率に影響するため、カードの蓄積が始まってからでは手戻りが発生します。

---

## ステップ 9: `/specre-whats-next` を実行する

すべてのセットアップが完了しました。AI エージェントに最初の診断を依頼しましょう：

```
/specre-whats-next
```

このコマンドは以下を自動的に行います：

1. `specre.toml` を読み取り、プロジェクトの設定を確認
2. `specre health-check` でエコシステムの健全性を診断
3. 現在の状態に基づいて、**次に取るべき具体的なアクションを1つ**提案

初回実行時は、specre カードがまだ存在しないため、カバレッジが低い状態から始まります。これは正常です。コマンドが提案するアクションに従って、少しずつ specre エコシステムを育てていきましょう。

### 繰り返し実行する

`/specre-whats-next` は繰り返し実行するように設計されています。1つのアクションを完了したら、再度実行して次のステップを確認してください。

```
/specre-whats-next → アクション実行 → /specre-whats-next → アクション実行 → ...
```

この反復的なプロセスにより、1ステップずつ specre カバレッジが拡大し、エコシステムが成熟していきます。

---

## セットアップ完了後のプロジェクト構成

すべてのステップを完了すると、プロジェクトは以下のような構成になります（Claude Code の場合）：

```
your-project/
├── specre.toml                            ← specre 設定ファイル
├── glossary.toml                          ← 検索ヒント用プロジェクト語彙
├── CLAUDE.md                              ← 基底プロンプト（specre 行動指針を含む）
├── docs/
│   └── specres/                           ← specre カード格納ディレクトリ
│       └── (ドメイン名)/
│           └── (振る舞い名).md
├── .claude/
│   ├── settings.json                      ← MCP サーバ設定
│   ├── commands/
│   │   ├── specre-whats-next.md           ← 診断・次のアクション提案
│   │   ├── specre-generate.md             ← specre カード一括生成
│   │   ├── specre-sdd-new.md              ← SDD で新機能実装
│   │   ├── specre-sdd-fix.md              ← SDD で既存機能修正
│   │   ├── specre-sdd-quality-improvement.md
│   │   ├── specre-refine-glossary.md
│   │   └── scripts/
│   │       └── source-dir-scope.sh
│   └── skills/
│       ├── specre-investigate-intent/      ← 仕様意図の調査スキル
│       │   ├── SKILL.md
│       │   └── scripts/
│       │       └── specre-investigate.sh
│       └── specre-author/                  ← specre カード作成スキル
│           └── SKILL.md
└── src/                                    ← あなたのソースコード
```

---

## 次のステップ

### 導入戦略を選ぶ

specre カードをどう作り始めるかは、プロジェクトのテスト品質やカバレッジによって異なります。[導入戦略ガイド](adoption-strategy.ja.md) では、3つの戦略を詳しく解説しています：

| 戦略 | 適用条件 |
|------|---------|
| **テスト由来の抽出** | テストが振る舞い指向で、カバレッジが高い場合 |
| **コード振る舞い分析** | テストが実装指向、またはカバレッジが低い場合 |
| **トップダウンドメイン分解** | 新規開発、大規模リライト、またはテストが少ない場合 |

### よく使うコマンド

| コマンド | 用途 |
|---------|------|
| `/specre-whats-next` | 次にやるべきことを診断・提案 |
| `/specre-generate <domain>` | 指定ドメインの specre カードを一括生成 |
| `/specre-sdd-new` | SDD ワークフローで新機能を実装 |
| `/specre-sdd-fix` | SDD ワークフローで既存機能を修正 |
| `specre coverage` | カバレッジの確認 |
| `specre health-check` | エコシステムの健全性確認 |
| `specre status` | specre カードのステータス一覧 |

### 大切にしてほしいこと

- **1ドメインずつ進める** — 一度にすべてをカバーしようとせず、1つのドメインを確実に仕上げてから次に移ってください。
- **`/specre-whats-next` を信頼する** — 何をすべきか迷ったら、このコマンドに聞いてください。
- **`stable` を安易に付けない** — `draft` は正直な状態です。実装とテストが仕様と一致していることを確認してから `stable` に昇格してください。
- **完璧を目指さない** — コードベース全体の100%カバレッジは目標ではありません。アクティブに開発中の振る舞いをカバーすることが目標です。
