# Vism が View を要求する(VIEWS)と、surface の uv

状態: **施工済み**(2026-09-24、利用者方針「表現を Rust に焼かない。Vism は要求を宣言し、実体化は host」)。

## 契約

```jsonc
"VIEWS": [ { "FROM": "layer", "LOOK": [x, y, z], "UP": [x, y, z], "FOV": 90 } ],  // 0..16 個
"VIEW_SIZE": 256                                                                // 16..1024 px
```

- `FROM` は必須。今ある語は `"layer"` だけ: 層(全ての複製)の中心に立ち、その層を除いた世界を見る。カメラ相対(Mirror の鏡像カメラ、Portal の別カメラ)は未決の意味論(下)。
- host が準備(prepare)で層ごとに 1 回描く: re_renderer の ViewBuilder・texture pool・`queue_commands`。並べた 1 枚を、その層の run の `view_capture` に束ねる。
- WGSL: `view_count()`、`view_origin()`、`view_sample(i, uv, lod)`(`vism/material.wgsl`)。
- 要求は層に属する: Repeater の複製 1/10/200、View(Stage/Camera)1/2/5、Plate の中でも 6 枚は 6 枚(`surface_tests`、`tick_tests`)。
- `SurfaceIn::uv` は絵の上の 0..1: 画像・図形(path mesh)・押し出しで同じ(fork `Material::texcoord_frame`、`a_surfaces_uv_is_the_same_on_an_image_a_shape_and_a_solid`)。

## 棚の例

- Cube Mirror(`vism/cube_mirror.wgsl`): 6 View の cubemap は manifest の 6 行。host に cubemap 専用の関数は無い。
- CCTV(`vism/cctv.wgsl`、id `shelf.cctv`): 1 View を面へ写す。motolii 以外の名前空間の札が同じ契約で動く。
- Checker(`vism/checker.wgsl`): uv の契約の試験台。
- 標準 material(`vism/material.wgsl`)も棚の module: Standard Glass の見た目は保存で変わる。

## 実窓(dev、Rust の再 build なし)

Cube Mirror の VIEWS 有無、CCTV の追加・LOOK/FOV の変更・VIEWS の除去・FROM 無しの拒否(last-good 保持と理由の表示)と修正での復帰・Repeater 12 枚が 1 View を共有、を同じ起動中に確認。

## 未決

- `FROM` のカメラ相対(鏡像・別カメラ): 「どの View のカメラか」(Stage と Camera で違う)を決める意味論が要る。
- release build は Vism を焼き込む(見張りは debug workspace だけ): 出荷した窓へ第三者の Vism を足す置き場は未決。
