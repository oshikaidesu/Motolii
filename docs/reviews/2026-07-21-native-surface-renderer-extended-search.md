# native surface renderer拡張サーチ（egui以外の追加候補・2026-07-21）

状態: **比較中**。[native renderer再選定](2026-07-21-native-surface-renderer-reselection.md)の続編。
再選定で確定した問い（React複合を維持し、native Stage/Timelineをdirect wgpu primitive batch第一候補
+ 採択済みVello 0.9局所利用で実装する）を前提に、再選定時点で未監査だった候補を4群でサーチした。

結論の先出し: **第一候補と各処分区分を覆す証拠は出なかった**。新たに得たのは(1)依存にしない
pattern、(2)日付つきwatchlist、(3)「Motolii同型のsystem WebView child + native wgpu sibling構成を
productionで出荷した実例が見つからない」というリスク事実である。本書は採否を確定せず、egui撤去、
依存追加、公開API/Document/plugin契約の変更を許可しない。

## 1. 方法と範囲

- 4群並列サーチ: Linebender生態系 / 未監査Rust UI framework / 非Rust・エンジン級2D renderer /
  先例アーキテクチャと支援基盤。
- 出典は一次資料（official repo README・ソースファイル・crates.io metadata・merged PR・公式blog）に
  限定し、全主張へURLを付す。調査環境のproxyが一部ホスト（figma.com、rive.app、linebender.org、
  news.ycombinator.com等）への直接fetchを遮断したため、遮断先の引用はsearch snippet経由であることを
  **[snippet]** と個別注記する。GitHub直接取得は注記なし。
- version日付はcrates.io `created_at` またはGitHub releasesの表示日。すべて2026-07-21取得。

## 2. Linebender生態系（採択済みVello 0.9の周辺）

| 項目 | 一次資料上の事実 | 処分 |
|---|---|---|
| vello_hybrid 0.0.9 | sparse strips系のCPU/GPU hybrid renderer。workspaceは[wgpu 29.0.3固定](https://github.com/linebender/vello/blob/main/Cargo.toml)で、`Renderer::new(device: &Device, ..)`が呼び出し側のwgpu deviceを受ける。wgpu backendソース（[render/wgpu.rs](https://github.com/linebender/vello/blob/main/sparse_strips/vello_hybrid/src/render/wgpu.rs)）にcompute参照は0件で、vertex/fragmentのみ。ただし[フォルダREADME](https://github.com/linebender/vello/tree/main/sparse_strips)は「not yet suitable for production use」、releaseタグは「pre-release alpha」（[0.0.9, 2026-05-30](https://github.com/linebender/vello/releases)）。TMIL-25（2026-04-19）は「roughly beta quality」 | **WATCH**。device共有とcompute不要は将来の局所renderer差し替え候補として最良。0.0.xの間は採択しない |
| vello_cpu 0.0.9 | READMEは「ready for production use cases」を自称するがAPI不安定を明記。SIMD/multithread CPU renderer | **WATCH**。deterministic export/CPU fallback比較の将来候補。tiny-skiaより既存peniko/kurbo語彙と整合 |
| Glifo 0.1.1 | Vello 0.9.0がtext描画を[Glifo crate](https://github.com/linebender/vello/releases)へ移行済み。「explicit Resources object for persistent image and glyph caches」「image atlas residency now preserved across renders」。TMIL-25はatlas-based glyph cachingの反復のためVello repoへ移したと記す | **既存採択の内訳**。採択済み`draw_glyphs`経路の実体。Vello READMEのglyph caching未完記述は部分的に解消へ向かうが公式は未完扱いを維持 |
| parley 0.11.0 (2026-06-26) | 「rich text layoutのAPI」。依存はFontique + **HarfRust** + Skrifa + ICU4X（[README](https://github.com/linebender/parley/blob/main/README.md)、swashは中核依存から外れた）。TMIL-25はBevyのparley移行を報告 | **CANDIDATE (text layoutのみ)**。既決のfontique + harfrust + `draw_glyphs`の正当な上位層。Timeline labelがfallback/bidi/省略記号を要した時だけ比較する。UI framework同梱物ではない |
| kurbo 0.13.1 / peniko 0.6.1 | Vello 0.9.0が消費するgeometry/styling語彙。Apache-2.0 OR MIT | **既存採択の内訳**。vello 0.9が再輸出するversionに固定 |
| Xilem / Masonry 0.4.0 (2025-10-29) | 「An experimental Rust architecture for reactive UI」。Vello + wgpu + winit + Parley + AccessKitのfull UI framework。license **Apache-2.0のみ**（生態系他crateと異なりMIT非併記） | **REJECT / PATTERN**。第二UIフレームワーク。Vello一次消費者としてMasonryのVello接続コードのみ読む |
| piet 0.8.0 | 「largely stabilized, no major API additions planned」の保守モード | **REJECT**。無関係 |
| tiny-skia 0.12.0 | CPU専用・text非対応のSkia部分移植。Linebender stewardship下で保守。BSD-3-Clause | **REJECT**。CPU fallback比較はvello_cpuを先に見る |
| Vello 1.0 roadmap | 公式な1.0公約は**存在しない**。classic READMEは現在も「alpha state」でGPU memory allocation / glyph caching / blur・filter / conflationを未完列挙。統一「Vello API」計画は放棄され、外部抽象（[AnyRender](https://github.com/dioxuslabs/anyrender)、forest-rs imaging）を紹介（TMIL-25）。中核2名はCanva勤務下でLinebender作業継続と公表 | **事実として記録**。採択済みVello 0.9の「alpha明記」前提は不変。sparse stripsのtop-level昇格声明を再確認トリガーにする |

## 3. 未監査Rust UIフレームワーク

全候補が再選定でSlint/Icedを不採用にしたのと同じ構造理由（window/event/layout/state所有、既存wgpu
deviceへの描画不可）で不採用相当だった。個別事実のみ記す。

| 候補 | 一次資料上の事実 | 処分 |
|---|---|---|
| Floem ([lapce/floem](https://github.com/lapce/floem)) | renderer可換（vger/vello/AnyRender-Skia/tiny-skia fallback）。crates.io最終release 0.2.0は2024-11-14でgit追随前提。「occasional breaking changes on our way to v1」 | **REJECT**。第二UIフレームワーク。FloemもVello/anyrenderへ収斂した事実だけ傍証として記録 |
| Makepad ([makepad/makepad](https://github.com/makepad/makepad)) | 独自Live DSL + 独自shader stack（Metal/D3D11/GL/WebGL、**wgpu不使用**）。makepad-widgets 1.0.0（2025-05-13）以後crates.io更新なし。実app（Robrix、Moly）あり | **REJECT**。非wgpuの全域独自stackでinterop最悪 |
| Dioxus Native + Blitz ([DioxusLabs/blitz](https://github.com/DioxusLabs/blitz)) | Stylo + Taffy + Parley + Vello(anyrender)のHTML engine。README「pre-alpha… would not yet recommend building apps」。一方[`anyrender_vello`の`CustomPaintSource`](https://github.com/DioxusLabs/blitz/blob/main/examples/wgpu_texture/src/demo_renderer.rs)は**共有wgpu Device/Queueでアプリ独自passをVello文書へ合成する出荷済み実装**（`examples/wgpu_texture`で動作、MIT/Apache） | **REJECT as framework / PATTERN as `CustomPaintSource`**。実browserを持つ本計画がpre-alpha HTML engineへ戻る理由はない。共有device合成の参照実装としてのみ読む |
| Freya ([marc2332/freya](https://github.com/marc2332/freya)) | **Skia** (rust-skia)描画。0.4.0（2026-07-16）で**Dioxus離脱・自前reactive coreへ全面交換**。実質単独maintainer | **REJECT**。第二GPU stack + 基盤交換直後の churn |
| Vizia ([vizia/vizia](https://github.com/vizia/vizia)) | femtovgからSkiaへ移行済み。年1 minor程度の緩い頻度。audio plugin向けbaseview埋込が独自性 | **REJECT**。無関係な差別化 |
| Cushy ([khonsulabs/cushy](https://github.com/khonsulabs/cushy)) | wgpu native（Kludgine）だがREADME自認「alpha and unsupported」。release 2024-08-20以降なし、main最終commit 2025-03-25 | **REJECT**。唯一の全wgpu候補だが停止状態 |
| Ribir ([RibirX/Ribir](https://github.com/RibirX/Ribir)) | wgpu既定だが0.4.0-alpha.65（2026-04-21）まで約2年alpha連番。「API is not stable yet」 | **REJECT** |
| Tessera ([tessera-ui/tessera](https://github.com/tessera-ui/tessera)) | 2025年発の実験framework。組込componentを持たず**custom WGPU shader/pipelineの登録界面**を第一級で公開。ただし外部所有deviceへの描画は非対応で自らwindowを所有 | **REJECT / 副次pattern**。「custom wgpu passを第一級にするUI」の若い傍証 |
| Ply ([TheRedDeveloper/ply-engine](https://github.com/TheRedDeveloper/ply-engine)) | 2026-03初出、GLSL/SPIR-V系でbackend不詳。v1.1.0は初出4か月後 | **REJECT**。実績不足 |

## 4. 非Rust・エンジン級2Dレンダラ（Vello局所役の対抗馬）

合格に必要な条件は「共有wgpu 29 deviceの上でtextureへ描ける・第二graphics stackを持ち込まない・
permissive license」。全候補が最低1つを満たさない。

| 候補 | 一次資料上の事実 | 処分 |
|---|---|---|
| Rive Renderer ([rive-runtime](https://github.com/rive-app/rive-runtime)) | C++。backend実装はmetal/gl/d3d11/d3d12/vulkan/webgpuのディレクトリ実在を確認（webgpuは実在確認のみで成熟度未証明）。[Vector Feathering](https://rive.app/blog/introducing-vector-feathering) **[snippet]**（2025-02）はGPU vectorの実新機能。licenseは[2024-03-19からMIT](https://github.com/rive-app/rive-runtime/blob/main/LICENSE)で旧source-available懸念は解消。公式Rust binding [rive-rs](https://github.com/rive-app/rive-rs)は「Vello backendで描く」と明記し、main最終commit 2025-07-04で約1年停滞 | **REJECT / WATCH**。技術的最有力対抗だがwgpu-rs interop経路が存在せず、Rust経路は結局Vello。featheringは機能benchmarkとして、rive-rsのRive Renderer backend出荷を再確認トリガーとして記録 |
| Flutter Impeller ([README](https://github.com/flutter/flutter/blob/main/engine/src/flutter/impeller/README.md)) | offline shader compile前提のFlutter renderer。Metal/Vulkan/GLES。single-header C APIのstandalone SDKを公称するが自前deviceを所有 | **REJECT**。第二C++ stack、device共有不可 |
| WebRender ([servo/webrender](https://github.com/servo/webrender)) | 「currently uses the OpenGL API internally」。GitHubはmirrorでupstreamはmozilla-central。CSS向けdisplay listで汎用path fill APIなし。MPL-2.0 | **REJECT**。GL専用でwgpu device共有不可 |
| ThorVG ([thorvg/thorvg](https://github.com/thorvg/thorvg)) | C++、SW/GL/WebGL/WebGPU backend。v1.0.0（2026-01-31）、v1.0.7（2026-07-02）と活発。MIT。[C API](https://github.com/thorvg/thorvg/blob/main/src/bindings/capi/thorvg_capi.h)の`tvg_wgcanvas_set_target`は呼び出し側のdevice/texture handleを受けるが、それは**wgpu-nativeのC handle**であり、Rust `wgpu` crateとの公開bridgeは無い（=第二のwgpu core複製をlinkする） | **REJECT as renderer / WATCH as Lottie import**。renderer役は不可。Lottie/SVG取込pipeline候補として別論点で比較する |
| forma ([google/forma](https://github.com/google/forma)) | 2024-07-18 archived | **REJECT**。終了 |
| femtovg ([femtovg/femtovg](https://github.com/femtovg/femtovg)) | NanoVG系Rust。活発（0.26.0、2026-07-20）だがmasterは**wgpu 30固定**でMotoliiの29と不一致。canvas風immediate APIで複雑fill/blend/text品質はVello未満 | **REJECT**。スコープ不足 + version skew |
| vger-rs ([audulus/vger-rs](https://github.com/audulus/vger-rs)) | UI規模のwgpu vector renderer。image未実装、wgpu 27止まり、bump中心の保守 | **REJECT** |
| NanoVG ([memononen/nanovg](https://github.com/memononen/nanovg)) | GL専用C。README自認「not actively maintained」 | **REJECT** |
| 新顔捜索 | 2025-26に信頼できる新規GPU vector renderer出現なし。当該分野の最新実装は採択済みVello自身のsparse strips系。Rive commitに内部名「Wagyu」が見えるが公表なし | **事実として記録**。6か月後再確認 |

## 5. 先例アーキテクチャ

### 5.1 Graphite — 最接近先例はTauri/wryを放棄しCEF合成へ移行済み

[GraphiteEditor/Graphite](https://github.com/GraphiteEditor/Graphite)（Apache-2.0、約88% Rust +
Svelte UI）は「web chrome + Rust所有state + wgpu/Vello viewport」というMotolii同型の分割を実装する。

- [公式codebase overview](https://github.com/GraphiteEditor/Graphite/blob/master/website/content/volunteer/guide/codebase-overview/_index.md):
  frontendは「as lightweight and minimal as possible」でwasm境界の`FrontendMessage` queueへ即引き渡し、
  可変状態はすべてRust側。**message bridge型の一方向投影**はMotoliiのtyped bounds/intent投影と同型。
- desktop版はTauriを放棄した（LWN 2025-10 **[snippet]**「the Tauri build was abandoned due to an
  insurmountable technical incompatibility」）。現行は[desktop/Cargo.toml](https://github.com/GraphiteEditor/Graphite/blob/master/desktop/Cargo.toml)にtauri/wryが無く、
  [desktop/ui/Cargo.toml](https://github.com/GraphiteEditor/Graphite/blob/master/desktop/ui/Cargo.toml)が
  **CEF offscreen rendering + `accelerated_paint`（IOSurface / D3D12 / Vulkan共有texture）**でweb UIを
  wgpu windowへ合成する。viewportはnative wgpu/Velloのまま。
- 運用課題もmerged PRとして公開: dirty rectによるUI texture upload削減(#4305)、CEFがMacで奪う
  keyboard shortcut(#4322)、CEFの別crate/process隔離(#4321)、CEF内network requestのallowlist(#4225)。

Motoliiへの含意: (1)再選定§6-6「child surfaceが不成立の時、CPU bridgeや透明WebViewへ逃げずに残る
構成」への実在回答候補が**CEF OSR + 共有GPU texture**であると判明した。(2)ただしGraphiteの放棄理由が
Motoliiの非重複sibling構成（WKWebView/WebView2）にも当たるかは未検証で、spike前にCEF枝へ倒す根拠には
ならない。

### 5.2 Tauri v2 / wry — 必要な原始機能は公式実在、ただしunstable

- [wry公式`examples/wgpu.rs`](https://github.com/tauri-apps/wry/blob/dev/examples/wgpu.rs)がwinit +
  wgpu描画 + `build_as_child`のchild WebViewを実演。Windowsは`with_clip_children(false)`が必要。
- [wry README](https://github.com/tauri-apps/wry): `build_as_child`は「macOS, Windows and Linux
  (X11 Only)」。**Waylandは非対応**でGTK埋込へ誘導。macOSの一部機能はprivate API使用。
- Tauri v2のmultiwebviewは`unstable` feature gate（[PR #9059](https://github.com/tauri-apps/tauri/pull/9059)、2024-03merge）。
  既知課題: macOSでclickまでkeyboard focusが入らない([tao #208](https://github.com/tauri-apps/tao/issues/208))、
  wgpu例のWindows透過不全([wry #1331](https://github.com/tauri-apps/wry/issues/1331))。
- wgpuを**WebViewの上**へ重ねる汎用patternは維持者公認では存在しない（[discussion #11944](https://github.com/tauri-apps/tauri/discussions/11944)）。
  Motoliiは非重複sibling前提なのでこの制約自体は既定計画と整合する。

### 5.3 対極と傍証

| 先例 | 事実 | 使い方 |
|---|---|---|
| Rive Editor | Flutter（CanvasKit→自社C++ renderer）で**全部を単一GPU toolkit**に統一 **[snippet]**（[flutter.dev showcase](https://flutter.dev/showcase/rive)） | hybrid分割の**反対側先例**。hybrid支持の根拠に引用しない |
| Figma | 「[Figma rendering: powered by WebGPU](https://www.figma.com/blog/figma-rendering-powered-by-webgpu/)」**[snippet]**（2025-09-18）: C++→WASM renderer、WebGL→WebGPU移行、React chromeと同一page内 | 「custom GPU canvas + React chrome」の最有名傍証。ただしbrowser内合成でnative child surfaceの証拠ではない |
| Warp | [How Warp Works](https://www.warp.dev/blog/how-warp-works): Electron実験後に**全native Rust + Metal**へ | 全native極。hybridが失敗した場合の対極として記録 |
| Bevy / [bevy_vello](https://github.com/linebender/bevy_vello) | editor向けwidget層（bevy_feathers）は実験段階 | Vello-in-app-shellの傍証のみ。代替UI基盤ではない |

**重要なリスク事実**: 「system WebView child + native wgpu sibling surfaceを同一windowで出荷した
production実例」は今回の捜索で**見つからなかった**。最も近い出荷物はCEF合成（Graphite）と全native
（Warp/Zed）である。coordinatorと合成層がMotolii最大の新規性＝最大リスクであることが確定し、
windowed spikeの優先度は上がる。

## 6. 支援基盤の更新

- **AccessKit 0.24.0（2026-01-10）で[multi-tree support](https://github.com/AccessKit/accesskit/pull/655)がmerge済み**。
  graft nodeで`TreeId`付き子treeを接ぎ木でき、Servo/eguiがNVDA/Orca/VoiceOverで検証。再選定の
  「bounded proxy」a11y計画はこの機構でちょうど実装可能。未検証はWKWebView/WebView2自身のa11y treeとの
  縫合で、これをspike項目へ追加する。制約: cross-tree参照不可、multi-tree updateは非atomic。
- **[wgpu-profiler](https://github.com/Wumpf/wgpu-profiler)**はinterleaved command bufferを公式対応し、
  Stage/Timeline 2 surface + 1 deviceの計測に適合。`Features::TIMESTAMP_QUERY`必須。
- **wgpu 30.0.0が2026-07-01に出た**（[releases](https://github.com/gfx-rs/wgpu/releases)）。本比較の
  副作用でbumpしない。29固定の間はwgpu 30系ecosystem（femtovg等）とdevice共有不可という制約も固定される。
- surface-lost対応の公式語彙: `SurfaceError::Outdated`→re-configure、`Lost`→surface再生成
  （[docs](https://docs.rs/wgpu/latest/wgpu/enum.SurfaceError.html)）。復旧はapp単位でなくsurface単位に書く。

## 7. GPUI「開発停止」風説の検証

HNに「Zed also stopped GPUI development」と題する2026-02のthread（[item 47003569](https://news.ycombinator.com/item?id=47003569)、
本文はproxy遮断で未読）が存在するが、一次証拠は継続開発を示す: [gpuiはcrates.io公開済み](https://crates.io/crates/gpui)
（[v0.2.0公式告知](https://x.com/zeddotdev/status/1976309201744937039)）、gpui 0.2.2への
[issue #57732](https://github.com/zed-industries/zed/issues/57732)、2026年6-7月のZed本体でのgpui変更。
風説は未検証のまま採否根拠にしない。既決の「GPUIはpatternであり依存しない」はどちらに転んでも頑健。

## 8. 再選定への帰結

1. **覆らない**: direct wgpu primitive batch第一候補、Vello 0.9局所利用、React複合維持、egui baseline
   温存、全full UI framework不採用相当——のいずれにも反証は出なかった。
2. **patternの追加**（依存にしない）: Blitz/anyrenderの`CustomPaintSource`（共有deviceでのVello +
   独自pass合成の出荷済み参照実装）、Graphiteの`FrontendMessage`一方向投影、GraphiteのCEF OSR +
   共有GPU texture合成（child WebView不成立時の予備枝）。
3. **watchlistの追加**（再確認トリガー付き）: vello_hybrid/vello_cpuのtop-level昇格声明、rive-rsの
   Rive Renderer backend出荷、ThorVGのLottie import役、parleyのtext layout役、wgpu 30系への移行時期。
4. **リスク更新**: 同型構成の出荷実例ゼロにより、windowed spike（再選定§5）の優先度が上がる。spike
   検証項目へ追加: wry `build_as_child`のfocus初回click問題、Windows `with_clip_children`、Wayland
   非対応の扱い、AccessKit graft treeとWebView a11y treeの縫合。

## 9. Fableへ追加する反証質問

1. GraphiteがTauri/wryを放棄した「technical incompatibility」の具体は何で、Motoliiの非重複sibling
   WKWebView/WebView2構成にも該当するか。該当するならspike前に判定できるか。
2. CEF OSR + 共有texture予備枝を持つ場合、第一候補（system WebView child）との分岐判定はどの証拠・
   どの時点で行うのが最小コストか。
3. anyrender `CustomPaintSource`の設計を依存なしでcoordinatorへ写す時、最小の界面（device handle
   受領・texture登録・suspend/resume）は何か。
4. AccessKit multi-tree graftとWebView側a11y treeの縫合で、window単位のfocus traversalはどちらが
   所有すべきか。
5. wgpu 29固定を維持する期限と、30系移行を強制する外部条件（依存のMSRV、securityfix）は何か。

## 10. 停止線（本書で追加）

- watchlist項目（vello_hybrid、vello_cpu、rive-rs、ThorVG、parley）をspike証拠なしに依存へ昇格しない。
- wgpu 30へのbumpを本比較の副作用として行わない。
- CEF予備枝の存在を理由にchild WebView spikeを省略しない。逆にwry unstable指定を理由にspike前へ
  CEF枝へ倒さない。
- **[snippet]** 注記付きの主張と風説（GPUI停止等）を、一次資料で再確認するまで採否根拠に使わない。
- 本書の処分「REJECT相当」はspike完了までの比較上の整理であり、decision-indexの状態は再選定と同じ
  「比較中」を維持する。
