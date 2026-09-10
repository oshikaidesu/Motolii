# ガラスの重さと嘘の棚卸し、プリズムの入口

状態: 重さ・嘘は **観察**。プリズムは **縮小採用**(2026-09-10 利用者「今の重たい部分を洗い出しておきたい、ガラスの嘘の部分や、プリズムを出せるようにしておきたい」で自走。Glass に Dispersion の欄を足しただけで、床の虹は含まない。実窓は未検収)。

## 重さ — 実測

ガラス画廊の作品 `assets/2026-09-09-glass-gallery/light-in-form.rrd`(15 層、1600×1000、Glass 3 枚)を正本 Engine で描いた。Apple M4、dev build(opt-level 1)、`cargo run -p motolii-render --example reflection_compare`、各条件 20 frame の中央値。合計は CPU の準備・提出・GPU 完了待ち・readback 込みで、実窓の fps ではない。GPU は timestamp query(`valid` のもののみ)。

| 条件 | 合計 | CPU 準備 | 提出 | 完了待ち | GPU | readback | 反射撮影 | run | backdrop 写し |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 静止・反射 cache あり | 16.7 ms | 2.8 | 0.5 | 12.8 | 4.4 | 0.6 | 0 面 | 4 | 2 |
| 静止・cache なし | 24.5 ms | 5.2 | 0.8 | 17.8 | 9.7 | 0.5 | 12 面 | 4 | 2 |
| 動く(受け手を回す) | 22.5 ms | 5.4 | 0.7 | 16.2 | 8.3 | 0.5 | 12 面 | 4 | 2 |

生データ: [reflection.json](assets/2026-09-10-glass-weight/reflection.json)。9/9 の[経路分離計測](2026-09-09-frame-pipeline-measurements.md)(Glass 3 枚を外すと native render 16.7→13.7 ms)と矛盾しない。

重い順:

1. **GPU 完了待ちの空白**。静止で完了待ち 12.8 ms に対し GPU 実行は 4.4 ms。差の約 8 ms は GPU の仕事ではない(9/9 の sample でも main thread の 86% が device poll)。候補は poll の粒度、1 buffer 同期の readback、論理 frame あたり 14 本の Metal command buffer。ガラス固有ではなく、frame 全体の最大項。
2. **反射の 12 面撮影**。受け手が動くたびに 2 地点 × 6 面(1 面 = min(幅,高さ)/2 = 500 px)を描き直す。GPU +4 ms、CPU 準備 +2.5 ms。cache の鍵は入力全部(`reflection_cache.rs` `reflection_key`)なので、受け手以外が動いても全撮影。
3. **backdrop の写しと mip**。表面効果の手前で run を切り、ここまでの合成を 1600×1000 で copy して mip 11 段を焼く(`sequential.rs` `backdrop_pyramid`)。この作品で 2 回/frame。
4. **run の分割**。4 run = 4 つの ViewBuilder と main pass。層の順序で決まり、ガラスが増えると増える。
5. 反射 atlas の VRAM: 3×4 面 × 500² × Rgba16Float ≈ 12 MB + 6 面の scratch。

## 嘘 — 今のガラスが本当でない所

| 嘘 | 場所 | 何が起きるか |
|---|---|---|
| 厚み = instance scale の 1 軸の長さ | fork `instanced_mesh_base.wgsl:131` | 細長い網は過大、平たい網は過小。網ごとの厚みの欄は無い |
| 2D の板の厚みは常に 1.0 | fork `rectangles.rs:274`、Motolii は設定しない | 板のガラスは 1 world unit の板として屈折する。大きさに追従しない |
| 屈折は入射面で 1 回だけ、厚み分まっすぐ抜ける | fork `lighting.wgsl` `transmitted_backdrop` | 出射面の 2 度目の屈折・全反射・内部の複数面は無い。三角柱の光路は解いていない |
| refract が失敗(全反射)すると反射方向で backdrop を読む | 同上 `select(reflected, refracted, …)` | 全反射は「鏡になる」のではなく背後を反射方向にずらして読む |
| backdrop は画面空間で clamp | 同上 `clamp(uv, 0, 1)` | 画面外へ屈折した光は端の色を引き伸ばす。画面外は環境で補われない |
| 同じ run の上の層は屈折されない | `sequential.rs:255` | 網より上の順序の層はガラス越しに見えない。網 2 枚は後ろだけ前の backdrop に入る |
| 粗さ = mip 段 `pow(r,0.8)·(levels−1)·0.55` | fork `lighting.wgsl` | vgpu transmission の作法。物理の散乱角ではない |
| 反射は 12 面 probe 2 地点まで、box 投影、影響半径で fade | fork `local_reflection` / `scene_specular`、[安定化の記録](2026-09-09-stable-reflection-execution.md) | 自己像・多重反射・正確な遮蔽は無い。撮影時の材質は環境反射まで |
| Fresnel は Schlick のスカラー、吸収なし | fork `shade_surface` | 色ガラスは albedo の掛け算だけ。厚みで濃くならない |
| 2D の法線は平面 | fork `rectangle_fragment.wgsl:198` | 正面の板はレンズ状に膨らまない(擬似法線は次段階) |
| 環境が無いと固定 2 灯へ落ちる | fork `lighting.wgsl:213` | 環境層の無い作品ではガラスにならない |
| footprint による AA は切ってある | fork `FILTER_SURFACE_FOOTPRINT = false`、[表面 AA](2026-09-09-surface-antialiasing.md) | 細かい背景は屈折でちらつく |
| 分散(色ずれ)は既定 0 | `vism/glass.wgsl` Dispersion | 既定は無分散。下の欄で入れる |

coverage(素材の alpha)と透過率を分けているのは意図した嘘で、文字や切り抜きが四角いガラス板に化けないため([共通表面](2026-09-09-shared-surface-plan.md))。

## プリズム — 入れた入口

夜間計画の N1 は「最初は明るい背景を透かした色分散だけを分離比較」([夜間代表作品](2026-09-09-expression-dogfood-night-plan.md))。その入口として Glass に **Dispersion** の欄を足した。

- 定規は [KHR_materials_dispersion](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_dispersion/README.md)(値 = 20 / Abbe 数、0 = 無分散、クラウンガラス 0.33、ダイヤ 0.36、ポリカ 0.63)と、three.js `transmission_pars_fragment` の読み(IOR を `(ior−1)·0.025·dispersion` だけ両側へ広げ、R と B を別の IOR で屈折させ、G は中央)。
- fork `0ddfa2ff`: `shade_surface(…, surface, dispersion)`。backdrop の読出しを `transmitted_backdrop` に括り、dispersion > 0 の画素だけ R・B を別の出口で読む(sample 3 回 + 環境 2 回)。0 は従来と同じ 1 回で、経路も同じ。
- Motolii: `vism/glass.wgsl` に欄(0〜2、既定 0)、`subtype.rs` の解析スタブは新しい signature。欄は manifest から生成されるので Inspector には自動で載る(実窓は未検収)。
- 検収 oracle: render `engine.rs` `dispersion_splits_the_backdrop_edge_into_colors` — 白い空・左黒右白の板・その手前の斜め(rotation.y 60°)で厚い(scale 48)ガラス(ior 3)。灰色しか無い場面が dispersion 0 では灰のまま(|R−B| ≤ 1)、2 では境目で赤と青が割れる。

含まないもの: 床の虹(集光・投影)、三角柱内部の光路・全反射、厚みの意味(上の嘘のまま)。N1 の反例(物体回転と camera 移動を分ける、受け面の移動、画面外)は未実施。「RGB を画面方向へずらすだけ」ではなく屈折角の差だが、それでも床への虹を完了扱いにはしない。

## 次の最小作業

1. 完了待ちの空白 8 ms を分ける(poll / readback / Metal の分割)。ガラスより先に効く。
2. 反射の鍵を「動いた物が撮影範囲に入るか」で絞る。受け手以外の移動で 12 面を撮り直さない。
3. 厚みを欄に出す(網も板も)。2D の 1.0 固定を消す。
4. Dispersion を実窓で触り、Undo・保存・export を確認してから N1 の分離比較へ。
