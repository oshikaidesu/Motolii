//! 運転席 — Inspector の Blend 巡回ボタン(BL2、BL3 で分離可能11種を追加、
//! `inspector_drive.rs` と同じ流儀)。
//!
//! 見るのは発注書の3点:
//! - 巡回で `attrs.blend_mode` が `SUPPORTED_BLEND_MODES` の宣言順どおり一周して戻る
//!   (下書きを経由しない即時操作、`ToggleHidden` と同型)
//! - 1クリック = 1 undo エントリ
//! - どのモードへ切り替えても `refresh_frame`(`update()` の末尾が毎回呼ぶ)が
//!   `EngineError::UnsupportedBlendMode` を出さない(engine 側の受け皿が
//!   実際に繋がっていることの通し証拠 — **ORACLE「Inspector 巡回が全モードを
//!   一周する drive テスト」**そのもの)

use motolii_shell::inspector_pane::{self, SUPPORTED_BLEND_MODES};
use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

fn mode_name(mode: motolii_store::BlendMode) -> String {
    format!("{mode:?}")
}

/// **巡回: `SUPPORTED_BLEND_MODES` の宣言順どおり一周して Normal へ戻る、
/// 1クリック = 1 undo**。
#[test]
fn cycling_blend_mode_visits_every_supported_mode_in_order_with_one_undo_per_click() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "新規 layer の既定 blend mode が Normal でない"
    );

    // 先頭(Normal)の次から最後(Exclusion)まで、宣言順どおりに巡回する。
    for &mode in &SUPPORTED_BLEND_MODES[1..] {
        let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
        assert_eq!(
            shell.inspector_selection().unwrap().attrs.blend_mode,
            mode_name(mode),
            "巡回が {mode:?} へ進んでいない"
        );
        assert!(shell.can_undo());
    }

    // もう1回巡回すると先頭(Normal)へ一周して戻る。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "最後まで巡回した後、先頭(Normal)へ戻っていない"
    );

    // **1クリック = 1 undo**: `SUPPORTED_BLEND_MODES.len()` 回巡回したので、
    // 同じ回数 Undo すれば AddLayer 直後(Normal)まで戻る。
    for _ in 0..SUPPORTED_BLEND_MODES.len() {
        let _ = shell.update(Message::Undo);
    }
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "巡回した回数だけ Undo しても AddLayer 直後(Normal)へ戻らない"
    );
    assert_eq!(shell.layer_count(), 1, "巡回の Undo が AddLayer まで戻した");
}

/// **どのモードへ切り替えても engine の対応外ガードに引っかからない**
/// (`translate_blend_mode` が該当 mode を受け付ける腕を持っていない実装だと、
/// `refresh_frame` が `Stage を描けない: blend mode … はまだ合成器が対応していない…`
/// を status へ出す)。BL2 は Add 1つだけを見ていたが、BL3 で
/// `SUPPORTED_BLEND_MODES` の**全**モードを一周してこのガードが一度も
/// 発火しないことを縛る。
#[test]
fn cycling_through_every_supported_mode_never_trips_the_unsupported_blend_mode_guard() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    for &mode in &SUPPORTED_BLEND_MODES[1..] {
        let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
        assert_eq!(
            shell.inspector_selection().unwrap().attrs.blend_mode,
            mode_name(mode)
        );

        if let Some(status) = shell.status() {
            assert!(
                !status.contains("合成器が対応していない"),
                "{mode:?} へ切り替えたのに engine が対応外として弾いている: {status}"
            );
        }
    }
}
