# M3 React / WebView UI runtime再選定（2026-07-21）

ステータス: **比較中**。2026-07-18のegui採用判断と、現行mainで完了したegui shell、native texture preview、layout投影、render worker等を比較基準として保持しつつ、React / WebViewを製品UI runtime候補へ戻す。G0-9の比較が終わるまで、完了済み基準を越えるtoolkit固有の製品shell・panel・Timeline実装と、plugin UI公開契約の固定を停止する。Rust/wgpu core、M2 Document、D2 command、単一writer、正準座標、preview/export同一評価、UI toolkit隔離は変更しない。

## 1. 再選定を開く理由

egui採用は、Rust coreが所有する既存wgpu deviceとnative textureを直接共有できること、UI toolkitを`motolii-ui`へ閉じられること、初期統合費が小さいことを根拠に成立した。この測定事実は撤回しない。

一方、採用後のReact統合モックとコミュニティ拡張の検討から、当初の比較で十分に重く見ていなかった次の要求が製品境界へ影響し始めた。

1. Host作者とコミュニティ作者が同じUI kit、component、検査方法を使えること
2. Rust UI実装を第三者panel参加の必須条件にしないこと
3. panel、Roto/Pen、Graph、Easing等の専用UIを既存Web技術とLLMで組み立てやすいこと
4. UIだけをRust core再起動なしでhot reloadできること
5. Reactモックのcomponent、fixture、Storybook、Playwright、stable IDを製品資産として継続利用できること

これは「Webで全処理する」案ではない。映像評価、GPU compositor、media、export、Document/commandはRust coreに残し、UI窓口だけを再選定する。

## 2. この会話で決めたこと

### 2.1 Host UIとコミュニティUIを別kitにしない

Hostだけが完全なUI kitを使い、第三者には縮小した別DSLまたは別toolkitだけを渡す二層構造を長期の正規形にしない。HostのBrowser、Inspector、Timeline等を組むcomponentと、コミュニティpanelを組むcomponentは同じruntime・theme・interaction・test kitから供給する。

ただし、これは未監査の任意JavaScriptを即座に同一process権限で実行する決定ではない。配布、sandbox、権限、version互換、CSP、network/file access、署名、障害隔離は未決であり、plugin UI contractをコードで先に発明しない。

### 2.2 Reactモックを製品候補資産として扱う

固定参照`origin/codex/m3-mock-components`の`eb16d06f980b6f9bea3901b6f10cbcc21dbfb3d0`には、少なくとも次の資産がある。

- `TimelineCandidate.jsx`、`DiscoveryBrowserCandidate.jsx`、`EasingGraphCandidate.jsx`
- `ResizablePanelLayout.jsx`
- React 19 + Vite 6の開発入口
- Storybook 10のcomponent catalog
- Playwright 1.61のinteraction/visual test
- `component-map.json`のstable component IDと責任分解

これらのcomponent責任、fixture、操作試験、stable ID、視覚比較は製品候補資産である。React state、DOM event、CSS px、legacy HTML bridge、仮JSON、モック内だけの意味は製品契約ではない。

### 2.3 WebGPUは全面置換でなく局所surfaceとして比較する

Reactを選んでも全要素をDOMにする必要はない。高密度surfaceは同じReact runtime内でCanvas 2D、browser WebGPU、または別のGPU surfaceへ分離できる。

```text
React / WebView
├─ DOM: shell、menu、Inspector、Browser、a11y、focus
├─ virtualized DOM: visible asset、track header、通常密度clip
└─ Canvas / browser WebGPU: ruler、waveform、密集key、guide、graph

Rust core
├─ Document / D2 command / single writer
├─ render / eval / media / export
└─ native wgpu Stage texture
```

ここでbrowser WebGPUの`GPUDevice`とnative Rust/wgpuの`Device`は同一だと仮定しない。WebView境界を越えたTexture共有、同期、色/alpha、resize、overlay、device lostはOS/runtime依存の未検証事項であり、CPU pixel bridgeを正規経路にしない。

## 3. 一次資料から確認した既成解

### 3.1 Browser / Explorer

[React Aria Virtualizer](https://react-aria.adobe.com/Virtualizer)はvisible itemだけをDOMへ置くList/Grid系virtualizationを提供し、公式の[Photo Library example](https://react-aria.adobe.com/examples/photos)はvirtualized photo grid、folder tree、search、multi-selection、accessible drag-and-dropを一つの例で示す。[React Aria v1.17.0](https://react-aria.adobe.com/releases/v1-17-0)ではFinder型expandable rowとhorizontal virtualizationも追加された。

[TanStack Virtual](https://tanstack.com/virtual/v3/docs)はmarkupを所有しないheadless virtualizerで、vertical、horizontal、両軸を組み合わせたgrid状virtualizationを持つ。したがってBrowser/Explorerは「既成解が無い」問題ではなく、React Ariaのcollection意味を採るか、TanStackでMotolii固有layoutを組むかをfixtureで選ぶ問題である。

### 3.2 Timeline

Timeline全体を解く単一libraryは確認できなかった。ただし両軸virtualization、Canvas/WebGPU描画、pointer capture、Reactの状態投影を組み合わせる部品はある。

OpenCut v0.3.0は、[Timeline track内の全elementをReact DOMへ展開](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/timeline/timeline-track.tsx)する一方、[audio waveformはCanvasでvisible rangeだけを描画](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/timeline/audio-waveform.tsx)している。[Asset viewも同時点では全itemをDOM展開](https://github.com/opencut-app/opencut/blob/f4bd689f51cf12a4dd0a32f602f761be314d9686/apps/web/src/components/editor/panels/assets/views/assets.tsx)している。

これはReact人口の多さだけで製品固有の2D virtualizationが自動完成しない反例である。同時に、DOMの操作性とCanvasの高密度描画を同じReact製品で混ぜる先例でもある。MotoliiはOpenCutの型・状態・DOM構造を輸入せず、visible range、virtualization、hit-test、selection維持をMotolii fixtureで判定する。

### 3.3 Graph / 専用panel / LLM

[React Flow](https://reactflow.dev/api-reference/react-flow)はcustom node、pan/zoom、selection、snap、visible element限定描画を提供し、[custom node](https://reactflow.dev/learn/customization/custom-nodes)には通常のReact componentを埋め込める。これはGraph、node editor、専用可視化panelの部品母集団が大きいことの証拠だが、Motolii TimelineやDocument意味の完成品ではない。

[Vite HMR](https://vite.dev/guide/features)と既存モックのStorybook/Playwright構成により、Rust core再起動と分離したUI iterationを構成できる。LLM作成容易性は2026-07-21時点の開発運用上の比較軸とし、公開API・永続形式・安全性の根拠にはしない。

## 4. 解けている範囲とMotolii固有の残件

| 面 | 既成部品で大半を解ける | Motoliiで決める |
|---|---|---|
| Browser | list/grid/tree virtualization、keyboard、selection、DnD、lazy thumbnail | asset identity、query、cache、Workspace状態、Document非汚染 |
| Inspector/panel | form、popover、dock、theme、component catalog | `NodeDesc`投影、gesture/Undo、plugin権限、互換 |
| Graph | pan/zoom、custom node、edge、selection | 型互換、evaluation、保存、command |
| Timeline | 両軸virtualization、Canvas、pointer primitive | RationalTime↔x、semantic zoom、snap、offscreen drag、selection、1 gesture=1 Undo |
| Roto/Pen | SVG/Canvas pointer pathとoverlay | 正準座標、pressure、hit-test、Stage合成、D2 command |
| Stage | Web側のpanel/overlay配置 | native wgpu Texture共有、色/alpha、OS別WebView合成、device lost |

人口の厚さで減るのはwidget、virtualization、a11y、test、toolingの再発明である。Motolii固有の時間意味、Document所有、GPU texture境界、plugin securityは人口では解けない。

## 5. G0-9 比較スパイク

G0-9は製品UI runtimeの再選定ゲートである。候補は少なくとも次を含む。

1. 現行egui + native wgpu
2. React/WebView + DOM virtualization +局所Canvas/browser WebGPU
3. React/WebView shell + native wgpu Stageを別surfaceとして合成するhybrid

同じMotolii fixture、同じ操作、同じ計測手順で比較し、候補ごとに別の完成条件を置かない。

### 5.1 必須fixture

- Browser: 1,000 / 10,000 itemのlist/grid/tree、検索、multi-selection、DnD、thumbnail遅延
- Timeline: 1,000 clip / 100,000 keyのdensity・cluster・individual、横縦scroll、pointer中心zoom、snap、画面外drag、box selection
- Panel: `NodeDesc`自動panelと、curve/graph/penを含む専用panel候補
- Stage: 640×360以上のnative wgpu preview、resize/minimize/restore、overlay、alpha、色、古いgeneration破棄
- DX: component変更のhot reload、Storybook、interaction test、LLMによる既存component再利用
- OS: macOSに加えWindows WebView/IME。Linuxは採用候補runtimeの配布形が決まった後に同じfixtureを適用

数値は性能契約ではなく比較用stress tierである。初回実測後にG0-4手順で製品閾値を独立決定する。

### 5.2 共通合否

- UI threadがrender、同期readback、blocking transportを待たない
- Rust/wgpu previewのCPU pixel roundtripを正規経路にしない
- DOM node数がproject総数でなくvisible rangeに概ね比例する
- zoom anchor、snap、selection、dragがvirtualizationで意味を変えない
- UI座標、DOM event、CSS px、toolkit型をDocument/公開domain APIへ出さない
- Hostとコミュニティが同じUI component/test kitを使える
- UI component変更でRust coreを再起動せず反映できる
- IME、keyboard、focus、screen readerの保証範囲を候補ごとに明記できる
- plugin crash/loop、権限、依存更新、offline配布を隔離できる見通しがある

### 5.3 STOP

- browser WebGPUとnative wgpuのTexture共有を実測なしで「ゼロコピー」と記述した
- React mockのJSON、CSS、DOM eventを製品契約へ昇格した
- 比較前に既存egui骨格を削除した、またはReact側へRust core意味を複製した
- Host用とcommunity用で別のcomponent kitを正規化した
- 自由UIを理由にDocument mutation、file/network、GPU resourceを無制限公開した
- OpenCut、React Aria、TanStack、React Flowの内部状態をMotolii仕様へ逆算した

## 6. 既存台帳の扱い

- [egui採用判断](2026-07-18-m3-egui-selection.md)は削除しない。native wgpu共有、IME、CJK、lifecycle、計測の証拠として比較入力にする
- [Rerun学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)のselection、density、cache、GPU lifecycle、testingはtoolkit横断の先例として残す。`re_ui`/`egui_tiles`/egui callbackの依存・vendoring・移植だけG0-9まで停止する
- React component mapとStorybookは比較oracleへ昇格するが、DOM/CSSを仕様正本にしない
- G0-3の`NodeDesc`自動panelは必須fallbackとして維持する。ただし「第三者の自由UIを長期に公開しない」という結論はG0-9とplugin sandbox/compatibility判断まで再固定しない
- 現行mainのegui shell、native texture preview、layout投影、render worker、依存方向CIは比較基準として保持する。比較完了前に削除せず、toolkit固有の製品面をさらに拡張または公開型化しない

## 7. 現在の実装許可

G0-9完了まで許可するのは、一次資料調査、固定fixture、benchmark harness、Reactモック内の比較prototype、toolkit非依存の状態所有/domain intent/Command境界、既存Rust coreの作業である。

egui固有の製品shell/panel/Timeline、React/WebViewの製品組込み、plugin自由UI公開API、永続layout形式、WebView権限モデルは実装しない。比較結果を採択する時は、本書を根拠にM3 spec、implementation ledger、G0-3/G0-1、Rerun転移分類を同じ変更で改訂する。
