# probe の関数グループ台帳 — 全関数を 8 群へ

- 日付: 2026-09-03
- 位置づけ: **設計提案**(コードに触らない)。[ホットリロード設計](2026-09-03-hot-reload-replaceable-functions.md) §2.6 の
  3 原始(Lens / Gesture / Control)を、`main` c2c7a89 + PR #479 の `probe/src` に在る**全関数**へ当てはめた台帳。
  次のセッションが**機械的に `git mv` と関数の切り出しだけ**で移せる粒度にする(挙動は 1 mm も変えない)
- 発端: 利用者「関数のグループを作ってみませんか。コードが見やすくなるはず」

## 0. 群は 8 つ、依存の向きは 1 方向

```text
view ──▶ table ──▶ { control, gesture, paint, verb } ──▶ { lens, read } ──▶ motolii-store / core
 ▲                                                                              
 └── shell(副作用: Document 書き込み・GPU・window・clock)は全部を使い、誰からも使われない
```

| 群 | 役 | 状態 | 副作用 | hot |
|---|---|---|---|---|
| **lens/** | 性質 1 つの get / set / delta | 無 | 無(Intent を返すだけ) | patch |
| **read/** | Document → 表示用の行・値(Lens を束ねた読み) | 無 | 無 | patch |
| **verb/** | Lens を跨ぐ Intent 生成(Split・Duplicate・Delete・spawn)と `targets`/`lift` | 無 | 無 | patch |
| **gesture/** | `step: (State, &UiEvent) -> (State, Option<Msg>)` と当たり判定 | data(殻が持つ) | 無 | patch |
| **control/** | `Value` ↔ 面(スクラブの増分・トグル・key ◇・色) | 無 | 無 | patch + RSX |
| **paint/** | `(&View, &mut Scene)`(帯・ダイヤ・枠・色) | 無 | 無(Scene に描くだけ) | patch |
| **table/** | 配線表(打鍵・献立・登録表)。**static ではなく fn** | 無 | 無 | patch |
| **view/** | `rsx!`(並び・条件・見た目) | Signal | 無 | **RSX** |
| **shell/** | `impl Widget`・`app`・`Session`・`Clock`・GPU・`apply_all` | 有 | **有** | 再ビルド(触る頻度が低い) |

柵(grep で機械検収):

- `lens/ read/ verb/ gesture/ control/ paint/ table/` に `&mut Document`・`Arc<Mutex<Document>>`・`wgpu::`・`Signal<` が現れない
- `shell/` 以外に `impl Widget`・`apply(`・`apply_all(` が現れない
- `table/` に `static` が現れない(§0 の subsecond 制約)

## 1. 台帳(probe/src の全関数、test を除く)

凡例: **群** = 移す先 / **形** = 移した後の署名(変える物だけ。「—」は同じ) / **備考**

### app.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `app` | shell | — | `IntentCtx` の組み立てと Signal だけ残す。keydown は `table::keymap::lookup` → `verb::lift::run_intent`(既にそう) |

### browser.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `new_layer_intents` | verb/spawn | — | 既に純関数 |
| `spawn_layer` | shell | — | `apply` を呼ぶ殻。`new_layer_intents` → `apply_all` → `refresh_rows` の 3 行にする |
| `add_effect` | **lens/effect** | `set(target, plugin_id) -> Vec<Intent>` | 今は doc を lock して書く。読み(`effects()`)+ `SetEffects` の Lens に分け、書きは殻 |
| `set_shape_fill_color` | lens/color | — | 既に純(ShapeNode を書き換える helper) |
| `apply_layer_color` | **lens/color** | `set(target, rgba) -> Vec<Intent>` | 同上。Browser と Inspector の両方から使う = 2 面以上 |
| `browser_panel` | view | — | rsx だけ残す。onclick は `verb::lift` 経由 |
| `gpu` / `readback` | shell(test 器具) | — | test 専用。`fixture.rs` か `tests/` へ |

### context_menu.rs(#479)

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `entries` | table/menu | — | 既に純関数、既に表 |
| `context_menu` | view | — | rsx |

### dispatch.rs(#479)

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `IntentCtx::comp_frame` | lens/playhead | `get(clock) -> Frame` | playhead も Lens(書き先が clock なだけ) |
| `IntentCtx::refresh_rows` | shell | — | Signal と channel を触る殻 |
| `IntentCtx::set_selection` | shell | — | 同上 |
| `duplicate_layer` | **verb/duplicate** | `fn(&StoreView, LayerId) -> Vec<Intent>` | 今は doc を lock して `apply_all` まで行う。**Intent を返すだけにして書きは殻**(`split_layer` も同じ) |
| `run_intent` | verb/lift | — | `targets` + `lift` をここへ足す([持ち上げ](2026-09-03-selection-lifting.md) B-1) |

### fixture.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `canvas_rows_from_doc` | read/rows | — | 既に純 |
| `layer_rows_from_doc` | read/rows | — | 既に純 |
| `label_rgb` / `hex_of` | paint/palette | — | 色の割り当て |
| `push_swatch` / `shape_fill_colors` / `used_colors_from_doc` | read/colors | — | 既に純 |
| `AssetFamily::label` / `asset_family` | read/asset | — | 既に純、test あり |
| `inspector_data_from_doc` | **read/inspector** | — | Lens の束。**Lens が揃えば `PropRow` を Lens 名の列で組める**(§2.6.5) |
| `admit_testdata` / `load_fixture` | shell(検分器具) | — | `MOTOLII_TILT` などの器具。`fixture.rs` を **器具だけ**にする |
| `fmt_timecode` | control/time | — | 表示 |

### inspector.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `increment` | control/scrub | — | 既に純 |
| `nudge` | control/scrub | — | 既に純。`Value -> Value` |
| `write_key` | **lens/property** | `set_key(target, prop, t, value) -> Vec<Intent>` | 今は doc を書く。Intent を返すだけへ |
| `commit_drag` | shell | — | `nudge` → `lens.delta` → `apply_all` の殻 |
| `write_content` | **lens/text** | `set(target, text) -> Vec<Intent>` | 同上 |
| `prop_row` / `content_row` | view | — | rsx。**`PropRow` の `cells`/`dims` は Lens から出せる** |
| `inspector_panel` | view | — | rsx |

### keymap.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `BINDINGS`(static) | **table/keymap** | `fn bindings() -> &'static [Binding]` | static のままだと hot-patch で変更が反映されない |
| `lookup` / `is_bound` | table/keymap | — | 既に純、test あり |

### playback.rs / session.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `Clock::*` | shell | — | 時計は副作用(スレッド・audio clock 予定) |
| `Selection::*` / `Session::new` | shell | — | 選択は Arc<Mutex>。読み口(`get`/`all`/`contains`)は `read/` から使ってよい |

### stage_widget.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `c` | paint/palette | — | `timeline_widget::c` と重複 → 1 つに |
| `Fit::to_comp` | gesture/stage | — | 画面→comp の座標写像。観測カメラが入るとここが `camera_screen_from_world_at_z` の逆へ |
| `current_rt` | lens/playhead | — | `comp_frame` と同じ物 |
| `selection_geom` / `selection_geom_in` | **read/geometry** | `fn(&StoreView, &Engine, LayerId, t) -> Option<SelGeom>` | 枠の幾何。カメラ設計 §3.2 で四隅投影に変わる場所 |
| `vec2_at` / `f64_at` | **lens/property** | `get(view, target, prop, t) -> Value` | **これが Lens の get の原型**。Inspector の `inspector_data_from_doc` と重複している |
| `compute_scale` / `compute_rotation` / `rotate_around` | gesture/stage | — | 既に純。1 ジェスチャ 1 回 |
| `track_intent` | **lens/property** | `set(target, prop, t, value) -> Vec<Intent>` | **これが Lens の set の原型**。`write_key` と同じ物が 2 つある |
| `create_target` | shell | — | wgpu |
| `layer_under` | gesture/stage(純部分) | `fn(&[SelGeom], cx, cy) -> Option<LayerId>` | 今は `&self` で state を読む。幾何の列を受けて判定だけにする |
| `handle_event` | **shell + gesture/stage** | 殻: `step` を呼んで `GizmoDrag` を保持 / 核: `step(GizmoDrag, &UiEvent, &[SelGeom]) -> (GizmoDrag, Option<Msg>)` | 130 行の match を殻 10 行 + 核へ |
| `paint` | **shell + paint/stage** | 殻: `subsecond::call(\|\| paint::stage(..))` / 核: `fn(&StageView, &mut Scene)` | 枠・ハンドルの描画 |
| `connected`…`destroy_surfaces` / `requires_redraw` | shell | — | Widget 契約 |

### timeline_widget.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `c` | paint/palette | — | 重複 |
| `keyframe_shift_intents` | **lens/timing** | `delta(target, frames) -> Vec<Intent>` | 帯 Move の「key 追随」。Lens の delta そのもの |
| `TimelineWidget::new` / `with_*` / `sender` | shell | — | 組み立て |
| `sfac` / `max_scroll_y` / `set_scroll_y` | gesture/timeline | 純部分は `fn(rows, viewport_h, sfac) -> f64` | scroll 状態は殻の data |
| `band_hit` / `hit_test` | **gesture/timeline** | `fn(&TimelineView, x, y) -> Hit` | 今は `&self`。view struct(rows・pps・scroll)を受ける |
| `process_messages` | shell | — | channel |
| `attrs_to_patch` | lens/attrs | — | 既に純 |
| `split_layer` | **verb/split** | `fn(&StoreView, LayerId, frame) -> Vec<Intent>` | `duplicate_layer` と同じ変更(書きは殻へ) |
| `fill_rect` / `diamond` | paint/primitives | — | 既に純 |
| `handle_event` | **shell + gesture/timeline** | 核: `step(DragState, &UiEvent, &TimelineView) -> (DragState, Option<Msg>)` | `PointerUp` の `LayerTiming` 計算(Move/TrimStart/TrimEnd の clamp)は **lens/timing の delta** へ |
| `paint` | **shell + paint/timeline** | 核: `fn(&TimelineView, &mut Scene, w, h, scale)` | 190 行。殻は `process_messages` と `subsecond::call` だけ |

### thumbnail.rs / tokens.rs / tri_widget.rs / timeline_shell.rs / main.rs

| 関数 | 群 | 形 | 備考 |
|---|---|---|---|
| `image_data_uri` / `video_data_uri` / `encode*` | read/thumbnail | — | I/O を含むが Document を書かない。read に置く(ffmpeg 起動は将来 `motolii-media` 委譲先へ) |
| `hex` / `UiScale::*` / `css_root` | control/tokens | `css_root` は fn のまま | static にしない |
| `TriWidget` / `ActiveRenderer` 一式 | shell(検分器具) | — | Triangle LED の器具。`probe/src/tools/` へ |
| `timeline_shell` | view | — | rsx。M/S/L の onclick は `lift(lens::attrs.set)` |
| `main` | shell | — | — |

## 2. 数えると

| 群 | 関数数(概算) | うち今すぐ `git mv` で済む | 切り出しが要る |
|---|---|---|---|
| lens | 9 | 3(`attrs_to_patch`・`set_shape_fill_color`・`keyframe_shift_intents`) | 6(`vec2_at`/`f64_at`/`track_intent`/`write_key`/`add_effect`/`apply_layer_color`: **doc を書く部分を殻へ返す**) |
| read | 9 | 9 | 0 |
| verb | 4 | 1(`new_layer_intents`) | 3(`split`/`duplicate`/`run_intent`: 書きを殻へ) |
| gesture | 9 | 4(`compute_*`・`rotate_around`・`Fit::to_comp`) | 5(`hit_test`/`band_hit`/`layer_under`/2 つの `handle_event`: `&self` を view struct へ) |
| control | 5 | 5 | 0 |
| paint | 6 | 4 | 2(2 つの `paint` の本体) |
| table | 3 | 2 | 1(`BINDINGS` を fn へ) |
| view | 6 | 6 | 0 |
| shell | 残り | — | — |

**重複が 3 組見つかった**(この台帳を作った副産物):
`c`(2 箇所)、`vec2_at`/`f64_at` と `inspector_data_from_doc` の値読み、`track_intent` と `write_key`。
全部 Lens が無いために面ごとに書かれた物で、lens/ を作ると自然に 1 つになる。

## 3. 順序(挙動を変えない移動だけ)

1. ディレクトリを切って **`git mv` で済む物を動かす**(read・control・view・table の 22 本)。ビルドが通る以外に何も変わらない
2. `lens/property.rs` を `vec2_at`/`f64_at`/`track_intent` から作り、`write_key`・`inspector_data_from_doc` を乗せ替える(重複 2 組が消える)
3. `verb/` の 3 本を「Intent を返すだけ」へ(書きは `run_intent` の殻へ)
4. `gesture/` の `hit_test`/`band_hit`/`layer_under` を view struct 受けへ
5. 2 つの `handle_event` を殻 + `step` へ、2 つの `paint` を殻 + 核へ(`subsecond::call` はこの時に 1 行)
6. `table/keymap` の static を fn へ

1〜2 で読みやすさの利益はほぼ出る。3〜6 が [持ち上げ](2026-09-03-selection-lifting.md) と [ホットリロード](2026-09-03-hot-reload-replaceable-functions.md) の前提を作る。

## A. 利用者裁定待ち

| 件 | 分岐 |
|---|---|
| 群を **ディレクトリ**にするか **1 ファイル 1 群**にするか | 推し = ディレクトリ(`lens/property.rs` `lens/attrs.rs` …)。1 ファイル 1 群だと lens が 300 行を超える |
| `fixture.rs` の名 | 推し = `tools.rs`(検分器具だけになる)。`load_fixture` は `motolii-fixture` crate の呼び出しに痩せる |
| `shell/` に `app.rs` を入れるか root に残すか | 推し = root(`main.rs` の隣。入口は見える所に) |
