# 探索者の監査(2026-09-24)と Composer の採択

状態: **監査済み・採択は既決から一意な物だけ**。方式: 共通の憲法(Vism = cassette、View = 作品の観測、表現 graph は自由・実行は有限、Motolii は GPU 実行を所有しない、fork は最後の手段、画を減らして速くしない、gap を fixture で隠さない)を渡した探索者が証拠と patch 候補を出し、Composer が「既決から一意」なら採択、新しい意味は人間へ。

## F: Rust にまだ焼いてある表現(30 件)

| 分類 | 件 | 代表 |
|---|---|---|
| EXPRESSION_IN_RUST | 14 | 標準 material の文字列(`surface_program.rs` STANDARD_MESH/PICTURE/MOVED_PICTURE: 粗さ 1・IOR 1.5・`sun_shade` の facing_floor 0.5・場で動いた絵は無照明)、Turbulent Displace の CPU 写し(点群だけ、`noise.rs`)、Track Overlay 4 札の全部(`extensions/overlay.rs`・`engine/overlay.rs`)、Rope(剛性 60・減衰 6・`ROPE_PASS` の WGSL 文字列)、room-scope Block の欄を位置で読む(`engine/blocks.rs:266-274`)、物性→摩擦/反発の写像(`engine/physics.rs`)、light cookie 512²、場の格子 128/8px/4px、Plexus 線の定数、Group の影 32 環 |
| 宿主の fork に漏れた Motolii の意味 | 1 | `backdrop_levels_read`(material.wgsl の lod 曲線の写し)→ **採択済み**: Motolii 側へ移動(`07f5d731e`)。fork 側は次の fork 改訂で削除 |
| CONTRACT_CONSTANT / HOST_LOWERING | 9 | 2D の積み順 bias 0.02、Add は α 0、FOV/VIEW_SIZE の既定、太陽の投影、blend 番号 |
| UNSURE(製品判断) | 6 | 8-bit sRGB の stack(HDR が run 間で切れる)、Glass は Glass を見ない、generator は層の形で切る、Motion Blur の標本密度、点の既定径 |

採択判定:
- **一意に決まる(順に着手可)**: 標準 material の既定値を棚の Vism へ(a693456ba の続き)。room-scope Block の欄を manifest の名前で読む(隠れ ABI の解消)。`backdrop_levels_read` は済み。
- **gap として残す(新しい一般 primitive が要る)**: 点群に届く場は Turbulent Displace 1 種だけ(field Vism が点群へ届く経路が無い)。影は `strength` 1 欄だけ・cookie 512 固定。場の格子密度を Vism が宣言できない。
- **人間へ(意味)**: stack の HDR 化(Target/View/Backdrop が float を持てるか)、Glass が Glass を見るか、generator の切り方、Track Overlay/Rope を Vism 化する時の contract。

## G: fork の差分(上流 `2309184bbb` 比 +4750/−785 行、64 file)

| 分類 | 内容 |
|---|---|
| そのまま上流候補(UPSTREAM_FIX/SEAM) | `read_buffer`、`queue_commands`/`before_submit` の戻り値、`new_with_external_resolved` + texture import、compute pipeline pool、型の再 export、`instance_vertex_positions` |
| 手直し後に上流候補 | mipmap 生成(pool を通していない) |
| fork に残る一般 primitive | ClipPlane、surface/field/motion/tint の hook と `params[24]`、group 0 の View 資源(environment・backdrop・view_capture・coverage・motion・program_constants・near_fade)、DrawOrder/`new_ordered`、`select_source_instances`、material の旗、CurveFill |
| **上流の挙動を変えている(上流に出せない)** | mesh が MSAA 標本ごとに shading(既定)、不透明 mesh を 2 回描く(depth pre-pass)、頂点 α<255 で透明扱い、rect の outline mask が texture α に従う、`params: [0.0; 12]` のままの bench/例が build できない |
| **死んでいる** | `new_layered`/`new_clipped`、`MeshProgram*` 互換 alias(互換 wrapper は既決で禁止)、`compose_mesh_program_source` 再 export、`surface_thickness`(常に 1.0)、`roughness_to_lod`/`equirect_uv_from_direction`、lib.rs の未使用 re-export、Motolii 側 `Engine::with_device` |

採択判定:
- **採択済み**: Motolii 側の死んだ `Engine::with_device`・`with_device_using_headless_defaults` を削除(`07f5d731e`)。
- **一意に決まる(fork 改訂で一括)**: 死んだ API の削除(約 45 行)、`surface_thickness` の削除、`main_target()` + その試験(Motolii 側 6 行の書き換えで不要)、`new_from_device`(adapter を渡せば上流の `RenderContext::new` で足りる、Motolii 側 10 行)、`backdrop_levels_read`(Motolii へ移動済み)、environment.rs の CPU 補助関数を Motolii へ。
- **候補(比率は低いが fork が最も縮む)**: paths module(約 1000 行)を Motolii 側の lowering へ(`into_mesh` が既に上流の `CpuMesh` を出す)。`draw_into` を上流の `draw()` + `queue_commands` で置換。
- **人間へ**: `layer_sort_key`(消すと opt-in していない描画物との順序が変わる)、`SurfaceSampling::FilteredPixel`(production は使っていない、試験だけ)。

## 自分で確かめた物

- run の切れ目 MeshAfterRect は歴史的ではなかった: 消すと 2 本の contract の絵が変わる(gap 台帳に記録)。
- 空の run の固定費は GPU 約 0.08 ms、CPU は pass 数に比例(wgpu の `finish` が View 6 面で約 4 ms)。
