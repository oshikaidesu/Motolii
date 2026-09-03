# 2026-09-03 売り込み 第 2 波(営業 13 社)— カタログを持った利害関係者

利用者の依頼: 第 1 波(rerun・Dioxus/Blitz、[vendor-pitches](2026-09-03-vendor-pitches.md))が「feature 2 語で 20 件級の宿題」を掘り当てたので、同じ形で全社を回す。営業 = **カタログを持った利害関係者**が、売る動機で code を読み、file:line で裏を取り、「弊社に無い物」も白状する。買い手 1 名(rerun 上流)は逆向き。全員読み取り専用、code も backlog も触っていない。宿題番号は [persona-backlog](2026-09-03-persona-backlog.md)。

## 要約(1 社 1 行)

| 社 | 一番の売り | 正直な限界 |
|---|---|---|
| vello / peniko | custom widget は既に `draw_glyphs` を記録できる(BU4 は誤診)。parley 1 行 + ≒75 行で V8 目盛の数字・ST10・S12 が窓に出る | blur / filter は窓の経路に届かない(GC5)。問い返しで Stage の形は re_renderer 側へ移し、vello は文字だけ |
| cosmic-text / fontdb | `load_system_fonts` + fallback + `set_rich_text` + `set_wrap` で LD4・S6・S8・S10 が ≒70 行。locale "en-US" 固定で Han が PingFang に落ちている | 縦書き(S4)・ルビ(S9)は無い。Justify は空白配分で日本語に効かない |
| naga | 実行時 `.fs` 読み込み(SE6)は naga 側 0 行。行番号付きの診断 `emit_to_string` と INPUTS/本文の齟齬検出が既に払った代金の中 | `#include` 無し、GLSL 4.5 のみ |
| ffmpeg | ProRes 4444・PNG 連番・GIF は flag 表 ≒25 行(X1)。`unpremultiply` 1 語で X5 | sidecar の download は署名済み .app を壊す・ffprobe が付かない。SE2 の法務は残る。decision-index:361 の sidecar 不採用と thumbnail.rs が矛盾 |
| cpal / symphonia | seek で device を捨てなくてよい(AU1)、既定出力の切替追従は 0 行(AU2)、feature 4 語で ADPCM・id3v2(cover art 付き MP3 が今落ちる) | BPM 検出・ループ・スクラブ音の判断は無い。Opus / AC-3 は読めない |
| accesskit / kittest | VoiceOver の押下が blitz-shell で **捨てられている**(application.rs:87 TODO)。≒55 行で「試験も読み上げも同じ ActionRequest」 | kittest は探すだけで押せない。custom widget の木は上流 PR 待ち |
| dioxus-dnd(2 度目) | **前回 B8 を撤回**: FileDrop 系は blitz で死んでいる。pointer 系(Draggable / DropZone / Sortable)は動く見込み。M8・C1・F20 が新規 ≒270 行 | 置換ではなく新規。窓で実走はしていない。問い返しで Stage の Canvas 系は取り下げ |
| Apple | winit は app delegate を空けている。NSMainMenu + `applicationShouldTerminate` で H1〜H4・H12・H14 が ≒100 行、⌘Q の未保存素通りも直る。Core Text の列挙で書体の再配布問題が消える | AVFoundation の crate は本機に無く未検証。NSDocument の自動保存は載らない |
| Lottie 仕様 | 書き出しは完成品で呼ぶ側が 0 件(SE5)。S7 の Range Selector は仕様と model が一対一で対応表済み。**縁取り・blend は落ちない**(依頼文の誤り) | 取り込みは無い。`lottie_coverage.rs` の照合試験が tree に無い |
| ISF 仕様 | bool を int に・color を 4 成分にで公開 327 本の compile が 210→277(IS1)。audioFFT で A 波と繋がる。実行時読み込みは仕様の前提 | PERSISTENT(49 本)は裁定と相容れない。効果の鎖は仕様外 |
| vgpu(外部証拠) | 交換の前の検証が無い: `vgpu check` の形を naga で写して admission gate に(VG1)。tri_led の `glow` は INPUTS にあるが本文で束縛されておらず空回り(VG2)。param を 1 struct に畳む(VG3)、prelude を `#import` にして staging を消す(VG4) | last-good を保つ原子的交換そのものは弊社にも無い。1pass 畳みは他の vism に転用できない |
| After Effects(model) | Adjustment / Shy / Motion Blur は **kind でなく flag**、`LayerAttrs` に 3 つ(AE1)。`PropertyId` の prefix 規約を `property_tree()` に格上げすれば P/S/R/T/A・値グラフ・S7 が同じ木(AE2)。work area(AE4)、undo 群の名前(AE5) | Expression は売らない(PropertyLink が既に型付きで持つ)。precomp も売らない |
| rerun 上流(買い手) | fork +1461 行のうち Motolii が呼ぶのは re_renderer 5 file だけ、≒750 行は egui 期の遺物。UP1(from_device)は即引き取り、UP2 は閉包署名を変えずに分割、UP4 は複製 180 行を畳んでから | rebase の負債は dynamic_resource_pool.rs の署名変更が最大。2 rev の checkout が dx の 7 GB の一因 |

## 横断で見えた物

- **誤診の訂正が 3 件**: BU4「widget が文字を描けない」(描ける、無いのは字形)、B8「dnd で新規 0」(FileDrop は死んでいる)、Lottie「縁取り・blend が落ちる」(落ちない)。営業に file:line を強いる形が効いている。
- **同じ穴を 3 社が別の道で塞ぐ**: 書体(cosmic-text CT1 / Apple AP4 / vello VL2)、実行時 .fs(naga NG1 / ISF IS7)、音の切替(cpal AU1 / Apple AP5)、サムネイル(ffmpeg FF6 / rerun R6)。正本を 1 つ選ぶ裁定が要る。
- **既に払った代金で未使用**: naga の診断、symphonia の id3v2、cpal の DefaultOutputMonitor、accesskit の ActionRequest、Lottie の export 本体、fontdb の system fonts。feature / 配線の話で新規 code は薄い。
- **問い返し「Stage は re_renderer の世界に入るか」**(利用者)を Stage に手を出した 3 社へ返した。vello: 枠・取っ手・回転帯は (a) LineDrawableBuilder / PointCloudBuilder で世界の中に置き R2/R3/R8 と同じ段へ、文字だけ (b) 2D(rerun 本体も label は egui で描く)、取っ手の box shadow は取り下げ(R3 の OutlineConfig の役)。dioxus-dnd: 入らない。CanvasDropZone・SnapGrid・Bounds::clamp は Stage で使わず、素の DropZone が要素基準の 2D 点を 1 回渡し、写像は `Fit::to_comp` / `at_z`、格子は R5 の WorldGrid。cosmic-text: (a) 成立、`plane_map` の homography で UV に戻し `Buffer::hit(px, py − dy)` の 2 段。
- **裁定待ちに上がった物**: precomp(AE・Lottie の 2 社が別々に指した)、PERSISTENT(ISF)、sidecar 不採用の再裁定(ffmpeg)、fork の spikes pin 落とし(rerun 上流)。

---

## vello / peniko(vello 0.10.0、peniko 0.6.1)

Motolii の窓は `dioxus-native` の `vello` feature(motolii/Cargo.toml:58)→ anyrender_vello 0.14.0 → vello 0.10.0 で描く(Cargo.lock)。custom widget 3 つ(stage_widget.rs:1171、timeline_widget.rs:1255、ease_widget.rs:226)はどれも `anyrender::Scene` を返し、それが anyrender_vello の `VelloScenePainter` で vello の `Scene` に写る。売り込みはこの経路の上に限る。

### 今日、既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| VL1 | 「custom widget が文字を描けない」(BU4)。timeline_widget.rs:1292-1320 の目盛は `fill_rect` の縦線だけ、数字無し(V8) | widget が返す `anyrender::Scene` は既に `draw_glyphs` を記録する(anyrender-0.13.0 recording.rs:251、trait は lib.rs:232-245: `font: &FontData, font_size, normalized_coords, glyphs: Iterator<Glyph{id,x,y}>`)。anyrender_vello-0.14.0 scene.rs:139-172 がそれを vello-0.10.0 scene.rs:455 `Scene::draw_glyphs` → `DrawGlyphs`(487-624)へ流す。**描く口は塞がっていない**。塞がっているのは blitz-paint text.rs:541 `stroke_text` が `pub(crate)` で、CSS の文字を widget から借りられない事だけ | BU4 の再定義。上流 PR 不要 |
| VL2 | 無いのは字形と字送り。Motolii は parley / FontContext をどこでも持たない(grep 0 件)。窓の書体は styles.css:100 `-apple-system` | 手本は blitz-paint text.rs:571-635(parley の `run.font()` / `font_size()` / `normalized_coords()` / `positioned_glyphs()` を `draw_glyphs` に渡す 20 行)。parley 0.11.1 は Cargo.lock に在り(blitz 経由)、`FontData` は parley lib.rs:132 と peniko lib.rs:43 が同じ物を re-export。`FontContext` は blitz-dom lib.rs:96 が re-export。共有は `DocumentConfig.font_ctx`(blitz-dom config.rs:49、document.rs:366-370 で受ける)— Motolii は host.rs:307 / gui.rs:55 で `DocumentConfig` を組んでいるので、そこで 1 つ作って窓と widget に clone を渡す(Document 側の `font_ctx` は document.rs:245 で `pub(crate)`、後から取り出す口は無い) | Cargo に `parley` 1 行 + 小さな `Type`(FontContext + LayoutContext、`fn glyphs(text, px) -> (FontData, Vec<Glyph>)`)≒35 行 |
| VL3 | V8 目盛の数字 | VL2 の `Type` を timeline_widget.rs:1305-1310 の主目盛ループで呼ぶ。級数は tokens.rs:23 `TEXT_DENSE`、色は 1283 `c_text` | ≒15 行、V8(R1 の `format_compact` と組むと表記も揃う) |
| VL4 | ST10 変形中の数値。stage_widget.rs:180 / 1084 に `cursor` は在るが、1477-1500 の取っ手は塗るだけ | 同じ `Type` で `lit` の取っ手(1493-1497)の脇に値を 1 run | ≒15 行、ST10 |
| VL5 | S12 印の名前(V8 と同根で未)。timeline_widget.rs:1321-1335 は印を矩形で描く | 同上、印の右に 1 run | ≒10 行、S12 |
| VL6 | 選択枠と取っ手は平らな矩形(stage_widget.rs:1495-1500)。指の下は白く 4px 大きく | `PaintScene::draw_box_shadow`(anyrender lib.rs:248)→ vello scene.rs:256 `draw_blurred_rounded_rect`(gaussian、`std_dev`)。lit の取っ手の下に 1 回敷けば「光る」。ST5 のアンカー取っ手(1481-1487 の十字)にも同じ | 各 1 行、ST5・ST11 の見え方 |
| VL7 | compositor.rs:43-62 が blend の番号(mix<<8 \| 3)を手写し。wgsl の prelude は `reference/vello-blend.wgsl` を既に借りている(wgsl_fragment.rs:23) | peniko-0.6.1 blend.rs:10 `#[repr(u8)] enum Mix`(Multiply=1 … Luminosity=15、86 行目)と blend.rs:113 `Compose::SrcOver = 3` — 同じ番号体系。`(Mix::X as u32) << 8 \| Compose::SrcOver as u32` | 数字 16 個が名前になる。行数は ≒10 減、prelude と番号の正本が 1 つになる |

### 検討の価値はあるが、売りにくい物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| VL8 | blend_preview.rs(129 行)が W3C の式を CPU で 1 画素解く(C4 は ☑、R9 で「CPU の式で代替」と裁定済み) | vello_cpu 0.1.0 は Cargo.lock に在る(anyrender_vello_cpu 経由)。render.rs:471-477 `push_layer(clip, blend_mode, opacity, mask, filter)` + 278 `fill_path` + 601 `set_paint` で Pixmap に本物の合成を焼ける。画素単位の `mix` は fine/lowp/blend.rs:9 で `pub(crate)` なので、式だけを借りる道は無い | 129 → ≒40 行だが直接依存が 1 本増え、裁定と食い違う。札を vello の合成と画素一致させたい時だけ |

### 弊社に無い物(正直に)

- **blur・glow・filter(GC5)**: anyrender_vello-0.14.0 scene.rs:69-79 の `push_layer` は `_filter` / `_backdrop_filter` を捨てる。vello 0.10.0 の `Scene` に filter は無く、あるのは blurred rounded rect(scene.rs:256-315)だけ。vello_common-0.1.0 の `filter/`(gaussian_blur.rs・drop_shadow.rs)と vello_cpu render.rs:555 `push_filter_layer` は cpu / hybrid 経路の物で、窓の経路(anyrender_vello → vello classic)には届かない。`vism/glow.wgsl` は残る。
- **gradient**: peniko gradient.rs:344-393(linear / radial / sweep)は `PaintRef::Gradient` で widget から使えるが、Motolii で勾配が要る所は色相環と S/V 面で、それは CSS(styles.css:1599 `conic-gradient`、color.rs:312 / 367 `linear-gradient`)で既に動いている。`vism/gradient.wgsl` は作品側の効果で、窓の Scene の話ではない。売る物が無い。
- **stroke**: 破線(stage_widget.rs:1444 `with_dashes`)も菱形(timeline_widget.rs:935-944 `rotate_about`)も kurbo で既に正しい。回転帯(1537-1543 の 64 分割)は `plane_map` の非アフィン写像を通るので `kurbo::Ellipse` には置き換えられない。
- **文字組み**(S4 縦書き・S8 runs・S9 ルビ・S10 折り返し)は parley の領分で、弊社(描画)の外。作品の文字は cosmic-text + tiny-skia(engine/text.rs、Cargo.toml:101-102)で焼いており、そこは触らない。
- **当たり判定・a11y**: `Scene` は描くだけ。BU5 の `accessibility_tree`(custom_widget.rs:143 コメントアウト)は blitz 側。

### 上流に無い物 — 弊社 PR で対応

| # | 内容 | Motolii 側の影響 |
|---|---|---|
| VLU1 | blitz-dom `Widget::paint`(node/custom_widget.rs:130-137)が `styles` は渡すが `FontContext` を渡さない。`Document.font_ctx` は document.rs:245 `pub(crate)` で accessor 無し | PR: `paint` の引数か `Document::font_ctx()` 1 本。通れば VL2 の「config で clone を配る」配線(host.rs:307・gui.rs:55)が消える。無くても VL2 で成立する |
| VLU2 | blitz-paint text.rs:541 `stroke_text` は `BaseDocument` と node id を要るので、公開しても widget からは呼べない | BU4 は VLU1 に書き換えるのが正確。text.rs の公開は要らない |

見込み: Cargo 1 行 + 新規 ≒75 行(VL2〜VL5)で V8・ST10・S12 が窓に出る。VL6 は各 1 行、VL7 は ≒10 行減。上流 PR 待ちは 0(VLU1 は配線を短くするだけ)。

注記: Motolii 側の行番号は 2026-09-03 の作業ツリー(color.rs・styles.css は他セッションが編集中で、その 2 件の行はずれ得る)。widget 3 file と compositor.rs は未変更。

### 問い返し: Stage は re_renderer の世界に入るか(vello)

先に今の構造。ViewBuilder は motolii-render の中で回る(render_basic.rs:86、sequential.rs:313 / 411 / 486)。stage_widget.rs はその解決済み texture を `try_register_custom_resource`(1236-1237)で handle にし、`ImageBrush` として貼る(1428-1434)。枠・取っ手・帯は**その後**に vello で描くが、座標は `camera_screen_from_world_z0`(1362 / 1380)と層平面の射影写像 `plane_map` / `homography_from_unit_square`(1566-1600)で同じカメラから出している。つまり今の overlay は「視点と一緒に回る 2D」— 回るが、奥行き判定と outline / picking の処理段には入っていない。

re_renderer に文字は無い: `crates/viewer/re_renderer/src` に glyph / font を含む file は 0、`shader/` にも text 用の wgsl は無い。rerun 本体の label は egui の galley(re_view_spatial/src/ui.rs:242-262 `fonts.layout_job`)で、3D の外に描いている。

| # | 判定 | re_renderer での形 | 見込み |
|---|---|---|---|
| VL1 / VL2(glyph の口と `Type`) | (b) | re_renderer に glyph draw data が無い。rerun 自身も label を egui(2D)に出す(ui.rs:242-262)。Stage の数字は「世界の中に置く」先例が上流に無い | Timeline と共用の `Type` を Stage でも使うのが上流と同じ形 |
| VL4(変形中の数値) | (b) | 同上。値は取っ手の脇の 2D 文字。位置は既存の `l2s`(stage_widget.rs:1470-1475)= 同じカメラ射影なので視点と一緒に動く。奥行きで隠れない事は値の表示では利点 | ≒15 行、ST10。(c) にするなら re_renderer に glyph atlas + text renderer を新設(renderer/ の 1 本、点群級の規模 ≒600 行以上)— 売れない |
| VL6(取っ手の box shadow) | (b) だが売り取り下げ | re_renderer に blur は無い(shader/ に無し)。光らせるなら R3 の `OutlineConfig { outline_radius_pixel, color_layer_a, color_layer_b }`(outlines.rs:106-112)の縁取りが世界の中の形。取っ手そのものは下の行 | VL6 は R3 と競合するので取り下げ。R3 が入るまで vello の 1 行で仮置きできる、という位置 |
| 枠(1454-1475 の outline、1444 の破線) | (a) | `LineDrawableBuilder::add_rectangle_outline(top_left, extent_u, extent_v)`(line_drawable_builder.rs:402-419)を層の `world_from_obj`(227)で置く。太さは `.radius(Size::new_ui_points)`(513、size.rs:43)で視点に依らず 1.5px。`.outline_mask_ids`(540)を付ければ R3 の縁取りと同じ処理段に入る。破線 flag は無い(`LineStripFlags` に dash 無し)— 書き出し枠の破線は再現できない | 枠 ≒20 行を motolii-render 側へ移す。奥行きで正しく隠れ、ST20 の「paint でしか更新されない fit」も VB 側に寄る |
| 取っ手(1477-1500 の 8 個の矩形) | (a) | 同じ builder の `add_axis_aligned_rectangle_outline_2d`(476)か `PointCloudBuilder::add_points`(point_cloud_builder.rs:188、`radius_boost_in_ui_points_for_outlines` 59)。`picking_object_id`(241)で R2 の picking に乗り、当たり判定と描画が同じ物になる(ST11 の再発を型で止める、R8) | ≒30 行、R2・R8 と同じ PR に載る |
| 回転帯・奥行きの取っ手(1533-1550) | (a) | 帯は `add_strip`(274)に 64 点を世界座標で渡す(射影は GPU がやるので 1566-1600 の射影写像が要らなくなる)。奥行きの柄は `add_segment`(289)+ 点 1 個 | ≒25 行減(homography の CPU 実装は掴む側にまだ要る) |
| VL7・VL8(blend 番号、CPU 札) | 対象外 | Stage の描画ではない | — |

結論: **形(枠・取っ手・帯)は (a)** — re_renderer の LineDrawableBuilder / PointCloudBuilder に世界座標で渡し、R2 / R3 / R8 と同じ段で描く。これは第 1 波の rerun 営業と競合せず同じ物で、vello 側では売らない。**文字は (b)** — re_renderer に text は無く、上流(rerun 本体)も egui の 2D で描いている。Stage の数値(VL4)は vello の `draw_glyphs` で 2D に置くのが上流と同じ形で、これは Timeline の `Type`(VL2)の使い回し。**(c) は薦めない** — re_renderer に text renderer を足すのは点群級の新設で、Motolii が持つ行数ではない。VL6 は取り下げ、その役は R3 の `OutlineConfig`。

## cosmic-text / fontdb(cosmic-text 0.19.0、fontdb 0.23.0)

`$C` = `cosmic-text-0.19.0/src`、`$F` = `fontdb-0.23.0/src`(どちらも `~/.asdf/.../registry/src/index.crates.io-1949cf8c6b5b557f/`)。Motolii 側は `crates/motolii-doc/src/vector/text.rs` = `text.rs`、`crates/motolii-render/src/engine/text.rs` = `engine/text.rs`。

Motolii の今(共通の前提): `text.rs:83-89` が呼ぶたび `fontdb::Database::new()` → `load_font_file(1 本)` → `FontSystem::new_with_locale_and_db("en-US", db)`。`engine/text.rs:84` は `styles.first()` だけ、`runs` / `ranges` / `axes` は読まない(Lottie 書き出し `export/lottie/text.rs:17-22` が「在るなら断る」判定に使うのみ)。書体は `src/ui/browser.rs:295-298` と `crates/motolii-doc/src/fixture.rs:413` に path 直書き。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| CT1 | `text.rs:83-88` fontdb に 1 file、`browser.rs:295` の `.ttc` 絶対 path。無ければ `TextShapeError::FontFile` で描画が止まる(LD4) | `$F/lib.rs:400-450` `Database::load_system_fonts()` — macOS は `/Library/Fonts`・`/System/Library/Fonts`・`/Network/Library/Fonts`・`~/Library/Fonts` を全部積む(lib.rs:427-450)。`FaceInfo`(lib.rs:811-855)に `families`・`post_script_name`・`style`・`weight`・`stretch`・`source`・`index` が揃うので、`FontRef { family, style }` を CSS 風 `Database::query(&Query { families, weight, style, stretch })`(lib.rs:661-684、`Query` は 929-948)で引ける。path は fingerprint 兼用の副情報に降格できる | `text.rs` の 6 行が `Family::Name(&family)` + `Weight` へ、`browser.rs:295` の path 行が消える。LD4・S6 |
| CT2 | 1 file しか無いので、その書体に無い字(絵文字・欧文・記号)は glyph 0 | `FontSystem::new_with_locale_and_db_and_fallback`(`$C/font/system.rs:213`)+ `PlatformFallback`(`$C/font/fallback/macos.rs:11-58`): Hiragana / Katakana は locale に関係なく "Hiragino Sans"(macos.rs:82-85)、Han は locale "ja" で Hiragino Sans(macos.rs:48)、共通の後詰めは ".SF NS"・"Apple Color Emoji" 等(macos.rs:30-38)。実際の差し替えは `$C/shape.rs:119` `shape_fallback` と `shape.rs:522` が `get_font_matches`(`system.rs:359`)を script ごとに回す。**locale は `system.rs:421-424` が `sys_locale` から取る** — 今の `"en-US"` 固定(text.rs:89)だと Han が PingFang SC に落ちる(macos.rs:56) | CT1 と同じ変更に乗る。`"ja"` を渡す 1 行、LD7 の locale |
| CT3 | `text.rs:83-89` が呼ぶたび DB と FontSystem を作り直す(LD7)。`system.rs:188-191` の doc は「release で最大 1 秒、一度だけ作って共有せよ」 | `FontSystem` を engine に 1 つ持つ(`Engine` は `engine.rs:109` に `shape_textures` の cache を既に持つ)。`font_matches_cache`(system.rs:359-372)と feature `shape-run-cache`(`$C/Cargo.toml` features、`system.rs:163-164` `shape_run_cache: ShapeRunCache`)で同じ行の再 shape が消える | 新規 ≒10 行、削除 ≒6 行。LD7 |
| CT4 | S6 ウェイト: `FontRef.style: "W3"`(store/text.rs:13)は文字列で、shaper に渡していない(text.rs:103-107 は family と metrics だけ) | `Attrs::weight / style / stretch`(`$C/attrs.rs:335-347`)。可変フォントは `$C/font/mod.rs:141-143` が `wght` 軸に `weight` を写し、輪郭取得も `$C/swash.rs:94-109` で `normalized_coords` に同じ weight を入れる — Motolii が使う `get_outline_commands`(swash.rs:175)と同じ経路 | `style` → `Weight(u16)` の変換 ≒8 行。S6、`TextStyleAxis`(store/text.rs:120)は `wght` だけ弊社が写す |
| CT5 | S8 runs: `TextRun { len, style }`(store/text.rs:224-227)と `validate_runs`(282-300)は在るが `engine/text.rs:84` は先頭 style のみ | `Buffer::set_rich_text(spans: (&str, Attrs), default_attrs, shaping, align)`(`$C/buffer.rs:1102-1110`)。内部は `AttrsList::add_span(Range, &Attrs)`(`attrs.rs:531-539`)。級数・行送りは span 単位に `Attrs::metrics`(attrs.rs:365)、色は `Attrs::color`(attrs.rs:323)→ `LayoutGlyph.color_opt`(`$C/layout.rs`) | `runs` を `(len → &str, Attrs)` に切る ≒25 行。S8。fill/stroke は Motolii 側で glyph ごとに `metadata` で振り分け(CT6) |
| CT6 | S7 Range Selector: `TextRangeSelector`(store/text.rs:171-176)まで model 在り、描画無し。`LineMeasure.glyph_xs`(text.rs:69, 129)は誰も読まない(LD6) | `LayoutGlyph`(`$C/layout.rs`)に `start / end`(元の行の byte 範囲)・`x / y / w`・`font_id`・`metadata`(`Attrs::metadata(usize)` attrs.rs:353 から素通し)。`based_on: Words` は `$C/shape.rs:24` が `unicode_segmentation` を使う。`Lines` は `LayoutRun.line_i`(buffer.rs:36-55) | `LineMeasure` を「glyph ごとの (start,end,x,w,metadata)」に置き換え ≒15 行。S7 の描画側 L8 の入口、`commands_to_contours` を glyph 単位に分けるだけ |
| CT7 | S10 折り返し: `wrap_size`(store/text.rs:233)は `engine/text.rs:93` で幅だけ `set_size` に渡り、`Wrap` は既定のまま | `Buffer::set_wrap(Wrap)`(buffer.rs:759)、`Wrap::{None, Glyph, Word, WordOrGlyph}`(`$C/layout.rs:128-136`、既定 `WordOrGlyph` buffer.rs:392)。改行機会は `$C/shape.rs:970` `unicode_linebreak::linebreaks`(UAX #14 — 行頭の「。、」は既に禁則、LD9 の最低限)。高さは `set_size(_, Some(h))` buffer.rs:818 | 点文字は `Wrap::None`、枠文字は `Word` の 1 分岐 ≒5 行。S10 |
| CT8 | S5 Justify: `TextJustify` は Left/Right/Center(store/text.rs:17-21) | `Align::Justified`(`$C/layout.rs:152-156`)。**ただし `shape.rs:2826-2832` は空白の数で配分し、最終行は除外** — 日本語の歌詞(空白無し)では効かない | enum に 1 枝 + Lottie 側 `enums.rs:107`。欧文の S5 のみ |
| CT9 | LD9 palt: `features: ["palt"]`(browser.rs:309)は `FontFeatures`(text.rs:94-101)で既に届く | harfrust へ `Feature` として渡る(`$C/shape.rs:158-163`)。既定 on は Motolii 側の 1 行 | 0 行、既に弊社経由 |
| CT10 | ヒット判定: 印を掴む(S12)は menu 側、本文の文字位置は無い | `Buffer::hit(x, y) -> Option<Cursor>`(buffer.rs:1144)、`layout_cursor`(659)、`cursor_motion`(1260) | Stage で歌詞の文字を直接掴む(ST10 の文字版)の足場、新規 0 |

弊社に無い物(正直に):

- **縦書き(S4・LD9)**: `$C` 全体で `vertical` は `Scroll.vertical`(cursor.rs:142)と `Motion::Vertical`(cursor.rs:117)だけ。`harfrust::Direction` は `shape.rs:137-139` で LTR/RTL の 2 択、`vert` / `vrt2` の feature も `LayoutDirection`(shape.rs:1244)も横組み専用。縦書きは弊社の外(Motolii 側で glyph 単位に回して積む: CT6 の形が在れば `x` を `y` に読み替える自作になる)。
- **ルビ(S9)**: 無い。`set_rich_text` は 1 段の流し組み、行の上に小段を置く機構は無い。CT5 の span で級数を変えられても、親文字への位置合わせは Motolii 側。
- **wght 以外の可変軸**: `font/mod.rs:143` と `swash.rs:94-109` は `wght` だけ。`TextStyleAxis`(store/text.rs:120)の他の tag(wdth・slnt)は届かない。
- **禁則の追い込み・ぶら下げ**: UAX #14 の break 機会まで。日本語組版の詰め(JIS X 4051)は無い。
- **描画**: 弊社は輪郭 `Command`(swash.rs:175)まで。縁取り 2 度焼き(engine/text.rs:113-130)は今のまま。

見込みの合計: `text.rs` / `engine/text.rs` へ ≒70 行の差し替え(削除 ≒15)で LD4・LD7・S6・S8・S10 と S7 の入口、feature 1 語(`shape-run-cache`)。S4・S9 は弊社に無い。

### 問い返し: Stage は re_renderer の世界に入るか(cosmic-text)

判定は **(a) 成立、2 段**。ただし 1 段目は PickingLayerProcessor を待たなくても今の Motolii に既に在る。

1 段目(3D → 層の UV): `src/ui/stage_widget.rs:1627-1666` `plane_map` が、compositor と同じ `tilted_corners`(`crates/motolii-render/src/compositor/render_basic.rs:41-51` の `TexturedRect { top_left_corner_position, extent_u, extent_v }` と同じ式)で層の 4 隅を `camera_projection`(1642-1643)で窓へ落とし、`homography_from_unit_square` の逆 `uv_from_screen`(1666)を作る。`Fit::to_uv(sx, sy)`(stage_widget.rs:100-102)が pointer → 層の UV。rerun fork の `PickingLayerProcessor::picked_world_position`(`re_renderer/src/draw_phases/picking_layer.rs:62`)は今 Motolii では未配線(`compositor/point_cloud.rs:43` が既定 id を詰めるのみ)で、R2 が入れば「どの層か」を正しく選ぶ側に効く。層が決まった後の UV は今の homography で足りる。

2 段目(UV → Buffer::hit): 文字層の texture は `engine/texture.rs:140-145` で `Canvas { width: comp.width, height: comp.height, origin 0,0 }`、層の `size` も同じ(texture.rs:151, 172-174)。よって `(u × comp.width, v × comp.height)` がそのまま texture の画素 = `shape_text` の layout 座標。ずれは 1 つだけ: `engine/text.rs:97-101` が塊を縦中央へ `dy` 動かしている(輪郭の点だけを動かし Buffer は動かさない)ので、`Buffer::hit(px, py − dy)`。x は `set_size(wrap_width)`(text.rs:117、幅は `wrap_size[0]` か `canvas.width` engine/text.rs:93)の左端起点で補正不要。

条件: `text.rs:115-143` は `Buffer` を関数内で捨てる。hit を使うには CT3 の共有 `FontSystem` と一緒に Buffer(または `LayoutGlyph` の写し)を層ごとに保持する必要がある — CT6 と同じ保持で賄える。

CT6 について: 同じ理由で使える。`LayoutGlyph.x / w` と `LayoutRun.line_top / line_height`(`$C/buffer.rs:36-55`)は texture 画素の矩形なので、`plane_map` の `screen_from_uv`(stage_widget.rs:1666 の逆)に `(x/comp.width, (line_top+dy)/comp.height)` を通せば 3D 平面上の四角として窓に出る。回転・rotation_x/y・z は homography が吸うので文字ごとの追加変換は無い。

## naga(29.0.4)

前提: Motolii は既に naga 29.0.4 を `glsl-in` + `wgsl-out` で直接依存している(`motolii/Cargo.toml:84`、`crates/motolii-render/Cargo.toml:23`)。Cargo.lock 上の naga は 1 本で wgpu 29.0.4 と同じ版(`Cargo.lock:4033-4035`、`8805-8807`)— 二重 naga は無い。使っているのは `isf/mod.rs:303-320` の 3 呼び出し(`Frontend::parse` → `Validator::new(ValidationFlags::all(), Capabilities::all())` → `back::wgsl::write_string`)だけ。

Motolii の今: `.fs` は `crates/motolii-render/build.rs:8-40` が `vism/` を走査して `include_str!` に焼き、`effects/mod.rs:15-21` の `VismSource { source: &'static str }` で持つ。ヘッダの JSON が壊れていれば `effects/mod.rs:50-52` で **panic**、GLSL が壊れていれば `device.rs:29-38` で `CompositorError::Isf(String)` になり、`src/ui/stage_widget.rs:880` の `println!("PROBE … engine-error")` で終わる(窓に出ない、Engine ごと立たない)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| NG1 | `.fs` は build 時に焼く(`build.rs:35`、`VismSource.source: &'static str` `effects/mod.rs:18`)。SE6 | `Frontend::parse(&Options, &str)` は文字列を受けるだけで build 時の前提が無い(`front/glsl/mod.rs:197-201`)。同じ `Frontend` を使い回せる(`mod.rs:159-165`)。生成 WGSL を実ファイルに置く道は `vism.rs:54-71 stage_source_on_disk` で既にある | naga 側 0 行。Motolii は `VismSource` を `String` にし dir 読みを足す ≒30 行 |
| NG2 | 失敗は `errors.to_string()` / `e.to_string()`(`isf/mod.rs:310`、`:317`)。`ParseErrors` の `Display` は各 error の `{e:?}` 連結で行番号が無い(`front/glsl/error.rs:236-240`) | `ParseErrors::emit_to_string(source)`(`error.rs:208-225`)が codespan で行・列・該当行を描く。1 件ずつ要るなら `Error.location(source)` → `SourceLocation { line_number, line_position, offset, length }`(`error.rs:157-159`、`span.rs:80-135`)。検証側も `WithSpan<ValidationError>::emit_to_string` / `location`(`span.rs:240-247`、`:308-325`)。parse は 1 件で止まらず全件を溜める(`mod.rs:207-219`、`error.rs:165-167`) | ≒10 行。文言は英語(窓の規則・D11)。行番号は Motolii が前置する prelude(`isf/mod.rs:261-297`)の行数を引けば利用者の `.fs` の行になる |
| NG3 | 束縛は manifest の並びから Motolii が決め打ち(`vism.rs:36-47`、`isf/mod.rs:265-294`)。INPUTS と本文の食い違いは wgpu が後で落とすまで分からない | `Module.global_variables`(`ir/mod.rs:2854`)の `GlobalVariable { name, binding: Option<ResourceBinding{group,binding}>, ty }`(`ir/mod.rs:1167-1187`)で宣言側を読める。本文が実際に読む uniform は `ModuleInfo` → `FunctionInfo[global] : GlobalUse::READ`(`valid/analyzer.rs:130-140`、`:326-329`)。本文が未宣言の名を使えば `ErrorKind::UnknownVariable`(`error.rs:95-96`)が名前入りで返る | ≒20 行で「INPUTS に在るが未使用」「本文が使うが INPUTS に無い」を実行時に人の言葉で。束縛番号の規約自体は Motolii のまま |
| NG4 | `IMG_THIS_PIXEL` / `IMG_NORM_PIXEL` / `name → sampler2D(tex, samp)` は文字列連結の `#define`(`isf/mod.rs:278`、`:296-297`) | `Options.defines`(`front/glsl/mod.rs:52-61`)が pp-rs へ `add_define` される(`lex.rs:35-41`)。pp-rs 0.2.1 は関数形マクロを解く(`pp.rs:221`、`:733-735`)。`sampler2D(tex, samp)` は `MacroCall::Sampler` が image→sampler の対応表に入れ(`builtins.rs:1596-1599`)、`texture()` で `ImageSample` に結ぶ(`builtins.rs:2071`)— 今動いているのはこの経路 | 0〜10 行。注意: `add_define` の失敗は `unwrap`(`lex.rs:38` TODO)なので define は Motolii の固定集合に限る |
| NG5 | `Capabilities::all()` で検証(`isf/mod.rs:314`)。wgpu は device の feature から caps を作り直して再検証する(`wgpu-naga-bridge-29.0.4/src/lib.rs:181-189`)。re_renderer の `create_shader_module` は「エラーは非同期、submit まで分からない」(`shader_module_pool.rs:94-101`) | `wgpu_naga_bridge::features_to_naga_capabilities` / `create_validator` は pub(同 `lib.rs:181`)。device の features を渡せば Motolii の検証と wgpu の検証が一致し、submit 前に落とせる | Cargo.toml に 1 行(lock には既に在る)+ 引数 1 つ |

弊社に無い物(正直に):
- ISF の JSON ヘッダ・`PASSES`・`PERSISTENT`・`TIME`/`FRAMEINDEX`/`PASSINDEX` の意味は naga の外(Motolii の `parse_isf_source` `isf/mod.rs:134-224`)。なお `PASSINDEX` は束縛だけ確保して GLSL 側の宣言が無い(`vism.rs:28`、`:163`、`:416-421` に対し `wrap_fragment_source` に宣言無し)— naga の話ではない。
- `#include` は pp-rs に無い(`pp.rs:569-592` の directive は error/line/define/version/extension/pragma、他は `UnknownDirective` `token.rs:93`)。
- 対応版は 440(部分)/450/460 のみ(`front/glsl/mod.rs:6-10`)。Motolii が `#version 450` を前置する(`isf/mod.rs:261`)ので版指定の無い community `.fs` は通るが、GLSL 1.x 流儀(`varying`・`texture2D`)の可否は今回未確認。
- 実行時にファイルを見張る機構(hot reload)は naga に無い — re_renderer の FileServer 側。GPU 実行時の非同期エラーも naga の範囲外。
- 弊社は文法・型・束縛属性の検証器であって、絵が正しいかは見ない。

見込み合計: naga 側の新規 0 行、Motolii ≒60 行で「実行時 dir から `.fs` を読み、失敗は行番号付き英語で窓へ、INPUTS と本文の齟齬は起動前に」が既に払った代金の中で出る。C1(bypass)/C2(番号)は EffectInstance の話で弊社の範囲外。

## ffmpeg / ffmpeg-sidecar(2.5.2)

前提(実測): `which ffmpeg ffprobe` = /opt/homebrew/bin、ffmpeg 7.1.1。encoders に `libx264` / `prores_ks`(yuv422p10le・yuv444p10le・yuva444p10le)/ `prores_videotoolbox`(profile 4444 / xq)/ `png`(rgba)/ `gif` / `apng` / `hap` / `ffv1` が在る。filters に `palettegen` / `paletteuse` / `split` / `unpremultiply`(`inplace`)/ `fps` / `tile` / `thumbnail` / `zscale` が在る。Motolii の ffmpeg 呼び出しは 3 本 — `crates/motolii-render/src/media/encode.rs:56-89`(rawvideo rgba を stdin から libx264 + aac)、`media/probe.rs:161-170`(`ffprobe -show_streams -show_format -print_format json`)、`src/ui/thumbnail.rs:64-69`(`FfmpegCommand` で `-vframes 1 … -f image2pipe -vcodec png`)。ffmpeg-sidecar は thumbnail.rs だけが使う(Cargo.toml:35、default feature = `download_ffmpeg`)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| FF1 | encode.rs:69-74 が `-c:v libx264` 固定、export_sheet.rs:133/146-147 が `.mp4` 固定(X1・GC7) | 同じ stdin rawvideo に出口の flag だけ差す。ProRes 4444: `-c:v prores_ks -profile:v 4444 -pix_fmt yuva444p10le x.mov`(α 付き、`alpha_bits`)。PNG 連番: `-f image2 -c:v png -pix_fmt rgba -start_number 0 x_%05d.png`。GIF: `-vf "split[a][b];[a]palettegen[p];[b][p]paletteuse" -loop 0 x.gif`(`-loop` は gif muxer の option)。1 入力 → 4 出力を 1 プロセスで出せることを scratchpad で実測(mp4 / mov prores 4444 / png×30 / gif) | Encoder に `enum Format` 1 つと flag 表 ≒25 行。sheet は拡張子の 1 行 |
| FF2 | encode.rs:76-85 の `scale=out_color_matrix=bt709:out_range=tv` に `in_range` が無い、`-color_trc bt709`(中身は sRGB)(X5) | `scale=in_range=pc:in_color_matrix=…:out_range=tv`(filter option 実在)、`-color_trc iec61966-2-1`(`-h full` に値 13 として在る)。どちらを tag するかは製品の裁定 — 再生側は大半が bt709 扱いなので、flag は在るが「正解」は弊社に無い | flag 2 語 |
| FF3 | export.rs:191-197 が `render_frame` の premultiplied 画素を `image::save_buffer` で PNG に(X5「PNG が premultiplied のまま」) | FF1 の PNG 出口に `-vf unpremultiply=inplace=1` を足す(実測 ok)。背景抜きは engine.rs:179 `render_frame_without_background` が既に在る | 1 語。export_still の image 依存が 1 本消える |
| FF4 | export_range_with_progress(export.rs:118-176)は 1 job = 1 Encoder。D8 バッチは render を job 数だけ回す形になる | ffmpeg は 1 入力に出力を並べられる(`-map 0:v … a.mov -map 0:v … b.mp4`)。同じ frame を 1 度描いて N 本に書く。GIF の palettegen は出力ごとの `-filter_complex` label 管理が要る(実測で無 label 出力が別出力へ流れ込んだ) | Encoder::open が `&[Output]` を取る形へ ≒20 行。「同一 comp の複数 format」までが弊社の範囲、複数 comp は Motolii のループ |
| FF5 | probe.rs:118-122 `FfprobeFormat` が duration と format_name だけ、M5 の「容量」が無い | 既に叩いている `-show_format` の json に `size` / `bit_rate` が在る(実測)。尺・fps・解像度は `ProbedVideoStream` に既在(probe.rs:29-39)、窓に出していないだけ | serde の field 2 つ。あとは browser.rs の札の文字 |
| FF6 | thumbnail.rs:66 が `-vframes 1`(= 先頭 1 コマ。黒フェードインの素材は黒札)。M7 hover scrub 無し | 1 プロセスで連番: `-vf fps=N/dur,scale=160:-2 -f image2pipe -c:v png -` → 1 本の stdout に PNG が N 枚(実測 5 枚)。sidecar の `.rawvideo()` + `iter().filter_frames()`(iter.rs:114、`OutputVideoFrame{frame_num,width,pix_fmt}`)なら PNG の切れ目を数えずに済む。1 枚の sprite なら `tile=Nx1`。代表コマは `thumbnail=10` filter | 既存 14 行の flag 差し替え。R6(re_video 化)と競合 — 札の絵の正本をどちらにするかは利用者裁定 |
| FF7 | encode.rs:118-128 は EPIPE の後にだけ stderr を読む(GAP-26 「continuous stderr drain」は decision-index:355 で未成立) | sidecar `FfmpegChild::iter()` は `spawn_stderr_thread`(iter.rs:345)で stderr を糸で吸い続け `Log(LogLevel::Error, …)` に分ける(log_parser.rs:101)。`quit()`(child.rs:83)は `q` を送る行儀の良い中断 | 但し decision-index:361 が M0-S2 で sidecar を**不採用**にした記録が在り、thumbnail.rs:64 は既にその裁定を破っている。Encoder へ広げるなら裁定を先に更新 |
| FF8 | tools_available(media.rs:98-106)は `Command::new("ffmpeg")` — PATH 直、thumbnail.rs は `FfmpegCommand::new()` = `paths::ffmpeg_path()`(exe 隣 → PATH)。探し方が 2 通り | `ffmpeg_path()` / `ffprobe_path()`(ffprobe.rs:14)を encode.rs:21 / probe.rs:161 にも渡す | 3 行。SE2 の前提 |
| FF9 | SE2「客に brew を言わせる」 | `download::auto_download_with_progress`(download.rs:116)が exe の隣へ落とす。**落とし穴 3 つ(正直に)**: (1) macOS の zip は ffmpeg 単体、`// no ffprobe on mac`(download.rs 移動部)→ tools_available の `&& ok("ffprobe")` が偽のまま。ffprobe 無しで行くなら probe.rs を `ffmpeg -i` の stderr 解析(sidecar `ParsedInputStream` / `ParsedDuration`、event.rs:10-14)へ書き直すが、`Stream` は format と index しか持たず nb_frames・SAR・rotation・color_range が無い → 弊社に無い。(2) `ffmpeg_manifest_url` は非 x86_64 で bail(download.rs:26-27)、arm64 の落とし先は osxexperts.net 固定(download.rs:53)。(3) 落とし先が `current_exe().parent()`(paths.rs:30-38)= 署名済み .app の Contents/MacOS → SE1 の署名を壊す。ライセンス: 配布元の build 構成(libx264 = `--enable-gpl`)次第で GPL。弊社は URL を渡すだけで法務は請け負えない | brew の文言は消せるが、SE2 の「法務判断」は残る。勧めるなら download 先 dir の変更 + ffprobe 別調達が要り、「1 語」ではない |

弊社に無い物(正直に): 音の decode は symphonia(audio/decode.rs:8-14、ordinal 0 固定 decode.rs:22)なので A19 は UI の選択口の問題で ffmpeg の出番無し(`-map 0:a:N -f f32le` で出せはするが経路が 2 本になる)。D3 relink は path の話で弊社無関係。Lottie(SE5)。色の「正解」(FF2)。ffprobe の JSON は sidecar に parser が無く(ffprobe.rs は version と path のみ)、Motolii の probe.rs:72-122 の方が上。re_video が既に H.264 を読む(engine/texture.rs:411-440)ので、書き出し以外で ffmpeg を増やす理由は薄い。

見込み: FF1〜FF3・FF5 は flag と field で ≒50 行、X1・X5・GC7・M5 が動く。FF4 は Encoder の口 1 つ。FF6〜FF9 は decision-index:361(sidecar 不採用)の再裁定と R6 との住み分けが先。

## cpal / symphonia(cpal 0.18.2、symphonia 0.6.1)

`$C` = cpal-0.18.2、`$S` = symphonia-0.6.1、`$SC` = symphonia-core-0.6.1(registry src 配下)。Motolii 側は `crates/motolii-render/src/audio/` と `src/ui/playback.rs`。

### 今日、feature flag / 既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| AU1 | seek も一時停止も session を捨てて device を開き直す(playback.rs:272, 294 `state.session = None`、理由は「ring に残った古い sample を消せない」playback.rs:292-293)。`PlaybackSession::seek`(session.rs:92-97)→ `MixProducer::seek`(producer.rs:83-86、loop 側 171-182 で pending を捨てる)は既に stream を保ったまま動く。残るのは rtrb の ring 4096 frame(session.rs:14、48k で ≒85 ms)と device 側 buffer | stream は `StreamTrait::play/pause`($C/src/traits.rs:451, 469)で保つ。ring の残りは callback 側(ring.rs:4-29、Motolii の 25 行)に「世代」atomic を 1 本足し、seek 世代が変わっていたら pop し切ってから詰める。device 側は `StreamConfig.buffer_size = BufferSize::Fixed(n)`($C/src/lib.rs:388-392、`SupportedStreamConfig::buffer_size()` の Range 内、lib.rs:460)— CoreAudio は device.rs:1009 で honour。今は `negotiated.config.config()`(device.rs:85)で Default のまま。弊社の例は ring を `latency*2` で切る($C/examples/feedback.rs:212-229) | 開き直し 0 回、古い音 ≤ 1 block。A1 の「音付きスクラブ」の土台(スクラブ中に短い区間を流す判断自体は producer 側、弊社に無い) |
| AU2 | device の error callback は `eprintln` だけ(device.rs:92-94)。AirPods 等で既定出力が変わった時の扱いが無い | CoreAudio の `DefaultOutputMonitor`($C/src/host/coreaudio/macos/mod.rs:255-259, 342-348)が既定 device の切替を追い、stream は生きたまま `ErrorKind::DeviceChanged` を返す($C/src/error.rs:15-23)。無くなった時は `DeviceNotAvailable`。後者だけ `PlaybackHealth::VisualOnly(Device)`(playback.rs:95)へ流す channel を 1 本 | 0 行で切替追従、+10 行で「Visual playback」表示が正しくなる |
| AU3 | A7 メーター: `AudioMeter`(meter.rs:53-96、peak L/R + clip latch)と `mix_audio(…, meter)` の口(mix.rs:138-140)は在るが、producer は `None` を渡す(producer.rs:247)。UI に読む側が無い(`src/ui` に meter 参照 0 件) | 弊社の物ではない(symphonia の dsp は fft/mdct のみ、`$SC/src/dsp/`。peak/RMS は無い)。ただし配線は `Some(&meter)` と `Arc` の共有で済む。表示は mix 時刻で出るので DAC より ring+device 分先行する — 補正は cpal の `OutputStreamTimestamp.playback`($C/src/timestamp.rs:56-64)を既に `DeviceWaitLatency::update_from_output_callback`(clock.rs:66)で取っている値をそのまま使う。RMS は meter.rs に無い(peak のみ、+8 行) | ≒15 行、A7 |
| AU4 | 素材を丸ごと decode して f32 に持つ(decode.rs:75-104、天井 `MAX_SAMPLES` = 4 時間 ×48k、decode.rs:19)、その後 rubato で丸ごと再標本化(convert.rs:6-11)。どちらも `AudioProgram::from_view` の中で同期(playback.rs:129, 200)— 生成中は無表示(A17)、長尺は UI が止まる | `FormatReader::seek(SeekMode::Accurate, SeekTo::Time)`($SC/src/formats/mod.rs:110-118, 579-591)+ seek 後の `AudioDecoder::reset`($SC/src/codecs/audio.rs:261)で任意時刻から packet 単位に decode できる。isomp4 は demuxer.rs:671、mp3 は Coarse/Accurate 両方(bundle-mp3 demuxer.rs:232-294)と Xing/VBRI の尺(431-456)。`FormatOptions.seek_index_fill_period_ms` は既定 1000 ms、「対話的用途は下げよ」と明記($SC/src/formats/mod.rs:130-140)。今の decode loop(decode.rs:75-104)は packet 位置から進捗が出せるので A17 の「生成中」表示は同じ loop に 1 行 | 丸ごと decode を残すなら A17 だけ(+5 行)。長尺を stream にするなら mixer の乱択読み(mix.rs:289 `lerp_stereo`)を窓化する必要があり、それは Motolii 側 |
| AU5 | Cargo.toml:103 の feature は `aac aiff alac flac isomp4 mp1 mp2 mp3 ogg pcm vorbis wav`(default-features = false)。素材棚の受付は media.rs:26-28 | 未使用で在る物: `adpcm`(IMA/MS ADPCM の WAV — 古い SE/音声 WAV は今 unsupported codec)、`caf`、`mkv`(mkv/webm の音、vorbis は既に有効)、`id3v1 / id3v2 / ape`($S/Cargo.toml:79-83)。symphonia-metadata は `default-features = false` で引かれ($S/Cargo.toml:205-207)、`Id3v2Reader` は feature 下でしか probe に登録されない($S/src/lib.rs:296-300)。無いと probe は ID3v2 tag の中身を MPEG sync(`0xFF 0xFE…`、bundle-mp3 demuxer.rs:98-103)で走査し、既定の探索天井 1 MB($SC/src/formats/probe.rs:290, 304, 593-594)を超える cover art 付き MP3 は「no suitable format reader found」で落ちる。`opt-simd` も切れている($S/Cargo.toml:86-87, 121-126、core の fft/mdct 向け、効果は未計測) | 4 語(`adpcm caf mkv all-meta`)。id3v2 は MP3 の取り込み失敗 1 種を消す |
| AU6 | S13 BPM/拍 | 検出は弊社に無い。ただし tag に書かれた BPM は読める: `StandardTag::Bpm(u64)`($SC/src/meta.rs:204、ID3v2 の TBPM)— AU5 の `id3v2` が前提 | 素材の名札に BPM を出す分だけ |
| AU7 | A19 動画の音は 1 本目だけ | `decode_file_audio_ordinal`(decode.rs:25, 54-62)が `format.tracks()` の `CodecParameters::Audio` を序数で選び、cache も `(path, ordinal)` で引く(program.rs:257-271)。symphonia 側は揃っている — 層がどの序数を持つかは Motolii の meta | 弊社側 0 行 |
| AU8 | device 交渉を自前の優先表で行う(device.rs:109-195、PREFERRED 表 149-151) | `DeviceTrait::default_output_config()`($C/src/traits.rs:231)が device の現在の rate/format を 1 回で返す。f32/2ch ならそれで済み、外れた時だけ今の探索へ | ≒40 行減(条件付き) |

### 弊社に無い物(正直に)

- **A3 ループ区間**: cpal・symphonia とも無関係。producer の終端判定(producer.rs:230-237)を `loop_end → loop_start` で巻き、`PlaybackClock::position`(clock.rs:180)を巻き戻し毎に rebase する Motolii 側の仕事。AU1 で stream を保てば区間の跨ぎで device は開き直さない。
- **BPM / 拍の検出**(S13): 無い。tag 読みだけ(AU6)。
- **A8 ステレオ波形・A10 対数振幅**: waveform.rs:254-267 が mono に畳み `split_channels: false`、normalize_peak(waveform.rs:320)が線形 — これは waveform-data 0.1.1 の口で、弊社外。
- **codec で読めない物**: Opus(symphonia 0.6.1 に codec 無し — `$S/Cargo.toml:149-207` の依存に不在)、WavPack、AC-3/E-AC-3/DTS(mp4/mov の映画音声)、Opus 入り WebM。これらは ffmpeg 経由か別 crate。
- **スクラブ音そのもの**(A1 の「掴んで動かすと鳴る」): 音を出す口は AU1 で残るが、どの区間を何 ms 鳴らすかの判断・可変速再生は cpal に無い。rubato も固定比(resample.rs:9-18 `FftFixedIn`)。
- **cpal の `pause()` は backend 次第で `UnsupportedOperation`**($C/src/traits.rs:460-469 の注記)。CoreAudio で通るかは窓で確認が要る。通らなければ ring を無音で埋めて stream は回し続ける形。

見込み: feature 4 語 + AU1〜AU3 で ≒60 行(うち device 開き直し経路 playback.rs:361-376 の呼び出しが seek/pause から消える)。A1・A2・A7・A17・A19 に手が届き、A3・A8・A10・S13 は弊社の出番が無い。

## accesskit / kittest(accesskit 0.24.1)

前提(cargo tree で確認): Motolii の graph に居る弊社物は `accesskit 0.24.1` → `accesskit_consumer 0.38.0` → `accesskit_macos 0.26.3` → `accesskit_xplat 0.1.1`(blitz-shell の `accessibility` feature、dioxus-native Cargo.toml:30 経由)。**kittest 0.4.0 は registry に在るが Motolii の graph には無い**(accesskit 0.24 + consumer 0.35 依存 — consumer が 0.35/0.38 に割れる)。`egui_kittest 0.35` は crates/motolii-ui/Cargo.toml:99 に dev-dep 宣言だけで src に使用箇所無し。NodeBuilder は 0.24 に無い(`Node::new(role)` lib.rs:1737 に統合、blitz も accessibility.rs:7,42 でそれを使う)。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| AK1 | harness の合成 click は座標の PointerDown/Up 2 発(keys.rs:199-224 `activate_focused_control`、gui.rs:349-357 `click_super`)。K19「1・2 が構造上捕まらない」、Q15 | 木の node id は blitz の NodeId そのもの(accessibility.rs:40 `NodeId(node.id.as_u64())`)。harness に `act(node, Action)` を 1 本置き、`Action::Click`(lib.rs:291)→ 中心座標の pointer 対、`Focus`/`Blur`(:293-294)→ `set_focus_to`/`clear_focus`(document.rs:1633)、`Increment`/`Decrement`(:303-305)→ spinbutton の鍵。`ActionRequest { target_node, data }`(lib.rs:2949、data は :2926)を窓と試験の共通の入口にする | ≒40 行。keys.rs の合成 click がここへ畳まれる |
| AK2 | 窓側: VoiceOver の押下(AppKit `accessibilityPerformPress`)は accesskit_macos node.rs:668-675 で `Action::Click` の ActionRequest になり、xplat lib.rs:171 → blitz-shell accessibility.rs:23-28 → **application.rs:87 `ActionRequested(_req) => // TODO` で捨てられる** | host.rs:712 は `BlitzShellEvent` を丸ごと `inner.handle_blitz_shell_event` へ流す。その前に `Accessibility { data }` を 1 腕 match し、AK1 の `act` へ渡す。**試験と読み上げが同じ関数を通る** | ≒15 行(上流 PR 無しで可)。VO の押下が生きる |
| AK3 | 試験は CSS selector と文字で node を探す(gui.rs:71-160)。名前は `.a11y` span(VO1) | blitz-test-harness は blitz-dom を `accessibility` 付きで建てる(Cargo.toml:15)ので `doc.build_accessibility_tree()` は今の gui.rs から呼べる。`accesskit_consumer::Tree::new(update, true)`(tree.rs:598)→ `node_by_id`(:498)/`focus`(:544)/`role`(node.rs:388)/`supports_action`(:670)/`bounding_box`(:315)/`live`(:889)。kittest は filter.rs:89 `label` / :119 `role` / :125 `value` の query と state.rs:36 だけで、click / 打鍵の API は持たない(駆動は harness 側 = AK1) | accesskit_consumer を dev-dep 1 行(版は lock 済み)。kittest は consumer 0.35 を引くので入れない方が軽い |
| AK4 | VO2「node に矩形が無い」。macOS は `bounding_box` が無いと root 以外 `NSRect::ZERO`(node.rs:461-467) | `Node::set_bounds(Rect)`(lib.rs:2177)。blitz の木は role と html_tag しか置かない(accessibility.rs:47-57)。keys.rs:199-200 と同じ `absolute_position` + `final_layout().size` を上流 accessibility.rs:56 の隣に 3 行 | 上流 B13 の中身は 3 行 |
| AK5 | VO7 live が届かない。output.rs:315 / app.rs:1639 に `role:"status" aria_live:"polite"`、settings.rs:33,42 に `role:"status"` | 通知の条件は accesskit_macos event.rs:236-238 / :299-305「`value` が在り `live != Off`」。blitz は `set_live`(lib.rs:2139、`Live` は :584)を一度も置かず、文字は子 TextRun の value(accessibility.rs:58-61)なので status 自身に value が無い。上流で `aria-live` → `set_live` + `text_content` → `set_value` | 上流 B14 の中身は 4 行 |
| AK6 | 名前。VO1 は `.a11y` span を可視文字の代わりに置いた | blitz の木は TextRun を **直親**の `labelled_by` にしか繋がず(accessibility.rs:61)、consumer の `label()` は labelled_by 先の `label` か `Role::Label` の value しか読まない(node.rs:731-780)。よって button の `label()` は None、kittest `by().label("Play")` は今の木では当たらない。`aria-label` → `set_label`(lib.rs:1916)の上流 B12 が先 | B12 は 2 行、それで AK3 の query が名前で引ける |
| AK7 | BU5・VO13: Stage / Timeline / Ease が木に出ない。custom_widget.rs:142-143 `// fn accessibility_tree` がコメントアウト。widget は Pointer / Wheel しか受けない(stage_widget.rs:911,945,1083,1151、timeline_widget.rs:969-1201、ease_widget.rs:164-206) | 木を組むのは blitz-dom(accessibility.rs:5)で Motolii から node を足す口は無い → 上流 PR: `Widget::accessibility_nodes(&self) -> Vec<(NodeId, Node)>` を :15 の visit に足す。node は `Node::new(Role::ListBoxOption / Slider)` + `set_bounds` + `set_numeric_value`(:1977)+ `add_action`(:1760)、id は `NodeId(u64)` なので層・キーごとに刻める。押下・Increment は AK2 の道で widget の `handle_event` へ | 上流 ≒30 行 + Motolii 側 widget ごと 20〜40 行。Ease の代替路は `Increment/Decrement` で初めて成立 |
| AK8 | KB8 Tab 順(Position まで 16、層 16 枚で 80) | Tab の停止は DOM の `is_focussable`(node.rs:492、document.rs:1617-1629)で決まり、弊社は変えられない。弊社で出来るのは木の側: 層一覧を 1 停止にして `set_active_descendant`(lib.rs:1902)で今の行を告げる(roving の a11y 側)。`Action::SetSequentialFocusNavigationStartingPoint`(:342)は定義だけで誰も処理しない | Tab 圧縮そのものは tabindex の仕事(KB8 の本体)、弊社は読み上げの整合だけ |

**弊社に無い物(正直に)**: 合成 pointer / 打鍵 / IME の生成(blitz-test-harness input.rs:30-228 の領分、B7)。文字欄への `SetValue`(lib.rs:347)は blitz 側に受け口が無い(BU3、TextInputData の公開 API 無し)ので鍵の列でしか書けない。Tab 順の変更、drag(`Action` に drag は無い)、menu の roving(BU6)。kittest は「探す」だけで「押す」は持たない — 押す物は上流の harness か AK1 の自前 40 行。DOM 差分の TreeUpdate は blitz が毎回全木を組み直す(accessibility.rs:5-33、window.rs:376-381)ので弊社の増分更新は今の配線では効かない。

見込み合計: Motolii 側 ≒55 行(AK1・AK2)+ dev-dep 1 行で「試験も読み上げも同じ `ActionRequest`」。VO2・VO7・名前(AK4〜AK6)は上流 3 件で計 ≒10 行、custom widget(AK7)は上流 PR 待ち。

## dioxus-dnd / dioxus-workbench(2 度目、dnd 3.1.0、workbench 0.1.0)

前回 B8 で「5 型しか使っていない」と言った営業の宿題。今日はカタログを全部並べ、**blitz(dioxus-native)で動く物と動かない物を先に分けます**。前回の B8 は一部が誇張でした — 先に訂正します。

ソース: `$D` = `/Users/member_ottoto/.asdf/installs/rust/stable/registry/src/index.crates.io-1949cf8c6b5b557f/dioxus-dnd-3.1.0/src`、`$W` = 同 `dioxus-workbench-0.1.0/src`、`$B` = `~/.asdf/installs/rust/stable/git/checkouts/blitz-66635cc3152d32bd/64eb278/packages`、Motolii は `motolii/src/ui/`。

### 今 Motolii が使っている物

- dnd: `transition / GestureEffect / GestureEvent / GesturePhase / Point` の 5 型だけ(dock.rs:7、app.rs:2)。`Cargo.toml:62` は `default-features = false` — `serde`・`desktop`・`web` 全部 off。
- workbench: model だけ(`DockZone LayoutNode PanelId PanelLayout PanelPlacement SplitAxis SplitId TileId Tile`、dock.rs:4-6、dock_hit.rs:2)。component(`PanelWorkspace` 等)は未使用 — **これは正しい選択**(後述 DD9)。

### 前回の訂正(誇張だった物)

- **`FileDropZone` / `ExternalDropZone` / `TypedDropZone` は blitz で死んでいます。** 3 つとも HTML の `ondragenter/ondragover/ondrop` + `DataTransfer` で動く(files.rs:373-400、external.rs:66-130)。blitz-shell は winit の `DragEntered/DragDropped` を no-op にしていて(`$B/blitz-shell/src/window.rs:852-855`)、dioxus-native-dom の event 名に `drop`/`drag*` は無い(pointer/mouse/wheel/scroll/click のみ)。おまけに picker は `document::eval`(files.rs:335)で、dioxus-native の `eval` は `NoOpDocument`(`$B/dioxus-native/src/contexts.rs:19-21`)。→ M8/K9 の「FileDrop で新規 0」は取り消し。Motolii の `FileDropSurface`(session.rs:15-34)・`drop_role_at`(keys.rs:81-93)・host.rs:618-649・`file_drop_overlay`(app.rs:211-244)≒ 70 行は**残ります**。`FileFilter`(files.rs:48-145)も引数が `FileData` で `PathBuf` は通らない。
- **`MultiWindowProvider` は dioxus-desktop 専用**(bridge.rs:6-8 `dioxus_desktop::tao::…` / `use_wry_event_handler`、docs/api/multi-window.md:11-14)。winit の Motolii では link できない。

### blitz で動く根拠(pointer 系)

core と sortable/canvas/tree/a11y/autoscroll は `onpointerdown/move/up/cancel` + `onkeydown` だけ(draggable.rs:562-835、handle.rs:44-53)。blitz はこれを配る(上の event 名一覧)。測定は `MountedData::get_client_rect / get_scroll_offset / scroll`(dnd 側 registry.rs:951、drop_zone.rs:271、autoscroll.rs:303-379)で、dioxus-native-dom は `RenderedElementBacking for NodeHandle` に全部実装済み(`$B/dioxus-native-dom/src/events.rs:194-262`)。pointer capture は無いが、capture できない時は全画面の代替層を出す設計(platform.rs:5-7、draggable.rs:1029)— Motolii の `.dock-capture`(app.rs:1273-1275)と同じ手。**窓で実走はしていない**(DragSim で VirtualDom 検証まで)。

### 賄える物

| # | Motolii の今 | 弊社の形 | 見込み |
|---|---|---|---|
| DD1 | Browser の札は click で即 `spawn_layer`、Alt+click で差し替え(browser.rs:639-660)— M8 引けない | `Draggable<Payload>`(draggable.rs:94-135、`on_drag_end(bool)`)を `.tcard` に、Timeline の `lrow`(timeline_shell.rs:148)に `DropZone { edge: EdgeSet::Vertical }`(drop_zone.rs:110-116、`DropOutcome::edge` で行間)、Stage に `CanvasDropZone`(canvas.rs:155-175、`CanvasDrop { position, pointer }` + `SnapGrid` canvas.rs:29)。`DndProvider` は app 直下 1 枚 | 新規 ≒40 行、消える 0。M8 |
| DD2 | K9/M15 素材を鍵で置けない | Draggable の鍵 drag(Space で持つ・矢印で zone を歩く・Esc、draggable.rs:835)+ `CanvasKeyboardPlacement`(canvas.rs:48、canvas_keyboard_pointer:124)+ `LiveRegion`(a11y.rs:9-31) | DD1 に乗れば新規 ≒5 行 |
| DD3 | C1 効果の並べ替え無し(inspector.rs:1272-1289 は × だけ) | `SortableList { len, render, on_sort }`(sortable.rs:388-433、live_preview)+ `apply_sort`(367)+ `ReorderButtons`(a11y.rs:53)で鍵の代替も同時。書き戻しは既存 `Intent::SetEffects`(browser.rs:438) | 新規 ≒25 行、消える 0 |
| DD4 | 層の並べ替えは Intent::Reorder(±1)の鍵だけ(app.rs:1117-1136、keymap.rs:197-204) | 同じ SortableList を `lrow` に(`item_key` 必須、属性行 timeline_shell.rs:136 は混ぜない)→ `SetOrder` | 新規 ≒20 行 |
| DD5 | dock の tab drag: `TabDrag`(dock.rs:307-378)、当たり判定 `dock_target_at`/`dock_side_at`(dock_hit.rs:9-52)、app.rs:653-691 の配線 | `Draggable<Panel>` + tile ごとの `DropZone { edge: EdgeSet::All }` → `edge` を `Side` に写す(dock.rs:108-117 は残る) | dock.rs ≒70 行 + dock_hit.rs ≒40 行 → 新規 ≒30 行。**窓の外に落として別窓**(app.rs:66-72、host.rs:131-139 `outside`)は残す |
| DD6 | Q16 drag の後始末は `GestureSurface`(session.rs:9-13,36-70)と focus_lost の手畳み(app.rs:520-538) | `on_drag_end(false)` / `CancelReason` / `DragOverlay`+`SettleSlot`(overlay.rs:78,565)。ただし custom widget 内の drag(timeline_widget.rs:69-79 `DragState`)は DOM の外で対象外 | DOM 側だけ ≒15 行減 |
| DD7 | Q15 harness は合成 event(gui.rs:335) | `DragSim`(test.rs:63-411: place / pick_up / move_to / release / cancel / announcement)は VirtualDom だけで delivery 本経路を通す | DD1-5 の試験が blitz 無しで書ける |
| DD8 | 引きながら格子・行を送れない | `AutoScroll { threshold, axis, active, drag_pointer, on_scroll }`(autoscroll.rs:224-260)— `get_scroll_offset`/`scroll` は blitz 実装あり | 新規 5 行 |
| DD9 | 別窓へ drag(H14 の続き) | core の world は platform 非依存(`use_dnd_world` world/mod.rs:100、`DndWorld::vdom`、`track_global`/`drop_at_global` world/host.rs:66,129)。**feed は Motolii が winit から書く**(手本は desktop/feed.rs 95 行) | 新規 ≒100 行、消える 0 |
| DD10 | F20 ピックウィップ | 親付けは `Draggable<LayerId>` + 行ごとの `DropZone { accepts }`(循環は `tree::would_create_cycle` tree.rs:76)。**線は描けない** | 新規 ≒30 行 |

その他在庫: `Selection`(multiselect.rs:12-190: click/shift 範囲/keyboard_range)、`SortableGrid`(grid.rs:59)、`TreeNodeTarget`(tree.rs:112、Before/Into/After)、`FlipItem`(animate.rs:42、blitz では render 経由の fallback)、`DndDebugOverlay`(debug.rs:18)、`DropEffects` の交渉(effects.rs:9-35)。

### workbench

- `PanelLayout` に `encode/decode/validate/reconcile`(model.rs:337-394)、`dock_panel/split_tile/split_active/remove_empty_tile/move_panel_by_tile`(480-564)。Motolii の `Dock` は `Serialize` 派生で `layout.json` を自分で書く(dock.rs:121-165)— **弊社の encode を使っても消えるのは 2 行**。
- **`PanelWorkspace` / `Workbench` / `StatusBar` は blitz で動きません**: pointer capture・flex-basis・scroll・focus が全部 `document::eval`(dom.rs:27-105)、tab の drag が HTML5 `ondragstart/ondragenter`(workspace.rs:807,922)。model だけ使う今の形が正解。

### 弊社に無い物(正直に)

- OS のファイル drop(blitz が DOM drop event を出さない — BU2 の領分)。Motolii の 70 行は残る。
- winit 用の multi-window bridge / geometry feed(`desktop` は tao/wry)。
- workbench に `hidden`・`detached`・別窓・机の割合の記憶(dock.rs:123-128)・eval 無しの splitter(app.rs:385-418, 662-670 は残る)。
- custom widget 内の drag(Timeline の帯・Stage の取っ手)、ピックウィップの線、属性行を含む木の live-preview 並べ替え。

見込み合計: 消える ≒125 行(DD5・DD6)、新規 ≒270 行で M8・K9/M15・C1・Q15 の DOM 側・F20 の親付け本体が乗る。多くは**新規で、置換ではない**。B8 の「ほぼ新規 0」は撤回します。

### 問い返し: Stage は re_renderer の世界に入るか(dioxus-dnd)

結論から: **入りません。dnd の canvas 系は 2D DOM の道具で、Stage には「落ちた」という事実と窓の座標を渡すまでが弊社の仕事**です。3D への写像は Motolii の Stage が既に持っている `Fit` の責任で、dnd はそれを知りません。

根拠。`CanvasDropZone` が渡す `CanvasDrop { position, pointer }` は DOM の矩形基準の 2D 点で、`canvas_position = pointer − grab → snap → clamp`(canvas.rs:107-122)、`client_to_canvas` は `client − rect.origin`(canvas.rs:96-98)。奥行き・視点・作品枠の概念はどこにも無い。一方 Stage 側の pointer の受け口は custom widget の `UiEvent::PointerDown`(stage_widget.rs:952-953)で、そこで `self.fit.to_comp(p.element.x, p.element.y)`(stage_widget.rs:92-99 — `image_from_world` の逆写像)、奥行きの面は `Fit::at_z`(stage_widget.rs:63-71、`camera_screen_from_world_at_z`)で解く。つまり「窓の点 → 作品枠の平面のどの点か」は Stage が既に持つ写像で、drop でも同じ関数に通すだけです。

| 提案 | 振り分け | 理由 |
|---|---|---|
| DD1 Stage の `CanvasDropZone` + `SnapGrid` | **(c) Stage では使わない** → 素の `DropZone` で (a) | `CanvasDrop.position` は DOM 2D の左上座標。`grab` の補正も snap も「窓の矩形」基準で、視点を回した Stage では意味を持たない。素の `DropZone`(drop_zone.rs:90-120)が返す `DropOutcome.element`(要素基準の pointer)を `Fit::to_comp` / `at_z(depth)` に通し、既存の `center_intents`(browser.rs:148-150)の代わりに置く。当たり判定の主体は R2 の PickingLayerProcessor でも今の箱でも構わない — dnd は関与しない |
| `SnapGrid`(canvas.rs:29-35、`round(p / step) * step`) | **使わない** | 窓の px 格子で、作品座標の格子ではない。格子を持つなら R5 の WorldGrid(作品座標・描画)に寄せ、吸い付きは `to_comp` の後で Motolii が丸める(timeline_widget の `SNAP_PX` と同じ流儀)。**二重になります** — SnapGrid は捨ててください |
| DD2 `CanvasKeyboardPlacement`(canvas.rs:48-56、Center/Origin/Fixed(2D)) | **(b) 2D DOM に残す — ただし値だけ** | 鍵で落とした時に「要素の中心」を pointer とみなす方針は妥当(canvas_keyboard_pointer:124-131)。ただしそれも 2D の element 点なので、上と同じく `to_comp` へ。Fixed は不要 |
| DD2 鍵 drag 本体(Space / 矢印 / Esc、draggable.rs:835)+ `LiveRegion` | **(b) 2D DOM で正当** | 鍵の drag は zone を歩く操作で、Stage は zone 1 つ。3D に入る必要が無い |
| DD10 ピックウィップの Stage 側 | **(c)** | 親付けは Timeline の行(DOM)で完結。Stage 上の層へ whip するなら当たりは Stage の hit(R2)で、dnd の DropZone は矩形しか知らない |
| `Bounds::clamp`(canvas.rs:64-93) | **使わない** | 窓の矩形で clamp すると作品枠の外(視点を寄せた時)を誤って切る |

つまり Stage に掛かる dnd の役目は **「drop が Stage 要素に落ちた」+ 要素基準の 2D 点を 1 回渡す**だけ。奥行き(どの z の面に置くか)は dnd に語彙が無いので、Motolii の規則(選択層の depth `stage_widget.rs:468` か 0)で決める。前の表の DD1 の「CanvasDropZone + SnapGrid を Stage に」は取り下げ、`DropZone` + `Fit::to_comp` に置き換えます。

## Apple(AppKit / Core Text / AVFoundation / VideoToolbox、objc2 経由)

前提(裏を取った所): Motolii は既に objc2-app-kit 0.3.2 を直接持つ(motolii/Cargo.toml:42-43、tokens.rs:116 で `NSWorkspace::accessibilityDisplayShouldReduceMotion` のみ使用)。窓は blitz-shell `create_default_event_loop`(blitz lib.rs:55-66)→ winit-appkit 0.31.0-beta.2 が `NSApplication::sharedApplication` を握る(event_loop.rs:189)。winit は **app delegate を登録しないと保証**しており、自前の `NSApplicationDelegate` を `define_class!` で載せる例が lib.rs:8-40 に在る。これが弊社を「使い切る」入口。

### 今日、objc2 の既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| AP1 | NSMainMenu は自前の DOM menubar(semantic_menu.rs)、Open Recent も recents.json(project.rs:166-181)。ただし winit が既定で App menu(Hide ⌘H / Hide Others / Show All / Quit ⌘Q → `terminate:`)と Services を入れている(winit-appkit menu.rs:38-72, 85-86、event_loop.rs:164 `default_menu: true`)。delegate が無いので ⌘Q は `terminate:` で**未保存確認を素通り**(app.rs:1201 Intent::Quit の Save 3 択に届かない) | `NSApplication::setMainMenu`(NSApplication.rs:709)/ `setWindowsMenu`(:998)/ `setHelpMenu`(:722)/ `setServicesMenu`(:1517)、`NSMenu::addItemWithTitle_action_keyEquivalent`(NSMenu.rs:229)、`NSMenuItem::setKeyEquivalentModifierMask`(NSMenuItem.rs:213)、`setTarget`(:358, unsafe)、`separatorItem`(:67)。feature は `NSMenu`/`NSMenuItem`(objc2-app-kit Cargo.toml:864, 875)。winit の app_state.rs:130 の注釈どおり NewEvents 前後で上書き可。Window menu は `setWindowsMenu` で AppKit が Minimize / Zoom / 窓一覧を自動で埋める | H1・H2(Edit の標準 selector `cut:` `copy:` `paste:` `selectAll:` と Emoji `orderFrontCharacterPalette:` は responder chain へ送るだけ)・H3・H4・H14(main menu は app に 1 本なので別窓に自動で付く) |
| AP2 | ⌘Q・⌘W・Dock からの終了、Finder のダブルクリック | 自前 delegate に `applicationShouldTerminate`(NSApplication.rs:1118、Cancel / Later を返して app.rs:1201 の 3 択へ)、`applicationShouldTerminateAfterLastWindowClosed`(:1199、host.rs:683 の close→exit を止める)、`application_openURLs`(:1130)、`applicationDockMenu`(:1215、Open Recent を Dock に) | H12・SE 教育向けの「開き方」 |
| AP3 | 最近の作品を recents.json に自前保存、5 件を DOM menu に(app.rs:1342-1343) | `NSDocumentController::sharedDocumentController().noteNewRecentDocumentURL`(NSDocumentController.rs:316)/ `recentDocumentURLs`(:320)/ `clearRecentDocuments`(:307)/ `maximumRecentDocumentCount`(:300、OS 設定に従う)。NSDocument の subclass は不要 | ≒20 行減、H8 が OS の Open Recent と Dock に揃う |
| AP4 | 書体は path 直書き(browser.rs:295・fixture.rs:412 に `/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc`)、`shape_text` が呼ぶたび `fontdb::Database::new` + `load_font_file(path)`(text.rs:83-89)、無ければ Err で黙る、weight の口が無い(S6・LD4・LD7) | 列挙: `CTFontManagerCopyAvailableFontFamilyNames`(CTFontManager.rs:32)、`CTFontCollection::from_available_fonts`(CTFontCollection.rs:124)→ `matching_font_descriptors_for_family`(:437)、descriptor の `attribute(kCTFontURLAttribute)`(CTFontDescriptor.rs:903, :54)で **file path を今の fontdb にそのまま渡せる**。weight は `kCTFontWeightTrait`(CTFontTraits.rs:30)か `NSFontManager::availableMembersOfFontFamily`(NSFontManager.rs:208、weight 整数と traits 付き)。fallback: `CTFont::for_string_with_language`(CTFont.rs:791、= CTFontCreateForString)と `default_cascade_list_for_languages`(:935)で「この run にこの字が無い→OS の cascade」。**既に tree に在る**: parley→fontique 0.11.1 が objc2-core-text を link し同じ手順を書いている(fontique Cargo.toml:130、backend/coretext.rs:94, 107) | FontRef を「PostScript 名 + weight」に変え、path は起動時に Core Text で解く。再配布問題はヒラギノを .rrd に**入れない**(名前だけ持ち、無い環境は cascade)で消える。fontique の列挙を借りれば新規 ≒40 行 |
| AP5 | 音は cpal 0.18.2(既に coreaudio-rs + objc2-audio-toolbox を link: cpal Cargo.toml:314, 328) | 音 file の decode: `ExtAudioFileOpenURL`/`ExtAudioFileRead`(objc2-audio-toolbox ExtendedAudioFile.rs:111, 250)— AAC/ALAC/mp3/動画の音声 track を ffmpeg 無しで PCM に。スクラブ: `AVAudioPlayerNode::scheduleBuffer_atTime_options_completionHandler`(objc2-avf-audio AVAudioPlayerNode.rs:288)+ `AVAudioUnitVarispeed::setRate`(AVAudioUnitVarispeed.rs:66)で seek 時の短い片を再生 | A19(2 本目以降の音)・X9(mix の支度)。A1 の「seek が device を捨てる」は producer.rs:83 側の設計で、弊社の問題ではない |
| AP6 | dark 固定(H18)、accent 無視(H19)、Reduce Motion は起動時 1 回(H20) | H18: winit `Window::set_theme` が `setAppearance` を呼ぶ(winit-appkit window_delegate.rs:1814-1815)、app 全体は `NSApplication::setAppearance`(NSApplication.rs:872)、判定は `effectiveAppearance`(:877)+ `bestMatchFromAppearancesWithNames`(NSAppearance.rs:91)。変化は winit が effectiveAppearance を観測して ThemeChanged に写す(window_delegate.rs:486-488、B10 と同じ)。H19: `NSColor::controlAccentColor`(NSColor.rs:632) | H18 は AppKit 側 0 行(CSS の @media のみ、B10)、H19 は起動時 1 読み+ token 1 本 |

### crate は在るが本機で未検証の物(正直に)

| # | 内容 | 状態 |
|---|---|---|
| AP7 | SE2 / X1 / M5: 書き出しは ffmpeg に rawvideo rgba を pipe → libx264(encode.rs:57-73、qp0 は yuv444p)、probe は ffprobe 子プロセス(probe.rs:161)。AVAssetWriter(H.264 / HEVC / ProRes 422・4444 α付き、音の mux 込み)と AVAsset の duration・naturalSize・nominalFrameRate、AVAssetReader で ProRes/HEVC decode | objc2-av-foundation / objc2-video-toolbox / objc2-core-media は crates.io に 0.3.2 が在る(fetch で確認)が**本機の registry に無く file:line で裏が取れていない**。手元で確認できたのは objc2-core-video の `CVPixelBufferCreateWithBytes`(CVPixelBuffer.rs:647)まで(rgba の frame を CVPixelBuffer に包む口)。VideoToolbox の H.264 に qp0 の lossless は無い — 「劣化なし」は ProRes 4444 が弊社の答え。ffmpeg 同梱の法務を消せるのは書き出しの H.264/HEVC/ProRes と mp4/mov の probe まで。Lottie・PNG 連番は元々 ffmpeg 不要 |
| AP8 | H1 を muda で | muda 0.19.3 は objc2-app-kit ^0.3 依存(index cache)で 0.3.2 と衝突しない。本機に source が無く API は未検証。AP1 の直叩きなら menu は 4 本で ≒80 行、muda は event channel が付く代わりに依存 1 本 |

### 弊社に無い物・Rust から届かない物(正直に)

- **二重 link**: objc2-app-kit 0.2.2 が accesskit_macos 0.26.3(Cargo.toml:57-58 `"0.2.0"`)← accesskit_xplat(blitz)経由で残り、0.3.2 は arboard・rfd・webbrowser・winit-appkit・motolii(cargo tree)。objc2 本体も 0.5.2 / 0.6.4 の二重。Motolii 側では消せない — accesskit_macos の bump(registry の最新は 0.26.3)か blitz の accesskit_xplat 待ち。
- **NSDocument の自動保存・版(H10 の本体)**: `autosavesInPlace`(NSDocument.rs:475)/ `browseDocumentVersions`(:486)は NSDocument subclass + NSWindowController が窓を持つ前提。窓は winit が作るので**載らない**。persist.rs:101 `auto_save` を Motolii が自分で呼ぶ(SE4)のが正道。
- **縦書き・ルビ(S4・S9・LD9)**: `kCTVerticalFormsAttributeName`(CTStringAttributes.rs:216)・`kCTRubyAnnotationAttributeName`(:528)・`kCTFrameProgressionAttributeName`(CTFrame.rs:98)は Core Text の typesetter に文字組みを丸ごと渡す時だけ効く。shaper が cosmic-text(text.rs:3)の今は届かない。CTFont::path_for_glyph(CTFont.rs:1612)で vello へ出す道は在るが、それは shaper の置換で「技術は rerun の部品」に反する。
- **署名・公証(SE1)**: codesign / notarytool は CLI と Developer ID の話で、objc2 の範囲外。
- **winit が握る所**: NSApplication の生成・run loop・NSWindow の delegate(window_delegate.rs:132-136)は winit の物。Motolii が触ってよいのは main menu・app delegate・`NSDocumentController`・appearance の 4 つで、NSWindow の delegate を差し替えると winit の close / focus / theme が死ぬ。

見込み: AP1〜AP3 で H1・H2・H3・H4・H8・H12・H14 が ≒100 行、AP4 で S6・LD4・LD7 と再配布問題が ≒40 行、AP6 は 1 読み。AP7 は crate を取ってからもう一度来る。

## Lottie(bodymovin)仕様

弊社の仕様は手元に在る(`motolii/reference/lottie.schema.json`、地図 `reference/lottie-coverage.tsv` 738 行)。web は引かず、この 2 つと code で裏を取った。`L` = `crates/motolii-render/src/export/lottie`、`D` = `crates/motolii-doc/src`。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| LT1 | `export_lottie`(L.rs:61)は完成品 — comp/`fr`/`ip`/`op`、layer の `ip`/`op`/`st`/`sr`/`ao`/`bm`/`parent`/`tt`・`tp`・`td`、mask、marker(`cm`/`dr`/`tm`)、slots、shape、text まで書く。だが呼ぶ側が `export.rs:14` の `pub use` 以外に **0 件**、module 内 `#[test]` も **0**、`tests/golden` に Lottie の見本無し。Export sheet は `.mp4` 固定(`src/ui/export_sheet.rs:133,148-150`) | sheet に format 1 つ足して `LottieExport.json` を書くだけ。`unsupported: Vec<UnsupportedForLottie>`(L.rs:20-24)は既に人に見せる文言なので、完了行にそのまま出せる | SE5・X1 の format 分。新規 ≒30 行 |
| LT2 | S7: model は在る — `TextDocument.ranges`(D/store/text.rs:236)、`TextRange{name, selector, variation_axes}`(:184)、`TextRangeSelector{based_on, range_units, shape, randomize}`(:171)、property 束は `ids.rs:66-146`。描画は読まず(render/src で `.ranges` を読むのは export/lottie/text.rs だけ)、export も `text.rs:16-29` で「未実装」と報告して落とす | 仕様側は `text/text-data.a` → `text/text-range{nm,s,a}` → `text/text-range-selector` → `text/text-style`(`helpers/transform` を継承)で丸ごと定義済み。対応表(schema `$defs/text`): `text-data.a` ↔ `ranges`;`text-range.nm/s` ↔ `name/selector`;selector `b` ↔ `TextBasedOn`、`r` ↔ `TextRangeUnits`、`sh` ↔ `TextShape`(Square〜Smooth 6 種は同じ列挙)、`rn` ↔ `TextRandomize{seed}`、`s/e/o/a` ↔ `text_range_selector_start/end/offset/max_amount`(ids.rs:66-79);`text-style` の `fc/sc/sw/ls/t` ↔ ids.rs:90-107、transform の `a/o/p/r/s` ↔ `text_range_origin/opacity/position/rotation/scale`(ids.rs:118-134)。tsv の `text/text-range*` 行は全て 採用済 で evidence 付き | 書き出しは `a` 配列を組む ≒80 行。**評価(どの glyph に幾ら効くか)は弊社の仕様が文で定めるが code は無い** — GC4/LD6 の描く側は Motolii の仕事 |
| LT3 | keyframe: `Interp`(D/eval/track.rs:37-59)9 種のうち Hold/Linear/Bezier だけ `i`/`o`/`h` に写る(L/properties.rs:154-176)。Bounce/Elastic/Cyclic/Random/Steps/ElasticSteps は **`Err` で export 全体が止まる**(:174) | 弊社は `properties/easing-handle` の cubic 1 段のみ。落とすのではなく `bake_property`(:117-148、link/modulator は既にこれで frame 毎の Hold に焼いている)を通せば全 ease が出る | F11 の値グラフとは独立。分岐 1 つ |
| LT4 | spatial(F14): `SpatialTangent`(track.rs:444)を `to`/`ti` に書く encoder は在る(properties.rs:259-263)が `vector_property` が `spatial=false` 固定(:40)で position の path が落ちる | `properties/position-keyframe.to/ti` そのもの。引数を `true` に | 1 語 |
| LT5 | layer 種: `Null`/`Group`→`ty:3`、`Shape`→4、`Text`→5、`File`→2/6(L.rs:227-255)。Solid は無い(L9) | `layers/solid-layer{sc,sh,sw}`(tsv では 採用済 だが `rect_shape` へ畳む判断)。Create の Solid は矩形 shape で作れば export も今のまま通る | L9 は Motolii 側の 1 項目、弊社側は 0 |
| LT6 | E12 Pre-compose 無し。`TIME_REMAP` は layer 直付けで export は unsupported 報告(L.rs:210-220) | `layers/precomposition-layer{refId,tm,w,h,st}` + `assets/precomposition` が仕様に在る。tsv は GOALS で 不採用(fr/xt/refId)。採るなら `LayerSource` に variant 1 つで `tm` の落ちも消える | 裁定待ち。弊社が押せる物ではない |
| LT7 | blend 17 種(D/store/attrs.rs `BlendMode`)は `enums.rs:9-25` で 0-16 に全部写る。text の縁取り `sc/sw/of`(text.rs:127-131)、shape の `st/gs`+dash(shapes.rs:217-240)も写る | **落ちない**。依頼文の「縁取り・blend は落ちる」は当たらない | 訂正のみ |

弊社に無い物(正直に): mesh(`is_mesh_path` の拡張子は `build_media_asset` の表に無く `ty:2` へ倒して報告、L.rs:335-352)、点群(`ty:3` の空 null、L.rs:227-237)、ISF/effect 全部(`plugin_id` は文字列、弊社の `effects` は数値 id — L.rs:259-270)、camera(裁定65)、`pinned`、audio-settings の中身、可変フォント軸(`TextVariationAxis` ids.rs:146 — `text-style` に無い)、Shape の property animation(shapes.rs は全て `a:0` 静的)、runs(S8 — `text-document` は 1 スタイル、複数スタイルは弊社にも無い)。expressions(`properties/property.x`)と `ne/xe/sm` は弊社に在るが tsv で 不採用、押さない。

取り込み(Lottie → Motolii): **無い**。`lottie` を名に含む code は export 配下だけ。tsv の solid-layer 行に「読み込みは矩形シェイプへ写す」と書かれているが reader は存在しない。tsv 頭書が指す `lottie_coverage.rs`(schema と表の照合試験)も tree に無い — 地図の「試験が grep して確かめる」約束が今は空手形。

見込み合計: SE5 ≒30 行 + LT3/LT4 で 2 行 + S7 書き出し ≒80 行。描く側(GC4)と precomp(E12)は Motolii の裁定。

## ISF 仕様(ISF 2.0、mrRay/ISF_Spec README.md、`$F` = vidvox/ISF-FILES clone、MIT)

Motolii の ISF 実装は `crates/motolii-render/src/compositor/effects/isf/mod.rs`(parser + naga 翻訳)と `effects/vism.rs`(束縛・多段描画)。仕様の節名は README の見出し。

### 実装状況(仕様 × Motolii)

| 仕様の項 | 節 | Motolii | 裏 |
|---|---|---|---|
| JSON ヘッダ `/*{…}*/`、`DESCRIPTION`、`INPUTS`、`PASSES`(`TARGET`/`FLOAT`) | vsn 2.0 | 済 | isf/mod.rs:134-224、vism.rs:180-192・267-276 |
| `PASSINDEX`・`RENDERSIZE`・`isf_FragNormCoord` | ISF-created variables | 済 | isf/mod.rs:244-250・291-294、vism.rs:162-163・416-421 |
| `IMG_THIS_PIXEL`・`IMG_NORM_PIXEL` | Additional Functions | 済 | isf/mod.rs:296-297 |
| `IMG_PIXEL`・`IMG_SIZE`・`IMG_THIS_NORM_PIXEL` | 同上 | 未(#define 無し) | 同 file に 2 行しか無い。`$F` の 70/327 本が使う |
| `TIME`・`TIMEDELTA`・`DATE`・`FRAMEINDEX` | ISF-created variables | 未 | uniform は param + RENDERSIZE + PASSINDEX のみ(vism.rs:393-421)。`$F` の 43 本 |
| INPUT `image`・`float` | vsn 2.0 | 済 | isf/mod.rs:39-48 |
| INPUT `bool` | 同 | 半分 — `float` uniform に写す(isf/mod.rs:63)ので `if (flag)` が naga で落ちる | 手元の naga 29.0.4(Cargo.lock:4033)測定: float 写しで 210/327、`int` + `#define name (name__i != 0)` で 277/327 |
| INPUT `point2D`・`color` | 同 | 半分 — parser は 4 成分読むが実行時は成分 0 だけ(vism.rs:396-399、translate.rs:104-116 は `default[0]`/`min[0]`)。Document 側には `Value::Vec2`/`Color` が在る(motolii-doc eval/value.rs:19-20)のに translate.rs:51-56 が `F64` しか通さない | C12 の根 |
| INPUT `long`(`VALUES`/`LABELS`)・`event` | 同 | 未(from_isf_name が捨てる → 入力ごと消える) | `$F` の long 58 本・event 16 本 |
| INPUT `audio`・`audioFFT`(`MAX` = 列数) | 同 | 未 | `$F` の 7 本 |
| `PERSISTENT` | Persistent Buffers | 明示拒否 | isf/mod.rs:25-26・200-202、裁定 M5-FEEDBACK-P0(decision-index.md:117 「製品 render 自己 Feedback 禁止」) |
| `WIDTH`/`HEIGHT` の式 | vsn 2.0 PASSES | 未(常に描画サイズ、isf/mod.rs:79-80) | `$F` の 40 本 |
| `IMPORTED`・`.vs` + `isf_vertShaderInit()` | vsn 2.0 / Converting | 未(頂点は固定の三角形 isf/mod.rs:242-252) | `.vs` 同名 38 本は `varying` で naga が落ちる |
| `ISFVSN`・`VSN`・`CREDIT`・`CATEGORIES`・`LABEL`・`IDENTITY` | vsn 2.0 | 読まない(parser が拾う鍵は ID/EXPOSE/OUTPUT_FLOAT/PADDING/DESCRIPTION/INPUTS/PASSES) | bloom.fs:5-8 に CATEGORIES を書いているのに UI へ出ない |
| `inputImage` = filter、`startImage`/`endImage`/`progress` = transition | ISF Conventions | 未(判定無し) | `$F` は filter 207・transition 68・generator 52 |

弊社の側の正直な注記: bloom.fs の `ID`・`EXPOSE`・`OUTPUT_FLOAT`・`PADDING` は弊社仕様に無い Motolii 拡張、`ISFVSN` が無い(仕様は「必須と見なせ」)。

### 売り込み

- **IS1 bool を `int` + `#define` に、point2D/color を 4 成分で渡す** — isf/mod.rs:59-66・vism.rs:396-399・translate.rs:51-56 の 3 箇所、≒25 行。手元測定で `$F` の compile が 210→277 本。C12(効果の色 param)は仕様の `color` 型がそのまま答え。
- **IS2 `IMG_PIXEL`/`IMG_SIZE`/`IMG_THIS_NORM_PIXEL` と `TIME` 系の uniform** — #define 3 行と RenderInfo に float/int を 4 つ足す(vism.rs:407-415)。TIME は作品時刻(Document の時間)を渡せば任意時刻へ飛べる裁定と矛盾しない(TIMEDELTA/FRAMEINDEX は再生順依存なので 0 固定でも仕様違反ではない)。
- **IS3 `long`(pop-up)と `event`** — from_isf_name に 2 行、Document 側は `Value::Enum(i64)`(value.rs:23)が既に在る。Inspector の行は `VALUES`/`LABELS` で選択肢になる。`LABEL` を読めば C13 の「名前」は仕様で賄える。
- **IS4 C1/C2 を仕様の語で** — 効果の鎖そのものは仕様外(下記)だが、「`inputImage` を持つ物は filter として運用できる」(ISF Conventions)が鎖の定義: 各段の `inputImage` = 前段の出力、bypass = その段を飛ばす、同じ .fs を二度 = `EffectInstance`(store/effect.rs:16-19)が別 `EffectId` を持つだけで仕様上は自然。段内の順序は `PASSES` の配列順・`PASSINDEX` が正本(vism.rs:327-350 が既にそれ)。param 単位の「素通し」は仕様の `IDENTITY` 鍵(bool/float の bypass 値)。
- **IS5 audioFFT で A 波と繋ぐ** — 仕様は音を「1 行 = 1 ch、1 列 = 1 sample/FFT bin の float texture」として image 入力に流す。Motolii には `realfft 3.5.0` が既に Cargo.lock:6294(rubato 経由)、`audio/waveform.rs:139 columns()` と `meter.rs:58 observe_interleaved_stereo` に時刻→標本の口が在る。`MAX` の列数の Rgba16Float 1 枚を作って image slot に挿すだけで、`$F` の FFT Spectrogram / Waveform Displace 等 7 本が動く。A7 メーター・A8 ステレオも同じ texture の消費者にできる。
- **IS6 公開資産までの距離** — `$F` 327 本(MIT)。今日の subset(image/float/bool、TIME 無し、IMG_PIXEL 系無し、PERSISTENT 無し)で compile する filter は 34 本、IS1 で 42、IS1+IS2 で 92、IS3 まで 125、`.vs`(36 本)と PERSISTENT(49 本)を除いた全型で 161 filter / 277 file。残り 14 本は `WIDTH`/`HEIGHT` の式か GLSL 1.x 構文。測定は scratchpad の naga 29.0.4 ハーネス(repo 外)で「compile」まで、絵の一致は未測。
- **IS7 SE6 実行時読み込みは仕様の前提** — 「How ISF works: 読み込み時にメモリ上でコードを書き換える」。Motolii は build.rs:21-37 が `vism/` を include_str で焼き、device.rs:29-39 が起動時に全部 compile。`IsfProgram::compile(ctx, &str, format)`(isf/mod.rs:333)は既に文字列を取るので、`vism_definitions()`(effects/mod.rs:42-56)の `&'static` を外して `read_dir` の口を足せば `.fs` を落とすだけで載る。`load_shaders_from_disk` の hot path(vism.rs:53-71)も既に在る。

### 弊社に無い物(正直に)
- **効果の鎖・bypass・複製・並べ替え**(C1/C2)— 仕様は 1 file の中しか語らない。host(VDMX)の layer FX chain は仕様外。
- **PERSISTENT の代替**(motion blur・trail・optical flow の 49 本)— 仕様は「前フレームを保つ」以外の意味を持たず、Motolii の裁定(任意時刻へ飛ぶ・決定的 replay)とは相容れない。仕様で救えない。
- **`WIDTH`/`HEIGHT` の式評価**(DDMathParser 前提、40 本)、**サムネイル・検索・絵**(C13 は `CATEGORIES`/`DESCRIPTION` の文字までが仕様)、**キーフレーム・ease・Undo**、**書き出し**、**色管理**(仕様は linear/sRGB を語らない — CV2/CV4 は Motolii 側の問題)。
- **`.vs` の `varying`/GLSL 1.x** — 仕様の問題ではなく naga の GLSL 4.5 前端の問題。38 本は書き換えが要る。

## vgpu(Vercel Labs、外部証拠として)

前提: 弊社は runtime 依存にならない(decision-index.md:41)。売るのは**形**だけで、実装は naga / re_renderer の既存経路に写す。弊社側の裏は README.md、packages/wgsl/README.md(HMR behavior・Reflection)、docs/cli.md(`check`/`doctor`/`docs`/`examples`)、docs/topics/shader-workflow.docs.md、getting-started.docs.md、examples/triangle-led-front/{settings,scene-renderer}.ts。

Motolii の今(共通の観察): `vism/` は build.rs:8-40 が `include_str!` に焼き、`vism_definitions()` は `OnceLock`(effects/mod.rs:42-56)、header 不正は panic(同 :51)。debug 窓の reload は re_renderer が `begin_frame` で変更 path の module を作り直し(shader_module_pool.rs:146-166)、pipeline を作り直す(render_pipeline_pool.rs:205-238)。ただし WGSL の誤りは wgpu 側で**非同期**(shader_module_pool.rs:94、render_pipeline_pool.rs:228「We don't know yet if this actually succeeded」)、path 解決に失敗すると**空文字の module** を作る(shader_module_pool.rs:67-71 `unwrap_or_default`)。「失敗時に旧 pipeline を置換しない」のは layout / handle 欠落の `Err` だけで、文法誤りの絵は last-good にならない。

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| VG1 | 交換の前に検証が無い。reload は「変更された → 作り直す」だけ(上記)。起動時は device.rs:29-39 で全部 compile、1 本失敗で Engine ごと立たない | `vgpu check` は load の**前**に `{validation{mode,attempted,ok}, diagnostics(fix/where 付き), reflection, wgsl}` を JSON で返し、失敗でも payload を出して exit 1(cli.md「vgpu check」)。Motolii では同じ関数を naga(`wgsl-in` を 1 語足す — 今は glsl-in/wgsl-out のみ、naga 節)+ `parse_isf_source` で作り、**transaction の admission gate** と `motolii vism check <file>` の両方に使う。診断の行番号は NG2、device caps 一致は NG5 と同根 | ≒40 行 + feature 1 語。decision-index が「未完成」と書く「安定読取 → 検証 → 準備 → 同時交換」の 2 段目 |
| VG2 | 束縛番号は manifest の並びから決め打ち(vism.rs:17-30・36-47)、shader が実際に何を束縛したかは見ない。実例: tri_led.wgsl:7 は `glow` を INPUTS に宣言するが本文に `@group`/`@binding` が 1 つも無く、Inspector の行(tests/vism_catalog.rs:86-94 が name だけ確認)は絵に効かない。`MAPS` は parse する(isf/mod.rs:76・180・187)が消費者が無い(grep で他に無し) | reflection `bindings{group,binding,name,type,kind,layout}` + `overrides` + `hostShareableLayouts`(packages/wgsl README「Reflection」)。Motolii は naga `Module.global_variables` で同じ物が取れる(NG3 の WGSL 版)。「INPUTS に在るが束縛が無い」「本文が束縛するが INPUTS に無い」を VG1 の diagnostics に載せる | ≒20 行。tri_led の `glow` が空回りしている事実が試験で出る |
| VG3 | param は 1 つずつ別 uniform buffer + 別 binding(vism.rs:393-406、`param_count` 分の entry :159-163)。名前照合は host 側のみ(:398)。color/point2D は成分 0 だけ(IS1 と同根) | `struct Params { time: f32, texel: vec2f }` 1 本に `set({ params: { time } })` で**構造体のメンバ名**で入れる(getting-started)。layout は reflection の `hostShareableLayouts`。Motolii は manifest から 1 struct を生成し naga の layout で詰める → binding 数が param 数に依存しない、shader 側は `params.glow` | ≒40 行(vism.rs の bind 生成を 1 buffer に畳む)。C12 の 4 成分は IS1 で |
| VG4 | prelude を前置きした blend/matte は temp へ置き watch 対象外(wgsl_fragment.rs:43-44・67-71、vism.rs:53-71 の hash 名 staging)。reference/vello-blend.wgsl を直しても反映しない | 弊社 HMR は**推移的 import を bundler の watch graph に登録するだけ**(packages/wgsl README「HMR behavior」: `addDependency`/`addWatchFile`)。re_renderer は既に `#import` の依存を集めて再 compile 対象にしている(shader_module_pool.rs:150-155 `source_interpolated.imports`)。prelude を文字列連結でなく `#import` に変えれば staging が消え、依存も watch に載る | ≒−25 行。要確認: resolver の探索 path が crate 外(`reference/`)を許すか |
| VG5 | agent の検証口は tests/vism_catalog.rs(catalog 名 + 実画素の不一致)だけ。catalog を窓無しで列挙する口が無い。AGENTS.md は 15 行で index へ誘導 | `docs ls/cat/grep/find/symbols`、`examples search/pull`(sha256 検証付き)、`doctor`(実描画で環境判定)、agents.md/llms.txt(cli.md、shader-workflow「every claim about pixels must come from pixels you read」)。Motolii 版は `known_effects()` の JSON dump と `Engine::new()` headless 1 枚読みを 1 subcommand に | ≒30 行。VG1 の出力を再利用 |
| VG6 | tri_led は 2pass→1pass に畳んだ(tri_led.wgsl:15-19)。glow.wgsl は PASSES 4 段(glow.wgsl:12-17) | 弊社の元は draw() 3 本: floor-noise を **setup 時に 1 回だけ** target へ、毎 frame raycast→target→floor、LED 配列は storage buffer(scene-renderer.ts)。畳めたのは中間が**静的入力の関数**だったから(hash が代行)。分離 blur(blur_h/blur_v)は畳めない(段数 = 計算量)。「1 回だけ描く target」は frame を跨ぐ持ち越しで裁定 M5-FEEDBACK-P0 に触れる — 売らない | 他 vism への転用は無し(正直に) |
| VG7 | 共通 `VismProgram` の宣言語彙は ISF の INPUTS 5 型 + MAPS + PASSES(isf/mod.rs:29-104) | 弊社が持ち ISF に無い物: 構造体メンバの入れ子指定(VG3)、storage buffer 入力(LED 配列 — ISF の型に無い)、`override` 定数(reflection `overrides`; pipeline 定数なので keyframe には向かず、静的調整だけ)。弊社に無く ISF に在る物: DEFAULT/MIN/MAX(settings.ts の TUNABLE_DEFAULTS に範囲無し、viability §「宣言は誰が持つか」と一致)、LABEL/VALUES、PASSES/TARGET(host の TS が組む)、MAPS。なお MAPS の `EXPR` は WGSL そのものなので本文の関数で書ける — 弊社は宣言に式を持たない | 語彙の追加は storage 入力 1 型のみ検討。MAPS は消費者を作るか捨てるかの裁定待ち |

弊社に無い物(正直に): **last-good を保つ原子的交換そのもの**(HMR は bundler の watch graph 登録止まりで、失敗時に旧 module を保つかは bundler 側 — 弊社の文書に記述無し)。C13 の名前・絵(`CATEGORIES`/`LABEL` 相当無し、C13 は IS3 で)。C1/C2 の bypass・並べ替え(効果の鎖は host の TS)。時刻は `clock()` の壁時計(`time/deltaTime/frameCount`)で作品時刻ではない — Motolii の WGSL 経路には時刻 uniform 自体が無い(vism.rs:407-421 は render size + PASSINDEX のみ)が、答えは IS2 の TIME と同根。GLSL、Rust runtime、golden 比較の枠組み(workflow は「画素を読め」と言うだけ)。`set()` の未知名・型不一致の挙動は reference.md が 404 で未確認。

見込み合計: feature 1 語 + ≒130 行、staging −25 行。VG1+VG2 で decision-index.md:41 の「未完成」5 段のうち検証と layout 準備が埋まり、catalog/Inspector の同時交換(`OnceLock` の撤去)は Motolii 側の仕事として残る。

## After Effects(競合の model 営業)

読んだ物: `motolii/AGENTS.md`、`concept.md`、`decision-index.md`、`2026-07-16-ae-layer-system-disposition.md`、`2026-09-03-persona-backlog.md`(第 6 波 E)、`crates/motolii-doc/src/store/`。使い勝手は言わない。弊社(AE)の scripting object model の名で語る。

### 対応表(要点だけ)

| AE(scripting の class / 属性) | Motolii の今 | 判定 |
|---|---|---|
| `CompItem`(width/height/frameRate/duration/bgColor) | `Composition` store.rs:155-163、`/composition` 1 本 | 同等。多 comp は既決で作らない |
| `AVLayer`(footage / `SolidSource`)・`ShapeLayer`・`TextLayer`・null は `AVLayer.nullLayer` | `LayerSource::{File,Null,Shape,Text,Group}` store.rs:130-141、Solid は `rect_shape` の Shape 層 store.rs:39-75 | 同等。Solid を型にしないのは既決 |
| `adjustmentLayer` / `guideLayer` / `shy` / `motionBlur` / `collapseTransformation` / `frameBlendingType`(**全部 AVLayer の bool・enum で、kind ではない**) | `LayerAttrs` attrs.rs:118-136 に hidden/solo/locked/pinned/flatten/frozen/auto_orient。adjustment・shy・motion blur は無し | **AE1** |
| `trackMatteType` + `setTrackMatte(layer, type)`(AE 23 の任意層マット) | `Matte { layer, mode }` attrs.rs:109-113 | Motolii が先行(上の層固定ではない) |
| `parent`、`inPoint/outPoint/startTime/stretch` | `attrs.parent`、`LayerTiming { start, duration, source_in, speed }` store.rs:178-184 | 同等 |
| `timeRemapEnabled` + Time Remap property | `property::TIME_REMAP` + `SPEED` track resolve.rs:340-357 | 同等(speed の Bezier 積算は未) |
| `PropertyGroup` の木(`Transform` / `Effects` > effect > param / `Masks` > mask > Mask Path… / `Text` > Animators > Range Selector) | `PropertyId` は平らな文字列。`mask.<id>.shape`、`effect.<id>.param.<name>`、`text_range.<id>.selector.start` ids.rs:39-116。木は**命名規約としてだけ**在り、行は UI が手で組む(fixture.rs:43-55 `TRANSFORM_PROPS`、:269 `agg_props=[OPACITY,POSITION]` 決め打ち、:701 effect param) | **AE2** |
| `keyInInterpolationType/keyOut…`(LINEAR/BEZIER/HOLD)、`KeyframeEase{speed,influence}` の in/out、`keyTemporalAutoBezier`、`keyRoving` | `Interp` は**左キー 1 つで区間全体**を持つ track.rs:599-602、`Bezier{x1,y1,x2,y2}` は CSS 型 track.rs:40-45。加えて Bounce/Elastic/Cyclic/Random/Steps(弊社は式で書かせる物) | 区間 1 型で in/out 両端は表せる。roving・auto bezier は無いが写す価値は低い |
| `keyInSpatialTangent/keyOutSpatialTangent` | `SpatialTangent { in, out }` track.rs:444-447 | 同等(F14 は UI だけ) |
| `Property.expression`(JS 文字列) | `PropertyLink { source_layer, source_property, time_offset, plugin_id, params }` slot.rs:97-103、identity/linear/remap slot.rs:248-278 | 後述 |
| `MarkerValue`(comment/duration/label…)、comp marker と **layer marker**(`Layer.marker`) | `Marker { name, time, duration, body }` marker.rs:7-14、置き場は composition だけ view.rs:364-366 | **AE3** |
| `CompItem.workAreaStart/workAreaDuration` | 無し(`grep work_area` 0 件) | **AE4** |
| `RenderQueueItem { timeSpanStart, timeSpanDuration }` + `OutputModule` | `ExportJob { out_path, qp0 }` export.rs:36-39 | **AE4** の系 |
| `app.beginUndoGroup(undoString)` | `Document { head, tip, floor }` document.rs:205-214、`Intent` に名前無し :28-136 | **AE5** |
| `CameraLayer` / `LightLayer` | comp 直付けの 2D camera(center/zoom/roll)camera.rs:5-9 | 既決(2D User View)。売らない |
| precomp(`CompItem` を source にする `AVLayer`) | `LayerSource::Group` + parent、group.rs:11-58、Freeze apply.rs:478-487 | 既決で作らない(concept.md:222, 267-269) |

### 売り込み(Document の契約に足すと同族が一括で解ける物)

**AE1 層の振る舞いは kind でなく flag — `LayerAttrs` に 3 つ足す。** 弊社でも Adjustment / Shy / Motion Blur は `AVLayer` の bool であって Layer の種類ではない。Motolii の `LayerAttrs`(attrs.rs:118-136)は既にその形をしている(flatten = 弊社の collapseTransformation の裏返し)。`adjustment: bool`(描画側は「自分の下の合成結果を source にする層」の 1 分岐)、`shy: bool`(行の投影で捨てるだけ)、`motion_blur: bool`(comp 側の shutter と対で `Composition` に 2 値)。`LayerAttrsPatch` の写し apply_to(attrs.rs:196-232)に 3 行ずつ。E12 のうち precomp 以外が全部この 1 struct に乗る。「Layer に kind を持たせる」案は**弊社の model と逆**なので勧めない(Solid を Shape で作る既決とも整合)。

**AE2 `PropertyId` の名前空間から `PropertyGroup` の木を doc 側で 1 回引く。** 弊社は `Property` と `PropertyGroup`(INDEXED_GROUP / NAMED_GROUP)が同じ `PropertyBase` の木で、P/S/R/T/A も値グラフも「群を選ぶ = 子のキー全部」も同じ木を歩くだけ。Motolii は木を ids.rs の文字列規約(`mask.`/`effect.`/`text_range.` prefix、store.rs:98-104)で持っていながら、行の組み立ては UI が `TRANSFORM_PROPS` と `agg_props` を決め打つ(fixture.rs:43-55, 269)。`StoreView::property_tree(layer) -> Vec<(group, Vec<PropertyId>)>` を doc に 1 本置けば、E1 の残り(UU)、F11(層の行のキー=子の和、値グラフの縦軸=葉の型)、S7 の行(Range Selector = `text_range.<id>.selector.*` の群)、mask / effect の行が同じ関数の出力になる。**新しい型は要らない**、prefix の規約を関数に格上げするだけ。

**AE3 印に持ち主を持たせる。** `Marker` に `layer: Option<LayerId>`(None = comp 印)。弊社の layer marker は precomp の中身が親 comp の帯に出る唯一の窓で、Group を採る Motolii では「group の子の印が group の行に出る」がそのまま同じ意味になる。E14 の番号は `MarkerValue.label` 相当の `u8` 1 つ。

**AE4 `Composition` に work area 2 値。** `workAreaStart/workAreaDuration` を `Composition`(store.rs:155-163)に足すと、E12 の B/N、E13 の緑帯の範囲、`ExportJob` の `timeSpan`(export.rs:36-39、今は全尺固定)、再生の loop 範囲が 1 つの値を見る。preview==export の既決(decision-index:32)にも沿う(同じ範囲を同じ評価器で回す)。

**AE5 undo 群に名前。** 弊社の `beginUndoGroup(undoString)` は「1 batch = 1 文字列」以上の物ではない。`apply_all`(document.rs:316-338)の `at` に composition path で `UndoLabel(String)` を 1 component 書けば、履歴の行に操作名が出る(backlog:9 の ✗「model 側の口が先」の口)。rerun の timeline に乗るので時間旅行で自然に消える。

### 正直に: 弊社の売り物で Motolii に合わない物

**Expression(JS 文字列)は売らない。** concept.md:288 で既に不採用(diff 不能・補完不能)、Motolii の憲法(編集の意味だけ作る)にも合わない。弊社の expression が実際に売れている用途を分解すると (a) 他層の属性参照 + 時間ずらし(pickwhip = `thisComp.layer().transform.position.valueAtTime(time-n)`)、(b) 線形写像、(c) wiggle / loopOut、(d) lookAt。(a)(b) は `PropertyLink` slot.rs:97-103 と identity/linear/remap が既に**型付きで持っている**(F20 のピックウィップは UI だけ)。(c) は `Interp::Random/Cyclic` が区間の型として持っている。残るのは (d) `LookAt { target }` / `Follow { target, offset }` だけで、これは concept.md:288 が自ら名指しした物 — `translate_link` の match に plugin_id を 2 つ足す形で済み、文字列は要らない。`Value::LayerId`(value.rs:24)が既に在るので target は値で持てる。

**precomp は売らない。** Group が代替と既決。ただ 1 点だけ弊社から: precomp が持っていて Group に無い物は「子の時計」(precomp = 1 本の time remap で中身を丸ごと retime)。group.rs:36-38 は group に `LayerTiming` を与えているので、`TIME_REMAP` が group に置かれた時に子の `t` を書き換えるか(resolve.rs:348 は自層の source_frame だけ)は、Group で precomp を完全に置き換える上で残る 1 問。裁定待ちとして置く。

**Slot(共有 track、slot.rs:16-19)は弊社に無い。** 弊社では expression で参照するしかない物で、Motolii の方が正しい。

見込み: AE1 = attrs 3 flag + patch 3 行、AE2 = doc に関数 1 本(UI 側の決め打ち 2 箇所が消える)、AE3 = Marker に 2 欄、AE4 = Composition に 2 欄、AE5 = component 1 つ。いずれも新しい型を作らず、既存 struct への欄の追加だけ。

## rerun 上流(買い手として)

前提: fork `oshikaidesu/rerun` は上流 `954bf95a4`(2026-07-19、`0.35.0-alpha.1+dev`)の上に oshikaidesu の 16 commit(`ccbdad275`〜`346a0b3ba`、2026-08-11〜08-26)。`git diff --stat 954bf95a4..HEAD` = 31 files、+1461 / −59。checkout に上流の新しい ref は無い(`git for-each-ref` は master と motolii/* の 2 branch のみ)ので、上流 main との距離は「7 月 19 日から今日までの上流全部」としか言えない。`[patch]` 節は motolii/Cargo.toml に無く、全 re_* を同 rev の git 依存で引く(motolii/Cargo.toml:75-112)。

### 1. fork が持つ物と、Motolii が実際に呼ぶ物

Motolii の lock に居る fork crate は re_renderer / re_video / re_importer / re_sdk_types / re_types_core / re_chunk / re_chunk_store / re_entity_db / re_log_encoding / re_log_types / re_query など store 系と renderer だけ。**re_view_spatial・re_viewer_context・re_ui・egui は motolii/Cargo.lock に無い**。つまり fork 1461 行のうち:

| 場所 | 行 | Motolii が呼ぶか |
|---|---|---|
| re_renderer `context.rs` `RenderContext::new_from_device` + `device_caps.rs` `DeviceCaps::from_device` | +86 | 呼ぶ(compositor/device.rs:25) |
| re_renderer `texture_manager.rs` `import_gpu_premultiplied` / `copy_from_gpu_premultiplied` / `release_imported` / `is_premultiplied` | +176 | 呼ぶ(device.rs:143、sequential.rs:146,186,353,782、render_effects.rs:172) |
| re_renderer `dynamic_resource_pool.rs` `insert` + **`begin_frame` の閉包署名変更** `FnMut(&Res)`→`FnMut(Handle,&Res)`、`texture_pool.rs` `import` | +90 | 上の下請け |
| re_renderer `view_builder.rs` `new_with_external_resolved`(+268)、`main_target()`(+15)、tests/motolii_main_target_accessor.rs(+91) | +374 | 呼ぶ(sequential.rs:108,313,571,714) |
| re_view_spatial `spatial_stage.rs`(509)/`stage_camera.rs`(122)/`eye.rs`(+19)、re_viewer_context `store_hub.rs add_chunk`(+61)/`image_to_gpu.rs`、re_ui/re_viewer の Cargo feature 切り出し | ≒750 | **呼ばない**(egui/iced 埋め込み期の遺物。root workspace の spikes だけが別 rev `483b855` で引く: Cargo.toml:28-32,142-146) |

### 2. 査定

**UP1 `DeviceCaps::from_device` + `RenderContext::new_from_device` — 引き取る。** 「既にある wgpu::Device で RenderContext を建てる」は README の "can be used standalone" と整合し、adapter 無しの埋め込み(iced/blitz/bevy)全部が欲しい口。条件: (a) `new_impl` へ共通化した形のまま +86 なら CONTRIBUTING の「+100 −100 以下は議論不要」に収まる、(b) tier 判定(`adapter_info().backend` が GL なら Limited)を `from_adapter` と同じ表で出す、(c) `re_renderer_examples` に 1 本 device-driven の例。名前は `new_from_device` でよい。

**UP2 `TextureManager2D` の外部 GPU texture 取り込み — 条件付きで引き取る、ただし分割。** 「他所で描いた premultiplied な wgpu::Texture を GpuTexture2D として貸す」は妥当な要求だが、今の形は (i) `DynamicResourcePool::begin_frame` の閉包署名を変える破壊的変更(bind_group_pool / buffer_pool の呼び手も 2 行ずつ改変)と、(ii) `import_gpu_premultiplied` / `copy_from_gpu_premultiplied` / `release_imported` / `is_premultiplied` の 4 API が 1 塊。条件: まず `texture_pool.import` + `insert` だけを「所有しない資源を handle で参照する」PR(+49/−4 程度)として出し、`begin_frame` は closure 署名を変えずに handle を渡す別 method にして既存呼び手の diff を 0 にする。`is_premultiplied(key)` は fork の image_to_gpu.rs(re_viewer_context)のために生えた物で、Motolii は呼ばない → 出さない。`copy_from_*` は `import` の上に書ける helper なので二段目。

**UP3 `ViewBuilder::main_target()` — 引き取る(最小)。** +15 行の read accessor と test。裁定161(docs/reviews/2026-08-21-blend-fork-accessor-decision.md)の根拠 —`composite()` が呼び出し毎に unmultiply→srgb→premultiply を焼く— は上流でも通用する説明。条件: test file 名を `motolii_*` から中立な名前へ、doc comment の「Motolii seam(裁定256)」を消す。

**UP4 `ViewBuilder::new_with_external_resolved` — 今の形では引き取らない。** `new`(view_builder.rs:450-725)と本体 273 行のうち diff は 98 行、≒180 行が複製。上流の書き方は `TargetConfiguration` に `Option<resolved target>` を 1 欄足して `new` の中で分岐する形。条件: 複製を畳んで +60 以内、format/size/usage 検査は残す。これは UP2 の `texture_pool.import` に依存するので順番は UP2→UP4。

**UP5 re_view_spatial / re_viewer_context / re_ui の埋め込み seam(≒750 行)— 引き取らない、Motolii 側でも捨てる候補。** `SpatialStage` は egui Viewer 抜きで Spatial 3D を 1 枚動かす物で、Motolii の現行製品経路(blitz + re_renderer 直)は使っていない。上流には「毎リリース壊れる想定」の eframe 埋め込み例しか無く(decision-index.md:453)、これを本体に入れる議論は最初から大きい。spikes の rev `483b855` pin を落とせば fork の rebase 対象が **re_renderer 5 file だけ**に縮む。

**Motolii 固有(上流に出す物ではない)**: compositor/effects/vism.rs の自前 render pass(vism.rs:356 `begin_render_pass`、reference/owned-budget.tsv で 3 本まで許容)、ISF の GLSL→WGSL 変換(isf/mod.rs:303-318、naga 直)、vello-blend.wgsl の借用(wgsl_fragment.rs:23)、motolii-doc が EntityDb + rrd Encoder を文書 store に使う件(store/document.rs:13-18、persist.rs:39-77)は API の一般利用で fork 改変は無い。ただし SE3「.rrd の version と migration 無し」は上流の rrd 形式が動けば即痛む。

**R1〜R9 の売り込みは上流視点では「まだ使っていない既存 API」で、PR 材料ではない。** R6 だけ補足: media/probe.rs:161 の ffprobe は audio-only / 全 stream 列挙のため残す判定済み(docs/reviews/2026-08-30-media-delegation.md:85-98)。engine/texture.rs:411-441 は既に `re_video::VideoDataDescription` + `re_renderer::video::Video::frame_at` を使っており、thumbnail.rs:64 の ffmpeg spawn だけが残余。R7 ✗ は正しい(`load_gltf_from_buffer` は `&RenderContext` を要求: importer/gltf.rs:37-41)。

### 3. rebase の負債(半年後、file 単位)

| file | 種類 | 痛み |
|---|---|---|
| re_renderer/src/wgpu_resources/dynamic_resource_pool.rs | 既存署名の改変(`begin_frame`) | **最大**。上流が速く動く場所。bind_group_pool.rs / buffer_pool.rs / texture_pool.rs の呼び手 3 箇所も連動 |
| re_renderer/src/view_builder.rs | `new` の複製 273 行 | `new` が変わる度に手で同期。UP4 を畳めば消える |
| re_renderer/src/resource_managers/texture_manager.rs | 追加 176 行 | 追加のみだが `TextureManager2D` 内部 map の形に依存 |
| re_renderer/src/context.rs | `new`→`new_impl` 分割 | 中。上流が `new` を触ると conflict |
| re_view_spatial/src/{spatial_stage,stage_camera,eye}.rs、re_viewer_context/src/store_hub.rs、re_ui・re_viewer・re_viewer_context の Cargo.toml | 未使用 | 使っていないのに conflict だけ払う。落とすのが先 |
| Motolii 側 compositor/effects/vism.rs:9-11,115-199 | `RenderPipelineDesc` / `BindGroupLayoutDesc` / `ShaderModuleDesc` 直用 | fork 改変ではないが re_renderer の内部 pool API は semver 保証が無い。rev を上げる度に真っ先に赤くなる |
| motolii/Cargo.toml:75-112 と root Cargo.toml:142-146 の 2 rev | checkout 2 本 | dx が 7 GB を舐める問題(decision-index.md:510)の一因。1 rev に揃える |

### 4. 引き取り順の提案

UP1(単独、即)→ UP2-a(`texture_pool.import` + `insert`、閉包署名は触らない)→ UP3(accessor 15 行)→ UP4(`new` へ畳む)。UP5 は出さず、spikes の pin を落として fork から消す。利用者裁定「勘違いの線が濃い時は上流 PR を出さない」(decision-index.md:510、dx の件)は dx 側の話で、ここは fork の実 diff に基づくので該当しないが、UP2 の閉包署名変更だけは「上流の設計意図を読み違えている」可能性が残るので issue で先に聞く形が CONTRIBUTING(「大きい未議論の PR は無言で閉じる」)に沿う。

## 対応(2026-09-03 夕)

- AK2 ☑ host.rs が `BlitzShellEvent::Accessibility` の `ActionRequested` を 1 腕受け、keys.rs `act` → `press_node`(Enter/Space と同じ口)。Click/Focus/Blur。
- CT1 ☑ / CT2 ☑ / CT3 ☑ text.rs: `FontSystem` は process に 1 つ(`OnceLock<Mutex>`)、`load_system_fonts()`、locale "ja-JP"。`path` は台帳に無い family を足す口として残す。
- 残り(VL・AE・AP・FF・LT・IS・UP)は backlog に節を足すかの利用者裁定待ち。
