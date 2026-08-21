//! Settings パネル(タスク#18)の `Message` 経由の柵。
//!
//! - 背景プリセット/チャンネル編集 → **1回の Undo で戻る**(read-modify-write の
//!   `Intent::SetComposition` が1操作=1 intent であること)。
//! - 市松トグルは Handle 表示だけを変える — screenshot/export が読む
//!   `frame_rgba()`(生 rgba)には**一切乗らない**(`settings_pane` モジュール
//!   doc「合成器が出せる」と「書き出しが吐く」は別問題)。**発火元は裁定163で
//!   Stage 下縁状態帯へ引っ越した**(`Message::Stage(stage::Message::
//!   ToggleCheckerboard)`)が、書き込み先(`Shell::checkerboard`)・合成ロジック
//!   (`settings_pane::composite_checkerboard`)は無改変なので、この試験は
//!   引き続きここに置く(ピクセル合成の正しさを見る試験であって、発火元の
//!   場所を見る試験ではない — 発火元の配線は `tests/stage_band_drive.rs` 側)。
//! - パネルの開閉は表示だけの分岐 — Document にも undo 履歴にも乗らない。
//!
//! `ui_scale` の書き戻し(`tokens::write_ui_scale_to_path`)は `tests/
//! ui_scale_fence.rs` 側で隔離した一時ファイルを使って検分する。ここでは
//! `tokens/dimensions.json`(複数 worktree・並列試験間で共有される delicate な
//! ファイル、`../reference/KNOWN.md`)へ実際に触る `Shell::update` 経路は
//! 意図的に叩かない。

use motolii_shell::settings_pane::{self, BackgroundChannel, BackgroundPreset};
use motolii_shell::{screenshot, stage, Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

#[test]
fn background_preset_changes_composition_and_undoes_in_one_step() {
    let mut shell = shell();
    let before = shell.composition().expect("既定 comp がある").background;
    assert_eq!(before, [0.0, 0.0, 0.0, 1.0], "既定は不透明黒のはず");

    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundPreset(BackgroundPreset::White)));
    let after = shell.composition().expect("comp がある").background;
    assert_eq!(after, [1.0, 1.0, 1.0, 1.0], "White プリセットが反映されていない");

    let _ = shell.update(Message::Undo);
    let restored = shell.composition().expect("comp がある").background;
    assert_eq!(restored, before, "1回の Undo で背景プリセットが戻らない");
    assert!(!shell.can_undo(), "1回で床まで戻るはず(既定は編集ではない)");
}

#[test]
fn gray18_preset_is_a_neutral_gray_not_black_or_white() {
    let mut shell = shell();
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundPreset(BackgroundPreset::Gray18)));
    let background = shell.composition().expect("comp がある").background;
    assert!(background[0] > 0.0 && background[0] < 1.0, "Gray18 が中間値でない: {background:?}");
    assert_eq!(background[0], background[1]);
    assert_eq!(background[1], background[2]);
    assert_eq!(background[3], 1.0);
}

#[test]
fn background_channel_submit_writes_only_that_channel_and_undoes_in_one_step() {
    let mut shell = shell();

    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelInput(
        BackgroundChannel::A,
        "0".to_owned(),
    )));
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelSubmit(
        BackgroundChannel::A,
    )));

    let background = shell.composition().expect("comp がある").background;
    assert_eq!(
        background,
        [0.0, 0.0, 0.0, 0.0],
        "A だけ変えたはずが他チャンネルまで動いた"
    );

    let _ = shell.update(Message::Undo);
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
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelInput(
        BackgroundChannel::R,
        "not a number".to_owned(),
    )));
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelSubmit(
        BackgroundChannel::R,
    )));

    assert!(shell.status().is_some(), "拒否理由が出ていない = M13 違反");
    let background = shell.composition().expect("comp がある").background;
    assert_eq!(background, [0.0, 0.0, 0.0, 1.0], "読めない入力で背景が動いてしまった");
}

/// **市松トグルの本命**: screenshot/export が読む生 rgba には市松が絶対に乗らない。
#[test]
fn checkerboard_toggle_never_touches_the_raw_export_rgba() {
    let mut shell = shell();
    // 透明を作る(市松が効く条件、発注書「背景の alpha を 0 にした時に効く」)。
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelInput(
        BackgroundChannel::A,
        "0".to_owned(),
    )));
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundChannelSubmit(
        BackgroundChannel::A,
    )));

    let before = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));

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
// 裁定141: 市松 = AE型の透明可視化モード
// ---------------------------------------------------------------------------

/// **裁定141 の本命**: 既定の不透明黒背景のままでも、市松トグルは screenshot の
/// 画素を実際に変える — プレビューは `Composition.background` を敷かずに合成
/// した結果(`motolii_engine::Engine::render_frame_without_background`)へ市松を
/// 敷く(AE型の透明可視化モード)。旧仕様(裁定141以前)ではこのケースは無反応
/// だった — その場しのぎの理由文(`checkerboard_invisible_reason`、現在は撤去
/// 済み)ではなく、無反応な状態自体をここで消す。
#[test]
fn checkerboard_toggle_changes_the_screenshot_pixels_on_default_opaque_background() {
    let mut shell = shell();
    let background = shell.composition().expect("既定 comp がある").background;
    assert_eq!(background, [0.0, 0.0, 0.0, 1.0], "既定は不透明黒のはず");
    // `Shell::new()` は `refresh_frame` をまだ呼んでいない(`frame` が `None`)まま
    // — ここで初回描画を済ませておかないと、次の `screenshot::render` の差分が
    // 「市松の効果」ではなく「frame が None→Some になっただけ」を拾ってしまう。
    let _ = shell.update(Message::FlushDrops);

    let without = screenshot::render(&mut shell).into_raw();
    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));
    let with = screenshot::render(&mut shell).into_raw();

    assert_ne!(
        without, with,
        "不透明背景のままでも市松トグルで screenshot の画素が変わるはず(裁定141)"
    );
}

/// **raw 生値の不変、既定の不透明黒背景**: `frame_rgba()`(export/screenshot が
/// 読む生値、常に背景込みの真値)は市松トグルで一切変わらない。裁定141で
/// プレビューは背景なし合成へ切り替わるが、それは表示専用の入力差分であって
/// export 真値には触れない(`checkerboard_toggle_never_touches_the_raw_export_rgba`
/// と対の試験 — あちらは alpha=0 背景、こちらは既定の不透明背景で同じ不変を見る)。
/// 「実際に画面へ出る絵」が変わることは
/// `checkerboard_toggle_changes_the_screenshot_pixels_on_default_opaque_background`
/// が別に固定する。
#[test]
fn checkerboard_toggle_does_not_touch_the_raw_export_rgba_when_background_is_opaque() {
    let mut shell = shell();
    let before = shell.composition().expect("既定 comp がある").background;
    assert_eq!(before, [0.0, 0.0, 0.0, 1.0], "既定は不透明黒のはず");
    // `Shell::new()` は `refresh_frame` をまだ呼んでいない(`frame` が `None`)
    // — `update` を1回通してフレームを立てておく(`Message::FlushDrops` は
    // `pending_drops` が空なら何も変えない、`new_fixture` が `refresh_frame` を
    // 明示的に呼ぶのと同じ理由)。
    let _ = shell.update(Message::FlushDrops);

    let without = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));

    let with = shell
        .frame_rgba()
        .map(|(w, h, px)| (w, h, px.to_vec()))
        .expect("frame がある");

    assert_eq!(
        without, with,
        "市松トグルが export 用の生 rgba を変えてしまった(不透明背景でも不変のはず)"
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
    let _ = shell.update(Message::Settings(settings_pane::Message::BackgroundPreset(BackgroundPreset::Transparent)));
    let background = shell.composition().expect("comp がある").background;
    assert_eq!(background, [0.0, 0.0, 0.0, 0.0], "Transparent プリセットが反映されていない");

    let without = screenshot::render(&mut shell).into_raw();
    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));
    let with = screenshot::render(&mut shell).into_raw();

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

    let _ = shell.update(Message::Settings(settings_pane::Message::ToggleSettingsPanel));
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
    let _ = shell.update(Message::Settings(settings_pane::Message::ToggleSettingsPanel));
    let _ = shell.update(Message::Settings(settings_pane::Message::ToggleSettingsPanel));
    // 3回押した = 開いている状態のはず。ここでは直接の可視 API が無いので
    // view() が panic しないことだけ見る(Q0 の他の柵が widget 単位の検分を担う)。
    let _ = shell.view();
}
