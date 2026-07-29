# CU-0A08BTR Browser read-projection 依存再締結

- 日付: 2026-07-29
- 状態: **決定**
- CU-0A08BTR: **DONE**

## 1. 目的

VS-1 の通常製品 route を迂回せず、`CU-0A08BT` に束ねられた
read-only projection と typed intent を分ける。既決の
[CU-0A08RS](2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md)
を依存セルへ反映し、Browser catalog projection の製品component入力から
`CandidateCreateBrowser` へ scoped identity を透過する R5 実装を開始可能にする。

## 2. code fact と authority

1. `browserCatalogDecoder.js` は product-owned pure decoder として `DONE` だが、
   production importer は0件である。
2. `CandidateCreateBrowser` は module-private、引数0、呼出元は
   `CandidateBrowserTabs` の1件だけである。
3. `developmentProjection` の decode 結果は `CandidateProjectBrowser` へだけ届き、
   projection 定義時は `CandidateCreateBrowser` を含む通常 tabs を描画しない。
4. したがって既存 product source 内に scoped identity の到達可能な runtime producer は無い。
   development fixture、literal、branch反転では通常製品 route を成立させられない。
5. `CU-0A08RS` は VS-1 read-only projection に `U4a-2` は load-bearing でないと
   裁定済みだが、その粒の非目標により `CU-0A08BT` の依存セルは変更されなかった。
6. `CU-0A08RMD` は typed-intent 半分を Rectangle Place chain に分類し、
   `CU-0A08BDD` は Browser source-seam first を採択した。
7. `CU-0A08SSCI-P1` により provenance chain、`CU-0A08SSCI-T1` により
   private component の正負 AST guard は閉じた。残る阻害要因は producer route だけである。
8. React直接移管契約の順序は R5 projection/intent → R6 → H1b → 製品縦切りである。
   H1b exact contract を先に作る順序ではない。
9. この順序では R5 が product component の入力契約を先に閉じ、最初の非mock runtime caller は
   R6後の H1b が同じ契約を使って成立させる。BTP単独で通常製品route到達を名乗らない。

## 3. 裁定

`CU-0A08BT` を次の2責任へ分ける。

| ID | 責任 | 依存 | 現在状態 |
|---|---|---|---|
| `CU-0A08BTP` | VS-1 Rectangle 1件の decoded Browser catalog identity を product-owned Browser root のcomponent入力として受け、private `CandidateCreateBrowser` と既存 `elementProps` から同じ Rectangle card へ非推測透過する read-only projection / JSX connection | `CU-0A08BP`、`CU-0A08RS`、`CU-0A08SSCI-P1`、`CU-0A08SSCI-I`、`CU-0A08SSCI-T1` | **DO** |
| `CU-0A08BTI` | Browser Place source から既決 Place chain へ渡す typed intent | `CU-0A08BTP`、`CU-0A08RMD`、`CU-0A08BDD`、`CU-107PV`→`CU-107TC`→`CU-107AD`→`CU-107TD` | **WAIT** |

親 `CU-0A08BT` は **SPLIT** とする。

`CU-0A08BTP` は R5 の Browser projection half であり、H1b transportでも
runtime producerでもない。product-owned Browser root のcomponent入力は
decode 済み catalog projection に限定し、
decoder output の `(scope_ref, item_id)` を VS-1 Rectangle 1件についてそのまま使う。
catalog ID、label、thumbnail token、bare `itemId` から identity を導出しない。

`CU-0A08SSCI` の実装責任は `CU-0A08BTP` へ吸収し、親を **SPLIT** とする。
`CU-0A08BTP` は component入力と private seam を同じ closed diff で実装する。
最初の非mock runtime callerと通常製品route到達は H1b が所有し、BTPの完了条件へ偽装しない。
typed intent、drag payload、Host wire、D2、drop終端は含めない。

## 4. `CU-0A08BTP` の実装許可範囲

- `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
- `ui/motolii-web/guard-tests/browser-ownership.test.mjs`
- `ui/motolii-web/source-provenance.json` の append-only post-promotion entry
- `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` のauthority hash reclosure。
  decoderの拒否oracleや期待値を弱めない
- `docs/mocks-ui/current-route-provenance.json` の当該product source hash reclosure
- product component contract と同じ入力を使う product-owned test
- 完了記録に必要な本決定の current mirror

exact props名、test file、post-promotion task entry は実装粒で固定できる。
package exportを増やさず、既存 `DiscoveryBrowserCandidate` exportの製品内部入力境界に閉じる。

## 5. 非目標

- H1b WebView Host、offline bundle、wire schema、origin、navigation、lifecycle。
- BTP単独で非mock runtime callerまたは通常製品route到達を成立済みとすること。
- typed intent、drag payload、D2、drop終端、Place commit、Undo/Redo。
- `developmentProjection`、docs/mock route、fixture scriptを通常 producer にすること。
- Rectangle以外への一般化、catalog検索、fallback/default、identity推測。
- Document、selection、Undo、semantic stateをReactへ所有させること。
- `U4a-2`、`U0e-3`、`G0-6H`の意味または状態を変えること。
- plugin UI公開API、community runtime、永続形式、serde、journalの変更。

## 6. 必須負例

1. `CU-0A08BTP` が `U4a-2` または `G0-6H` 待ちになる。
2. `developmentProjection`、literal、bare `itemId`、label、thumbnail tokenから
   Rectangle identityを作る。
3. raw catalogを `CandidateCreateBrowser` 内で検索する。
4. non-Rectangle cardへ2-field identityを渡す。
5. typed intent、drag payload、Host transport、D2、drop終端を同じ差分へ入れる。
6. package export、新module、新依存、公開plugin契約を増やす。
7. provenanceの過去entryを書き換える、guard期待値を弱める、testをskipする。

## 7. handoff

| ID | 状態 | 内容 |
|---|---|---|
| `CU-0A08BTR` | **DONE** | read-only projectionへ`CU-0A08RS`の非`U4a-2`依存を反映し、親BTを分割 |
| `CU-0A08BTP` | **DO** | product Browser rootのcomponent入力からRectangle private seamへdecoded identityを透過 |
| `CU-0A08BTI` | **WAIT** | BTP後、既決Place chainへtyped intentを接続 |
| `CU-0A08SSCI` | **SPLIT** | private seam実装責任をBTPのclosed diffへ吸収 |

PRODUCT-ASSET lane の完全一致 `` `DO` `` は `CU-0A08BTP` の1件だけとする。

## 8. 同期した current mirror

- `docs/README.md`
- `docs/specs/M3-ui-integration.md`
- `docs/implementation-ledger.md` のM3行、lane行、短い運用判断
- `docs/decision-index.md` のVS-1行、Browser split行、SSCI系列行
- `docs/reviews/2026-07-22-m3-comfortable-use-granulation.md` のW0行
- `docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md` の
  Browser catalog / journal / selection行
- `docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` のparent BT state oracle

allowlist外に矛盾する current mirror は残さない。過去粒が「当時は未選定」と記録した
履歴文は書き換えない。

## 9. STOP条件

1. product Browser rootへdecode済みprojectionを渡すためにpackage export、新module、
   Host wire、公開plugin APIが必要になる。
2. decoder output以外からRectangle identityを推測する必要がある。
3. component入力から`CandidateCreateBrowser`へ同じ2-field identityを透過できない。
4. `CU-0A08SSCI-T1`の正負条件またはappend-only provenance authorityを弱める必要がある。
5. typed intent、drag payload、D2、drop終端を含めないとread-only projectionが成立しない。
6. PRODUCT-ASSETの完全一致 `` `DO` `` を1件に保てない。
