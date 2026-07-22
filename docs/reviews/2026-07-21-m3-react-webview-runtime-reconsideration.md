# M3 React / WebView UI runtime再選定（2026-07-21）

ステータス: **責任境界・surface topology決定 / platform受入比較中**。本調査と後続レビューを受け、React chrome + native Stage/Timeline + headless interactionを[UI runtime責任境界](../ui-runtime-architecture.md)へ、通常windowを[1 top-level wgpu Surface + 2 native viewport + opaque child WebView islands](2026-07-21-ui-surface-topology-decision.md)へ正本化した。2026-07-18のegui shell等は比較baselineとして保持し、G0-9実機spikeまでは製品統合とegui撤去を停止する。plugin UI公開契約は2026-07-22追補2によりG0-3 / GAP-13へ分離した。Rust/wgpu core、M2 Document、D2 command、単一writer、正準座標、preview/export同一評価、UI toolkit隔離は変更しない。

2026-07-22追補: 「Reactモックを製品候補資産とする」の実装方法は
[React製品資産の直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)を正本とする。
React所有面は固定sourceをproduct packageへ直接所有移管し、別の縮約componentへ作り直さない。
このsource移管とmock consumer化は可能だが、WebView Hostの製品統合、egui撤去、plugin UI公開契約、
platform合格を意味しない。

2026-07-22追補2: 本書は標準製品surfaceのruntime選定とplugin UI公開runtimeを同じG0-9へ寄せすぎていた。[surface実装と拡張所有の軸分離](2026-07-22-m3-surface-extension-axis-separation.md)により、OS topologyとnative／React surfaceはG0-9、Core／bundled Host module／pluginの所属とfirst／third-partyの公開・信頼境界はG0-3 / GAP-13で別に判定する。本書のHost/community同一kitはcomponent/test語彙の再利用原則であり、同じorigin、process、権限、window topologyを意味しない。

## 1. 再選定を開く理由

egui採用は、Rust coreが所有する既存wgpu deviceとnative textureを直接共有できること、UI toolkitを`motolii-ui`へ閉じられること、初期統合費が小さいことを根拠に成立した。この測定事実は撤回しない。

一方、採用後のReact統合モックとコミュニティ拡張の検討から、当初の比較で十分に重く見ていなかった次の要求が製品境界へ影響し始めた。

1. Host作者とコミュニティ作者が再利用可能なUI component、theme、検査語彙を持てること。公開kitのversion保証範囲はG0-3で別途決める
2. Rust UI実装を第三者panel参加の必須条件にしないこと
3. panel、Roto/Pen、Graph、Easing等の専用UIを既存Web技術とLLMで組み立てやすいこと
4. UIだけをRust core再起動なしでhot reloadできること
5. Reactモックのcomponent、fixture、Storybook、Playwright、stable IDを製品資産として継続利用できること

これは「Webで全処理する」案ではない。映像評価、GPU compositor、media、export、Document/commandはRust coreに残し、UI窓口だけを再選定する。

## 2. この会話で決めたこと

### 2.1 Host UIとコミュニティUIのcomponent／test語彙を不必要に二重化しない

Hostだけが完全なcomponent／test語彙を使い、第三者には理由なく縮小した別DSLまたは別toolkitだけを渡す二層構造を長期の正規形にしない。ただしHostのBrowser、Inspector、Timeline等はbundled first-party製品面であり、それ自体をplugin kitの公開契約にはしない。何を同じversioned kitとして外部保証するかはG0-3 / GAP-13で決める。

ただし、これは未監査の任意JavaScriptを即座に同一process権限で実行する決定ではない。配布、sandbox、権限、version互換、CSP、network/file access、署名、障害隔離は未決であり、plugin UI contractをコードで先に発明しない。

### 2.2 Reactモックを製品source assetとして扱う

固定参照`origin/codex/m3-mock-components`の`eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`には、少なくとも次の資産がある。

- `TimelineCandidate.jsx`、`DiscoveryBrowserCandidate.jsx`、`EasingGraphCandidate.jsx`
- `ResizablePanelLayout.jsx`
- React 19 + Vite 6の開発入口
- Storybook 10のcomponent catalog
- Playwright 1.61のinteraction/visual test
- `component-map.json`のstable component IDと責任分解

これらのcomponent責任、fixture、操作試験、stable ID、視覚比較は製品source assetである。
React所有面は[直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)に従ってproduct packageへ
単一ownerとして移し、mockをそのconsumerへ反転する。React state、DOM event、CSS px、legacy HTML bridge、
仮JSON、モック内だけの意味は製品契約ではないが、この禁止をsource assetの縮約再実装理由にしない。

### 2.3 Canvasを要する高頻度workspaceはReactの所有理由にしない

当初はReact runtime内のCanvas 2D/browser WebGPUも製品候補に含めたが、部分spikeとrenderer再選定後は
Web先例baselineへ限定した。Canvasへ独自scene、selection、pointer、a11y proxyを持つ時点でReact/DOMの
優位を使わず第二UI engineになるため、Stage/Timelineの製品ownerにはしない。

```text
React / WebView
├─ DOM: shell、menu、Inspector、Browser、a11y、focus
└─ virtualized DOM: visible asset、panel collection

Native Rust / wgpu
├─ Stage: preview、handle、gizmo、roto overlay
└─ Timeline: ruler、track header、lane、clip、key、playhead、graph

Rust core / domain
├─ Document / D2 command / single writer
└─ render / eval / media / export
```

Web所有panel内の小さなvisualizationはDOMのform/a11yが主体でsemantic stateを二重所有しない場合だけ
Canvasを許す。browser WebGPUの`GPUDevice`とnative Rust/wgpuの`Device`は同一だと仮定せず、CPU pixel
bridgeを正規経路にしない。

## 3. 一次資料から確認した既成解

### 3.1 Browser / Explorer

[React Aria Virtualizer](https://react-aria.adobe.com/Virtualizer)はvisible itemだけをDOMへ置くList/Grid系virtualizationを提供し、公式の[Photo Library example](https://react-aria.adobe.com/examples/photos)はvirtualized photo grid、folder tree、search、multi-selection、accessible drag-and-dropを一つの例で示す。[React Aria v1.17.0](https://react-aria.adobe.com/releases/v1-17-0)ではFinder型expandable rowとhorizontal virtualizationも追加された。

[TanStack Virtual](https://tanstack.com/virtual/v3/docs)はmarkupを所有しないheadless virtualizerで、vertical、horizontal、両軸を組み合わせたgrid状virtualizationを持つ。したがってBrowser/Explorerは「既成解が無い」問題ではなく、React Ariaのcollection意味を採るか、TanStackでMotolii固有layoutを組むかをfixtureで選ぶ問題である。

### 3.2 Timeline

Timeline全体を扱う既成libraryも存在する。ただし2026-07-21時点では、そのままMotoliiの
完成品として採るだけの成熟・keyframe stress証拠は確認できない。

[Canvas Timeline](https://github.com/techsquidtv/canvas-timeline)はheadless state、snapping、
command-stack history、React interaction layer、worker-backed Canvas rendererを分離した
MPL-2.0の直接先例である。一方、現時点で0.1.0、9 stars、44 commitsで、対象runtimeは
Node 24以上、React 19.2.7以上であり、100,000 keyの公開fixtureは確認できない。構造は学ぶが、
依存採択は固定fixture、API/ライセンス/更新性監査まで保留する。

[React Video Editor Timeline](https://www.reactvideoeditor.com/docs/core/components/timeline)は
drag、snap、multi-select、zoom、Undo/Redoを持つcopy可能なReact実装だが、公式自身が開発中で、
大規模project性能はbrowserやfile規模に依存すると明記する。これも先例であって性能証明ではない。

[elah](https://www.elah.dev/)はReact UI、integer-frame timeline、Immer history、WebGL2、
OffscreenCanvas、pure resolverを分離したApache-2.0のbrowser-native editor engineである。
MotoliiのRust Document/render/exportを置換または二重化する採択はしないが、UI bindingとengineを
分離し、drag中previewとcommitを分ける構造の比較資料にする。

OpenCut v0.3.0は、[Timeline track内の全elementをReact DOMへ展開](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/timeline/timeline-track.tsx)する一方、[audio waveformはCanvasでvisible rangeだけを描画](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/timeline/audio-waveform.tsx)している。[Asset viewも同時点では全itemをDOM展開](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/assets/views/assets.tsx)している。

これはReact人口の多さだけで製品固有の2D virtualizationが自動完成しない反例である。同時に、DOMの操作性とCanvasの高密度描画を同じReact製品で混ぜる先例でもある。MotoliiはOpenCutの型・状態・DOM構造を輸入せず、visible range、virtualization、hit-test、selection維持をMotolii fixtureで判定する。

### 3.3 CanvasをDOM状に扱う既成scene graph

[Konva](https://konvajs.org/docs/overview.html)はStage / Layer / Group / ShapeというDOM状の
virtual nodeと、描画用Canvasとは別のhidden hit Canvasを持ち、event、drag、transform、layer単位の
再描画を提供する。公式の[20,000 Nodes](https://konvajs.org/docs/sandbox/20000_Nodes.html)は
20,000 circleすべてをhover/drag可能にし、drag対象を専用Layerへ移す既成patternを示す。
Pen/Roto、少数の編集handle、直接操作の有力候補だが、100,000 keyを常時node化する採択はしない。

[PixiJS scene graph](https://pixijs.com/8.x/guides/components/scene-objects)はContainer / Sprite /
Graphics / Mesh等の階層とDOM風の
[federated pointer events](https://pixijs.com/8.x/guides/components/events)を持つ。
[ParticleContainer](https://pixijs.com/8.x/guides/components/scene-objects/particle-container)は公式に
100,000 particle例を示し、position等をstatic/dynamicに分けてGPU uploadできる。ただしParticleは
個別event、child、filterを省いた軽量型で、APIはstableだがexperimentalと明記される。
したがってdense key描画はParticle/Mesh、選択handleだけ通常scene nodeという分担を候補にする。

React統合には公式の薄いbindingである[@pixi/react](https://react.pixijs.io/getting-started)、
pan / wheel / pinch / clamp / snapには
[pixi-viewport](https://viewport.pixijs.io/jsdoc/index.html)がある。これらを無視してMotolii独自の
JS scene graph、汎用gesture、viewportを先に作らない。

[Fabric.js](https://fabricjs.com/docs/why-fabric/)もCanvas上のobject model、interaction、event、
serialization、transform controlを提供する。Stage上の少数object編集候補として残すが、
Document正本をFabric JSONへせず、dense Timeline性能は別fixtureなしに推定しない。

### 3.4 Graph / 専用panel / LLM

[React Flow](https://reactflow.dev/api-reference/react-flow)はcustom node、pan/zoom、selection、snap、visible element限定描画を提供し、[custom node](https://reactflow.dev/learn/customization/custom-nodes)には通常のReact componentを埋め込める。これはGraph、node editor、専用可視化panelの部品母集団が大きいことの証拠だが、Motolii TimelineやDocument意味の完成品ではない。

[Vite HMR](https://vite.dev/guide/features)と既存モックのStorybook/Playwright構成により、Rust core再起動と分離したUI iterationを構成できる。LLM作成容易性は2026-07-21時点の開発運用上の比較軸とし、公開API・永続形式・安全性の根拠にはしない。

## 4. 解けている範囲とMotolii固有の残件

| 面 | 既成部品で大半を解ける | Motoliiで決める |
|---|---|---|
| Browser | list/grid/tree virtualization、keyboard、selection、DnD、lazy thumbnail | asset identity、query、cache、Workspace状態、Document非汚染 |
| Inspector/panel | form、popover、dock、theme、component catalog | `NodeDesc`投影、gesture/Undo、plugin権限、互換 |
| Graph | pan/zoom、custom node、edge、selection | 型互換、evaluation、保存、command |
| Timeline | headless pointer/viewport/geometry部品、wgpu/Vello | RationalTime↔x、semantic zoom、snap、offscreen drag、selection、1 gesture=1 Undo |
| Roto/Pen | headless pointer/path部品、Vello | 正準座標、pressure、hit-test、Stage合成、D2 command |
| Stage | Web側のpanel/overlay配置 | native wgpu Texture共有、色/alpha、OS別WebView合成、device lost |

人口の厚さで減るのはwidget、virtualization、a11y、test、toolingの再発明である。Motolii固有の時間意味、Document所有、GPU texture境界、plugin securityは人口では解けない。

## 5. G0-9 比較スパイク

G0-9は製品UI runtimeの再選定ゲートである。候補は少なくとも次を含む。

1. 現行egui + native wgpu baseline
2. React/WebView + DOM virtualization + Web所有panel内だけの局所Canvas/browser WebGPU
3. React/WebView shell + native wgpu Stage/TimelineをOS compositorで合成するhybrid

同じMotolii fixture、同じ操作、同じ計測手順で比較し、候補ごとに別の完成条件を置かない。

### 5.1 必須fixture

- Browser: 1,000 / 10,000 itemのlist/grid/tree、検索、multi-selection、DnD、thumbnail遅延
- Timeline: 1,000 clip / 100,000 keyのdensity・cluster・individual、横縦scroll、pointer中心zoom、snap、画面外drag、box selection
- Panel: `NodeDesc`自動panelと、curve/graph/penを含む専用panel候補
- Stage: 640×360以上のnative wgpu preview、resize/minimize/restore、overlay、alpha、色、古いgeneration破棄
- DX: component変更のhot reload、Storybook、interaction test、LLMによる既存component再利用
- OS: macOSに加えWindows WebView/IME。Linuxは採用候補runtimeの配布形が決まった後に同じfixtureを適用

数値は性能契約ではなく比較用stress tierである。初回実測後にG0-4手順で製品閾値を独立決定する。

### 5.2 製品surfaceの共通合否

- UI threadがrender、同期readback、blocking transportを待たない
- Rust/wgpu previewのCPU pixel roundtripを正規経路にしない
- DOM node数がproject総数でなくvisible rangeに概ね比例する
- zoom anchor、snap、selection、dragがvirtualizationで意味を変えない
- UI座標、DOM event、CSS px、toolkit型をDocument/公開domain APIへ出さない
- UI component変更でRust coreを再起動せず反映できる
- IME、keyboard、focus、screen readerの保証範囲を候補ごとに明記できる

Host/communityの公開kit再利用、plugin crash/loop、権限、依存更新、offline配布はG0-3 / GAP-13の合否へ移す。G0-9ではWebView realmとtyped brokerのplatform証拠を採取できるが、plugin公開契約の合否に代えない。

### 5.3 STOP

- browser WebGPUとnative wgpuのTexture共有を実測なしで「ゼロコピー」と記述した
- React mockのJSON、CSS、DOM eventを製品契約へ昇格した
- 比較前に既存egui骨格を削除した、またはReact側へRust core意味を複製した
- Host用とcommunity用で別のcomponent kitを正規化した
- 自由UIを理由にDocument mutation、file/network、GPU resourceを無制限公開した
- OpenCut、React Aria、TanStack、React Flowの内部状態をMotolii仕様へ逆算した

### 5.4 2026-07-21 部分スパイク結果

[G0-9 UI runtime部分スパイク](../spikes/g0-9-ui-runtime.md)で、React virtual listは
10,000 itemを24 DOM rowへ限定し、1,000 clip / 100,000 keyのdense surfaceはCanvas 2Dと
browser WebGPUの両経路で成立した。追試ではPixiJS 8.19.0とKonva 10.3.0を用い、20,000 visible
keyの上で1 / 1,000 / 10,000 key、1 / 100 / 1,000 objectをgroup dragした。React stateと
semantic commitをmove中に更新せず、Cancel復元とrelease時1 commitをadapter harnessで確認した。
Vite HMRもRust再起動なしでacceptした。現行native wgpu timelineとegui shell境界もApple M4 /
Metalで再合格した。

追試の現在値は[全確認点マトリクス](../spikes/g0-9-verification-matrix.md)へ分離した。Playwrightの
実mouseでKonva group drag、10 CSS px snap、canvas外移動後Cancel、marqueeと標準pointer captureを
確認し、macOS Safari実機ではWebKit描画とAX treeへのCanvas説明・bounded selection proxy公開を
確認した。これはMotolii RationalTime snap、D2 Undo、日本語IME候補窓、VoiceOverの合格ではない。

object handle追試では、Konva Transformerで2D move/scale/rotate/multi-select/Cancel、Three.js
TransformControlsで3D translate/scale/rotate、world/local、snap、camera orbit排他を実mouseで確認した。
これは既成DCC機構を再利用できる証拠であり、製品renderer/runtime採否の点数ではない。
[Native Stage gizmo所有境界](2026-07-21-native-stage-gizmo-ownership.md)で、2D/3D handleはcanonical出力外の
native wgpu presentation overlay、hit-testはCPU解析幾何、commitはD2が所有すると決定した。Webは同じkitの
toolbar/control/a11y proxyを所有し、transparent/composition overlayをgizmo要件にしない。M5 P2Uの
ScaleとDepth Moveの別channel、perspective/orthographic、D2 Undoは別Stage spikeで未接続である。
Host/communityのUI kit統一とStage描画surfaceの所有は別論点として扱う。

後続の[surface topology決定](2026-07-21-ui-surface-topology-decision.md)により、WebView/native Stageは
Texture共有でなく、1 top-level wgpu Surface内のStage/Timeline viewportとopaque child WebView islandsを
OS compositorで合成する。Host/communityは同じversioned React kitを使うが、
community realmはnamed least-privileged WebView + typed Rust brokerを安全基準に分離する。これらは
一次資料上の候補である。軽量比較のopaque-origin iframeではparent DOM、storage、network、native
bridgeの直接access拒否を部分確認したが、別WebViewのnative IPC、loop/crash/OOM隔離は未証明である。
製品組込みと公開契約を許可する採択ではない。browser WebGPUとnative wgpuを
同じdeviceとみなす禁止は維持する。

したがってG0-9が比較するWeb runtime範囲はshell、panel、Browser、form、Timeline外側control、hot reload、
community realm、IME/a11y、offline配布である。Stage gizmoのvisual rendererは候補間の比較項目から外す。
さらにTimelineのruler/lane/clip/key/playhead/selection/graphはnative所有へ絞り、Reactは外側のtoolbar/menu/
parameter編集を持つ。native実装は[renderer再選定](2026-07-21-native-surface-renderer-reselection.md)どおり
direct wgpu第一候補 + Vello局所利用をwindowed spikeし、eguiは成立済みbaselineとして残す。

## 6. 既存台帳の扱い

- [egui採用判断](2026-07-18-m3-egui-selection.md)は削除しない。native wgpu共有、IME、CJK、lifecycle、計測の証拠として比較入力にする
- [Rerun学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)のselection、density、cache、GPU lifecycle、testingはtoolkit横断の先例として残す。`re_ui`/`egui_tiles`/egui callbackの依存・vendoring・移植だけG0-9まで停止する
- React component、CSS、component map、Storybook、PlaywrightはReact所有面の直接移管sourceと比較oracleにする。
  DOM/CSSを仕様・公開契約の正本にはしない
- G0-3の`NodeDesc`自動panelは必須fallbackとして維持する。「第三者の自由UIを長期に公開しない」という結論の再評価はG0-3 / GAP-13で行い、G0-9のsurface証拠を入力にするが合否を共用しない
- 現行mainのegui shell、native texture preview、layout投影、render worker、依存方向CIは比較基準として保持する。比較完了前に削除せず、toolkit固有の製品面をさらに拡張または公開型化しない

## 7. 現在の実装許可

G0-9完了まで許可するのは、一次資料調査、固定fixture、benchmark harness、Reactモック内の比較prototype、
[直接移管契約](2026-07-22-m3-react-product-asset-promotion-contract.md)R0〜R6のproduct-owned React packageと
mock consumer化、toolkit非依存の状態所有/domain intent/Command境界、既存Rust coreの作業である。

egui固有の製品shell/panel/Timeline、WebView Hostとnative surfaceの製品組込み、plugin自由UI公開API、
永続layout形式、WebView権限モデルは実装しない。React source assetをproduct packageへ移すことを、
WebView Hostやplatform受入の完成へ数えない。比較結果を採択する時は、本書を根拠にM3 spec、
implementation ledger、G0-3/G0-1、Rerun転移分類を同じ変更で改訂する。
