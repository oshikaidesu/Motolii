# 現行Motolii UIとRerunの対応表

状態: **現行コード対応表／新規製品意味を作らない**（2026-08-11、`ui/motolii-rn/` Build B002）

この表は、現在画面にあるものを起点に「Rerunのどの機構を使うか」「Motoliiに何を残すか」「今どこまで接続済みか」を一枚で確認するための地図である。製品構造の正本は[UI runtime責任境界](ui-runtime-architecture.md)、Stage接続の正本は[Stage Heroとprojection root決定](reviews/2026-08-11-m3-m5-stage-hero-projection-root-decision.md)と[Rerun Spatial Viewer採択再締結](reviews/2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md)が所有する。本表はそれらから新しい意味や実装順を増やさない。

## 状態語

| 状態 | 意味 |
|---|---|
| `DIRECT FIXTURE` | 現行製品component内でRerun codeを実行しているが、固定fixtureでありDocument接続ではない |
| `WRAP TARGET` | 採択するRerun機構は確定済みだが、現行UIへの製品入力接続が残る |
| `PATTERN ONLY` | Rerun実装から問題分割を学ぶだけで、Rerun UIやschemaを製品へ入れない |
| `MOTOLII` | Rerunに対応する製品機能を求めず、Motoliiが所有する |

## 画面対応表

| 現在のUI | 利用者にとっての役割 | Rerun側の対応 | 採択 | Motoliiが所有する部分 | 現在の接続状態 |
|---|---|---|---|---|---|
| title bar（project名、Settings、Undo／Redo、Export） | projectと製品操作 | viewer callbackは外側Hostへtime／selection等を通知できるが、project／Undo／Export機能の代替ではない | `MOTOLII` | ProjectSession、D2／journal、保存、ExportJob、製品command | 表示fixture。各操作の製品command接続は別契約 |
| command bar（select、move、shape、text等） | creator toolの選択 | Spatial Viewerのcamera navigation、picking、selection highlightをtool実行の下位機構として使える | `WRAP TARGET` | tool mode、gesture、snap、terminal intent、1 Undo | RN表示のみ。Rerun picking／D2へ未接続 |
| Browser / Media | 素材の検索、選択、配置 | image／video visualizerは配置後の表示先。Rerun storeやentity treeをasset catalogにはしない | `MOTOLII` + 表示時`WRAP TARGET` | asset identity、import、thumbnail、bin／tag、Place intent | 5,000件のRN fixture。Document／Rerun入力へ未接続 |
| Browser / Effects | effectの発見、preview、適用 | custom archetype／visualizerは採択後effectのStage表示を拡張できる | `MOTOLII` + runtime extension | Effect／Vism catalog、作者向けparameter、admission、適用command | 3件のRN fixture。Rerun visualizer登録やD2へ未接続 |
| Browser / Create | Text／Shape／Adjustment／Cameraの作成 | 作成後のshape、image、camera等はSpatial Viewer visualizerの入力になり得る | `MOTOLII` + 表示時`WRAP TARGET` | 作成recipe、stable identity、D2 Add command、既定値 | placeholder表示のみ |
| panel split／resize、tabs | 制作workspaceの配置 | `re_viewport_blueprint`／`egui_tiles`はdeferred layoutの先例 | `PATTERN ONLY` | RN layout、session／user設定、focus、a11y、native component lifecycle | RN local stateで動作。Rerun Blueprintは使用しない |
| Stage外側chrome（Fit、倍率、GPU切替、identity） | Stageの表示条件と状態 | camera／viewport state、View identityを内側機構として利用できる | `WRAP TARGET` | creator向け名称、quality、tool状態、表示policy | chromeはRN。GPU mount切替だけ現行native componentへ接続 |
| Stage base preview | 2D／3D、image、video、shapeを同じ空間で表示 | `re_view_spatial`、store／query、visualizer、camera、picking、outline、`re_renderer` | `ADOPT / WRAP` | Document snapshotのidentity／time／asset翻訳、admission、surface、Preview／Export policy | `DIRECT FIXTURE`。既存surface内でforkした`re_view_spatial::SpatialStage`の同じ`SpatialView3D`をEgui callbackとして実行する。初期cameraは正面透視投影で、視差を保ったまま2D入力は正準高さ1.0のz=0 XY平面に置く。CreateのRectangleは既存`pathgeom`で評価し、Bezierを許容誤差テッセレーションした塗り／輪郭の標準`Mesh3D`として同storeへ記録する。別の2D Viewや直接`re_renderer::ViewBuilder`は使わない。ただしRerun storeへのaccepted Document projection、SVGからPathへのlower、Path編集、Document cameraとの接続は未実装 |
| Stage authoring overlay（frame、bounds、gizmo、path、snap） | 作品を直接選択・編集 | picking／outline／camera結果を入力に使う。overlay UIそのものは採択しない | `MOTOLII` | egui描画、hit／gesture、selection意味、D2 terminal intent | 性能評価用にeguiの既製transform gizmoをStageの同一frameへ重ね、固定Rectangle fixtureをXYZ移動／XYZ回転で一時再投影する。値はFabric eventでRN Inspectorへ相互投影し、Inspectorの±操作も同じ一時値を更新する。Document／D2／Undo、LayerId選択、keyframe、snap／magnet、2D専用制約は未接続 |
| Stage transport／timecode／quality | 再生位置とPreview品質 | time query／time control、viewer callbackは下位機構として利用できる | `WRAP TARGET` | audio-clock Transport、RationalTime、Draft／Final、Preview／Export同一路 | static RN表示。Rerun timeと製品Transportへ未接続 |
| Inspector / Effect | 選択対象の意味とparameter編集 | query／selection結果を表示入力にできる。`re_component_ui`はproperty UIの先例 | runtime `WRAP` + UI `PATTERN ONLY` | primary selection、型付きparameter、form buffer、D2 command、diagnostic | Echo BloomはRN local state。Lottie Path Operationsの8種は固定Rectangle fixtureに限り、selection→既存`pathgeom::apply`→Bezier許容誤差テッセレーション→Stageの標準`Mesh3D`へ接続済み。PathOpのDocument適用／D2、実asset/SVGからPathへのlower、編集、カメラzoom連動の再テッセレーションは未接続 |
| Inspector / Extensions | bundled first-party custom panel | custom visualizerはStage側の描画拡張に使えるが、panel UIやplugin ABIではない | `MOTOLII` + runtime extension | RN panel registry、信頼境界、permission、authoring UI | Tags／Notes panel registryがlocal stateで動作。Rerunとは未接続 |
| Timeline / Packing | layer／clip／keyを作者向けに編成 | time query、time ruler、density、selection同期を下位機構または先例として使える | runtime `WRAP` + UI `PATTERN ONLY` | clip／track／key意味、move／trim／snap、gesture、Undo | RN fixture。Document playhead／selection／commandsへ未接続 |
| Timeline / RN 500 | 大量clip表示の密度確認 | `re_time_panel`のdata density／semantic zoomが先例 | `PATTERN ONLY` | RN virtualization、Motoliiのclip identityと表示密度policy | 20 track／500 clipのRN固定fixture |
| Timeline / Native 500 | native canvasで大量clipとplayheadを操作 | time ruler／visible-time query／selection同期が対応機構 | runtime `WRAP` + UI `PATTERN ONLY` | rust-skia canvas、bounded projection、hit test、seek／selection intent | rust-skia固定fixture。RNとのplayhead／clip feedbackは動くがDocument／Rerun timeへ未接続 |

## まだ画面に席がない境界

| 必要な制作面 | Rerunで使えるもの | Motoliiに必要なもの | 現在地 |
|---|---|---|---|
| mask authoring | segmentation／opacity表示、Stage output | D7のAlpha／Luminance／InvertAlpha／InvertLuminance、path編集、feather等、D2／Undo、composition | 4種maskのDocument→graph→GPUは実装済み。製品Stageのmask UI／path editorは未接続 |
| Curve Editor | time／selection query、densityの先例 | key／Bezier handle、tangent編集、同じD2 command | 現行RN shellにsurfaceなし |
| Stage内の実Document selection | Spatial picking／outline | `LayerId`写像、primary selection、stale拒否、terminal intent | Rerunのpicking runtimeはmount済みだが、`LayerId`写像／selection／D2への接続は未実装 |

## 読み方

最も大きい置換点はStage base previewの一箇所である。既存`MotoliiGpuView`、surface、lifecycleを残したまま、その内側の固定`re_renderer` fixtureを`re_view_spatial::SpatialStage`へ置き換えた。埋め込みtargetにはtable UI／desktop window runtimeを入れない。次の接続は、accepted Document snapshotを読むRerun storeへのprojectionである。

Browser、Inspector、TimelineはRerun UIへ置き換えない。これらは同じDocument revision／`LayerId`／timeを読み、必要なquery、picking、time機構だけRerun runtimeと接続する。Rerun store、Blueprint、selection、playheadを第二の製品authorityにしない。

## コード根拠

- 現行画面とfixture: [`ui/motolii-rn/App.tsx`](../ui/motolii-rn/App.tsx)
- Stage／Timeline native ABI: `ui/motolii-rn/native-renderer/src/lib.rs`(2026-08-16撤去。原文は `git show e6e64265^:` + パス)
- Stage surface／lifecycleの所有: `renderer_core.rs`(2026-08-16撤去。原文は `git show e6e64265^:` + パス)
- Rerun Spatial ViewerのEgui adapter: `rerun_stage.rs`(2026-08-16撤去。原文は `git show e6e64265^:` + パス)
- 現行依存はforkした`re_view_spatial`／`re_renderer`: `native-renderer/Cargo.toml`(2026-08-16撤去。原文は `git show e6e64265^:` + パス)
- Rerun source別の転移境界: [Rerun学習・転移計画](reviews/2026-07-20-rerun-learning-transfer-plan.md)
