# iced 再生機構移植 — 証拠

2026-08-19。レーン: `claude/playback-20260819`(egui shell の再生機構 —
`playing` フラグ・audio clock 同期・loop — を iced shell へ移植)。

## 背景

近道キー移植レーン(`docs/reviews/evidence/iced-timeline-port/shortcuts/`)の
検収で判明: iced shell には再生機構が一切無かった(`playing` フラグも音の
クロックも無い)。そのため `Space`(再生/一時停止)と `L`(loop)が実装
できず、「機構が無い」とだけ報告されていた(同ディレクトリの README 参照)。
このレーンはその機構そのものを移植する。

## 何を足したか

### `motolii-ui`(共用 — 再実装しない)

`crates/motolii-ui/src/timeline_editor/playback.rs`(新規)に
`TimelineEditor` の新しいメソッドを足した:

- `toggle_playing()` — `Space`。掴んでいる最中は入り切りしない。
- `toggle_loop()` — `L`。
- `advance_playback(dt)` — 1 tick ぶん進める。
- `loop_on()` — 読み(`is_playing()` は既に mod.rs に在ったので増やしていない)。

中身は egui 版 `show()` の再生ブロック(`timeline_editor/mod.rs` の
`Key::Space` / `Key::L` 処理)と**同じ材料**である —
`audio_seat::open_playback` / `audio_seat::playhead_moved` /
`wrap_playhead` / `advance_playhead` を1つも書き直していない。egui 側の
`show()` はこの file を呼ばない(触っていない — 柵「egui shell 本体は
変更しない」を守る)。

### `motolii-ui/src/blitz_shell/intent.rs`(gateway の新しい口)

`ShellGateway` に `toggle_playing` / `toggle_loop` / `tick_playback` /
`is_playing` / `loop_on` を足した。**どれも `UiIntent` を経由しない**
(`poll_export` と同じ「非 intent だが gateway 経由」の型)。理由は
module doc の「ここに無いもの」と、下の「intent にしたか否か」を参照。

### `motolii-shell-iced`(配線)

- `message.rs`: `TogglePlayPressed` / `ToggleLoopPressed` /
  `ToStartPressed` / `PlaybackTick` を追加。
- `shell.rs`: 上記4つの top-level 処理。`TogglePlayPressed` /
  `ToggleLoopPressed` / `PlaybackTick` は `gateway` の非-intent 口を直接
  呼ぶ。`ToStartPressed` だけは既存の `UiIntent::SetPlayhead { at_us: 0 }`
  を dispatch する(新しい intent を作っていない)。`last_playback_tick:
  Option<Instant>` を新しい field として持つ(dt を測るためだけの host 側
  の刻み — toggle のたびに `None` へ戻す)。
- `window_input.rs`: command 修飾の無い `KeyPressed` を受ける新しいアーム
  を1本足した(`crate::shortcuts::playback_shortcut`)。
- `shortcuts.rs`: `playback_shortcut(key)` を追加。表の `Space` / `L` を
  `implemented: true` へ。
- `main.rs`: `shell.timeline_playing()` のあいだだけ `PlaybackTick` を
  刻む購読を1本足した(既存の `ExportPolled` / `WaveformPolled` /
  `StagePolled` と同じ「必要な間だけ」の型)。
- `timeline/semantics.rs`: `TimelineHit::ToStart` / `PlayPause`
  variant と、2つのボタン矩形を返す `to_start_button_rect` /
  `play_pause_button_rect` を追加。`hit_test` の transport 帯の判定を
  「表示専用」から「2ボタンだけクリック可」へ広げた。
- `timeline/canvas.rs`: transport 帯に to_start(⏮)・play/pause(▶/⏸)の
  2ボタンを描く(egui 版と同じ記号・同じ並び。フォントに無い記号は
  painter で三角/縦棒を直接置く egui 版の流儀をそのまま踏襲)。押下の
  hit-test・`mouse_interaction`(Pointer カーソル)も配線。

## intent にしたか否か(判断と理由)

**`toggle_playing` / `toggle_loop` / `advance_playback`(tick)はどれも
`UiIntent` にしていない。**

- **tick(時間経過)**: ミッション記述のとおり「再生は時間で進む =
  『操作』ではない」— `intent.rs` module doc が最初から `Timeline の
  zoom / pan / scroll` を intent にしないと明記している同じ scope。
  加えて、intent 化すると **replay ごとの実時間の揺れがそのまま
  journal に載る**という技術的な問題がある: `dispatch` は無条件に
  journal へ記録するので、tick を intent にすると「1 tick = 1 journal
  行」になり、しかも実行のたびに tick の回数・間隔が違う(壁時計 /
  audio device の実時間に依存)。これは replay oracle の前提(同じ
  intent 列 → 同じ結果)を壊す。
- **start/stop(`Space`)**: 同じ理由が波及する。「再生を始めた」だけを
  記録しても、次に記録される intent(たとえば scrub 確定)までに
  playhead がどれだけ進んでいるかは実時間依存で、replay 実行の速度が
  元の記録と一致する保証が無い。つまり start/stop だけを journal に
  載せても、その後の playhead 位置は replay で再現できない —
  「意図」として記録する価値が無い(むしろ嘘になる)。
- **loop の ON/OFF(`L`)**: egui 版 `TimelineEditor` 自身が
  `LoopRegion` を「Project session の状態で、Document には入れない」
  (`timeline_editor/mod.rs` の doc コメント)と明記している。session
  状態なら zoom/pan と同じ scope に置くのが一貫する。

一方 **`ToStartPressed`(先頭へ戻るボタン)は既存の
`UiIntent::SetPlayhead { at_us: 0 }` を再利用**した — こちらは
「playhead を特定の値に置く」操作で、scrub 確定と意味的に同じであり、
実時間に依存しない決定的な intent として replay できる。新しい
`UiIntent` 変種は作っていない(発注どおり)。

egui 側には `UiIntent`/journal の概念自体が無い(Timeline の編集は
`editor_mut` を直接叩く)ので、「egui の扱いに合わせる」という直接的な
参照点は無かった。上記の判断は、iced 側で先に確立していた
「zoom/pan・LoopRegion は session 状態」という分類を、時間経過にも
一貫して適用したものである。

## 音の座席をどう共用したか

**再実装していない。** `crates/motolii-ui/src/timeline_editor/
audio_seat.rs`(`AudioSeat` / `AudioPlayback` / `open_playback` /
`follow_audio_clock` 等)は1行も変更していない。`playback.rs`
(`timeline_editor` の子 module)から `super::audio_seat::` 経由で
既存の `pub(crate)` API をそのまま呼んでいる。クロックの正本は
引き続き `motolii-transport` の `Transport`(供給済みサンプル数)で、
`playhead` はそこから写すだけ(補間を発明しない)という規約もそのまま。

soundtrack が無い project(このレーンのテストで使った fixture すべて)は
従来どおり壁時計(`advance_playhead`)で進む。

## フレーム落ちの実測

このレーンで新しく計測はしていない。`advance_playback` の壁時計経路は
`MAX_STEP`(0.05s)で1呼びの進みをクランプする既存の安全弁をそのまま
使っており(「窓が隠れていた分をまとめて進めない」)、これは
`motolii-ui` のユニットテスト(`playback.rs::tests::
playing_advances_the_playhead_by_dt_on_the_wall_clock` 等)で確認して
いる。iced host 側の実フレームレート・実際のフレーム落ち率は実窓での
手動確認が要る(下記)。

## 手動確認事項(実窓でしか見えないもの)

- **音が実際に鳴るか**: soundtrack 付き project を `--project` で開き、
  `Space` で再生して聴感を確認する。このレーンのテスト fixture は
  すべて soundtrack 無しなので、壁時計経路しか自動では審判できない
  (`audio_seat.rs` 自体の実 device テストは既存 — このレーンでは
  soundtrack 付き project を通していない)。
- **Stage の追従の滑らかさ**: 再生中、Stage の絵が playhead に実時間で
  追いつくか(コマ落ちの体感)。pixel oracle は「止めたときに正しい絵か」
  は審判できるが、「動いている間、体感で何コマ落ちるか」は実窓が要る。
- **transport の2ボタンの見た目**: `docs/reviews/evidence/
  iced-timeline-port/playback/transport-buttons*.png`(下記)は
  headless の `--screenshot` で撮ったもの — 実窓のクリック感触・カーソル
  形状の切り替わりは実機で確認するとより確実。

## 証拠一覧

| file | 何の証拠か | 生成元 |
|---|---|---|
| `unit-tests.txt` | `motolii-ui` 側の再生ユニットテスト(dt を直接動かす、決定的)7件 green | `cargo test -p motolii-ui --lib timeline_editor::playback` |
| `shell-tests.txt` | `motolii-shell-iced` crate 全体 — lib unit test 30件 + 全 integration test(このレーンの `drive_playback.rs` 6件 / `playback_stage_pixels.rs` 1件込み)140件、**全 green・regression 0**(既存 `intent_gateway_fence` / `drive_timeline` / `snapshot_start_screen` 等も無傷) | `cargo test -p motolii-shell-iced` |
| `transport-buttons.png` / `transport-buttons-wgpu.png` | to_start / play / pause ボタンが transport 帯に出ている窓全体のスクリーンショット | `cargo run -p motolii-shell-iced -- --project <fixture>.json --screenshot .../transport-buttons.png 120` |
| `playback-stops-on-second-shot-wgpu.png` | 再生を1.5sぶん進めて止めた playhead で、Stage が2ショット目(青、占有 >0.7)を正しく映す pixel oracle の証拠。実 GPU adapter が在ったのでこの環境では skip されず実行された | `cargo test -p motolii-shell-iced --test playback_stage_pixels`(内部で書く) |

合計: `motolii-ui` 7件 + `motolii-shell-iced` 140件 = **147件 green、失敗 0**。
