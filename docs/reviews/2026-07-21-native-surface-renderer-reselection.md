# React複合下のnative surface renderer再選定（2026-07-21）

状態: **比較中**。React/WebView複合を第一候補として維持し、React資産はAsset Browser、Inspector、
parameter/form、panel、toolbar、検索、設定、community UIへ使う。高頻度で同期して動くStageとTimelineは
native所有とし、そこでegui widgetを使う前提を外す。製品native rendererの第一候補は既存deviceを使う
direct wgpu、複雑path/textだけ採択済みVelloを局所利用する構成である。

本書はegui撤去、WebView採択、event-loop実装、公開API、Document、plugin GPU契約を許可しない。
先に同一fixtureのwindowed spikeで反証する。egui以外の追加候補・先例・支援基盤の続編サーチは
[拡張サーチ](2026-07-21-native-surface-renderer-extended-search.md)を参照。

## 1. 問いの修正

比較する問いは「eguiかReactか」ではない。

```text
Native coordinator
├─ React / WebView
│  ├─ Asset Browser
│  ├─ Inspector / parameters / forms
│  ├─ panel / toolbar / dialog / search
│  └─ Hostとcommunityのversioned UI kit
├─ native wgpu Stage
│  ├─ canonical display texture
│  └─ handle / gizmo / roto overlay
└─ native wgpu Timeline
   ├─ ruler / lanes / clips / keys / playhead
   └─ selection / marquee / graph / transient preview
```

比較対象はnative領域の実装方式である。

1. direct wgpuのdomain-specific retained projection
2. direct wgpu + Vello等の局所2D renderer
3. egui/GPUI/Slint/Iced/Qt Quick等の別UI frameworkをnative領域にも置く方式

Timelineのruler、lane、clip/key、playhead、selection、graphは同じzoom/scroll/focus/gestureへ同期するため
一つのnative interaction surfaceとして扱う。Reactとの境界をtrack headerとkey canvasの間へ置かず、
Reactは外側のtoolbar、menu、popover、parameter編集を所有する。

## 2. Motoliiの現行事実

### 2.1 direct wgpu Timelineは機構成立済み

`spikes/timeline-bench`はwgpu 29でclip 1,000 + keyframe 100,000をCPU cullし、visible itemを
storage bufferへ詰めて単一render passでinstance描画する。2026-07-21のApple M4 / Metal再実行は
warmup 60、180 frameで次だった。

| 項目 | 実測 |
|---|---:|
| 全clip / key | 1,000 / 100,000 |
| 最終visible clip / key | 362 / 36,002 |
| median | 2.166 ms |
| p95 | 6.252 ms |
| 判定 | 60 fps frame budget内 |

ただしこれはheadless render-to-textureで、各frameの`device.poll(Wait)`、CPU cull/upload、GPU完了を含む一方、
window present、text、実input、D2、WebView同居、a11yは含まない。さらにspikeはbind groupをframeごとに作るため、
製品の「ループ内GPU resource生成禁止」を満たさない。性能可能性の証拠であって製品設計の合格ではない。

### 2.2 Velloは新候補ではなく採択済み局所renderer

[Vello 0.9.0固定tag `875f324`](https://github.com/linebender/vello/tree/875f324f21da93019cae9e8e61d4abfd69893206)
はwgpu 29.0.4と同じdeviceで動くことを`spikes/vello-eval`で再確認した。

| 項目 | 2026-07-21再実行 |
|---|---:|
| `Renderer::new` | 1.025 s |
| procedural shape + usvg最小path render/readback | 14.513 ms |
| alpha | straight |
| device | Apple M4 / Metal、wgpu 29.0.4 |

このreadbackはalpha審判用だけで製品経路には入れない。Rendererは長寿命、straight→premulは既存の単一境界、
`vello_svg`を増やさず既存usvg adapterを使う、というS3条件を維持する。Vello公式も大規模2D sceneを
wgpu computeで描くrendererとする一方、alpha状態でGPU memory allocationとglyph caching等を未完と明記する。
[Vello公式README](https://github.com/linebender/vello#readme)

Timelineの大量rect/line/keyはdirect wgpu primitive batchを第一選択にし、Velloはroto/path、curve、
採択済み`draw_glyphs`等、複雑vectorが必要な箇所だけ同じwgpu device上で使う。Velloを第二のscene graph、
input system、Document正本にはしない。

## 3. 追加候補の一次資料監査

| 候補 | 一次資料上の事実 | Motoliiでの処分 |
|---|---|---|
| direct wgpu primitive batch | wgpu 29はpre-record可能な[`RenderBundle`](https://docs.rs/wgpu/29.0.3/wgpu/struct.RenderBundle.html)とindirect drawを持つ。Motoliiの同一device、instance buffer、固定pipeline実績がある | **FIRST CANDIDATE**。Timeline/Stageのowner。product spikeでresource再利用、present、inputを閉じる |
| Vello 0.9 | wgpu上のGPU compute 2D renderer。shape/path/image/textをsceneへ記述できるが公式はalphaを明記 | **ADJUNCT / 既存採択**。複雑path/textだけ。Timeline全primitiveやinputを委ねない |
| GPUI | 公式READMEはhybrid immediate/retained GPU UI、Windows/macOS/Linux backend、custom low-level elementを提供する一方、pre-1.0でbreaking changeとZed sourceが主要学習資料であると明記。[固定監査commit `e8bfce7`](https://github.com/zed-industries/zed/tree/e8bfce7614b88f2ec32bd8fae4d8ae08e6f3834d/crates/gpui) | **PATTERN**。Zedの少数primitive GPU batch、large-list、input testを学ぶ。Reactと競合するwindow/state/layout/input frameworkを製品依存にしない |
| Slint 1.17 | [`unstable-wgpu-29`](https://docs.slint.dev/latest/docs/rust/slint/docs/cargo_features/#unstable-wgpu-29)で既存device統合可能だが、minorで変更/削除し得る非安定API。既存S1の日本語/CJK、custom drawing、LLM反復、community入口の反証も残る | **REJECT for product path**。既存歴史証拠を保持し、React複合へ第二UI frameworkを戻さない |
| Iced | 公式にwgpu custom shader widgetを持つ。[Iced shader module](https://docs.iced.rs/iced_widget/shader/index.html) | **REJECT for product path**。direct wgpuを別widget/state/event frameworkで包む追加価値が未証明 |
| Qt Quick | scene graphへunder/over、texture、inlineの3方式でcustom renderを入れられるが、inline QRhi例は`Qt::GuiPrivate`と互換保証の弱いprivate headerを要求する。[Qt公式](https://doc.qt.io/qt-6/qtquick-scenegraph-customrendernode-example.html) | **REJECT**。C++/QRhi/Qt render loopという第二graphics/UI stackを増やす |
| Skia | GraphiteはDawn/Metal/Vulkan等、Ganeshも独自backend contextを持つ。[Skia GPU API](https://api.skia.org/namespaceskgpu.html) | **REJECT**。既存wgpu/Velloと重複するrenderer、cache、alpha、backend lifetimeを持ち込む |
| lyon | GPU backend非依存のCPU path tessellationでtriangle列を作る。[固定source `8071ec0`](https://github.com/nical/lyon/tree/8071ec066c610b006e58086fea30cd96d4cef153) | **PATTERN / fallback candidate**。Velloで閉じない特殊pathだけ個別比較し、Timeline engineやscene graphにはしない |
| glyphon / cosmic-text | wgpu textとshapingの既成部品だが、Motoliiはfontique + harfrust + Vello `draw_glyphs`を既決 | **REJECT as duplicate stack**。Timelineだけ別font discovery/shaping/atlasを持たない |

Zedの公式解説も、任意2D renderer全体ではなくrect、shadow、text、icon、imageという必要primitiveを
data-drivenにGPUへ送る方式を採る。[Zed GPU UI設計](https://zed.dev/blog/videogame)
これはTimelineのdomain-specific batchを支持する先例だが、GPUI依存を要求する証拠ではない。

## 4. 暫定アーキテクチャ

```text
React layout computes non-overlapping rectangles
          │ typed bounds / intent / semantic projection
          ▼
native coordinator
  ├─ WebView child surfaces
  ├─ Stage wgpu surface ─ canonical texture + presentation overlay
  └─ Timeline wgpu surface ─ primitive batch + local Vello passes
              │
              ├─ CPU retained layout / cull / hit-test
              ├─ Transient preview (semantic write 0)
              └─ D2 commit once on release
```

native surfaceが2枚必要か、同じsurface内の2 viewportにするかは未決である。WebView rectangleの間に
native領域が分かれる通常layoutでは2 child surfaceが自然だが、device/queue共有、surface lost、DPI、
z-order、focus、別window化をmacOS/Windows実機で測るまで固定しない。

eguiは現行成立済みbaseline、debug/dev UI候補として残す。native Stage/Timelineの描画、layout、inputを
egui callback/widgetへ新規実装せず、direct wgpu spikeが不合格だった場合だけ不足機能を具体的に比較へ戻す。

## 5. 次のwindowed spike

同じfixtureと操作列で次の2枝だけを比較する。

- A: direct wgpu primitive batchのみ
- B: A + Vello path/text局所pass

egui、Canvas/browser WebGPU、Pixi/Konva/Threeの既存結果はbaseline/oracleで、第三の製品枝として増やさない。

### 自動合格条件

- 1,000 clip + 100,000 key、固定visible range、同じstable ID/selection/playhead
- pan/zoom、marquee、multi-select drag、snap、pointer capture、Cancelをactual pointerで再生
- move中Document/revision/Undo変更0、releaseでD2/Undo 1回、Escape/focus lossで確定変更0
- pipeline/buffer/bind group/texture/font atlasをframe loop内で生成しない
- UI threadの`poll(Wait)`、texture/buffer readback、worker join 0
- React layout resizeとnative bounds更新後もDPI、hit target、time mapping、selection不変
- Stage/Timelineが同じcore-owned device/queueを共有し、surface lostを個別復旧、device lostを一元診断
- Velloを使う枝はRenderer長寿命、straight→premul単一境界、canonical/export画素非汚染
- Japanese/CJK label、数値、icon、grayscale selectionを固定captureで確認

### 実機合格条件

- macOS WKWebView + CAMetalLayer、Windows WebView2 + native wgpu child surfaceの非重複合成
- resize、minimize/restore、DPI monitor移動、fullscreen/別window、focus traversal
- native Timeline drag中にReact toolbar/Inspectorへfocusが飛ばず、終了後に明示操作で移動できる
- VoiceOver/NVDAは全keyをnode化せず、選択・playhead・tool・現在値をbounded proxyで操作可能
- WebView content process停止後もnative preview/Timelineが停止せず、再生成後に同じsnapshotへ戻る

## 6. Fableへ渡す反対側レビュー質問

1. direct wgpuに寄せることで実質的な独自UI frameworkを再発明していないか。Motolii固有でない部分は何か。
2. Stage/Timelineを2 surfaceに分ける場合、device/queue共有とplatform compositorの最小安全構成は何か。
3. Timeline text/pathへVelloを局所使用する境界は、alpha、atlas、cache、surface lostで本当に一元化できるか。
4. React/native間のbounds、focus、pointer、a11y同期で、typed bridgeに不足する最小eventは何か。
5. egui baselineがdirect wgpu枝より優れる再現可能なfixtureはあるか。あれば採否を戻す条件は何か。
6. macOS/Windowsのどちらかでchild surfaceが成立しない時、CPU bridgeや透明WebViewへ逃げずに残る構成は何か。

Fableの回答は助言であり、Document/API/plugin契約を発明しない。採否は上記Motolii fixtureと実機証拠で行う。

## 7. 停止線

- direct wgpuの名で汎用widget tree、flex、form、text editor、community UI runtimeを再実装しない。
- Reactとnativeの両側へTimeline semantic state、selection、Undo、snapを二重所有させない。
- native high-frequency surfaceへThree/Pixi/Konvaのscene/camera/inputを加えない。
- Vello/GPUI/Qt/Skia/eguiの内部型をDocument、domain公開API、plugin契約へ出さない。
- headless 6.252 msからwindowed 60/120 fps、input latency、Windows性能を合格扱いしない。
- product loopへspikeのframe内bind group生成、`poll(Wait)`、readbackを持ち込まない。
