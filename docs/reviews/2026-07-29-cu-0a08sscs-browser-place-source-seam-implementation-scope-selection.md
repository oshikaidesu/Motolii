# CU-0A08SSCS Browser Place source seam 実装範囲 選定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSCS: **DONE**

## 1. 目的

`CU-0A08SSCD`が採択した`CandidateCreateBrowser`境界を最小closed implementation orderへ落とすため、次のdocs-only `CU-0A08SSCSD`が閉じる唯一の実装範囲の問いを選定する。本粒はその問いに**答えない**。

## 2. 事実

- `CandidateCreateBrowser`はpropsを受けず、既存`elementProps`が各`ElementCard`へpropsを配る。
- Rectangle cardはscoped identityを受けていない。
- `ElementCard`のdrag開始はbare `itemId`だけを書き、drop終端を持たない。
- top-level `DiscoveryBrowserCandidate`の現行`developmentProjection`はMedia専用で、Browser catalogのraw入力境界は未決である。
- Browser catalog identityは `(scope_ref, item_id)` であり、それ以外から意味を導かない。

## 3. CU-0A08SSCSD が閉じる唯一の問い

`CU-0A08SSCD`が採択した`CandidateCreateBrowser`境界を最小closed implementation orderへ落とすとき、そのorderが閉じる唯一の実装範囲をどこまでとするか。

## 4. 可能な候補（優劣を付けない）

**(A) 内部source seamのみ**

既にdecode済みのBrowser catalog identityを`CandidateCreateBrowser`が受け、VS-1 Rectangleだけを既存`elementProps`から`ElementCard`へ配る。`DiscoveryBrowserCandidate`のraw input / decode、Host transport、D2、drop終端は含めない。

**(B) top-level入力から同時配線**

`DiscoveryBrowserCandidate` top-levelのraw Browser catalog入力 / decodeから`CandidateCreateBrowser`までを同時に配線する。

## 5. 非目標

- §3へ答える、候補を重み付けする、推奨・採択を示すこと。
- callback、event、payload、型、API、module、export、crate、wire、transport、新props名を命名または新設すること。
- bare `itemId`またはJSX `identity` literalをscoped identity契約として肯定すること。
- Placeの意味、lifecycle、drop終端、commit ownerをBrowser側へ移すこと。
- BT/IT/RM行、M3仕様、source asset、公開契約を変更すること。
- VS-1 Rectangle以外へ一般化すること。
- 実装粒を起動すること。

## 6. 必須負例

- 候補(A)/(B)に選好語を付ける。
- §3を二問以上へ増やす。
- `developmentProjection`の拡張形、入力key、Browser用decoder名、新props名を発明する。
- bare `itemId`またはJSX `identity` literalの `S` を解消済みにする。
- Place lifecycleまたはdrop終端をBrowser側の責任へ移す。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SSCSD`を発注依存証跡へ追加する。

## 7. 同期した current mirror

同じ7箇所を、`CU-0A08SSCS` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SSCSD`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSCS` | **DONE** | 最小実装orderの範囲を一問へ限定 |
| `CU-0A08SSCSD` | **DO** | §3の一問だけをdocs-onlyで閉じる |
| `CU-0A08RM` | **WAIT** | source seam実装範囲裁定後にmirror修復を再開 |

## 10. STOP条件

1. §3を記録するために回答、候補の優劣、第二の問いが必要になる。
2. 型、callback、event、payload、API、module、export、wire、transport、新props名を先に決める必要がある。
3. bare `itemId`またはJSX `identity` literalを契約化しないと文が閉じない。
4. Place owner、BT/IT/RM行、M3仕様、公開契約、source assetの変更が必要になる。
5. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSCSD` の1件に保てない。
