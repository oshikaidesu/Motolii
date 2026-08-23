//! 運転席 — 第6波 shell 結線(B36: create タブのカード実体化)。
//! `browser_pane::model::CreateKind` → `LayerSource::{Shape,Solid,Null}` の
//! `AddLayer` 合成(既存 `Message::AddLayer` と同じ「1 apply_all = 1 undo」の
//! 流儀)・生成後に選択、を検分する。pane 側は `Message::CreateFromCard` を
//! no-op のまま通す(`state.rs` の ORACLE)ので、実体化は shell 側のみで
//! 完結していることも合わせて見る。

use motolii_shell::{browser_pane, inspector_pane, Message, Shell};
use motolii_store::{LayerSource, PathSource, ShapeNode};

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

/// **本命(2026-08-24「ブラウザに8枚の札」発注 §1)**: PolyStar カードも
/// Rectangle/Ellipse と同じ `LayerSource::Shape` を書き、shape の中身が
/// `PathSource::PolyStar` になっている(空 layer にならない)。
#[test]
fn create_poly_star_adds_a_shape_layer_with_a_polystar_path_source() {
    let mut shell = Shell::new_fixture().0;

    create(&mut shell, browser_pane::model::CreateKind::PolyStar);

    let layer = shell.session().selection.expect("生成後に選択されていない");
    let source = shell.store_view().meta(layer).ok().flatten().unwrap().source;
    assert!(
        matches!(source, LayerSource::Shape),
        "PolyStar カードが LayerSource::Shape を書いていない"
    );
    let shapes = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(shape) = &shapes[0] else {
        panic!("PolyStar カードは Leaf を作るはず");
    };
    assert!(
        matches!(shape.source, PathSource::PolyStar { .. }),
        "PolyStar カードの shape source が PathSource::PolyStar でない: {:?}",
        shape.source
    );
}

/// **本命(歌詞動画/MV ペルソナ、2026-08-22 利用者裁定「追加するものは
/// Browser の中に全部入れる」)**: Text カードが `LayerSource::Text` を書き、
/// 生成直後から `TextDocument` を持つ(空 layer にならない — `create_from_card`
/// doc 参照)。
#[test]
fn create_text_adds_a_text_layer_with_a_default_text_document() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.layer_count();

    create(&mut shell, browser_pane::model::CreateKind::Text);

    assert_eq!(shell.layer_count(), before + 1, "Text カードが layer を増やしていない");
    let selected = shell.session().selection.expect("生成後に選択されていない");
    let source = shell
        .store_view()
        .meta(selected)
        .ok()
        .flatten()
        .expect("生成した layer の meta が引けない")
        .source;
    assert!(matches!(source, LayerSource::Text), "Text カードが LayerSource::Text を書いていない");

    let document = shell
        .store_view()
        .text_document(selected)
        .ok()
        .flatten()
        .expect("Text カード生成直後に TextDocument が無い(空 layer になっている)");
    assert_eq!(
        document,
        inspector_pane::default_text_document(),
        "Text カードの既定 TextDocument が Inspector の既定値とずれている"
    );
    assert!(shell.can_undo());
}

/// Text の生成も他3種と同じ「1 apply_all = 1 undo」— AddLayer/SetMeta/
/// SetAttrs/SetTextDocument の4 Intent が1回の Undo で丸ごと戻る。
#[test]
fn create_text_is_one_undo_step() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.layer_count();

    create(&mut shell, browser_pane::model::CreateKind::Text);
    assert_eq!(shell.layer_count(), before + 1);

    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        before,
        "1回の Undo で create 前の層数へ戻らない(1操作=1undoでない)"
    );
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
