# front-end 選定の再比較 — Makepad / Xilem-Masonry / React+direct wgpu / egui(2026-08-29)

状態: **観察(歴史記録)**。本書は2026-08-29の1日の探索・probe実測の記録であり、**それ自体を設計根拠にしない**(レビュー規律1: 調査文書の結論をそのまま設計根拠にしない。利用者指示「文書に足すのも大事ですが、あくまで歴史として」)。同日の利用者裁定はdecision-index(stage4)が正本。§1〜1.8は当日の探索過程(初期4候補比較→GitHub新顔探索→wgpu29軸再検索→Rerun View埋め込みの壁→Rerunフロントの実装をソースで確認)、§3が中間の視点転換、§4〜7がprobe実測。

対象: `app/motolii`(現行front-end)。[Rerun部品目録調査](2026-08-29-rerun-parts-catalog-survey.md)で
Motoliiが`re_renderer`(Rerunフォーク)の上に建っていることが実測で確定した直後、「Makepad自身は
wgpuではなく独自Metal実装であり、`re_renderer`とのゼロコピー共有には自前IOSurfaceブリッジが要る」
という構造上の摩擦が改めて意識された。頭がクリアな状態で正式に再比較してほしいという利用者依頼。

## 0. 経路(この結論に至るまでの決定の変遷、事実のみ)

各文書は本書の判断材料であり、本書がこれらを上書きするわけではない。日付順:

| 日付 | 決定/観察 | 内容 | 出典 |
|---|---|---|---|
| 2026-07-18 | 決定(後に撤回) | egui採用。`egui-wgpu 0.35`の`WgpuSetup::Existing`でMotolii既存wgpu deviceを共有、`register_native_texture`でCPU bridge無し。Apple M4/Metal実機でdevice共有・lifecycle・IME(CJK fallback registration後)を実測 | [2026-07-18-m3-egui-selection.md](2026-07-18-m3-egui-selection.md) |
| 2026-07-21 | 観察 | native surface renderer拡張サーチ。Xilem/Masonryは「第二UIフレームワーク」としてREJECT/PATTERN、MakepadもREJECT(「非wgpuの全域独自stackでinterop最悪」)。第一候補は「React複合維持+direct wgpu primitive batch」で反証出ず | [2026-07-21-native-surface-renderer-extended-search.md](2026-07-21-native-surface-renderer-extended-search.md) |
| 2026-07-24 | 決定 | egui、製品runtime候補から撤回。比較・診断baselineとしてのみ保持 | [2026-07-18-m3-egui-selection.md](2026-07-18-m3-egui-selection.md) §1 |
| 2026-08-07 | 決定 | 標準をReact Native + rust-skia + wgpuへ再基線化。旧`direct wgpu primitive batch`第一候補・opaque child WebView islands構成は新規実装標準から外す(egui同様baseline保持) | [2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) |
| 2026-08-20 | 決定 | 裁定251: front-endをMakepadに確定(裁定5「frontはicedのみ」・裁定44「CPU経路」を覆す)。ゼロコピー契約を非交渉として設定 | decision-index.md:450 |
| 2026-08-26/27 | 決定 | Makepad fork切り方確定(3室・`SharedPresentable`・`ViewBuilder::new_with_external_resolved`)。Makepad非wgpuの事実そのものは維持したまま、共有面(IOSurface/DXGI/dma-buf)ブリッジで対処する方針を明文化 | [2026-08-26-stage-zero-copy-makepad-fork-seam.md](2026-08-26-stage-zero-copy-makepad-fork-seam.md) |
| 2026-08-29 | 観察 | Rerun部品目録調査で「Motoliiは`re_renderer`(自フォーク)の上に建つ」ことが再確認され、フォークに既に`ViewBuilder::new_with_external_resolved`等の「外部ホストがRerunの描画経路に直接繋がるembedder API」が実装済みと判明 | [2026-08-29-rerun-parts-catalog-survey.md](2026-08-29-rerun-parts-catalog-survey.md) |

**注記**: React Native+rust-skia(08-07)から Makepad(08-20)への転換の一次資料(裁定5/26/27/44/45)は
`next/DECISIONS.md`にあるが、`next/`は裁定270(2026-08-27)で「世界の分断」により生きた世界から
除外済み(現行mainのworktreeにこのファイルは存在しない。過去のagent worktreeにのみ残存)。
転換理由の一次文言は本書では未回収——読みたい場合はgit historyまたは古いworktreeを辿る必要がある。

## 1. 比較表(3軸)

### (a) Makepad継続(現行IOSurfaceブリッジを完成させる)

| 軸 | 事実 |
|---|---|
| LLM/人間可読性 | DSL(`script_mod!`)は独自言語。同一`script_mod!`ブロック内の裸名参照が静かに空になる等、Makepad固有の罠が複数実測済み(2026-08-28実測、セッション内メモ)。学習コストの証拠として社内skillが14本(`makepad-2.0-*`)必要になっている。skillsは`design-judgment`を先頭に毎回適用する運用規律が別途要る |
| re_rendererとのゼロコピー共有コスト | Makepad自身はwgpuではなくMetal/D3D11/OpenGL直叩き([platform/src/os/apple/metal.rs](https://github.com/makepad/makepad))。共有には(1)Makepad fork側に`SharedPresentable`+`Cx::create_presentable_texture`(IOSurface/DXGI/dma-buf)、(2)Rerunフォーク側に`ViewBuilder::new_with_external_resolved`、の**2本のfork patch**が要る。3室(Host/窓の葉/r7継ぎ目)の責任分担は設計済みだが、mac版のみ2026-08-26時点で「共有面が0×0を返す」実装済み・未接続、win/linuxは未着手 | 
| 移行コスト | ゼロ(現状維持)。ただし完成コストは非ゼロで残っている——fork 2本(Makepad/Rerun)の未push、win/linux実装、`SharedPresentable`パスの検収がまだ | 
| その他の実測事実 | `makepad/makepad`本家に「wgpu化した前例」は無い(2026-08-29 GitHub検索・issue/PR/discussion確認)。作者Rik Arends自身が「wgpuは薄い実行器をもう1個足すだけ」と位置づけ、同一Device化を目指す設計ではない([HackMD](https://hackmd.io/@dspfac/r1LdPpDJ3)、[Issue #86](https://github.com/makepad/makepad/issues/86)) |

### (b) Xilem/Masonry(Vello+wgpu+winit)への再移行

| 軸 | 事実 |
|---|---|
| LLM/人間可読性 | 素のRust(struct/trait)で書く。DSLなし。ただしreactive view treeの語彙(`View`/`Widget`/`WidgetPod`)を新たに学ぶ必要がある。Makepad DSLほどの罠報告は無いが、実プロダクトでの運用実績自体が薄い(下記) |
| re_rendererとのゼロコピー共有コスト | MasonryはVello(wgpu 29系)上に構築されており、原理上は`re_renderer`と同じ`wgpu::Device`/`Queue`を共有できる可能性が高い(egui-wgpuの`WgpuSetup::Existing`と同型の口があるかは2026-08-29時点で未検証・未確認)。IOSurfaceブリッジのような別GPU API間のfork patchは原理上不要になりうるが、**実証はまだ無い**(egui-wgpuで示したような自プロジェクトでの実機証拠は無し) |
| 移行コスト | 大。現行`app/motolii`は既にMakepad DSLで書かれておりゼロから書き直しに近い。2026-07-21調査時点でMasonry 0.4.0は「experimental」「alpha state」、2026-08-29のWebSearchでも「alpha」表記継続を確認——5週間で成熟度に大きな変化のシグナルなし |
| その他の実測事実 | 2026-07-21調査時点でXilem/Masonryは「第二UIフレームワーク」としてREJECT/PATTERN判定済み(採用ではなく設計パターン参照止まり)。license: Apache-2.0のみ(他Linebenderクレートと異なりMIT非併記) |

### (c) React複合+direct wgpu primitive batchへの回帰

| 軸 | 事実 |
|---|---|
| LLM/人間可読性 | React部分はTS/JSXで一般的な可読性。wgpu primitive部分は素のRust。ただしTS↔Rustの境界(typed intent、snapshot投影)自体が追加の複雑さ。React Native+rust-skia基線(08-07)ではこの境界のruntime不変条件(D2 single writer、snapshot専有、境界をper-frame同期にしない等)を明文で多数課しており、学習面はMakepad DSLと別種の複雑さ |
| re_rendererとのゼロコピー共有コスト | Stageはネイティブwgpu面なので、`re_renderer`と同一Device共有は原理上直接可能(Makepadのような別GPU APIブリッジが不要)。2026-08-06/07の隔離probeで「separate wgpu preview texture + rust-skia overlay + wgpu composite + CAMetalLayer present」を実機確認済み(Retina 2560×1440、live resize、native child view内drag、RN TextInputのfocus transfer)。ただしこのprobeは`re_renderer`との統合ではなくMotolii独自wgpu Stageでの検証 |
| 移行コスト | 大。現行Makepad DSL実装をReact Native shell + Rust wgpu/rust-skia nativeコンポーネントへ作り直す。08-07決定時点で「全面置換を一度に行わない、outcome単位でcutover」という移行手順まで設計済みだったが、実装は着手前に08-20でMakepadへ再転換されており、その後の実装進捗はゼロ |
| その他の実測事実 | 2026-07-21調査時点で「system WebView child + native wgpu sibling surfaceを同一windowで出荷したproduction実例」は捜索で見つからず(最も近い出荷物はGraphiteのCEF OSR合成、Warp/Zedの全native)。Windows実機(RNW Fabric Component View、DX12 present、device lost)は2026-08-07時点で`WINDOWS_EXTERNAL_GATE_PENDING`のまま未実行 |

### (d) egui再検討

| 軸 | 事実 |
|---|---|
| LLM/人間可読性 | immediate mode、素のRust。DSLなし。当時の採用理由に「editor型高密度UI・Host自動生成panel・Rust/LLMによるcomponent単位変更との相性の良さ」が明記されている([2026-07-18-m3-egui-selection.md](2026-07-18-m3-egui-selection.md) §3-4) |
| re_rendererとのゼロコピー共有コスト | **4候補中で唯一、自プロジェクトでの実機ゼロコピー証拠がある**。`egui-wgpu 0.35`の`WgpuSetup::Existing`でMotolii既存wgpu deviceをそのまま共有、`register_native_texture`で`Rgba8Unorm` TextureViewを直接表示。Apple M4/Metal実機で「CPU pixel bridge無し」「device二重化無し」を確認済み(2026-07-18)。これは`re_renderer`ともほぼ同型の共有が可能なことを強く示唆する(re_renderer自体もwgpu 29系) |
| 移行コスト | 中〜大。現行Makepad DSL実装からの書き直しは必要だが、egui採用時点(U0a〜U1b)で「workspace UI shell、静止preview、layout投影、render worker」までは一度実装・実機確認済みの資産が残っている(baselineとして保持、削除されていない) |
| その他の実測事実 | 撤回理由(2026-07-24)の一次文言は本書で回収した`2026-07-18-m3-egui-selection.md`には明記されていない(「製品runtime候補から外す」という処分のみ記載)。日本語IMEは既定fontに日本語グリフが無く、CJK font同梱かOS別system font resolverが必須という明示的な制約が付く |

## 1.5 GitHub上の未発見候補サーチ(2026-08-29追加、利用者指示)

「候補を絞ることではなく、まだ見ぬ候補があるのではという事実」を確認するため、07-21調査のREJECT/WATCHリストに無い候補をGitHub横断で洗った(担当agent、WebSearch/WebFetch)。

| 候補 | DSL | wgpu共有API | 成熟度 | ライセンス | 出典 |
|---|---|---|---|---|---|
| kas([kas-gui/kas](https://github.com/kas-gui/kas)) | なし(マクロ拡張Rust) | 独立`kas-wgpu`バックエンドcrateは存在。外部Device注入APIの有無は[未確認、ソース精読要] | README自認「not currently stable」 | Apache-2.0 |
| Repose([mlm-games/repose](https://github.com/mlm-games/repose)) | なし(Compose風宣言的Rust) | [未確認、ソース精読要] | 非常に高頻度更新(2026-08-28にv0.28.9)、pre-1.0、単独maintainer | MPL-2.0 |
| yakui([SecondHalfGames/yakui](https://github.com/SecondHalfGames/yakui)) | なし | 独立`yakui-wgpu`レンダラcrateあり(ゲームUI向け、複数backend切替可能な設計)。外部Device注入の可否は[未確認、ソース精読要] | 継続更新 | [未確認] |
| WGPUI([Readout-Studio/WGPUI](https://github.com/Readout-Studio/WGPUI)) | なし(DearImGui風) | [未確認] | README自認「Functional and Unstable」 | MIT |
| Blinc([project-blinc/Blinc](https://blinc.rs/)) | なし | なし(README上、自前でwindow/GPUコンテキストを所有する自己完結型と明記——Motolii要件と逆方向) | 2026年初頭公開の新規、デスクトップ"stable"自称 | Apache-2.0 |
| Pax([paxdotdev/pax](https://github.com/paxdotdev/pax)) | **あり**(独自GUI DSL) | `gpu` feature下でwgpu使用と見られるが外部Device共有の記載なし | アクティブ、デスクトップはmacOSのみ(2025時点) | [未確認] |
| GPUIエコシステム(gpui-component等) | なし | GPUI本体は非wgpu(Metal/Vulkan/DirectX直叩き)。派生は全てGPUI依存で単体wgpu共有レイヤーとして切り出されていない | gpui-component活発(13.6k star) | [未確認] |

**結論(この節のみ)**: `egui-wgpu`の`WgpuSetup::Existing`に匹敵する「外部wgpu Device/Queueを一次資料で確証できる形で共有可能」と言える新顔は、今回の探索では見つからなかった。kas/Repose/yakuiの3つは「wgpuベースの独立バックエンドcrateを持つ」という点で可能性はあるが、いずれも外部Device注入APIの有無をソースコードレベルで確認していない未検証状態。07-21調査のREJECT判定を覆す証拠は無い(観察のみ、決定ではない)。

## 1.6 wgpu 29系軸での再検索 — dear-imgui-wgpuが条件を満たす新候補として浮上(2026-08-29追加)

利用者の追加裁定: 「`re_renderer`が軸、言語はRust不問」「クリエイタソフトなので大半は独自ウィジェットになる」を踏まえ、必須条件を**「既存`wgpu::Device`/`Queue`を注入でき、既存`TextureView`をゼロコピー表示できること」**に絞って再検索した。

| 候補 | 外部Device/Queue注入 | 既存TextureViewゼロコピー表示 | wgpu 29系対応 | ライセンス | 活発さ |
|---|---|---|---|---|---|
| **dear-imgui-wgpu**([Latias94/dear-imgui-rs](https://github.com/Latias94/dear-imgui-rs)) | **Yes** — `WgpuRenderer::new(WgpuInitInfo::new(device, queue, surface_format), ...)`([init.rs](https://raw.githubusercontent.com/Latias94/dear-imgui-rs/main/backends/dear-imgui-wgpu/src/renderer/init.rs)) | **Yes** — `register_external_texture(&wgpu::TextureView)`/`update_external_texture`。コメント明記「The application retains ownership of the texture contents」([external_textures.rs](https://raw.githubusercontent.com/Latias94/dear-imgui-rs/main/backends/dear-imgui-wgpu/src/renderer/external_textures.rs)) | Yes — `wgpu-27`/`28`/`29`/`30`を相互排他featureで明示的に切替可能(既定30) | MIT OR Apache-2.0 | 活発(直近commit 2026-08-17、★95、docking/ImPlot/ImGuizmo/ImNodes拡張群あり、単発個人プロジェクトでない) |
| egui-wgpu(既知) | Yes(実機確認済み) | Yes(`register_native_texture`) | Yes | MIT OR Apache-2.0 | rerun-io本体が保守 |
| yakui-wgpu([SecondHalfGames/yakui](https://github.com/SecondHalfGames/yakui)) | Yes — `pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self`([lib.rs:82](https://raw.githubusercontent.com/SecondHalfGames/yakui/main/crates/yakui-wgpu/src/lib.rs)) | Yes — `add_texture`/`update_texture` | **未確認**(現mainは`wgpu = "30.0.0"`固定、29系対応履歴は未調査) | MIT/Apache-2.0 | 中(★345) |
| kas-wgpu | **No** — `Instance::new`が`wgpu::Instance`を自前生成し外部Device注入口が無い(GPU初期化チェーン全体を自己所有) | 未確認 | 部分 | Apache-2.0 | 活発(★, 直近commit 2026-08-25) |
| tessera-ui | 未確認・おそらくNo — `run_desktop`等の自己完結ランタイムのみ公開、`ExternalTextureRegistry`は内部offscreen用途で外部受入ではない(ソース確認済み) | 未確認 | **Yes** — `wgpu = "29.0.1"`とMotolii側と完全一致 | MIT/Apache-2.0 | 中(★260、最終commit 2026-05-31) |
| rui(audulus) | 未確認 | 未確認 | **No** — `wgpu = "0.20.0"`(CalVer化以前の旧世代) | MIT | 除外(★2040だが世代不一致) |
| Repose | 未確認(GitHub検索で実体リポジトリを特定できず) | 未確認 | 未確認 | 未確認 | — |
| Rerun公式がegui以外を検討した形跡 | — | — | — | — | issue/PR検索で「instead of egui」等の言及なし(不在の証拠であって検討皆無の確定証拠ではない) |

**この節の結論(断定ではない)**: 必須条件(外部Device/Queue注入+ゼロコピーTextureView表示+wgpu 29系)を一次資料で明確に満たすのは**egui-wgpu**と**dear-imgui-wgpu**の2つ。dear-imgui-wgpuは「クリエイタ/開発者ツールでの独自ウィジェット描画」という文脈でDear ImGui自体が業界的に最も実績のある選択肢(RenderDoc、Unreal Editorデバッグ層、多くのDCCツール内製パネルの先例)である一方、本体はC++でFFI境界を1本背負う。egui-wgpuはRerun公式viewer自身が採用している組み合わせという「原作者の実績」を持つ。yakui-wgpuも技術要件は満たすがwgpu 29系対応が未確認、tessera-uiはwgpuバージョンが完全一致するが外部Device注入口が見当たらない。

## 1.7 Rerun View埋め込み自体の壁(2026-08-29追加、利用者指示「一次資料を検索」)

UI toolkit選びとは別に、「`re_renderer`/Rerun View系を外部ホストへ埋め込む」こと自体の一次資料上の壁を調べた。

**最重要の発見**: Rerun公式の`rerun-docs`リポジトリ「What is Rerun?」に次のロードマップ記述がある。

> "Over time, Rerun will evolve from a tool to a fully customizable toolkit, [...] **You'll even be able to embed single views inside other applications.**"

これは裏を返せば、**「個別Viewを他アプリへ埋め込む」はRerun公式自身が「まだ実装していない将来機能」と自認している**ということ。Motoliiが07-29〜08-29にかけてforkへ足してきた7 commit(`ViewBuilder::new_with_external_resolved`等)は、公式ロードマップ上でもまだ到達していない地点を自力で先取りしている。

**壁の所在(重要な訂正)**: 一次資料を読むと、壁は`re_renderer`自体にはほぼ無い。`re_renderer/README.md`が明記する通り「`re_viewer`やRerun chunk store libraryへの依存なし、standalone利用可能、公式example付き」。**壁があるのは一段上の`re_viewer_context`/`re_view_spatial`層**:

- `ViewerContext`(`viewer_context.rs`)が生きた`egui::Context`を型として必ず内包する
- `ViewClass::ui()`(view系の毎フレームエントリポイント)の引数が直接`egui::Ui`。この結合は**上流開発者自身が`TODO(wumpf)`コメントで「将来切り離したい」と認めている未解消の設計負債**(`view_class.rs`)
- カメラのpan/orbit/picking(`ui_3d.rs`)が`egui::Response`/`egui::InputState`に直結
- カメラ以外の状態変更(選択、時間制御等)は`SystemCommand::AppendToStore`という中央集権的なcommand-queue/blueprint-store経由が前提

**Motoliiへの意味**: 現行forkの7 commitは全て`re_renderer`本体または`re_view_spatial`のカメラ/テクスチャ層止まりで、egui結合の本丸である`ViewClass::ui()`本体には未到達。つまり**Motoliiは既にegui結合層を迂回する形で`re_renderer`をstandalone利用しており、front-end側のUI toolkit選定は、この壁とは独立した問題**(front-endにeguiを埋め込む必要は、Rerunの内部実装が理由では生じない)。ただし将来Rerunのpicking/選択UXを再利用したくなった場合、同じ迂回commitパターンをもう一段(`ui_3d.rs`相当)積む必要が高い確度で見込まれる。

出典: [ARCHITECTURE.md](https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md)、[re_renderer README](https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_renderer/README.md)、[Embed a Rerun Viewer](https://rerun.io/docs/howto/embed-rerun-viewer)、[rerun-docs index.md](https://github.com/rerun-io/rerun-docs/blob/main/docs/index.md)、`re_viewer_context/src/viewer_context.rs`・`view/view_class.rs`、`re_view_spatial/src/ui_3d.rs`(以上rerun-io/rerun main)、oshikaidesu/rerun commits `346a0b3 7cca401 856f597 483b855 252c9ce 037579e 71a3127`

## 1.8 Rerun自身のフロント切り分けをソースで確認(2026-08-29追加、利用者指示「Rerunのフロント部分はどう切り分けられるか」)

08-29時点でMotoliiがpinしている`oshikaidesu/rerun`(rev `346a0b3`)のチェックアウト(`~/.cargo/git/checkouts/rerun-bdb1f1ac6277bf7e/7cca401/`)を直接読んだ。

`crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs`(全文):

```rust
pub fn new_renderer_callback(
    view_builder: re_renderer::ViewBuilder,
    viewport: egui::Rect,
    clear_color: re_renderer::Rgba,
) -> egui::PaintCallback {
    egui_wgpu::Callback::new_paint_callback(viewport, ReRendererCallback { view_builder: Mutex::new(view_builder), clear_color })
}

impl egui_wgpu::CallbackTrait for ReRendererCallback {
    fn prepare(&self, _device, _queue, _screen_descriptor, _egui_encoder, paint_callback_resources) -> Vec<wgpu::CommandBuffer> {
        let ctx = paint_callback_resources.get::<re_renderer::RenderContext>().unwrap();
        self.view_builder.lock().draw(ctx, self.clear_color).map(|cb| vec![cb]).unwrap_or_default()
    }
    fn paint(&self, info: egui::PaintCallbackInfo, render_pass: &mut wgpu::RenderPass<'static>, paint_callback_resources) {
        let ctx = paint_callback_resources.get::<re_renderer::RenderContext>().unwrap();
        render_pass.set_viewport(...); // callback の viewport矩形へクリップ
        self.view_builder.lock().composite(ctx, render_pass); // egui の *同じ* render pass へ直接合成
    }
}
```

呼び出し側は`re_view_spatial/src/ui_3d.rs:416`の`ui.painter().add(gpu_bridge::new_renderer_callback(...))`。

**分かったこと**: Rerun自身のfront-endは「re_rendererが描いたtextureをegui側でimageとして貼る(`register_native_texture`)」構成では**ない**。`egui_wgpu::Callback`という公式拡張点(`egui-wgpu`が最初から持つAPI)を使い、egui側が今まさに描画中の**その同じ`wgpu::RenderPass`**の中へ、re_rendererが直接`composite()`する。Device・Queue・RenderPassそのものを共有し、textureの受け渡しすら発生しない。このコードは**Motoliiの現行依存木に既に存在する**(`re_viewer_context`クレート、`app/`側は未importなだけ)。

**含意**: OS レベルの共有面ブリッジ(IOSurface/DXGI/dma-buf、`2026-08-26-stage-zero-copy-makepad-fork-seam.md`の3室設計)が必要だったのは**Makepadが非wgpuだから**であって、「front-endを別に持つ」こと自体が払うべき代償ではなかった。front-end側が**wgpu-native**でありさえすれば(egui/dear-imgui-wgpu/yakui-wgpu問わず)、Rerun自身が本番で使っているこの`egui_wgpu::Callback`型の直接合成で繋がり、OS共有面もfork patchも原理上不要になる。この事実は§1(a)〜(d)の「re_rendererとのゼロコピー共有コスト」の評価に影響する——非wgpuの選択肢(Makepad)だけが特別に重い代償を払っている、という構図が明確になった。

egui-wgpu以外の候補(dear-imgui-wgpu、yakui-wgpu)が同種の`CallbackTrait`相当(「同じrender passに直接コマンドを積める」拡張点)を持つかは[未確認]——本節はegui-wgpu/`egui_wgpu::Callback`について一次資料(ソース)で確認した事実であり、他候補への一般化はまだ検証していない。

出典: `~/.cargo/git/checkouts/rerun-bdb1f1ac6277bf7e/7cca401/crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs`、`crates/viewer/re_view_spatial/src/ui_3d.rs:416`(いずれもMotolii `app/Cargo.toml`がpinするrev `346a0b3`のツリー内)

## 1.9 Rerun front層の全crateを機械検査 — egui非依存の流用先は皆無(2026-08-29追加、利用者指示「re_renderer以外にはないか」)

`crates/viewer/*/Cargo.toml`全件を`egui.workspace = true`形式(単純な`^egui`grepでは見落とす)まで含めて検査した。

結果: `re_renderer`/`re_renderer_examples`と、UI外の周辺crate(`re_gamepad`・`re_web_viewer_server`・`re_viewer_mcp`)を除き、**`crates/viewer/`配下は例外なく`egui`に依存する**。`View`の基底trait自体を持つ`re_view`、`re_viewport`、`re_time_panel`、`re_selection_panel`、`re_view_graph`/`re_view_map`/`re_view_tensor`/`re_view_bar_chart`、`re_context_menu`まで全て。

**結論**: Rerunが公開する「front」はegui一本であり、egui非依存で流用できる層は`re_renderer`以外に存在しない。egui採用を選ばない場合、Rerunから流用できるのは(既にMotoliiが行っている)`re_renderer`単体の利用が上限で、それ以上の追加流用先は無い。

## 2. 軸横断の事実サマリ(結論ではない)

- **ゼロコピー実機証拠の有無**: egui(2026-07-18、Apple M4実機)> React+direct wgpu(2026-08-06/07、ただしMotolii独自Stageでの検証でre_renderer統合ではない)> Makepad(3室設計は完了、mac版は「共有面0×0」で未接続、win/linux未着手)> Xilem/Masonry(未検証・未着手)
- **DSL学習コストの重さ**: Makepad(独自DSL+14 skill)が最重量。他3候補は素のRust/TS
- **移行コストの向き**: Makepad継続が最小(現状維持、ただしブリッジ完成という残債あり)。他3候補は全て「現行DSL実装を書き直す」規模の移行
- **「非wgpu」という事実**: Makepadだけが唯一、GPU API層そのものが`re_renderer`(wgpu)と別系統。他3候補(Xilem/Masonry、React+direct wgpu、egui)は全てwgpu上に建っており、原理上は同一`Device`共有が可能
- **本家不在の前例**: 「Makepadをwgpu化した前例」はゼロから作る話(2026-08-29確認)。一方「egui-wgpuで既存wgpu deviceを共有する」「Vello/MasonryをRerunと同じwgpu deviceで動かす」は、いずれもライブラリの公開APIレベルでは前例パターンが存在する(実プロジェクトでの`re_renderer`統合実績があるのはegui-wgpuの`WgpuSetup::Existing`パターンのみ、Masonryは未実証)

## 3. 到達点 — 2026-08-29終盤に利用者が到達した視点(結論ではない、次回入口)

§1系列の探索(4候補比較→GitHub新顔探索→wgpu29軸再検索→Rerun View埋め込みの壁→Rerunフロントのソース確認→全crate機械検査)を経て、利用者は次の裁定列を積み重ねた。上書きせず時系列で記録する。

1. **ゼロコピーは絶対に譲れない**(既定、[[motolii-zero-copy-is-non-negotiable]]の再確認)
2. **D&D(素材のBrowser→Timeline、レイヤーのTimeline→Stage等)を考えると、front-endに境界を作ってはならない** — 単一プロセス・単一入力系で全panelを描く構成でないと、07-21調査が指摘した「system WebView child + native wgpu sibling」「HTML5 DnD」「DOM/native pointer二重owner」と同種の壁に当たる
3. **egui は採用しない**(利用者裁定、理由は本書では未言語化——採否は利用者判断であって本書が代弁しない)
4. **Rerunの前提と実際に切り分けた**: 「Rerunが軸」は事実だが、Rerunが front として公開しているのは`re_renderer`単体のみ(§1.9)。`re_renderer`より上(`re_ui`・`gpu_bridge`・`ViewClass`)は全てegui依存であり、egui不採用の時点でこれらは流用対象から外れる——これは「Rerunを軽視する」ことではなく、「Rerunが実際に公開している境界がどこまでか」を正確に引き直した結果
5. **`re_renderer`自体がfrontに要求する契約を最小まで分解した**(§1.6〜1.8のソース調査から抽出):
   - 既存`wgpu::Device`/`Queue`を共有できること(fork `RenderContext::new_from_device`)
   - 描画結果を(a)`ViewBuilder::new_with_external_resolved`で用意した外部textureへ`draw()`、または(b)自分が握る`&mut wgpu::RenderPass`へ`composite()`で直接差し込む、のいずれかで受け取れること
   - frame loop・presentの所有権を持つこと
   
   **この契約には特定のUI toolkit名が一切現れない**。(b)の具体的な実装手段として`egui_wgpu::Callback`が存在するだけで、原理そのものはegui固有ではない
6. **「独自ウィジェットになる」(クリエイタソフトの前提)を踏まえると、この最小契約だけを満たす自作の薄いfront-end(winit+wgpuを直接持ち、`re_renderer`を`composite()`で同じrender passに差し込み、text shapingやa11yは`parley`/`AccessKit`のような単機能crateを個別に組む)が、既製フレームワーク(Makepad/egui/Dear ImGui/Xilem-Masonry/React複合)をどれも採用しない第5の方向として浮上した**。ウィジェットカタログ・DSL・reactive view treeといった「フレームワークの型」を一切引き継がない代わりに、入力・text shaping・IME・a11y・hot reloadを個別に部品選定・自作する責務をMotolii側が負う

**未決着(次回への引き継ぎ)**:
- 自作front方向のホットリロード解——素のRustコードである以上、Makepad型の「DSL部分だけ差し替え」は無く、汎用Rust hot-patchツール(`subsecond`/`hot-lib-reloader`/`dexterous_developer`等)の成熟度を未調査のまま残している(本書冒頭でこの調査に着手しかけたが、Rerun View埋め込みの壁調査に話が移り未実施)
- `parley`/`AccessKit`以外に、この自作front方向で要る部品(hit-test、focus管理、clipping、レイヤー合成順)の一覧化は未着手
- egui不採用の理由そのものは利用者裁定として記録するに留め、本書では代弁・推測しない

## 4. Dioxus Native(Blitz)probe — 賭けの実測(2026-08-29、利用者裁定「少し掛けてみるか」)

§3の後、利用者の追加整理(「LLMのポン出し品質はHTML/React系が実証済みで最強」「フロントとバックの接続がテストしきれずガタガタだったのはWebView/JSブリッジ由来」)から、§1.5でREJECT扱いだった**Dioxus Native(Blitz)**——HTMLエンジンのRust再実装、Vello/wgpu描画、WebViewもJSも無し——が再浮上。07-21のREJECT理由は「実browserを持つ計画がpre-alpha HTMLエンジンへ戻る理由は無い」であり、browserを捨てて境界を消したい今の前提では失効している、という読み直し。probeを`spikes/dioxus-native-probe/`(独立workspace、Blitz main `64eb2785`=当日HEADにpin)として実装・実測した。

| 検証項目 | 結果 |
|---|---|
| wgpu版整合 | **29.0.4単一**(app/Cargo.lockと同一版に自然解決、二重化なし)。Blitz workspace自体が`wgpu = "29"`固定 |
| HTML chrome | RSX(React風)+CSSで描画成立。日本語テキスト全箇所正常(system-fonts fallback、Makepadの`.notdef`豆腐は起きない) |
| 日本語IME | **composition・候補窓・確定・エコー往復まで実機動作**(利用者自身が入力して確認) |
| front↔back接続 | `oninput`→signal→再レンダ、ボタン→signal→mpsc→GPU widgetの2経路とも同一プロセス内の関数呼び出しのみ。ブリッジ無し |
| 共有wgpu pass | `blitz_dom::Widget`実装で`DeviceHandle`(共有Device/Queue)を受領、自前texture(`Rgba8Unorm`, RENDER_ATTACHMENT)へ描き`try_register_custom_resource`で登録、Velloが最終合成でsample。**頂点ごとに色が違う回転三角形が表示された**(Makepadの`DrawQuad`が1面1色に潰していた形そのもの)。CPU経路・OS共有面なし、リサイズ時のみtexture再作成 |
| RSXホットリロード | `dx serve`(dioxus-cli 0.7.10、cargo-binstallで8秒導入)でRSX文言編集→保存→**約1秒でlive反映**。リビルド・再起動なし、widget状態(回転角)・GPU資源保持 |
| **Rustロジックhot-patch** | `dx serve --hotpatch`(subsecond)で`tri_widget.rs`のclear色(関数本体)を編集→**1357msで適用**。dxログに再launch行なし=プロセス・Device・pipeline・texture全部保持のまま関数だけ差し替わった。**Makepad `--hot`(DSL再解釈のみ、Rust変更は再ビルド+再起動)を体験面で上回る** |
| ビルド | 冷3分10秒(stylo込み)/差分14秒/dx初回は自前`desktop-dev`プロファイルで別途冷ビルド398秒(一回きり) |
| CJK行分割 | `complex-scripts` feature必須(無いとICU4Xエラー連発)。点火で解消 |

**実測で確認できたデメリット(予告どおりの種類)**: `object`要素に`width:100%/height:100%`を指定したらBlitzでは0×0に潰れ、custom widgetは**layoutが0だと無言でスキップ**(エラーも警告も無し)——「LLMがフルブラウザ前提のCSSを書き、Blitzが黙って無視する」段差を初回から踏んだ。正解はexample同様`display: grid`のstretch。devtools不在のため、この種の原因特定はソースを読むしかなかった(`blitz-paint/src/lib.rs process_custom_widget_node`)。

**未検証のまま残る本丸**: この登録textureへ`re_renderer`の`ViewBuilder::new_with_external_resolved`で直接描かせるRerun統合(構造上は同じtextureを指すだけ)、D&D、負荷時性能、Blitz成熟度の縁(スクロール・テーブル・複雑なCSS)。

probe現物: `spikes/dioxus-native-probe/`(Cargo.toml / src/main.rs / tri_widget.rs / shader.wgsl / styles.css)。

## 5. Timeline見た目+入力probe(2026-08-29同日続行、利用者指示「probe技術で本体のタイムラインを作成する。まずは見た目」)

probeを三角形からTimeline本体の見た目に差し替えた。構成=**左レイヤー欄はHTML**(グループ字下げ・畳みボタン)、**右トラック面はcustom paint 1枚**(anyrender Sceneへのベクタ描画、DOMノード不使用)。配色・寸法は平面文法の正本([2026-08-19-flat-grammar-canon-revision.md](2026-08-19-flat-grammar-canon-revision.md): 面3段 `#141414/#1a1a1a/#222222`・hairline `#111111`・行格子20px・文字11px)をそのまま使用。

**利用者裁定: 「要件としては十二分に良い」(見た目合格)、「スクロールはかなり自然です」(入力合格)。**

| 項目 | 結果 |
|---|---|
| レイヤー行・目盛りゼブラ・キーフレームダイヤ・グループ解釈 | 全て描画成立。グループ行の帯は裁定272「尺は中身に従う」どおり子キーのmin..maxから導出 |
| 横zoom | wheelで**カーソル直下の時刻を固定点に**8〜600px/秒で伸縮+横パン。`Widget::handle_event`に`UiEvent::Wheel`が要素ローカル座標付きで届く(`blitz_traits::events`) |
| 入力判定の描画(利用者指示「入力判定のログを出しているか。そこを描画してくれれば恐らく勝てます」) | hover行ハイライト・カーソル追従縦線・キーへのhit-test(hover拡大/クリック選択=白ダイヤ+リング)を実装、stdoutに`PROBE room=input down t=…s hit=…`の判定ログ |
| **バックエンド比較(重要)** | 既定の`vello-hybrid`は**ウィンドウ移動中に前フレームの残像**(present/damage処理起因、我々のコード外)。**full `vello`(GPU compute)に切り替えたら解消**(利用者実機確認)。probeの採用featureは`vello`に固定した。注記: Blitz公式exampleのdefaultはhybrid側なので、full velloが非主流レーンである可能性は残る——上流の両backendの位置づけは今後確認 |
| releaseビルドの効能の限界 | 残像はreleaseでも消えなかった(利用者観察)——present層のアーキテクチャ問題はビルドプロファイルでは直らない、という切り分けが取れた |

**運用知見(dx serve)**: (1)`--hotpatch`はRust関数本体の差し替えを1.4〜2.6秒で行いGPU/プロセス状態を保持する。**新規ファイル追加はwatch対象に入らない**(起動後に作ったファイルの編集は拾われない)→ファイルを足したらdx再起動 (2)新moduleをhotpatchで入れた直後はcanvasが空のままになる状態不整合があり得る→再起動で解消 (3)dxは自前`desktop-dev`プロファイルで別途冷ビルドする(初回398秒、以後差分)。

**残課題**: ルーラー数字(canvas内テキスト=`draw_glyphs`+FontData配管)、畳みボタン▾/▸の豆腐(font-family指定で解消見込み)、パネル分離/ドック機構の設計、`re_renderer`統合probe(登録textureへ`new_with_external_resolved`で直接描く本丸)。

## 6. 本丸 — re_renderer統合probe成立(2026-08-29同日、利用者指示「本丸までいきましょう」)

`spikes/dioxus-native-probe`に`stage_widget.rs`を追加し、**Motoliiフォーク(`oshikaidesu/rerun` rev `346a0b3`、`app/Cargo.toml`と同一pin)の`re_renderer`を直接依存**に入れて、Stage相当の接続を実証した。

**構成**: `can_create_surfaces`でBlitzの共有`DeviceHandle`から`RenderContext::new_from_device`(裁定170の口)を建て、毎フレーム`ViewBuilder::new_with_external_resolved`(裁定256の口)でBlitz登録texture(`Rgba8UnormSrgb`)へ直接描画。手順は`motolii-compositor/src/presentable.rs`の写し、ただし部品目録調査(§2)でliveパスのバグと確定した`poll(wait_indefinitely)`は**意図的に写していない**。描画内容=`TestTriangleDrawData`+回転カメラ(`look_at_rh`円軌道)。レイアウト=Stage(上260px)+Timeline(下)のMotolii画面構成の縮図。

**結果: 成立。** `re_renderer-context-up`→`first-paint 1600x518`→`first-submit-ok`、検証エラーゼロ、虹色三角形が回転カメラで生描画され、下のTimeline/HTMLレイヤー欄と同居。**同一プロセス・同一wgpu Device・OS共有面なしでRerunフォーク→Blitz合成が接続された** = 裁定251のゼロコピー契約はMakepadのIOSurfaceブリッジ(3室・fork 2本・win/linux未着手)無しで満たせる。

**途中で踏んだ罠2つ(いずれも将来の製品実装で再現しうる)**:
1. **無音の全滅**: 最初のtextureに`COPY_SRC`が無く、wgpu検証エラーがvelloのframe submit全体を巻き添えにして**HTMLごと窓が真っ暗**になった。しかもエラーは出なかった——`RenderContext::new_from_device`が共有deviceの`on_uncaptured_error`ハンドラを乗っ取りre_logへ流すため、**re_logのロガー未初期化だと検証エラーが無音で消える**。`re_log::setup_logging()`(feature `setup`)を立てた瞬間に1発で見えた。教訓: **Blitz+re_renderer同居ではre_log初期化が観測の前提条件**
2. **anyrender_velloの外部texture取り込みはsampleではなくGPU copy**(`COPY_SRC`必須が動かぬ証拠)。合成経路に毎フレームGPU texture copyが1回入る——egui-wgpuの`register_native_texture`(直接sample)との差であり、裁定251の字義(「最終TextureからのGPU blit禁止」)に照らすと**利用者裁定が要る**。ただしanyrenderはMITで、copy→sampleの薄いpatchは裁定「wrapper over hack」の通常の型の規模(MakepadのOS共有面ブリッジとは階級が違う)

**別途確定した事実**: リサイズのガタつきは**未解決のまま**(velloバックエンドで直ったのは移動中の残像のみ)。Blitz上流に「Very slow resizing」というopen issueが既に存在し、既知問題の可能性が高い。

## 7. 実Document+再生+書き戻し(2026-08-29同日最終、利用者指示「テストのドキュメントを読み込み、再生ボタンを、canvasから値の書き込みを」)

probeに`motolii-fixture`/`motolii-store`をpath依存で入れ(自作の写経ゼロ)、3点を同日中に通した。

1. **実Document**: `motolii_fixture::build()`(--fixtureのMV風15層・Bezierイージング入り)を製品API(`view.layers()/attrs()/meta()/track()`)で読み、レイヤー名・timing帯・キー時刻をTimelineへ投影。日本語レイヤー名15本が実窓に表示された
2. **再生**: 共有`Clock`(Arc)をTimeline/Stage両widgetが読む。▶ボタンでプレイヘッドが滑走し、Stageのカメラは**サビ歌詞position(Bezier)とタイトルロゴopacityを`KeyframeTrack::eval()`(motolii-evalの製品補間コード)でサブフレーム評価**した値で駆動。利用者評: 「イージング以前に3Dの回転体が居る時点でかなり保証されている」
3. **書き戻し**: 帯ドラッグ→ドラッグ中はtransient表示のみ→release時に`Intent::SetTiming`を1回`doc.apply()`→`view`再読みで定着(D2規律と同じ形)。キーは動かさない(裁定272: キーフレーム時刻はcomp絶対)。実ログ: `applied SetTiming start 0->68`等、利用者自身のドラッグで複数commit成立

これで front↔back の読み(§5)・評価(motolii-eval)・書き(Intent/apply)の3方向すべてが、実Document・製品コード経由で1つの窓の中で閉じた。probe構成は6ファイル+`playback.rs`(`spikes/dioxus-native-probe/`)。
