//! 運転席 — 2026-08-24「ブラウザに8枚の札」発注 §2(`OpKind` 7種の
//! 「選択へ適用する」札)。
//!
//! `browser_pane::Message::ApplyOpFromCard` は pane-local には状態を動かさない
//! (`state.rs` の ORACLE)ので、`mask_effect_from_card_drive.rs` と同じ形で
//! shell 側の実体化だけを見る:
//! - 演算子は `Intent::SetShapes`(丸ごと差し替え)を経由して実際に
//!   `Shape.ops` へ積まれる
//! - **shape の無いレイヤー(Solid/Text/Null)を選んでいる時は拒否する**
//!   (M13「無反応ゼロ」— 黙って何も起きないのは違反、`status` に理由が出る)
//! - 単一選択の時だけ意味を持つ(0件/複数選択では何もしない)

use motolii_shell::stage::marquee::SelectLayers;
use motolii_shell::{browser_pane, Message, Shell};
use motolii_store::{LayerId, ShapeNode};

fn apply_op(shell: &mut Shell, op: browser_pane::model::ShapeOpKind) {
    let _ = shell.update(Message::Browser(browser_pane::Message::ApplyOpFromCard { op }));
}

fn select(shell: &mut Shell, ids: Vec<LayerId>) {
    let _ = shell.update(Message::Marquee(SelectLayers {
        ids,
        additive: false,
    }));
}

/// 単一選択された Rectangle shape layer を1枚だけ持つ fixture を組む
/// (`mask_effect_from_card_drive.rs::shell_with_one_selected_layer` の
/// shape 版 — 演算子は shape が要るので Solid ではなく Rectangle を使う)。
fn shell_with_one_selected_shape_layer() -> (Shell, LayerId) {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Rectangle,
    }));
    let layer = shell.session().selection.expect("生成後に選択されていない");
    (shell, layer)
}

/// **本命**: TrimPath カードのダブルクリックが実際に `Intent::SetShapes` を
/// 経由して選択レイヤーの shape へ演算子を1段積む。
#[test]
fn apply_op_from_card_pushes_the_operator_onto_the_shape() {
    let (mut shell, layer) = shell_with_one_selected_shape_layer();
    let before = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(before_shape) = &before[0] else {
        panic!("Rectangle カードは Leaf を作るはず");
    };
    assert!(before_shape.ops.is_empty(), "生成直後の shape に演算子が無いはず");

    apply_op(&mut shell, browser_pane::model::ShapeOpKind::TrimPath);

    let after = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(after_shape) = &after[0] else {
        panic!("演算子適用後も Leaf のはず(演算子は shape の中身を変えない)");
    };
    assert_eq!(after_shape.ops.len(), 1, "演算子が1段増えていない");
    // variant の中身(恒等値でないこと)は次のテストで見る。
}

/// **本命2**: 演算子の既定値が恒等でない(M13「無反応ゼロ」— 押しても見た目が
/// 変わらないなら意味が無い、`Shell::default_op_kind` doc 参照)。TrimPath の
/// `end` が 1.0(全部描画=恒等)ではないことで確かめる。
#[test]
fn apply_op_from_card_uses_a_non_identity_default() {
    let (mut shell, layer) = shell_with_one_selected_shape_layer();

    apply_op(&mut shell, browser_pane::model::ShapeOpKind::TrimPath);

    let shapes = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(shape) = &shapes[0] else {
        panic!("Leaf のはず");
    };
    match &shape.ops[0].kind {
        motolii_vector::OpKind::TrimPath { end, .. } => {
            assert!(*end < 1.0, "TrimPath の既定 end が恒等値(1.0)のまま — M13 違反");
        }
        other => panic!("TrimPath を積んだのに {other:?} が積まれている"),
    }
}

/// **本命3(M13)**: shape の無いレイヤー(Solid)を選んでいる時は拒否する —
/// 黙って何も起きない(無反応ゼロ違反)にはしない。
#[test]
fn apply_op_from_card_refuses_a_layer_without_a_shape() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Solid,
    }));
    let layer = shell.session().selection.expect("生成後に選択されていない");
    assert!(shell.store_view().shapes(layer).unwrap().is_empty());

    apply_op(&mut shell, browser_pane::model::ShapeOpKind::Twist);

    assert!(
        shell.store_view().shapes(layer).unwrap().is_empty(),
        "shape の無い Solid レイヤーに演算子が積まれている"
    );
    assert!(
        shell.status().is_some(),
        "拒否した理由が status に出ていない(M13 無反応ゼロ)"
    );
}

/// 選択が無い時は何もしない(拒否・status で理由を出すだけ)。
#[test]
fn apply_op_from_card_does_nothing_without_a_selection() {
    let (mut shell, layer) = shell_with_one_selected_shape_layer();
    select(&mut shell, vec![]);

    apply_op(&mut shell, browser_pane::model::ShapeOpKind::Repeater);

    let shapes = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(shape) = &shapes[0] else {
        panic!("Leaf のはず");
    };
    assert!(shape.ops.is_empty(), "選択が無いのに演算子が積まれている");
}

/// 演算子適用も undo が効く(通常の編集と同じ経路、専用の履歴機構を持たない)。
#[test]
fn apply_op_from_card_is_undoable() {
    let (mut shell, layer) = shell_with_one_selected_shape_layer();

    apply_op(&mut shell, browser_pane::model::ShapeOpKind::RoundedCorners);
    let shapes = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(shape) = &shapes[0] else {
        panic!("Leaf のはず");
    };
    assert_eq!(shape.ops.len(), 1);

    let _ = shell.update(Message::Undo);
    let shapes = shell.store_view().shapes(layer).unwrap();
    let ShapeNode::Leaf(shape) = &shapes[0] else {
        panic!("Leaf のはず");
    };
    assert!(
        shape.ops.is_empty(),
        "1回の Undo で演算子追加前へ戻らない"
    );
}
