# CU-0A08BTP Browser read projection / JSX connection 実装決定

- 日付: 2026-07-29
- 状態: **決定**
- 対象grain: **CU-0A08BTP**

## 1. 目的

VS-1 Rectangle 1件のdecode済みBrowser catalog identity
`(scope_ref, item_id)`を、product-owned `DiscoveryBrowserCandidate`の
component入力からprivate `CandidateCreateBrowser`と既存`elementProps`を通して、
同じRectangle cardへ非推測透過する。

## 2. 実装

- product rootへprivate入力`rectangleIdentity`を追加した。
- `CandidateBrowserTabs`は同じobjectを加工せず`CandidateCreateBrowser`へ渡す。
- `CandidateCreateBrowser({ scope_ref, item_id })`は2 fieldを
  `elementProps("rectangle", scope_ref, item_id)`へ一度だけ渡す。
- 他card、DOM属性、drag payload、click、typed intentは変更しない。
- root→tabs→private seamとprivate seam→Rectangleの両方を実product sourceへの
  AST検査で固定した。既存synthetic正負18分類も維持する。
- source provenanceへ`CU-0A08BTP` entryをappendし、過去entryは変更していない。
- Browser/Inspector decoder guardは共有provenance fileのhashだけを再締結し、
  decoder拒否oracleと期待値は変更していない。
- `docs/mocks-ui/current-route-provenance.json`はproduct Browser sourceの
  現行hashだけを再締結した。capture、manifest、CURRENT pointerは変更していない。

## 3. 変わらないもの

既存export、DOM/CSS/ARIA/interaction、`developmentProjection`、decoder、
fixture、Document、selection、Undo、D2、Host transport、wire、plugin契約、
永続形式は変更しない。BTP単独でruntime producerや通常製品route到達を名乗らない。

Rectangleの`ElementCard` JSX要素は2 fieldをpropsとして受けるが、本粒では
DOM、drag、intentへ消費しない。card内部での利用やruntime producer成立を意味しない。

## 4. 負例

- literal、bare `itemId`、label、thumbnail tokenからidentityを導出しない。
- raw catalog検索、fallback、default、条件分岐をprivate seamへ入れない。
- Rectangle以外へ2 fieldを一般化しない。
- mock/dev route、diagnostic UI、Host wire、intent、drag payloadへ迂回しない。
- provenanceの過去entry、decoder拒否oracle、visual thresholdを変更しない。

## 5. 検証

```text
node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs
1..5
# tests 5
# pass 5
# fail 0

node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs
1..118
# tests 118
# pass 118
# fail 0

node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs
1..39
# tests 39
# pass 39
# fail 0

node --test docs/mocks-ui/guard-tests/current-route-provenance.test.mjs
1..17
# tests 17
# pass 17
# fail 0

./scripts/check-docs.sh
OK: docs整合チェック全項目通過

npm run test:reference-guard
# pass 516
# fail 2
```

残る2 failureは、immutable generation
`44e538c97807-ead41d4d6562`の`manifest.json`がBTP前の
`sourceManifestSha256`を保持するため、現行provenance manifest hashとの一致検査と
CLI checkが`CR2-SCHEMA`で拒否するもの。`generate-current-route`も既存generationの
置換を設計上拒否する。capture/golden/publicationをBTPへ混ぜず、G0-6H
current-route evidence laneが新しいimmutable generationのpublicationを所有する。

## 6. handoff

`CU-0A08BTP`は**DONE**。`CU-0A08BTI`は既決Place chain待ちの`WAIT`を維持する。
最初の非mock runtime callerと通常製品route到達は、R6後のH1bが所有する。
次のPRODUCT-ASSET `DO`は本粒では選定しない。current-route publicationの再生成を
BTPや背骨の代替成果にせず、後続`G0-6H-V1G-RP`へ移管する。
