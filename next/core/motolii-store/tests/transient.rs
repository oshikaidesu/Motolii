//! transient overlay(タスク#20 の恒久解) — 「ドラッグ中の途中経過は履歴に入れない。
//! store が非履歴の overlay を1枚持ち、評価だけに重ねる」の柵。
//!
//! `Document::set_transient`/`clear_transient`/`clear_all_transients` は edit
//! timeline に一切触れない。`StoreView::value_at` は overlay を最優先で返すが、
//! `StoreView::track` は生の意味だけを返す(overlay を一切見ない)。

use motolii_store::{Composition, Document, Fps, Intent, LayerId, PropertyId, RationalTime, Value};

fn prop(name: &str) -> PropertyId {
    PropertyId::new(name).expect("property name")
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).expect("rational time")
}

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.mark_undo_floor();
    (doc, layer)
}

/// overlay が無い property は素通し(track も値も無い)。
#[test]
fn value_at_falls_through_when_no_overlay_is_set() {
    let (doc, layer) = doc_with_layer();
    let position = prop("position.x");
    assert_eq!(doc.view().value_at(layer, &position, t(0)).unwrap(), None);
}

/// overlay を置くと `value_at` がそれを返す。track が既に打たれていても overlay が
/// 勝つ(裁定「評価だけに重ねる」— 生の track の上に乗る)。
#[test]
fn set_transient_is_reflected_in_value_at() {
    let (mut doc, layer) = doc_with_layer();
    let position = prop("position.x");

    doc.set_transient(layer, position.clone(), Value::F64(42.0));

    assert_eq!(
        doc.view().value_at(layer, &position, t(0)).unwrap(),
        Some(Value::F64(42.0)),
        "overlay が value_at に映っていない"
    );
    // 時刻を変えても overlay は同じ値のまま(評価済みの値をそのまま重ねているだけ、
    // 時刻依存の track ではない)。
    assert_eq!(
        doc.view().value_at(layer, &position, t(30)).unwrap(),
        Some(Value::F64(42.0))
    );
}

/// **overlay は `track()` には映らない**(裁定134 の線引き — `track()` は生の
/// 意味だけを返す)。overlay しか無い property の `track()` は `None` のまま。
#[test]
fn set_transient_does_not_appear_in_track() {
    let (mut doc, layer) = doc_with_layer();
    let position = prop("position.x");

    doc.set_transient(layer, position.clone(), Value::F64(42.0));

    assert_eq!(
        doc.view().track(layer, &position).unwrap(),
        None,
        "track() が overlay を返してしまっている(裁定134 の線引きに違反)"
    );
}

/// overlay の宛先は layer をまたいで誤爆しない。同じ property 名を持つ別 layer は
/// overlay の影響を受けない。
#[test]
fn transient_is_scoped_to_its_layer_not_leaked_to_other_layers() {
    let mut doc = Document::new();
    let dragged = LayerId(1);
    let other = LayerId(2);
    doc.apply_all([Intent::AddLayer(dragged), Intent::AddLayer(other)])
        .unwrap();
    doc.mark_undo_floor();

    let position = prop("position.x");
    doc.set_transient(dragged, position.clone(), Value::F64(99.0));

    assert_eq!(
        doc.view().value_at(dragged, &position, t(0)).unwrap(),
        Some(Value::F64(99.0))
    );
    assert_eq!(
        doc.view().value_at(other, &position, t(0)).unwrap(),
        None,
        "別 layer の同名 property に overlay が誤って乗っている"
    );
}

/// undo/redo は overlay に一切影響されない(逆方向): overlay を置いた状態でも
/// undo/redo は履歴の意味どおりに動く。
#[test]
fn undo_and_redo_are_unaffected_by_a_pending_transient() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let opacity = prop("opacity");

    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: motolii_store::KeyframeTrack::new(),
    })
    .unwrap();
    let head_after_edits = doc.edit_head();

    doc.set_transient(layer, opacity.clone(), Value::F64(0.25));

    assert!(doc.undo(), "overlay があっても undo は効くはず");
    assert_eq!(doc.edit_head(), head_after_edits - 1);
    // overlay はまだ生きている(undo は history だけを動かす)。
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.25)),
        "undo が overlay を巻き込んで消してしまっている"
    );

    assert!(doc.redo());
    assert_eq!(doc.edit_head(), head_after_edits);
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.25)),
        "redo 後も overlay は生きているはず"
    );
}

/// undo/redo は overlay に一切影響されない(逆方向2): overlay を置いても edit
/// timeline には何も追加されない — `apply` を1回もしていないのと undo できる
/// 回数が変わらない。
#[test]
fn setting_a_transient_does_not_grow_the_undo_history() {
    let (mut doc, layer) = doc_with_layer();
    let position = prop("position.x");
    let can_undo_before = doc.can_undo();
    let head_before = doc.edit_head();

    doc.set_transient(layer, position.clone(), Value::F64(1.0));
    doc.set_transient(layer, position.clone(), Value::F64(2.0));
    doc.set_transient(layer, position.clone(), Value::F64(3.0));

    assert_eq!(
        doc.can_undo(),
        can_undo_before,
        "overlay の書き換えが undo 履歴を伸ばしている"
    );
    assert_eq!(
        doc.edit_head(),
        head_before,
        "overlay が edit_head を進めている"
    );
}

/// 保存(`flattened()`)に overlay は乗らない。
#[test]
fn flattened_does_not_carry_the_transient_overlay() {
    let (mut doc, layer) = doc_with_layer();
    let position = prop("position.x");

    doc.set_transient(layer, position.clone(), Value::F64(500.0));

    let flat = doc.flattened().unwrap();
    assert_eq!(
        flat.view().value_at(layer, &position, t(0)).unwrap(),
        None,
        "flattened() が transient overlay を保存へ乗せてしまっている"
    );
}

/// `display_revision` は overlay の変化で動くが、`revision()` は overlay で動かない
/// (履歴の意味は overlay と無関係のまま)。
#[test]
fn display_revision_moves_with_the_overlay_but_revision_does_not() {
    let (mut doc, layer) = doc_with_layer();
    let position = prop("position.x");

    let revision_before = doc.revision();
    let display_before = doc.display_revision();

    doc.set_transient(layer, position.clone(), Value::F64(1.0));

    assert_eq!(
        doc.revision(),
        revision_before,
        "overlay の変化だけで revision() が動いてしまっている(履歴の意味が汚染される)"
    );
    assert_ne!(
        doc.display_revision(),
        display_before,
        "overlay を置いても display_revision が動かない(front が再描画できない)"
    );

    let display_after_first_set = doc.display_revision();
    doc.set_transient(layer, position.clone(), Value::F64(2.0));
    assert_ne!(
        doc.display_revision(),
        display_after_first_set,
        "overlay の値を書き換えても display_revision が動かない"
    );
}

/// `clear_transient` で overlay が消え、`value_at` は素の(track の)値へ戻る。
#[test]
fn clear_transient_removes_the_overlay() {
    let mut doc = Document::new();
    let layer = LayerId(1);
    let opacity = prop("opacity");
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: opacity.clone(),
        track: motolii_store::KeyframeTrack::new(),
    })
    .unwrap();
    doc.mark_undo_floor();

    doc.set_transient(layer, opacity.clone(), Value::F64(0.1));
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        Some(Value::F64(0.1))
    );

    doc.clear_transient(layer, &opacity);
    assert_eq!(
        doc.view().value_at(layer, &opacity, t(0)).unwrap(),
        // 空 track(キー0本)の eval は `Value::F64(0.0)`(`KeyframeTrack::eval` の
        // 既定、`motolii-eval` の仕様)であって `None` ではない — track 自体は
        // `SetTrack` で置いてあるので「property が無い」わけではない。
        Some(Value::F64(0.0)),
        "clear_transient 後も overlay の値が残っている(素の track の値へ戻っていない)"
    );
}

/// `clear_all_transients` は宛先を覚えていなくても全部消す(確定/キャンセルの
/// どちらでも呼んでよい保険口)。
#[test]
fn clear_all_transients_empties_the_overlay_regardless_of_scope() {
    let mut doc = Document::new();
    let layer_a = LayerId(1);
    let layer_b = LayerId(2);
    doc.apply_all([Intent::AddLayer(layer_a), Intent::AddLayer(layer_b)])
        .unwrap();
    doc.mark_undo_floor();

    let position = prop("position.x");
    let opacity = prop("opacity");
    doc.set_transient(layer_a, position.clone(), Value::F64(1.0));
    doc.set_transient(layer_b, opacity.clone(), Value::F64(0.5));

    doc.clear_all_transients();

    assert_eq!(doc.view().value_at(layer_a, &position, t(0)).unwrap(), None);
    assert_eq!(doc.view().value_at(layer_b, &opacity, t(0)).unwrap(), None);
}

/// カメラ property 版(`set_camera_transient`/`clear_camera_transient`)も同じ形で動く。
#[test]
fn camera_transient_overlays_camera_value_at_and_not_layer_properties() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.mark_undo_floor();

    let zoom = prop(motolii_store::property::CAMERA_ZOOM);
    doc.set_camera_transient(zoom.clone(), Value::F64(2.5));

    assert_eq!(
        doc.view().camera_value_at(&zoom, t(0)).unwrap(),
        Some(Value::F64(2.5))
    );
    // layer には zoom という property は無いので、たとえ同名で読んでも別の
    // component 名前空間(`Composition:zoom` ではなく `Layer:zoom`)なので無関係。
    assert_eq!(doc.view().value_at(layer, &zoom, t(0)).unwrap(), None);

    doc.clear_camera_transient(&zoom);
    assert_eq!(doc.view().camera_value_at(&zoom, t(0)).unwrap(), None);
}
