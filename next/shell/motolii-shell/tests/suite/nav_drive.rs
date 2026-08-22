//! 運転席 — Timeline playhead ナビゲーション動詞束(U2、正典
//! `next/reference/timeline-grammar.md` §5・§8.1)の落ちるテスト先行(発注書
//! ORACLE の a〜f)。`drive.rs`/`timeline_key_gesture_drive.rs` と同じ形:
//! `Shell::update` だけを叩く。(e)/(f)だけは例外 — 「text 入力中に奪わない」
//! 「Cmd+文字を奪わない」はキー解決そのもの(`resolve_navigation_key`、
//! `Shell::update` を経由しない window イベント翻訳の層)の検分なので、
//! そちらを直接叩く。
//!
//! fixture(`Shell::new_fixture()`)の実測値(`fixture.rs`):
//! - comp 尺 1800 フレーム、playhead 既定 900(尺の半分)
//! - 既定選択「サビ歌詞」(`LayerId(7)`): position キー 510/570、clip
//!   [start=510, duration=270] → In=510/Out=780
//! - 「タイトルロゴ」(`LayerId(1)`): opacity キー 0/20/70/90、clip
//!   [start=0, duration=90] → In=0/Out=90
//! - marker 3個: Aメロ@150・サビ@510・ラスサビ@1200

use motolii_shell::timeline_pane::nav::{ClipEdge, JumpDirection};
use motolii_shell::{Message, Shell};
use motolii_store::LayerId;

fn shell_with_logo_selected() -> Shell {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Select(LayerId(1)));
    assert_eq!(shell.session().selection, Some(LayerId(1)), "タイトルロゴを選べていない");
    shell
}

// ---------------------------------------------------------------------------
// (a) Step Forward/Back 1(Shift で 10)フレーム + clamp
// ---------------------------------------------------------------------------

#[test]
fn a_step_playhead_moves_by_one_or_ten_frames() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.session().playhead, 900, "fixture の既定 playhead が想定と違う");

    let _ = shell.update(Message::StepPlayhead(1));
    assert_eq!(shell.session().playhead, 901, "素の Step Forward が1フレーム進んでいない");
    let _ = shell.update(Message::StepPlayhead(-1));
    assert_eq!(shell.session().playhead, 900, "素の Step Back が1フレーム戻っていない");

    let _ = shell.update(Message::StepPlayhead(10));
    assert_eq!(shell.session().playhead, 910, "Shift+Step Forward が10フレーム進んでいない");
    let _ = shell.update(Message::StepPlayhead(-10));
    assert_eq!(shell.session().playhead, 900, "Shift+Step Back が10フレーム戻っていない");
}

#[test]
fn a_step_playhead_clamps_at_the_composition_bounds() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::ScrubTo(0));
    let _ = shell.update(Message::StepPlayhead(-1));
    assert_eq!(shell.session().playhead, 0, "0フレーム未満へ行ってしまっている");

    let _ = shell.update(Message::ScrubTo(1799)); // DURATION_FRAMES(1800) - 1
    let _ = shell.update(Message::StepPlayhead(1));
    assert_eq!(shell.session().playhead, 1799, "尺の終端を超えてしまっている");
}

// ---------------------------------------------------------------------------
// (b) Go to Start/End(Home/End)
// ---------------------------------------------------------------------------

#[test]
fn b_home_and_end_jump_to_the_composition_bounds() {
    let mut shell = Shell::new_fixture().0;

    let _ = shell.update(Message::JumpPlayheadToEnd);
    assert_eq!(shell.session().playhead, 1799, "End が尺の最終フレームへ跳んでいない");

    let _ = shell.update(Message::JumpPlayheadToStart);
    assert_eq!(shell.session().playhead, 0, "Home が0秒へ跳んでいない");
}

// ---------------------------------------------------------------------------
// (c) JumpPrev/NextMeaningPoint: キーと marker を正しい順で渡る + Shift 限定
// ---------------------------------------------------------------------------

/// タイトルロゴ(opacity キー 0/20/70/90)選択のまま、playhead=0 から Next を
/// 連打すると自分のキー→marker(150/510/1200)の順で時系列どおりに渡る。
#[test]
fn c_next_meaning_point_visits_the_selected_layers_keys_then_markers_in_time_order() {
    let mut shell = shell_with_logo_selected();
    let _ = shell.update(Message::ScrubTo(0));

    for expected in [20, 70, 90, 150, 510, 1200] {
        let _ = shell.update(Message::JumpMeaningPoint {
            direction: JumpDirection::Next,
            layer_only: false,
        });
        assert_eq!(shell.session().playhead, expected, "Next の渡り順が正典どおりでない");
    }

    // これ以上先に意味点が無い — no-op(拒否ではなく単に動かない)。
    let _ = shell.update(Message::JumpMeaningPoint {
        direction: JumpDirection::Next,
        layer_only: false,
    });
    assert_eq!(shell.session().playhead, 1200, "先が無いのに動いてしまっている");
}

/// Prev も同じ渡り順を逆向きに辿る。
#[test]
fn c_prev_meaning_point_visits_markers_then_the_selected_layers_keys_backward() {
    let mut shell = shell_with_logo_selected();
    let _ = shell.update(Message::ScrubTo(1799));

    for expected in [1200, 510, 150, 90, 70, 20, 0] {
        let _ = shell.update(Message::JumpMeaningPoint {
            direction: JumpDirection::Prev,
            layer_only: false,
        });
        assert_eq!(shell.session().playhead, expected, "Prev の渡り順が正典どおりでない");
    }
}

/// Shift 付き(`layer_only`)は選択レイヤー自身のキーだけ — marker は comp
/// 単位なので対象から外れる(正典 §8.1「Shift 付きで選択レイヤー限定」)。
#[test]
fn c_shift_next_meaning_point_is_limited_to_the_selected_layers_own_keys() {
    let mut shell = shell_with_logo_selected();
    // タイトルロゴの自前キーは 0/20/70/90 だけ。100 より後ろに自前キーは無いが、
    // marker(150)は comp 側に存在する — layer_only がその違いを見分ける。
    let _ = shell.update(Message::ScrubTo(100));

    let _ = shell.update(Message::JumpMeaningPoint {
        direction: JumpDirection::Next,
        layer_only: true,
    });
    assert_eq!(
        shell.session().playhead, 100,
        "layer_only なのに comp の marker(150)へ跳んでしまっている"
    );

    let _ = shell.update(Message::JumpMeaningPoint {
        direction: JumpDirection::Next,
        layer_only: false,
    });
    assert_eq!(shell.session().playhead, 150, "marker込みなら次の意味点(150)へ跳ぶはず");
}

// ---------------------------------------------------------------------------
// (d) JumpToClipIn/Out
// ---------------------------------------------------------------------------

#[test]
fn d_jump_to_clip_in_and_out_uses_the_selected_layers_clip_bounds() {
    let mut shell = shell_with_logo_selected(); // start=0, duration=90 → In=0/Out=90
    let _ = shell.update(Message::ScrubTo(900));

    let _ = shell.update(Message::JumpClipEdge(ClipEdge::In));
    assert_eq!(shell.session().playhead, 0, "JumpToClipIn が clip の start へ跳んでいない");

    let _ = shell.update(Message::ScrubTo(900));
    let _ = shell.update(Message::JumpClipEdge(ClipEdge::Out));
    assert_eq!(shell.session().playhead, 90, "JumpToClipOut が clip の終端へ跳んでいない");
}

/// 既定選択(サビ歌詞、start=510/duration=270)でも同じ式で In/Out が出る —
/// 発明した特別扱いが無いことの確認。
#[test]
fn d_jump_to_clip_in_and_out_on_the_default_selection() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::JumpClipEdge(ClipEdge::In));
    assert_eq!(shell.session().playhead, 510);
    let _ = shell.update(Message::JumpClipEdge(ClipEdge::Out));
    assert_eq!(shell.session().playhead, 780);
}

/// 選択が無ければ何もしない(`nudge_keyframe` と同じ「選択が無ければ no-op」)。
#[test]
fn d_jump_to_clip_edge_is_a_no_op_without_a_selection() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::DeselectAllLayers);
    assert_eq!(shell.session().selection, None);
    let _ = shell.update(Message::ScrubTo(42));

    let _ = shell.update(Message::JumpClipEdge(ClipEdge::In));
    assert_eq!(shell.session().playhead, 42, "選択が無いのに playhead が動いてしまっている");
}

// ---------------------------------------------------------------------------
// (e) リネーム/text 入力中は横取りしない
// ---------------------------------------------------------------------------

/// `resolve_navigation_key` はキー解決そのもの(`Shell::update` を経由しない
/// window イベント翻訳の層)。`captured=true`(text_input 等がそのキーを既に
/// 消費した、`inspector_pointer_event` の `status == Captured` に相当)を渡すと、
/// 新設した5動詞のどれも一切 `Message` を出さない。
#[test]
fn e_navigation_keys_do_not_fire_while_a_text_field_has_already_captured_the_key() {
    use iced::keyboard::{key::Named, Key, Modifiers};

    // 第5波結線(B21+B18): J/K はシャトルへ移設されたが「text 入力中は
    // 横取りしない」対象であることは変わらない。移設先の `,`/`.` と新設の
    // L(シャトル順)/B/N(Set Work Area In/Out)も同じ裸キー群として並べる。
    let candidates: Vec<Key> = vec![
        Key::Named(Named::ArrowLeft),
        Key::Named(Named::ArrowRight),
        Key::Named(Named::Home),
        Key::Named(Named::End),
        Key::Character("j".into()),
        Key::Character("k".into()),
        Key::Character("l".into()),
        Key::Character("i".into()),
        Key::Character("o".into()),
        Key::Character(",".into()),
        Key::Character(".".into()),
        Key::Character("b".into()),
        Key::Character("n".into()),
    ];

    for key in &candidates {
        assert!(
            motolii_shell::resolve_navigation_key(key, Modifiers::default(), true).is_none(),
            "captured=true(text 編集中相当)なのに {key:?} が Message を出している"
        );
    }

    // 対照実験: captured=false なら同じキーが普通に発火する
    // (奪えないよう作りすぎて何も動かなくなっていないことの確認)。
    for key in &candidates {
        assert!(
            motolii_shell::resolve_navigation_key(key, Modifiers::default(), false).is_some(),
            "captured=false なのに {key:?} が発火しない"
        );
    }
}

// ---------------------------------------------------------------------------
// (f) Cmd+文字 が奪われない(j/k/i/o は裸キー)
// ---------------------------------------------------------------------------

#[test]
fn f_command_modified_bare_keys_are_not_stolen() {
    use iced::keyboard::{Key, Modifiers};

    // 第5波結線で裸キーが増えた(j/k/l = シャトル、,/. = 意味点ジャンプ移設先、
    // b/n = Set Work Area In/Out)。Cmd 付きを奪わない柵は全員に同じく掛かる —
    // ただし Cmd+L だけは意図した割当(ループトグル移設先)、Cmd+N は File 束
    // (New Project)なので、この「奪わない」検査の対象から除く。
    for ch in ["j", "k", "i", "o", ",", "."] {
        let key = Key::Character(ch.into());
        assert!(
            motolii_shell::resolve_navigation_key(&key, Modifiers::COMMAND, false).is_none(),
            "Cmd+{ch} を裸キーの動詞が奪ってしまっている(既存/将来のCmdショートカットと衝突する)"
        );
        assert!(
            motolii_shell::resolve_navigation_key(&key, Modifiers::default(), false).is_some(),
            "裸の {ch} が発火しない"
        );
    }
}

// ---------------------------------------------------------------------------
// (g) 第5波結線の keymap 移設(B21+B18、supervisor 裁定・拘束6の仮置き)
// ---------------------------------------------------------------------------

/// **移設の差分表そのもの**: J/K/L = シャトル(逆/停止/順)、旧 J/K(意味点
/// ジャンプ)は `,`/`.` へ、旧 L(ループトグル、正典 §5 既定)は Cmd+L へ。
/// B/N = Set Work Area In/Out、Shift+Home/End = 作業範囲の先頭/末尾へ。
#[test]
fn g_the_fifth_wave_keymap_reassignments_resolve_to_the_new_verbs() {
    use iced::keyboard::{key::Named, Key, Modifiers};
    use motolii_shell::timeline_pane::{Message as TlMessage, ShuttleCommand};

    let resolve = |key: &Key, modifiers: Modifiers| {
        motolii_shell::resolve_navigation_key(key, modifiers, false)
    };

    // J/K/L = シャトル。
    assert!(matches!(
        resolve(&Key::Character("j".into()), Modifiers::default()),
        Some(Message::Timeline(TlMessage::Shuttle(ShuttleCommand::Reverse)))
    ));
    assert!(matches!(
        resolve(&Key::Character("k".into()), Modifiers::default()),
        Some(Message::Timeline(TlMessage::Shuttle(ShuttleCommand::Stop)))
    ));
    assert!(matches!(
        resolve(&Key::Character("l".into()), Modifiers::default()),
        Some(Message::Timeline(TlMessage::Shuttle(ShuttleCommand::Forward)))
    ));
    // Shift+J/L(map 1056/1058 Fast)は「連打相当」— 同じ Message。
    assert!(matches!(
        resolve(&Key::Character("l".into()), Modifiers::SHIFT),
        Some(Message::Timeline(TlMessage::Shuttle(ShuttleCommand::Forward)))
    ));

    // 意味点ジャンプは `,`/`.`(Shift = layer_only。Shift 押下で `<`/`>` が
    // 届くレイアウトも同じ動詞)。
    assert!(matches!(
        resolve(&Key::Character(",".into()), Modifiers::default()),
        Some(Message::JumpMeaningPoint { direction: JumpDirection::Prev, layer_only: false })
    ));
    assert!(matches!(
        resolve(&Key::Character(".".into()), Modifiers::default()),
        Some(Message::JumpMeaningPoint { direction: JumpDirection::Next, layer_only: false })
    ));
    assert!(matches!(
        resolve(&Key::Character("<".into()), Modifiers::SHIFT),
        Some(Message::JumpMeaningPoint { direction: JumpDirection::Prev, layer_only: true })
    ));
    assert!(matches!(
        resolve(&Key::Character(">".into()), Modifiers::SHIFT),
        Some(Message::JumpMeaningPoint { direction: JumpDirection::Next, layer_only: true })
    ));

    // ループトグルは Cmd+L へ。
    assert!(matches!(
        resolve(&Key::Character("l".into()), Modifiers::COMMAND),
        Some(Message::Timeline(TlMessage::ToggleLoop))
    ));

    // B/N = Set Work Area In/Out(裸キー。Cmd+N は File 束のまま)。
    assert!(matches!(
        resolve(&Key::Character("b".into()), Modifiers::default()),
        Some(Message::Timeline(TlMessage::SetWorkAreaIn))
    ));
    assert!(matches!(
        resolve(&Key::Character("n".into()), Modifiers::default()),
        Some(Message::Timeline(TlMessage::SetWorkAreaOut))
    ));
    assert!(matches!(
        resolve(&Key::Character("n".into()), Modifiers::COMMAND),
        Some(Message::NewProjectRequested)
    ));

    // Shift+Home/End = 作業範囲の先頭/末尾へ(素の Home/End は comp 先頭/末尾
    // のまま)。
    assert!(matches!(
        resolve(&Key::Named(Named::Home), Modifiers::SHIFT),
        Some(Message::JumpToWorkAreaStart)
    ));
    assert!(matches!(
        resolve(&Key::Named(Named::End), Modifiers::SHIFT),
        Some(Message::JumpToWorkAreaEnd)
    ));
    assert!(matches!(
        resolve(&Key::Named(Named::Home), Modifiers::default()),
        Some(Message::JumpPlayheadToStart)
    ));
}

/// Alt+←/→ は既存の `NudgeKeyframe` の領分 — 新設の Step とここで衝突しては
/// いけない(`resolve_navigation_key` は Alt 付きの矢印を一切返さない)。
#[test]
fn f_alt_modified_arrows_are_left_to_the_existing_nudge_keyframe_binding() {
    use iced::keyboard::{key::Named, Key, Modifiers};

    let left = Key::Named(Named::ArrowLeft);
    let right = Key::Named(Named::ArrowRight);
    assert!(motolii_shell::resolve_navigation_key(&left, Modifiers::ALT, false).is_none());
    assert!(motolii_shell::resolve_navigation_key(&right, Modifiers::ALT, false).is_none());
}
