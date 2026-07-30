# CU-0A08BDD Browser typed-intent 依存方向の裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08BDD: **DONE**

## 1. 目的

[CU-0A08BD0選定 §3](2026-07-29-cu-0a08bd0-browser-typed-intent-dependency-direction-scope-selection.md#3-cu-0a08bdd-が閉じる唯一の問い)について、`CU-0A08BT` と既決Place責任連鎖の非循環な実装方向を一問だけ裁定する。

## 2. 事実

- `CU-0A08BD0` はBrowser source-seam firstとshared internal contract prerequisiteを優劣なく候補化した。
- `CU-0A08RMD` はVS-1 Rectangleに限り、`CU-0A08BT` typed-intent半分を既決Place責任連鎖へのBrowser側Place source / adapterとした。
- Placeの意味、lifecycle、commitは `CU-101` / `CU-102` と `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` が所有する。
- `CU-110` はnon-test production drop sourceを待っている。
- bare `itemId` drag payloadは `S`（open defect）のままである。

## 3. 裁定

候補 **(A) Browser source-seam first** をVS-1 Rectangleに限って採択する。

`CU-0A08BT` は `CU-110` に依存しない。先に狭く段階化されたBrowser側のnon-test Place source seamが成立し、その後 `CU-110` が既決Place前提と併せてそのseamをconsumeする。

これは実装方向だけの裁定であり、event、payload、型、API、wireの決定ではない。`CU-0A08BT`がPlaceの意味、lifecycle、commitを所有する主張でもない。`CU-110`を`CU-0A08BT`の直接前提として記録しないことで依存循環を避ける。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態と依存セル。
- W0/W1表、M3仕様、`U4a-2`、既決Place ownerの責任。
- bare `itemId` drag payloadの `S`。

## 5. 非目標

- Browser source seamの型、名前、event shape、payload、wire、transport、module、crate、exportを決めること。
- BT/IT/RM行、W0/W1表、M3仕様、公開API、Document、journal、plugin契約、serde、永続形式を変更すること。
- bare `itemId`を有効な契約として肯定すること。
- VS-1 Rectangle以外へ一般化すること。
- Rust / JS / JSX / CSS / fixture / guard / schema / goldenを変更すること。

## 6. 必須負例

- `CU-110`を`CU-0A08BT`の直接前提にして循環を作る。
- `CU-0A08BT`がPlaceの意味、lifecycle、commitを所有すると書く。
- 具体的なevent、payload、wire、transport、型、APIを新設する。
- bare `itemId`の `S` を解消済みにする。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SS0`を発注依存証跡へ追加する。

## 7. 同期した current mirror

BD0と同じ7箇所を、`CU-0A08BDD` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SS0`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08BDD` | **DONE** | Browser source-seam firstをVS-1 Rectangleに限定採択 |
| `CU-0A08SS0` | **DO** | Browser source seamの最小実装境界を特定する一問を選定 |
| `CU-0A08RM` | **WAIT** | source seam境界裁定後にmirror修復を再開 |

## 10. STOP条件

1. 契約の型、event、payload、API、module境界を先に決めないと方向を記録できない。
2. `CU-110`を`CU-0A08BT`の直接前提にしないと整合しない。
3. BT/IT/RM行、W0/W1表、M3仕様、公開契約の変更が必要になる。
4. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SS0` の1件に保てない。
