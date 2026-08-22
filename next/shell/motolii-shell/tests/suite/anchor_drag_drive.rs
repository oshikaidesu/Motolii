//! 運転席 — 第6波 shell 結線(GZ2: anchor drag pairing、
//! `gizmo.rs::GizmoValue::Anchor` doc「shell は両方を同時に書く」)。
//! anchor drag の Move/Commit が **anchor と position を対で** 書き、Commit は
//! 1 drag = 1 undo(2 property を1回の `apply_all` へ束ねる)ことを検分する。
//! Cancel は両方の transient を外すことも合わせて見る。

use motolii_core::RationalTime;
use motolii_shell::{stage, Message, Shell};
use motolii_store::{property, Fps, LayerId, PropertyId, Value};

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

fn value_at(shell: &Shell, layer: LayerId, name: &str) -> Option<Value> {
    let t = RationalTime::try_from_frame(shell.session().playhead, fixture_fps()).ok()?;
    let property = PropertyId::new(name).ok()?;
    shell.store_view().value_at(layer, &property, t).ok()?
}

fn gizmo(shell: &mut Shell, layer: LayerId, phase: stage::GizmoPhase) {
    let _ = shell.update(Message::Gizmo(stage::GizmoDrag { layer, phase }));
}

#[test]
fn anchor_move_writes_both_properties_as_a_transient_overlay() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1); // タイトルロゴ
    let _ = shell.update(Message::Select(layer));
    assert!(!shell.can_undo());

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Anchor });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move {
            value: stage::GizmoValue::Anchor { anchor: [10.0, 20.0], position: [300.0, 150.0] },
        },
    );

    assert_eq!(
        value_at(&shell, layer, property::ANCHOR),
        Some(Value::Vec2([10.0, 20.0])),
        "Move の anchor transient が value_at に映っていない"
    );
    assert_eq!(
        value_at(&shell, layer, property::POSITION),
        Some(Value::Vec2([300.0, 150.0])),
        "Move の position(補償)transient が value_at に映っていない"
    );
    assert!(!shell.can_undo(), "Move(transient)が undo 履歴に乗ってしまっている");
}

#[test]
fn anchor_commit_lands_exactly_one_undo_covering_both_properties() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1);
    let _ = shell.update(Message::Select(layer));
    let anchor_before = value_at(&shell, layer, property::ANCHOR);
    let position_before = value_at(&shell, layer, property::POSITION);

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Anchor });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move {
            value: stage::GizmoValue::Anchor { anchor: [10.0, 20.0], position: [300.0, 150.0] },
        },
    );
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Commit {
            value: stage::GizmoValue::Anchor { anchor: [10.0, 20.0], position: [300.0, 150.0] },
        },
    );

    assert_eq!(value_at(&shell, layer, property::ANCHOR), Some(Value::Vec2([10.0, 20.0])));
    assert_eq!(value_at(&shell, layer, property::POSITION), Some(Value::Vec2([300.0, 150.0])));
    assert!(shell.can_undo(), "Commit が undo 履歴に乗っていない");

    let _ = shell.update(Message::Undo);
    assert_eq!(
        value_at(&shell, layer, property::ANCHOR),
        anchor_before,
        "1回の Undo で anchor が drag 前へ戻らない"
    );
    assert_eq!(
        value_at(&shell, layer, property::POSITION),
        position_before,
        "1回の Undo で position が drag 前へ戻らない"
    );
    assert!(!shell.can_undo(), "anchor+position の1 gesture が複数 undo 段になっている");
}

#[test]
fn anchor_cancel_clears_both_transients_without_touching_the_document() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1);
    let _ = shell.update(Message::Select(layer));
    let anchor_before = value_at(&shell, layer, property::ANCHOR);
    let position_before = value_at(&shell, layer, property::POSITION);

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Anchor });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move {
            value: stage::GizmoValue::Anchor { anchor: [10.0, 20.0], position: [300.0, 150.0] },
        },
    );
    gizmo(&mut shell, layer, stage::GizmoPhase::Cancel);

    assert_eq!(
        value_at(&shell, layer, property::ANCHOR),
        anchor_before,
        "Cancel 後も anchor transient が残っている"
    );
    assert_eq!(
        value_at(&shell, layer, property::POSITION),
        position_before,
        "Cancel 後も position transient が残っている"
    );
    assert!(!shell.can_undo());
}
