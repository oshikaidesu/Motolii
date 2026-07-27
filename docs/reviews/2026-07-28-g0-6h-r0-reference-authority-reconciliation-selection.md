# G0-6H-R0 reference authority再照合粒の選定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-R0: **DONE**

## 1. 目的

`G0-6H-E`で現行`#plugin-browser-candidate`のnormal色5画面への肯定的応答を
限定観察として取込済みにした。次に、旧U0e-2 reference generationが固定する
React source authorityと、現行product-owned React source authorityの関係を
docs-onlyで再照合する`G0-6H-R`を選定する。

## 2. 確認した事実

- `G0-6H-E`は[G0-6H-E限定観察](2026-07-28-g0-6h-e-candidate-approval-observation.md)
  と証拠READMEを追加し、`./scripts/check-docs.sh`とBrowser decoder専用test 118件を通過した。
- [reference handoff](../mocks-ui/reference-handoff.md)はReact source authority
  `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`と旧generation
  `u0e2-08f96cbd7754-85c0fc529ab1`を固定している。
- [UI参照地図](../ui-reference-map.md)と
  `ui/motolii-web/source-provenance.json`は、現行React source assetを
  `56c318edcddab7cf95d263cc2f7dd2b4e6791134`としている。
- `eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`は
  `56c318edcddab7cf95d263cc2f7dd2b4e6791134`のGit ancestorである。
- 現行5画面の承認だけでは、旧30 PNG、派生25枚、旧route、具体token、
  G0-6H完了、U0e-3解禁を確定できない。

## 3. 次粒が閉じる一成果

`G0-6H-R`は二つの固定commitがそれぞれ何のauthorityであるかを、現行仕様、
現行コード事実、既存fixtureに照らして分類する。旧generationの再現authorityと
現行product source authorityを混同せず、次にroute裁定が必要か、派生variant供給が
必要か、または既存handoffをそのまま使えるかを一意にhandoffする。

## 4. 非目標

- `reference-handoff.md`のDecision templateまたはchecklistを埋める。
- route、画像、generation、`CURRENT`、React / CSS / Rust / fixture / test / guardを変更する。
- 旧30 PNGまたは派生variantの人間採否を推測する。
- 具体token、製品theme、公開API、Document、plugin契約、永続形式を変更する。
- `G0-6H`、`CU-0B01`、`U0e-3`を完了または解禁する。
- route裁定、variant生成、U0e-3実装を同じ粒へ束ねる。

## 5. 必須負例

- 新しいcommitを古いgenerationのsource authorityとして遡及記載する。
- 古いcommitを現行product ownerの唯一のauthorityとして扱う。
- Git ancestryだけをvisual parityまたは人間承認の証明にする。
- `check-reference`成功を現行5画面との同一性や人間審判の代替にする。
- 現行候補のnormal色承認を旧派生25枚へ拡張する。

## 6. STOP条件

1. authorityの分類にroute、公開契約、Document、plugin契約、永続形式の変更が必要になる。
2. 未監査sourceまたは新しい画像生成を根拠にしないと分類できない。
3. 旧generationの期待値、threshold、golden変更が必要になる。
4. 現行候補と旧referenceのvisual同一性を証拠なしに仮定する必要がある。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-E` | **DONE** | 現行候補normal色5画面への肯定的応答を限定観察として取込 |
| `G0-6H-R0` | **DONE** | 次のdocs-only authority再照合粒を選定 |
| `G0-6H-R` | **DO** | 旧generation authorityと現行product source authorityを分類 |
| `G0-6H` | **DO / HUMAN** | 旧generationを含む人間審判は未完了 |
| `CU-0B01` | **HUMAN / WAIT** | 据え置き |
| `U0e-3` | **WAIT** | 据え置き |
