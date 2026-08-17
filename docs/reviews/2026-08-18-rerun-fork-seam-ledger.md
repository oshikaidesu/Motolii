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
