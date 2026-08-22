//! 運転席 — 裁定205 施工第2号(§A マスク追加・§B エフェクト適用)。
//!
//! `browser_pane::Message::AddMaskFromCard`/`ApplyEffectFromCard` は pane-local
//! には状態を動かさない(`state.rs` の ORACLE)ので、`create_from_card_drive.rs`
//! と同じ形で shell 側の実体化だけを見る:
//! - マスク追加は `Intent::AddMask` 1本 = 1操作 = 1undo で、生成直後から
//!   shape 込みで resolve が通る(壁7の恒久修正が実際に効いていることの証明)
//! - どちらも**単一選択の時だけ**意味を持つ(0件/複数選択では何もしない)
//! - Glow の適用は `Intent::SetEffects` を経由して実際に Document へ届く

use motolii_shell::stage::marquee::SelectLayers;
use motolii_shell::{browser_pane, Message, Shell};
use motolii_store::LayerId;

fn add_mask(shell: &mut Shell) {
    let _ = shell.update(Message::Browser(browser_pane::Message::AddMaskFromCard));
}

fn apply_glow(shell: &mut Shell) {
    let _ = shell.update(Message::Browser(browser_pane::Message::ApplyEffectFromCard {
        plugin_id: "motolii.glow",
    }));
}

fn select(shell: &mut Shell, ids: Vec<LayerId>) {
    let _ = shell.update(Message::Marquee(SelectLayers {
        ids,
        additive: false,
    }));
}

/// 単一選択された layer を1枚だけ持つ fixture を組む(`create_from_card_drive.rs`
/// と同じ「Solid を1枚置いて、生成直後の選択をそのまま使う」手口)。
fn shell_with_one_selected_layer() -> (Shell, LayerId) {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Solid,
    }));
    let layer = shell.session().selection.expect("生成後に選択されていない");
    (shell, layer)
}

/// **本命(§A)**: マスク追加は「一覧への追加」と「shape の初期値」が同じ
/// `apply()` 呼び出しで届き、生成直後から `resolve` が通る(壁7の恒久修正の
/// 実地証明 — `Intent::AddMask` を使わず `SetMasks`+`SetTrack` を手組みで
/// 順序を誤ると、旧実装ではここが `Err` になっていた)。
#[test]
fn add_mask_from_card_lands_with_a_resolvable_shape() {
    let (mut shell, layer) = shell_with_one_selected_layer();
    assert!(
        shell.store_view().masks(layer).unwrap().is_empty(),
        "生成直後の layer にマスクが無いはず"
    );

    add_mask(&mut shell);

    let masks = shell.store_view().masks(layer).unwrap();
    assert_eq!(masks.len(), 1, "マスクが1枚増えていない");
    // 新規 layer は `LayerTiming::place(playhead, ..)` で playhead から始まる
    // (`create_from_card` doc 参照) — fixture の playhead は尺の中盤
    // (`fixture.rs` doc)なので、frame 0 ではなく layer が実際に居る時刻で
    // resolve する。
    let fps = shell.composition().expect("comp が無い").fps;
    let query_t = motolii_store::RationalTime::try_from_frame(shell.session().playhead, fps)
        .expect("playhead から時刻を組めない");
    let resolved = shell
        .store_view()
        .resolve(layer, query_t)
        .expect("shape が無いマスクは resolve が Err を返すはず(壁7)")
        .expect("layer が resolve できない");
    assert_eq!(resolved.masks.len(), 1);
    // 既定 shape は矩形(4頂点・閉路) — 「何も起きていない」ように見えない
    // 大きさが乗っていることまで確かめる。
    assert_eq!(resolved.masks[0].shape.vertices.len(), 4);
    assert!(resolved.masks[0].shape.closed);
}

/// **1操作 = 1undo**: マスク追加を undo すると、一覧も shape の track も
/// 一緒に消える(2 intent 手組みだと undo の粒度がずれる恐れがあったのが
/// `AddMask` 1本化で解消されている、という主張の undo 面での固定)。
#[test]
fn add_mask_from_card_is_one_undo_step() {
    let (mut shell, layer) = shell_with_one_selected_layer();
    let before_undo = shell.can_undo();

    add_mask(&mut shell);
    assert!(shell.store_view().masks(layer).unwrap().len() == 1);

    let _ = shell.update(Message::Undo);
    assert!(
        shell.store_view().masks(layer).unwrap().is_empty(),
        "1回の Undo でマスク追加前へ戻らない(1操作=1undoでない)"
    );
    assert_eq!(shell.can_undo(), before_undo);
}

/// 選択が無い時は何もしない(拒否・status で理由を出すだけ)。
#[test]
fn add_mask_from_card_does_nothing_without_a_selection() {
    let (mut shell, layer) = shell_with_one_selected_layer();
    select(&mut shell, vec![]);

    add_mask(&mut shell);

    assert!(
        shell.store_view().masks(layer).unwrap().is_empty(),
        "選択が無いのにマスクが追加されている"
    );
}

/// 複数選択の時も何もしない(「どのレイヤーへ足すか」が曖昧な状態では動かない
/// — この codebase 既存の「曖昧な物には何もしない」慣習、
/// `browser_pane::model::can_replace_source` doc と同型)。
#[test]
fn add_mask_from_card_does_nothing_with_multiple_selected_layers() {
    let (mut shell, first) = shell_with_one_selected_layer();
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Solid,
    }));
    let second = shell.session().selection.expect("2枚目の生成後に選択されていない");
    select(&mut shell, vec![first, second]);

    add_mask(&mut shell);

    assert!(shell.store_view().masks(first).unwrap().is_empty());
    assert!(shell.store_view().masks(second).unwrap().is_empty());
}

/// **本命(§B)**: Glow カードのダブルクリックが実際に `Intent::SetEffects` を
/// 経由して Document へ届く(P3b が「Glow カードの `creates` が shell へ
/// 繋がっていない」と報告した壁10の恒久修正)。
#[test]
fn apply_effect_from_card_reaches_the_document() {
    let (mut shell, layer) = shell_with_one_selected_layer();
    assert!(shell.store_view().effects(layer).unwrap().is_empty());

    apply_glow(&mut shell);

    let effects = shell.store_view().effects(layer).unwrap();
    assert_eq!(effects.len(), 1, "effect が1件増えていない");
    assert_eq!(effects[0].plugin_id, "motolii.glow");
    assert!(effects[0].enabled, "追加直後は enabled のはず");
}

/// effect 適用も undo が効く(通常の編集と同じ経路、専用の履歴機構を持たない)。
#[test]
fn apply_effect_from_card_is_undoable() {
    let (mut shell, layer) = shell_with_one_selected_layer();

    apply_glow(&mut shell);
    assert_eq!(shell.store_view().effects(layer).unwrap().len(), 1);

    let _ = shell.update(Message::Undo);
    assert!(
        shell.store_view().effects(layer).unwrap().is_empty(),
        "1回の Undo で effect 追加前へ戻らない"
    );
}

/// 選択が無い時は何もしない。
#[test]
fn apply_effect_from_card_does_nothing_without_a_selection() {
    let (mut shell, layer) = shell_with_one_selected_layer();
    select(&mut shell, vec![]);

    apply_glow(&mut shell);

    assert!(shell.store_view().effects(layer).unwrap().is_empty());
}

/// 複数選択の時も何もしない(マスクと同じゲート)。
#[test]
fn apply_effect_from_card_does_nothing_with_multiple_selected_layers() {
    let (mut shell, first) = shell_with_one_selected_layer();
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Solid,
    }));
    let second = shell.session().selection.expect("2枚目の生成後に選択されていない");
    select(&mut shell, vec![first, second]);

    apply_glow(&mut shell);

    assert!(shell.store_view().effects(first).unwrap().is_empty());
    assert!(shell.store_view().effects(second).unwrap().is_empty());
}

/// **本命(§D、engine レーンからの引き継ぎ)**: Rectangle/Ellipse カードが
/// `Intent::SetShapes` を実際に積み、生成直後から中身のある `Shape` を持つ
/// (`LayerSource::Shape` だけを書いて `shapes` component が空のままだった
/// 旧実装の欠陥の恒久修正)。局所原点中心(`PathSource::Rectangle`/`Ellipse`
/// の契約)であることも合わせて確かめる。
#[test]
fn create_rectangle_writes_a_centered_filled_shape() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Rectangle,
    }));
    let layer = shell.session().selection.expect("生成後に選択されていない");

    let shapes = shell.store_view().shapes(layer).unwrap();
    assert_eq!(shapes.len(), 1, "Rectangle カードが shapes component を書いていない");
    match &shapes[0] {
        motolii_store::ShapeNode::Leaf(shape) => {
            assert!(shape.fill.is_some(), "塗りが無いと何も描かれない(shape.rs doc)");
            match shape.source {
                motolii_store::PathSource::Rectangle { size } => {
                    assert!(size.x > 0.0 && size.y > 0.0);
                }
                ref other => panic!("Rectangle カードのパス源が矩形でない: {other:?}"),
            }
        }
        other => panic!("Rectangle カードが Leaf でない ShapeNode を書いている: {other:?}"),
    }
}

#[test]
fn create_ellipse_writes_a_filled_ellipse_shape() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Browser(browser_pane::Message::CreateFromCard {
        kind: browser_pane::model::CreateKind::Ellipse,
    }));
    let layer = shell.session().selection.expect("生成後に選択されていない");

    let shapes = shell.store_view().shapes(layer).unwrap();
    assert_eq!(shapes.len(), 1, "Ellipse カードが shapes component を書いていない");
    match &shapes[0] {
        motolii_store::ShapeNode::Leaf(shape) => {
            assert!(shape.fill.is_some());
            assert!(matches!(shape.source, motolii_store::PathSource::Ellipse { .. }));
        }
        other => panic!("Ellipse カードが Leaf でない ShapeNode を書いている: {other:?}"),
    }
}
