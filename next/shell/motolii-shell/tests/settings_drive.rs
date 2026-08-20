//! Settings パネル(タスク#18)の `Message` 経由の柵。
//!
//! - 背景プリセット/チャンネル編集 → **1回の Undo で戻る**(read-modify-write の
//!   `Intent::SetComposition` が1操作=1 intent であること)。
//! - 市松トグルは Handle 表示だけを変える — screenshot/export が読む
//!   `frame_rgba()`(生 rgba)には**一切乗らない**(`settings_pane` モジュール
//!   doc「合成器が出せる」と「書き出しが吐く」は別問題)。
//! - パネルの開閉は表示だけの分岐 — Document にも undo 履歴にも乗らない。
//!
//! `ui_scale` の書き戻し(`tokens::write_ui_scale_to_path`)は `tests/
//! ui_scale_fence.rs` 側で隔離した一時ファイルを使って検分する。ここでは
//! `tokens/dimensions.json`(複数 worktree・並列試験間で共有される delicate な
//! ファイル、`../reference/KNOWN.md`)へ実際に触る `Shell::update` 経路は
//! 意図的に叩かない。

use motolii_shell::settings_pane::{BackgroundChannel, BackgroundPreset};
use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

#[test]
fn background_preset_changes_composition_and_undoes_in_one_step() {
    let mut shell = shell();
    let before = shell.composition().expect("既定 comp がある").background;
    assert_eq!(before, [0.0, 0.0, 0.0, 1.0], "既定は不透明黒のはず");

    shell.update(Message::SettingsBackgroundPreset(BackgroundPreset::White));
    let after = shell.composition().expect("comp がある").background;
    assert_eq!(after, [1.0, 1.0, 1.0, 1.0], "White プリセットが反映されていない");

    shell.update(Message::Undo);
    let restored = shell.composition().expect("comp がある").background;
    assert_eq!(restored, before, "1回の Undo で背景プリセットが戻らない");
    assert!(!shell.can_undo(), "1回で床まで戻るはず(既定は編集ではない)");
}

#[test]
fn gray18_preset_is_a_neutral_gray_not_black_or_white() {
    let mut shell = shell();
    shell.update(Message::SettingsBackgroundPreset(BackgroundPreset::Gray18));
    let background = shell.composition().expect("comp がある").background;
    assert!(background[0] > 0.0 && background[0] < 1.0, "Gray18 が中間値でない: {background:?}");
    assert_eq!(background[0], background[1]);
    assert_eq!(background[1], background[2]);
    assert_eq!(background[3], 1.0);
}

#[test]
fn background_channel_submit_writes_only_that_channel_and_undoes_in_one_step() {
    let mut shell = shell();

    shell.update(Message::SettingsBackgroundChannelInput(
        BackgroundChannel::A,
        "0".to_owned(),
    ));
    shell.update(Message::SettingsBackgroundChannelSubmit(
        BackgroundChannel::A,
    ));

    let background = shell.composition().expect("comp がある").background;
    assert_eq!(
        background,
        [0.0, 0.0, 0.0, 0.0],
        "A だけ変えたはずが他チャンネルまで動いた"
    );

    shell.update(Message::Undo);
    let restored = shell.composition().expect("comp がある").background;
    assert_eq!(
        restored,
        [0.0, 0.0, 0.0, 1.0],
        "チャンネル編集も1回の Undo で戻らない"
    );
}

/// 数値として読めない入力は**黙って消さず** status 帯へ理由を出す(M13)。
#[test]
fn an_unreadable_channel_value_is_rejected_with_a_reason() {
    let mut shell = shell();
    shell.update(Message::SettingsBackgroundChannelInput(
        BackgroundChannel::R,
        "not a number".to_owned(),
    ));
    shell.update(Message::SettingsBackgroundChannelSubmit(
        BackgroundChannel::R,
    ));

    assert!(shell.status().is_some(), "拒否理由が出ていない = M13 違反");
    let background = shell.composition().expect("comp がある").background;
    assert_eq!(background, [0.0, 0.0, 0.0, 1.0], "読めない入力で背景が動いてしまった");
}

/// **市松トグルの本命**: screenshot/export が読む生 rgba には市松が絶対に乗らない。
#[test]
fn checkerboard_toggle_never_touches_the_raw_export_rgba() {
    let mut shell = shell();
    // 透明を作る(市松が効く条件、発注書「背景の alpha を 0 にした時に効く」)。
    shell.update(Message::SettingsBackgroundChannelInput(
        BackgroundChannel::A,
        "0".to_owned(),
    ));
    shell.update(Message::SettingsBackgroundChannelSubmit(
        BackgroundChannel::A,
    ));

    let before = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    shell.update(Message::ToggleCheckerboard);

    let after = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    assert_eq!(
        before, after,
        "市松トグルが export/screenshot 用の生 rgba を変えてしまった(書き出しに乗ってはいけない)"
    );
}

/// パネルの開閉自体は表示分岐だけ — Document にも undo 履歴にも乗らない。
#[test]
fn settings_panel_toggle_is_purely_a_view_flag() {
    let mut shell = shell();
    let layers_before = shell.layer_count();

    shell.update(Message::ToggleSettingsPanel);
    assert_eq!(
        shell.layer_count(),
        layers_before,
        "パネル開閉が Document を触っている"
    );
    assert!(
        !shell.can_undo(),
        "パネル開閉が undo 履歴に乗ってしまっている"
    );

    // トグルなので、もう一度で元に戻る(座学的だが「トグル」の定義そのもの)。
    shell.update(Message::ToggleSettingsPanel);
    shell.update(Message::ToggleSettingsPanel);
    // 3回押した = 開いている状態のはず。ここでは直接の可視 API が無いので
    // view() が panic しないことだけ見る(Q0 の他の柵が widget 単位の検分を担う)。
    let _ = shell.view();
}
