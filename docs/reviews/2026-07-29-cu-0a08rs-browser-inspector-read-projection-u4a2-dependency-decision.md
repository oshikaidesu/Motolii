# CU-0A08RS Browser / Inspector read-only projection U4a-2 依存裁定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08RS: **DONE**

## 1. 目的

次の一問だけを docs-only で閉じる。

VS-1 が必要とする Browser / Inspector read-only projection 責任に、`U4a-2` Direct 製品入口依存は load-bearing か。

## 2. 事実

- **F1**: [U枝番分解](2026-07-16-m3-ui-concept-to-tickets.md) の `U4a-2` は Effect Inspector 内の自動生成 panel と nonblocking preview を所有し、全保存 param の編集、100 slider update、最新 preview、1 gesture = 1 Undo を完了条件に持つ。依存は `U4a-1` / `U0e-3` / `U1b-2` / `U2c-5` である。
- **F2**: [implementation ledger](../implementation-ledger.md) の `CU-0A08BT` / `CU-0A08IT` は `WAIT` である。記録済み依存はそれぞれ `CU-0A08BP` + `U4a-2` Direct製品入口、`CU-0A08IP` `DONE` + `U4a-2` Direct製品入口未成立である。
- **F3**: [快適利用粒度化](2026-07-22-m3-comfortable-use-granulation.md) の `CU-0A08BT` / `CU-0A08IT` は typed intent、1 gesture = 1 intent、Cancel / 失敗 = 変更0を完了条件に含む。
- **F4**: [M3仕様](../specs/M3-ui-integration.md) で `CU-0A08IP` と `CU-0A08BP` は `DONE` であり、product-owned・非exportのdecoderとfixture/testだけを閉じている。
- **F5**: `ui/motolii-web/src/read-model/browserCatalogDecoder.js` と `ui/motolii-web/src/read-model/inspectorReadModelDecoder.js` は存在する。`ui/motolii-web/src/index.js` は両read-modelをexportせず、`ui/motolii-web/src` と `docs/mocks-ui/src` にproduction consumerは0件である。
- **F6**: [M3仕様](../specs/M3-ui-integration.md) の `U2b-1` は完了済みで、Apply / Undo / Redo 成功時だけ snapshot を publish する。
- **F7**: [implementation ledger](../implementation-ledger.md#発注依存証跡) で `U2h-1I` / `CU-104` / `CU-104E` は `DONE` であり、Host Transient primary selection、selection publish envelope、projection generation 枯渇境界をそれぞれ閉じている。
- **F8**: [CU-0A08RS0選定](2026-07-29-cu-0a08rs0-browser-inspector-read-projection-dependency-scope-selection.md) が選定した候補は、(A) load-bearing である、(B) load-bearing でない、の2つだけである。

## 3. 裁定

候補 **(B)** を採用する。VS-1 が必要とする Browser / Inspector read-only projection 責任に、`U4a-2` Direct 製品入口依存は **load-bearing ではない**。

- F1 は `U4a-2` の責任を保存 param の編集、preview、Undoへ置く。これはread-only projectionの責任ではない。
- F2 / F3 は現行 `CU-0A08BT` / `CU-0A08IT` がread-only projectionとtyped intentを束ねている事実を示す。本裁定だけで両粒の`WAIT`や依存セルを変更しない。
- F4 / F5 はread-only decoder実装が完了している一方、production consumerがまだ存在しないことを示す。consumer不在を`U4a-2`編集入口で埋めない。
- F6 / F7 はread-only投影が読むsnapshot publish、selection publish envelope、projection generationを`U4a-2`と別の既存責任として示す。

したがってread-only Host projectionに必要な既存依存クラスは、次の閉集合である。

1. `CU-0A08BP`: product-owned Browser catalog decoder。
2. `CU-0A08IP`: product-owned Inspector read-model decoder。
3. `U2b-1`: Apply / Undo / Redo 成功時の snapshot publish。
4. `CU-104` / `U2h-1I` / `CU-104E`: selection publish envelope、Host Transient primary selection、projection generation。

この裁定は `U4a-2` を不要・完了済みとするものではない。`U4a-2` は保存param編集、nonblocking preview、Direct製品入口側の責任として維持する。

## 4. 変わらないもの

- `CU-0A08BT` / `CU-0A08IT` / `U2c-2` は `WAIT` を維持する。
- `CU-0A08BT` / `CU-0A08IT` の記録済み依存セルは本粒で変更しない。
- `U4a-2` / `U4a-1` / `U4c` / `U2b` / `U2h-1` の意味、順序、完了条件を変更しない。
- `U3a-2Q-V` / `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD` / `CU-107` / `CU-110` / `CU-111` / `CU-0B05` の状態を変更しない。

## 5. 非目標

- event shape、WebView wire、Host transport 名、typed intent 型・名前、公開 API、visibility、serde field、default 値、閾値、rejection precedence、drag payload、`S` 行を決めること。
- Rust / JS / JSX / CSS / fixture / guard test / schema / Document / journal / plugin 契約 / 期待値 / golden の変更。
- React sourceの縮約再実装、二重copy、productからの`docs/mocks-ui` runtime import、decoderの公開raw API化。
- 既存decision文書や発注依存証跡の既存行の意味を書き換えること。
- Fable 5 / Composer の助言をauthorityとして引くこと。
- allowlist外fileの変更。

## 6. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08RS` | **DONE** | 候補(B)を採択し、VS-1 read-only projectionの既存依存クラスを限定 |
| `CU-0A08RM` | **DO** | 本裁定の既存依存クラスを`CU-0A08BT` / `CU-0A08IT`の記録済み依存セルへdocs-only反映。両行は`WAIT`維持 |

## 7. STOP 条件

1. 既存IDだけでは依存クラスを記述できず、新しいAPI・event shape・transport・ticket意味を作る必要が生じた。
2. `CU-0A08BT` / `CU-0A08IT` / `U2c-2` の`WAIT`や依存セルを本粒で変更しないと整合しなくなった。
3. `U4a-2`を全体として不要・完了済みにしないと文書が成立しなくなった。
4. 公開API、Document、永続形式、journal、plugin契約、typed intent、guard期待値への波及が必要になった。
5. PRODUCT-ASSET `DO`を0件または2件以上にしないと整合しなくなった。

## 8. allowlist 外に残る stale mirror

本粒のallowlist外であるため、次の4 pathは同期していない。`CU-0A08RM`以降の独立mirror修復粒へ送る。

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md`
