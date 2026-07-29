# CU-0A08SSCI-I0 Browser scoped identity input seam 選定

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08SSCI-I0**

## 1. 目的

[CU-0A08SSCI-P1 guard整合](2026-07-29-cu-0a08ssci-p1-browser-post-promotion-provenance-chain-guard-reconciliation-decision.md)で
前提(P)がauthorityとguardの両面で閉じたため、未採番前提(I)を
`CU-0A08SSCI-I0`として選定する。本粒は次のdocs-only裁定が閉じる一問だけを固定し、
その問いには答えない。

## 2. 既決事実

- scoped identityはdecode済みの`(scope_ref, item_id)`であり、bare `itemId`や既存JSX
  `identity` literalから推測しない。
- 責任境界はmodule-private `CandidateCreateBrowser`、既存配布所有点は
  `elementProps`である。
- 対象はVS-1 Rectangleだけ。raw input/decode、Host transport、D2、drop終端は含めない。
- `CU-0A08SSCI`は前提(I)/(T)が閉じるまで`WAIT`。(T)は未採番のまま。

## 3. 次粒が閉じる唯一の問い

`CandidateCreateBrowser`がdecode済み`(scope_ref, item_id)`を受ける最小private input seamの
契約形を、既存React component境界内でどこまでに限定するか。

次のdocs-only `CU-0A08SSCI-I`が候補を並記して一つを裁定する。型、callback、event、
payload、props名、module、export、wire、transport、decoder名は本粒で決めない。

## 4. 変わらないもの

- React source、guard、`source-provenance.json`実データ、公開API。
- Document、journal、serde、永続形式、Undo単位、plugin契約、Place owner。
- bare `itemId` drag payloadとJSX `identity` literalの`S`。
- `CU-0A08SSCI`の`WAIT`、(T)の未採番。

## 5. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCI-I0` | **DO** | (I)を採番し、次粒が閉じる唯一の問いを選定 |
| `CU-0A08SSCI-I` | 未登録 | I0完了後のdocs-only裁定粒 |
| `CU-0A08SSCI` | **WAIT** | (I)/(T)未完了 |

## 6. STOP条件

公開API、Document、永続形式、plugin契約、Place ownerへ波及する／bare IDやmock literalを
scoped identityとして肯定する／React sourceやguardを変更する／(T)を同時採番する必要が
生じた場合は停止する。
