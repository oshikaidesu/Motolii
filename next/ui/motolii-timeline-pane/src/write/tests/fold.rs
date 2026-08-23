//! `Message::ToggleFold`(裁定173 H2)。SP-2 分割で元は `fold_message_tests`
//! (`write.rs` 内の兄弟モジュール)だった物をそのまま移設(中身は無改変)。

use crate::write::*;
use motolii_store::Composition;

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: motolii_store::Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    doc
}

/// **オラクル(赤→緑)**: `Message::ToggleFold` は `PaneState::update` だけで
/// 完結する — Shell 側(5例外の先取り)の改修は不要(mod doc の
/// 「shell/src は改修不要」節の柵)。
#[test]
fn toggle_fold_flips_session_state_without_touching_the_document() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let layer = LayerId(1);

    assert!(!session.timeline_fold.is_folded(layer));
    let reason = pane.update(
        Message::ToggleFold(layer),
        &mut doc,
        &mut session,
        iced::keyboard::Modifiers::default(),
    );
    assert!(reason.is_none(), "ToggleFold が拒否理由を返している");
    assert!(session.timeline_fold.is_folded(layer), "1回目のToggleFoldで畳まれていない");

    pane.update(Message::ToggleFold(layer), &mut doc, &mut session, iced::keyboard::Modifiers::default());
    assert!(!session.timeline_fold.is_folded(layer), "2回目のToggleFoldで開き直っていない");
}

/// 存在しない LayerId への ToggleFold も panic しない(fold 状態は
/// LayerId の存在に依存しない Session 側の集合、`Message::ToggleFold` doc 参照)。
#[test]
fn toggle_fold_on_a_missing_layer_does_not_panic() {
    let mut doc = doc_with_comp();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let ghost = LayerId(999_999);

    let reason =
        pane.update(Message::ToggleFold(ghost), &mut doc, &mut session, iced::keyboard::Modifiers::default());
    assert!(reason.is_none());
    assert!(session.timeline_fold.is_folded(ghost));
}
