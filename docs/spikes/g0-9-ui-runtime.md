# G0-9 UI runtime部分スパイク（2026-07-21）

状態: **部分合格／runtime採否は比較中**。React/WebViewを製品へ組み込まず、G0-9で先に
確認できるBrowser virtualization、dense Timeline surface、browser WebGPU、Vite HMRと、
現行native wgpu基準を同じ規模のfixtureで実測した。本書はWebView採択、plugin UI公開、
Document/API変更を許可しない。

## 結論

1. **BrowserをReactにしても10,000 itemを全DOM化する必要はない**。固定高virtual listは
   10,000 itemに対して24 rowだけをDOMへ置き、最終itemのstable ID選択をscroll後も保持した。
2. **TimelineをReactにしても100,000 keyをDOMへ置く必要はない**。Reactはsurfaceを所有し、
   Canvas 2Dまたはbrowser WebGPUがvisible rangeを単一面へ描画できた。
3. **browser WebGPUの局所採択は技術的候補として残る**。ただし今回のadapterはheadless
   ChromiumのSwiftShaderであり、native Rust/wgpuのApple M4 Metal deviceとは別物だった。
4. **hot reloadの配線は成立した**。Vite virtual module更新をRust process再起動なしで
   acceptした。ただし製品component編集、状態保持、plugin更新の完全な審判ではない。
5. **大量dragは既成scene graphで候補を維持した**。20,000 visible keyを背景に、PixiJSと
   Konvaで最大10,000 key / 1,000 objectの選択overlayをgroup移動した。move中はReact stateと
   semantic commitを更新せず、Cancel復元、release 1回だけcommitするadapter条件を保った。
6. **G0-9の最終採否はまだ閉じない**。[全確認点マトリクス](g0-9-verification-matrix.md)で
   自動合格、部分合格、対象実機必須を分離した。actual mouse/snap/marquee/pointer capture primitiveと
   macOS WebKit/AXは前進したが、WebView/native Stage同居、IME/VoiceOver、sandbox負例、Windowsは残る。

したがって現時点の方向は「React DOMかnative wgpuか」の二者択一ではなく、
**React DOM＝shell/Browser/form、Canvas/browser WebGPU＝Web runtime内の高密度面、
Rust/native wgpu＝映像Stage/render core**という責任分担をhybrid候補として次段へ送る。

## スパイク実装

[spikes/g0-9-web-ui](../../spikes/g0-9-web-ui/)は製品workspace外の隔離ハーネスである。

- React 19.0.0 / Vite 6.4.3 / Playwright 1.61.1 / PixiJS 8.19.0 / Konva 10.3.0
- `g0-9-dense-ui-v1`: Browser 10,000 item、Timeline 1,000 clip / 100,000 key / 32 track
- Browserはscroll位置からvisible rowだけをReact要素へ投影
- TimelineはCanvas 2Dとbrowser WebGPUを同じページ内の交換可能な局所rendererとして計測
- 動的dragはPixiJS WebGL `ParticleContainer`とKonva Canvas2D `Group` / drag `Layer`を比較。
  Reactはsurfaceをmountするだけでper-frame stateを持たない
- Vite dev serverのvirtual moduleをinvalidated/reloadし、HMR acceptまでを計測
- 製品WebView、native texture共有、Document/command、plugin sandboxは含めない

依存監査時にVite 6.0.11の既知脆弱性が検出されたため、既存Reactモックと同じVite 6系列の
6.4.3へ上げた。`npm audit --audit-level=moderate`は0件である。このversionはスパイク固定値で、
製品runtime契約ではない。

PixiJSとKonvaはいずれもMIT。production buildは最大chunk 548.88 kB（gzip 169.56 kB）となり、
Viteの500 kB警告が出た。これは採択前にtree shaking / code splitとWebView起動時costを測るべき
負債であり、性能値だけで無視しない。

## 既知技術の調査結果

独自scene graph、hit-test、pan/zoom、timeline engineを新設する前に公式資料を調べた。

| 候補 | 既成範囲 | 現時点の扱い |
|---|---|---|
| [Konva](https://konvajs.org/docs/overview.html) | DOM状のStage/Layer/Group/Shape、hidden hit Canvas、event、drag、transform。公式に[20,000 interactive node](https://konvajs.org/docs/sandbox/20000_Nodes.html)例 | **今回実測**。Pen/Rotoや少数handleに有力。10万key全node化は未採択 |
| [PixiJS](https://pixijs.com/8.x/guides/components/scene-objects) | GPU scene graph、DOM風event、[100,000 Particle](https://pixijs.com/8.x/guides/components/scene-objects/particle-container)例、static/dynamic upload | **今回実測**。dense Timeline有力。ただしParticle APIはexperimental、個別Particle eventなし |
| [@pixi/react](https://react.pixijs.io/getting-started) / [pixi-viewport](https://viewport.pixijs.io/jsdoc/index.html) | React binding、drag/pinch/wheel/clamp/snap viewport | 次段候補。独自wrapper/viewportより先に検証する |
| [Fabric.js](https://fabricjs.com/docs/why-fabric/) | Canvas object model、hit detection、control、event、serialization | Stage object編集候補。dense key性能とFabric JSON非正本化を未検証 |
| [Canvas Timeline](https://github.com/techsquidtv/canvas-timeline) | headless timeline、snap/history、React interaction、worker Canvas | 直接先例だが0.1.0、9 stars、44 commits、MPL-2.0。keyframe stress証拠前に依存しない |
| [React Video Editor Timeline](https://www.reactvideoeditor.com/docs/core/components/timeline) | drag/snap/multi-select/zoom/historyのcopy可能component | 公式が開発中・大規模性能依存を明記。先例のみ |
| [elah](https://www.elah.dev/) | React binding、integer-frame timeline、Immer history、WebGL2/OffscreenCanvas、pure resolver | engine分離の先例。Motolii Rust coreを置換・二重化しない |

調査から採る原則は「React DOM対Canvas」の二択ではなく、DOM shell、既成scene graph、dense batch、
native Stageを責任分担すること、同一deltaで動くmulti-selectionは各要素の座標をper-frame永続更新せず
選択overlayのgroup transformでpreviewすることである。Motolii固有なのはD2 commandへの最終変換、
正準座標、snap意味、native Stage合成であり、汎用scene graph自体ではない。

## 実測環境

- macOS Darwin 24.5.0 / arm64 / Apple M4
- native: wgpu 29.0.4 / Metal
- Web: Playwright Chromium 149 headless
- browser WebGPU adapter: Google SwiftShader
- Web viewport: Timeline 1200×512、Browser 420×480
- native timeline viewport: 1920×512

## 実測結果

| 対象 | 操作／同期 | 結果 | 判定 |
|---|---|---|---|
| React Browser | 10,000 itemを120段階scrollし各段階で次のanimation frameまで待つ | DOM 24 row、median 16.60 ms、p95 18.70 ms。`asset-09999`選択をscroll後も保持 | **構造合格** |
| Canvas 2D Timeline | 120 frame、48秒window、visible key 20,000。clip+keyを描画 | median 3.90 ms、p95 5.40 ms | **局所面成立** |
| browser WebGPU Timeline micro-probe | 120 frame、48秒window、visible key 20,000。submit後`onSubmittedWorkDone`待機 | median 3.80 ms、p95 4.00 ms | **利用可能性合格** |
| native wgpu Timeline | 600 frame、visible clip 210 + key 20,005。submit後`device.poll(Wait)` | median 4.17 ms、p95 6.26 ms | **native基準再合格** |
| Vite HMR | virtual module revision 0→1をaccept | 17 ms、Rust再起動なし | **配線合格** |
| React製品候補資産 | 固定比較worktreeでbuild + Playwright | build成功、43/43 test成功、npm audit 0 | **再利用可能性維持** |

### 動的drag追試

90 sample × 5 draw（450 draw）をcaseごとに実行した。p95は1 draw換算。Pixiは20,000 key背景を
含むstageを毎回renderし`gl.finish()`が利用可能なrendererでは完了を待つ。Konvaは静的背景Layerを
再描画せず専用drag Layerだけを同期描画する。

| library / 対象 | 選択数 | overlay構築 | drag median / p95 | 判定 |
|---|---:|---:|---:|---|
| PixiJS key | 1 / 1,000 / 10,000 | 1.50 / 1.10 / 5.00 ms | 0.10 / 0.38、0.10 / 0.24、0.10 / 0.32 ms | **group drag成立** |
| Konva key | 1 / 1,000 / 10,000 | 1.40 / 7.20 / 46.10 ms | 0.00 / 0.02、0.52 / 0.68、5.02 / 6.36 ms | **10,000まで候補維持** |
| PixiJS object | 1 / 100 / 1,000 | 0.70 / 0.50 / 0.80 ms | 0.06 / 0.08、0.06 / 0.08、0.06 / 0.08 ms | **group drag成立** |
| Konva object | 1 / 100 / 1,000 | 0.90 / 1.70 / 6.70 ms | 0.00 / 0.02、0.08 / 0.10、0.58 / 0.72 ms | **group drag成立** |

全caseでadapter harness上のmove中semantic writeは0、Cancelでoverlay transformが原点へ戻り、
release callbackは1回だった。これは既存D2 command/Undoの実接続試験ではなく、UI hot loopが
Document writerを呼ばない構成証拠である。
Pixi surfaceは計測前にpixel extractionを1回だけ行い、背景色と異なる10,791 pixelを確認した。
このreadbackはblank surface拒否用で、drag計測sampleには含めていない。

証拠は[Web report](g0-9-web-ui-evidence/report.json)と
[native timeline report](g0-9-web-ui-evidence/native-timeline-report.json)に保存した。

## 数値を直接比較してはいけない理由

上表は同じ規模の入力を使うが、rendererの勝敗表ではない。

- Browserの時間はReact処理時間だけでなく60 Hzの`requestAnimationFrame`待ちを含む。
- Canvas 2DはAPI呼出し完了までで、GPU present完了を強制していない。
- browser WebGPU micro-probeはkey pointだけを描き、clip rectangleをまだ描いていない。
- 動的dragは全選択要素へ同じdeltaを適用するgroup transformで、個別keyの異なる座標変更、snap、
  hit-test、marquee、pointer captureを測っていない。
- PixiJSはWebGL/SwiftShader、KonvaはCanvas2Dでrendererが異なり、直接の勝敗値ではない。
- browser WebGPUはSwiftShader、native wgpuはApple M4 Metalでadapterが異なる。
- Webは1200×512、nativeは1920×512でviewportが異なる。
- nativeはCPU cull、upload、submit、GPU完了待ちをすべて含む。

このため今回確定できるのは、virtual DOMと局所GPU surfaceが候補から脱落しないことだけである。
「browser WebGPUがnativeより速い」「Canvas 2Dで製品Timelineが完成する」とは結論しない。

## native egui基準の再検証

現行main相当のegui基準について次を再実行した。

- `cargo test -p motolii-ui --test public_boundary` — 3/3
- `cargo test -p motolii-ui --test u1a1_static_viewport` — 3/3
- `cargo test -p motolii-ui --test u1a1_window_smoke` — 3/3
- `cargo run --manifest-path spikes/timeline-bench/Cargo.toml --release -- --json` — PASS

これによりtoolkit型の公開境界流出拒否、event loop内render/join/readback拒否、実windowの
resize/minimize/restore、single-writer編集とlatest worker resultの製品preview到達を保持した。
React候補の存在はこの成立証拠を撤回しない。

## 未検証と次の停止線

G0-9を閉じる前に少なくとも次を別スパイクで検証する。

1. macOS/Windowsの製品候補WebViewで、native wgpu StageをCPU pixel readbackなしに合成できるか
2. resize、minimize/restore、DPI移動、overlay、alpha、色、device lost、古いgeneration破棄
3. 日本語IME、keyboard/focus、screen readerとCanvas上のaccessible proxy
4. Hostとcommunityが同じReact kitを使いつつ、file/network/GPU/Document mutationを権限分離できるか
5. 実ReactモックのBrowser/Timelineを今回のstress fixtureへ接続し、通常操作を保てるか
6. Windows WebView実機とoffline配布、dev serverを出荷物へ含めないproduction bundle
7. `@pixi/react` / `pixi-viewport`、Konva event delegationでactual pointer drag、snap、
   offscreen drag、marquee、selected handleだけのa11y proxyを既成APIから組めるか
8. Canvas Timeline等の既成timelineを依存監査し、Motolii固有実装量をさらに削れるか

上記の現在値、既成部品への割当、実機順序は
[G0-9全確認点マトリクス](g0-9-verification-matrix.md)を正とする。

次のどれかを仮定した時点で停止する。

- browser WebGPUとnative wgpuを同じ`GPUDevice`または共有Textureとして扱う
- WebGPUが使えない環境で製品StageをCPU readbackへ黙ってfallbackする
- 24 DOM rowという結果から可変高tree/gridの完成を主張する
- HMR約17 msからplugin互換、sandbox、状態migrationまで解決済みと主張する
- SwiftShaderの数値をApple MetalまたはWindows GPUの製品性能値にする

## 再現

```sh
cd spikes/g0-9-web-ui
npm ci
npm audit --audit-level=moderate
npm run build
G0_9_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/report.json \
G0_9_INTERACTION_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/interaction-report.json \
G0_9_SANDBOX_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/sandbox-report.json npm test

cd ../..
cargo test -p motolii-ui --test public_boundary --test u1a1_static_viewport
cargo test -p motolii-ui --test u1a1_window_smoke
cargo run --manifest-path spikes/timeline-bench/Cargo.toml --release -- --json
```
