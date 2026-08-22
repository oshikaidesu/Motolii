//! 運転席 — 第6波 shell 結線(B36: create タブのカード実体化)。
//! `browser_pane::model::CreateKind` → `LayerSource::{Shape,Solid,Null}` の
//! `AddLayer` 合成(既存 `Message::AddLayer` と同じ「1 apply_all = 1 undo」の
//! 流儀)・生成後に選択、を検分する。pane 側は `Message::CreateFromCard` を
//! no-op のまま通す(`state.rs` の ORACLE)ので、実体化は shell 側のみで
//! 完結していることも合わせて見る。

use motolii_shell::{browser_pane, Message, Shell};
use motolii_store::LayerSource;

fn create(shell: &mut Shell, kind: browser_pane::model::CreateKind) {
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard { kind }));
}

#[test]
fn create_solid_adds_a_layer_and_selects_it() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.layer_count();

    create(&mut shell, browser_pane::model::CreateKind::Solid);

    assert_eq!(shell.layer_count(), before + 1, "Solid カードが layer を増やしていない");
    let selected = shell.session().selection.expect("生成後に選択されていない");
    let source = shell
        .store_view()
        .meta(selected)
        .ok()
        .flatten()
        .expect("生成した layer の meta が引けない")
        .source;
    assert!(matches!(source, LayerSource::Solid { .. }), "Solid カードが LayerSource::Solid を書いていない");
    assert!(shell.can_undo());
}

#[test]
fn create_null_adds_a_null_layer() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.layer_count();

    create(&mut shell, browser_pane::model::CreateKind::Null);

    assert_eq!(shell.layer_count(), before + 1);
    let selected = shell.session().selection.expect("生成後に選択されていない");
    let source = shell.store_view().meta(selected).ok().flatten().unwrap().source;
    assert!(matches!(source, LayerSource::Null), "Null カードが LayerSource::Null を書いていない");
}

#[test]
fn create_rectangle_and_ellipse_both_add_a_shape_layer() {
    let mut shell = Shell::new_fixture().0;

    create(&mut shell, browser_pane::model::CreateKind::Rectangle);
    let rect_layer = shell.session().selection.expect("生成後に選択されていない");
    let rect_source = shell.store_view().meta(rect_layer).ok().flatten().unwrap().source;
    assert!(
        matches!(rect_source, LayerSource::Shape),
        "Rectangle カードが LayerSource::Shape を書いていない"
    );

    create(&mut shell, browser_pane::model::CreateKind::Ellipse);
    let ellipse_layer = shell.session().selection.expect("生成後に選択されていない");
    let ellipse_source = shell.store_view().meta(ellipse_layer).ok().flatten().unwrap().source;
    assert!(
        matches!(ellipse_source, LayerSource::Shape),
        "Ellipse カードが LayerSource::Shape を書いていない"
    );
    assert_ne!(rect_layer, ellipse_layer, "2回の create が同じ layer を指している");
}

#[test]
fn create_from_card_is_one_undo_step() {
    let mut shell = Shell::new_fixture().0;
    assert!(!shell.can_undo());
    create(&mut shell, browser_pane::model::CreateKind::Solid);
    assert!(shell.can_undo());
    let before = shell.layer_count();

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), before - 1, "1回の Undo で create 前の層数へ戻らない(1操作=1undoでない)");
}
