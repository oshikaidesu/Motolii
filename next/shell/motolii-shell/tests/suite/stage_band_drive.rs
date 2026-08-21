//! 運転席 — Stage 下縁状態帯(裁定163 S 空間スコア、`docs/ui-spatial-score.md`
//! S5「下縁=状態帯」・S6「状態は隠れない」の初適用)。発注書 ORACLE (b) の
//! Shell 経由の柵:
//!
//! - プレビュー解像度の cap は `Shell::update` 経由で Auto→½→¼→Auto と回る
//!   (実効スケールが実際に変わることは `render_pipeline_fence.rs::
//!   cycling_the_resolution_cap_composes_via_min_with_the_auto_derived_scale`
//!   が `metrics` 経由で固定する — 全 suite が共有する `metrics` static を
//!   汚染しないため、そちらは専用プロセス `tests/metrics_main.rs` 側に置いた、
//!   `tests/suite_main.rs` 冒頭 doc 参照)。
//! - 観測開始で状態帯に視点項目が現れ、その項目自体をクリックすると
//!   `ResetToRenderCamera` が発行される(S6 — キーボード Shift+F と表面入口の
//!   両方が同じ動詞を指す、Q0: 触れる)。
//! - 市松トグルは新しい家(`stage::Message::ToggleCheckerboard`)から効く —
//!   旧 `settings_pane::Message::ToggleCheckerboard` はもう存在しない
//!   (`tests/settings_drive.rs` 側で確認済みの合成ロジック自体は無改変、
//!   ここでは「発火元が Stage 側にある」ことだけを見る)。
//!
//! **`canvas` と違い、この帯は標準 widget(`button`/`row`/`text`)構成**
//! なので `iced_test::simulator` で直接クリックできる。**座標(bounds 中心)
//! ではなく `Simulator::click(text)` の文字列一致で選ぶ**(`iced_selector::
//! Selector for &str` は `Candidate::Text` の内容と完全一致するものだけを
//! 拾う、`iced_test` 冒頭 doc の counter 例と同じ使い方) — `row!` の並び順や
//! `button` が(container+内側 text の)何個の candidate を登録するかという
//! iced 内部の実装詳細に依存しない、意味で選ぶ形。

use motolii_engine::ObservationCamera;
use motolii_shell::{stage, Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

fn zoomed() -> ObservationCamera {
    ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 1.8,
    }
}

// ---------------------------------------------------------------------------
// 解像度 cap の巡回(Shell::update 経由、状態そのもの)
// ---------------------------------------------------------------------------

#[test]
fn resolution_cap_defaults_to_auto() {
    let shell = shell();
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Auto);
}

#[test]
fn cycling_resolution_cap_through_shell_update_wraps_auto_half_quarter() {
    let mut shell = shell();

    let _ = shell.update(Message::Stage(stage::Message::CycleResolutionCap));
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Half);

    let _ = shell.update(Message::Stage(stage::Message::CycleResolutionCap));
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Quarter);

    let _ = shell.update(Message::Stage(stage::Message::CycleResolutionCap));
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Auto);
}

// ---------------------------------------------------------------------------
// 市松トグルが新しい家(Stage)から効く
// ---------------------------------------------------------------------------

#[test]
fn toggle_checkerboard_fires_from_the_stage_message_now() {
    let mut shell = shell();
    assert!(!shell.checkerboard_enabled(), "既定は OFF のはず");

    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));
    assert!(
        shell.checkerboard_enabled(),
        "Message::Stage(stage::Message::ToggleCheckerboard) で市松が ON にならない"
    );

    let _ = shell.update(Message::Stage(stage::Message::ToggleCheckerboard));
    assert!(!shell.checkerboard_enabled(), "もう一度で OFF に戻らない");
}

// ---------------------------------------------------------------------------
// `stage::state_band_view` 単体 — widget レベルの Q0/S6 柵
// ---------------------------------------------------------------------------

/// 呼び出しごとに固定の `auto_scale`(0.5)・`resolution_cap`(`Auto`)を使う —
/// この2つの組が [`VIEWPOINT_LABEL`]/[`RESOLUTION_LABEL`] の期待文字列を決める。
fn band(observation: Option<ObservationCamera>, checkerboard: bool) -> iced::Element<'static, stage::Message> {
    let dims = motolii_shell::tokens::Dimensions::default();
    let colors = motolii_shell::tokens::Colors::default();
    stage::state_band_view(
        observation,
        stage::PreviewResolutionCap::Auto,
        0.5,
        checkerboard,
        dims,
        colors,
    )
}

/// [`zoomed`] の観測カメラが表示するラベルそのもの
/// (`stage::state_band_view` の `format!("観測 {:.1}×(Shift+F で復帰)", ...)`)。
const VIEWPOINT_LABEL: &str = "観測 1.8×(Shift+F で復帰)";
/// [`band`] が常に `auto_scale=0.5`・`cap=Auto` で呼ぶので、実効スケールは
/// 常に `0.50`(`effective_preview_scale(0.5, Auto) == 0.5`、no-op)。
const RESOLUTION_LABEL: &str = "Auto(0.50×)";

/// **既定視点(`None`)では視点項目そのものが木に無い**(発注書 EXACT TARGET 1
/// 「既定視点では項目ごと非表示」)——観測中だけ同じラベルが見つかる。
#[test]
fn the_viewpoint_item_only_exists_in_the_tree_while_observing() {
    let mut idle = iced_test::simulator(band(None, false));
    assert!(
        idle.find(VIEWPOINT_LABEL).is_err(),
        "既定視点なのに視点項目が木にある(項目ごと非表示のはず)"
    );

    let mut observing = iced_test::simulator(band(Some(zoomed()), false));
    assert!(
        observing.find(VIEWPOINT_LABEL).is_ok(),
        "観測中なのに視点項目が木に無い"
    );
}

/// **本命(S6 — κ FINDING 4根治)**: 観測中、視点項目そのものをクリックすると
/// `ResetToRenderCamera` が発行される——Shift+F と表面入口の両方が同じ動詞を
/// 指す(唯一の入口を隠し場所にしない)。
#[test]
fn clicking_the_viewpoint_item_emits_reset_to_render_camera() {
    let mut ui = iced_test::simulator(band(Some(zoomed()), false));
    ui.click(VIEWPOINT_LABEL).expect("視点項目が見つかるはず");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![stage::Message::ResetToRenderCamera],
        "視点項目のクリックが ResetToRenderCamera を出さない: {messages:?}"
    );
}

/// 解像度項目(常時表示)をクリックすると `CycleResolutionCap`。
#[test]
fn clicking_the_resolution_item_emits_cycle_resolution_cap() {
    let mut ui = iced_test::simulator(band(None, false));
    ui.click(RESOLUTION_LABEL).expect("解像度項目が見つかるはず");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![stage::Message::CycleResolutionCap],
        "解像度項目のクリックが CycleResolutionCap を出さない: {messages:?}"
    );
}

/// 市松項目(常時表示)をクリックすると `ToggleCheckerboard`。
#[test]
fn clicking_the_checkerboard_item_emits_toggle_checkerboard() {
    let mut ui = iced_test::simulator(band(None, false));
    ui.click("市松: Off").expect("市松項目が見つかるはず");

    let messages: Vec<_> = ui.into_messages().collect();
    assert_eq!(
        messages,
        vec![stage::Message::ToggleCheckerboard],
        "市松項目のクリックが ToggleCheckerboard を出さない: {messages:?}"
    );
}
