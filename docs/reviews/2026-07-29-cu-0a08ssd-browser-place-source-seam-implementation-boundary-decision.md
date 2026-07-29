# CU-0A08SSD Browser Place source seam の最小実装境界 裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSD: **DONE**

## 1. 目的

[CU-0A08SS0選定 §3](2026-07-29-cu-0a08ss0-browser-place-source-seam-implementation-boundary-scope-selection.md#3-cu-0a08ssd-が閉じる唯一の問い)について、VS-1 RectangleのBrowser Place source seamで最小の最初の実装境界をどこに置くかをdocs-onlyで裁定する。

## 2. 事実

- product-owned React source `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx` に、draggableなRectangle `ElementCard`とBrowser component境界が実在する。
- 現行drag dataは `application/x-motolii-browser-item` にbare `itemId`を入れるだけであり、[CU-G09決定](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md)ではdrag payload意味を `S` のまま残している。
- `crates/` にWebView Host transport実装は存在せず、Host-side adapter seamを先に置くには未決のtransport、wire、payloadを新設する必要がある。
- [React製品資産の直接移管契約 §6.2](2026-07-22-m3-react-product-asset-promotion-contract.md#62-交換する)は、product-owned React componentからtyped intentを経てHost coordinator / D2 single writerへ至る正規経路を定め、同契約の実行順はprojection / intent交換をWebView Hostより前に置いている。
- `CU-0A08BDD`はBrowser source-seam firstをVS-1 Rectangleに限定採択し、`CU-0A08RMD`はtyped-intent半分を既決Place責任連鎖へのBrowser側Place source / adapterと裁定している。

## 3. 裁定

候補 **(A) product-owned React Browser gesture / source seam** をVS-1 Rectangleに限って採択する。

最小の最初の実装境界は、既存のproduct-owned React Browser component内のRectangle `ElementCard` / Browser component境界に置く。Host transportとD2 single writerの手前で止める。

これは境界の位置だけの裁定であり、callback、event、payload、型、API、module、export、wire、transportの決定でも命名でもない。Placeの意味、lifecycle、commit ownerは既決ownerのまま変えない。bare `itemId`は `S`（open defect）のままで、有効な契約として肯定しない。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態と依存セル。
- W0/W1表、M3仕様、`U4a-2`の意味、順序、完了条件。
- `CU-101` / `CU-102` / `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` のPlace責任。
- bare `itemId` drag payloadの `S`。
- source assetのbyte、公開API、Document、journal、plugin契約、serde、永続形式、Undo単位、guard期待値。

## 5. 非目標

- callback、event、payload、型、API、module、export、crate、wire、transportを命名、新設、具体化すること。
- bare `itemId`を有効な契約として肯定すること。
- Placeの意味、lifecycle、commit ownerを変更またはBrowser側へ移すこと。
- `CU-110`を`CU-0A08BT`の直接前提として記録すること。
- BT/IT/RM行、W0/W1表、M3仕様、`U4a-2`を変更すること。
- VS-1 Rectangle以外へ一般化すること。
- Rust / JS / JSX / CSS / fixture / guard / schema / golden / package.jsonを変更すること。
- 実装粒を起動し、`WAIT`を解除すること。

## 6. 必須負例

- callback、event、payload、型、API、module、export、crate、wire、transportの具体名を新設する。
- bare `itemId`の `S` を解消済みまたは解決済みと書く。
- `CU-110`を`CU-0A08BT`の直接前提にする。
- `CU-0A08BT`がPlaceの意味、lifecycle、commitを所有すると書く。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態セルまたは依存セルを変える。
- allowlist外のstale mirror、guard期待値、threshold、固定SHA、golden、fixtureを変更する。
- rolling mirrorの一部だけを同期する。
- TODO、TBD、空節を残す。
- VS-1 Rectangle以外へ裁定を一般化する。

## 7. 同期した current mirror

SS0と同じ7箇所を、`CU-0A08SSD` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SSC`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSD` | **DONE** | product-owned React Browser source seamをVS-1 Rectangleに限定採択 |
| `CU-0A08SSC` | **DO** | 採択済みsource seamの契約具体化について、次に閉じる一問だけを選定 |
| `CU-0A08RM` | **WAIT** | source seam契約具体化後にmirror修復を再開 |

## 10. STOP条件

1. 裁定を記録するために、型、callback、event、payload、API、module、export、wire、transportを先に決める必要が生じる。
2. bare `itemId`を契約化しないと文が閉じない。
3. `CU-110`を`CU-0A08BT`の直接前提にしないと整合しない、または依存循環が生じる。
4. BT/IT/RM行、W0/W1表、M3仕様、`U4a-2`、公開契約、Place ownerの変更が必要になる。
5. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSC` の1件に保てない。
6. source assetまたはguard期待値の変更が必要になる。
