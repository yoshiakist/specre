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

## 結論

specre が要件定義・コードレビューのボトルネック低減に寄与するための鍵は:

1. **決定論的なプリミティブ（impact, trace, drift）を、ワークフローレベルのコマンド（blast-radius, scope, review-checklist）に合成すること**
2. **LLM の推論の「入力」を構造化すること** — specre の価値は LLM の推論を代替することではなく、推論の前提となるデータを決定論的・網羅的に収集すること
3. **「何が仮説で、何が決定論的か」を明確に分離すること** — 混同すると、ツール全体が「当てにならない」という評価になる

現行ロードマップの v0.5（impact, diff, export）は必要なプリミティブだが、それだけでは「要件定義が楽になった」「レビューが速くなった」という体験には直結しない。blast-radius と review-checklist を v0.5 に組み込むことで、プリミティブからワークフローへの接続が生まれる。
