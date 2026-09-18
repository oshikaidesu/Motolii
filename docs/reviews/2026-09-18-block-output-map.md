# ブロックの出力の配管の地図(2026-09-18、読み取り専用の調査)

## 要約(5 行)

1. `Offset` は WGSL の `struct Offset { translate: vec2f, rotate: f32, scale: f32 }`(block_program.rs:110)で 16 byte(`OFFSET_BYTES` block_program.rs:45)、Rust 側は `BlockOffset`(block_program.rs:31)。`NO_OFFSET` は同 :120。
2. state の buffer は物 1 つにつき 1 個の `Offset` を 2 本(ping-pong、block_program.rs:219)。合成は外枠が固定で、translate と rotate は**加算**、scale は**乗算**(block_program.rs:157)。
3. 描く道は state → `WorldPass`(block_program.rs:409-428)→ re_renderer の `motion` buffer(3 × vec4f / 物)→ 頂点の `motion_offset()`(fork の global_bindings.wgsl:92)。**`scale` はこの訳で落ちる**(block_program.rs:425 は translate と rotate だけ書く)。
4. 物理(Rapier)はブロックの前段: `solve_physics` の結果が `seed` として `begin_from` に入り(blocks.rs:608-622)、逆にブロックの出力は Rapier へ戻らない。書類へ戻るのは `set_physics_shifts` の平行移動だけ(blocks.rs:462-465、layout.rs:1833)。
5. 大きさ・不透明・色・時間のずれは今すべてブロックの外にあり、書類の component(`property::SCALE` / `OPACITY` / `SHAPE_FILL_COLOR`、store.rs:147-149・:133)と CPU の場(`apply_fields`、resolve.rs:630-678)、時刻は `layer_time` / `looped_time` / `schedule_shift`(layout.rs:735-782)が持つ。

## 配管(text)

```
書類(motolii-doc)
  resolve.rs  ResolvedLayer{ placement{transform, opacity, ...}, effects, masks }
      │  layer_time / looped_time / schedule_shift(時刻のずれ = Stagger・Loop・Split)
      ▼
render/engine/render.rs:330  prepare_blocks ─────────────┐
      │   layer_box・輪郭・住む箱 → BlockItem(48 byte)  │
      │   効果列 → BlockBatch{stage, plugin, params}     │
      │   SCOPE:room の物 → state.fields                 │
      ├─→ solve_physics(blocks.rs:532) → Rapier          │
      │        └→ set_physics_shifts(平行移動だけ書類へ)│
      ▼                                                   │
render.rs:331 build_layers → attach_block(blocks.rs:502)  │
      │   built.shading.params[23] = k + 1(物の番号)     │
      ▼                                                   │
render.rs:334 run_blocks(blocks.rs:600) ◄─────────────────┘
      │  BlockWorld::begin_from(seed = Rapier のずれ)      block_program.rs:313
      │  for stage in 0.. : for batch : BlockProgram::record_from  :240
      │        外枠: state_out[k] = Offset(s.t + d.t, s.r + d.r, s.s * d.s)  :157
      │  FollowPass(付いて置く札)  :347-401
      │  WorldPass  :404-428   Offset → motion[3k..3k+3] = (u*t.x+v*t.y, radians(rotate)), centre, axis
      ▼
re_renderer(fork)  motion buffer  @group(0) @binding(11)   global_bindings.wgsl:90
      │  motion_offset(slot, world_position)               global_bindings.wgsl:92
      ├─ instanced_mesh_base.wgsl:111(網)
      ├─ rectangle_vertex.wgsl:25 / rectangle_vs.wgsl:8(板)
      ▼
頂点が動く(位置と回りだけ。色・透明・大きさはここを通らない)
      │
      └─ ずれの後に箱で切る: cut_after_motion(blocks.rs:471)→ box_cut_in_comp(:479)→ cut_moved_in_their_boxes(render.rs:1178)
```

## 1. `Offset` の定義・NO_OFFSET・出力 buffer の layout

| 事 | file:line |
| --- | --- |
| WGSL の `struct Offset { translate: vec2f, rotate: f32, scale: f32 }` | crates/motolii-render/src/compositor/effects/block_program.rs:110(PRELUDE 内) |
| `NO_OFFSET = Offset(vec2f(0.0), 0.0, 1.0)` | block_program.rs:120 |
| Rust の `BlockOffset`(同じ並び) | block_program.rs:31-35、`Default` は scale = 1.0 :37-41 |
| byte 数 `OFFSET_BYTES = 16` / 物の箱 `ITEM_BYTES = 48` | block_program.rs:44-45 |
| byte への詰め・読み戻し | `offset_bytes` :60、`offsets_from_bytes` :64 |
| state の buffer 2 本(ping-pong)と `current` | `BlockWorld` block_program.rs:216-225、作り直しは :313-321 |
| `state_in` / `state_out` の束ね(binding 1 / 4) | block_program.rs:113,116 |
| `BlockItem`(block が読む物の箱) | block_program.rs:18-27、WGSL の `Item` :109 |
| `BlockHost`(time, members, objects, round, source) | block_program.rs:111、書き込み :257-264 |
| 作者が使える補助: `now_lo` / `now_hi` / `neighbor*` | block_program.rs:121-124 |
| block の本体の例 | vism/arrive.wgsl:27-54、bounce.wgsl:19-40、wave.wgsl:14-17、hang.wgsl:19-33、push_apart.wgsl:13-39、field.wgsl:39-41(NO_OFFSET を返すだけ) |

## 2. 出力を誰が読むか

| 事 | file:line |
| --- | --- |
| `program_for`(test の入口、棚の札から組む) | block_program.rs:483-487、`catalog_definition` :478 |
| `begin`(ずれ 0 から)/ `begin_from`(seed 付き) | block_program.rs:308 / :313 |
| `run_blocks` の道(597 行付近の「描く道」) | blocks.rs:600-646(seed :608、reach :616、begin_from :622、stage 順の記録 :625-636、FollowPass :641、WorldPass :643、`compositor.motion` :645) |
| 呼ばれる所(1 コマに最大 3 回: 通常・辿り直し・再組み) | render.rs:322/324、:330/334、:342/344 |
| `attach_block`(物の番号を描く側の欄へ、comp→world の基底) | blocks.rs:502-522、番号は `params[PARAM_SLOTS - 1]` :510(surface_program.rs:17-18) |
| WorldPass の WGSL(Offset → motion の 3 × vec4f) | block_program.rs:409-428。**scale を読まない** :425 |
| motion buffer の宣言と読み方(fork) | /Users/member_ottoto/rust_ae/rerun-s2-seam-20260818/crates/viewer/re_renderer/shader/global_bindings.wgsl:85-106 |
| 頂点で足される所(fork) | instanced_mesh_base.wgsl:111、rectangle_vertex.wgsl:25、rectangle_vs.wgsl:8 |
| `MotionBuffer` の作成と持ち回り | blocks.rs:642、compositor.rs:444、surface_scene.rs:104/635、render_basic.rs:91、sequential.rs:157/280 |
| 物理がブロックのずれを読むか → 読まない(逆向きだけ) | 前段 `solve_physics` blocks.rs:532-597 → `seed` blocks.rs:608-613 |
| 物理のずれを書類へ返す(平行移動だけ) | blocks.rs:462-465 → layout.rs:1829-1838(`PHYSICS_SHIFTS`、読むのは layout.rs:506) |
| layout がブロックの後にする事 | 直接は無い。`set_physics_shifts` は `prepare_blocks` の中(GPU のブロックの前)で、読むのは「つなぐ線・付いて置く札」の位置(layout.rs:506) |
| 付いて置く札が GPU の中でずれを追う所 | `FollowPass` block_program.rs:347-401(対 (札, 相手) で `translate` だけ足す :364)。対の作り方 blocks.rs:637-641、札の登録 blocks.rs:388-407 |
| ずれの後に箱で切る | blocks.rs:471-499、render.rs:1089-1096、render.rs:1178 |
| 間引きを止める(画面外から入る物) | render.rs:598(`self.blocks.moves(layer.id)`)、`moves` blocks.rs:121 |
| 測り・可視の読み戻し口(描く道では使わない) | blocks.rs:129/145/159/163/167/180/201、`read_state` block_program.rs:463 |

## 3. 物ごとの component は今どこに

### 書類(motolii-doc)

| component | file:line |
| --- | --- |
| `anchor` / `position` / `scale` / `rotation` / `opacity` / `skew` ほか | crates/motolii-doc/src/store.rs:143-165(`property` mod) |
| 塗りの色(単色) `shape.fill_color`、gradient は `fill.stop.<n>.color` | store.rs:133、:139-142 |
| 解決で opacity が入る所 | crates/motolii-doc/src/store/view/resolve.rs:543(`placement.opacity`) |
| CPU の場が scale・opacity・押し出しを書く(既存の前例) | resolve.rs:630-678(`apply_fields`、scale :663、opacity :665/672、push :667) |
| その欄の宣言 | crates/motolii-doc/src/store/layout.rs:80-83、表 :251-254 |
| Repeater の写しが opacity を割る | resolve.rs:994、:1085 |
| 時刻のずれ: `layer_time`(親の順番の札で子をずらす) | layout.rs:735-741 |
| `schedule_shift`(Stagger 本体、STAGGER / STAGGER_FROM / FROM_END) | layout.rs:763-782 |
| Loop(`layout.loop_duration` / `layout.loop_direction`) | layout.rs:98-99、表 :248-249、畳む式 `looped_time` :745-761、判定 :308 |
| Split(`text_split`)の単位の箱 | layout.rs:786-794(`text_units`)、名前 names.rs:60/:35、箱の切り出し text_frame.rs の `split_boxes` |
| GSAP 型の Stagger の配り(未追跡 file) | crates/motolii-doc/src/store/stagger.rs:1-185(`Stagger` 構造 :27-40、`distribute` :60-)。**`blocks.rs` からも `layout.rs` の `schedule_shift` からも呼ばれていない(別系統)** |

### 描く側(motolii-render)

| component | file:line |
| --- | --- |
| `Layer`(描く 1 枚)の持ち物 | crates/motolii-render/src/compositor.rs:290-311(`placement`・`blend_mode`・`shading`・`clip`・`frame` ほか) |
| 効果の欄の slot(24 個、最後の 1 個は物の番号) | surface_program.rs:16-18 |
| surface / field の hook の欄の組み方 | surface_program.rs:68-95(`snippet`)、:97-110(`program_desc`)、:112-(`params`) |
| opacity が層の間引き・板の合成で読まれる所 | render.rs:544、:558、:577、:624、:908、:1660 |

## 4. 複数の block が同じ物に掛かった時の合成

| 事 | file:line |
| --- | --- |
| 外枠(合成の式)— translate 加算・rotate 加算・scale 乗算 | block_program.rs:151-159(:157 が式) |
| 段(効果列の順)ごとに回す | blocks.rs:624-636(`stage` 0.. の二重 loop)、段の決まり方 blocks.rs:449-458 |
| `ROUNDS`(同じ block を n 回)— 毎回 copy して state を進める | block_program.rs:255-288、manifest の欄 isf/mod.rs:214 |
| ブロックごとの欄の buffer(uniform は別 buffer で渡す) | block_program.rs:247-253(`params_buffer`、96 byte = `array<vec4f, 6>`)、束ね :115 |
| 掛かった物の番号の buffer(`members`) | block_program.rs:245-246、束ね :117 |
| 同じ段・同じ plugin・同じ欄を束ねる `BlockBatch` | blocks.rs:14-21、束ね方 blocks.rs:454-457 |
| 近くの物の升目(CSR、`REACH` で広げる) | block_program.rs:73-106、reach の読み blocks.rs:616-620 |

## 5. 頭書き(ISF 型 JSON)を読む所と STAGE の種類

| 事 | file:line |
| --- | --- |
| `IsfStage`(pass / warp / surface / field / clip / block) | crates/motolii-render/src/compositor/effects/isf/mod.rs:26-37、名前の対応 :40-47、知らない STAGE の error :16 |
| `IsfScope`(members / room) | isf/mod.rs:226-234 |
| `IsfManifest` の欄(ROUNDS / REACH / SCOPE / PHYSICS ほか) | isf/mod.rs:192-224 |
| 既定値 | isf/mod.rs:236-262 |
| `IsfInput`(NAME / LABEL / TYPE / DEFAULT / MIN / MAX / SUBTYPE ...) | isf/mod.rs:106-133、型 :54-、成分数 :84 |
| block の札を棚に載せる所(欄は 1 成分だけ、module を組んで naga で検証) | catalog.rs:289-300 |
| `EffectStage` への訳 | catalog.rs:425-432、enum は catalog.rs:10-31 |
| 棚の札 → 窓の記述子(欄・範囲・単位) | catalog.rs:433-445 |
| module の組み立て(`BlockParams` struct と外枠) | block_program.rs:135-161、検証 :164-173 |
| 窓側の block の扱い | ui/native/src/editor/effect_sample.rs:95 |

**新しい STAGE / 新しい出力型を足す時に触る file**

- crates/motolii-render/src/compositor/effects/isf/mod.rs(enum・名前・manifest の欄・既定)
- crates/motolii-render/src/compositor/effects/catalog.rs(:289 の組み立て分岐、:425 の `EffectStage` への訳、:10 の enum)
- crates/motolii-render/src/compositor/effects/block_program.rs(PRELUDE の型・外枠の式・buffer の byte 数・WorldPass)
- crates/motolii-render/src/compositor/effects/surface_program.rs(:53 の stage の網羅 match)
- crates/motolii-render/src/engine/blocks.rs(段の集め方・seed・run_blocks)
- crates/motolii-render/src/compositor.rs(`Layer` と `motion` の持ち回り)
- ui/native/src/editor/effect_sample.rs(:95 の stage の網羅 match)
- fork: rerun-s2-seam-20260818 の global_bindings.wgsl(motion の layout)と読む側 3 file

## 6. 「ブロックは色を書かない」「色は component で場が書く」に関する既存の記述

| 出所 | file:line | 中身 |
| --- | --- | --- |
| 拡張の面の台帳(6) | docs/reviews/2026-09-17-extension-surface-ledger.md:165 | block が読む物と書く物の表。「書く: `Offset(translate,rotate,scale)` だが WorldPass は translate+rotate だけ motion へ(block_program.rs:425、**scale は落ちる**)。alpha・色・aspect・中心は書けない」 |
| 同上 | 2026-09-17-extension-surface-ledger.md:181 | 「欠け 1: block は translate+rotate しか書けない…→ Offset に scale と opacity を通すか、**色は pass に任せるかは法なので先に相談**」 |
| 最小の核(提案) | docs/reviews/2026-09-17-minimal-core.md:20 | 「物を動かす」の口。書く △: translate + rotate だけ(scale は WorldPass で捨て、alpha・色は無し) |
| 同上 | 2026-09-17-minimal-core.md:21 | 「物を作る」: Split の単位は物にならない(blocks.rs:235 の `copy == 0`) |
| 裁定の索引(最小の核) | docs/decision-index.md:641 | 「箱の矩形・変換・透明・**色**・画・札の値は (id, 時刻) の上の component」「Motolii の欠け: ブロックが translate + rotate しか書けない」。裁定待ち 6 件 |
| block の宣言に色の欄が無い | vism/arrive.wgsl:1-15 ほかの INPUTS | block の全 `.wgsl` の INPUTS に色(`TYPE: color`)は無い。catalog.rs:291-293 が 1 成分の型だけに制限している |

「ブロックは色を書かない」という明文のコメントは**コード中には無い**。事実上の制約は catalog.rs:291-293(欄が 1 成分だけ)と block_program.rs:110/425(出力の型と訳)にある。

## 7. 昨日の commit と block の出力の交わり

| commit | 交わる所 | file:line |
| --- | --- | --- |
| `bcdd6140e` Loop | 層の時刻を周期で畳んでから鍵・効果を読む。ブロックが読む時刻 `host.time` は comp の時刻(blocks.rs:634 の `t.as_seconds_f64()`)で**畳まれない**。畳まれた時刻を使うのは物理側の物ごとの frame だけ(`layer_time`、blocks.rs:448) | layout.rs:98-99、:745-761、blocks.rs:448、blocks.rs:634 |
| `54432172e` Split | 文字の単位の箱を作るが**書類に子の層を作らない**ので、物(`BlockItem`)にならない。blocks.rs は `copy == 0 && !ghost` の層だけを物にする | layout.rs:786-794、text_frame.rs の `split_boxes`、blocks.rs:235/271/296/321/391/410 |
| `20feb6c51` 関係で書き直す 束 1 | 書類の欄と台本だけ(docs と doc の layout)。block の出力は触っていない | — |
| `0594ed6e7` MaskFrame | ブロックが動かす層の箱の切りを、ずれの後に comp 大で掛ける道を足した。**block の出力に直接ぶら下がる唯一の後段** | blocks.rs:471-499(`cut_after_motion` / `box_cut_in_comp`)、render.rs:1089-1096、render.rs:1178、mask.rs:83-87 |

## Offset を広げる時に触る file の一覧と、行数規模の見積り

| file | 触る所 | 規模(目安) |
| --- | --- | --- |
| compositor/effects/block_program.rs | PRELUDE の `Offset` と `NO_OFFSET`(:110/:120)、`BlockOffset`(:31)、`OFFSET_BYTES`(:45)、`offset_bytes`/`offsets_from_bytes`(:60/:64)、外枠の合成式(:157)、FollowPass の WGSL(:354-365)、WorldPass の WGSL と record(:409-459) | 40〜80 行(型を 1 つ広げる = 6 箇所 + 訳 2 箇所) |
| engine/blocks.rs | seed の作り方(:608-613)、`attach_block` が渡す番号と基底(:502-522)、`block_states` の読み戻し(:201-207) | 10〜25 行 |
| fork rerun `.../shader/global_bindings.wgsl` | motion の 3 × vec4f の layout と `motion_offset` の戻り値(:85-106) | 15〜30 行(戻り値を vec3f から struct にすると読む側 3 file も) |
| fork `instanced_mesh_base.wgsl` / `rectangle_vertex.wgsl` / `rectangle_vs.wgsl` | :111 / :25 / :8 の呼び出し。色・不透明を返すなら fragment まで運ぶ配線が要る | 各 3〜10 行(fragment へ運ぶなら 30〜80 行) |
| fork `re_renderer/src/view_builder.rs` + `lib.rs` | `MotionBuffer` の大きさと binding(物 1 つ 3 × vec4f の前提) | 5〜20 行 |
| compositor/effects/catalog.rs | block の欄を 1 成分だけに縛る所(:291-293)を色の型まで許すなら | 5〜15 行 |
| compositor/effects/isf/mod.rs | 新しい出力型を manifest で宣言させるなら欄を 1 つ(:192-224 と既定 :236-262) | 5〜15 行 |
| compositor/effects/surface_program.rs | 物の番号の slot(:17-18、:510 と対)。物ごとの色・透明を欄で運ぶなら slot の割り当て | 5〜20 行 |
| compositor.rs / surface_scene.rs / render_basic.rs / sequential.rs | `motion` の持ち回り(compositor.rs:444、surface_scene.rs:104/635、render_basic.rs:91、sequential.rs:157/280)。buffer の形が変われば型だけ | 各 1〜3 行 |
| motolii-doc store/layout.rs | ずれを書類へ返す口(`PHYSICS_SHIFTS` :1829-1838)を平行移動以外へ広げるなら | 10〜30 行 |
| vism/*.wgsl(block の 6 file) | `Offset(...)` の呼び出し(arrive.wgsl:53、bounce.wgsl:40、hang.wgsl:33、push_apart.wgsl:39、wave.wgsl:16、field.wgsl:40)。既定値を足す形なら位置引数の書き換え | 各 1〜3 行 = 計 10 行前後 |
| tests/vism_catalog.rs、block_program.rs の test(:490-543)、blocks.rs の test(:649-) | 型が変われば組み直し | 10〜40 行 |

合わせて **Motolii 側 100〜200 行、fork(rerun)側 30〜140 行**。

## 分からなかった事

- `placement.opacity` が実際に GPU の draw へ入る最終地点(surface_scene.rs のどの行で instance / rectangle の色へ掛かるか)は追い切れていない。render.rs での使われ方は間引きと板の合成の判断のみ確認した。
- `crates/motolii-doc/src/store/stagger.rs`(GSAP の `distribute`)の呼び出し元が見つからない。`layout.rs:763` の `schedule_shift` は独自の式で、stagger.rs を使っていない。未接続か、別の口(台本側)から使う想定かは不明。
- `MotionBuffer::new` の実体(fork の view_builder.rs 内)の中身は確認していない。buffer の大きさの式と binding の作り方は未読。
- Split の単位(`text_units` / `split_boxes`)が `text_frame.rs` のどの行かは行番号まで特定していない(commit 54432172e が同 file に 45 行足している)。
- `docs/` と `motolii/docs/` の grep では「ブロックは色を書かない」という明文は見つからなかった。近いのは extension-surface-ledger.md:181 の「色は pass に任せるかは法なので先に相談」まで。

## 追記: この調査の最中に作業樹が動いた(HEAD ではない)

この地図は **HEAD(6cfb7b8ff)** を読んで書いた。調査の途中で作業樹に未 commit の変更が入り、`Offset` が既に 5 つ目の欄を持っている。

| file | 作業樹の状態 | 行 |
| --- | --- | --- |
| block_program.rs | `struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f }` | :113(HEAD は :110 で 4 欄) |
| block_program.rs | `NO_OFFSET = Offset(vec2f(0.0), 0.0, 1.0, vec4f(1.0))` | :123 |
| block_program.rs | `BlockOffset { .., tint: [f32; 4] }`、既定 `[1.0; 4]` | :31-42 |
| block_program.rs | 詰め・読み戻しが 32 byte に | :64、:70 |
| block_program.rs | 外枠の合成: tint は**乗算** | :160 |
| block_program.rs | FollowPass / WorldPass の `Offset` も 5 欄、motion に `motion[base + 3u] = tint` が増えた | :357/:367、:413/:432 |
| vism の block 6 file | `Offset(..., vec4f(1.0))` へ書き換え済み(arrive/bounce/hang/push_apart/wave) | arrive.wgsl:53 ほか |
| blocks.rs | 1 行だけ変更 | — |

`git status`: `M block_program.rs`, `M blocks.rs`, `M vism/{arrive,bounce,hang,push_apart,wave}.wgsl`。
上の「触る file と規模」の見積りのうち、block_program.rs(39 行)と vism の 6 file は**既に着手済み**にあたる。fork(rerun)側の motion の layout(3 × vec4f → 4 × vec4f)と読む側は、作業樹の grep では未確認。
