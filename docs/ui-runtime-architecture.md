# UI runtime責任境界

状態: **React Native + Rust/rust-skia + wgpuへ再基線化済み / 製品移行は未完了**（2026-08-07）

正本決定: [M3 React Native + Rust/Skia UI runtime再基線決定](reviews/2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md)

MotoliiはUIを一つのtoolkitへ統一しない。通常UIをReact Native、高密度canvasと直接操作をRust/rust-skia、Stageのbase previewとGPU処理をwgpu、編集・再生・保存をRust coreが所有する。

旧React/WebView islands + 1 top-level wgpu Surface + direct wgpu/Vello UIは新規製品実装の標準ではない。既存実装は移行oracleとして保持し、新routeが同じ利用者outcomeを閉じた後にだけretireする。

## 1. runtime topology

```text
React Native application/window
├─ product shell / dock / tabs / toolbar / menus
├─ Browser / Inspector / settings / forms / text / dialogs
├─ product-owned normal-density and custom panels
├─ Native Timeline Component
│  └─ Rust interaction + rust-skia canvas
├─ Native Curve Editor Component
│  └─ Rust interaction + rust-skia canvas
└─ Native Stage Component
   ├─ wgpu base preview texture
   ├─ rust-skia transparent overlay, dirty update
   └─ wgpu composite / present

Rust Host / core
├─ revisioned read-only projection
├─ transient interaction/session state
├─ D2 single writer / Undo / journal
├─ playback / media / render / resource
└─ platform adapters
   ├─ macOS AppKit + Fabric native view
   └─ Windows RNW Fabric Component View + Composition
```

OS window数、dock、detachはRN shellのlayout責任であり、Documentへ保存しない。native componentは割り当てられたrectangle、scale、visibility、focus、lifecycleを受ける。surface数を製品意味にせず、componentごとのplatform adapterへ閉じる。

## 2. React Nativeが所有するもの

- application shell、dock、tabs、split、toolbar、menu、popover、dialog
- Asset／Create／Effects Browser
- Inspector、parameter form、numeric/text input、search、settings
- Stage／Timeline／Curveの外側chromeとstatus
- 通常密度のproduct-owned custom panel
- IME、focus traversal、standard controls、semantic labelsの第一責任
- React component test、visual fixture、開発時hot reload

既存React mockは画面concept、情報階層、component分割、文言、状態表現、test oracleとして移す。DOM、CSS、HTML event、WebView bridgeをそのまま製品契約にはしない。React web componentをRNへ移す際は、意味と操作を保ち、RN primitives、StyleSheet、native component props/eventsへ変換する。

RNはDocument、selection、Undo、playhead、terminal gestureの正本を持たない。local hover、focus-visible、open/closed、未確定form bufferはpresentation stateとして持てる。

## 3. native canvasが所有するもの

### Timeline

track header、ruler、lane、clip、key、playhead、selection、marquee、scroll、zoom、drag、trim、snap、edge scrollを一つのcanvas interactionとして所有する。clipやkeyごとにRN componentを生成しない。短いclipはsemantic zoomでlabel、thumbnail、waveform、handleを段階的に省略する。

### Curve Editor

curve、key、tangent、grid、marquee、pan、zoom、preset previewをrust-skiaで描く。Timelineとviewport transform、pointer lifecycle、selection gestureのheadless語彙を共有できるが、巨大な汎用widget frameworkへ統合しない。

### Stage

base previewはwgpuが所有する。rust-skia overlayはgrid、safe area、selection bounds、path、gizmo、snap補助だけを描く。overlayはdirty時だけraster/uploadし、CPU readbackなしでwgpu previewとcomposeする。

100〜500 gizmoは上限stressであり、通常表示目標ではない。visible、selected、group root、semantic importanceで情報を間引く。inactive objectはboundsだけ、active selectionはfull gizmoを基本とする。

## 4. interactionとsingle writer

```text
platform pointer/key/focus/lifecycle
        ↓
normalized native component input
        ↓
headless Rust gesture state
        ↓
transient preview + dirty canvas
        ↓ release
typed semantic intent / existing D2 command
        ↓
single writer / journal / one Undo
        ↓
new revisioned snapshot
        ↓
RN + Timeline + Curve + Stage reprojection
```

- pointer moveをframeごとにJSへ往復させない。
- drag中のDocument writeは0、terminal commitは高々1回。
- cancel、focus loss、capture loss、stale revision、invalid targetは0 write。
- group selection／group layerも一つのgestureからbounded command setまたは既存macroへ変換し、各objectが独立ownerにならない。
- Inspector編集も同じtyped intent → D2 → snapshot経路を使う。canvasだけを特別なwriterにしない。

## 5. RN/native boundary

境界で許可するもの:

- component identity、logical bounds、scale factor、visibility、focus request
- revision／generation付きread-only snapshotまたはbounded projection
- viewport、tool mode、selection summary、quality/status
- terminal typed intent、cancel、diagnostic counters
- visible／selected／focused中心のbounded accessibility projection

境界へ出さないもの:

- DOM event、CSS pixel、RN internal node、Skia scene、wgpu handle
- per-object component同期、per-pointer-move bridge message
- Document mutable reference、Undo stack、surface別selection owner
- platform固有handleを含むpublic plugin API

props/events/C ABIはmacOS／Windowsで同じsemantic contractにする。platform adapterだけがAppKit、C++/WinRT、CAMetalLayer、Composition surfaceを知る。

### 4.1 built-in WebView Hostの再入場条件（履歴互換anchor）

旧文書からの参照互換のため見出しを保持する。built-in WebView Hostは2026-08-07以降の標準product runtimeではない。旧routeのoffline bundle、closed codec、epoch、bounded inbox、fail-closed lifecycleは回帰oracleとして残す。将来限定WebViewが必要になった場合は、対象surface、RNでは成立しない理由、process／focus／DnD／a11y負債、security contractを独立decisionで閉じるまで再入場させない。

## 6. GPUとrenderer

- rust-skiaをTimeline、Curve Editor、Stage overlayの既定2D rendererとする。
- wgpuをStage preview、media/composite、final Stage compositionのownerとする。
- 同じStage component内でdevice、queue、surface、resource lifecycle ownerを一意にする。
- rust-skiaはまずCPU raster + dirty uploadを採用する。実製品計測で不足した面だけGPU-backed Skiaを比較する。
- direct wgpu primitive UIとVelloは新規標準ではない。既存codeはbenchmark、fixture、visual oracle、特殊render資産として保持する。
- renderer変更はDocument、semantic command、interaction fixtureを変更理由にしない。

## 7. focus、IME、DnD、DPI、a11y

- text compositionはRN standard TextInputをownerとし、composition中shortcutを発火しない。
- native canvasへfocusを移してもRN form bufferを破棄しない。
- native dragはcomponent外release、window focus loss、unmountをterminal/cancelとして受ける。
- external file DnDと内部object dragを別contractにする。
- logical coordinateとphysical pixelを明示し、scale変更時はresourceとhit-testを同じepochで更新する。
- RN standard controlsはRN/OS semanticsを使う。native canvasは全要素を無制限にAX node化せず、visible、selected、focused、navigation targetをbounded projectionする。
- IME composition、VoiceOver/NVDA、outside-window drag、DPI、device lostは実OS gateで判定し、unit testへ代用しない。

## 8. product-owned panelとplugin UI

RN採択により、bundled first-partyのproduct-owned custom panelをReact componentとして追加できる。ただしthird-party plugin custom UIの公開は別問題である。

- NodeDesc等からのHost-generated panelは安全なfallbackとして維持する。
- product-owned RN componentはbundled Host moduleであり、untrusted plugin ABIではない。
- third-party code loading、sandbox、permission、version、crash isolation、distributionはG0-3 / GAP-13で別途決定する。
- RN採択を理由に任意JS bundle、network、eval、同process権限をpluginへ開かない。

## 9. platform strategy

macOSで操作体系を先に成立させる。共通Rust renderer coreはWindows targetでcompile済みであるため、Windows実機未検証をmacOS implementationの停止条件にはしない。

一方で次を守る。

- Metal/AppKit固有型を共通interaction、projection、C ABIへ漏らさない。
- WindowsはRNW Fabric Component View + Microsoft.UI.Composition/DX12 adapterとして接続する。
- 最初の製品vertical slice後にWindowsでrender、resize、DPI、outside release、focus、remount、device lostを実行する。
- Windows gate未通過の状態をcross-platform完成またはDistribution Readyと呼ばない。

## 10. migration

1. 既存routeをfreezeし、意味fixtureと製品outcome oracleを保全する。
2. RN shell + Rust Host lifecycleを製品routeへ置く。
3. VS-1をRN shellで再閉鎖する。
4. Stage、Timeline、Curve Editorをoutcome単位でcutoverする。
5. Inspector／Browser等のReact conceptをRN product componentへ移す。
6. 新routeでautomated、macOS product、human gateを通した面だけ旧presentationをretireする。
7. Windows product gateを通してからcross-platform UI基盤を完了扱いにする。

全面書き直し、旧codeの一括削除、二つのUI基盤の恒久並走は行わない。移行中の二routeは比較のためだけに存在し、同じ製品windowでsemantic ownerを二重化しない。

## 11. status

決定済み:

- RN shell + rust-skia Timeline/Curve + wgpu/rust-skia Stage
- Rust core/D2/snapshotの継続利用
- macOS先行、Windows early external gate
- direct-wgpu/Vello/eguiの新規product UI凍結とoracle保持

未完了:

- Motolii製品repoへのRN runtime導入
- RN shellでのVS-1再閉鎖
- product Stage／Timeline／Curve移行
- full IME composition、VoiceOver、NVDA、device lost
- Windows native component実機受入
- third-party custom panel公開契約
