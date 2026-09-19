# 速さの的 — 引き継ぎ用の 1 枚(2026-09-19)

この文書だけで着手できるように書く。Motolii = Rust(描画・書類)+ Flutter(窓)の映像編集ソフト。
測った物は全部 **debug build**(`scripts/motolii-ui.sh dev`)。本番は `scripts/motolii-ui.sh profile`(Rust `--release` + Dart AOT、hot reload 無し)。
利用者の立場: 「開発用で遅い時は本番も遅い」。build の種類は言い訳にしない。置き場所の間違いを直す。

## 守る物(これを壊したら直しではない)

1. `PROBE room=stage-window verdict=matched lag=0` が出続ける。Stage の伸縮と Fit で、頼んだ窓(roi)の絵がその同じコマに出る。`lag=0` 以外の行が 1 本でも出たら退行。
2. 掴む契約: drag → preview → commit。Esc と focus 外れで取り消し、1 drag = undo 1 回。
3. 見た目の値は変えない(`EditorMetrics` / `EditorTheme` の token。lint `raw_dimension` / `raw_color` / `material_import`)。
4. `flutter test` の落ちる本数を増やさない(下の基準を自分で 1 回測ってから比べる)。
5. 書き出し(`export_range_with_progress`)と `motolii/crates/motolii-render/examples/zz_watch` は同期の描画を必要とする。非同期にするなら同期の口を残す。

## 測り方(毎回同じで)

```bash
# 窓を出す(debug、hot reload あり)
scripts/motolii-ui.sh native && scripts/motolii-ui.sh dev
# 本当の速さ(release + AOT、hot reload 無し)
scripts/motolii-ui.sh profile
```

計器は `kDebugMode` の裏に入れる。1 コマごとに: `_renderNow` の各段の ms、`SchedulerBinding.addTimingsCallback` の build / raster / total、16.7 ms 超と 33 ms 超の本数、12 秒のうち UI thread が `_renderNow` に居た割合。
前後を同じ書類・同じ秒数で。中央値と p90 と最大を出す(平均は使わない)。

## 的(重い順。1 と 2 は着手中)

### 1. GPU の待ちが UI thread の上にある ← 本丸

`motolii_probe_render`(`motolii/ui/native/src/lib.rs:203`)が `render_into` を呼び、その中で `device.poll(wgpu::PollType::wait_indefinitely())` が GPU の描き終わりを待つ。Dart は再生の Ticker から**同期に**呼ぶので、1 回描くたび UI thread が **中央 12 ms・p90 60 ms** 止まる(予算 16.7 ms)。

| 段(12 秒・468 回、ms) | 中央 | p90 | 最大 |
|---|---|---|---|
| `_renderNow` 全体 | 10.16 | 49.31 | 59.23 |
| render Camera (GPU) | 4.36 | 26.97 | 35.78 |
| render Stage (GPU) | 3.53 | 17.94 | 22.12 |
| status(軽い、5.9 KB) | 1.62 | 3.01 | 4.67 |
| jsonDecode | 0.08 | 0.14 | 4.04 |
| `_accept` | 0.34 | 0.50 | 0.93 |

UI thread が `_renderNow` に居た時間 = 12 秒中 **72 %**(同じ絵を飛ばす直し後で 61 %)。
**直し**: submit して返る。描き終わりは `Queue::on_submitted_work_done` か別 thread の poll で、既にある C の callback `motolii_probe_set_frame_ready(ctx, cb, user)` を鳴らす。roi は submit の前に決まるので「遅れ 0 コマ」は保てるはず。面ごとに surface は 2 枚(`_flip`)あるので、まだ signal の来ていない surface に描かない番人を入れ、飛ばした回数を数える。
**目標**: UI thread の割合 20 % 未満、33 ms 超のコマをほぼ 0。

### 2. Timeline の painter が `paint()` の中で作る

`motolii/ui/lib/panels/timeline.dart:~1532` の `_TimelinePainter.paint()` が `Paint` を約 30、`Path` を数個、毎回作る。借り物の規則(`.claude/skills/flutter-skills/custom-canvas-and-gestures`、MIT)は「`paint()` の中で `Paint`/`Path`/`Gradient` を作らない。field に持って安い属性だけ変える。`shouldRepaint` は不変値 1 個の比較。ticker の `setState` ではなく `repaint:` の Listenable」。
今は上位ではない(raster 中央 1.3 ms)が、1 が済むと次に来る。

### 3. 見ていない面も描いている

1 tick で Stage と Camera の 2 枚を描く。`_shownViews`(`editor_session.dart`、`stage.dart:86,1099`)で「どの tab も見ていない面は描かない」を入れたが、利用者の `layout.json` は Stage と Camera を左右に並べているので**この人には効かない**(既定の dock では効いて GPU が半分)。
残っている手: 見えていても**動いていない**面は描き直さない(Camera は作中カメラが動かなければ同じ絵)。`image_key` と同じ審判を面ごとに持つ。

### 4. 毎コマの往復と JSON

再生中は軽い status(5.9 KB / 1.62 ms、decode 0.08 ms)。**止まると全部入りの 140 KB / 7.1 ms + decode 1.3 ms** が走る。FFI の上を今も JSON が渡っている(encode / decode が両側)。
手: (a) 止まっている時の全部入りを 1 回にまとめるか差分にする (b) FFI の境目を JSON から平らな binary に替える(大仕事、効き 1〜2 ms/コマ) (c) `status` を「聞かれた物だけ」にする。

### 5. build がまだ 17 ms

1 の直し後で build 中央 17.39 ms(予算 16.7 ms を単独で超える)。`test/panel_layout_cost_test.dart` の予算超過が残っている:

| 何 | 予算 | 今 |
|---|---|---|
| 選択が変わった最初のコマ | 40 | 165(直す前 257) |
| 札 500 枚の棚を click | 100 | 199(直す前 208) |

手: 購読の範囲を葉に絞る(`DocumentSlice` の鍵付き購読は既にある。使っていない所を探す)、`const` の小 widget に切り出す(`inspector.dart` の `_transform` / `_world`、`ease_desk.dart` の 418 行の build、`browser.dart` の 252 行が残り)。

### 6. Rust 側の置き場所(棚)

`.claude/skills/wgsl-wgpu/SKILL.md` の指摘、どれも毎コマの道:
- `block_program.rs:519-534` 毎コマ・毎 round に `create_buffer` と `create_bind_group`。dynamic offset か ring にする。
- `block_program.rs:533` round ごとに状態 buffer を丸ごと copy(物の数に比例)。
- `block_program.rs:552` dispatch が `max_compute_workgroups_per_dimension`(65535)を見ていない → 419 万個で黙って切れる。
- `block_program.rs:433` 棚の検証が `Capabilities::all()`。device に無い機能の札が通って描く時に落ちる。

### 7. 組み直しの時間(開発の速さ)

`cargo build -p motolii-ui` が 7 分半。`[profile.dev] opt-level = 1`(Motolii 自身)、依存は全開。棚(`vism/*.wgsl`)は disk 読みなので組み直し不要(`docs/reviews/2026-09-17-build-placement.md`)。Rust を触った時だけ 7 分、が今の形。

## 今の test の基準(これを増やさない。直しの対象ではない)

- `flutter test`: 10 本落ちる(`panel_layout_cost` ×4 の予算超過、`ease_interaction`、`desk_workspace`、`visual_selection`、`module_contract`、`panel_placement`、棚の 1 本)。**着手前に自分で 1 回走らせて一覧を取る**(木は複数の未 commit の作業を含む)。
- `cargo test -p motolii-ui --lib`: 5 本落ちる(`every_example_builds_its_document`、`every_effect_on_the_shelf_has_a_snapshot` = 棚の札 34 枚に見本の絵が無い、`freeze_bakes_…`、`creating_particles_shows_the_particle_rows`、`the_cage_follows_the_drawn_text_…`)。今日の未 commit の棚の作業由来。
- lint: `cd motolii/ui/tool/motolii_lints && dart run bin/check.dart ../../lib` → `raw_dimension: clean` / `raw_color: 1`(`workspace_view.dart`)/ `material_import: 1`(同)。

## 読む順(引き継ぐ人へ)

1. `docs/reviews/2026-09-19-flutter-audit.md` の最後の 4 節(FFI の橋 / ガクつきの測定 / Material 離脱 / Stage の潰れ)。
2. `docs/skills/motolii-render-port/SKILL.md`(描く側の口)、`docs/skills/motolii-script/SKILL.md`(台本)。
3. `.claude/skills/flutter-skills/`(MIT、Riverpod と Material 前提なので規則だけ読む)、`.claude/skills/wgsl-wgpu/`。
4. `motolii/AGENTS.md`(家は 5 つ、足すなら消す、UX の合否は利用者)。
