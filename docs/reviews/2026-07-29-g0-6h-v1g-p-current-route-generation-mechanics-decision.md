# G0-6H-V1G-P 現行route generation mechanics決定

- 日付: 2026-07-29
- 状態: **決定**
- G0-6H-V1G-P: **DONE**

## 目的

`G0-6H-V1G`を`G0-6H-V1G-I`、`G0-6H-V1G-C`、`G0-6H-V1G-O`へ厳密直列化し、現行route evidence generationのmechanicsをdocs-onlyで閉じる。実装、画像、script、fixture、command実体は本粒で作らない。

## 現行コード事実

1. 旧`#reference/*` generationのrootは`docs/mocks-ui/reference-output/`で、`CURRENT`と`generations/u0e2-08f96cbd7754-85c0fc529ab1/{manifest.json,captures/}`を持つ。
2. 旧generationのcaptureは5 screen × 6 variantの30 PNGである。
3. 旧manifestは`schemaVersion / generation / browserVersion / sourceManifestSha256 / transformVersion / screens / captures`、captureは`path / screen / variant / sha256`の閉schemaで、`schemaVersion`は1である。
4. 旧commandは`generate-reference`と`check-reference`で、checkは`repositoryFingerprint()`により実行前後のrepository byteとstatusを比較する。
5. 旧source manifestは`docs/mocks-ui/reference-provenance.json`で、transformation source、font、toolchain lock、fixture layer、capture契約を検証する。
6. 旧screen順は`empty-browser / mixed-timeline / parameter-easing / stage-frame-tools / shared-effect-relative`、variant順は`normal / lightness / grayscale / protanopia / deuteranopia / tritanopia`である。
7. 旧captureはfont loadとcomputed familyを検証し、fallback時に停止する。
8. 旧captureは許可local origin以外を遮断し、外部requestが1件でもあれば失敗する。
9. 現行routeのscreen 1 carrierは既存Vite mode`current-route-capture`であり、専用configはport 4174を使う。Vite configにmode分岐はない。
10. `V1ETC`〜`V1ETE`によりscreen 1の空投影と`#root[data-current-route-capture-ready="true"]`が成立し、通常modeの同一routeは`.app[data-parity-ready="true"]`を保つ。
11. 現行route専用のgeneration root、provenance、manifest、generate/check commandはまだ存在しない。

## M-1 旧v1不変

- 裁定: 旧root、`CURRENT`、既存generationのmanifestと30 PNG、`schemaVersion: 1`の閉field、`reference-provenance.json`、`generate-reference`、`check-reference`の意味とbyteを不変にする。V1G-I/C/Oから読み替え、移動、改名、拡張、再生成しない。
- 停止線: 旧側のschema、期待値、commandへ現行route責任を混載する必要が生じたら停止する。
- 根拠authority: V1R R-6、現行コード事実1〜5。

## M-2 現行route rootとcommand

- 裁定: 出力rootを`docs/mocks-ui/current-route-output/`、source provenanceを`docs/mocks-ui/current-route-provenance.json`、commandを`generate-current-route`と`check-current-route`に固定する。旧rootと旧commandは再利用・分岐しない。script file名はV1G-Iで既存責任へ照合して固定する。
- 停止線: 旧root配下への配置、旧commandの多重化、別名の第二ownerが必要なら停止する。
- 根拠authority: V1R R-6、V0 V-7。

## M-3 同一routeと2 Vite mode

- 裁定: 5画面はすべて`#plugin-browser-candidate`から採る。screen 1は`current-route-capture` modeでreadyを待ち、screen 2〜5は通常modeでV1P P-2の既存操作とoracleにより到達する。manifestの`screens`は各要素のexact`screen / mode`で対応を記録する。
- 停止線: 新route、hash、query、追加mode、追加served entry、Vite config分岐、global stateが必要なら停止する。
- 根拠authority: V1S B-1、V1R R-8、V1P P-1/P-2、現行コード事実9〜10。

## M-4 5画面と順序

- 裁定: canonical順を`empty-browser`（empty project + asset browser、Browser検索0件相当）、`mixed-timeline`、`parameter-easing`、`stage-frame-tools`、`shared-effect-relative`の5件に固定する。captureとmanifestはscreen-major × 6 variant順とし、順序不一致を拒否する。
- 停止線: screenの追加、削除、改名、並べ替えを必要としたら停止する。
- 根拠authority: V0 V-2/V-6、現行コード事実6。

## M-5 manifest v2と環境9軸

- 裁定: 現行route manifestは`schemaVersion: 2`とする。top-level exact fieldは`schemaVersion / generation / sourceManifestSha256 / transformVersion / environment / screens / captures`。`environment`は`viewport / scale / locale / timezone / theme / reducedMotion / browserVersion / browserRevision / fontFixture`の9軸だけ、`viewport`は`width / height`、`fontFixture`は`files / computedFamily`、各fileは`path / sha256 / weight`、各screenは`screen / mode`、各captureは`path / screen / variant / sha256`の閉集合とする。9軸のliteral値の記録ownerはmanifestだけである。
- 停止線: fieldの追加・欠落・改名、またはREADME、handoff、provenance、test、commit messageへの第二ownerが必要なら停止する。
- 根拠authority: V1S B-4、V0 V-5/V-7、V1P P-3、V1R R-6。

## M-6 transitive source closure

- 裁定: `current-route-provenance.json`のtop-levelは`schemaVersion / assets`、各assetは`path / sha256`とする。product component、CSS、mock consumer、envelope、fixture、token、font、transform、toolchain lockの推移閉包をcanonical path昇順で列挙する。manifestの`sourceManifestSha256`はprovenance file bytesのSHA-256とする。
- 停止線: 欠落、余分、重複、非canonical path、repo escape、未解決またはdynamic local import、宣言外source、driftを検出したら停止する。第二closure定義を作らない。
- 根拠authority: V0 V-7、V1R R-6、旧provenance検証段。

## M-7 font fallback STOP

- 裁定: capture前にfont fixture loadとcomputed family一致を検証する。不一致時はpublication前に停止し、画像を受理せず、generationをpublishせず、部分成果物を残さない。product CSS、旧reference font、candidate font stackは変更しない。
- 停止線: fallbackを許容する、またはCSS変更で合わせる必要が生じたら停止する。
- 根拠authority: V1P P-3、現行コード事実7。

## M-8 offline-only

- 裁定: generate、capture、checkはnetworkを要求しない。実行時に明示した許可local originと`data:`/`blob:`以外を遮断し、外部requestが1件でもあれば失敗する。許可originの個数と値はV1G-Cの実測で閉じる。cross-platform、OS横断、toolchain横断のbyte決定性は主張しない。
- 停止線: 外部取得やbackground networkが必要なら停止する。
- 根拠authority: V0 V-7、G0-6H-A A-5、G0-6H-AF AF-3、現行コード事実8。

## M-9 共有fingerprint

- 裁定: V1G-CのcaptureとV1G-Oのpublicationは同じhelperが導出するsource fingerprintで束ねる。既存`reference-cli.mjs`のrepository fingerprint責任をcopyせず共有private moduleへ抽出し、旧CLIと新CLIが同じ実装をimportする。capture時とpublish時のfingerprint不一致を拒否する。checkはGit可視file名・byte・statusを実行前後で変えないread-only再導出とする。publish済みgenerationはimmutableで、上書き、部分更新、再publishを拒否する。
- 停止線: fingerprint実装の複製、失敗時mutation、既存generation上書きが必要なら停止する。
- 根拠authority: V0 V-7、V1R R-6、現行コード事実4。

## M-10 厳密直列

- 裁定: V1G-Iはmanifest v2、provenance closure、共有fingerprint、分離root/command基盤だけを閉じる。V1G-Cは2 mode、5 screen、6 variant、font、offline captureだけを閉じる。V1G-Oはimmutable publicationとread-only照合だけを閉じる。各粒を独立closed orderとし、先行粒がmainへ到達するまで後続closed orderを作らない。
- 停止線: 責任跨ぎ、並行実装、先行粒の未採用差分を後続へ継承する必要が生じたら停止する。
- 根拠authority: implementation ledger、AGENTS.mdの一契約境界規律。

## 確定しないこと

環境9軸のliteral値、generation ID、生成時hash、script file名、variant algorithm、threshold、tolerance、token値は本粒で確定しない。

## 非目標

- code、package、lockfile、JSON、PNG、CSS、React、route、script、fixture、guard、npm command実体、CI、依存の変更。
- 旧generation、`CURRENT`、旧manifest、旧provenance、Starter Mediaの生成・変更。
- public API、Document、plugin契約、永続形式、Undo、selectionの変更。
- 人間session、token採択、`G0-6H`完了、`U0e-3`解禁、隣接ticketの状態変更。

## handoff

| ID | 状態 | 根拠 |
| --- | --- | --- |
| G0-6H-V1ETE | DONE | integrated ready oracle |
| G0-6H-V1G-P | DONE | 本決定 |
| G0-6H-V1G-I | DO | 次の唯一の粒 |
| G0-6H-V1G-C | WAIT | V1G-Iのmain到達待ち |
| G0-6H-V1G-O | WAIT | V1G-Cのmain到達待ち |
| G0-6H-V1G | SPLIT | I/C/Oを管理 |
| G0-6H | DO / HUMAN | 人間判断は未完了 |

## Reactラベル

### REACT AUTHORITY

対象面は`#plugin-browser-candidate`上のproduct-owned`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`とmock consumer`docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx`、route registry`docs/mocks-ui/src/main.jsx`。移管契約は[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime境界は[UI runtime architecture](../ui-runtime-architecture.md)のbundled first-party Host module。対応spec IDはG0-6H-V0/V1S/V1P/V1R/V1ETC/V1ETB/V1ETT/V1ETE/V1G-P。

### SOURCE ASSET

固定source commitは`ui/motolii-web/source-provenance.json#fixedSourceCommit`の`56c318edcddab7cf95d263cc2f7dd2b4e6791134`、対象exportは`ui/motolii-web/src/index.js`の`DiscoveryBrowserCandidate`。`ui/motolii-web/src/candidates/*.css`、`docs/mocks-ui/tests/browser-candidate.spec.js`、`docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`、`ui/motolii-web/guard-tests/browser-ownership.test.mjs`を含むCSS/model/test closureを本粒では1 byteも変更しない。

### PRESERVE

`#plugin-browser-candidate`の既存DOM、stable ID、class、ARIA、interaction、visual state、`#stage`、`#inspector`、`#timeline`、`#project-browser`、`#interval-easing`、`#easing-panel`、`.app[data-parity-ready]`、`#root[data-current-route-capture-ready]`、post-promotion provenance、ownership guardを保持する。

### REPLACE

なし。mock/legacy stateからprojection/intentへの交換を行わない。

### STATE OWNER

capture envelopeとmode carrierはTransient/local presentation/development-onlyである。generation manifestはfixture-only evidence面が所有し、Document、User settings、Workspace、Project session、恒久Host契約へ保存しない。

### DIAGNOSTIC ROUTE

製品画面は不変の`#plugin-browser-candidate`。development確認は既存`current-route-capture` modeと旧`#reference/*` generationに限り、新route、hash、query、mode、entryを追加しない。

### NEGATIVE ORACLE

二重copy、legacy runtime import、opaque ID/label/thumbnailからの意味推測、二重state、二重9軸owner、visual threshold/golden変更、新しい公開export、描画後DOM mutation、global変数を拒否する。

### STOP

未決product意味、公開契約、source asset不在、state owner違反、allowlist外変更、環境literalまたはgeneration hashの発明が必要なら停止する。

## 関連

- [AGENTS.md](../../AGENTS.md)
- [G0-6H-V0契約](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [G0-6H-V1S裁定](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)
- [G0-6H-V1P裁定](2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md)
- [G0-6H-V1R裁定](2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md)
- [implementation ledger](../implementation-ledger.md)
