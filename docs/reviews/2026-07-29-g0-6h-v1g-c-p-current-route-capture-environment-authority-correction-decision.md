# G0-6H-V1G-C-P 現行route capture環境 authority 補正決定

- 日付: 2026-07-29
- 状態: **決定**
- G0-6H-V1G-C-P: **DONE**

## 目的
現行route-captureの生成環境レコード3点を、既存の決定を尊重して再束縛する。

- locale: `ja-JP`
- timezone: `UTC`
- browser識別子はPlaywright-bundled `chromium-headless-shell`
- font fixtureは`Inter`系統既存familyを検査し、`.app`の`computedFamily`先頭一致を再現証拠へ記録

これらを `docs/mocks-ui/scripts/current-route-generation.mjs` の既存schemaに合わせてmanifest化し、停止条件を明示する。

## 現行コード事実
1. `docs/mocks-ui/scripts/reference-capture.mjs:95-96` と `:209-210` は旧`#reference/*` capture contextを`locale: "en-US"`、`timezoneId: "UTC"`で記録する。
2. `docs/mocks-ui/reference-provenance.json`の`capture`記録は`locale: "en-US"`、`timezoneId: "UTC"`、`name: "Chromium Headless Shell"`、`version: "149.0.7827.55"`、`revision: "1228"`、`viewport: 1440×900`、`deviceScaleFactor: 1`、`colorScheme: "dark"`、`reducedMotion: "reduce"`。
3. `docs/mocks-ui/playwright.config.js:17` と `docs/mocks-ui/playwright.current-route-capture.config.js:21` は`locale: "ja-JP"`と`channel: "chrome"`を指定し、viewport `1440×900`、`deviceScaleFactor: 1`、`colorScheme: "dark"`、`reducedMotion: "reduce"`を設定する。timezoneは未設定。
4. `docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md:241` はproduct-promotion comparison contextを`viewport 1440×900`、`DPR 1`、`dark`、`ja-JP`、`reducedMotion`、font readyへ固定。
5. `docs/ui-visual-language.md:197` は`en-US`を現行UI source locale / initial display / fallback / reference fixtureの一般規定として保持し、first product judgmentは英語。これをV1G-C-Pは再解釈しない。
6. `docs/mocks-ui/scripts/reference-capture.mjs:26-27` は`EXPECTED_BROWSER_VERSION="149.0.7827.55"`、`EXPECTED_BROWSER_REVISION="1228"`を固定し、`readBrowserDescriptor()`は`playwright-core/package.json`経由で`browsers.json`を解決して`chromium-headless-shell`の`browserVersion`と`revision`一致検証、`chromium.launch({ headless: true })`後は`browser.version()`を突き合わせる。
7. `docs/mocks-ui/package.json` は`@playwright/test`を`1.61.1`、`@fontsource/inter`を`5.3.0`へ固定し、lock integrityは`docs/mocks-ui/reference-provenance.json`の`toolchain`に記録されている。
8. `docs/mocks-ui/reference-provenance.json`の`fontFiles`は、`public/reference-fonts/LICENSE`（SHA-256 `3b0a5fca3d17942cde889069889dedbbbd075e9b599968c82a95f4d944e9b345`）、`public/reference-fonts/inter-latin-400-normal.woff2`（`8909904ab6c872eb994093482a88a28eca2cd95912d7b6fecd72103b0dc07edc`）、`public/reference-fonts/inter-latin-600-normal.woff2`（`f9a06e79cd3a2a20951c0f0e28f66dd0e6d3fda73911d640a2125c8fcb78f21a`）の3件を列挙する。
9. `docs/mocks-ui/src/reference/reference-font.css`は旧reference route専用family`"MotoliiReferenceInter"`を宣言し、`.motolii-mock-app[data-reference-screen]`配下だけへ適用する。
10. `docs/mocks-ui/scripts/reference-capture.mjs:121-145`は`400 12px "MotoliiReferenceInter"`と`600 12px "MotoliiReferenceInter"`を`document.fonts.load`し、両weight、`getComputedStyle(referenceRoot).fontFamily`、`--mock-role-font-technical`を検証して、不一致を`reference font fallback detected`で拒否する。
11. `docs/mocks-ui/src/tokens/mock-candidates.css:34`は`--mock-candidate-font-sans: Inter, ui-sans-serif, system-ui, -apple-system, sans-serif`、`:73-74`はinterface roleをその候補へ、technical roleをmono stackへ対応させるが、`docs/mocks-ui/src`と`ui/motolii-web`にはinterface roleを`font-family`へ適用するruleがない。
12. `docs/mocks/m3-vism-host-boundary.html:20`は`body{...font:12px/1.35 Inter,ui-sans-serif,system-ui,-apple-system,sans-serif}`を持ち、`docs/mocks-ui/src/legacy/legacySource.js`が同HTMLを`?raw`で読み取った`legacyStyle`をverbatimに注入する。現行routeのcapture root`.app`はこのfamilyを継承し、先頭familyは`Inter`になる。
13. `docs/mocks-ui/src/main.jsx:16`は`import.meta.env.MODE === "current-route-capture"`を導出し、`:33`は`#root[data-current-route-capture-ready="true"]`を付与し、`:123-138`は`plugin-browser-candidate`を`LegacyHostBoundaryScreen`とproduct `DiscoveryBrowserCandidate`で登録する。capture-visible rootは`.app`で、通常modeのready oracleは`.app[data-parity-ready="true"]`（`docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js:35,62,288,390`）。
14. `docs/mocks-ui/scripts/current-route-generation.mjs:152-180` はschemaを9軸固定（`viewport / scale / locale / timezone / theme / reducedMotion / browserVersion / browserRevision / fontFixture`）し、`fontFixture`を`{ files, computedFamily }`、`files`を`{ path, sha256, weight }`昇順で固定。
15. `docs/mocks-ui`配下の`current-route`前提となるcapture生成・check・output rootは`BASE_SHA`時点で未存在。`scripts`は`generate-reference`/`check-reference`のみ。

## 衝突の整理
1. **C-1 locale衝突**: `ja-JP`(既存current-route)と`en-US`(旧`#reference/*` + `ui-visual-language.md`)。
2. **C-1 timezone衝突**: current-routeは未設定(Playwright config)と旧routeは`UTC`明示。
3. **C-2 browser衝突**: current-route captureでは`channel: "chrome"`設定と、`reference`固定sourceでは`chromium-headless-shell` `browserVersion/browserRevision`固定。
4. **C-3 font衝突**: reference scriptは`MotoliiReferenceInter`検証、current-route capture rootで参照は`Inter`候補+inherit、`reference-font.css`のみ、現行capture rootは`.app`。

## CP-1

### 裁定
capture環境`locale/timezone`は現行route sourceの`locale: "ja-JP"`、`timezone: "UTC"`の意味軸で固定する。capture contextを`locale: "ja-JP"`、`timezoneId: "UTC"`で生成し、artifact生成前に`Intl.DateTimeFormat().resolvedOptions()`等でruntime観測して、一致しない場合は`停止線`としてpublicationせず終了する。

### 停止線
- `ja-JP`または`UTC`と異なる観測値が返る。
- 追加のlocale/timezone ownerを作る。
- `old #reference/*` routeを起点に他routeへ値を再定義。

### 根拠authority
- `docs/reviews/2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md`
- `docs/mocks-ui/playwright.current-route-capture.config.js:21`（locale `ja-JP`の現行route）
- `docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md`
- `docs/reviews/2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md`

## CP-2

### 裁定
`browserVersion`は起動実体の`browser.version()`、`browserRevision`は`playwright-core` lock metadata上の`chromium-headless-shell`項目から導出する。既存の`channel: "chrome"`設定は維持し、capture実体としてはPlaywright-bundled `chromium-headless-shell`のみを採用する。

### 停止線
- `playwright-core` lock metadata内`chromium-headless-shell`欠落。
- 実体版とmetadata `revision`不一致。
- 既存の`readBrowserDescriptor`/`channel`不変条件を改変して検証を回避する。

### 根拠authority
- `docs/mocks-ui/scripts/reference-capture.mjs:26-27` の`readBrowserDescriptor`実装（既存導出パターン）
- `docs/mocks-ui/playwright.config.js:17`と`docs/mocks-ui/playwright.current-route-capture.config.js:21`
- `docs/mocks-ui/package.json`

## CP-3

### 裁定
現行capture前に`docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2`（400）と`inter-latin-600-normal.woff2`（600）をfamily `Inter`の`FontFace`として読み込み、`document.fonts.add()`して`.app`の`computedFamily`先頭が`Inter`かをruntime観測する。

### 停止線
- 2件どちらかのfontFaceロード失敗。
- `.app`の`getComputedStyle`先頭tokenが`Inter`でない。
- 1つ目/2つ目weightの`document.fonts.check`不成立。
- `reference-font.css`や既存CSS、`old` reference route assetを変更。

### 根拠authority
- `docs/mocks-ui/src/tokens/mock-candidates.css:34`
- `docs/mocks/m3-vism-host-boundary.html:20`と、それを`?raw`注入する`docs/mocks-ui/src/legacy/legacySource.js`
- `docs/mocks-ui/src/main.jsx:123-138`および`docs/mocks-ui/tests/current-route-capture-v1etc.playwright.js`
- `docs/mocks-ui/scripts/reference-capture.mjs:121-145`（既存FontFace導入とfallback停止の実装）

## 実装oracle
1. capture contextを`locale: "ja-JP"` / `timezoneId: "UTC"`で生成し、page内の`Intl.DateTimeFormat().resolvedOptions()`または同等手段でlocale/timeZoneを再読して、一致しなければartifactを書かず中止する。
2. manifest `environment.locale === "ja-JP"`かつ`environment.timezone === "UTC"`。同値はこの文書以外に第二ownerを残さない。
3. launchはPlaywright-bundled `chromium-headless-shell`を使用し、`environment.browserVersion === browser.version()`、`environment.browserRevision === browsers.json.chromium-headless-shell.revision`。不一致またはdescriptor未取得はpublication前の`停止線`。
4. `environment.fontFixture.files`は
   - `docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2`（SHA: `8909904ab6c872eb994093482a88a28eca2cd95912d7b6fecd72103b0dc07edc`）
   - `docs/mocks-ui/public/reference-fonts/inter-latin-600-normal.woff2`（SHA: `f9a06e79cd3a2a20951c0f0e28f66dd0e6d3fda73911d640a2125c8fcb78f21a`）
   の2件のみを昇順で記録し、weightを400 / 600で記録する。
5. `environment.fontFixture.computedFamily`は`.app`で観測した`font-family`文字列とし、先頭familyが`Inter`。
6. `docs/mocks-ui/scripts/current-route-generation.mjs` schema 9軸を追加なしで満たす。
7. 上記1〜5のいずれか違反であれば、manifest・画像・部分artifactを残さない。

## 確定しないこと
- `ui-visual-language.md`の一般UI言語既定（`en-US`）の変更。
- `reference` legacy routeの`locale/localeFallback`再定義。
- FontのCSS/stackの恒久値変更。
- 文字化け回避の暫定値の追加。
- `browser`の新規runtime API追加。

## 非目標
- scripts, tests, configs, packages, fixtures, assets, CSS, React source, routes, public APIs, Document/plugin contracts, output artifacts, provenance instancesの変更。

## handoff

| ID | 状態 | 根拠 |
| --- | --- | --- |
| G0-6H-V1G-P | `決定` | `G0-6H-V1G`分割前提を満たす既存decisionの参照元 |
| G0-6H-V1G-I | `決定` | manifest/polished provenance系依存と前提整合 |
| G0-6H-V1G-C-P | `決定` | locale/timezone・browser・font family authorityを再締結し、`G0-6H-V1G-C`起動条件を満たす |
| G0-6H-V1G-C | `観察` | 本決定の3軸完了を`発注依存証跡`で確認後起動 |
| G0-6H-V1G | `未統一` | `G0-6H-V1G-C`完了後に`V1G-O`へ接続 |

## Reactラベル

### REACT AUTHORITY

対象面は`#plugin-browser-candidate`上のproduct-owned`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`とmock consumer`docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx`、route registry`docs/mocks-ui/src/main.jsx`。移管契約は[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。UI runtime境界は[UI runtime architecture](../ui-runtime-architecture.md)のbundled first-party Host module。対応spec IDはG0-6H-V0 / V1S / V1P / V1R / V1ETC / V1ETB / V1ETT / V1ETE / V1G-P / V1G-I / V1G-C-P。

### SOURCE ASSET

固定source commitは`ui/motolii-web/source-provenance.json#fixedSourceCommit`の`56c318edcddab7cf95d263cc2f7dd2b4e6791134`、対象exportは`ui/motolii-web/src/index.js`の`DiscoveryBrowserCandidate`。`ui/motolii-web/src/candidates/discovery-browser-candidate.css`、`docs/mocks-ui/tests/browser-candidate.spec.js`、`docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`、`ui/motolii-web/guard-tests/browser-ownership.test.mjs`、`docs/mocks-ui/src/tokens/mock-candidates.css`、`docs/mocks-ui/src/reference/reference-font.css`、`docs/mocks/m3-vism-host-boundary.html` — はこのgrainで1 byteも変更しない。

### PRESERVE

既存DOM、stable IDs、classes、ARIA、interaction、visual state、`#plugin-browser-candidate`、`.app`、`#stage`、`#inspector`、`#timeline`、`#project-browser`、`#interval-easing`、`#easing-panel`、`.app[data-parity-ready]`、`#root[data-current-route-capture-ready]`、post-promotion provenance、ownership guards、`docs/mocks/m3-vism-host-boundary.html:20`のlegacy `body` font宣言を維持する。

### REPLACE

なし。mock/legacy stateからprojection/intentへの交換を本粒で行わない。

### STATE OWNER

capture envelope、`current-route-capture` mode carrier、ページ側`FontFace`登録は`Transient / local presentation / development-only`。
`generation manifest`所有はfixture-only evidence surface。Document、User settings、Workspace、Project session、永続Host契約へ保存しない。

### DIAGNOSTIC ROUTE

製品画面は不変の`#plugin-browser-candidate`。開発確認は既存`current-route-capture` Vite modeと旧`#reference/*`生成で限定。新route/hash/query/mode/entryの追加なし。

### NEGATIVE ORACLE

duplicate component copies、legacy runtime imports、opaque catalog tokenからの意味推測、duplicate state、二重`9`環境axis owner、visual threshold/golden変更、new public export、post-render DOM mutation、product/reference CSS編集、machine-channel Chromeをcapture browserとして採用は棄却。

### STOP

未決product意味、公開契約変更、source asset不在、state-owner違反、allowlist外変更、environment literalまたはgeneration hashの未定義新規発明が必要な場合は停止。

## 関連

- [AGENTS.md](../../AGENTS.md)
- [G0-6H-V1G-P mechanics決定](2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md)
- [G0-6H-V1P 捕捉前提裁定](2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md)
- [G0-6H-V1ETB-P Browser移管境界裁定](2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md)
- [implementation-ledger](../implementation-ledger.md)
