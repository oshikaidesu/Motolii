# CU-0A08SSC Browser Place source seam 契約具体化 選定範囲

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08SSC: **DONE**

## 1. 目的

`CU-0A08SSD`が採択したVS-1 Rectangle限定のproduct-owned React Browser source seamについて、次のdocs-only `CU-0A08SSCD`が閉じる契約具体化の唯一の問いを選定する。本粒はその問いに**答えない**。

## 2. 事実

- 最初の実装境界は既存Rectangle `ElementCard` / Browser component境界であり、Host transportとD2の手前で止まる。
- Rectangle Placeのdrop終端はReact Browser component内に存在せず、既存`onDrop`はtag割当だけである。
- [CU-G09決定 §5](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md#5-catalog-item-identity)はcatalog item identityを `(scope_ref, item_id)` とした。
- 現行Rectangle cardは`identity`を受け渡しておらず、`elementProps`が各`ElementCard`へpropsを配る既存所有点である。
- bare `itemId` drag payloadは `S`（open defect）のままである。

## 3. CU-0A08SSCD が閉じる唯一の問い

既存Browser catalog projectionのscoped identityをBrowser component境界で受け取り、RectangleのPlace source要求へ渡す責任を、既存のどのcomponent境界へ置くか。

## 4. 可能な候補（優劣を付けない）

**(A) `ElementCard` leaf境界**

card自身がscoped identityを受けて保持する。

**(B) `CandidateCreateBrowser` component境界**

Browser側で受けて既存`elementProps`から各cardへ配る。

## 5. 非目標

- §3へ答える、候補を重み付けする、推奨・採択を示すこと。
- callback、event、payload、型、API、module、export、crate、wire、transport、props名を命名または新設すること。
- bare `itemId`を有効な契約として肯定すること。
- Placeの意味、lifecycle、commit ownerを変更すること。
- BT/IT/RM行、W0/W1表、M3仕様、`U4a-2`、source asset、公開契約、guard期待値を変更すること。
- VS-1 Rectangle以外へ一般化すること。
- 実装粒を起動すること。

## 6. 必須負例

- 候補(A)/(B)に選好語を付ける。
- §3を二問以上へ増やす。
- 新しい識別子、component、props、event、payload、型、APIを具体化する。
- bare `itemId`の `S` を解消済みにする。
- Place lifecycleまたはdrop終端をBrowser側の責任へ移す。
- PRODUCT-ASSETの完全一致 `DO` を0件または2件以上にする。
- `CU-0A08SSCD`を発注依存証跡へ追加する。

## 7. 同期した current mirror

同じ7箇所を、`CU-0A08SSC` `DONE`、`CU-0A08RM` `WAIT`、次 PRODUCT-ASSET `DO` はdocs-only `CU-0A08SSCD`（1件）へ同期した。

## 8. allowlist外に残る stale mirror

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`

## 9. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08SSC` | **DONE** | scoped identityの受け渡し責任を置く既存component境界の一問を選定 |
| `CU-0A08SSCD` | **DO** | §3の一問だけをdocs-onlyで閉じる |
| `CU-0A08RM` | **WAIT** | source seam契約裁定後にmirror修復を再開 |

## 10. STOP条件

1. §3を記録するために回答、候補の優劣、第二の問いが必要になる。
2. 型、callback、event、payload、API、module、export、props名を先に決める必要がある。
3. bare `itemId`を契約化しないと文が閉じない。
4. Place owner、BT/IT/RM行、M3仕様、公開契約、source assetの変更が必要になる。
5. PRODUCT-ASSETの完全一致 `DO` を `CU-0A08SSCD` の1件に保てない。
