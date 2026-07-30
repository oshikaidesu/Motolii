# CU-0A08SS0 Browser Place source seam の最小実装境界 選定範囲

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SS0: **DONE**

## 1. 目的

`CU-0A08BDD`が採択したBrowser source-seam firstに対し、最小の最初の実装境界をどこに置くかという一問だけを、docs-only `CU-0A08SSD` が閉じる範囲として選定する。本粒はその問いに**答えない**。

## 2. 事実

- `CU-0A08BDD` / `CU-0A08BD0` / `CU-0A08RMD` は [発注依存証跡](../implementation-ledger.md#発注依存証跡) で `DONE` である。
- [CU-0A08BDD裁定 §3](2026-07-29-cu-0a08bdd-browser-typed-intent-dependency-direction-decision.md#3-裁定)は、Browser source-seam firstをVS-1 Rectangleに限定採択した。
- `CU-0A08BT`は`CU-110`に依存せず、Placeの意味、lifecycle、commitは既決ownerに残る。
- bare `itemId` drag payloadは `S`（open defect）のままである。
- `CU-0A08BT` / `CU-0A08IT` / `CU-0A08RM` / `U2c-2` / `U3a-2Q-V` は `WAIT` である。

## 3. CU-0A08SSD が閉じる唯一の問い

`CU-0A08BDD` が採択した Browser source-seam first に対し、最小の最初の実装境界を次のどちらにするか。

## 4. 可能な候補（優劣を付けない）

**(A) product-owned React Browser gesture / source seam**

既存 Rectangle `ElementCard` / Browser component 境界に置き、Host transportとD2の手前で止める。

**(B) Host-side adapter seam**

既存Browser gestureをconsumeし、product React presentationは触らない。同じくD2の手前で止める。

## 5. 非目標

- §3へ答える、候補を重み付けする、推奨・採択を示すこと。
- callback、event、payload、型、API、module、export、crate、wire、transportを命名または新設すること。
- bare `itemId`を有効な契約として肯定すること。
- source asset、BT/IT/RM行、W0/W1表、M3仕様、`U4a-2`、Place ownerを変更すること。
- 公開API、Document、journal、plugin契約、serde、永続形式、Undo単位を決めること。
- VS-1 Rectangle以外へ一般化すること。
- Rust / JS / JSX / CSS / fixture / guard / schema / goldenを変更すること。

## 6. 必須負例

- 候補(A)/(B)に選好語を付ける。
- §3を二問以上に増やす。
- callback、event、payload、型、API、module、export、crate、wire、transportを具体化する。
- bare `itemId`の `S` を解消済みにする。
- `CU-110`を`CU-0A08BT`の直接前提にする。
- `CU-0A08BT`がPlace意味・lifecycle・commitを所有すると書く。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SSD`を発注依存証跡へ追加する。

## 7. 同期した current mirror

同じ7箇所を、`CU-0A08SS0` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SSD`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SS0` | **DONE** | Browser Place source seamの最小実装境界を一問へ限定 |
| `CU-0A08SSD` | **DO** | §3の一問だけをdocs-onlyで閉じる |
| `CU-0A08RM` | **WAIT** | seam境界裁定後にmirror修復を再開 |

## 10. STOP条件

1. §3を記録するために回答、候補の優劣、第二の問いが必要になる。
2. 契約の型、callback、event、payload、API、module、exportを先に決める必要がある。
3. BT/IT/RM行、source asset、W0/W1表、M3仕様、公開契約の変更が必要になる。
4. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSD` の1件に保てない。
