# CU-0A08SSCD Browser Place source seam 契約具体化 裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSCD: **DONE**

## 1. 目的

[CU-0A08SSC選定 §3](2026-07-29-cu-0a08ssc-browser-place-source-seam-contract-concretization-scope-selection.md#3-cu-0a08sscd-が閉じる唯一の問い)について、scoped identityをRectangleのPlace source要求へ渡す責任を置く既存component境界を一問だけ裁定する。

## 2. 事実

- `ElementCard`は既存`identity` propsとDOM属性を持つが、drag開始時はbare `itemId`だけを書き、drop終端を持たない。
- `CandidateCreateBrowser`の`elementProps`が各`ElementCard`へpropsを配る既存の唯一の所有点である。
- Rectangle cardは`identity`を渡しておらず、既存JSXの`identity` literalはCU-G09のscoped identityと同一物ではない。
- Browser catalog identityは `(scope_ref, item_id)` であり、それ以外から意味を導かない。
- top-levelの現行`developmentProjection`はMedia用で、Browser catalog入力境界はまだ決まっていない。

## 3. 裁定

候補 **(B) `CandidateCreateBrowser` component境界** をVS-1 Rectangleに限って採択する。

Browser側の既存配布所有点でscoped identityを受け、既存`elementProps`からcardへ配る責任を置く。`ElementCard` leafはpresentationとdrag interactionを維持し、Host projectionの所有点にしない。

これは責任境界の位置だけの裁定であり、callback、event、payload、型、API、module、export、wire、transport、新props名の決定でも命名でもない。現行authorityだけではBrowser catalog入力境界を閉じた実装orderへ落とせないため、次粒はdocs-only `CU-0A08SSCS`とする。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態セルと依存セル。
- W0/W1表、M3仕様、`U4a-2`の意味、順序、完了条件。
- `CU-101` / `CU-102` / `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` のPlace意味、lifecycle、commit責任。
- bare `itemId` drag payloadとJSX `identity` literalの `S`。
- source assetのbyte、公開API、Document、journal、plugin契約、serde、永続形式、Undo単位、guard期待値。

## 5. 非目標

- 新しいcallback、event、payload、型、API、module、export、wire、transport、props名を決めること。
- bare `itemId`またはJSX `identity` literalをscoped identity契約として肯定すること。
- Place lifecycle、drop終端、commit ownerをBrowser側へ移すこと。
- BT/IT/RM行、M3仕様、source asset、公開契約を変更すること。
- VS-1 Rectangle以外へ一般化すること。
- 実装を起動すること。

## 6. 必須負例

- 候補(A)と(B)を両方採択する。
- `ElementCard` leafをHost projection ownerにする。
- `developmentProjection`の拡張形、入力key、新props名を発明する。
- bare `itemId`の `S` を解消済みにする。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SSCS`を発注依存証跡へ追加する。

## 7. 同期した current mirror

同じ7箇所を、`CU-0A08SSCD` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SSCS`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCD` | **DONE** | CandidateCreateBrowser境界をVS-1 Rectangleに限定採択 |
| `CU-0A08SSCS` | **DO** | Browser catalog入力境界を発明せず、最小closed implementation orderの範囲を選定 |
| `CU-0A08RM` | **WAIT** | source seam実装境界確定後にmirror修復を再開 |

## 10. STOP条件

1. 型、callback、event、payload、API、module、export、wire、transport、新props名の決定が必要になる。
2. bare `itemId`を契約化しないと文が閉じない。
3. Place owner、BT/IT/RM行、M3仕様、公開契約、source assetの変更が必要になる。
4. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSCS` の1件に保てない。
