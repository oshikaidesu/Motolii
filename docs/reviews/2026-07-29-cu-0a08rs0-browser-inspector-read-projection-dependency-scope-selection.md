# CU-0A08RS0 Browser / Inspector read-projection 依存範囲の選定

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08RS0: **DONE**

## 1. 目的

VS-1 が必要とする Browser / Inspector read-only projection について、`U4a-2` Direct 製品入口依存が load-bearing かどうかを閉じる docs-only 裁定粒 `CU-0A08RS` へ送る未決範囲を、次の一問だけに限定して選定する。本粒はその問いに**答えない**。

## 2. 事実

- `ui/motolii-web/src/read-model/browserCatalogDecoder.js` と `ui/motolii-web/src/read-model/inspectorReadModelDecoder.js` が存在する。
- `ui/motolii-web/package.json` の `exports` は `{".": "./src/index.js"}` のみ。`ui/motolii-web/src/index.js` は read-model を export しない。
- `ui/motolii-web/src` と `docs/mocks-ui/src` の全体で、上記2 decoderを import する行は0件。両 decoder は product-owned・非 export・fixture/test 専用で、production consumer は存在しない。
- [implementation ledger](../implementation-ledger.md) の PRODUCT-ASSET lane では、着手時点で状態が完全一致 `DO` の行は `CU-0A08RS0` の1件のみであった。
- [発注依存証跡](../implementation-ledger.md#発注依存証跡) に `CU-107W` / `CU-107W-R1` / `CU-0A08BP` / `CU-0A08IP` / `CU-0A08IS` / `CU-G09` / `CU-G09O` / `CU-G09R` / `CU-101` / `CU-102` の一意な `DONE` 行がある。`CU-0A08RS0` 行は着手時点では未登録であった。
- `CU-0A08BT`（Direct Browser connection）と `CU-0A08IT`（Direct Inspector connection）は lane で `WAIT` である。依存はそれぞれ `CU-0A08BP` + `U4a-2` Direct 製品入口、`CU-0A08IP` + `U4a-2` Direct 製品入口である。
- `docs/mocks-ui/node_modules` と `ui/motolii-web/node_modules` は着手時点で存在せず、`docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs` と `npm run test:reference-guard` は依存未導入の状態では実走できない。

## 3. CU-0A08RS が閉じる唯一の問い

以下は次粒の問いであり、本粒の回答ではない。

VS-1 が必要とする Browser / Inspector read-only projection 責任に、`U4a-2` Direct 製品入口依存は load-bearing か。

## 4. 可能な候補（優劣を付けない）

**(A) load-bearing である。**

次の docs-only 粒で、VS-1 の Browser / Inspector read-only projection 責任に対し、`U4a-2` Direct 製品入口依存が load-bearing であると裁定する案。**本粒ではその根拠・順序・具体的依存の書き方は決めない。**

**(B) load-bearing でない。**

次の docs-only 粒で、上記 projection 責任に対し、`U4a-2` Direct 製品入口依存は load-bearing ではないと裁定する案。**本粒では代替依存・順序・具体的な切り分けは決めない。**

## 5. 未検証参考の扱い

本ループ外の Fable 5 read-only 助言（allowlist 候補は本5 docs）は**未検証の助言**であり authority ではない。本粒の全結論は §2 の事実と、発注書が名指しした authority 文書だけから導く。Fable 助言の逐語再掲・引用による正当化は行わない。

## 6. 非目標

- §3 の一問に答えること、または答えを示唆する評価語・重み付けを書くこと。
- `CU-107PV` / `CU-107TC` / `CU-107AD` / `CU-107TD`、`CU-107`、`CU-110`、`CU-111`、`CU-0B05`、`CU-0A08BT` / `CU-0A08IT`、`U2c-2`、`U3a-2Q-V` の `WAIT` を解く、`DO` へ上げる、依存リストを書き換えること。
- `U4a-2` / `U4a-1` / `U4c` の意味、順序、完了条件を書き換える、または `U4a-2` を「不要」「完了済み」と扱うこと。
- event shape、WebView wire、typed intent 型・名前、公開 API、visibility、serde field、default 値、閾値、rejection precedence、drag payload、`S` 行を決めること。
- Rust / JS / JSX / CSS / fixture / guard test / schema / Document / journal / plugin 契約 / 期待値 / golden の変更。
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md` の W0 表、`docs/specs/M3-ui-integration.md`、`docs/README.md`、`docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`、`docs/reviews/2026-07-28-cu-107w-w0-mirror-rewrite-decision.md` の編集。
- 既存 decision 文書・発注依存証跡の既存行の意味の書き換え。
- 隣接チケット（`CU-109` / `CU-110` / `CU-111` / `CU-106P` / `CU-106F` / `U2h-1P`）への拡張。
- allowlist 外 file への一切の変更。

## 7. 必須負例

- 候補 (A)/(B) のどちらかを推奨・採用・「有力」「望ましい」等と書く、または片方だけ厚く書いて実質的に選好を出す。
- §3 の一問を2問以上に増やす、または `CU-0A08RS` に追加の裁定対象を足す。
- `U4a-2` の意味変更、`CU-0A08BT` / `CU-0A08IT` の `WAIT` 解除、event shape / 公開 API / Document / 永続形式の先取り。
- PRODUCT-ASSET `DO` を0件または2件以上にする。
- `CU-0A08RS` を発注依存証跡へ `DONE` として追加する。
- 既存 decision 文書または発注依存証跡の既存行の意味を書き換える。
- [reviews 索引](README.md) または [decision-index](../decision-index.md) への登録を省く、重複主題行を新設する、固定語彙外の状態語を使う。
- lint 抑制、テスト期待値・golden の書換え、fixture 特例、guard の個別 ID 除外・skip。
- React component の二重 copy、product からの `docs/mocks-ui` runtime import、decoder 公開 raw API、未実装 stub。
- allowlist 外の stale mirror を黙って無かったことにする。

## 8. allowlist 外に残る stale mirror

本粒の allowlist 外のため、次の3 pathは同期していない。後続の独立 mirror 修復粒へ送る。

- `docs/specs/M3-ui-integration.md`
- `docs/README.md`
- `docs/reviews/2026-07-16-m3-ui-concept-to-tickets.md`

## 9. STOP 条件

1. 選定文書を書くために §3 の一問へ答えざるを得なくなった。
2. `U4a-2` の意味、`CU-0A08BT` / `CU-0A08IT` / `U2c-2` の依存、W0 表、`docs/specs/M3-ui-integration.md` のいずれかを変えないと整合しない。
3. allowlist 5 file だけでは要求を満たせない。
4. PRODUCT-ASSET `DO` を0件または2件以上にしないと整合しない。
5. 着手時に `CU-0A08RS0` lane 行が `DO` でない、または §2 の10依存のいずれかが発注依存証跡で `DONE` でない。
6. 公開 API、Document、永続形式、journal、plugin 契約、event shape、typed intent への波及が必要に見えた。
7. Fable 5 の未検証助言を authority として引かないと文書が成立しない。
8. 既存 guard の期待値・pattern・除外リストを変えないと緑にならない。

## 10. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08RS0` | **DONE** | Browser / Inspector read-projection の `U4a-2` 依存裁定を docs-only `CU-0A08RS` へ選定 |
| `CU-0A08RS` | **DO** | §3 の一問だけを docs-only で閉じる |
