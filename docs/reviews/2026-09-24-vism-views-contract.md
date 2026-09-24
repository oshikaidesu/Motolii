# Vism が View を要求する(VIEWS)と、surface の uv

状態: **施工済み**(2026-09-24、利用者方針「表現を Rust に焼かない。Vism は要求を宣言し、実体化は host」。同日「View は作品の view、依存グラフは制限しない、実行だけ有限」)。

## 契約

```jsonc
"VIEWS": [ { "FROM": "layer", "LOOK": [x, y, z], "UP": [x, y, z], "FOV": 90 } ],  // 0..16 個
"VIEW_SIZE": 256                                                                // 16..1024 px
```

- `FROM` は必須。今ある語は `"layer"` だけ: 層(全ての複製)の中心に立ち、その層を除いた世界を見る。カメラ相対(Mirror の鏡像カメラ、Portal の別カメラ)は未決の意味論(下)。
- **View は作品の view**(利用者裁定 2026-09-24「世界を素材として読める。読んだ世界の中にも、また世界を読む表現を置ける」): Camera・Stage と同じ `record_stack`(層の順・run・plate・2D の順・screen pass・Backdrop・空・View)を、別の観測者(`Observer`: 投影と near fade)から記録する。違いは観測者・解像度・要求した層(その複製は写らない)だけ。第二の場面組み立ては削除(`compositor/layer_views.rs`)。
- 準備済みのコマから tick に 1 回描く(Stage/Camera の数で増えない)。並べた 1 枚を、その層の run の `view_capture` に束ねる。
- View の中の View: host が「各面の視野(円錐)に相手の境界球が入るか」で依存を張り、見る物を先に描く(Main → View A → View B が解ける)。同じコマの循環は検出して各 1 回・固定順で描く(`SameFrameCycle`)。**循環で未だ描かれていない View を何で埋めるかは型付きの seam**(今は束ねない = `view_count()` 0)。再帰の深さは表現の契約に入れない。
- plate: 1 回で撮れる plate は作中カメラの絵を Camera/Stage に出す(`composition_picture` の seam は不変)。コマに別の目がある時は中身も持ち、依頼された View がその目から組み立てる。中身が View を読む plate は view ごとに組み立てる(依存で決める、効果名で分けない)。
- screen pass の長さは層の px(既決 2026-09-13): 密度 = この目に写る層の大きさ ÷ 出力での大きさ。窓が comp と違う Stage/Camera の既存のずれも同じ式で直る。
- WGSL: `view_count()`、`view_origin()`、`view_sample(i, uv, lod)`、`view_lod(roughness)`(`vism/material.wgsl`)。
- `"VIEW_BLUR": "roughness"`(任意、`BACKDROP_BLUR` と同じ形): その float 入力の粗さが `view_sample` の lod の上限。host は View の atlas の mip をその段まで(`backdrop_levels_read`)しか焼かず、`view_sample` は焼いた段で clamp する。宣言する Vism は lod を `view_lod(p.roughness)` で出す(backdrop と同じ写像)。無ければ全段(既存 Vism は不変)。無い名前は名指しで拒否。
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
- Prism Garden(`examples/prism_garden.js`・`vism/prism_view.wgsl`)の gap 台帳(2026-09-24):

| # | gap | 分類 | 状態 |
|---|---|---|---|
| 1 | View は 3D の一部だけ(第二の場面組み立て) | CURRENT_CONTRACT_VIOLATION | 解決: 本番の `record_stack` |
| 2 | 2D が View に入らない | 誤認(2D は作中カメラ平面に居る)+ capture で 2D の順が落ちる IMPLEMENTATION_GAP | 解決: 順も本番どおり。球の View に文字が写る |
| 3 | 効果後の絵が View に入らない | screen pass と plate の効果が落ちる IMPLEMENTATION_GAP | 解決 |
| 4 | OBJ に View が届かない | 誤認: 届いている。`vt` の無い OBJ の uv が 0 | uv を読む Vism には座標が無い: 自動 uv は作らない。`world_position`・`normal`・屈折方向(Lens)で読める。正規化した局所座標を generic な入力にするかは NEW_PRODUCT_SEMANTICS(seam) |
| 5 | FROM が layer だけ | NEW_PRODUCT_SEMANTICS(鏡像・別カメラ) | 未解決(seam) |
| 6 | View 数に比例して GPU が増える | 面ごとの raster は View 依存。run ごとの固定費(1080p で約 2 ms、512² で約 0.24 ms)が同じ仕事の繰り返し | 一部解決(下の数字)。run の束ね方は未着手 |
| 7 | View なしでも GPU 36 ms | 誤帰属: frame_cost の no_effects は Repeater も外す。本当の持ち主は花の輪(Repeater の plate + Glow)が View の 6 面で複製ごとに描かれていたこと | 解決 |
| 8 | 循環の埋め方 | NEW_PRODUCT_SEMANTICS | seam(`SameFrameCycle`) |
| 9 | 面の札が WESL の import を解決しない(block の札だけ) | IMPLEMENTATION_GAP | 未解決(同じ shader を 2 枚に書かないため、View の向きの違う札を作らなかった) |
| 10 | Vism の INPUTS を増やすと書類に編集が積まれ、script の再実行が断られる | IMPLEMENTATION_GAP | 未解決(再起動で回避) |
| 11 | MP4 書き出しの R と B が入れ替わる | IMPLEMENTATION_GAP | 未解決(ffmpeg で戻して納品) |
| 12 | `shelf.prism_view` に棚の見本画像が無い | 既存の失敗試験に 1 件追加 | 未解決 |
| 13 | CPU 1 コマ 5.5 → 10.4 ms: View の面ごとに本番の run を組む | 観測(layout ではない) | 未着手 |

実測(M4、1920×1080、headless warm、同じ書類 = 旧 Prism Garden): GPU 待ち 67.9 → 46.1 ms。持ち主(Metal System Trace): View の面の raster 38.6 → 16.6 ms・View の Blit 7.5 → 0.7 ms・本番 view の raster 30.0 → 30.2 ms・効果と合成 7.6 → 9.8 ms。新しい Prism Garden(板 96・OBJ 7・View 2 層 × 3 = 6 枚、要求 6 = 実体化 6): GPU 48.5 ms・CPU 10.4 ms。release 実窓 843 描画 / 1769 落ち(約 19 fps、前版 約 14 fps)。

## run の境目と 1 コマの pass 数(計測 2026-09-24、Prism Garden 第 2 版、M4、headless warm)

`compositor/view.rs` の `plan_runs` が run(1 つの ViewBuilder が一緒に描く層)の切れ目と理由を返す(純関数)。`SurfaceWork::run_breaks` が理由ごとに数える。`zz_run_cost` が空の run の固定費、`zz_owner_cost` が層ごとの費用と CPU の内訳を出す。

| 切れ目 | 根拠 | 1 コマの数(view 1 + View の面 6) |
|---|---|---|
| Alone(mix blend の絵は単独) | 混色は下を読む(Glow の SPILL "screen") | 28 — 全部 Heart の plate の中 |
| Plate | plate は自分の stack | 0(plate は Flat で切れていた。今は明示) |
| Flat(2D / 非 2D) | 2026-09-12 の法 | 7 |
| Glass | 2026-09-23 の法 | 7 |
| MeshAfterRect | **歴史的**(e761117e6)。今は Transparent 段が rect と mesh を DrawOrder で一緒に並べる | 0 |
| ScreenPasses | 効果は自分の絵だけを見る | 14 |
| ViewOwner | 1 draw に束ねられる View は 1 層分 | 1 |

- 空の run の固定費(GPU): 1080p 約 0.08 ms、512² 約 0.09 ms。前の報告の「1 run 約 2 ms」は誤り。
- 1 コマの GPU pass: view-run 10・layer-view-run 51・vism-pass(合成・効果)124・mip 44・sky 10、合計約 250。
- CPU 10.4 ms の持ち主(`sample`): wgpu の `CommandEncoder::finish`(pass ごとの Metal encoder 生成・pipeline 切替)が View の面 6 枚分で約 4 ms(39%)。run の準備 0.53・記録 0.78・合成 0.12。`before_submit`(staging の upload: plate の中身の mesh instance を stack ごとに再 upload、2178 個/コマ)約 0.7。frame graph・lowering 約 0.4。layout ではない。
- GPU 48.5 ms の持ち主: Heart(2 つの 2D plate を View の面 6 枚が中身から描き直す: 花弁 raster・Glow・spill の alone 4 本/面)16.7、板 96 枚(Prism View の shading の重なり)13、球 7 個(含 3 面)11、残り(海・粒・核・文字・present)11.6。

構造として一度にできる候補(cache ではない):
1. ~~seam~~ **利用者裁定(2026-09-24)**: 「Plate だから平面カードになる、は無し。View から見ればその View の意味で見える」。View は plate の中身をその目から描く(現状の実装)。Plate = semantic isolation + optimization permission であって rasterize 命令ではない(既決)に沿う。16.7 ms は払うのではなく、2〜5 の構造で取り返す。Camera/Stage が plate の作中カメラの絵を見る件は `composition_picture` seam のまま(この裁定は「別 View から見る時」について)。
2. plate の中身の mesh instance を stack 間で共有(`SharedMeshScene` の先例を plate へ): upload 2178 → 約 400/コマ。
3. View の atlas の mip を、Vism が読む段数だけ作る(`BACKDROP_BLUR` と同じ形の `VIEW_BLUR`): 約 −20 pass/コマ。
4. ~~MeshAfterRect の切れ目を消す~~ → **消せない(2026-09-24 実測)**: 消すと `glass_refracts_the_layers_drawn_behind_it` と `gpu_instance_subsets_preserve_rect_mesh_boundaries_and_surface_parameters` の絵が変わる(rect と mesh を 1 つの ViewBuilder に入れると同じ絵にならない)。さらに「先の 3D の絵が後の mesh より手前にある」場面で、切っても切らなくても絵が手前に出ない(中心が mesh の色)。rect と mesh の深度の意味(Opaque 段の mesh と Transparent 段の rect)は未監査 = gap。境目は残す。
3. ~~View の atlas の mip を、Vism が読む段数だけ作る(`BACKDROP_BLUR` と同じ形の `VIEW_BLUR`): 約 −20 pass/コマ。~~ 施工(上の契約、`surface_tests::view_mips_stop_where_the_declared_roughness_stops_reading`)。
4. MeshAfterRect の切れ目を消す(歴史的。絵の試験 1 本と一緒に)。
5. run を中間 canvas なしで stack に直接描く(MSAA の target に load): fork の seam(UPSTREAM_SEAM)。今回はしない。

### 裁定 B を本番経路で有効にした後の実測(2026-09-24、`eafbf7144`、Prism Garden 第 2 版)

| | 直前(View は plate をカードで見ていた) | 今(中身をその目から) |
|---|---|---|
| GPU/コマ(headless warm) | 48.5 ms | 72.2 ms |
| CPU/コマ | 10.4 ms | 13.3 ms |
| run/コマ | 66(空 7・alone 28) | 45(空 0・alone 0・plate 12) |
| View atlas の mip pass | 20 | 2 |
| release 実窓 | 約 19 fps | 約 14 fps(745 描画 / 2420 落ち) |

増分の持ち主: Heart の plate 2 つ(各 約 20 ms)。View の面 6 枚それぞれで、面いっぱいに重なる花弁(曲線の塗り、MSAA 標本ごとの shading)を描くため。Glow の鎖は 約 7 ms。粒の plate 約 3 ms。板の窓の中には光る花が 3D で写る(カードではない)。
取り返す候補(裁定待ち): rect の per-draw blend(Screen/Add)、透明 mesh instance の共有(C の patch)、mesh の標本ごと shading を画素ごとへ(fork が上流の既定を変えている件、絵の縁が変わる)。
