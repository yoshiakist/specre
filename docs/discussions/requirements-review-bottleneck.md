# 要件定義・コードレビューのボトルネックに specre が何をできるか

## 問題の構造

エンジニア3人 × 1ヶ月（120〜150ストーリーポイント）規模のプロジェクトにおいて、生産性のボトルネックとなるのは実装そのものではなく、以下の2つの人間判断レイヤーである:

1. **要件定義** — 何を作るか決め、漏れなく・矛盾なく仕様を確定する作業
2. **コードレビュー** — 変更が既存の振る舞いを壊さないことを検証し、潜在的影響を見抜く作業

GitHub Spec Kit、Amazon Kiro、あるいは specre を使おうが、この2つのレイヤーは人間の認知負荷に依存しており、ツール導入だけでは劇的な改善が見込めないという指摘は正当である。

## 現状のロードマップの位置づけ

specre のロードマップ（v0.4〜v0.7）は、これらのボトルネック解消に必要な **プリミティブ** を提供する計画になっている:

| バージョン | プリミティブ | 要件定義への寄与 | レビューへの寄与 |
|-----------|------------|----------------|----------------|
| v0.4 drift | 仕様とコードの乖離検出 | 間接的（陳腐化した仕様の発見） | 間接的（stale な仕様の警告） |
| v0.5 impact | 推移的依存グラフ | **直接的**（変更の波及範囲特定） | **直接的**（影響範囲の網羅） |
| v0.5 diff | 仕様変更の可視化 | 間接的 | **直接的**（何が変わったかの把握） |
| v0.6-v0.7 multi-repo | 境界を越えたトレーサビリティ | 間接的（契約の可視化） | 間接的 |

**問題点:** これらのプリミティブは個々には有用だが、「要件定義のワークフロー」「レビューのワークフロー」として **組み上がっていない**。個々のレンガはあるが、建物がない。

## 仮説の整理と評価

### 仮説1: リスクファクター推論の質の向上

> こんな機能を追加したいが、どんなリスクファクターがあるか？といった質問に対するコーディングエージェントの推論の質の向上

**評価: 有望だが、現状のロードマップでは実現手段が不足**

LLM が「リスクは何か？」と問われたとき、現状では以下のように推論する:

- コードベースを `grep` / `glob` で探索し、関連しそうなファイルを読む
- コンテキストウィンドウに収まった範囲で推測する
- **漏れがあっても検知できない**（知らないものは推論できない）

specre のトレーサビリティグラフがあれば、推論の前提が変わる:

- 「この振る舞いに影響するファイルは A, B, C（traceability graph より）」→ **決定論的に完全**
- 「この振る舞いが依存する他の振る舞いは X, Y（cross-reference より）」→ **推移的に網羅**
- LLM の推論は「当てずっぽう」から「事実に基づく分析」に変わる

**しかし、現ロードマップの `specre impact` だけでは不十分。** 「まだ存在しない機能」の影響を分析するには、既存のグラフから「隣接領域」を特定する仕組みが必要。

### 仮説2: git差分からの潜在影響の決定論的特定

> PRが振る舞いを変える前に潜在的な影響があったはずの場所をgit差分から特定

**評価: 最も具体的かつ実現可能性が高い**

これは以下のように機械的に実行できる:

1. git diff から変更ファイル一覧を抽出
2. 変更ファイル → `@specre` マーカー → 影響を受ける specre カードを特定
3. 影響を受ける specre カード → `trace` → 関連する **全ファイル** を特定
4. 「diff に含まれないが影響範囲内にあるファイル」を特定（= 潜在影響）
5. 影響を受ける specre カードの Scenarios をリストアップ（= 検証すべき振る舞い）

これは LLM の推論ではなく、**グラフ走査による決定論的な操作** である。LLM はこの結果を「解釈」する役割に専念できる。

## 具体的な機能提案

### A. コードレビュー支援（確度: 高）

#### A-1. `specre blast-radius <commit-range>`

git diff を入力とし、影響を受ける振る舞いの全範囲を決定論的に出力する。

```
$ specre blast-radius main..feature/add-discount

Affected behaviors (3):
  01ABC... order_total_calculation_applies_tax     [stable]
  01DEF... discount_code_reduces_order_total        [in-development]
  01GHI... invoice_generation_reflects_final_price   [stable]

Blast radius (files not in diff but potentially affected):
  src/invoice/generator.rs        ← governed by 01GHI...
  tests/invoice/test_generator.rs ← governed by 01GHI...

Scenarios to verify:
  [01ABC...] Given an order with tax → total includes tax after discount
  [01GHI...] Given a completed order → invoice reflects discounted total
```

**なぜこれが価値を持つか:**
- レビュアーは「何を見ればいいか」を **diff を読む前に** 知ることができる
- 「このPRは3つの振る舞いに影響し、invoice の生成に潜在的影響がある」— この情報だけでレビューの焦点が定まる
- 現状のレビューは diff を上から順に読んで頭の中で影響範囲を組み立てるが、この操作を機械化する

**実装の前提:** v0.4（drift）と v0.5（impact）のプリミティブ。ただし `blast-radius` 自体は v0.5 と同時に実装可能。

#### A-2. `specre review-checklist <commit-range>`

blast-radius の結果を、レビュアーが消化可能なチェックリストに変換する。

```markdown
## Review Checklist for feature/add-discount

### Directly modified behaviors
- [ ] `discount_code_reduces_order_total` — 3 files changed, 2 scenarios
  - [ ] Scenario: valid code reduces total by percentage
  - [ ] Scenario: expired code is rejected with error message

### Indirectly affected behaviors
- [ ] `order_total_calculation_applies_tax` — 0 files changed, but depends on discount calculation
  - [ ] Verify: tax is calculated on post-discount amount (not pre-discount)
- [ ] `invoice_generation_reflects_final_price` — 0 files changed, reads from order total
  - [ ] Verify: invoice shows discounted total, not original total

### Untraced changes
- `src/utils/format.rs` — changed but not linked to any specre card
  - [ ] Manual review required (no specification coverage)
```

**なぜこれが価値を持つか:**
- レビュアーの認知負荷を「全部理解する」から「チェックリストを検証する」に変換する
- 「untraced changes」セクションが **specre カバレッジの穴** を可視化し、カード追加の動機づけにもなる

#### A-3. MCP プロンプト `review`

blast-radius + review-checklist の結果を LLM に渡し、レビューコメントのドラフトを生成させる。specre 自体は決定論的な情報収集のみを担い、LLM は解釈・判断を担う。

```
[MCP prompt: review]
Input: commit range or PR reference
Process:
  1. specre blast-radius → affected behaviors + blast radius
  2. specre diff → what changed in affected specre cards
  3. Read affected scenarios
  4. LLM: "Given these behavioral specifications and this code diff,
           identify potential regressions, missing test coverage,
           and scenarios that may need updating."
```

### B. 要件定義支援（確度: 中）

#### B-1. `specre scope <query-or-description>`

新機能の追加や既存機能の変更を計画する際に、影響範囲と必要な作業を見積もるための情報を提供する。

```
$ specre scope "ディスカウントコードの適用機能を追加"

Related existing behaviors (by keyword + graph proximity):
  01ABC... order_total_calculation_applies_tax     [stable] — 直接影響
  01GHI... invoice_generation_reflects_final_price  [stable] — 間接影響
  01JKL... payment_gateway_charges_final_amount    [stable] — 間接影響

Structural complexity:
  Directly related source files: 8
  Transitively related source files: 14
  Test files in scope: 6
  Cross-domain touches: 2 (order → invoice, order → payment)

Suggested new specre cards:
  - discount_code_reduces_order_total (core behavior)
  - expired_discount_code_is_rejected (error case)
  - discount_and_tax_interaction (edge case — tax × discount の適用順序)
```

**なぜこれが仮説レベルを超えうるか:**
- 「関連する既存の振る舞い」の特定は、traceability graph + keyword search で **決定論的に** 実行可能
- 「構造的な複雑度」はファイル数・ドメイン横断数から機械的に算出可能
- 「必要な specre カード」の提案は LLM の推論だが、入力が構造化されているため精度が上がる

**限界:** 要件定義の本質的な難しさ（何を作るべきか）は解決しない。しかし「作ると決めたものが何に影響するか」の見積もり精度を上げることで、要件定義の **フィードバックループ** を速くする。

#### B-2. `specre contradiction-check <specre-path>`

新しい specre カードまたは既存カードの変更を、他のカードの Scenarios と突き合わせて矛盾を検出する。

```
$ specre contradiction-check docs/specres/order/discount_code_reduces_order_total.md

Potential conflicts found:

  With: order_total_calculation_applies_tax (01ABC...)
    Your scenario: "Given a 20% discount → total is reduced by 20%"
    Existing scenario: "Given an order → total includes 10% tax"
    Question: 税はディスカウント前の金額に適用？後の金額に適用？
    → specre カードで明示的に定義することを推奨

  With: minimum_order_amount_required (01MNO...)
    Your scenario: "Given a 100% discount → total is 0"
    Existing scenario: "Given an order below ¥500 → order is rejected"
    Question: 100%ディスカウントで合計が ¥0 になった場合、最低注文金額チェックはどうなる？
    → specre カードで明示的に定義することを推奨
```

**実装の現実性:** 完全に決定論的な矛盾検出は困難。しかし:
- Scenarios のキーワード重複検出（決定論的）で候補を絞り込み
- LLM に「この2つのシナリオは矛盾する可能性があるか？」と判断させる
- specre は候補の絞り込みを担い、LLM は判断を担う — 役割分担が明確

### C. 技術的難易度の見積もり支援（確度: 中〜高）

#### C-1. `specre complexity <ULID | specre-path>`

specre カード（既存または新規）に対する構造的な複雑度メトリクスを出力する。

```
$ specre complexity 01DEF...

Behavior: discount_code_reduces_order_total

Structural metrics:
  Direct source files: 3
  Transitive source files: 11
  Direct test files: 2
  Transitive test files: 7
  Cross-domain dependencies: 2 (→ invoice, → payment)
  Referenced specifications: 4
  Depth in dependency graph: 3

Historical metrics (git):
  Average change frequency of related files: 2.3 commits/month
  Last modification: 5 days ago
  Contributors to related files: 3
```

**なぜこれが見積もり精度を上げるか:**
- 「このカードに対応する実装は11ファイルに影響し、2つのドメインをまたぎ、依存の深さが3」→ これは従来の「感覚的な見積もり」を構造的なデータで裏付ける
- 複雑度の数値そのものが正確な工数予測を意味するわけではないが、**相対的な比較** に有用（「A の実装は B の3倍複雑」）
- git の変更頻度データを組み合わせることで、「よく変わる領域に触る変更は慎重に」というシグナルを機械的に提供

## ロードマップへの統合案

現行のロードマップを破壊せず、プリミティブの上にワークフローレイヤーを追加する形で統合する:

```
v0.4 — Drift Detection [変更なし]
  └ specre drift, specre ci, GitHub Actions template

v0.5 — Decision Support [QA Support から拡張]
  ├ specre impact        [既存計画]
  ├ specre diff          [既存計画]
  ├ specre export        [既存計画]
  ├ specre blast-radius  [新規: A-1]  ← impact + trace + git diff の合成
  ├ specre complexity    [新規: C-1]  ← impact + git history の合成
  └ MCP prompt: review   [新規: A-3]  ← blast-radius の結果を LLM に渡す

v0.5.x — Review & Planning Workflows [新規マイルストーン]
  ├ specre review-checklist  [新規: A-2]  ← blast-radius → checklist 変換
  ├ specre scope             [新規: B-1]  ← search + impact + complexity の合成
  └ specre contradiction-check [新規: B-2] ← scenarios のクロスチェック

v0.6〜v0.7 — Multi-Repository [変更なし]
```

## なぜ「仮説だらけで弱い」のか — 構造的な理由

指摘の通り、これらの提案はまだ仮説を含む。しかし「仮説である」ことの構造的な理由を明確にしておく:

### 決定論的に解決可能な部分（確度: 高）

- git diff → affected files → affected specre cards → blast radius （グラフ走査）
- 構造的な複雑度メトリクス （ファイル数、ドメイン横断数、依存深度）
- Scenarios のリストアップ （パース済みデータの出力）

これらは **specre エコシステムが健全であれば確実に機能する**。LLM に依存しない。

### LLM の推論品質に依存する部分（確度: 中）

- blast-radius の結果からの「レビューコメント」生成
- scope の結果からの「必要な specre カード」の提案
- contradiction-check の「矛盾する可能性がある」という判断

これらは **入力の質** で精度が変わる。specre が提供する構造化された入力は、grep で集めた断片的な情報より遥かに高品質だが、LLM の推論そのものの改善は specre のスコープ外。

### 人間の行動変容に依存する部分（確度: 低）

- レビュアーが実際にチェックリストを使うか
- 要件定義者が scope の出力を信頼して意思決定するか
- チームが specre カバレッジを十分に高く維持するか

ここはツールの品質だけでは解決できない。しかし **決定論的な部分の品質が高ければ、行動変容のハードルは下がる**。

## D. 半決定論的影響評価 — `@specre-todo` マーカー（確度: 中〜高）

### 問題: impact と potential impact の区別

前述の `specre impact` や `specre blast-radius` が扱うのは **retrospective impact** — 「既存のコードと仕様の関係性に基づいて、変更の影響範囲を特定する」。これは既にあるグラフの走査であり、完全に決定論的である。

しかし、計画段階で本当に必要なのは **prospective impact** — 「まだ存在しない変更が、将来どこに影響を及ぼしうるか」。ここには本質的な不確実性がある。

従来の手法ではこの不確実性への対処法は2つしかなかった:

1. **人間が推測する** — 経験に基づくが、漏れやすく、スケールしない
2. **LLM に全て任せる** — コンテキストウィンドウに収まった範囲で推測するが、網羅性に保証がない

### 提案: `@specre-todo` による確率的マーキング + 決定論的伝播

新しい手法として、**確率的な判断と決定論的な分析を明確に分離する** アプローチを提案する。

```
Phase 1 [確率的]   コーディングエージェントが将来の変更箇所を推定し
                   @specre-todo マーカーをコード上に配置する

Phase 2 [人間検証] マーカーの妥当性をレビューする
                   （個々のマーカーは小さな判断 = レビューしやすい）

Phase 3 [決定論的] マーカーを起点にトレーサビリティグラフを走査し
                   影響範囲を完全に列挙する
```

#### マーカーの設計

```rust
// @specre-todo 01ABC... "ディスカウント適用後の金額でtax計算に変更"
fn calculate_tax(order: &Order) -> Amount {
    order.subtotal * TAX_RATE  // 現状: subtotal に対して計算
}
```

マーカーの構成要素:
- `@specre-todo` — 将来の変更を宣言するプレフィックス（`@specre` と区別可能）
- ULID — 変更の根拠となる specre カード（新規カード or 変更予定カード）
- 説明文 — 何が変わるかの簡潔な記述（LLM が生成、人間がレビュー）

#### なぜこれが従来手法と本質的に異なるか

| 手法 | 「どこが変わるか」 | 「何が影響するか」 | 検証可能性 |
|------|------------------|------------------|-----------|
| 人間の勘 | 確率的 | 確率的 | 低（暗黙知） |
| LLM に全部任せる | 確率的 | 確率的 | 低（散文出力） |
| 静的解析（call graph） | N/A（既存コードのみ） | 決定論的 | 高 |
| **@specre-todo** | **確率的 → 人間が検証** | **決定論的** | **高（コード上のマーカー）** |

鍵は: **確率的な不確実性を「どこが変わるか」という小さく検証可能な判断に閉じ込め、「変わった場合に何が影響するか」を決定論的に処理する** こと。

これは既存のソフトウェア工学の Change Impact Analysis（Bohner & Arnold, 1996）にも、Feature Flags にも、静的解析にも見当たらない組み合わせである。LLM の確率的推論をコードベース上の物理的なマーカーとして「物質化」し、そこから先を決定論的に処理するという二段構えは、LLM 時代の新しいパターンと言える。

#### マーカーが監査可能（auditable）であることの意味

LLM が「この変更はここに影響するでしょう」と散文で述べるのと、コード上に `@specre-todo` マーカーを物理的に配置するのとでは、レビュアビリティが根本的に異なる:

- マーカーは `grep` で一覧でき、PRの diff に出る
- 各マーカーは個別に承認/却下できる（小さな判断の集合）
- マーカーの正確性はコードの文脈で検証可能（散文の妥当性評価よりはるかに容易）
- マーカーは永続化される（LLM のセッションをまたいで残る）

#### 具体的なワークフロー

```
$ specre plan docs/specres/order/discount_code_reduces_order_total.md

Planning impact for: discount_code_reduces_order_total

Step 1: Agent analyzing specre card and existing codebase...
Step 2: Placing @specre-todo markers in potentially affected files...

Markers placed (5):
  src/order/total.rs:42        @specre-todo 01DEF... "subtotal計算にdiscount適用"
  src/order/tax.rs:18          @specre-todo 01DEF... "tax計算の基準額変更"
  src/invoice/generator.rs:67  @specre-todo 01DEF... "invoice表示にdiscount行追加"
  src/payment/charge.rs:23     @specre-todo 01DEF... "課金額がdiscount後の金額に"
  tests/order/test_total.rs:5  @specre-todo 01DEF... "discount適用のテスト追加"

Step 3: Computing deterministic impact from markers...

Impact from @specre-todo markers:
  Directly marked files: 5
  Transitively affected (via specre graph): 3 additional files
    src/payment/receipt.rs      ← governed by payment_receipt_matches_charge
    src/report/daily_sales.rs   ← governed by daily_report_aggregates_revenue
    tests/payment/test_receipt.rs

  Affected behaviors (total): 4
    01DEF... discount_code_reduces_order_total  [in-development] (primary)
    01ABC... order_total_calculation_applies_tax [stable]
    01GHI... invoice_generation_reflects_final_price [stable]
    01PQR... daily_report_aggregates_revenue [stable]

Review the markers with: git diff --cached
Remove incorrect markers with: specre plan --remove <file:line>
```

#### チーム間の早期情報提供

`@specre-todo` マーカーの副次的だが重要な効果: **別の作業をしている同じプロダクトチームへの注意喚起**。

例えば、チームAが discount 機能を開発中で、`src/invoice/generator.rs` に `@specre-todo` マーカーを配置したとする。チームBは同時期に invoice の表示改善を進めている。

```
$ specre todo-status

Active @specre-todo markers in this repository:

  discount_code_reduces_order_total (01DEF...) — Team A / Sprint 24
    src/invoice/generator.rs:67  "invoice表示にdiscount行追加"
    src/payment/charge.rs:23     "課金額がdiscount後の金額に"
    ... (3 more)
```

チームBのエンジニアが `invoice/generator.rs` を開くと:

```rust
// @specre-todo 01DEF... "invoice表示にdiscount行追加"  ← これが目に入る
fn generate_invoice(order: &Order) -> Invoice {
    // ...
}
```

この情報は:
- **Slack や口頭での「あ、そこ今触ってるんだけど」を機械化する**
- マーカーはコード上に物理的に存在するため、IDE の検索やPRの diff で自然に目に入る
- `specre todo-status` でリポジトリ全体の計画中の変更を俯瞰できる
- 従来の「スプリント計画で口頭共有 → 忘れる」というパターンを、コードベース上の永続的なシグナルに変換する

#### ライフサイクル管理

`@specre-todo` マーカーにはライフサイクル管理が必須。放置されたマーカーはノイズになる。

```
状態遷移:
  placed → reviewed → implemented → removed
           → rejected (→ removed)
           → stale (→ reviewed again or removed)
```

- **placed**: LLM がマーカーを配置した直後
- **reviewed**: 人間がマーカーの妥当性を確認
- **implemented**: 実際の変更が完了（`@specre-todo` → `@specre` に昇格）
- **stale**: 一定期間（configurable）実装されなかったマーカー → 再レビューまたは削除

`specre ci` は stale な `@specre-todo` を警告として報告できる（エラーではなく）。

### D-1.5. マーカー密度による責務集中の早期検出

#### 問題: 技術的負債は実装後にしか検出できない

従来のコード品質メトリクス（循環的複雑度、行数、結合度）は全て **実装が完了した後** にしか測定できない。ファイルが肥大化し、if 分岐が増え、責務が混在してから「ここは負債だ」と気づく。そして気づいた時には、リファクタリングのコストが既に高い。

#### 観察: マーカー密度は計画段階のアーキテクチャ品質指標になる

`@specre-todo` マーカーを使って将来の変更を計画すると、1つのファイルに複数のマーカーが集中することがある。これは「複数の振る舞いがこのファイルに依存している（or 依存することになる）」ことを **定量的に** 示す。

```
src/order/total.rs:
  // @specre 01AAA...     ← 既存: 注文合計の計算
  // @specre 01BBB...     ← 既存: 税込み計算
  // @specre 01CCC...     ← 既存: 送料の加算
  // @specre-todo 01DDD... ← 計画: ディスカウント適用
  // @specre-todo 01EEE... ← 計画: ポイント利用による減額
  // @specre-todo 01FFF... ← 計画: クーポン併用ロジック
  // @specre-todo 01GGG... ← 計画: サブスクリプション割引
```

ちょっと慣れた人がこれを見れば、即座に嗅覚が働く:

- 「7つの振る舞いが1ファイルに集中している — 責務過多の兆候」
- 「振る舞いを追加する前に、まず構造を分割すべきでは？」
- 「ここにさらに条件分岐を足すと、if/match の入れ子が深くなって筋が悪い」

#### CI による自動検出

この嗅覚を定量化し、CI で自動警告できる:

```toml
# specre.toml
[markers]
max_markers_per_file = 7  # 1ファイルあたりのマーカー上限（@specre + @specre-todo の合計）
```

```
$ specre ci

Warning: Marker concentration exceeds threshold (7)
  src/order/total.rs: 7 markers (3 @specre + 4 @specre-todo)
    → Consider decomposing responsibilities before adding new behaviors

  Suggested actions:
    - Review governing specre cards to identify separable concerns
    - Extract discount/coupon logic into src/order/pricing.rs
    - Run `specre plan --refactor src/order/total.rs` for decomposition suggestions
```

#### 健全なマーカー分布のパターン

マーカー密度の閾値は一律ではない。設計パターンによって健全な分布は異なる:

**典型的な実装ファイル: 1〜3 マーカーが健全**

```rust
// src/order/discount.rs
// @specre 01DEF...  ← このファイルは1つの振る舞いに専念
```

1つのファイルが1〜3個の明確な振る舞いに対応する。これは Single Responsibility の範囲内。

**集約パターン（Config, Repository, Router）: 意図的に多数**

```rust
// src/config.rs
// @specre 01AAA...  ← DB設定の読み込み
// @specre 01BBB...  ← API認証設定の読み込み
// @specre 01CCC...  ← キャッシュ設定の読み込み
// ... (10+ markers)
// #[specre::allow(marker_concentration)]  ← 意図的な集約
```

Config、Repository、Router などのパターンは、設計上バリエーションを集約する役割を持つ。これらは閾値チェックから明示的に除外する設定が必要:

```toml
# specre.toml
[markers]
max_markers_per_file = 7

# 意図的な集約パターン — 閾値チェックから除外
allow_concentration = [
  "src/config.rs",
  "src/routes/mod.rs",
  "src/repository/*.rs",
]
```

**4〜6 マーカーの「グレーゾーン」**

このゾーンは、現時点ではまだ管理可能だが、今後のマーカー追加で閾値を超える可能性がある状態。CI は警告ではなく情報レベルで通知し、計画段階で分割を検討する材料を提供する。

#### なぜこれが「半決定論的な」技術的負債防止か

従来の技術的負債防止:
```
[実装完了] → [メトリクス測定] → [負債検出] → [リファクタリング]
             すでに書かれたコードを事後的に測定
             リファクタコストは既に高い
```

マーカー密度による防止:
```
[specre-todo 配置] → [密度計測] → [集中警告] → [設計修正] → [実装]
                     まだ書かれていないコードの密度を測定
                     修正コストはほぼゼロ（マーカーを移動するだけ）
```

「半決定論的」と呼ぶ理由:
- マーカーの配置自体は確率的（LLM の推定 + 人間のレビュー）
- しかし密度の計測と閾値チェックは完全に決定論的
- 「このファイルにマーカーが7つある」は事実であり、解釈の余地がない
- そこから「責務を分割すべきか」の最終判断は人間がするが、**気づく仕組みが機械的に保証される**

### D-2. スケルトンファイルへの拡張 — コードベース上の詳細設計

#### 概念の拡張

`@specre-todo` を既存コードの変更予告だけでなく、**まだ存在しない新規ファイルのスケルトン** にも適用する。これにより、`@specre-todo` の性格が本質的に変わる:

- **変更予告マーカー** → **コードベース上に物理的に存在する詳細設計書**

#### スケルトンファイルの構造

```rust
// @specre-todo 01DEF... "ディスカウントコードの検証と適用"

// pub struct DiscountCode {
//     /// ディスカウントコード文字列（例: "SUMMER2026"）
//     code: String,
//     /// 割引率（0.0〜1.0）。固定額割引は別の variant で表現する
//     rate: Decimal,
//     /// 有効期限。None は無期限を意味する
//     expires_at: Option<DateTime<Utc>>,
// }

// impl DiscountCode {
//     /// コードが現在有効かどうかを判定する。
//     /// 期限切れの場合は DiscountError::Expired を返す。
//     pub fn validate(&self) -> Result<(), DiscountError> { ... }
//
//     /// 指定された金額に割引を適用し、割引後の金額を返す。
//     /// 割引後の金額が負になることはない（最低 0）。
//     pub fn apply(&self, amount: Amount) -> Amount { ... }
// }
```

このファイルは:
- コンパイルされない（全てコメント）— ビルドを壊さない
- しかしファイルとして物理的に存在する — ディレクトリ構造が設計判断を表現する
- `@specre-todo` で specre カードと双方向に繋がる — トレーサビリティが成立する
- 主要なメンバー・メソッドの **責務** がコメントで記述される — 型と振る舞いの意図が読める

#### specre カードとスケルトンの関係

specre カード（振る舞い仕様）の Related Files セクションに、スケルトンファイルが `@specre-todo` 経由で自動的に紐づく:

```markdown
---
id: "01DEF..."
name: "discount_code_reduces_order_total"
status: "draft"
---

## Related Files

(以下は specre trace / specre index が @specre-todo マーカーから自動収集)

| File | Status | Description |
|------|--------|-------------|
| src/order/discount.rs | @specre-todo | DiscountCode 構造体と適用ロジック |
| src/order/total.rs | @specre-todo | subtotal 計算に discount 適用 |
| src/invoice/generator.rs | @specre-todo | invoice 表示に discount 行追加 |
| tests/order/test_discount.rs | @specre-todo | discount 適用のテスト |

## Scenarios
...
```

#### レビュー責任者のワークフロー

```
1. specre カード 10枚を読む（振る舞い仕様）
   → 「何を作るか」が分かる
   → 所要時間: 30分程度

2. スケルトンファイル 30-40個を眺める（構造設計）
   → ファイル名・ディレクトリ構成で「どう分解するか」が分かる
   → コメントアウトされた型・メソッドで「各モジュールの責務」が分かる
   → 不明なモジュールがあれば @specre-todo タグで specre カードに辿れる
   → 所要時間: 1-2時間程度

3. 判断を下す
   → 「この分解で多分いける」「ここの責務分離がおかしい」
   → 実装が始まる前にアーキテクチャレベルのフィードバックが可能
```

**なぜこれが従来の設計レビューより精度が高いか:**

- **具体性**: Confluence 上の箇条書きではなく、実際のファイルパス・型名・メソッドシグネチャがある。抽象的な「〇〇モジュール」ではなく `src/order/discount.rs` の `DiscountCode::apply()` が見える
- **検証可能性**: スケルトンがコードベース上にあるため、既存コードとの整合性を即座に確認できる（隣のファイルを開けばいい）
- **トレーサビリティ**: どのスケルトンがどの振る舞いに対応するか、`@specre-todo` で機械的に追跡可能。設計書とコードの対応を人間が頭の中で維持する必要がない
- **漸進性**: スケルトンのコメントを外して実装コードにするだけ。設計と実装の間に断絶がない

#### 設計と実装の連続的グラデーション

従来のソフトウェア開発では「設計」と「実装」の間に断崖がある:

```
従来:
  [設計書] ──── 断崖 ──── [実装コード]
  (Confluence)              (Git)
  別の場所、別のフォーマット、別のレビュープロセス
```

`@specre-todo` スケルトンでは、設計から実装へのグラデーションが生まれる:

```
specre-todo スケルトン:
  [specre カード] → [スケルトンファイル] → [実装コード]
  (振る舞い仕様)    (構造設計)             (実装)
  全て Git 上、全て specre で追跡、同じレビューツール

  設計レビュー時:
    specre カード ✓  スケルトン ✓  実装 (未着手)

  実装完了時:
    specre カード ✓  スケルトン → 実装に昇格 ✓
    (@specre-todo → @specre)
```

#### QA シフトレフトへの寄与

スケルトンファイルが存在する段階で、QA は以下が可能になる:

1. **テスト計画の早期作成**: specre カードの Scenarios + スケルトンの型定義から、テストケースの骨格を設計できる
2. **影響範囲の事前把握**: `specre plan` の出力（直接マーク + 推移的影響）から、リグレッションテストの範囲を実装前に特定できる
3. **リスクアセスメントへの参加**: レビュー責任者がスケルトンを承認した時点で、QA にレポートを提出できる。QA は「どの振る舞いが変わるか」「どのファイルが影響を受けるか」を実装開始前に把握する

```
従来のタイムライン:
  [要件定義] → [設計] → [実装] → [コードレビュー] → [QAテスト]
                                                      ↑ QA が初めて関与

specre-todo スケルトンのタイムライン:
  [specre カード作成] → [スケルトン配置 + 影響分析] → [設計レビュー + QA参加] → [実装] → [コードレビュー] → [QAテスト]
                                                      ↑ QA がここで関与開始
```

QA のシフトレフトにおいて、従来は「要件定義ドキュメントを渡す」というアプローチが主流だったが、これには具体性が欠ける。スケルトンファイルは **実装の具体的な構造** を伴うため、QA が「何をテストすべきか」をより正確に判断できる。

### D-3. specre の責務境界 — 「安全かどうか」の判断について

#### WHERE / WHAT / HOW MUCH は決定論的、HOW は人間の判断

このディスカッションで提案してきた機能群が提供できるものを整理する:

```
決定論的に提供可能:
  WHERE    — どのファイルが影響を受けるか（blast-radius, impact）
  WHAT     — どの振る舞いが影響を受けるか（specre card の特定）
  HOW MUCH — 何ファイル、何ドメイン、依存の深さ（complexity）

人間の判断に委ねるべき:
  HOW      — 変更が安全かどうか
  WHETHER  — リリースして問題ないか
```

「変更が安全かどうか」は意味論の問題であり、コードの実行時の振る舞いに依存する。これを決定論的に解くことは（停止問題と同様に）原理的に不可能であり、specre のスコープ外である。

#### specre が追加で貢献できる層: WHERE TO LOOK

ただし、「安全かどうか」と「安全かどうかを判断するために何を検証すべきか」は別の問題である。後者は specre が構造化できる:

```
specre card: discount_code_reduces_order_total
Scenarios:
  1. Given a valid discount code → total is reduced
  2. Given an expired code → error is returned
  3. Given a discount + tax → tax is on post-discount amount
  4. Given a 100% discount → total is 0, minimum order check applies

@specre-todo 変更点:
  src/order/total.rs:42           — subtotal計算にdiscount適用
  src/invoice/generator.rs:67     — invoice表示にdiscount行追加

→ レビュアーが検証すべき問い:
  「Scenario 3: tax計算は discount 適用後の金額で行われるか？」
  「Scenario 4: minimum order check は discount 後の金額に対して実行されるか？」
  「既存の税計算パスと最低注文チェックのパスに、フラグOFF時に影響がないか？」
```

レビュアーが事故を起こすのは「あのシナリオのことを忘れていた」であって、見えているシナリオの判断を間違えることは比較的少ない。**検証すべきシナリオの完全な列挙** は、「安全かどうか」の判断そのものではないが、判断の漏れを構造的に防ぐ。

#### リリースフラグと specre の接点

リリースフラグの管理そのものは specre のスコープ外だが、一つだけ有用な情報を機械的に出せる:

**変更点の「深度」の可視化**

`@specre-todo` が置かれた場所がアーキテクチャ上のどの層にあるかを分類する:

```
$ specre plan --depth docs/specres/order/discount_code_reduces_order_total.md

Change depth analysis:

  Surface (controller/API/frontend) — flag-guardable:
    src/api/orders_controller.rs:15   @specre-todo "discount パラメータの受け取り"

  Domain (business logic) — flag guard adds complexity:
    src/order/total.rs:42             @specre-todo "subtotal計算にdiscount適用"
    src/order/discount.rs (new)       @specre-todo "DiscountCode 構造体と適用ロジック"

  Infrastructure (DB/external) — flag guard is risky:
    src/repository/order_repo.rs:88   @specre-todo "discount_code カラムの永続化"

  Summary:
    Surface changes: 1 (flag-guardable)
    Domain changes: 2 (flag guard adds complexity — avoid if possible)
    Infrastructure changes: 1 (flag guard is risky — plan migration carefully)
```

この分類があれば:
- 「controller 層でフラグを入れれば、domain と infrastructure の変更はフラグの内側に隠せるか？」という問いに構造的に答えられる
- 「domain 層にフラグを入れないと制御できない」ケースが事前に見え、リスクの高い変更点が計画段階で浮かび上がる

層の分類は `specre.toml` のディレクトリ規約や明示的な設定で定義できる:

```toml
# specre.toml
[layers]
surface = ["src/api/**", "src/controllers/**", "frontend/src/**"]
domain = ["src/order/**", "src/payment/**", "src/invoice/**"]
infrastructure = ["src/repository/**", "src/migrations/**"]
```

#### specre の責務の明確な線引き

```
specre が提供するもの:
  ✓ 影響を受ける振る舞いの完全なリスト
  ✓ 検証すべきシナリオの完全なリスト
  ✓ 変更点の深度分類（surface / domain / infrastructure）
  ✓ マーカー密度による責務集中の警告

specre が提供しないもの（人間 + LLM の判断領域）:
  ✗ 変更が安全かどうかの判断
  ✗ リリースフラグの配置戦略の決定
  ✗ リリース可否の判断
```

specre は **判断を代替しない** が、**判断に必要な情報の漏れを構造的に防ぐ**。これが specre の責務の上限であり、同時に十分な価値提供でもある。

## ロードマップへの統合案（改訂）

`@specre-todo` を加味してロードマップ統合案を改訂する:

```
v0.4 — Drift Detection [変更なし]
  └ specre drift, specre ci, GitHub Actions template

v0.5 — Decision Support [QA Support から拡張]
  ├ specre impact        [既存計画]
  ├ specre diff          [既存計画]
  ├ specre export        [既存計画]
  ├ specre blast-radius  [新規: A-1]  ← impact + trace + git diff の合成
  ├ specre complexity    [新規: C-1]  ← impact + git history の合成
  └ MCP prompt: review   [新規: A-3]  ← blast-radius の結果を LLM に渡す

v0.5.x — Prospective Impact & Team Coordination [新規マイルストーン]
  ├ @specre-todo マーカー          [新規: D]      ← 将来変更の物理的宣言
  ├ specre plan <specre-path>     [新規]         ← LLM によるマーカー配置 + スケルトン生成 + 決定論的影響分析
  ├ specre plan --skeleton        [新規: D-2]    ← 新規スケルトンファイルの生成（コメントアウト設計）
  ├ マーカー密度チェック            [新規: D-1.5]  ← max_markers_per_file 閾値による責務集中の早期検出
  ├ specre todo-status            [新規]         ← リポジトリ全体の計画中変更の俯瞰
  ├ specre review-checklist       [新規: A-2]    ← blast-radius → checklist 変換
  ├ specre scope                  [新規: B-1]    ← search + impact + complexity の合成
  └ specre contradiction-check    [新規: B-2]    ← scenarios のクロスチェック

v0.6〜v0.7 — Multi-Repository [変更なし]
```

## 結論

specre が要件定義・コードレビューのボトルネック低減に寄与するための鍵は:

1. **決定論的なプリミティブ（impact, trace, drift）を、ワークフローレベルのコマンド（blast-radius, scope, review-checklist）に合成すること**
2. **LLM の推論の「入力」を構造化すること** — specre の価値は LLM の推論を代替することではなく、推論の前提となるデータを決定論的・網羅的に収集すること
3. **「何が仮説で、何が決定論的か」を明確に分離すること** — 混同すると、ツール全体が「当てにならない」という評価になる
4. **確率的判断と決定論的分析の明確な分離** — `@specre-todo` は LLM の確率的推論を「小さく検証可能なマーカー」に物質化し、そこから先の影響伝播を決定論的に処理する。この「半決定論的影響評価」は、LLM 時代の新しい Change Impact Analysis パターンとなりうる
5. **マーカー密度を計画段階のアーキテクチャ品質指標として活用する** — 1ファイルへのマーカー集中は責務過多の定量的シグナルであり、実装前に構造上の問題を検出できる。従来のコード品質メトリクスが「実装後の事後検出」であるのに対し、これは「計画段階の事前検出」であり、修正コストがほぼゼロの時点で介入できる

現行ロードマップの v0.5（impact, diff, export）は必要なプリミティブだが、それだけでは「要件定義が楽になった」「レビューが速くなった」という体験には直結しない。blast-radius と review-checklist でプリミティブからワークフローへの接続を作り、`@specre-todo` で retrospective から prospective への拡張を実現することで、specre は「仕様管理ツール」から「開発意思決定支援ツール」へと進化する。

さらに、`@specre-todo` をスケルトンファイルに拡張することで、specre は従来のソフトウェア開発における「設計」と「実装」の断絶を解消する。specre カード（振る舞い仕様）→ スケルトンファイル（構造設計）→ 実装コードという連続的なグラデーションがコードベース上に生まれ、全てが同じトレーサビリティグラフで繋がる。これにより:

- **レビュー責任者** は実装前にアーキテクチャレベルのフィードバックが可能になる
- **QA** は実装前にテスト計画とリスクアセスメントに参加できる（シフトレフト）
- **他チーム** はコードベース上の物理的なシグナルを通じて計画中の変更を認識できる

設計と実装が同じ場所・同じフォーマット・同じツールで管理され、設計が実装に「昇格」するだけで断絶なく移行する — これは LLM がスケルトンを高速に生成できる時代だからこそ実用的になるアプローチであり、specre の独自性を最も明確に表現する機能となりうる。
