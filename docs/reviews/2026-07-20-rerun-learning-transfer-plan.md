# Rerun → Motolii 学習・転移計画（2026-07-20）

ステータス: **決定**。Rerunをegui製品実装の主要先例とし、外観だけでなくshell、時間面、GPU viewport、選択、実行系、試験系を層別に学ぶ。本文書は「何を、どの順で、どの境界まで習うか」の運用正本であり、個別crateの依存追加・vendoring・forkを単独では許可しない。

決定の背景と検証済み事実は[Rerun先例調査](2026-07-20-rerun-prior-art-survey.md)を併読する。Motoliiの状態所有・command・thread・単位・toolkit隔離は[GR-UI](2026-07-14-m3-ui-boundary-prevention.md)、製品UIの意味は[M3仕様](../specs/M3-ui-integration.md)と[UI操作言語](../ui-interaction-language.md)、見た目は[UI視覚言語](../ui-visual-language.md)が優先する。

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

本計画は2026-07-20にRerun公式リポジトリのcommit [`954bf95a`](https://github.com/rerun-io/rerun/commit/954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e)（2026-07-19）をsource archiveで監査した結果に基づく。

| 項目 | baseline |
|---|---|
| Rerun workspace | `0.35.0-alpha.1+dev` |
| UI | egui / eframe / egui-wgpu 0.35 |
| panel | egui_tiles 0.16 |
| GPU | wgpu 29 |
| repository | MIT OR Apache-2.0 |
| `re_ui` | `(MIT OR Apache-2.0) AND OFL-1.1`。font/icon資産の個別確認が必要 |

Rerunの全`re_*`crateは独立したsemver安定部品とは見なさない。学習時はcommitを固定し、後続調査で別commitを読む場合は差分を記録する。

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
- code、font、icon、shaderごとのlicenseを分離する
- Motoliiの既存wgpu/Vello/egui版との重複・feature差を列挙する
- Rerun更新への追従を自動前提にしない

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

## 4. 既存タスクへの接続

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
3. `re_ui`追従費が限定vendoringより小さいか
4. Vello 2Dと`re_renderer`の二本立てが必要か
5. immediate modeの毎frame計算をcacheで隠しただけではないか
6. CJK/IME、編集Undo、100,000 keysというRerunが証明していない領域を補えているか
7. Rerun CTOがegui作者である単一実例依存を、Motolii fixtureで再現できたか
8. upstream更新停止時にMotoliiが固定版を保守できる範囲か

## 8. 停止線

- RerunのEntity、Chunk Store、Blueprint schema、View classをDocument・plugin契約・公開APIへ出さない
- `re_ui`、`egui_tiles`、`re_renderer`のserde形をUser settings正本へしない
- recording閲覧用Time Panelを編集Timelineとしてそのまま採用しない
- `re_renderer`導入を理由にMotoliiのVello採用、色変換一元化、preview/export同一関数を崩さない
- GPU pickingを理由に同期readbackを入れない。async方式もM5裁定前は採用しない
- Rerunのcache、thread、query modelでMotoliiの単一writer/latest mailbox/generationを置換しない
- Rerunの見た目を理由にReactモックの操作決定やMotolii visual languageを黙って変更しない
- GPL/AGPLの第三者コードをRerun周辺の類似例から混入させない

## 9. 完了の定義

本計画は文書を読んだだけでは完了しない。次の順に証跡が揃った時に段階完了とする。

1. RR-0 inventoryと反対側レビュー
2. RR-1〜8それぞれの`DEPEND/VENDOR/PORT/PATTERN/REJECT`判定
3. 判定を既存M3/M5タスクの完了条件へ個別反映
4. RR-9統合fixtureを同一device・非blockingで実行
5. React referenceとの視覚・操作比較
6. CJK/IME、1,000 clips、100,000 keys、idle、resize/minimize/restoreの証跡
7. 不採用資産と理由を残し、「Rerunを使ったから完成」と短絡しない

最終目的はRerun依存を増やすことではない。Rerunが既に払った再発明の費用を学び、Motoliiが映像制作固有の意味、操作、楽しさへ資源を集中できる状態を作ることである。
