# CU-0A08SSCI-T1 Browser private component verification harness 実装決定

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-T1**

## 1. 目的

`CU-0A08SSCI-T`で採択したAST静的検査境界に、module-private
`CandidateCreateBrowser`の単一object inputと、VS-1 Rectangle 1件だけへの
2-field identity非推測透過を検査するsynthetic正負harnessを実装する。

## 2. 着手前code fact

- product Browser sourceの`CandidateCreateBrowser`はmodule-private・引数0である。
- decode済みprojectionは`CandidateProjectBrowser`へだけ届き、通常の
  `CandidateCreateBrowser`へ届く供給routeは未決である。
- ownership guardは既存`@babel/parser`、`parseModule`、
  `collectNamedExports`とsynthetic正負試験patternを持つ。

## 3. 実装対応

| 条件 | 対応 |
|---|---|
| S-1 | top-level functionが1件でexportされないことを検査 |
| S-2 | 引数1件かつ`ObjectPattern`を検査 |
| S-3 | 呼出側が渡す2 fieldとの完全一致、rest/default/nest不在を検査 |
| S-4 | 各bindingの参照がちょうど1件であることを検査 |
| S-5 | JSX属性値またはRectangle `elementProps`直接引数だけを許可 |
| S-6 | 2 fieldが同じ`ElementCard element="rectangle"`へ届くことを検査 |
| P-A | JSX属性による直接透過を`assert.doesNotThrow`で確認 |
| P-B | `elementProps("rectangle", ...)`による透過を`assert.doesNotThrow`で確認 |
| N-1 | exported componentを拒否 |
| N-2〜N-3 | 引数0件と非object引数を拒否 |
| N-4〜N-8 | field過不足、rest、default、catalog全体入力を拒否 |
| N-9〜N-10 | member lookupとmap lookupを拒否 |
| N-11〜N-13 | fallback、nullish default、条件分岐を拒否 |
| N-14〜N-15 | Document、selection、Undo正本化を拒否 |
| N-16 | 非Rectangle cardへの透過を拒否 |
| N-17 | bare `itemId`による全card一般化を拒否 |
| N-18 | 2 fieldの別card分散とliteral代替を拒否 |

identity field名はvalidator引数とtest-local placeholderに限定し、製品sourceには
validatorを適用しない。

## 4. 変わらないもの

product React source、公開export、DOM/CSS、provenance実データ、既存hash literal、
既存3 test、`validatePostPromotionChanges`、Document、journal、serde、永続形式、
Undo単位、plugin契約、Place ownerは変更しない。

## 5. 非目標

供給route、props、型、event、payload、wire、transport、decoder名を決めること、
Host/D2/drop終端を接続すること、Rectangle以外へ一般化することは含めない。

## 6. 必須負例

N-1〜N-18を1つのtest内で独立label付き`assert.throws`として実装した。
bare ID推測、lookup/map、fallback/default/分岐、Document・selection・Undo正本、
非Rectangle透過、複数参照、別card分散を許可しない。

## 7. 検証

```text
NODE_PATH=/Users/member_ottoto/rust_ae/Motolii/spikes/g0-9-web-ui/node_modules \
  node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs
1..4
# tests 4
# pass 4
# fail 0

NODE_PATH=/Users/member_ottoto/rust_ae/Motolii/spikes/g0-9-web-ui/node_modules \
  node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs
1..118
# tests 118
# pass 118
# fail 0

NODE_PATH=/Users/member_ottoto/rust_ae/Motolii/spikes/g0-9-web-ui/node_modules \
  node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs
1..39
# tests 39
# pass 39
# fail 0

./scripts/check-docs.sh
OK: docs整合チェック全項目通過

git diff --check
（出力なし、exit 0）

shasum -a 256 ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx
866124a69caaa168fa19c67e6c723db97fec67a61071bdbe66973576266c42f4

git diff --stat -- ui/motolii-web/src docs/mocks-ui \
  ui/motolii-web/package.json ui/motolii-web/source-provenance.json
（出力なし、product source / mocks-ui / package / provenanceの変更0）
```

## 8. 同期した current mirror

ORACLE-GUARD `CU-0A08SSCI-T1`は`DONE`。`(T)`はauthorityとguard実装の
両面で閉じた。次の唯一の`DO`は未選定（完全一致`DO`は0件）。
`CU-0A08SSCI`は`WAIT`継続。PRODUCT-ASSETの製品実装（コード変更を伴う）
完全一致`DO`は0件。

## 9. handoff

`CU-0A08SSCI-T1`は**DONE**。供給routeは通常`CandidateCreateBrowser`へ未到達のため、
`CU-0A08SSCI`を`DO`へ上げない。route/producerを閉じる別のdocs前提が必要だが、
本粒では採番・選定しない。

## 10. STOP条件

公開component契約またはprojection-only分岐の意味を変えないと供給routeが閉じない間は、
製品React実装を開始しない。公開API、Document、Undo、plugin契約、Place ownerへ
波及する場合も停止する。
