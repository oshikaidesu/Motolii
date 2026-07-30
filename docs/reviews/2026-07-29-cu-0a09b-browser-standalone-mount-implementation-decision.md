# CU-0A09B Browser standalone mount 実装決定

- 日付: 2026-07-29
- 状態: **決定 / DONE**
- 親: `CU-0A09`（R6、`SPLIT`）

## 1. 目的

product-owned `DiscoveryBrowserCandidate`を、`html-react-parser`の
`node / options`とlegacy Host scriptなしで同じexportから直接mountできるようにする。
これはAfter Effects CEP型のHost接続へ入る直前のpresentation境界であり、
WebView、offline bundle、wire、Rust Host、D2を本粒へ含めない。

## 2. React直接移管契約

1. `REACT AUTHORITY`:
   対象面はBrowser。正本は
   [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)
   と[UI runtime責任境界](../ui-runtime-architecture.md)。対応specはM3 R6 / `CU-0A09B`。
2. `SOURCE ASSET`:
   固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`から移管済みの
   `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`、既存export、
   CSS/pattern/test closureを用いる。
3. `PRESERVE`:
   既存parser consumer側のDOM、class、stable ID、ARIA、初期Effects状態、
   Browser card、read-only Rectangle identity seamを維持する。
4. `REPLACE`:
   `node`が無い時だけproduct component自身がBrowser rootを組み立て、
   Media / Effects / Create tabのlocal presentation stateをReact内で所有する。
   parser consumerは従来経路を維持する。
5. `STATE OWNER`:
   tabは再mountで失われても作品意味を失わないlocal presentation。
   Rectangle identityはHost read-only projection。Document、selection、Undo、
   Workspace、Project sessionの正本をReactへ追加しない。
6. `DIAGNOSTIC ROUTE`:
   `/browser-standalone.html`は同じproduct exportを観測するdevelopment専用entry。
   成果はproduct componentのlegacy-free mount能力であり、route自体を製品画面、
   H1b、Motolii Studio Previewとして扱わない。既存hash registryと
   current-route source closureへ追加しない。
7. `NEGATIVE ORACLE`:
   legacy Host `.app`、Inspector、Timeline、旧source import、別Browser copy、
   localhost/runtime bundle、Rust wireを含めず、既存ownership guardと専用Playwrightで拒否する。
8. `STOP`:
   WebView、offline manifest、origin/navigation、instance/layout epoch、typed intent、
   Inspector mock state、D2、公開plugin API、永続形式が必要なら停止し、H1bまたは各R5子粒へ戻す。

## 3. 実装

- `DiscoveryBrowserCandidate`は`node === undefined`時だけstandalone rootを描画する。
- `BrowserStandaloneContext`は三surfaceのhidden状態だけを配り、
  `CandidateCreateBrowser`の既存2-field private identity seamを変更しない。
- 既存parser consumerではthumbnail setting bridgeとlegacy interactionを維持する。
- development routeのtoken alias / frame CSSは`docs/mocks-ui`だけに置き、
  product CSS bytesと具体token候補を変更しない。
- provenanceへ`CU-0A09B`をappendし、過去entryを変更しない。

## 4. 非目標とhandoff

`CU-0A09B`だけを`DONE`とする。親`CU-0A09`は残るR6 surfaceのため`SPLIT`。
`CU-0B03`以降、H1b、製品window、offline bundle、Host codec、Inspector接続、
typed intentは`WAIT`を維持する。

## 5. 検証

```text
node --test ui/motolii-web/guard-tests/browser-ownership.test.mjs
7 passed

cd docs/mocks-ui
npx playwright test tests/browser-candidate.spec.js tests/browser-standalone.spec.js
16 passed

npm run build
passed

cd ../..
cargo test --workspace
passed

./scripts/check-docs.sh
passed
```

`current-route-provenance.json`のsource closure pinはproduct sourceの新hashへ
再締結した。`npm run check-current-route`は次段の既知
`CR2-SCHEMA: sourceManifestSha256 does not match current provenance manifest`
で停止する。公開済みimmutable generationは本粒より前から現行provenance manifestと
不一致であり、本粒ではgolden／threshold／immutable generationを更新して緑化せず、
独立gateとして残す。
