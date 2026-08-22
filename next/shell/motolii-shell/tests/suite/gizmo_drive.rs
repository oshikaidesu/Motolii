//! 運転席 — 第5波 shell 結線(GZ: Stage ギズモ drag → Inspector と同経路の
//! transient/commit)の落ちるテスト先行。ギズモの座標解(hit/scale/rotate)は
//! `motolii-stage-pane::gizmo` の純関数試験が持つ — ここで見るのは**結線の
//! 契約**(`stage::GizmoDrag` doc): Start → Move* → Commit|Cancel が
//! `Document::set_transient` → `Intent::SetTrack`(1 drag = 1 undo)へ写ること。
//!
//! fixture(`Shell::new_fixture()`)実測値(`fixture.rs`):
//! - 既定選択「サビ歌詞」(`LayerId(7)`): **position キー 510/570**(キー持ち
//!   track — playhead upsert の検分対象)、playhead 既定 900
//! - 「タイトルロゴ」(`LayerId(1)`): position は静的(キー無し — 静的
//!   書き換えの検分対象)

use motolii_core::RationalTime;
use motolii_shell::{stage, Message, Shell};
use motolii_store::{property, Fps, LayerId, PropertyId, Value};

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

fn position_property() -> PropertyId {
    PropertyId::new(property::POSITION).expect("position は正規 property 名")
}

/// 今の playhead(900)での position 評価値(transient overlay 込み —
/// `StoreView::value_at` は overlay を映す、store の transient 試験参照)。
fn position_at_playhead(shell: &Shell, layer: LayerId) -> Option<[f64; 2]> {
    let t = RationalTime::try_from_frame(shell.session().playhead, fixture_fps()).ok()?;
    match shell.store_view().value_at(layer, &position_property(), t).ok()? {
        Some(Value::Vec2(v)) => Some(v),
        _ => None,
    }
}

fn gizmo(shell: &mut Shell, layer: LayerId, phase: stage::GizmoPhase) {
    let _ = shell.update(Message::Gizmo(stage::GizmoDrag { layer, phase }));
}

// ---------------------------------------------------------------------------
// Move = transient(undo に乗らない)/ Commit = SetTrack 1回(1 drag = 1 undo)
// ---------------------------------------------------------------------------

#[test]
fn gizmo_move_writes_a_transient_and_commit_lands_one_undoable_set_track() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1); // position 静的の層(キー無し → 静的書き換え)
    let _ = shell.update(Message::Select(layer));
    assert!(!shell.can_undo(), "fixture は undo 床の上に立っているはず");
    let before = position_at_playhead(&shell, layer);

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Position });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move { value: stage::GizmoValue::Position([333.0, 222.0]) },
    );
    assert_eq!(
        position_at_playhead(&shell, layer),
        Some([333.0, 222.0]),
        "Move の transient が value_at に映っていない"
    );
    assert!(!shell.can_undo(), "Move(transient)が undo 履歴に乗ってしまっている");

    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Commit { value: stage::GizmoValue::Position([333.0, 222.0]) },
    );
    assert_eq!(
        position_at_playhead(&shell, layer),
        Some([333.0, 222.0]),
        "Commit 後の本 track に値が乗っていない"
    );
    assert!(shell.can_undo(), "Commit が undo 履歴に乗っていない");

    // 1 drag = 1 undo: 1回の Undo で drag 前へ完全に戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(position_at_playhead(&shell, layer), before, "1回の Undo で drag 前へ戻らない");
    assert!(!shell.can_undo(), "1 drag が複数 undo 段になっている");
}

// ---------------------------------------------------------------------------
// キー持ち track への Commit = playhead upsert(`edited_value_track` と同経路)
// ---------------------------------------------------------------------------

#[test]
fn gizmo_commit_on_a_keyed_track_upserts_a_key_at_the_playhead() {
    let mut shell = Shell::new_fixture().0;
    let layer = shell.session().selection.expect("fixture は1層選択済み"); // LayerId(7)
    let keys_before = shell
        .store_view()
        .track(layer, &position_property())
        .expect("track を読める")
        .expect("サビ歌詞は position キー持ち")
        .keys()
        .len();
    assert_eq!(keys_before, 2, "fixture の position キーは 510/570 の2個のはず");

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Position });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move { value: stage::GizmoValue::Position([10.0, 20.0]) },
    );
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Commit { value: stage::GizmoValue::Position([10.0, 20.0]) },
    );

    let track = shell
        .store_view()
        .track(layer, &position_property())
        .expect("track を読める")
        .expect("commit 後も track がある");
    assert_eq!(
        track.keys().len(),
        3,
        "キー持ち track への commit が playhead(900)へキーを upsert していない(静的化は AE 文法違反)"
    );
    assert_eq!(
        position_at_playhead(&shell, layer),
        Some([10.0, 20.0]),
        "playhead のキー値が commit 値になっていない"
    );

    // 1 drag = 1 undo(キー持ちでも同じ)。
    let _ = shell.update(Message::Undo);
    let restored = shell
        .store_view()
        .track(layer, &position_property())
        .expect("track を読める")
        .expect("undo 後も track がある");
    assert_eq!(restored.keys().len(), 2, "1回の Undo で upsert 前の2キーへ戻らない");
}

// ---------------------------------------------------------------------------
// Cancel / Esc = transient を外すだけ(履歴は最初から無傷)
// ---------------------------------------------------------------------------

#[test]
fn gizmo_cancel_restores_the_value_without_touching_history() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1);
    let _ = shell.update(Message::Select(layer));
    let before = position_at_playhead(&shell, layer);

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Position });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move { value: stage::GizmoValue::Position([333.0, 222.0]) },
    );
    gizmo(&mut shell, layer, stage::GizmoPhase::Cancel);

    assert_eq!(position_at_playhead(&shell, layer), before, "Cancel が transient を外していない");
    assert!(!shell.can_undo(), "Cancel なのに undo 履歴に何かが乗っている");
}

/// Esc 連鎖(clip/key/loop/gizmo の並び — `Message::EscapePressed`)からも
/// 同じ復元が効く。canvas 側の Esc(`GizmoPhase::Cancel`)と二重に届いても
/// 壊れない(冪等)ことも一緒に見る。
#[test]
fn escape_cancels_a_gizmo_drag_and_a_duplicate_cancel_is_harmless() {
    let mut shell = Shell::new_fixture().0;
    let layer = LayerId(1);
    let _ = shell.update(Message::Select(layer));
    let before = position_at_playhead(&shell, layer);

    gizmo(&mut shell, layer, stage::GizmoPhase::Start { property: stage::GizmoProperty::Position });
    gizmo(
        &mut shell,
        layer,
        stage::GizmoPhase::Move { value: stage::GizmoValue::Position([1.0, 2.0]) },
    );
    let _ = shell.update(Message::EscapePressed);
    assert_eq!(position_at_playhead(&shell, layer), before, "Esc が gizmo drag を復元していない");

    // canvas 側の Esc 経路(GizmoPhase::Cancel)が続けて届いても no-op。
    gizmo(&mut shell, layer, stage::GizmoPhase::Cancel);
    assert_eq!(position_at_playhead(&shell, layer), before);
    assert!(!shell.can_undo());
}

// ---------------------------------------------------------------------------
// 絵の結線: 選択があればギズモ overlay が組める(Solid = declared_size)
// ---------------------------------------------------------------------------

#[test]
fn a_selected_solid_layer_yields_a_gizmo_overlay_and_no_selection_yields_none() {
    let mut shell = Shell::new_fixture().0;
    // 既定 playhead(900)はサビ歌詞の clip [510, 780) の**外** — 見えていない
    // 層のギズモは出さない(Q0)が正なので、まず clip の中へ scrub する。
    let _ = shell.update(Message::ScrubTo(600));
    assert!(
        shell.stage_gizmo_overlay().is_some(),
        "選択中の Solid 層にギズモ overlay が組めていない(size=declared_size のはず)"
    );

    let _ = shell.update(Message::DeselectAllLayers);
    assert!(
        shell.stage_gizmo_overlay().is_none(),
        "選択なしでギズモ overlay が出ている(Q0: 触れない物を描かない)"
    );
}
