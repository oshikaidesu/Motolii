# Clip — 世界の平面で切る(板も点群も網も同じ式)

利用者裁定(2026-09-08): **3D は 2D の上位互換。z=0 の世界を 2D と定義する。** 切断は網の hook ではなく世界の平面として持ち、位置を持つ描画は全部同じ式に従う。vgpu `examples/clipping`(平面で切って断面に蓋)から、断面の蓋の考えを借りた。

## 編集の意味

- 効果棚の **Clip**(`vism/clip.wgsl`、STAGE "clip"、shader は無い)。欄は Axis(±X/±Y/±Z、層の枠の向き)・Offset(層の中心からの px)・Cap。
- 層に載せればその層が切れる。画像(z=0 の板)でも、点群でも、網でも同じ。平面は層の枠で定義し、描く時に世界へ写す(層を動かせば切り口も一緒に動く)。

## 描く側

- **fork(re_renderer、commit 5e32b3f2)**: `ClipPlane {normal, distance, cap}`(`clip.rs`)と `clip_outside()`(`shader/utils/clip.wgsl`)が 1 箇所の式。矩形は per-rect uniform + `world_position` の varying、点群は batch uniform(点の中心で判定)、網は group 2 の uniform。網は切り口から見える裏面(法線が視線と逆の面。巻き順は使わない — Motolii の網は両面描き)を平面の法線で陰影して蓋にする(`cap`)。
- **Motolii**: `ClipSpec`(`compositor/clip.rs`)を `Layer` が持ち、run で矩形の角と辺・点群と網の `world_from_obj` から `ClipPlane` を作って渡す。`IsfStage::Clip` / `EffectStage::Clip` を足し、hook(`hooks()`)は無視。

## 検収

- `compositor::clip::contract::a_flat_image_a_point_cloud_and_a_mesh_are_cut_by_the_same_plane`(実 GPU): 32×32 を (16,16) に置いた画像・点群・網の 3 つに +X の Clip で右半分が消え、−X で左半分が消える。
- 平面の式の単体 test(中心 + offset、スケール付きの枠)。render 46 本、fork 36 本。
- 観察: 透明背景の comp では点群の一部画素が出ない(Clip と無関係の既存の挙動。契約 test は不透明背景で色を見る)。

## 宿題

- 蓋は「裏面を平面の法線で塗る」近似。凸でない網では奥の壁が見える。本当の蓋(断面の多角形)は幾何が要る。
- 平面は 1 枚。箱や球で切る形は Clip を増やすか、Clip を shader 付きの stage にする時に。
- 2D 層の Offset は層の中心から。画像の左上基準にしたければ欄を足す。
