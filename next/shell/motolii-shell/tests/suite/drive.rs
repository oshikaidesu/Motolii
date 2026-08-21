//! 運転席 — 窓を開けずに shell を動かす。
//!
//! 見るのは背骨1(書き口が1箇所)と M13(拒否が必ず出る)と、
//! **描画キャッシュが `revision()` で正しく落ちること**。

use motolii_shell::timeline_pane::{self, BarPart, Hit, RowProjection};
use motolii_shell::tokens::{Colors, Dimensions};
use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

#[test]
fn adding_a_layer_shows_up_and_undo_takes_it_back() {
    let mut shell = shell();
    assert_eq!(shell.layer_count(), 0);

    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1, "layer を足しても増えない");

    // **1操作 = 1 Undo**。`AddLayer` は内部では AddLayer + SetMeta の2 intent だが、
    // 利用者から見れば1操作なので Undo 1回で消えなければならない(M10)。
    shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        0,
        "Undo 1回で消えない = 1操作が複数 undo になっている"
    );
    assert!(!shell.can_undo(), "底より前へ戻れてはいけない");
}

#[test]
fn rejection_reaches_the_status_band() {
    let mut shell = shell();
    shell.update(Message::Undo);
    assert_eq!(
        shell.status(),
        Some("これ以上戻せない"),
        "戻せない時に何も言わないのは M13 違反(無反応ゼロ)"
    );

    // 次の操作で理由が消えること(古い理由が居座らない)。
    shell.update(Message::AddLayer);
    assert_eq!(shell.status(), None);
}

#[test]
fn frame_cache_follows_revision_and_playhead() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let first = shell.frame_token().expect("frame");

    // 同じ入力なら描き直さない。
    shell.update(Message::Select(motolii_store::LayerId(1)));
    assert_eq!(
        shell.frame_token(),
        Some(first.clone()),
        "選択だけで描き直している"
    );

    // 再生位置が動いたら描き直す。
    shell.update(Message::ScrubTo(10));
    assert_ne!(
        shell.frame_token(),
        Some(first.clone()),
        "scrub で描き直していない"
    );

    // undo で Document が戻ったら描き直す(store 世代は変わらないので
    // `revision()` が edit 位置も見ていないとここが落ちる)。
    let scrubbed = shell.frame_token().expect("frame");
    shell.update(Message::Undo);
    assert_ne!(
        shell.frame_token(),
        Some(scrubbed),
        "undo で描き直していない"
    );
}

/// **核の一周の前半** — 落とす → 素材が立つ → Stage に絵が出る。
#[test]
fn dropping_a_video_puts_a_layer_on_the_stage() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
    use std::process::Command;

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("shell-drop");
    let video = dir.join("drop.mp4");
    let out = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=orange:s=128x128:d=1:r=30",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
        ])
        .arg(&video)
        .output()
        .expect("ffmpeg");
    assert!(out.status.success());

    let mut shell = shell();
    let before = shell.frame_token();

    shell.update(Message::AdmitPaths(vec![video]));

    assert_eq!(shell.layer_count(), 1, "落とした素材が layer にならない");
    assert_eq!(shell.status(), None, "受理できたのに拒否理由が出ている");
    assert_ne!(shell.frame_token(), before, "Stage が描き直されていない");

    // 1操作 = 1 undo
    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "落とした分が Undo 1回で消えない");
}

/// **開けない物は理由つきで飛ばす**(M2)。黙って消さない。
#[test]
fn unopenable_files_are_rejected_with_a_reason() {
    use motolii_testkit::tmp_dir;

    let dir = tmp_dir("shell-reject");
    let junk = dir.join("not-a-video.txt");
    std::fs::write(&junk, b"hello").unwrap();

    let mut shell = shell();
    shell.update(Message::AdmitPaths(vec![junk]));

    assert_eq!(shell.layer_count(), 0, "開けない物を layer にしてしまった");
    let status = shell.status().expect("拒否理由が出ていない = M2 違反");
    assert!(
        status.contains("not-a-video.txt"),
        "どのファイルが駄目だったか分からない: {status}"
    );
}

/// **3本まとめて落として1操作**。winit は1ファイル1事象で送ってくるので、
/// そのまま処理すると 3 undo になる(旧 workspace が `drive_drop.rs` で守っていた性質)。
#[test]
fn three_drops_become_one_operation() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
    use std::process::Command;

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("shell-batch");
    let mut paths = Vec::new();
    for i in 0..3 {
        let path = dir.join(format!("clip{i}.mp4"));
        let out = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=green:s=64x64:d=1:r=30",
                "-pix_fmt",
                "yuv420p",
                "-c:v",
                "libx264",
            ])
            .arg(&path)
            .output()
            .expect("ffmpeg");
        assert!(out.status.success());
        paths.push(path);
    }

    let mut shell = shell();
    for path in paths {
        shell.update(Message::DropReceived(path));
    }
    assert_eq!(shell.layer_count(), 0, "区切りが来る前に取り込んでいる");

    shell.update(Message::FlushDrops);
    assert_eq!(shell.layer_count(), 3, "3本とも入っていない");

    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "3本の取り込みが Undo 1回で消えない");
}

// ---------------------------------------------------------------------------
// Timeline pane(第1波)
// ---------------------------------------------------------------------------

/// **層3枚の行が立つ / 選択が行と Session で一致する**。
#[test]
fn timeline_rows_reflect_layer_count_and_selection() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    shell.update(Message::AddLayer);
    shell.update(Message::AddLayer);

    let rows = shell.timeline_rows();
    assert_eq!(rows.len(), 3, "層3枚の行が立たない");
    // `AddLayer` は置いた layer を選択する(lib.rs)ので、この時点で最後に足した
    // 行だけが selected のはず — Timeline の投影が同じ Session を読めているかの確認。
    let selected_after_add: Vec<_> = rows
        .iter()
        .filter(|row| row.selected)
        .map(|row| row.id)
        .collect();
    assert_eq!(
        selected_after_add,
        vec![rows[2].id],
        "AddLayer 直後の選択が Session と一致しない"
    );

    let target = rows[0].id;
    shell.update(Message::Select(target));

    let rows = shell.timeline_rows();
    let selected: Vec<_> = rows
        .iter()
        .filter(|row| row.selected)
        .map(|row| row.id)
        .collect();
    assert_eq!(
        selected,
        vec![target],
        "選択が Timeline の行と Session で一致しない(Stage も同じ Session を読む)"
    );
}

/// hidden な層はグレー表現の元になる `hidden` フラグを行が持つ。present(削除)とは
/// 別物なので、hidden でも present な限り行は立ち続ける。
#[test]
fn timeline_rows_carry_hidden_flag_without_dropping_the_row() {
    let rows = vec![RowProjection {
        id: motolii_store::LayerId(1),
        name: String::new(),
        hidden: true,
        solo: false,
        locked: false,
        start: 0,
        duration: 10,
        selected: false,
        dragging: false,
    }];
    assert!(rows[0].hidden, "hidden 属性が投影へ届いていない");
}

/// canvas の click/drag が送る frame 番号を決める**純粋な変換**。Message::ScrubTo が
/// 実際に playhead を動かし Stage を描き直すことは
/// `frame_cache_follows_revision_and_playhead` が既に見ている。
#[test]
fn timeline_scrub_math_maps_pixel_to_frame() {
    assert_eq!(timeline_pane::frame_at_x(0.0, 300.0, 300), 0);
    assert_eq!(timeline_pane::frame_at_x(150.0, 300.0, 300), 150);
    assert_eq!(timeline_pane::frame_at_x(299.0, 300.0, 300), 299);
    // canvas の外まで drag しても端で止まる。
    assert_eq!(timeline_pane::frame_at_x(-50.0, 300.0, 300), 0);
    assert_eq!(timeline_pane::frame_at_x(1000.0, 300.0, 300), 299);
    // 空 comp では 0 に落ちる(M16: panic しない)。
    assert_eq!(timeline_pane::frame_at_x(10.0, 300.0, 0), 0);
}

/// ルーラー帯は常に scrub、bar の区間だけが選択、同じ行のそれ以外は scrub。
#[test]
fn timeline_hit_test_distinguishes_bar_ruler_and_blank_row() {
    let rows = vec![RowProjection {
        id: motolii_store::LayerId(1),
        name: "clip".to_owned(),
        hidden: false,
        solo: false,
        locked: false,
        start: 100,
        duration: 50,
        selected: false,
        dragging: false,
    }];
    let width = 300.0;
    let duration_frames = 300;
    let ruler_height = 20.0;
    let row_height = 20.0;

    assert_eq!(
        timeline_pane::hit_test(
            iced::Point::new(120.0, 5.0),
            &rows,
            ruler_height,
            row_height,
            width,
            duration_frames,
            0.0,
            0,
            None,
        ),
        Hit::Blank,
        "ルーラー帯は click/drag で常に scrub のはず"
    );

    // frame 100..150 は 1:1 縮尺(width == duration_frames)なので x も 100..150。
    assert_eq!(
        timeline_pane::hit_test(
            iced::Point::new(120.0, 25.0),
            &rows,
            ruler_height,
            row_height,
            width,
            duration_frames,
            0.0,
            0,
            None,
        ),
        Hit::Bar(motolii_store::LayerId(1)),
        "bar の区間内が選択に化けない"
    );

    assert_eq!(
        timeline_pane::hit_test(
            iced::Point::new(10.0, 25.0),
            &rows,
            ruler_height,
            row_height,
            width,
            duration_frames,
            0.0,
            0,
            None,
        ),
        Hit::Blank,
        "同じ行でも bar の外は行の空白部(scrub)のはず"
    );
}

// ---------------------------------------------------------------------------
// Timeline レーンバー(裁定147・第2波T1)
// ---------------------------------------------------------------------------

/// **オラクル(a)**: M glyph クリックで Document の hidden(mute)が反転し、
/// Undo 1回で戻る。
#[test]
fn lane_bar_mute_toggle_flips_hidden_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    assert!(!shell.timeline_rows()[0].hidden, "初期状態が hidden であってはいけない");

    shell.update(Message::Timeline(timeline_pane::Message::ToggleMute(id)));
    assert!(
        shell.timeline_rows()[0].hidden,
        "M glyph クリックで hidden が反転しない"
    );

    shell.update(Message::Undo);
    assert!(
        !shell.timeline_rows()[0].hidden,
        "Undo 1回で hidden が元に戻らない(1操作=1undo、M10違反)"
    );
}

/// **オラクル(b)**: locked な行への解除以外の書き込み(M/S)は理由つきで拒否
/// される(M13)。`locked` 自身の解除/再ロックだけは locked な行でも常に通る
/// (`motolii_store::attrs::LayerAttrs::locked` の規則)。
#[test]
fn lane_bar_lock_blocks_other_writes_with_a_reason_but_unlock_always_wins() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;

    shell.update(Message::Timeline(timeline_pane::Message::ToggleLock(id)));
    assert!(shell.timeline_rows()[0].locked, "L glyph クリックで locked にならない");
    assert_eq!(
        shell.status(),
        None,
        "locked 自身の設定は常に通るはずなのに拒否理由が出ている"
    );

    shell.update(Message::Timeline(timeline_pane::Message::ToggleMute(id)));
    assert!(
        !shell.timeline_rows()[0].hidden,
        "locked な行への M(hidden)書き込みが通ってしまっている"
    );
    assert!(
        shell.status().is_some(),
        "locked な行への拒否が status 帯に出ていない(M13: 無反応ゼロ違反)"
    );

    shell.update(Message::Timeline(timeline_pane::Message::ToggleSolo(id)));
    assert!(
        !shell.timeline_rows()[0].solo,
        "locked な行への S(solo)書き込みが通ってしまっている"
    );
    assert!(
        shell.status().is_some(),
        "locked な行への S 拒否が status 帯に出ていない(M13)"
    );

    // 解除(locked 自身の書き込み)だけは常に通る — 二度と触れなくなる詰みを作らない。
    shell.update(Message::Timeline(timeline_pane::Message::ToggleLock(id)));
    assert!(
        !shell.timeline_rows()[0].locked,
        "locked の解除が locked な行自身の操作なのに拒否されている"
    );
}

/// **オラクル(c、構造テスト)**: クリップ面(`motolii-timeline-pane/src/canvas.rs`)
/// がレイヤー名(`row.name`)を一切参照しないこと(裁定147「クリップ面にレイヤー名を
/// 描かない」)。canvas は headless では rasterize できない(モジュール doc 参照)
/// ので、実際に画素を読む代わりにソースを読む — 修正前はここが `row.name` を
/// 含んでいたので赤から始まる。裁定160 切片7で `timeline/` は
/// `motolii-timeline-pane` crate へ抽出済み — パスは `motolii-shell` の
/// `CARGO_MANIFEST_DIR` から兄弟 crate(`next/ui/motolii-timeline-pane/`)へ
/// 相対で辿る。
#[test]
fn clip_face_source_no_longer_references_the_layer_name() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ui/motolii-timeline-pane/src/canvas.rs");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("motolii-timeline-pane/src/canvas.rs を読めない: {error}"));
    assert!(
        !source.contains("row.name"),
        "クリップ面(canvas.rs)がまだレイヤー名(row.name)を参照している — \
         裁定147: 名前の住所はレーンバー(lane_bar.rs)へ一本化したはず"
    );
}

// ---------------------------------------------------------------------------
// Timeline クリップの move/trim(第2波T2、正典 `next/reference/
// timeline-grammar.md` §2)
// ---------------------------------------------------------------------------
//
// 全試験共通のセットアップ: `Message::ScrubTo(50)` してから `Message::AddLayer`
// すると、Solid layer は `LayerTiming::place(playhead, None, comp_duration)`
// (`Shell::update` の `AddLayer` 腕)で `[50, 300)`(comp は 300frame、
// `Shell::new` 参照)に置かれる — comp 終端まで埋まる Solid の性質を逆手に取り、
// 左に 50frame ぶんの余地だけを持つ clip を作る(AddLayer 経由で任意の尺を
// 指定する公開 API は無い — メディア admit は ffmpeg 依存なのでここでは使わない)。

/// **オラクル(a)**: 本体ドラッグ → release で start が動き、Undo 1回で戻る。
#[test]
fn dragging_a_clip_body_moves_it_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    assert_eq!(shell.timeline_rows()[0].start, 50);

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 30,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));

    assert_eq!(shell.timeline_rows()[0].start, 30, "本体ドラッグで start が動いていない");
    assert_eq!(shell.status(), None, "動いただけなのに拒否理由が出ている");

    shell.update(Message::Undo);
    assert_eq!(
        shell.timeline_rows()[0].start,
        50,
        "Undo 1回で戻らない(1操作=1undo、M10違反)"
    );
}

/// **オラクル(b)**: ドラッグ中の Esc → 履歴完全無傷(can_undo/can_redo とも
/// 開始前と同一)。preview は捨てられ、bar は元の位置のまま。
#[test]
fn escape_during_a_clip_drag_leaves_history_completely_untouched() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;

    let can_undo_before = shell.can_undo();
    let can_redo_before = shell.can_redo();

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 10,
        px_per_frame: 1.0,
    }));
    shell.update(Message::EscapePressed);

    assert_eq!(
        shell.can_undo(),
        can_undo_before,
        "Esc がドラッグ以外の履歴まで動かしている"
    );
    assert_eq!(shell.can_redo(), can_redo_before, "Esc が redo 空間を汚している");
    assert_eq!(
        shell.timeline_rows()[0].start,
        50,
        "Esc なのに bar が動いてしまっている"
    );

    // 掴んだままのボタンで release が来ても、Esc で drag state は既に空 —
    // 何も起きない(二重確定の防止)。
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));
    assert_eq!(shell.timeline_rows()[0].start, 50);
    assert_eq!(shell.can_undo(), can_undo_before);
}

/// 右クリックも Esc と同じ意味(裁定151「キャンセルの一般化」)。
#[test]
fn right_click_during_a_clip_drag_cancels_it_just_like_escape() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    let can_undo_before = shell.can_undo();

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 10,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragCancelled));

    assert_eq!(shell.can_undo(), can_undo_before);
    assert_eq!(shell.timeline_rows()[0].start, 50);
}

/// 掴んだだけで未移動なら release は no-op(正典 §2)。
#[test]
fn grabbing_without_moving_and_releasing_writes_nothing() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    let start_before = shell.timeline_rows()[0].start;
    let can_undo_before = shell.can_undo();

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: start_before,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));

    assert_eq!(shell.timeline_rows()[0].start, start_before);
    assert_eq!(
        shell.can_undo(),
        can_undo_before,
        "動いていないのに undo できる操作が積まれている"
    );
}

/// **オラクル(c)**: trim In は start のみ動かし、end(`start + duration`)は
/// 固定のまま。
#[test]
fn trimming_the_in_edge_moves_only_the_start_and_keeps_the_end_fixed() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    let original_end = shell.timeline_rows()[0].start + shell.timeline_rows()[0].duration;

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::EdgeIn,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 80,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));

    let row = &shell.timeline_rows()[0];
    assert_eq!(row.start, 80, "trim In で start が動いていない");
    assert_eq!(
        row.start + row.duration,
        original_end,
        "trim In で end が動いてしまっている(固定のはず)"
    );

    shell.update(Message::Undo);
    assert_eq!(shell.timeline_rows()[0].start, 50, "Undo 1回で戻らない");
}

/// **オラクル(c)**: trim Out は end のみ動かし、start は固定のまま。
#[test]
fn trimming_the_out_edge_moves_only_the_end_and_keeps_the_start_fixed() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    let original_start = shell.timeline_rows()[0].start;

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::EdgeOut,
        at_frame: 299,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 250,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));

    let row = &shell.timeline_rows()[0];
    assert_eq!(
        row.start, original_start,
        "trim Out で start が動いてしまっている(固定のはず)"
    );
    assert_eq!(row.start + row.duration, 250, "trim Out で end が動いていない");
}

/// **オラクル(d)**: playhead 近傍(画面距離 `SNAP_PX` 相当)で吸着し、ドラッグ中の
/// Cmd 押下でスナップが一時的に無効化される(正典 §2・裁定151)。
#[test]
fn dragging_near_the_playhead_snaps_to_it_and_command_disables_snapping() {
    let mut shell = shell();
    shell.update(Message::ScrubTo(50));
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    // clip 作成後に playhead を動かす — 0秒/終端とは別の、はっきりした候補にする。
    shell.update(Message::ScrubTo(20));

    // px_per_frame == 1.0(1px=1frame)。raw target は 50+(25-50)=25、playhead(20)
    // との画面距離は5px(<= SNAP_PX=7px)なので吸着するはず。
    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 25,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));
    assert_eq!(
        shell.timeline_rows()[0].start,
        20,
        "playhead 近傍で吸着していない"
    );

    shell.update(Message::Undo); // 50 へ戻す

    // 同じ操作でも、ドラッグ中に Cmd を押していればスナップは一時的に無効
    // (正典 §2・裁定151)。
    shell.update(Message::KeyboardModifiersChanged(
        iced::keyboard::Modifiers::COMMAND,
    ));
    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: 50,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: 25,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));
    assert_eq!(
        shell.timeline_rows()[0].start,
        25,
        "Cmd 押下中なのにスナップが素通りしていない"
    );
}

/// **オラクル(e)**: ロック行は掴む前に拒否され、status 帯に理由が出る(M13)。
/// 拒否されているので move/release を送っても何も書かれない。
#[test]
fn grabbing_a_locked_clip_is_refused_with_a_reason() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    shell.update(Message::Timeline(timeline_pane::Message::ToggleLock(id)));
    let start_before = shell.timeline_rows()[0].start;

    shell.update(Message::Timeline(timeline_pane::Message::BarGrabbed{
        layer: id,
        part: BarPart::Body,
        at_frame: start_before,
    }));
    assert!(
        shell.status().is_some(),
        "ロック中の掴みが理由つきで拒否されていない(M13: 無反応ゼロ違反)"
    );

    shell.update(Message::Timeline(timeline_pane::Message::DragMoved{
        at_frame: start_before + 10,
        px_per_frame: 1.0,
    }));
    shell.update(Message::Timeline(timeline_pane::Message::DragReleased));
    assert_eq!(
        shell.timeline_rows()[0].start,
        start_before,
        "ロック中なのに動いてしまっている"
    );
}

// ---------------------------------------------------------------------------
// デザイン値の外出し(裁定117)
// ---------------------------------------------------------------------------

/// **トークンファイル変更で(debug)寸法が変わる**。debug watch が呼ぶのと同じ
/// `Dimensions::load_from_path` を直接叩く — 実ファイル監視(notify Subscription)は
/// 非同期の runtime を要るため、ここでは再読込ロジックそのものを headless に縛る。
#[test]
fn dimension_tokens_reload_when_the_file_changes() {
    let dir = motolii_testkit::tmp_dir("shell-tokens-dims");
    let path = dir.join("dimensions.json");

    let full = |row_height: i32| {
        format!(
            r#"{{"row_height": {row_height}, "transport_band": 32, "title_text": 12,
                "body_text": 11, "caption_text": 9, "micro_text": 8,
                "spacing_xs": 2, "spacing_s": 4, "spacing_m": 8, "spacing_l": 12,
                "border_width": 1.0, "panel_header_height": 29,
                "inspector_panel_width": 496, "inspector_row_height": 20,
                "inspector_section_header_height": 26, "inspector_value_width": 38,
                "inspector_glyph_width": 18, "ui_scale": 1.0}}"#
        )
    };

    std::fs::write(&path, full(24)).unwrap();
    let dims = Dimensions::load_from_path(&path).expect("dimensions.json を読めない");
    assert_eq!(dims.row_height, 24.0, "変更後の値を読めていない");

    std::fs::write(&path, full(18)).unwrap();
    let dims = Dimensions::load_from_path(&path).expect("dimensions.json を読めない");
    assert_eq!(
        dims.row_height, 18.0,
        "トークンファイルを書き換えても寸法が変わらない"
    );
}

/// 読めない・壊れている時は既定値へ落ちる(M16: 落ちても画面を空にしない)。
#[test]
fn dimension_tokens_reject_unreadable_paths() {
    let missing = std::path::Path::new("/nonexistent/motolii-dimensions-test/dimensions.json");
    assert!(
        Dimensions::load_from_path(missing).is_err(),
        "無い path を読めてしまっている(Shell 側は `unwrap_or_default` で拾う)"
    );
}

/// 色は `ui/motolii-tokens/sources/motolii-dark.json` を**そのまま**読む — 複製ではない。
#[test]
fn color_tokens_load_from_the_single_source_of_truth() {
    let colors = Colors::load_from_path(&Colors::debug_source_path())
        .expect("ui/motolii-tokens/sources/motolii-dark.json を読めない");
    assert!(
        (colors.text_primary.r - 0.7216).abs() < 0.001,
        "text.primary が正本 JSON(候補B)の値と一致しない: {:?}",
        colors.text_primary
    );
}

/// **正本ファイルが実際に語彙拡張の全フィールドを持つ**(json の `_note_*` は
/// serde が無視するだけで、値そのものは全キー要求どおり読めること)。
#[test]
fn dimensions_json_source_of_truth_has_the_full_vocabulary() {
    let dims = Dimensions::load_from_path(&Dimensions::debug_source_path())
        .expect("tokens/dimensions.json を読めない");
    assert_eq!(dims.row_height, 20.0);
    assert_eq!(dims.transport_band, 30.0);
    assert!(
        dims.title_text > dims.body_text,
        "title は body より大きいはず"
    );
    assert!(
        dims.body_text > dims.caption_text && dims.caption_text > dims.micro_text,
        "type scale が title>body>caption>micro の順になっていない: {dims:?}"
    );
    assert!(
        dims.spacing_xs < dims.spacing_s
            && dims.spacing_s < dims.spacing_m
            && dims.spacing_m < dims.spacing_l,
        "spacing scale が xs<s<m<l の順になっていない: {dims:?}"
    );
    assert!(dims.border_width > 0.0, "罫線幅が0以下");
    assert!(
        dims.panel_header_height > dims.row_height,
        "panel header が行高より低い"
    );
    assert!(
        dims.timeline_lane_bar_width > dims.inspector_glyph_width * 3.0,
        "レーンバー幅が M/S/L glyph 3個ぶんより狭い(收まらない): {dims:?}"
    );
}

/// 状態: 選択・無効は正本 JSON に無いロールなので**近縁色から導出**している
/// (発注書の指示)。導出結果が hover / muted と別の色になっていることを見る
/// — 「導出したつもりが既存ロールのコピーだった」事故を防ぐ。
#[test]
fn derived_state_colors_are_distinct_from_their_neighbors() {
    let colors = Colors::default();
    assert_ne!(
        colors.state_selected, colors.surface_hover,
        "選択の色が hover と同じ — 状態を区別できない"
    );
    assert_ne!(
        colors.state_disabled, colors.text_muted,
        "無効の色が muted と同じ — 状態を区別できない"
    );
    // 導出式(`derive_state_colors`)が Default と `parse()` の両方で同じ式を通る
    // こと(2箇所で別の式にならない事故を防ぐ)。`Default` の元値は JSON の
    // スナップショット(4桁丸め)であって同一 f64 ではないので、既存の
    // `color_tokens_load_from_the_single_source_of_truth` と同じ許容誤差で比べる。
    let parsed = Colors::load_from_path(&Colors::debug_source_path())
        .expect("ui/motolii-tokens/sources/motolii-dark.json を読めない");
    let close = |a: f32, b: f32| (a - b).abs() < 0.001;
    assert!(
        close(colors.state_selected.r, parsed.state_selected.r)
            && close(colors.state_selected.g, parsed.state_selected.g)
            && close(colors.state_selected.b, parsed.state_selected.b),
        "Default と正本 JSON 読み込みで state_selected が食い違う: {:?} vs {:?}",
        colors.state_selected,
        parsed.state_selected
    );
    assert!(
        close(colors.state_disabled.r, parsed.state_disabled.r)
            && close(colors.state_disabled.g, parsed.state_disabled.g)
            && close(colors.state_disabled.b, parsed.state_disabled.b),
        "Default と正本 JSON 読み込みで state_disabled が食い違う: {:?} vs {:?}",
        colors.state_disabled,
        parsed.state_disabled
    );
}
