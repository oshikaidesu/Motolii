# UI surface topology決定（2026-07-21）

状態: **topology決定 / platform受入試験継続**。

## 1. 決定

Motolii v1の通常windowは、**core-owned device/queueに接続したトップレベル`wgpu::Surface`を1枚だけ**持つ。
native Stageとnative Timelineは別swapchainへ分けず、同じsurface texture、同じframe submission内の
viewport/scissor rectangleとして描く。React chromeはsurface textureへ混ぜず、必要なdock/tab stackごとの
**opaque child WebView rectangle**をOS compositorが上へ合成する。

```text
native host window
├─ one wgpu Surface / one acquire-present loop
│  ├─ Stage viewport + presentation overlay
│  └─ Timeline viewport + primitive batch / local Vello pass
├─ opaque host WebView islands
│  ├─ top toolbar / dialog host
│  ├─ Browser stack
│  └─ Inspector / parameter / community-panel stack
└─ temporary opaque popup WebView rectangle when a DOM popup crosses a native region
```

WebViewは1 panelごとでなくdock/tab stackごとを基本とし、同じversioned React bundleをpanel role付きで起動する。
community panelは同じUI kitを使うが、権限分離が必要な時は別WebView realmとtyped brokerへ置く。通常のpopoverは
同じWebView内、native rectangleをまたぐpopupだけを必要寸法の一時child WebViewへ出す。透明な全画面WebView、
WebViewの穴、browser/native GPU texture共有を前提にしない。

## 2. platformの具体

### macOS

wgpuはAppKit viewから`CAMetalLayer`を作る。wryは`WKWebView`を同じhostのchild `NSView`として
`addSubview`する。[wgpu Metal実装（固定commit）](https://github.com/gfx-rs/wgpu/blob/0eb5b623df8f2721baa040ef02442bd0fa5800aa/wgpu-hal/src/metal/mod.rs#L160-L188)、
[wry WKWebView実装（固定commit）](https://github.com/tauri-apps/wry/blob/6b61fcd58b699323ed16956648c3cf566c5da535/src/wkwebview/mod.rs)、
[Apple `WKWebView`](https://developer.apple.com/documentation/webkit/wkwebview)、
[Apple `NSView.subviews`](https://developer.apple.com/documentation/appkit/nsview/subviews)、
[Apple `CAMetalLayer`](https://developer.apple.com/documentation/quartzcore/cametallayer)

### Windows

wryの通常経路は親windowの下に`WS_CHILD | WS_CLIPCHILDREN`のcontainer HWNDを作り、標準
WebView2 Controllerへboundsを渡す。
[wry WebView2実装（固定commit）](https://github.com/tauri-apps/wry/blob/6b61fcd58b699323ed16956648c3cf566c5da535/src/webview2/mod.rs)

Microsoftはwindowed hostingを、多くのappが最初に選ぶ方式として位置付ける。OSがinput、focus、tab、a11yの
多くを処理する。CompositionControllerはcustom visualへ描く代わりにinput、drag、focus、a11y接合の追加責任を
hostへ戻すため、通常経路にはしない。
[WebView2 windowed / visual hosting](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/windowed-vs-visual-hosting)、
[WebView2 API overview](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/overview-features-apis)

## 3. 公式sampleとmacOS実機証拠

wryの[公式`examples/wgpu.rs`（固定commit）](https://github.com/tauri-apps/wry/blob/6b61fcd58b699323ed16956648c3cf566c5da535/examples/wgpu.rs)
は、トップレベルwinit windowから1枚のwgpu surfaceを作り、`build_as_child`でWebViewを置く。
2026-07-21にApple M4 / macOSでこのsampleをbuildし、Computer Useで次を確認した。

1. `CAMetalLayer`の三角形とchild `WKWebView`が同じwindowへ表示された
2. window zoom後もsurfaceとWebViewが同時に表示され、AX treeではHTML contentがwindowの子に現れた
3. sampleを一時作業領域だけでopaque WebView + 左chrome + 右上Stage viewport + 右下Timeline viewportへ
   最小改変し、1 surfaceの2 viewportとWebView rectangleがCPU pixel bridgeなしで同時表示された
4. WebViewのnative text fieldへfocusを移し、Computer Useのtyped input後も2 native viewportが表示を維持した
5. window zoom後にWebView boundsと2 viewportが追従した

再現に使った公式sampleはwgpu 23依存なので、これは**OS合成topologyの証拠**でありwgpu 29製品統合、性能、
100回resize、IME、VoiceOverの合格証拠ではない。一時sampleとapp bundleはrepositoryへ入れていない。

## 4. boundsと入力の所有

Motolii layout intentは相対share、role、visibilityだけを持つ。event-loop上のcoordinatorがwindowのlogical
unit、physical pixel、scale factorを受け、単調増加`layout_epoch`ごとに次を一括反映する。

- child WebViewのlogical bounds
- Stage/Timelineのphysical viewport/scissor
- native hit-test transform
- bounded a11y treeのrectangle

古いepochを部分反映せず、0×0領域はacquire/present対象にしない。CSS px、AppKit point、Win32 pixel、DPIを
Document、D2、plugin契約へ流さない。OSはWebView rectangle内のinputをWebViewへ、露出したnative領域をhostへ
配送する。境界を越える継続dragはplatform captureを使うが、macOS/Windowsの実操作合格までは製品統合を閉じない。

## 5. 棄却した通常経路

| 案 | 処分 | 理由 |
|---|---|---|
| Stage/Timelineごとに`wgpu::Surface` | **REJECT v1 mainline** | acquire/present、surface lost、frame pacing、DPI同期を増やす。1 texture内viewportで要件を満たせる |
| 全画面透明WebViewの穴からnativeを見せる | **REJECT** | platform別透明化、private API、hit-test/z-orderへ製品成立を依存させる |
| browser WebGPUへnative textureを渡す | **REJECT** | system WebViewに標準共有経路がなく、別device/CPU bridgeを招く |
| WebView2 CompositionController | **FALLBACK** | opaque非重複rectに不要。windowed hostingで修正不能なWindows実機失敗が出た時だけ比較 |
| CEF OSR + shared texture | **LAST FALLBACK** | Graphiteに先例はあるがruntime、security、配布負担が大きい。system WebViewの失敗証拠前に採らない |
| 全native / eguiへ戻す | **REJECT as default** | React chrome/community入口という決定済み責任境界をsurface都合で撤回しない |

## 6. 残る受入ゲート

topologyは決定したが、G0-9の製品統合は次が揃うまで停止する。

- macOS: resize 100回、minimize/restore、fullscreen、異DPI monitor、native dragのWebView境界capture、
  Browser WebViewからnative Stageへのdrag token、複数WebView間focus traversal、日本語IME、VoiceOver、
  Web content process再生成
- Windows 10/11: 標準windowed WebView2、per-monitor DPI、resize/minimize、MS-IME、NVDA、pen、
  process failure、offline runtime
- 共通: 100,000 keyのwindowed fixture、readback 0、frame内GPU resource生成0、surface/device lost、
  latest layout epoch、複数dock WebViewのHMR/state投影、popup WebView、community realm crash/loop
- renderer: direct wgpuのみとdirect wgpu + Vello局所passを同条件egui baselineへ比較

通常windowed hostingでz-order、focus、DPI、captureの修正不能なplatform failureが再現した場合だけ、
platform非対称のCompositionController、別window、最後にCEF OSRの順で再審判する。

## 7. 既知issueの読み直し

- [tao #208](https://github.com/tauri-apps/tao/issues/208)のmacOS初回keyboard focus問題は2022-10-28に
  wry #740で修正済み。現行リスクの根拠にはせず、focus回帰fixtureとして使う
- [wry #1331](https://github.com/tauri-apps/wry/issues/1331)はWindowsの**透明overlap**問題で、
  winit既定`WS_CLIPCHILDREN`が原因だった。現行公式sampleは透明overlap時だけclippingを無効化する。
  Motoliiのopaque非重複rectは通常clippingを維持する
- wry READMEは`build_as_child`をmacOS、Windows、Linux X11で支持する。Waylandは同じ経路でないため、
  Linux対応はv1 macOS/Windows topologyの合格から推論しない
