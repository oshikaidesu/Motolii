# CU-0A08RMD Browser typed-intent 依存裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08RMD: **DONE**

## 1. 目的

[CU-0A08RM0選定 §3](2026-07-29-cu-0a08rm0-browser-typed-intent-dependency-adjudication-scope-selection.md#3-cu-0a08rmd-が閉じる唯一の問い)について、`CU-0A08BT` の typed-intent 半分が VS-1 Rectangle Place か、`U4a-2` に依存する別の Direct-entry 責任かを一問だけ裁定する。

## 2. 事実

- VS-1 の正常flowは、product-owned BrowserでRectangleを選び、Stageへ配置previewし、release時だけD2 single writerへcommitし、Stage / Timeline / React Inspectorを同じrevisionとLayerIdへ収束させる。
- `U4a-2` は Effect Inspector の保存param編集、nonblocking preview、1 gesture=1 Undoを所有し、Browser Create dragは所有しない。
- Browser製品sourceにはdraggableなRectangle `ElementCard` があるが、現行drag dataはbare `itemId`だけである。
- `CU-G09` はdrag-payload意味を `S`（open defect）に保っており、bare `itemId`を契約として確定していない。
- 製品Place intent、公開API、production callerは存在しない。
- Placeの意味とlifecycleは既決の `CU-101` / `CU-102` と `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` が所有する。
- `CU-110` はnon-test production drop sourceを待っている。

## 3. 裁定

候補 **(A)** を **VS-1 Rectangleに限って**採択する。

`CU-0A08BT` の typed-intent 半分は、確立済みPlace責任連鎖へのBrowser側Place source / adapterであり、`U4a-2` Direct-entryの責任ではない。Placeの意味、lifecycle、commitは既存ownerが持ち続け、Browser側に新しいPlace意味を作らない。

一方、`CU-110` を `CU-0A08BT` の直接前提として記録すると、`CU-110` のproduction drop source待ちと循環する。したがって依存の向きと実装順は本粒で決めず、docs-only `CU-0A08BD0`へ送る。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` の `WAIT` と記録済み依存セル。
- W0/W1表、M3仕様、`U4a-2` の意味・順序・完了条件。
- `CU-101` / `CU-102` / `CU-107*` / `CU-110` の既存責任。
- bare `itemId` drag payloadの `S`。本粒は祝福も否定もせず、payload、wire、event shape、型名を決めない。

## 5. 非目標

- `CU-0A08BT` とPlace連鎖の依存の向き、実装順、前提記録の形を決めること。
- `CU-110` を `CU-0A08BT` の直接前提にすること。
- `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `U3a-2Q-V` の状態、依存セル、責任分割を変更すること。
- event shape、WebView wire、Host transport名、typed intent型・名前・payload、drag payload、公開API、Document、journal、plugin契約、serde、永続形式を決めること。
- Browserの全card種別または全catalog itemへ一般化すること。
- Rust / JS / JSX / CSS / fixture / guard / schema / goldenを変更すること。

## 6. 必須負例

- `CU-110` を `CU-0A08BT` の直接前提として記録する。
- bare `itemId` を有効な契約として肯定する。
- VS-1 Rectangle以外へ裁定を一般化する。
- `CU-0A08BT` / `CU-0A08IT` 行、W0/W1表、M3仕様、公開契約を変更する。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08BD0` を発注依存証跡へ追加する。

## 7. 同期した current mirror

RM0と同じ7箇所を、`CU-0A08RMD` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08BD0`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

次のrolling文書は本粒で変更しない。

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08RMD` | **DONE** | 候補(A)をVS-1 Rectangleに限定して採択 |
| `CU-0A08BD0` | **DO** | `CU-0A08BT` とPlace連鎖の依存の向き・実装順を決める一問だけを選定 |
| `CU-0A08RM` | **WAIT** | 依存方向の裁定後にmirror修復を再開 |

## 10. STOP条件

1. 依存の向き・実装順を同時に決めないと本裁定を記録できない。
2. `CU-110` を `CU-0A08BT` の直接前提にしないと整合しない。
3. bare `itemId` または新しいAPI / event / payloadを契約化しないと閉じない。
4. VS-1 Rectangle以外へ一般化しないと裁定を書けない。
5. allowlist外の変更が必要になる。
