//! 運転席 — Inspector の Blend 巡回ボタン(BL2、`inspector_drive.rs` と同じ流儀)。
//!
//! 見るのは発注書の3点:
//! - 巡回で `attrs.blend_mode` が Normal→Add→Normal と回る(下書きを経由しない
//!   即時操作、`ToggleHidden` と同型)
//! - 1クリック = 1 undo エントリ
//! - Add へ切り替えても `refresh_frame`(`update()` の末尾が毎回呼ぶ)が
//!   `EngineError::UnsupportedBlendMode` を出さない(engine 側の受け皿が
//!   実際に繋がっていることの通し証拠)

use motolii_shell::inspector_pane;
use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

/// **巡回: Normal→Add→Normal、1クリック = 1 undo**。
#[test]
fn cycling_blend_mode_goes_normal_then_add_then_back_with_one_undo_per_click() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "新規 layer の既定 blend mode が Normal でない"
    );

    let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Add",
        "1回目の巡回が Add へ進んでいない"
    );
    assert!(shell.can_undo());

    let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "2回目の巡回が先頭(Normal)へ戻っていない"
    );

    // **1クリック = 1 undo**: 2回巡回したので Undo 2回で AddLayer まで戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Add",
        "Undo 1回で1手前(Add)へ戻らない"
    );
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.inspector_selection().unwrap().attrs.blend_mode,
        "Normal",
        "Undo 2回で AddLayer 直後(Normal)へ戻らない"
    );
    assert_eq!(shell.layer_count(), 1, "巡回の Undo が AddLayer まで戻した");
}

/// **Add へ切り替えても engine の対応外ガードに引っかからない**。`translate_blend_mode`
/// が Add を受け付ける腕を持っていない旧実装だと、`refresh_frame` が
/// `Stage を描けない: blend mode Add はまだ合成器が対応していない…` を status へ出す。
#[test]
fn switching_to_add_does_not_trip_the_unsupported_blend_mode_guard() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::CycleBlendMode));
    assert_eq!(shell.inspector_selection().unwrap().attrs.blend_mode, "Add");

    if let Some(status) = shell.status() {
        assert!(
            !status.contains("合成器が対応していない"),
            "Add へ切り替えたのに engine が対応外として弾いている: {status}"
        );
    }
}
