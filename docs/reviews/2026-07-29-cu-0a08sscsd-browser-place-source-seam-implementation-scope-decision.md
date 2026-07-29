# CU-0A08SSCSD Browser Place source seam 実装範囲 裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSCSD: **DONE**

## 1. 目的

[CU-0A08SSCS選定 §3](2026-07-29-cu-0a08sscs-browser-place-source-seam-implementation-scope-selection.md#3-cu-0a08sscsd-が閉じる唯一の問い)について、最小closed implementation orderが閉じる実装範囲を一問だけ裁定する。

## 2. 事実

- `CandidateCreateBrowser`はpropsを受けず、`elementProps`が各cardへpropsを配る既存所有点である。
- Rectangle cardはscoped identityを受けていない。
- `ElementCard`のdrag開始はbare `itemId`だけを書き、drop終端を持たない。
- top-level `developmentProjection`はMedia専用で、Browser catalog raw入力境界は現行authorityに存在しない。
- Browser catalog identityは `(scope_ref, item_id)` であり、それ以外から意味を導かない。

## 3. 裁定

候補 **(A) 内部source seamのみ** をVS-1 Rectangleに限って採択する。

decode済みBrowser catalog identityを`CandidateCreateBrowser`が受け、Rectangleだけを既存`elementProps`から`ElementCard`へ配る範囲で閉じる。raw input / decode、Host transport、D2、drop終端は含めない。

これは実装範囲だけの裁定であり、callback、event、payload、型、API、module、export、wire、transport、新props名の決定でも命名でもない。次は最小コード実装粒`CU-0A08SSCI`とする。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態と依存セル。
- W0/W1表、M3仕様、`U4a-2`。
- `CU-101` / `CU-102` / `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` のPlace意味、lifecycle、commit責任。
- bare `itemId` drag payloadとJSX `identity` literalの `S`。
- source asset、公開API、Document、journal、plugin契約、serde、永続形式、Undo単位、guard期待値。

## 5. 非目標

- raw入力/decode、Host transport、D2、drop終端を実装範囲へ含めること。
- 新しい型、callback、event、payload、API、module、export、wire、transport、props名を決めること。
- bare IDまたはJSX literalをscoped identity契約として肯定すること。
- Place owner、WAIT行、source asset、公開契約を変更すること。
- VS-1 Rectangle以外へ一般化すること。

## 6. 必須負例

- 候補(A)と(B)を両方採択する。
- top-level raw入力形またはdecoder名を発明する。
- bare IDの `S` を解消済みにする。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SSCI`を発注依存証跡へ追加する。

## 7. 同期した current mirror

同じ7箇所を、`CU-0A08SSCSD` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` は実装粒 `CU-0A08SSCI`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCSD` | **DONE** | 内部source seamのみをVS-1 Rectangleに限定採択 |
| `CU-0A08SSCI` | **DO** | decode済みidentityをCandidateCreateBrowserからRectangle cardへ配る最小コード実装 |
| `CU-0A08RM` | **WAIT** | source seam実装後にmirror修復を再開 |

## 10. STOP条件

1. raw入力形、型、event、payload、API、新props名を先に決める必要がある。
2. bare IDを契約化しないと文が閉じない。
3. Place owner、WAIT行、公開契約、source assetの変更が必要になる。
4. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSCI` の1件に保てない。
