# Rerun fork seam 台帳 — 上流とどこで乖離しているか

作成日: 2026-08-18

状態: **台帳**(camera seam は実測済み。既存 seam は commit 題名と diffstat からの整理で、
各差分は未精読)

対象: `oshikaidesu/rerun`(Motolii が `Cargo.toml` から引いている fork)。
「上流を追いかけたくなったとき、何を再適用すればよいか」を1枚にする。

関連: [E0 probe 実測](2026-08-18-rerun-e0-composition-probe.md)、
`docs/reviews/2026-08-18-rerun-as-composition-foundation.md`(裁定)。

## 0. fork の位置

| | commit | 一行 |
|---|---|---|
| 上流の最後 | `954bf95a4` | Improve error messages by hiding details |
| Motolii の最初 | `ccbdad275` | feat(viewer): expose embeddable spatial stage |
| 従来の tip | `501a0403b` | fix(renderer): repair the pool tests the handle argument broke |
| 本レーンの tip | `483b85596` | feat(spatial): let embedders place the view camera directly |

`git diff --stat 954bf95a4 483b85596` = 27 files, +1001 / -54。
うち `spatial_stage.rs`(509行)と `stage_camera.rs`(122行)は**丸ごと追加ファイル**なので、
上流 file への実質的な改変は残り。

## 1. seam 一覧

「追加」= 上流に無いものを足しただけ(rebase で conflict しにくい)。
「改変」= 上流の既存行に手が入っている(rebase で読み直しが要る)。

| 場所 | 種類 | 何のため |
|---|---|---|
| `re_view_spatial/src/spatial_stage.rs` | 追加(509行, file 丸ごと) | 埋め込み用の `SpatialStage`。Viewer app を建てずに Spatial 3D を1枚動かす |
| `re_view_spatial/src/stage_camera.rs` | 追加(122行, file 丸ごと) | **本レーン。** 埋め込み側が置くカメラ(§2) |
| `re_view_spatial/src/eye.rs` | 改変(+19 / -0) | **本レーン。** カメラ欄1つと読み側フック1ブロック(§2) |
| `re_view_spatial/src/lib.rs` | 改変(+6 / -0) | 上2つの module 宣言と再公開 |
| `re_renderer/src/resource_managers/texture_manager.rs` | 追加(+176) | `import_gpu_premultiplied`。GPU 常駐フレームを alpha 付きで texture cache へ入れる |
| `re_renderer/src/wgpu_resources/*` | 改変(+90 / -...) | pool の handle 引数まわり。`501a0403b` はその test 修理 |
| `re_viewer_context/src/store_hub.rs` | 改変(+61) | `StoreHub::add_chunk`(埋め込み側が ingest を持つときの正規経路) |
| `re_viewer_context/src/gpu_bridge/image_to_gpu.rs` | 改変(+12) | alpha channel の扱い |
| `re_viewer_context/src/{app_context,storage_context,viewer_context,lib}.rs` | 改変(小) | 上記を通すための可視性・配線 |
| `re_ui/*`, 各 `Cargo.toml` | 改変(小) | 依存の切り出し(埋め込み時に Viewer 一式を引かないため) |

**上流 rebase で最初に見る順**: `re_renderer/src/wgpu_resources/`(上流の変更が速い) →
`re_viewer_context/src/store_hub.rs` → `re_view_spatial/src/eye.rs`。
追加ファイル2枚は基本そのまま乗る。

## 2. camera seam(2026-08-18 追加)の詳細

**なぜ要るか。** Rerun のカメラは blueprint の `EyeControls3D` で決まるが、
`SpatialStage` には blueprint へ書く口が無い。`SystemCommand::AppendToStore` が
唯一の伝送手段で、それを `process_system_commands` が捨てているためである
(E0 probe §4.3 Seam 1/2)。

**なぜ blueprint 経路を採らなかったか。** `AppendToStore` を通すこと自体は数行で済むが、
それだけでは「カメラをここに置く」とは言えない。言えるのは `focus_entity(entity)` までで、
実際の姿勢は bounding box の発見的処理が決める。document camera を注入する用途には
間接的すぎるため、**素直な公開 API を1本生やす**方を採った(利用者裁定 2026-08-17
「単純なラッパー作業を優先する」)。blueprint 系 seam(E0 §4.3 の S1/S2/S3)は
**手つかずのまま残してある**。

**どこに何があるか**(`483b85596` 時点の行番号):

| file:line | 中身 |
|---|---|
| `re_view_spatial/src/stage_camera.rs`(新規) | `StageCamera` 型と、Rerun の `Eye` へ触れる唯一の変換 `to_eye` |
| `re_view_spatial/src/eye.rs:210-214` | `EyeState` の欄1つ(`pub stage_camera: Option<StageCamera>`) |
| `re_view_spatial/src/eye.rs:1198-1209` | `EyeState::update` 冒頭の読み側フック1ブロック |
| `re_view_spatial/src/spatial_stage.rs:154-181` | 公開 API(`set_camera` / `camera` / `clear_camera`) |
| `re_view_spatial/src/spatial_stage.rs:431-435` | 描画直前に view state の欄へ渡す1箇所 |
| `re_view_spatial/src/lib.rs:22-23, 41` | module 宣言と再公開 |

**上流耐性のための取り決め。**

- 公開署名は plain な数学型(`[f32; 3]`, `f32`)だけ。`Eye` や `EyeControls3D` は出さない。
  **上流が内部型を変えたとき、直すのは `StageCamera::to_eye` の数行で済む**
- 上流 file への差分は**追加のみ**(`+19 / -0`, `+6 / -0`)。既存関数の書き換えはしていない
- 読み側は1箇所(`eye.rs:1203`)。ブループリント読み出し・補間・入力処理には触れていない

**rebase 時の再適用手順。**

1. `stage_camera.rs` はそのまま置く(上流と衝突しない)
2. `eye.rs` — `EyeState` に欄を戻し、`EyeState::update` の**冒頭**にフックを戻す。
   `update` の署名や `stop_interpolation` が変わっていれば、そこだけ合わせる
3. `spatial_stage.rs` — 「Motolii seam」と書いた連続ブロックを戻す。
   `view_state.downcast_mut::<SpatialViewState>()` と
   `state_3d.eye_state` の綴りが変わっていないか見る
4. `StageCamera::to_eye` を `EyeController::get_eye`(`eye.rs`)と読み比べる。
   `Eye` の組み立て方が変わっていたらここを合わせる
5. `cargo test -p rerun-e0-composition-probe`(§3)を回す

## 3. 恒久 oracle — rev を上げたら落ちて教えてくれるもの

**`cargo test -p rerun-e0-composition-probe`。** 7件。窓を開かず、実時間にも依存しない。
このうち camera seam を守るのは次の2つ。

- `tests::injected_document_camera_maps_the_layer_onto_the_frame`
  (`spikes/rerun-e0-composition-probe/src/main.rs`)— 注入した document camera が
  既知配置のレイヤーを**期待座標へ写す**ことを見る。期待 pixel は画角と距離から
  probe 側だけで決めており、Rerun の `Eye` / `ui_from_world` を通さない。
  **描画に対する独立な照合である**
- fork 内 `stage_camera.rs` の unit test 3件 — `to_eye` の位置・前方・画角・縮退

実測値(2026-08-18, macOS / Metal, 640x480):

```
injected document camera: pos = (0, 0, 0.8560), look_target = (0, 0, -0.01), fov_y = 1.0472 rad
  last_eye() readback: pos = (-0.0000, -0.0000, 0.8560), fwd = (0.0000, 0.0000, -1.0000)
  frame corners: red / green / blue / yellow (all as expected)
  4象限 × 576点 = 2304点すべてが期待 pixel の色。wrong = 0
reset_view(): 既定カメラの PNG と sha256 まで一致(Rerun の画作りが戻る)
```

**rev bump の検収はこのテストを回すだけで済む**、というのがこの形の狙いである。

## 4. 測っていないこと

- **3 OS 未検証。** macOS / Metal 1台のみ
- **orbit との相互作用。** `set_camera` は sticky で、置いている間はマウス操作で
  動かない(`clear_camera` / `reset_view` で外れる)。埋め込みステージでは
  そもそも orbit が保存されない(blueprint 書き戻しが捨てられているため)ので、
  「明示 set → orbit で上書き」を成立させたければ S2 を通す別レーンが要る
- **orthographic。** E0 §4.3 Seam 3 は手つかず。`StageCamera` も perspective のみ
  (`fov_y` を必ず `Some` で渡している)。正対 2D を orthographic でやりたくなった
  時点で、`fov_y: None` を通す形へ広げられる
- **既存 seam の各差分。** §1 の表は commit 題名と diffstat からの整理である

## Seam: ViewBuilder::main_target() read accessor(裁定161 BL1b、2026-08-21)

- 上流 file: `crates/viewer/re_renderer/src/view_builder.rs`
- 実質差分: +15行(既存 struct/関数への追加メソッド1本のみ、既存コードの変更ゼロ)
- 内容: `ViewTargetSetup.main_target_resolved`(private、Rgba8UnormSrgb)への read-only accessor。`ViewBuilder::composite()`(ガンマ round-trip 込み)を経由せずに、線形合成後・ガンマ変換前の中間結果を embedder が直接読める
- oracle: fork 側 `crates/viewer/re_renderer/tests/motolii_main_target_accessor.rs`(`main_target_reflects_the_view_that_was_drawn`)。Motolii 側は `motolii-compositor/tests/sequential.rs`(`sequential_matches_render_for_overlapping_alpha_and_pinned_fixture`、バイト一致)がこの accessor に依存する消費者
- 消費者: `motolii-compositor::Compositor::render_sequential`(`next/engine/motolii-compositor/src/lib.rs`)。新規 WGSL ゼロ(既存 RectangleDrawData+import_gpu_premultiplied+ScreenshotProcessor の再利用)。実装の罠2件は同 lib.rs doc に file:line 込みで記録(ViewBuilder drop 時の texture destroy / 背景 rect の Nearest 固定)
- pin rev: **反映済み**(fork commit `856f597c3`、GitHub push 済み・`next/Cargo.toml` 全10 entry を bump、検証用 [patch] は除去済み — `cbbe4f2b`)

## Seam: RenderContext::new_from_device + DeviceCaps::from_device(裁定170 M3、2026-08-22)

- 上流 file: `crates/viewer/re_renderer/src/context.rs`(+56/-5、`new()` の本体を私的 `new_impl` へ畳む挙動不変リファクタ込み)・`crates/viewer/re_renderer/src/device_caps.rs`(+35)
- 内容: adapter の実物なしで `RenderContext` を組む姉妹コンストラクタ。`DeviceCaps::from_device` は wgpu 29 `Device` の `features()/limits()/adapter_info()` から導出し、downlevel 判定は backend 分岐(`Gl` のみ保守的に `Limited` — 裁定170 §2)。既存呼び手5箇所は無改変
- 理由: iced 0.15 shader widget の `Pipeline::new(device, queue, format)` に adapter が渡ってこない(iced 側 compositor のフィールドは private) — iced fork へ trait 改変を入れる案Aは rebase 複利で棄却し、こちらの口で閉じた
- oracle: `next/engine/motolii-compositor/tests/with_device.rs::with_device_matches_headless` — **adapter を明示 drop してから**組んだ Compositor が `headless()` とフレームバイト一致(常設)
- 消費者: `motolii-compositor::Compositor::with_device`(M4 で stage presenter の Pipeline が使う)
- pin rev: **反映済み**(fork commit `7cca401e`、branch `motolii/m3-device-caps` GitHub push 済み・`next/Cargo.toml` 全 entry bump・検証用 [patch] は除去済み)

## Seam: the view's main target is scene-linear half float(締めの裁定、2026-09-25)

- 上流 file: `crates/viewer_support/re_renderer/src/view_builder.rs`
- 実質差分: +10/-9(`MAIN_TARGET_COLOR_FORMAT` を `Rgba8UnormSrgb` から `Rgba16Float` へ。doc を「面は 1 を超える放射輝度を書ける。embedder か composite が表示へ写す」へ)
- 理由: HDR は Look の一部ではなく**レンダリング基盤**(利用者裁定)。ハイライト・発光・Look の bright-pass が 1 を超える値を要る。読み書きは sRGB 版と同じく linear。MSAA は linear で resolve するので明るい sample が縁で勝つ(doc に MRP / AMD の資料と代案を残した)
- oracle: fork 側 `tests/main_target_accessor.rs`(format を `MAIN_TARGET_COLOR_FORMAT` と比べる形へ)。Motolii 側は `engine::material::tests::a_flat_shape_with_gain_above_one_glows_through_the_looks_bloom`(1 を超える値が積みを通って Look に届く)
- 消費者: `motolii-render` の view canvas・Look pass(`compositor/look.rs`)
- pin rev: **反映済み**(fork commit `1e479e1f9`、branch `motolii/render-wrapup`、GitHub push 済み・root `Cargo.toml` の re_* 全 entry を bump、ローカル path の [patch] は無い)

## Seam: SurfaceProgramDesc::pixel_rate(締めの裁定、2026-09-25)

- 上流 file: `crates/viewer_support/re_renderer/src/renderer/mesh_program.rs`(+8/-3)
- 内容: 面の色が画素内で変わらない program(平坦な無照明の絵・解析的な曲線 coverage・小さな石・平らな蓋)が、文脈が per-sample で陰影する所でも画素に 1 回だけ陰影する。幾何の縁は MSAA の coverage が解く(既存の `surface_sampling_replacements` が Limited / MSAA Off で使う centroid 置換へ 1 条件足しただけ)
- 裁定: 2026-09-09 の「主描画と反射は sample 補間」の範囲を、小さい石と flat cap と平坦 2D の LOD としてここで狭める。silhouette / facet edge は MSAA、hero は per-sample のまま
- oracle: Motolii 側の平坦 2D の画素一致(U79)、`engine::surface_tests`
- pin rev: 上と同じ(`1e479e1f9`)

## Seam: CpuModel::is_faceted(締めの裁定、2026-09-25)

- 上流 file: `crates/viewer_support/re_renderer/src/importer/cpu_model.rs`(+11、追加メソッド 1 本のみ)
- 内容: 全三角形が flat(3 頂点の法線が一致)か。切り石(面ごと)と滑らかな体を host が見分ける。面の内側の陰影は法線でなく視線だけで変わるので、per-pixel が LOD として成り立つ
- pin rev: 上と同じ(`1e479e1f9`)
