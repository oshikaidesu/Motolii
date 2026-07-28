# G0-6H-V1P 現行route capture前提の裁定

- 日付: 2026-07-28
- 状態: **決定**
- G0-6H-V1P: **DONE**

## P-1 — screen 1の注入境界

### 裁定

- screen 1へ到達する入力は、`docs/mocks-ui`のmock consumerが所有するdevelopment専用の**型付きcapture projection envelope**一つだけとする。同じproduct-owned `DiscoveryBrowserCandidate`へ描画前に渡し、第二component、第二CSS copy、forkは作らない。
- 同じenvelopeがStage / Inspector / Timelineを空として投影する。Browserは`Media`面を表示し、既存の固定Starter Mediaカプセル4件（`starter-clip.mp4` / `starter-mark.svg` / `starter-still.png` / `starter-tone.wav`）だけを`starter-media-provenance.json`どおりに描画する。
- 実装粒`G0-6H-V1`は、mock consumerからproduct componentへenvelopeを運ぶために必要な最小の内部fixture adapter seamだけを作ってよい。このseamはdevelopment専用であり、`ui/motolii-web/src/index.js`へ新しいexportを加えず、envelopeが無い通常routeの出力を変えない。

### 停止線

- 描画後のDOM mutation、global変数、第二componentまたはCSS copy、opaque ID / label / thumbnail tokenからの推測、新route、hash key、query / search param、production catalog意味、Document state、公開API、恒久Host契約をseamにしない。
- 現行`CandidateBrowserTabs()`は`data-tab="effects"`をactive（`browser-tab on`）にするため、Browserを`Media`へ置く責任は描画後clickでなくdevelopment envelopeが持つ。
- 現行`DiscoveryBrowserCandidate`のsignatureは`({ node, options })`である。この内部signatureに必要最小の入力を加えても、新しい公開exportにはしない。

### 根拠authority

- [G0-6H-V1S B-2 / B-3](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)
- [G0-6H-A A-1 / A-2 / A-3 / A-7](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)
- [G0-6H-V0 V-3 / V-4](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [G0-6H-M §5](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)

## P-2 — screen 2〜5の操作とoracleの閉集合

| screen | 操作 | oracle |
|---|---|---|
| screen 2 | parity-ready後の初期candidate route | `.app[data-parity-ready="true"]` |
| screen 3 | 既存accessible button `Pulse rings · IntensityのInterval Easing Editorを開く`をclick | 既存の`Interval Easing Editor` complementary surfaceがvisibleかつ`aria-hidden="false"` |
| screen 4 | 既存`Hand` buttonをclick | buttonがactiveかつ`#stage.interaction-hand` |
| screen 5 | 既存`Relative Move` buttonをちょうど1回click | buttonがactive、`#stage.interaction-relative`、既存motion pathがvisible |

### 裁定

- oracleの閉集合は上表の4件だけとし、現行stable ID / ARIA / class / visible stateだけを使う。
- screen 2は`#plugin-browser-candidate`のparity-ready初期状態、screen 3は既存accessible buttonから開くeasing面、screen 4とscreen 5は既存interaction stateと可視性を審判する。
- `Relative Move`は1回のclickで`interaction-relative`へ到達する。2回目のclickによるcommitはcapture操作へ含めない。

### 停止線

- これらのoracleは状態へ到達できることとvisible / class / ARIAだけを審判し、element parityを主張しない。
- `G0-6H-M`の`partial` / `対応なし`を格上げせず、新しい評価意味も加えない。
- test ID、ARIA、class、threshold、goldenを新設または変更しない。

### 根拠authority

- [G0-6H-V1S B-1](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)
- [G0-6H-V0 V-2](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [G0-6H-M §3 / §5](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- 現行コード事実 7-14（`LegacyHostBoundaryScreen` parity-ready、`#interval-easing` / `#easing-panel`、`Hand` / `Relative Move`、`interaction-*` class、motion path表出）

## P-3 — font軸の観測点

### 裁定

- product CSSを変えずにcapture font軸を観測する。capture前にcapture環境が既存同梱font（`docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2` / `inter-latin-600-normal.woff2`）をloadして検証し、generation manifestがliteralなfont fixture path、digest、computed familyを記録する。
- load失敗またはfallback不一致はpublication前のSTOPとし、generationをpublishせず画像も受理しない。

### 停止線

- `reference-font.css`、candidate routeのfont stack、product CSSを変更しない。
- `#reference/*`専用assertionをproduct DOMへ拡張しない。
- font軸の記録ownerをgeneration manifest以外へ増やさない。B-4どおりcapture環境9軸の記録責任はmanifest一面に置く。

### 根拠authority

- [G0-6H-V1S B-4](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)
- [G0-6H-V0 V-5 / V-7](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- 現行コード事実 15-17（`docs/mocks-ui/public/reference-fonts/*`、`reference-font.css`、`reference-capture.mjs`）

## 確定しないこと

- envelopeのfield名、prop名、schema、algorithm、command、manifest schema、hash algorithm名、file配置、variant algorithm、threshold、tolerance、capture環境の具体値、token候補、human session内容は決めない。
- V0 / V1Sの境界、Starter Mediaのbyte / schema / path、旧reference generation、visual threshold / golden、product DOM / CSS / 通常interaction、human session、token選定、`G0-6H`の状態、`U0e-3` lockを維持する。

## 非目標

- product DOM / CSS、test、route、schema、fixture、token、command、thresholdの変更。
- 公開APIの変更。
- seam、envelope、schema、command、selectorの実装。
- route / hash / queryの新設、media byte / pathの再生成、production catalog意味の追加。

## 次の一粒

**`G0-6H-V1`**だけを次の一粒として`DO`へ戻し、P-1 / P-2 / P-3の境界内で実装する。

## handoff

| ID | 状態 | 内容 |
|---|---|---|
| `G0-6H-S` | **DONE** | 前提。screen 1/2〜5境界を受領し、current-route captureへ進行 |
| `G0-6H-M` | **DONE** | 前提。semantic gap の意味上限 |
| `G0-6H-A` | **DONE** | 前提。Starter Media fixture所有境界 |
| `G0-6H-AF` | **DONE** | 前提。source / provenance分類 |
| `G0-6H-AG0` | **DONE** | 前提。generator境界処分 |
| `G0-6H-AG` | **DONE** | 前提。FROZEN証拠カプセル維持 |
| `G0-6H-V0` | **DONE** | 前提。capture evidence契約 |
| `G0-6H-V1S` | **DONE** | 前提。screen1/2-5 boundary裁定 |
| `G0-6H-V1P` | **DONE** | 本決定を受領し、`G0-6H-V1`へ返送 |
| `G0-6H-V1` | **DO** | 本決定適用後に現実装へ進む |
| `G0-6H` | **DO / HUMAN** | 据え置き |

## 関連

- [G0-6H-V1P選定](2026-07-28-g0-6h-v1p-capture-prerequisite-selection.md)
- [G0-6H-V1S裁定](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)
- [G0-6H-V0現行route variant evidence契約](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)
- [G0-6H-M current-route semantic gap mapping](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)
- [G0-6H-A empty-project Starter Media scenario契約](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)

## React authority

### REACT AUTHORITY
- 対象面は`ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`の`#plugin-browser-candidate`上の`DiscoveryBrowserCandidate` Media面。mock consumerは`docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx`、route registryは`docs/mocks-ui/src/main.jsx`。
- 移管契約は[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)。
- UI runtime境界は[ui-runtime-architecture](../ui-runtime-architecture.md)。Browserはfirst-party Host moduleである。
- 対応spec IDは`G0-6H-S / G0-6H-M / G0-6H-A / G0-6H-AF / G0-6H-AG0 / G0-6H-V0 / G0-6H-V1S / G0-6H-V1P`、capsule commitは`e4ad5c9f`。

### SOURCE ASSET
- 固定source commitは`ui/motolii-web/source-provenance.json#fixedSourceCommit`の`56c318edcddab7cf95d263cc2f7dd2b4e6791134`。
- 対象exportは`ui/motolii-web/src/index.js`の`DiscoveryBrowserCandidate`。本決定でexportを追加しない。
- CSS / model / test closure（`ui/motolii-web/src/candidates/*.css`、`docs/mocks-ui/tests/browser-candidate.spec.js`、`docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`、`ui/motolii-web/guard-tests/browser-ownership.test.mjs`）は変更しない。

### PRESERVE
- `#stage` / `#interval-easing` / `#easing-panel` / `#project-browser` / `.app[data-parity-ready]`のstable IDとARIA / class / interaction / visual stateを維持する。
- `Hand` / `Relative Move` / `Pulse rings · IntensityのInterval Easing Editorを開く` / `Interval Easing Editor` / `Relative Move motion path`を含む対話文言を維持する。

### REPLACE
- 本決定は文書上の境界だけを決める。`G0-6H-V1`で、mock-local fixture stateを同じproduct componentへ描画前に渡すseamへ必要最小限に交換する。

### STATE OWNER
- envelopeは`Transient / local presentation / development-only`であり、`docs/mocks-ui` mock consumerが所有する。Document / User settings / Workspace / Project session / 恒久Host契約には属さない。

### DIAGNOSTIC ROUTE
- product screenは変更しない`#plugin-browser-candidate`。development検証は旧`#reference/*` guard generation / manifest照合と分離し、route / hash / queryを追加しない。

### NEGATIVE ORACLE
- 第二component / CSS copy、product packageへのlegacy runtime import、opaque ID / label / thumbnail token分岐、二重state、新しい公開export、描画後DOM mutation、global変数、visual threshold / golden変更を拒否する。

### STOP
- 未決のproduct意味、公開契約の追加、source asset不在、state owner境界違反、allowlist外の変更が必要なら停止してCodexへ戻す。
