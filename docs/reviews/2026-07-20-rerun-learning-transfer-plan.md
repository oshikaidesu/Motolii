# Rerun → Motolii 学習・転移計画（2026-07-20）

ステータス: **決定**。Rerunをegui製品実装の主要先例とし、外観だけでなくshell、時間面、GPU viewport、選択、実行系、試験系を層別に学ぶ。本文書は「何を、どの順で、どの境界まで習うか」の運用正本であり、個別crateの依存追加・vendoring・forkを単独では許可しない。

決定の背景と方向決定時のspot auditは[Rerun先例調査](2026-07-20-rerun-prior-art-survey.md)、固定commitのpackage全量と調査入口は[Rerun source asset inventory](2026-07-20-rerun-source-asset-inventory.md)を併読する。Motoliiの状態所有・command・thread・単位・toolkit隔離は[GR-UI](2026-07-14-m3-ui-boundary-prevention.md)、製品UIの意味は[M3仕様](../specs/M3-ui-integration.md)と[UI操作言語](../ui-interaction-language.md)、見た目は[UI視覚言語](../ui-visual-language.md)が優先する。

## 1. 製品命題

Rerunは、時間を持つ大量の技術データを複数View、selection、density、GPU sceneとして読める製品へまとめている。Motoliiはその構造を、映像制作者が直接操作し、Undoでき、作品として保存できるポップな制作言語へ再翻訳する。

```text
Reactモック
└─ Motoliiとして何を見せ、どう操作させるか

Rerun
└─ eguiで高密度viewerをどう製品として成立させたか

Motolii domain / command / renderer
└─ 作品意味、編集、Undo、preview/export同一性
```

Rerunの画面、Entity/Blueprint/Chunk Store語彙、保存形式を複製しない。ReactモックのCSS値やRerunのtoken値も契約ではない。借りるのは、問題分割、componentの責任、GPU接合、時間密度表現、cache、試験方法である。

## 2. 調査baseline

本計画は2026-07-20にRerun公式リポジトリのcommit [`954bf95a`](https://github.com/rerun-io/rerun/commit/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e)（2026-07-19）を固定して作成した。方向決定時は代表資産のspot auditに留まっていたため、package-level全量棚卸しと監査限界を[source asset inventory](2026-07-20-rerun-source-asset-inventory.md)へ分離した。

| 項目 | baseline |
|---|---|
| Rerun workspace | `0.35.0-alpha.1+dev` |
| UI | egui / eframe / egui-wgpu 0.35 |
| panel | egui_tiles 0.16 |
| GPU | wgpu 29 |
| repository | MIT OR Apache-2.0 |
| `re_ui` | `(MIT OR Apache-2.0) AND OFL-1.1`。font/icon資産の個別確認が必要 |
| 安定release anchor | 0.34.1 = tag commit `4efb18f`(2026-07-07)。同じegui 0.35 / egui_tiles 0.16 / wgpu 29構成で**出荷済み**、`[patch.crates-io]`全コメントアウト。alpha main監査との差分基準([inventory §2.2](2026-07-20-rerun-source-asset-inventory.md)) |

Rerunの全`re_*`crateは独立したsemver安定部品とは見なさない。学習時はcommitを固定し、後続調査で別commitを読む場合は差分を記録する。

RR-1〜9のレーンは調査routeであり、inventoryにあるpackageや候補分類をそのまま実装チケットへ変換しない。個別assetはfile/API、dependency closure、license、Motolii側gap/oracle、反対側レビューが揃ってから§6の分類を確定する。

## 3. 習う順序

優先順は「見た目」だけではない。Motoliiの完成を支える順に、次の10レーンへ分解する。

```text
RR-0 Source/License/Version
 ├─ RR-1 re_ui / component system
 ├─ RR-2 Viewport / Blueprint / panel projection
 ├─ RR-3 Time Panel / density / ruler
 ├─ RR-4 Selection / context / direct manipulation
 ├─ RR-5 egui-wgpu bridge / renderer lifecycle
 ├─ RR-6 View execution / cache / parallelism
 ├─ RR-7 Video / image / media presentation
 └─ RR-8 Snapshot / catalog / performance evidence
          ↓
 RR-9 Motolii vertical sliceで統合審判
```

### RR-0 — Source、license、version境界

| 読む場所 | 習うこと | 成果物 |
|---|---|---|
| [workspace Cargo.toml](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/Cargo.toml)、各crate `Cargo.toml`、release notes | egui/wgpu結合、feature、license、ロックステップ更新範囲 | commit付きasset inventory、依存候補／vendoring候補／設計参考のみの分類 |

合格条件:

- source URLを`main`だけで記録せず、監査commitを持つ
- 監査はmain固定commitと**安定release tag(現時点0.34.1=`4efb18f`)の二点anchor**で行い、出荷実績の主張はtag側、最新実装の観察はmain側と使い分ける([inventory §2.2](2026-07-20-rerun-source-asset-inventory.md))
- code、font、icon、shaderごとのlicenseを分離する
- Motoliiの既存wgpu/Vello/egui版との重複・feature差を列挙する
- Rerun更新への追従を自動前提にしない(定量入力: minor概ね月次+随時patch、`re_ui`累計278版、"Expect breaking changes!")
- [source asset inventory](2026-07-20-rerun-source-asset-inventory.md)のpackage全量、LFS制約、重点候補を入口にし、package名だけで採否を決めない

### RR-1 — `re_ui`とcomponent system

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`re_ui`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui) | design tokens、dark/light theme、button、list item、property row、form、modal、menu、notification、DnD、command palette、filter、text edit、time value | Browser、Inspector、Settings、popup、Status、検索、Effect/Asset listの共通component |
| [`re_ui_example`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui/examples/re_ui_example.rs) | component catalogを実行可能な一枚へ集める方法 | React Storybookと対応するegui catalog |
| [`re_component_ui`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_component_ui) | 型ごとの表示・編集UI登録、list itemのwide/narrow variation、snapshot matrix | `NodeDesc`からHostが作るparameter panelのcomponent registry |

最初に習うcomponent:

1. `DesignTokens`とtheme読込
2. `ListItem`とlabel/property/custom content
3. button/menu/modal/text edit
4. drag-and-drop feedback
5. filter/fuzzy search/command palette
6. dense tableとform
7. notification/loading/error表現

持ち込まないもの:

- Rerunの色値、font、iconをそのままMotolii tokenへ固定
- `re_ui`型を`motolii-ui`外へ公開
- Rerun component data registryをplugin公開契約へ転用
- component単位の即席raw color、spacing、radius

RR-1の出口は「Rerun風画面」ではない。同じMotolii fixtureをReact catalogとegui catalogで並べ、情報階層、状態、contrast、密度を比較できる状態である。

### RR-2 — Viewport、Blueprint、可変panel

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`re_viewport`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewport) | active Viewだけの実行、View state、loading、highlight、viewport UI | Preview、Graph、Browser等を同じshellへ載せる実行境界 |
| [`re_viewport_blueprint`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewport_blueprint) | 安定したView/Container記述から毎frame `egui_tiles::Tree`を構築し、変更をdeferred commandで反映 | Motolii所有layout model → `egui_tiles` runtime投影 |
| [`ViewportBlueprint`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewport_blueprint/src/viewport_blueprint.rs) | frame中immutable、末尾でcommand適用、auto-layoutから手動layoutへの遷移 | panel drag中の一時状態と確定layout更新の分離 |

転移条件:

- Motolii layout modelはDocumentから独立したWorkspace-session/User settings候補
- `egui_tiles::Tree`、`TileId`、Rerun Blueprint schemaを保存正本にしない
- layout変更を映像編集Undoへ混ぜない
- panelのsplit/tab/hide/restore/maximizeと別monitor previewを別々に試験する
- 永続互換はRerunを先例にしない: Rerun自身のデータ互換保証は「直前versionの`.rrd`を開ける」のみで([ARCHITECTURE.md @0.34.1](https://github.com/rerun-io/rerun/blob/0.34.1/ARCHITECTURE.md))、`.rbl` blueprintのversioning/migration方針は公式docに記載がない。Motolii layout modelの保存寿命・互換はMotolii側で別途決める

Blueprintの「store time travelによるUndo」は、MotoliiのD2 command/journalへ輸入しない。学ぶのはruntime投影とdeferred mutationである。

### RR-3 — Time Panel、time ruler、density

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`re_time_panel`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_time_panel) | 時間axis、selection、transport UI、treeと時間表示の同期 | Timeline / Scoreの時間navigation、playhead、visible range |
| [`data_density_graph.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_time_panel/src/data_density_graph.rs) | UIピクセル幅に比例したbucket、range分配、blur、前frame最大値による平滑な正規化 | zoom-out時のclip/key/effect/readiness密度 |
| [`re_time_ruler`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_time_ruler) | 時間範囲と目盛りの独立component | RationalTimeを読むruler、zoom/pan |

密度描画は、100,000 keyを全部描く代用品ではなくsemantic zoomの第一段である。

| zoom | 表示 |
|---|---|
| 遠景 | 密度、区間、warning/readinessの大域形状 |
| 中景 | packed bar、主要key cluster、Effect IN/OUT |
| 近景 | 個別clip/key、handle、hit target |

Rerun time panelはrecording閲覧面であり、Motoliiのclip/keyframe編集意味を持たない。次は自前である。

- trim、move、group、automation、easing
- 1 gesture = 1 command / 1 Undo
- fixed Track/Laneを所有者にしないpacking
- Group、Effect、Depth Rail、Character Score
- edit conflict、generation、stale表示

### RR-4 — Selection、context、picking、outline

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| `re_viewer_context`のselection/highlight、`re_selection_panel`、`re_context_menu` | 複数surfaceが同じ選択を投影し、context actionを集約する方法 | Browser / Timeline / Stage / Inspectorのstable ID選択 |
| [`re_view_spatial::picking`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_view_spatial/src/picking.rs) | pointer→GPU/scene hit→selection UIの流れ | Stage direct manipulationの比較材料 |
| [`picking_layer.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_renderer/src/draw_phases/picking_layer.rs)、[`outlines.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_renderer/src/draw_phases/outlines.rs) | picking ID、outline mask/composite、非同期readback | M5 viewportの選択輪郭候補 |

選択の正本はMotoliiのWorkspace-session/Transient stateであり、Rerun `Item`やEntity pathを公開型へしない。picking async readbackはM5のreadback規律と未裁定なので、RR-4だけで採用しない。

### RR-5 — egui↔wgpu bridgeとrenderer lifecycle

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`re_renderer_callback.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs) | `egui_wgpu::CallbackTrait`のprepare/paint、resource共有、viewport/scissor、ViewBuilder composite | `register_native_texture`方式との比較、Stage overlayとclipの扱い |
| [`re_renderer`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_renderer) | dynamic data、resource reuse/cache、lazy loading、multi-view/camera、hot shader reload | Preview/Stage/M5で再利用できるrenderer責任の切り方 |
| [`re_renderer_examples`](https://github.com/rerun-io/rerun/tree/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_renderer_examples) | standalone rendererの最小配線、multiview、outline | Motolii fixtureでの比較spike |

比較する二方式:

1. Motoliiの長寿命display textureを`register_native_texture`でsample
2. egui paint callback内でMotolii/Rerun型rendererを直接composite

`register_native_texture`は既存TextureViewをsampleするためCPU copyもGPU texture copyも前提にしない。Callback方式を「コピー1回削減」と表現せず、offscreen target、overlay、clipping、lifecycle、エラー隔離、計測容易性で比較する。

`re_renderer`丸ごとの採用はM3 UIの判断ではない。M3 preview接合、M5 3D renderer、Vello 2D rendererを一つの「便利な共通化」で束ねず、それぞれの契約境界で裁定する。

### RR-6 — View execution、cache、並列性

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`system_execution.rs`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_viewport/src/system_execution.rs) | active tileだけを対象に、once-per-frame contextとper-view visualizerを分け、Rayonで並列実行 | immutable snapshotからvisible surface用の投影を生成 |
| `re_view_spatial/src/caches`、renderer resource pools | query/cache key、mesh/resource reuse、lazy upload | Motolii cacheとUI投影cacheの境界 |
| Rerun profiling scope | UI、query、wait、GPU準備を別区間で測る | G0-4のp50/p95、idle、scrub、large fixture証跡 |

禁止する誤転移:

- Rerunの毎frame queryをMotolii Documentへ直結
- UI threadでMotolii frameをrender
- Rerun cache keyをDocument schemaへ保存
- worker完了を待ってegui frameをblock
- active panelだけに作品評価の正しさを依存

MotoliiではDocument snapshot、render/eval worker、latest-value mailbox、generation破棄が正本である。Rerunから学ぶのは可視Viewの投影とcacheであり、単一writerを置換しない。

### RR-7 — Video、image、media presentation

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| `re_view_spatial/src/visualizers/video`、video snapshot tests | codec別frame表示、開始前/終了後、非frame境界、image format、magnification filter | preview fixture、stale/end状態、codec/image format試験 |
| `re_renderer`のtexture/resource handling | texture upload、format、filter、composite | VRAM display poolとmedia presentation |

Rerunは再生・可視化の先例であり、Motoliiの編集、decode sidecar、audio master clock、export、色変換の正本ではない。`re_video`やFFmpeg構成をそのままD5へ持ち込まず、既存M2/D5契約との差分だけを抽出する。

なお`re_video`のH.264経路は**MotoliiのB-2本命と同一の`ffmpeg-sidecar`(CLI)**である(0.34.1 `Cargo.toml`の`ffmpeg = ["dep:ffmpeg-sidecar"]`、[inventory §5.9](2026-07-20-rerun-source-asset-inventory.md))。これはffmpeg-sidecar採用判断の独立収束事例として記録するが、Rerunのdecode contract・色変換・presentation timingをD5/M2既決の代わりにしない。

### RR-8 — Catalog、snapshot、性能証拠

| 読む場所 | 習うこと | Motoliiへの再翻訳 |
|---|---|---|
| [`re_ui::testing`](https://github.com/rerun-io/rerun/blob/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e/crates/viewer/re_ui/src/testing.rs)、`egui_kittest` | wgpu harness、UIと3Dで異なるpixel threshold | component catalogとGPU viewport snapshot |
| `re_component_ui/tests/snapshots` | component×layout×themeのmatrix | `NodeDesc`自動panelの状態matrix |
| `re_view_spatial/tests` | 2D/3D、透明、draw order、video、selectionの画像fixture | Stage/Output Frame/selection/alpha/video golden |

Motoliiで固定する試験層:

1. 純layout/hit-test test（window/eguiなし）
2. component catalog snapshot（dark/light、state、wide/narrow）
3. 統合reference screen（Reactモックとの比較）
4. GPU viewport golden（alpha、selection、frame内外）
5. interaction sequence（IME、drag、Undo、focus）
6. performance raw evidence（1,000 clips、100,000 keys、idle）

Rerunのpixel thresholdをそのままコピーしない。Motoliiの基準機、GPU、font、許容差をG0-4/G0-6で決める。

### RR-9 — Motolii vertical slice

RR-1〜8を個別に写経して終えず、同じfixtureへ統合して転移の成否を判定する。

対象はReact統合モックの一画面で、最低限次を含む。

- Browserのdense list、検索、selection
- Inspectorの型付きparameter row
- 可変panelとtab
- 640×360以上の同一device preview
- packed Timelineの遠景density、中景bar、近景key
- Timeline / Stage / Inspector間の同一stable ID選択
- popup、drag preview、notification、loading/stale状態
- dark/light
- 日本語IME

RR-9はM3入場後の製品縦切りであり、本計画の作成だけでは着手しない。

## 4. フェーズと既存タスクへの接続

RerunはM3だけの先例ではない。ただしフェーズごとに権限が異なる。M1/M2では既決基盤の監査・反証が主であり、Rerunを理由に締結済み契約を再設計しない。

| Motoliiフェーズ | Rerunの役割 | 許される出口 | 禁止する短絡 |
|---|---|---|---|
| M1 GPU / render基盤 | wgpu device共有、texture/buffer/pipeline、renderer lifecycle、UI compositor接合の監査材料 | M3 viewportまたはM5 GPU scene側の不足証跡。基盤変更が必要なら独立した解凍候補 | `re_renderer`型で既存core/render境界を置換 |
| M2 Document / 時間 / 所有 | recordingとview、正本と派生物、時間面、selection、layout、cacheの所有分離を反証 | Document外のWorkspace-session / Transient投影と、既存単一writer境界の確認 | Entity、Blueprint、store、Time Panel意味をDocumentへ輸入 |
| M3 UI | component、shell、panel投影、時間密度、selection、viewport接合の具体的転移 | RR-1〜9の個別裁定とMotolii fixture | Rerun viewerを編集UIの仕様正本にする |
| M4 cache / analysis | cache、profiling、snapshot、性能証拠の作り方 | 既存cache keyと性能審判への限定転移 | Rerun query/cache modelでDocument評価を置換 |
| M5 GPU scene / 3D | picking、outline、compositor、renderer resource lifecycleの先例 | M5意味論で裁定したGPU実装候補 | Rerunの3D、遮蔽、readback意味を無裁定輸入 |

M1/M2監査で不一致を見つけても、その場で旧フェーズを改修しない。対象spec、実コード証跡、破れている既決、最小の解凍範囲を別文書へ戻し、通常の仕様改訂・反対側レビューを通す。Rerunで別方式が動いていること自体は解凍理由にならない。

| 学習レーン | 主なM3/M5接続先 | 接続時に追加する審判 |
|---|---|---|
| RR-0 | M3入場PR、依存監査 | commit/license/version inventory |
| RR-1 | U0e-1/2/3、U0b、U4a、G0-6H | React↔egui component map、raw token拒否 |
| RR-2 | U1a、U1e | layout model非Document、split/tab/hide/restore |
| RR-3 | U3a、U3b、UI Score | semantic zoom、densityとindividual hit-testの切替 |
| RR-4 | U0b、U2b、U3b、U1f、M5 | stable ID、選択正本一つ、readback裁定 |
| RR-5 | U1a、U1b、U1f、M5 | 同一device、CPU bridgeなし、resource lifecycle |
| RR-6 | U1b、U1c、U3a、U3f | non-blocking、generation、visible range性能 |
| RR-7 | D5、U5、U6 | audio clock不変、preview/export分岐禁止 |
| RR-8 | U0e、U1c、U3a、G0-4、G0-6H | catalog/snapshot/interaction/perfの証跡分離 |
| RR-9 | M3最初の統合縦切り | 全停止線＋React referenceとの人間比較 |

既存タスクIDの完了条件は、本表だけで黙って変更しない。各タスクへ採用する時に仕様を改訂し、GR-UI審判割当表へ追加する。

## 5. 入場前と入場後

M3製品実装停止中でも可能:

- commit固定source読解
- asset/license/version inventory
- React component ↔ Rerun component ↔ Motolii taskの対応表
- アルゴリズム、責任分割、試験方法のobservation
- 反対側レビュー

この段階のinventoryに付した`候補分類`は観察であり、§6の裁定済み分類ではない。

M3入場PRと個別ゲート後だけ可能:

- workspace依存への`re_*`追加
- `re_ui` code/font/iconのvendoring
- Rerun codeを基にしたMotolii component実装
- `CallbackTrait`経路への変更
- `re_renderer` fork/部分移植
- product fixture/goldenの採択

## 6. 採り方の分類

各資産は必ず一つに分類する。

| 分類 | 意味 | 例 |
|---|---|---|
| `DEPEND` | crateを通常依存として使う | 公開APIと更新方針がMotoliiに適合する場合のみ |
| `VENDOR` | 必要な実装をMotolii所有へ移し、licenseと由来を保持 | `re_ui`の限定component候補 |
| `PORT` | アルゴリズムをMotolii型へ移植 | density bucket/blur候補 |
| `PATTERN` | 責任分割・試験方法だけを学ぶ | Blueprint投影、parallel View execution |
| `REJECT` | 意味・依存・保守費が合わず持ち込まない | Rerun store schema、recording編集への誤転用 |

`VENDOR`を既定にしない。公開crate依存で十分か、数十行のMotolii自作が小さいか、Rerun内部結合が強いかを比較する。逆に「自作の方が綺麗そう」という理由だけで実戦済みcomponentを再発明しない。

## 7. 必須の反対側レビュー

個別採用前に最低限次を反証する。

1. Rerunの問題がMotoliiにも本当に存在するか
2. Reactモックの要求をRerunの画面へ寄せて失っていないか
3. `re_ui`追従費が限定vendoringより小さいか(定量入力: minor概ね月次+随時patch、`re_ui`累計278版、semver約束なし)
4. Vello 2Dと`re_renderer`の二本立てが必要か
5. immediate modeの毎frame計算をcacheで隠しただけではないか
6. CJK/IME、編集Undo、100,000 keysというRerunが証明していない領域を補えているか(CJKはRerun自身が表示すら未解決 — [#12770](https://github.com/rerun-io/rerun/issues/12770)、blocked/egui)
7. Rerun CTOがegui作者である単一実例依存を、Motolii fixtureで再現できたか
8. upstream更新停止時にMotoliiが固定版を保守できる範囲か

## 8. 停止線

- RerunのEntity、Chunk Store、Blueprint schema、View classをDocument・plugin契約・公開APIへ出さない
- package名または[source asset inventory](2026-07-20-rerun-source-asset-inventory.md)の候補分類だけで依存・vendoring・移植を発注しない
- `re_ui`、`egui_tiles`、`re_renderer`のserde形をUser settings正本へしない
- recording閲覧用Time Panelを編集Timelineとしてそのまま採用しない
- `re_renderer`導入を理由にMotoliiのVello採用、色変換一元化、preview/export同一関数を崩さない
- GPU pickingを理由に同期readbackを入れない。async方式もM5裁定前は採用しない
- Rerunのcache、thread、query modelでMotoliiの単一writer/latest mailbox/generationを置換しない
- Rerunの見た目を理由にReactモックの操作決定やMotolii visual languageを黙って変更しない
- GPL/AGPLの第三者コードをRerun周辺の類似例から混入させない

## 9. Rerun参照を発注へ入れる強制動線

この節は助言ではなく、Rerunを参照する全発注の入口・実装・検収を拘束する。RerunはMotoliiの未知を減らす実装証拠であり、製品の主語、仕様、完成条件を供給しない。

```text
Motoliiの仕様ID・決定・完成条件
  ↓
現行コードで未成立の事実と再現証跡
  ↓
Rerun固定commitの対象file/APIが証明する範囲
  ↓
DEPEND / VENDOR / PORT / PATTERN / REJECT裁定
  ↓
Motolii境界内の変更許可とSTOP条件
  ↓
Motolii fixture / testによる実装・検収
```

逆向きの`Rerun asset → Motolii要件 → 公開契約`は通行禁止とする。

### 9.1 入口: Rerunを読む前に固定する

1. [決定逆引き台帳](../decision-index.md)を対象語で引き、対象spec ID、既決、未決、停止線を列挙する。
2. 現行コードと既存testを読み、「何が未成立か」を再現コマンド、型、call path、fixtureで示す。Rerunとの違いだけをgapと呼ばない。
3. 既存Motolii helper、crate、公開境界で小さく達成できる案を検索する。Rerun利用を既定にしない。
4. ここまでで目的とgapが成立しない場合、Rerun調査または発注へ進まない。

### 9.2 先例: Rerunに許す役割を限定する

5. §2の固定commitと対象file/crateを示し、そのassetが実際に証明する範囲と、Motoliiでは証明していない範囲を分ける。
6. §6の分類を一つ裁定する。未裁定ならread-only調査までで停止し、依存追加、vendoring、portを発注しない。
7. M1〜M5のどのフェーズへ関与するかを§4から選び、既存停止線と入場条件を確認する。

### 9.3 発注: 仕様と先例を同じ欄へ混ぜない

Rerunを一度でも根拠・再利用箇所・変更案に含める発注書は、次の6ラベルをこの順序で持つ。

| 必須ラベル | 書く内容 | 書いてはいけないこと |
|---|---|---|
| `MOTOLII AUTHORITY` | spec ID、決定、公開契約、完成条件 | Rerunの画面・型・crateを要件として記述 |
| `CODE FACT GAP` | 未成立の現行コード事実、再現証跡 | 「Rerunと違う」「Rerunにある」だけのgap |
| `RERUN EVIDENCE` | 固定commit、packageだけでなく対象file/API、監査済み範囲、証明範囲と非証明範囲 | Rerunの設計やinventoryの候補分類をMotoliiの採用決定として記述 |
| `TRANSFER CLASS` | 裁定済みの`DEPEND / VENDOR / PORT / PATTERN / REJECT` | 実装担当に分類を委任 |
| `TRANSFER LIMIT` | 変更許可file、持込禁止型・状態・意味、既存境界案 | 「必要なら共通化」「適宜API追加」 |
| `MOTOLII ORACLE` | Motolii fixture、負例、test、性能・人間比較 | Rerunとの外観・構造類似を合格条件にする |

通常の発注必須項目である目的、非目標、再利用箇所、STOP条件、必須負例、実行コマンドも省略しない。上表はそれらを置換せず、Rerun参照の権限を限定する追加欄である。

Codex事前審査は次を順に確認し、一つでもNoなら`CODEX PRECHECK: APPROVED`を書かない。

1. Rerunを削除して読んでも、Motoliiの目的と完成条件が完全か。
2. `CODE FACT GAP`は現行コード証跡で再現できるか。
3. asset分類とフェーズ入場条件は既に裁定済みか。
4. Rerun内部型を使わなくてもMotoliiの公開契約が閉じているか。
5. 合否はMotolii oracleだけで判定できるか。

### 9.4 実装中のSTOP

次のどれかが起きたら、Composerは代替設計や仕様変更を行わず`ORDER: STOP`で戻す。

- Rerun内部構造を採らないと既存Motolii契約では実装不能に見える
- 未裁定asset、別crate、font、icon、shader、licenseを追加で持ち込む必要がある
- 公開API、Document、plugin契約、serde、User settings正本の変更が必要
- Rerunに無いMotolii固有要件、負例、IME、Undo、非blocking、Preview / Export契約を削る必要がある
- Rerunのsnapshotや見た目へ合わせるため既存test、golden、期待値を変更したくなった
- 指定分類を越えて`PATTERN→PORT`、`PORT→VENDOR/DEPEND`へ広げたくなった

STOP後に別backendで仕様を補完して実装を続けない。CodexがMotolii正本へ戻り、発注書差し戻し、個別裁定、仕様改訂、作業中止のいずれかを選ぶ。

### 9.5 検収と統合

Grok検収とCodex統合判断では、テスト緑に加えて次を確認する。

1. 6ラベルと実差分が一致し、変更file・依存・分類が拡大していない。
2. RerunのEntity、Blueprint、store、cache key、View class、serde、UI tokenが公開型・Document・plugin契約へ漏れていない。
3. `DEPEND / VENDOR / PORT`ではcommit、license、由来、改変範囲が追跡できる。
4. Motolii固有の負例、Undo、IME、非blocking、色、座標、Preview / Export、plugin純関数の該当審判が残っている。
5. Rerunへの類似ではなく`MOTOLII ORACLE`が合格している。

Rerun sourceとの比較結果は補助証拠であり、Motolii fixture/testを置換しない。採用しなかった資産と理由も残し、後続発注が同じassetを無裁定で再導入しないようにする。

### 9.6 必須負例

次の発注書はprepare段階で差し戻す。

- 「RerunのTime PanelをMotoliiへ実装する」のようにRerun assetが目的になっている
- spec IDやコードgapがなく「Rerun風」「Rerunベース」を完成条件にしている
- `re_*`依存、Blueprint field、Entity path、Rerun cache keyを実装担当の判断で追加できる
- Rerunに存在しないためMotoliiの編集Undo、Vism/plugin拡張、IME、audio clock等を非目標へ落としている
- Rerun snapshotへ合わせるためReact reference、意味論golden、既存test期待値の変更を許している
- Composerへ分類、公開API、Document状態、代替設計の選択を委任している

## 10. 完了の定義

本計画は文書を読んだだけでは完了しない。次の順に証跡が揃った時に段階完了とする。

1. RR-0 inventoryと反対側レビュー
2. RR-1〜8それぞれの`DEPEND/VENDOR/PORT/PATTERN/REJECT`判定
3. 判定を既存M3/M5タスクの完了条件へ個別反映
4. RR-9統合fixtureを同一device・非blockingで実行
5. React referenceとの視覚・操作比較
6. CJK/IME、1,000 clips、100,000 keys、idle、resize/minimize/restoreの証跡
7. 不採用資産と理由を残し、「Rerunを使ったから完成」と短絡しない

最終目的はRerun依存を増やすことではない。Rerunが既に払った再発明の費用を学び、Motoliiが映像制作固有の意味、操作、楽しさへ資源を集中できる状態を作ることである。
