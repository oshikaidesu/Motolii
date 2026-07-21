# UI runtime責任境界

状態: **責任境界・surface topology決定**（2026-07-21）。platform受入とrenderer採否はG0-9実機spike待ち。

Motoliiの製品UIは、Reactとnativeのどちらか一方へ全面統一しない。ReactはDOMが強い領域、
native Rust/wgpuは高頻度GPU workspaceを所有する。この分割は採択済みであり、G0-9が今後比較するのは
責任境界そのものではなく、WebViewとnative surfaceを安全に同居させる実装方式である。

## 1. 所有境界

```text
Native coordinator
├─ React / WebView chrome
│  ├─ Asset Browser
│  ├─ Inspector / parameters / forms
│  ├─ panel / toolbar / dialog / search / settings
│  └─ Hostとcommunityのversioned UI kit
├─ native wgpu Stage
│  ├─ canonical display texture
│  └─ handle / gizmo / roto presentation overlay
└─ native wgpu Timeline
   ├─ ruler / track header / lanes / clips / keys / playhead
   └─ selection / marquee / graph / transient preview
```

React採択の理由は、CSS layout、form、text input、IME、a11y、component資産、hot reload、
Storybook/Playwright、LLM生成容易性、community作者の入口である。Canvasやbrowser WebGPUを
React componentへ包めること自体はReact所有の理由にしない。

StageとTimelineは、一つのzoom/scroll/focus/gestureへ高頻度同期する要素を領域内で分割しない。
特にtrack headerだけをReact、key surfaceだけをnativeにする構成は採らない。Reactは外側のtoolbar、
menu、popover、parameter編集を所有する。

Web所有panel内の小さなvisualizationは、DOMのform/a11y/component資産が主体で、native側へ
semantic stateやinteraction stateを複製しない場合に限ってCanvasを使える。Stage、Timeline、roto、
大量object/keyの直接操作面をこの例外へ入れない。

## 2. native surfaceは汎用UI toolkitを再実装しない

native側で自作するのはMotolii固有のdomain surfaceであり、flex、form、text editor、dialog、theme、
community runtimeを備えた汎用widget frameworkではない。

```text
platform input adapter
        ↓
NormalizedInput
        ↓
headless interaction kernel
├─ pointer lifecycle / capture / drag threshold
├─ pan / zoom / marquee / multi-selection
├─ snap候補照会 / edge scroll / cancel / focus loss
└─ deterministic gesture state machine
        ↓
toolkit非依存projection / transient preview
        ↓ release
D2 command / single writer / Undo 1回
        ↓
direct wgpu primitive batch + 必要箇所だけVello
```

headless libraryや既存実装は、新規自作より先に検索して使う。ただし採用単位は次を全て満たすものに限る。

- window、event loop、renderer、scene graphを所有しない
- 独自Document、selection正本、history、Undoを持ち込まない
- pointer cancel、focus loss、capture喪失を入力として扱える
- 固定入力列をwindow/GPUなしでdeterministicに再生できる
- library固有型をDocument、domain公開API、plugin契約へ出さない
- adapterで交換可能であり、React/nativeの状態二重所有を要求しない

外部へ委ねられるのはpointer lifecycle、drag開始距離、viewport操作、geometry、text shaping/layout、
path rasterization、a11y基盤である。Motoliiが所有するのはclip/key/objectの意味、RationalTime、snap優先度、
Edit Space、selection、transient preview、D2 commit、Undo/Cancelである。

新規nativeコードがclip/key/snap等のdomain語彙ではなく、汎用widget/layout/style/theme語彙を公開し始めたら
UI framework再発明として停止する。

## 3. rendererの現在位置

- 大量rect/line/key/gizmo: core-owned device上のdirect wgpu primitive batchが第一候補
- 複雑path、curve、roto、採択済みglyph描画: Velloを局所rendererとして再利用
- font discovery/shaping: 採択済みfontique + harfrust。単純layoutで不足する実例が出た時だけParleyを比較
- accessibility: AccessKitを基盤にし、全keyをnode化しないbounded semantic projectionを作る
- egui: 成立済みbaseline/debug候補として、同条件windowed比較が終わるまで削除しない

direct wgpuは採択候補であって、headless benchmarkだけで製品renderer確定とはしない。Velloの「局所」は
呼出頻度でなく、所有語彙、描画面積、primitive数、allocation、GPU時間で判定する。毎frame呼ばれても
scene/input/Documentを所有せず予算内なら境界違反ではない。

## 4. surface topologyとcoordinator境界

React/native間を流せるのは、typed bounds、domain intent、read-only semantic projection、focus移譲、
pointer capture境界、DPI epoch、bounded a11y projectionである。DOM event、CSS px、Canvas scene、
toolkit object、raw GPU handleをDocumentやplugin契約へ流さない。

通常windowはcore-owned device/queueへ接続した**トップレベル`wgpu::Surface`を1枚だけ**持つ。StageとTimelineは
同じsurface texture、同じframe submission内のviewport/scissor rectangleとして描く。React chromeはdock/tab
stackごとのopaque child WebView rectangleとしてOS compositorが上へ合成する。1 panelごとにWebViewを増やさず、
同じversioned React bundleをrole付きで起動する。native rectangleをまたぐDOM popupだけ、必要寸法の一時opaque
child WebViewを使える。

coordinatorは単調増加`layout_epoch`ごとにchild WebViewのlogical bounds、native viewportのphysical bounds、
hit-test transform、a11y rectangleを一括反映する。古いepochを部分反映せず、CSS px、AppKit point、Win32 pixel、
DPIをDocument、D2、plugin契約へ流さない。

Stage/Timeline別の複数surface、全画面transparent WebView、browser/native GPU texture共有、Windows
CompositionControllerは正規経路にしない。通常のwindowed child WebViewがWindows実機で修正不能な失敗を再現した
時だけCompositionController、別window、最後にCEF OSRの順で再審判する。証拠と停止線は
[surface topology決定](reviews/2026-07-21-ui-surface-topology-decision.md)を正とする。

## 5. 決定済みと未決の境界

### 決定済み

- React/WebView chrome + native Stage/Timelineという責任分担
- ReactはDOMの優位性がある領域へ使い、高密度Canvas workspaceのownerにしない
- Stage/Timelineのlayout、hit-test、interaction modelはtoolkit/renderer非依存に置く
- native interactionはheadless部品を優先し、Motolii固有意味だけを自作する
- React/nativeの両側へselection、snap、Undo、semantic stateを二重所有させない
- Hostとcommunityは同じversioned React UI kitを使う長期原則
- 1 top-level wgpu Surface + Stage/Timeline 2 viewport + opaque child WebView islandsという通常window topology

### G0-9実機spikeまで未決

- macOS/Windowsのfocus、DPI、resize、z-order、pointer capture、surface/device lost、a11y tree接合の受入
- direct wgpu枝とdirect wgpu + Vello局所pass枝の製品採択
- egui製品shellの撤去時期
- community WebViewのsandbox、権限、互換、配布という公開契約
- Linuxでsystem WebViewを使うかCEF比較へ進むか

未決項目を理由に決定済み責任境界を再び「全面egui対全面React」の比較へ戻さない。逆に責任境界の決定を、
未合格のWebView/native合成やplugin公開契約の実装許可として扱わない。

## 6. 証拠と後続

- [React / WebView再選定](reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md)
- [native surface renderer再選定](reviews/2026-07-21-native-surface-renderer-reselection.md)
- [拡張サーチ](reviews/2026-07-21-native-surface-renderer-extended-search.md)
- [Fable反対側レビュー](reviews/2026-07-21-native-surface-renderer-counter-review.md)
- [Fable伸長レビュー](reviews/2026-07-21-native-surface-renderer-growth-review.md)
- [surface topology決定](reviews/2026-07-21-ui-surface-topology-decision.md)
- [G0-9部分スパイク](spikes/g0-9-ui-runtime.md)と[確認点マトリクス](spikes/g0-9-verification-matrix.md)

実装合否と停止線は[M3仕様 G0-9/U3a](specs/M3-ui-integration.md)を正本とする。本書は設計責任を固定するが、
新しい公開API、Document field、永続layout形式、plugin GPU/UI契約を許可しない。
