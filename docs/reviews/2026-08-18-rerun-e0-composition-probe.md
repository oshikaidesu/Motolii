# Rerun を空間合成の基盤にできるか — E0 probe 実測

作成日: 2026-08-18

状態: **観察**(3点の実測。うち2点成立、1点不成立)

> **追記(同日、後続レーン): (b) は成立した。** 本稿 §4.3 が挙げた blueprint 系 seam
> (S1/S2/S3)ではなく、`SpatialStage` に素直な公開カメラ API を1本生やす形で通した。
> 経緯・差分・再適用手順は [Rerun fork seam 台帳](2026-08-18-rerun-fork-seam-ledger.md)。
> **以下の本文は追記前の実測記録としてそのまま残してある**(§4.2 の「絵が変わらない」は
> seam を入れる前の観察である)。現在の (b) の姿は §8 を見よ。

対象: embedded Spatial Viewer(`re_view_spatial::SpatialStage`)を、表示専用ではなく
**空間合成の座席**として使えるか。

fork rev: `oshikaidesu/rerun` `501a0403b6942d488798a60c66ade889855346ff`
(workspace `Cargo.toml:107-111` が指しているもの。**改変していない**)

関連: [Rerun表示座席の実測](2026-08-17-rerun-layer-display-seat-measurement.md)、
[M5 3Dとpost](../specs/M5-3d-and-post.md)。
利用者裁定「Rerunは合成のメイン基盤」の文書
(`docs/reviews/2026-08-18-rerun-as-composition-foundation.md`)は本稿執筆時点で
branch `claude/ux-cli-gui-integration-002b03`(`75d8228b`)にあり main に無いため、
リンクではなくパスで示す。本稿はその E0 節の3点に答える。

## 0. 何を測ったか

E0 が挙げる3点を、`SpatialStage` を通したまま実測した。

- **(a) offscreen** — OS 窓なしでシーンを texture へ描き、pixel を読み出せるか
- **(b) camera** — 定義済みカメラをプログラムから注入し、既知配置の矩形が期待座標へ写るか
- **(c) occlusion** — 画素 alpha を持つ2枚を前後に置いて、前が後を遮蔽し alpha 部は透けるか

結論を先に置く。

| | 判定 | 一行 |
|---|---|---|
| (a) offscreen | **成立** | 窓なしで描けて読める。2回実行で PNG が byte 一致する |
| (b) camera | **不成立(投影の一致だけ成立)** | カメラモデルは pixel と一致するが、**注入する口が無い**。fork seam 3箇所 |
| (c) occlusion | **成立** | 近い側が勝ち、alpha=0 の穴から奥が透ける。奥へ回した層は1画素も漏れない |

この文書の主張はすべて PNG またはテスト出力に紐づく。コード読解だけの推論は
§4 の seam 特定に限り、そこは「なぜ通らないか」の説明としてのみ置く。

## 1. どう通したか(拘束の確認)

2026-08-11裁定により Motolii は Spatial Viewer の wrapper であり、direct な
`re_renderer` scene も第二 runtime も作らない。probe も同じ拘束下で走らせた。

```text
premultiplied RGBA wgpu::Texture
  -> SpatialStage::copy_gpu_image
  -> GridMap visualizer / RectangleRenderer
  -> SpatialStage::show(&mut egui::Ui, &mut RenderContext)   ← 製品と同じ入口
  -> egui::Context::run_ui(窓なし) -> egui_wgpu::Renderer -> offscreen wgpu::Texture
  -> copy_texture_to_buffer -> PNG + 画素 oracle
```

**窓が要らないのは Rerun 側の構造からの帰結である。** `SpatialView3D` は
`ViewBuilder` を返さず、`egui_wgpu::Callback` に入れて `ui.painter()` へ積むだけで
終わる(`crates/viewer/re_view_spatial/src/ui_3d.rs:416-420`)。実際の `draw()` /
`composite()` を走らせるのは `egui_wgpu::Renderer` であり
(`crates/viewer/re_viewer_context/src/gpu_bridge/re_renderer_callback.rs:43`, `:89`)、
その相手は surface である必要が無い。よって
「egui を headless で回す + `egui_wgpu::Renderer` を offscreen texture へ向ける」
だけで、`ViewBuilder` を自前で組まずに済む。

**解像度・アスペクト・pixels_per_point に seam は要らなかった。** これらは
embedder が渡す egui の `screen_rect` から素直に降りてくる
(`ui_3d.rs:188-189` `resolution_in_pixel`、`:255` `aspect_ratio`、`:259`
`pixels_per_point`)。probe は 640x480 / ppp=1.0 を外から与えて成立させた。
**任意解像度での export に fork 改変は不要である。**

## 2. (a) offscreen は成立する(観察)

窓を1枚も開かずに、`SpatialStage` のシーンが offscreen texture へ出て、読み戻せた。

- adapter は `re_renderer::device_caps::select_adapter(&adapters, Backends::all(), None)`
  で surface 無しに選ぶ。ここが「窓なし」の実体である
- 出力先は `Rgba8Unorm` の自前 texture。`RenderContext::new` の `output_format_color`、
  `egui_wgpu::Renderer` の `output_color_format`、render attachment の3つを揃える必要がある

証拠: `evidence/rerun-e0-composition-probe/e0-a-offscreen.png`。4象限に塗り分けた
1枚のレイヤーが Rerun の world grid の上に載っている。画素 oracle は
red=10970 / green=7158 / blue=38084 / yellow=20311(いずれも閾値 2000 超)。

**決定性も成立する。** 同じ binary を別プロセスで2回走らせ、出力 PNG 7枚すべてが
sha256 一致した(内容は5種。`e0-b-after-*` の3枚は既定カメラと同一である — §4.2)。
実時間ではなくフレーム番号から `RawInput::time` を作っているため、カメラ補間も
bounding box の平滑化もフレーム数だけの関数になる。

```
633a105de151d7a8f399ddc65e6259fc96069fb1807a72ac54f981b1c6f6cd58  e0-a-offscreen.png
2d5a852f8d3c989e2f0543c4b876f9fb7eb783ecdd6cfddb44de373c75beb658  e0-b-default-camera.png
2d5a852f8d3c989e2f0543c4b876f9fb7eb783ecdd6cfddb44de373c75beb658  e0-b-after-reset-view.png
f017397b0aa05e6804941d97701790c0e5b0cadd6688b1074a3b27818add7c68  e0-c-disc-in-front.png
839e6e25c05808f44882ccd1a216db0ecad5505cde2a6ee964c9fb6373a35e1c  e0-c-disc-behind.png
```

## 3. (c) 遮蔽は成立する(観察)

**2026-08-17 §5 の「深度を書かないので遮蔽できない」は、レイヤー同士については
起きない。** 透明フェーズは距離で back-to-front に並べ替えてから premultiplied
alpha で重ねるため、平行なレイヤーの前後は正しく出る。

- 並べ替え: `crates/viewer/re_renderer/src/draw_phases/draw_phase_manager.rs:135-140`
  (`sort_for_transparent_phase` は `distance_sort_key` の**降順**、同値なら
  `secondary_sort_key`(= `draw_order`)の昇順)
- 混色: `crates/viewer/re_renderer/src/renderer/rectangles.rs:744`
  (`BlendState::PREMULTIPLIED_ALPHA_BLENDING`)
- フェーズ選択: 同 `:485-492` と `:607-610`。GPU import した texture は
  `AlphaChannelUsage::AlphaChannelInUse` でタグされる
  (`crates/viewer/re_renderer/src/resource_managers/texture_manager.rs:246`)ので、
  **中身が全面不透明でも透明フェーズへ行く**

実測は不透明な赤い矩形と、中央だけ不透明で外側 alpha=0 の緑の円板を z で 0.05 離した
2枚。`plane_clustering.rs:168` が平面距離を 1.0e-3 刻みで量子化するため、この距離なら
coplanar クラスタには入らず、それぞれ独立に距離ソートされる。

| 配置 | red | green | 判定 |
|---|---|---|---|
| 円板が手前(`e0-c-disc-in-front.png`) | 68037 | 11869 | 緑が乗り、周囲は赤が見える |
| 円板が奥(`e0-c-disc-behind.png`) | 72872 | **0** | 不透明な赤に完全に隠れる |

**円板にはわざと大きい `draw_order`(10.0 対 0.0)を与えてある。** draw order で
前後が決まってしまう実装なら、奥に置いた円板が漏れて 0 にならない。0 だったことが
「距離ソートが効いている」ことの対照になる。

### 3.1 どこまで言えるか

平行な2枚のレイヤーについてしか言っていない。深度を書かないことの影響は残る:

- **交差する透明面**は per-pixel でなく per-drawable(矩形中心)でソートされるので、
  貫通していれば破綻する。2.5D のレイヤースタックでは起きない
- **不透明ジオメトリとの混在は未測定。** フェーズ順は Opaque → Background →
  Transparent(`view_builder.rs:814-821`)で、Opaque が先に深度を書き Transparent が
  それを深度テストする、という構造からは正しく出るはずだが、`DeformedMesh`
  (`VSM-M5-G0`)や `PieceSet`(`VSM-M5-G1`)を混ぜた絵は**撮っていない**

## 4. (b) カメラ注入は不成立(観察 + seam 特定)

2つに分けて測った。

### 4.1 カメラモデルと描画は一致する(成立)

`SpatialStage::last_eye()` が返す `Eye` で 4象限をそれぞれ 24x24 の格子に投影し、
画面内へ落ちた点の色を照合した。**間違いは1点も無い。**

```
image top-left    (red):    576 samples inside the viewport,   0 outside, 0 wrong
image top-right   (green):  576 samples inside the viewport,   0 outside, 0 wrong
image bottom-left (blue):    91 samples inside the viewport, 485 outside, 0 wrong
image bottom-right(yellow): 410 samples inside the viewport, 166 outside, 0 wrong
```

副産物として、2026-08-17 §6 の「`GridMap` は行0を上端に置く」が world 座標つきで
確認できた。画像の行0は world の +y 側である
(`crates/viewer/re_view_spatial/src/visualizers/grid_map.rs:290-293` の
`extent_v = NEG_Y * height * cell_size`)。レイヤー面は XY 平面で、z が面の法線である。

**同時に、既定カメラは document の画枠を知らないことが見えた。** bottom-left 象限は
576点中485点が画面外へ落ちている。既定 eye は scene bbox からの fallback であって、
出力画枠という概念を持たない(`e0-b-default-camera.png` を見れば手前側の角が
画面下へはみ出している)。

### 4.2 カメラを注入する口が無い(不成立)

`SpatialStage` の公開 API でカメラへ触れるのは `focus_entity` と `reset_view` だけ。
document camera 相当の `Pinhole` を立てたうえで3通り試し、**どれも 1 byte も
絵が変わらず、`last_eye()` も動かなかった**。

```
focus_entity(layer)          : fnv1a 1db085eb3cba4c98 (identical), eye.pos = (-0.6813, -2.0440, 1.9263)
focus_entity(pinhole camera) : fnv1a 1db085eb3cba4c98 (identical), eye.pos = (-0.6813, -2.0440, 1.9263)
reset_view()                 : fnv1a 1db085eb3cba4c98 (identical), eye.pos = (-0.6813, -2.0440, 1.9263)
```

`e0-b-after-reset-view.png` は `e0-b-default-camera.png` と sha256 まで同一である。

### 4.3 通すのに必要な seam

**Seam 1 — blueprint への書き口が無い**

- `crates/viewer/re_view_spatial/src/spatial_stage.rs:182-185` — `ingest_chunk` は
  `self.recording_store_id` 固定で、blueprint store へ書く関数が無い
- カメラは blueprint 側の `EyeControls3D` で決まる。読み出しは
  `crates/viewer/re_view_spatial/src/eye.rs:287-329`(`EyeController::from_blueprint` が
  `position` / `look_target` / `eye_up` を引く)、property の構築は
  `crates/viewer/re_view_spatial/src/ui_3d.rs:275-279`
- 必要な追加: `SpatialStage` に blueprint chunk を入れる口(例
  `ingest_blueprint_chunk`)か、`EyeControls3D` を直接書く専用 setter

**Seam 2 — 既にある書き戻しが捨てられている**

- `crates/viewer/re_view_spatial/src/spatial_stage.rs:154-175`、特に `:172` の
  `_ => {}` が `SystemCommand::AppendToStore`
  (`crates/viewer/re_viewer_context/src/command_sender.rs:108`)を捨てる
- これは `ViewProperty::save_blueprint_component`
  (`crates/viewer/re_viewport_blueprint/src/view_properties.rs:191` →
  `crates/viewer/re_viewer_context/src/blueprint_helpers.rs:80` → `:133` → `:145` → `:183`)の
  唯一の伝送手段である
- 結果として `SpatialStage::focus_entity`(`spatial_stage.rs:145`)と
  `reset_view`(`:150`)は**現状 no-op である**。両者の効果はすべて
  `EyeState::focus_entity`(`eye.rs:1070-1120`)→
  `EyeController::save_to_blueprint`(`eye.rs:340-373`)を経由する。編集中の
  orbit 操作が保存されないのも同じ理由になる
- 必要な追加: `process_system_commands` が `AppendToStore` を受けて
  `store_hub.add_chunk` へ流す(recording と blueprint の両方の store id を許す)

**Seam 3 — orthographic は 3D view から選べない**

- `crates/viewer/re_view_spatial/src/ui_3d.rs:252-256` が
  `projection_from_view: Projection::Perspective { .. }` を直書きしている
- `EyeController::get_eye`(`crates/viewer/re_view_spatial/src/eye.rs:264-285`)も
  常に `fov_y: Some(self.fov_y.unwrap_or(Eye::DEFAULT_FOV_Y))` を返すので、
  `Eye` の側でも orthographic を表せない(`eye.rs:89-91` に
  `is_orthographic` のコメントアウトが残っている)
- `re_renderer` 側には `Projection::Orthographic`
  (`crates/viewer/re_renderer/src/view_builder.rs:102-130`、
  `OrthographicCameraMode` は `:77`)がある。**上流の欠落ではなく、
  3D view が選んでいないだけである**
- Seam 1/2 を通しても document camera は perspective 止まりになる。AE の
  カメラレイヤーとしてはそれで足りるが、「正対の 2D 合成」を orthographic で
  欲しいならここも要る

**Seam 4(seam ではない: 設計判断)** — 既定 eye が出力画枠を知らない件は、
Seam 1/2 を通して document camera を注入すれば消える。fork 側に別の改変は要らない。

### 4.4 `Pinhole` 経由(カメラレイヤーの外注)も同じ seam に乗る

logged `Pinhole` を view camera にする経路自体は存在する
(`Eye::from_camera`, `crates/viewer/re_view_spatial/src/eye.rs:38-45`。
選択は `EyeControls3D::tracking_entity` の読み出し `eye.rs:794-798`、
tracking の適用 `eye.rs:878-950`)。**AE のカメラレイヤーを Rerun へ外注する
という裁定の絵は、Rerun 側の機構としては既にある。** 塞がっているのは
`tracking_entity` を書く手段だけで、それが Seam 1/2 と同じ場所である。

## 5. 測っていないこと

- **3 OS 未検証。** macOS / Metal 1台のみ。決定性の一致もこの1台の中の話である
- **不透明ジオメトリとの混在(§3.1)。** `Mesh3D` 系を混ぜた絵は撮っていない
- **blend mode・effect 合成順**(E0 の対象外。合成基盤化の残り論点として生きている)
- **カメラ注入後の pixel。** seam が塞がっているので「注入したカメラで期待通り写るか」は
  測れていない。測れたのは「読み出したカメラのモデルが pixel と一致する」ところまで
- **Preview=Export の pixel 同一性。** offscreen 経路と窓あり経路の絵を突き合わせていない。
  同じ `SpatialStage::show` を通っているが、それは構造からの推測であって実測ではない
- **性能。** 1フレームの所要時間を測っていない

## 6. E1 へ進めるかの所見

- **(a) と (c) は E1 の前提として置いてよい。** export を「per-layer 評価 → シーン合成 →
  mux」へ差し替える際、シーン合成側が窓なしで決定的に回ることと、レイヤーの前後が
  遮蔽として成立することは実測で埋まった
- **(b) は E1 の着手前に fork seam を1つ入れる必要がある。** 必要なのは Seam 1 か
  Seam 2 のどちらか片方で足りる可能性が高い(Seam 2 だけ通せば `focus_entity` 経由で
  `tracking_entity` が書けるようになり、`Pinhole` を document camera として使える)。
  Seam 1 の方が直接的で、キーを打った camera を毎フレーム流す形に素直に繋がる
- **Seam 3 は急がない。** perspective の document camera で AE のカメラレイヤーは
  表現できる。正対 2D を orthographic でやりたくなった時点で判断すればよい
- 「ビューとエクスポートが同じシーンを通る」の**同一性は未証明**である。E1 では
  Preview と Export の pixel 一致 oracle を最初に置くべきで、それは本 probe の
  ハーネスをそのまま使える

## 7. 実装

`spikes/rerun-e0-composition-probe`(workspace member、製品 binary には入らない)。

- `src/harness.rs` — 窓なしの `SpatialStage` driver。wgpu device と egui だけを作り、
  シーン構築・カメラ・描画は `SpatialStage::show` に任せる
- `src/scene.rs` — レイヤー素材(4象限 / 単色 / 中央だけ不透明な円板)と配置
- `src/oracle.rs` — 支配チャンネルによる色の分類、ヒストグラム、PNG 書き出し
- `src/main.rs` — 3つの probe と落ちる oracle

```bash
cargo run -j 5 -p rerun-e0-composition-probe -- /tmp/e0-run1
cargo run -j 5 -p rerun-e0-composition-probe -- /tmp/e0-run2
shasum -a 256 /tmp/e0-run1/*.png /tmp/e0-run2/*.png
```

3点すべて通れば exit 0。現状は (b) が落ちるので **exit 1** である。
`cargo test -p rerun-e0-composition-probe` は素材と分類器の 6 件。

証拠一式: `evidence/rerun-e0-composition-probe/`(相異なる PNG 5枚 + `probe-output.txt`)。

## 8. 追記 — (b) をどう通したか(同日、後続レーン)

§4.3 は「Seam 1 か Seam 2 のどちらか片方で足りる」と見込んでいた。**その見込みは
技術的には正しいが、採らなかった。** `AppendToStore` を通しても、そこから言えるのは
`focus_entity(entity)` までで、実際のカメラ姿勢は bounding box の発見的処理が決める。
document camera を「ここに置く」用途には間接的すぎる。

代わりに **`SpatialStage` へ公開カメラ API を1本生やした**。blueprint 系 seam
(S1/S2/S3)は手つかずのまま残してある。

```rust
let camera = StageCamera::new([0.0, 0.0, camera_z], [0.0, 0.0, -0.01], [0.0, 1.0, 0.0])
    .with_fov_y_radians(std::f32::consts::FRAC_PI_3);
stage.set_camera(camera);
```

実測(fork `483b85596`、macOS / Metal、640x480):

| oracle | 結果 |
|---|---|
| 注入で絵が変わる | 成立(fnv1a が既定カメラと相違) |
| `last_eye()` が注入した姿勢を返す | 成立(pos/fwd/fov すべて 1e-4 以内) |
| レイヤーが画枠ちょうどに写る | 成立(四隅が赤/緑/青/黄) |
| 格子点が期待座標の色と一致 | 成立(**2304点中 wrong = 0**) |
| `reset_view()` で既定へ戻る | 成立(既定 PNG と sha256 一致) |

**期待 pixel は Rerun の `Eye` / `ui_from_world` を通さず、画角と距離から probe 側だけで
決めている。** よってこれは描画に対する独立な照合であり、§5 が「測れていない」と
していた「注入したカメラで期待通り写るか」がここで埋まった。

probe binary は **exit 0**。`cargo test -p rerun-e0-composition-probe` は7件
(`injected_document_camera_maps_the_layer_onto_the_frame` を含む)。
このテストは **fork の rev を上げたら落ちて教えてくれる恒久ゲート**として置いてある。

§5 の「測っていないこと」のうち、なお残るもの: 3 OS 未検証、不透明ジオメトリとの混在、
blend mode、Preview=Export の pixel 同一性、性能。加えて orbit との相互作用と
orthographic は台帳 §4 に挙げてある。
