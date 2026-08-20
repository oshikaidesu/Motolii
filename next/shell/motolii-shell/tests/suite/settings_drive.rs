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
use motolii_shell::{screenshot, Message, Shell};

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

// ---------------------------------------------------------------------------
// 実機報告の修正: その場の理由(M13)が実際に木へ現れる/消える
// ---------------------------------------------------------------------------

use motolii_shell::settings_pane::CHECKERBOARD_INVISIBLE_HINT;

/// **本命**: Settings パネルを開いて市松を ON にすると(既定の不透明背景の
/// まま)、その場の理由が実際に `view()` の木へ現れる。理由テキストが
/// 現れないまま黙って無反応 = Q0/M13 違反(この試験が今回の実バグの直接固定)。
#[test]
fn opening_settings_and_toggling_checkerboard_on_an_opaque_background_shows_the_reason() {
    let mut shell = shell();
    shell.update(Message::ToggleSettingsPanel);
    shell.update(Message::ToggleCheckerboard);

    let mut ui = iced_test::simulator(shell.view());
    assert!(
        ui.click(CHECKERBOARD_INVISIBLE_HINT).is_ok(),
        "不透明背景+市松ONでその場の理由が木に現れていない"
    );
}

/// 理由は「今まさに無反応」の時だけ — 市松 OFF なら出ない(常時表示の chrome を
/// 増やさない、既存の静的 hint_row とは別物であることの確認)。
#[test]
fn the_reason_does_not_appear_while_checkerboard_is_off() {
    let mut shell = shell();
    shell.update(Message::ToggleSettingsPanel);

    let mut ui = iced_test::simulator(shell.view());
    assert!(
        ui.click(CHECKERBOARD_INVISIBLE_HINT).is_err(),
        "市松OFFなのにその場の理由が出てしまっている"
    );
}

/// 理由は「Transparent」プリセットへ切り替えると消える — 1クリックで
/// 「市松が見える状態」を作れることの裏付け(発注書の修正方針そのもの)。
#[test]
fn the_reason_disappears_after_switching_to_the_transparent_preset() {
    let mut shell = shell();
    shell.update(Message::ToggleSettingsPanel);
    shell.update(Message::ToggleCheckerboard);
    shell.update(Message::SettingsBackgroundPreset(BackgroundPreset::Transparent));

    let mut ui = iced_test::simulator(shell.view());
    assert!(
        ui.click(CHECKERBOARD_INVISIBLE_HINT).is_err(),
        "Transparent プリセット適用後もその場の理由が残っている"
    );
}

// ---------------------------------------------------------------------------
// 実機報告の修正: 市松トグルが screenshot 器具の絵へ実際に届くこと
// ---------------------------------------------------------------------------

/// **容疑1の直接固定(真因側)、全画素**: 既定 comp(不透明黒)のままだと、
/// 市松トグルは frame のどの画素も変えない(容疑1の結論どおり、
/// 不透明背景では市松が原理的に見えない仕様 — 容疑2「`composite_checkerboard`
/// 自体のバグ」を切り分ける試験でもある)。
///
/// 歴史注記: 市松レーンの初回実装時、外周1周だけ alpha=0 になる engine 側の
/// アーティファクトが見つかり、このテストは一時的に内側画素限定だった。
/// 真因は背景 layer の `order: i16::MIN` が depth_offset シェーダで quad を
/// 全辺 ~1.25px 縮めていたことで、根治済み(`BACKGROUND_ORDER = -1`、
/// `next/engine/motolii-engine/tests/background.rs` が回帰柵)。以後は全画素比較。
#[test]
fn checkerboard_toggle_does_not_touch_any_pixel_when_background_is_opaque() {
    let mut shell = shell();
    let before = shell.composition().expect("既定 comp がある").background;
    assert_eq!(before, [0.0, 0.0, 0.0, 1.0], "既定は不透明黒のはず");
    // `Shell::new()` は `refresh_frame` をまだ呼んでいない(`frame` が `None`)
    // — `update` を1回通してフレームを立てておく(`Message::FlushDrops` は
    // `pending_drops` が空なら何も変えない、`new_fixture` が `refresh_frame` を
    // 明示的に呼ぶのと同じ理由)。
    shell.update(Message::FlushDrops);

    let (w, h, without) = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    // `frame_rgba()` 自体は市松に触れない(`checkerboard_toggle_never_touches_
    // the_raw_export_rgba` で既に固定済み)ので、ここでは screenshot.rs が
    // 実際に行うのと同じ組み合わせ(`composite_checkerboard` を明示的に当てる)
    // を自分で再現する。
    let mut with = without.clone();
    motolii_shell::settings_pane::composite_checkerboard(w, h, &mut with, shell.tokens().colors);

    let pixel = |buf: &[u8], x: u32, y: u32| -> [u8; 4] {
        let i = ((y * w + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    };
    let mut mismatches = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if pixel(&without, x, y) != pixel(&with, x, y) {
                mismatches.push((x, y));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "不透明背景なのに内側画素が市松で変わった(容疑2の疑いあり): {} 件、例: {:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(5)]
    );
}

/// **修正の本命**: 背景を「Transparent」プリセットへ動かした状態でなら、
/// 市松トグルは screenshot の画素を実際に変える — `settings_pane::
/// composite_checkerboard` の単体試験は既にあったが、それが
/// `--fixture --screenshot` 器具の出力にまで実際に繋がっていることは
/// この試験までは固定されていなかった(容疑1「連動不足」がこの繋ぎ目の
/// 話であって、`composite_checkerboard` 自体のバグではないことの裏付け)。
#[test]
fn checkerboard_toggle_changes_the_screenshot_pixels_when_background_is_transparent() {
    let mut shell = shell();
    shell.update(Message::SettingsBackgroundPreset(BackgroundPreset::Transparent));
    let background = shell.composition().expect("comp がある").background;
    assert_eq!(background, [0.0, 0.0, 0.0, 0.0], "Transparent プリセットが反映されていない");

    let without = screenshot::render(&shell).into_raw();
    shell.update(Message::ToggleCheckerboard);
    let with = screenshot::render(&shell).into_raw();

    assert_ne!(
        without, with,
        "透明背景なら市松トグルで screenshot の画素が変わるはず"
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
