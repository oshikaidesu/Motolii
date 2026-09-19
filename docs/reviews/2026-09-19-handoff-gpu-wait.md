# 引き継ぎ: GPU の待ちを UI thread から外す(2026-09-19)

そのまま渡せる指示書。読む人は Motolii を知らない前提で書く。**作業は途中まで木にある**(未 commit)。

## 何をする仕事か

Motolii = Rust(描画)+ Flutter(窓)の映像編集ソフト。`/Users/member_ottoto/rust_ae/Motolii`。
再生と操作のたびに、Dart が FFI で Rust の描画を呼び、**Rust が GPU の描き終わりを待って**から返る。この待ちが Flutter の UI/platform thread の上にあるので、Timeline が塗る番を失ってガクつく。
**やる事**: 描画を「投げたら返る」形にして、描き終わりは callback で受ける。UI thread を塞がない。

## 測ってある事実(推測ではない)

12 秒の再生、468 回の描画、debug build、単位 ms:

| 段 | 中央 | p90 | 最大 |
|---|---|---|---|
| `_renderNow` 全体 | 10.16 | 49.31 | 59.23 |
| render Camera(GPU) | 4.36 | 26.97 | 35.78 |
| render Stage(GPU) | 3.53 | 17.94 | 22.12 |
| status(FFI、軽い 5.9 KB) | 1.62 | 3.01 | 4.67 |
| jsonDecode | 0.08 | 0.14 | 4.04 |

- UI thread が `_renderNow` の中に居た時間 = 12 秒中 **72 %**。
- Flutter の 1 コマ: build 中央 42.8 ms、raster 中央 1.3 ms(= **build 重、painter は無実**)。
- 既に入れた直し(同じ document コマなら描かない、41 % が該当)で build 中央 17.4 ms、33 ms 超のコマ 108 → 72。**残りは 1 回の描画が中央 12 ms・p90 60 ms**。予算は 16.7 ms。

**目標**: UI thread の占有を 20 % 未満、33 ms 超のコマをほぼ 0。

## 今の木の状態(途中まで入っている、未 commit)

- `motolii/ui/native/src/lib.rs` — `motolii_probe_render_async(ctx, surface_id, view, skippable)` を新設(同期の `motolii_probe_render` は残す)。`RenderWait { Sync, Async, AsyncSkip }`。専用 thread `motolii-gpu-wait` が `Queue::on_submitted_work_done` を拾って、既存の C callback `motolii_probe_set_frame_ready(ctx, cb, user)` を鳴らす骨組み。
- `motolii/crates/motolii-render/src/compositor/selection_bounds.rs` / `presentable.rs` / `engine.rs` — 選択の枠の読み戻しに `wait: bool`。偽なら `PollType::Poll` で待たない。
- `motolii/ui/lib/bridge/native_frames.dart` — `renderAsync(surfaceId, view, {skippable})` の binding。
- `motolii/ui/lib/session/editor_session.dart` / `motolii/ui/macos/Runner/MainFlutterWindow.swift` — surface の番人と skip の数え上げ(`_pendingWork`、`_gpuSkips`)。
- **build は通る**。**動かして確かめていない**。Dart 側が非同期の口を使い切っているかは未確認。

## 仕上げる事

1. Dart の毎コマの道(`editor_session.dart` の `_renderNow`、再生の Ticker、`needsRender`)が `renderAsync` を使い、**pixels を待たない**。窓と roi はこれまで通り同じコマに決める(下の約束)。
2. 面ごとに surface は 2 枚(`_flip`)。**まだ描き終わりの signal が来ていない surface に描かない**番人。飛ばした回数を数えて log に出す。
3. Swift は `frame_ready` の callback を受けて `registry.textureFrameAvailable` を呼ぶ(既にその形)。callback が別 thread で来るなら main へ跳ばす。
4. `RenderWait::Sync` の道は残す(書き出し `export_range_with_progress` と `motolii/crates/motolii-render/examples/zz_watch` が必要とする)。壊れていない事を確かめる。

## 壊してはいけない約束(これを壊したら直しではない)

1. **`PROBE room=stage-window verdict=matched lag=0`** が出続ける。窓を伸縮した時と Fit を押した時、頼んだ窓(roi)の絵が同じコマに出る。`lag=0` 以外が 1 本でも出たら退行。roi は submit の前に決まるので、非同期にしても保てるはず。
2. 絵が破れない・古い絵が出ない。**必ず起動して再生とスクラブを目で見る**。
3. 掴む契約: drag → preview → commit。Esc と focus 外れで取り消し、1 drag = undo 1 回。
4. 見た目の値は変えない。色と寸法は `EditorMetrics` / `EditorTheme` の token のみ。
5. 書き出しと `zz_watch` が動く。

## 手順とコマンド

```bash
cd /Users/member_ottoto/rust_ae/Motolii
# Rust を触ったら(40 秒前後。fork まで触ると 7 分)
scripts/motolii-ui.sh native
# 窓を出す(debug、hot reload あり)
scripts/motolii-ui.sh dev
# 本当の速さ(release + AOT、hot reload 無し)
scripts/motolii-ui.sh profile
# 関門
cd motolii/ui && ../../.tools/flutter/bin/flutter analyze && ../../.tools/flutter/bin/flutter test
cd motolii/ui/tool/motolii_lints && dart run bin/check.dart ../../lib
cd /Users/member_ottoto/rust_ae/Motolii && cargo test -p motolii-ui --lib && cargo test -p motolii-render --lib
```

Flutter は `.tools/flutter`(3.47.2)。cargo は repo の根から。

## 今すでに落ちている test(これを直す仕事ではない。増やさない事だけ見る)

- `flutter test`: 10 本(`panel_layout_cost` ×5 の予算超過、`ease_interaction`、`desk_workspace`、`visual_selection`、`module_contract`、`panel_placement`)。**着手前に自分で 1 回走らせて一覧を取り、後で比べる**。
- `cargo test -p motolii-ui --lib`: 5 本(`every_example_builds_its_document`、`every_effect_on_the_shelf_has_a_snapshot`、`freeze_bakes_…`、`creating_particles_shows_the_particle_rows`、`the_cage_follows_the_drawn_text_…`)。別の未 commit の作業由来。
- lint: `raw_dimension: clean` / `raw_color: 1` / `material_import: 1`(どちらも `lib/workspace/workspace_view.dart`、触らない)。

## 測り方(前後で同じに)

計器は `kDebugMode` の裏。1 コマごとに `_renderNow` の各段の ms と、`SchedulerBinding.instance.addTimingsCallback` の build / raster / total。同じ書類・同じ 12 秒で、**中央値と p90 と最大**(平均は使わない)、16.7 ms 超と 33 ms 超の本数、UI thread の占有率。
前の計測の生 log は `/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/94a5313f-c882-4ff0-b8a0-866957579b6f/scratchpad/gpu-wait/`(`before*.log` / `after*.log`)。

## 読む順

1. `docs/reviews/2026-09-19-flutter-audit.md` の最後の 4 節(FFI の橋 / ガクつきの測定 / Material 離脱 / Stage の潰れ)。
2. `docs/reviews/2026-09-19-perf-targets.md`(的の全体。この仕事は「的 1」)。
3. `docs/skills/motolii-render-port/SKILL.md`(描く側の口の作法)。
4. `motolii/AGENTS.md`(家は 5 つ、足すなら消す、UX の合否は利用者)。

## 報告に必ず入れる物

- 前後の表(上の形式)。
- 変えた file を `file:line` で。
- `flutter analyze` / `flutter test` / lint / `cargo test` の行をそのまま。落ちる本数が増えていない事。
- `PROBE room=stage-window` の行(`lag=0` 以外が 0 本である事)。
- 起動して再生・スクラブ・窓の伸縮を**目で見た**結果。
- できなかった事と理由(推測で埋めない)。
- commit はしない。
