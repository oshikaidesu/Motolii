# G0-6H-V1ETA empty projection段階化の裁定

日付: 2026-07-28  
状態: 決定

## 対象

- `G0-6H-V1ET`: screen 1のBrowser / Inspector / Stage / Timeline空投影とready oracle
- 正本: [G0-6H-V1R裁定](2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md)
- UI runtime境界:
  [React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)

## 現行code事実

- `docs/mocks-ui/src/main.jsx`の`plugin-browser-candidate`が現行routeを登録し、
  `LegacyHostBoundaryScreen`へproduct Browserとmock Timelineを渡す。
- `LegacyHostBoundaryScreen.jsx`がlegacy script実行、Inspector / Stage / Timelineの
  parser置換を所有する。`main.jsx`だけでは空投影を作れない。
- product `DiscoveryBrowserCandidate`のMedia faceは固定legacy nodeの
  `#project-browser`を`CandidateProjectBrowser`へ交換する。現時点の公開signatureは
  `{ node, options }`である。
- `TimelineCandidate.jsx`がobject、key、automation、depth、5 band geometryと
  `KeyToolsCandidate`のcountを所有する。
- 通常Playwright serverはViteの`development` modeだけであり、
  `current-route-capture`専用channelは存在しない。
- `source-provenance.json` schema version 1のBrowser migrationは、旧固定sourceと
  現行component / CSS / patternのbyte一致をguardしている。

## A-1 実装粒と順序

`G0-6H-V1ET`を次の四粒へ分割する。

1. `G0-6H-V1ETC`: mock carrierとHost空投影。
2. `G0-6H-V1ETB`: product Browser projection seam、R-9描画、provenance再締結。
3. `G0-6H-V1ETT`: Timeline空投影と`KEYS / LAYERS` 0 / 0。
4. `G0-6H-V1ETE`: 同一routeの最終ready oracleと全閉集合の統合Playwright。

順序は`V1ETC → V1ETB → V1ETT → V1ETE`とし、`G0-6H-V1G`は
`V1ETE`完了まで待つ。

## A-2 carrierと実行channel

- mode名はVite標準の`current-route-capture`で固定する。
- `import.meta.env.MODE`を読むのは`docs/mocks-ui/src/main.jsx`だけとする。
- mode時もroute keyは`plugin-browser-candidate`のままにし、新route、hash、query、
  global、別served entry、Vite config、package scriptを追加しない。
- 専用Playwright channelは新規
  `docs/mocks-ui/playwright.current-route-capture.config.js`だけが所有する。
  web server commandは
  `npm run dev -- --mode current-route-capture --port 4174`、base URLは
  `http://127.0.0.1:4174`、test directoryは既存`tests`内の専用spec一件へ限定する。
- 実行commandは
  `npx playwright test --config playwright.current-route-capture.config.js`とする。
  通常`playwright.config.js`と通常route testは変更しない。

## A-3 V1ETCのHost空投影

- mode booleanは`main.jsx`から`LegacyHostBoundaryScreen`へtyped propとして渡す。
  `LegacyHostBoundaryScreen`内で`legacyScript`を実行しない。通常modeは現行effectを
  byte-equivalentな動作で維持する。
- `#inspector`は元nodeのtag / attributes / class / IDを保ち、children 0の`aside`として
  投影する。`InspectorContext`正本や空payloadを新設しない。
- Stageは`#stage`、output frame、panel chrome、transport、playheadを維持し、
  class selector `.scene-copy`、`.rings`、`.selection-bounds`、`.motion-path`、
  `.stage-hud`、`.stage-badge`のnodeをparser projectionで0件にする。
- DOM文字列の後処理、`innerHTML`書換え、legacy source変更、CSSでの単なる非表示を
  禁止する。
- V1ETCの専用Playwrightはlegacy scriptが未実行であること、空Inspector、
  Stageの維持selectorと除外selector、通常route不変を判定する。

## A-4 V1ETBのBrowser閉集合

- `DiscoveryBrowserCandidate`へ任意の`developmentProjection` propを追加する。
  propがない通常routeは現行DOM / class / stable ID / ARIA / interactionを不変にする。
- propがある時だけprivate `decodeStarterMediaProjection`をcomponent内で呼ぶ。
  decoder失敗はfallbackせずthrowする。
- tab rowは`Media`、`Effects`、`Create` buttonをchromeとして維持し、Mediaをactiveに
  する。Effects / Createのface、pack scope、plugin browser faceは描画しない。
- Media source railは次を維持する:
  `All Media` button、`Registered folders` / `Collections` / `Tags` / `Packs` heading、
  search、view toggle。Project / Recent button、folder / collection / tag / pack item、
  Add folder / New tag action、Hierarchy itemは0件にする。
- resultsは`#asset-source-title = All Media`、`#asset-scope-label`のtext空、
  `#asset-count = 4 ITEMS`、`.asset-tile` 4件、`.is-selected` / `[aria-pressed=true]`
  0件、`#asset-selection-count = 0 selected`とする。
- tileはdecoder出力順で、`data-asset`と表示nameをbasename、metaとARIAのmeta部分を
  literal `mediaType`にする。Project/status/pack/origin意味を付けず、
  `data-asset-origin` / `data-pack` / project copyを出さない。

### provenance version 1の追加field

top-levelに任意の`postPromotionChanges` arrayを一つ追加できる。各entryのkeyは
次の5個だけとする。

- `task`: `G0-6H-V1ETB`
- `file`: `ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx`
- `reason`: `development-only Starter Media projection`
- `fixedSourceSha256`: 固定commitの旧Browser component byte SHA-256
- `currentSha256`: V1ETB完了時の現行Browser component byte SHA-256

Browser migrationについてはcomponent byte一致を解除し、この2 hashを両方実byteへ
照合する。CSS / patternの固定byte一致は維持する。他migrationのbyte一致と
public export topologyは維持する。hash期待値だけを書き換えるのではなく、
post-promotion差分を許す新しい正負guardを同じ変更で追加する。

## A-5 V1ETTのTimeline閉集合

- `TimelineCandidate`へ任意の`emptyProjection` propを追加し、mode時だけtrueを渡す。
- header、ruler、playhead、既存5 bandの高さとpacking guide geometry、
  `KeyToolsCandidate`を維持する。
- object row / bar / key / selection、automation shelf、depth rail / marker、
  object由来mute / solo、band一括actionを0件にする。
- `KeyToolsCandidate`へ`keyCount={0}`、`layerCount={0}`を渡し、scopeの既存defaultを
  維持する。5 bandはDocument/object配列から導出せずpresentation定数を使う。
- 通常routeのTimeline interactionと既存期待値を変更しない。

## A-6 V1ETEのready oracle

- ready名はroot elementの
  `data-current-route-capture-ready="true"`で固定する。
  `data-parity-ready`を流用しない。
- `main.jsx`内のmode専用wrapperが`useLayoutEffect`で設定する。意味は
  「current-route-capture modeのReact subtreeが一度commitし、同期decoderと
  Host / Browser / Timeline projection renderが完了した」とする。
  network、timer、MutationObserver、legacy script完了を意味に含めない。
- cleanup時に属性を削除する。通常modeでは属性を一度も設定しない。
- 統合Playwrightはreadyを待った後、A-3 / A-4 / A-5の全正負selectorと通常route
  不変を同じ実行で再判定する。diagnostic routeだけを成果にしない。

## 状態所有

- modeとprojection envelopeは`Transient / development presentation`。
- Document、User settings、Workspace、Project session、Browser catalog、
  selection、Undoへ状態を追加しない。
- product componentはmock、legacy、fixture fileをruntime importしない。

## 非目標・停止線

- drag/drop、Preview object生成、D2 command、Undo、keyframe編集、等速直線運動。
- asset byte、golden、visual threshold、固定legacy HTML、CSS tokenの変更。
- package public export、Document、plugin契約、serde、永続形式の変更。
- selector閉集合をCSS非表示やtest期待値変更で成立させること。
- 各粒のPlaywright oracleが同じ粒で到達不能、通常routeが変化、または上記file /
  field閉集合で実装不能なら`ORDER: STOP`とする。

## 次の一粒

`G0-6H-V1ETC`だけを`DO`とする。
