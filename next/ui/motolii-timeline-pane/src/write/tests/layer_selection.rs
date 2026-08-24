//! Timeline rail の `SelectLayer` が pane-local `update` へ到達し、
//! `Session` の layer 選択集合と focus を更新することを確認する。

use crate::write::*;
use crate::rows::LayerSelectionOp;
use motolii_store::LayerId;

#[test]
fn select_layer_message_applies_single_toggle_and_range_to_session() {
    let mut doc = Document::new();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let order = vec![LayerId(1), LayerId(2), LayerId(3), LayerId(4)];

    let reason = pane.update(
        Message::SelectLayer {
            order: order.clone(),
            op: LayerSelectionOp::Single(LayerId(2)),
        },
        &mut doc,
        &mut session,
        iced::keyboard::Modifiers::default(),
    );
    assert!(reason.is_none());
    assert_eq!(session.selected_layers, vec![LayerId(2)]);
    assert_eq!(session.selection, Some(LayerId(2)));

    pane.update(
        Message::SelectLayer {
            order: order.clone(),
            op: LayerSelectionOp::Toggle(LayerId(4)),
        },
        &mut doc,
        &mut session,
        iced::keyboard::Modifiers::COMMAND,
    );
    assert_eq!(session.selected_layers, vec![LayerId(2), LayerId(4)]);
    assert_eq!(session.selection, None);

    pane.update(
        Message::SelectLayer { order, op: LayerSelectionOp::Range(LayerId(3)) },
        &mut doc,
        &mut session,
        iced::keyboard::Modifiers::SHIFT,
    );
    assert_eq!(session.selected_layers, vec![LayerId(3), LayerId(4)]);
    assert_eq!(session.selection, None);
}
