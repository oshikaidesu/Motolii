# G0-9 UI runtime確認点マトリクス（2026-07-21）

状態: **全確認点を経路へ割当済み／採否は継続**。`PASS`は記載した環境とfixtureだけ、
`PARTIAL`は一部の機構だけ、`PHYSICAL`は対象実機なしに合格へ上げない項目を表す。
調査だけの項目を実測済みと扱わない。

## 1. 現在の判定

| 確認点 | 状態 | 証拠／次の審判 |
|---|---|---|
| Browser 10,000 item | **PASS / automated** | 24〜30 DOM row、stable ID選択。可変高tree/gridは未証明 |
| Timeline 100,000 key描画 | **PASS / automated** | Canvas 2D、browser WebGPU micro-probe、native wgpu基準。renderer間の速度比較ではない |
| 10,000 key group drag | **PASS / automated** | PixiJS/Konva adapter。move中semantic write 0、Cancel復元、release callback 1 |
| actual mouse drag / snap / canvas外移動 | **PARTIAL / automated** | Playwrightの実mouse inputでKonva group drag、10 CSS px `dragBoundFunc`、canvas外移動後Escapeを確認。Motolii RationalTime snapと実D2 Undoは未接続 |
| pointer capture | **PASS / primitive** | DOM overlayで標準`setPointerCapture`を使い、surface外moveとrelease後capture解放を確認。`pointercancel`/pen/touch実機は未証明 |
| marquee | **PASS / adapter** | Konva `Rect` + `Konva.Util.haveIntersection`で選択集合を生成。reverse marquee、edge-panは未証明 |
| 2D object handle | **PASS / prior-art adapter** | Konva Transformerで実mouseのmove/scale/rotate、2 object選択、Escape取消、zoom後も14 CSS px表示/30 CSS px hit target、固定3操作のDOM proxyを確認。製品Stage ownerはnative wgpuへ決定し、Web runtime選定の合格点から外す |
| 3D object gizmo | **PARTIAL / prior-art adapter** | Three.js TransformControlsで実mouseのtranslate/scale/rotate、world/local、snap設定、drag中OrbitControls排他、Escape取消を確認。製品Stage ownerはnative wgpuで、Three.jsを製品renderer/runtimeとはしない |
| M5 Scale / Depth Move分離 | **NOT VALIDATED** | 汎用3D translate/scale成立はP2U合格ではない。Scaleは`scale.x/y`だけ、Depth Moveは`position.z`だけ、perspective/orthographic・DPI・D2 Undoを既存M5 fixtureで別審判する |
| Canvas a11y proxy | **PARTIAL / macOS実機** | proxyは選択数に比例させず1 focus target + count。SafariのmacOS AX treeでCanvas説明と選択listを確認。keyboard同等操作、VoiceOver読上げは未証明 |
| IME gate | **PARTIAL / automated** | composition中のshortcut抑止とevent順をsynthetic eventで確認。macOS日本語IME候補窓、preedit、確定/取消は`PHYSICAL` |
| hot reload | **PASS / harness** | Vite virtual moduleをRust再起動なしでaccept。製品component state、plugin単体reloadは未証明 |
| Reactモック資産 | **PASS / fixed comparison** | 固定worktree build + 43 Playwright test。stress fixtureの製品component直接接続は未証明 |
| native Stage + WebView | **RESEARCHED / spike required** | CPU readbackなしの非重複sibling surfaceは既成APIで成立見込み。native TextureのWebView共有ではない |
| overlay / alpha / color | **PHYSICAL** | macOS透明wryはprivate API停止線。まずopaque Stage + 非重複WebView、overlayは別審判 |
| native Stage上の2D/3D gizmo | **OWNER DECIDED / spike required** | canonical出力外のnative wgpu presentation overlayが描画し、CPU解析幾何でhit-testする。transparent WebViewは比較対象から外し、occlusion、screen一定サイズ、M5意味、D2、a11y proxyを実機spikeする |
| resize/minimize/restore/DPI/device lost | **PARTIAL** | 現行egui実window試験は合格。Safari実画面は最小化→Raise後もAX内容を保持。候補WKWebView/native Stage同居、DPI移動、device lostは未証明 |
| Host/community同一kit | **DESIGN EVIDENCE** | 同一versioned React component/test kitを使う。権限realmまで同一にはしない |
| community sandbox/権限 | **PARTIAL / automated** | opaque-origin iframeでparent DOM、storage、network、native bridgeの直接access拒否と明示messageだけを確認。named least-privileged WebView + Rust brokerを安全基準とし、iframeからnative IPC不能かは別負例が必要 |
| crash/loop/OOM隔離 | **PHYSICAL** | iframe/CSP/SESだけでは保証しない。別rendererの停止・単体reload・host継続を実測する |
| offline production bundle | **PARTIAL** | Vite production static buildは成立。Windows WebView2 clean-machine install、runtime更新、CDN/dev-server 0は`PHYSICAL` |
| Windows WebView/IME/GPU | **PHYSICAL** | Windows 10/11 + WebView2 + 実GPU + MS-IMEでのみ合格へ上げる |
| Pen/Roto | **PHYSICAL** | Pointer Events Level 3のpressure/tilt/coalesced samplesを薄いadapterで使う。実ペン未接続 |

これにより「確認していない論点」は残さず、合格、部分合格、実機待ちを分離した。ただし
`PHYSICAL`をmacOSのheadless結果や一次資料だけで閉じないため、G0-9自体はまだ完了ではない。

## 2. 既成技術から組む境界

### 2.1 Timeline / Roto入力

- dense key描画: PixiJS `ParticleContainer`。Particleを個別event/a11y nodeにしない
- mouse/touch/pen継続: Web標準
  [`setPointerCapture`](https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture)
- pen: [Pointer Events Level 3](https://www.w3.org/TR/pointerevents3/)のpressure、tilt、
  `getCoalescedEvents()`。React独自pen protocolを作らない
- pan/zoom/edge-pan: [pixi-viewport](https://viewport.pixijs.io/jsdoc/Viewport.html)。
  `mouseEdges`のwindowed viewport制約はWebView実測する
- marquee/少数handle: [Konva公式multi-select例](https://konvajs.org/docs/select_and_transform/Basic_demo.html)
- form/focus: native input + React Aria。100,000 key分のDOM proxyを作らない

[Canvas Timeline固定source](https://github.com/techsquidtv/canvas-timeline/blob/522ac2c68e3e024017648bef8125b9af4e51c5b5/packages/react/src/components/interactions/KeyframeInteractionLayer.tsx)
にはpointer capture、document fallback、keyboard interactionがあるため先例として利用できる。一方、
0.1.0 / MPL-2.0で独自state/historyを持ち、visible keyをDOM button化する。Motolii D2の正本へせず、
入力patternとfixtureだけを監査する。

### 2.2 WebViewとnative Stage

最初に試すのは、WebViewのshell/panelとopaqueなnative wgpu Stageを非重複のsibling rectangleへ
置き、OS compositorに合成させる方式である。これはCPU readbackを不要にできる見込みが高いが、
browser WebGPU deviceとnative wgpu device/Textureを共有する方式ではない。

- wryは[`build_as_child`](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html)と
  `set_bounds`を持つ
- wgpuはmacOSのCore Animation layerとWindowsの
  [`CompositionVisual`](https://docs.rs/wgpu/29.0.3/wgpu/enum.SurfaceTargetUnsafe.html)をsurface targetにできる
- Windowsの重なり合う構成には
  [WebView2 CompositionController](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/overview-features-apis#rendering-webview2-using-composition)
  があるが、stock wryの通常controllerではなく入力転送も必要なので第2候補
- macOSのwry透明化はprivate `drawsBackground`経路を使うため、App Store配布候補では停止線

最小sibling方式でもresize、0×0/minimize/restore、DPI、surface lost、Web content process終了、
色/alpha、frame pacingをmacOSとWindowsの製品候補WebViewで再現してから採択する。

### 2.3 2D handleと3D gizmo

2Dは[Konva Transformer](https://konvajs.org/api/Konva.Transformer.html)の既成resize/rotate、
multi-node、rotation snap、bound functionを使い、Web上の比較adapterで実操作を再現した。
3Dは[Three.js TransformControls](https://threejs.org/docs/pages/TransformControls.html)で
translate/rotate/scale、world/local、各snapを再現し、
[OrbitControls](https://threejs.org/docs/pages/OrbitControls.html)をdrag中だけ無効化した。
[Babylon.js Gizmo](https://doc.babylonjs.com/features/featuresDeepDive/mesh/gizmo)も同じ問題領域の既成先例である。

ただしKonva/Three.jsはDCC操作機構の比較adapterであり、native Rust/wgpu Stageの置換でもMotoliiの
Document/API正本でもない。製品のhandle/gizmoは
[Native Stage gizmo所有境界](../reviews/2026-07-21-native-stage-gizmo-ownership.md)どおりcanonical出力外の
native wgpu presentation overlayが描き、少数固定形状はCPU解析幾何でhit-testする。Motoliiは
[M5 P2U](../specs/M5-3d-and-post.md#見かけサイズを変える直接操作-scale--depth-move)どおり
単一XYZ worldを維持し、見た目が似るScaleとDepth Moveを別操作・別channelにする。汎用`translate Z`
をそのままDepth契約へ昇格しない。

次のfixtureではperspective/orthographic、local/world、camera orbitとのpointer競合、camera距離・zoom・DPIに
依存しないscreen-space hit target、前後遮蔽時の表示/picking、common-parentとmixed-parent拒否を測る。
Web UIは同じkitのtoolbar/a11y proxyを持つ。transparent/composition WebView overlayはgizmo要件から外し、
opaque native Stage + 非重複sibling WebViewで責任分担する。次の実機spikeは所有者比較ではなく、native passの
canonical画素非汚染、resource再利用/readback 0、occlusion、screen一定サイズ、D2/Cancelを審判する。

### 2.4 同一kitとcommunity保護領域

Hostとcommunityを別UI kitにはしない。ただし同一kit、同一origin、同一native権限を同義にしない。

```text
versioned React UI kit
├─ Host bundle ── privileged host adapter ── Rust core
└─ Community bundle ── isolated realm ── typed capability broker ── D2 command
```

安全基準はnamed・最小権限WebView、軽量比較は`allow-scripts`だけのopaque-origin sandbox iframe、
計算loop停止補助はDedicated Workerである。Tauri 2の
[capability](https://v2.tauri.app/reference/acl/capability/)はwindow/WebView単位なので、
privileged host内iframeがnative IPCへ到達できないことを負例で測るまで安全としない。
CSP、iframe、SESだけでCPU loop/OOM/crash隔離済みとも扱わない。

community codeへraw Document、汎用native invoke、filesystem/network plugin、native GPU handleを渡さない。
productionはVite static bundleだけとし、dev server、HMR endpoint、CDNを含めない。Windows offlineは
[WebView2 Evergreen / Fixed Version](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/evergreen-vs-fixed-version)
のclean-machine比較を残す。Fixed Versionはbundle増と更新責任を伴う。

## 3. Computer UseによるmacOS実機観察

2026-07-21、Apple M4 macOS上でSafariを実際に起動し、`http://127.0.0.1:4179/`のVite画面を
操作した。個人情報を含む既存tabは使わず、新規tabで行った。画像証拠には既存tab名が入るため
repositoryへ保存していない。

- WebKitでReact、Canvas 2D、Pixi WebGL、Konvaを描画し、20,000 visible keysを表示
- Safariでは`navigator.gpu unavailable`。WebGPU必須runtimeにはできないことを実機確認
- macOS AX treeから見出し、30 virtual rows、Canvasの説明、native text field、選択listを取得
- 2D handle Canvas説明、固定3操作toolbar、3D gizmo Canvas説明と5操作toolbarをAX treeで取得。
  Three.js WebGLのcube/grid/translate gizmoを目視し、AX経由のRotate選択で説明が更新された
- `Cmd+M`後にwindowをRaiseし、同じAX内容とinteraction stateを再取得
- Computer Useの`type_text`は`ime-`だけ入力し日本語部分を渡せず、座標dragもWeb eventを発火
  できなかった。このため日本語IME候補窓とactual pointerを実機合格にはしない

自動Playwrightのtrusted mouse試験とComputer Use観察は役割が異なる。前者は操作意味の回帰審判、
後者はOS上の可視性、WebKit差、AX露出の確認である。IME、VoiceOver、penは人間実機審判へ残す。

## 4. 次の実機順序

1. isolated wry sibling spikeをmacOS/Windowsで起動し、CPU readback counter 0を確認
2. resize 100回、minimize/restore、異DPI monitor移動、surface/process lostを注入
3. macOS日本語IME + VoiceOver、Windows MS-IME + NVDAを同じ入力fixtureで確認
4. sandbox iframe対zero-capability WebViewへ攻撃、loop、OOM、version不一致fixtureを流す
5. clean Windows 10/11でoffline install/start、dev server/CDN request 0を保存
6. 実ペンでpressure、tilt、coalesced sample、cancelを確認

この順序で非重複siblingが不合格なら透明overlay、Windows CompositionControllerへ進む。
macOS private API、CPU pixel fallback、raw plugin権限で合格を作らない。
