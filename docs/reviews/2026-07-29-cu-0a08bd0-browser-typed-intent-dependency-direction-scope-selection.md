# CU-0A08BD0 Browser typed-intent 依存方向の選定範囲

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08BD0: **DONE**

## 1. 目的

`CU-0A08BT` と既決Place連鎖の非循環な実装方向について、docs-only `CU-0A08BDD` が閉じる範囲を一問だけに限定する。本粒はその問いに**答えない**。

## 2. 事実

- `CU-0A08RMD` / `CU-0A08RM0` / `CU-0A08RS` は [発注依存証跡](../implementation-ledger.md#発注依存証跡) で `DONE` である。
- [CU-0A08RMD裁定 §3](2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md#3-裁定)は、VS-1 Rectangleに限り、`CU-0A08BT` typed-intent半分を既決Place責任連鎖へのBrowser側Place source / adapterとした。
- 同裁定はPlaceの意味、lifecycle、commitを `CU-101` / `CU-102` と `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-110` に残した。
- `CU-110` はnon-test production drop sourceを待つため、`CU-110` を `CU-0A08BT` の直接前提にすると循環する。
- bare `itemId` drag payloadは `S`（open defect）のままである。
- `CU-0A08RM` / `CU-0A08BT` / `CU-0A08IT` / `U2c-2` / `U3a-2Q-V` は `WAIT` である。

## 3. CU-0A08BDD が閉じる唯一の問い

`CU-0A08BT` と既決Place連鎖の非循環な実装方向を、次のどちらにするか。

## 4. 可能な候補（優劣を付けない）

**(A) Browser source-seam first**

先にBrowser側source seamを成立させ、後に確立するnon-test sourceを `CU-110` がconsumeする。`CU-0A08BT` は `CU-110` に依存しない。

**(B) shared internal contract prerequisite**

両側に先行する、より狭い共有内部contractを閉じる。その型やAPIは本粒で発明しない。

## 5. 非目標

- §3へ答える、候補を重み付けする、推奨または採択を示すこと。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` の状態または依存セルを変更すること。
- `CU-110` を `CU-0A08BT` の直接前提にすること。
- `U4a-2`、Place owner、W0/W1表、M3仕様の意味・順序・完了条件を変更すること。
- event shape、WebView wire、Host transport名、typed-intent型・名前・payload、drag payload、公開API、Document、journal、plugin契約、serde、永続形式を決めること。
- bare `itemId`を契約として肯定する、またはVS-1 Rectangle以外へ一般化すること。
- Rust / JS / JSX / CSS / fixture / guard / schema / goldenを変更すること。

## 6. 必須負例

- 候補(A)/(B)の推奨、採択、優先、望ましい等の選好を書く。
- §3を二問以上に増やす。
- `CU-110`を`CU-0A08BT`の直接前提として記録する。
- 具体的なevent、payload、wire、transport、型、APIを新設する。
- BT/IT/RM行、W0/W1表、M3仕様、guard期待値を変更する。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08BDD`を発注依存証跡へ追加する。

## 7. 同期した current mirror

次の7箇所を、`CU-0A08BD0` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08BDD`（1件）へ同期した。

1. `docs/implementation-ledger.md` 現在地表 M3 行
2. 同文書 M3への入場判定の運用判断
3. `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` journal durability 行
4. 同文書 selection / Undo再投影 行
5. `docs/decision-index.md` M3 VS-1 縦slice 主題行
6. 同文書 CU-110S〜CU-107W 主題行
7. 同文書 CU-0A08RS0〜CU-0A08BD0 主題行

## 8. allowlist外に残る stale mirror

次のrolling文書は本粒で変更しない。

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08BD0` | **DONE** | 非循環な実装方向の裁定範囲を一問へ限定 |
| `CU-0A08BDD` | **DO** | §3の一問だけをdocs-onlyで閉じる |
| `CU-0A08RM` | **WAIT** | 依存方向裁定後にmirror修復を再開 |

## 10. STOP条件

1. §3の一問を記録するために回答または第二の問いが必要になる。
2. BT/IT/RM行、`U4a-2`、W0/W1表、M3仕様の変更が必要になる。
3. 公開API、Document、永続形式、journal、plugin契約、event shapeの決定が必要になる。
4. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08BDD` の1件に保てない。
5. allowlist外のcurrent next-DO mirrorを直さないと整合しない。
