//! B03(ラベル色、第3切片): ident 帯の色チップ — `LayerAttrs.label_color` を
//! 投影が写し、click(`cycle_inspector_label_color`)が palette index を宣言順の
//! 次へ巡回する(即1回の `Intent::SetAttrs` = 1 click = 1 undo)。
//!
//! 塗り(index → `colors.label_palette`、未割当 → `way_timeline`)は timeline
//! スウォッチ(`lane_bar::swatch_color`)と同じ源 — 式の同一性は view 内部
//! (`label_color_chip`)にあり、iced_test の `Target` は style(色)を運ばない
//! (`inspector_pixel_fence.rs` 冒頭の実測)ため、ここでは投影と書き口の意味
//! だけを固定する。**未実行**(supervisor が波末一括 — 裁定189 の運転規律)。

use motolii_core::Fps;
use motolii_inspector_pane::{cycle_inspector_label_color, next_label_color, project};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming,
};
use motolii_tokens_rs::LABEL_PALETTE_LEN;

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`mask_section.rs` と同じ形)。
fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置けるはず");
    (doc, layer)
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

/// 投影は `LayerAttrs.label_color` をそのまま写す(未割当は `None` のまま —
/// フォールバック色の解決は view の仕事で、投影は index を捏造しない)。
#[test]
fn the_projection_mirrors_the_stored_label_color_index() {
    let (mut doc, layer) = doc_with_layer();
    let session = session_selecting(layer);

    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert_eq!(projection.attrs.label_color, None, "未割当は None のまま写すはず");

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            label_color: Some(Some(4)),
            ..Default::default()
        },
    })
    .expect("attrs を書けるはず");
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert_eq!(projection.attrs.label_color, Some(4));
}

/// チップ click は index を宣言順の次へ1歩(未割当→先頭)、1 undo で戻る。
/// 他の attrs(name/hidden/…)を巻き込まない(patch は label_color 1フィールド)。
#[test]
fn cycling_advances_one_palette_step_and_undoes_in_one_step() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            name: Some("named".to_owned()),
            ..Default::default()
        },
    })
    .expect("attrs を書けるはず");

    // 未割当 → 先頭(0)。
    cycle_inspector_label_color(&mut doc, Some(layer)).expect("巡回は成功するはず");
    let attrs = doc
        .view()
        .attrs(layer)
        .expect("attrs を読めるはず")
        .expect("layer は居るはず");
    assert_eq!(attrs.label_color, Some(0), "未割当は先頭(0)から始まるはず");
    assert_eq!(attrs.name, "named", "巡回が name を巻き込んでいる");

    // 0 → 1。
    cycle_inspector_label_color(&mut doc, Some(layer)).expect("巡回は成功するはず");
    let attrs = doc
        .view()
        .attrs(layer)
        .expect("attrs を読めるはず")
        .expect("layer は居るはず");
    assert_eq!(attrs.label_color, Some(1));

    doc.undo();
    let attrs = doc
        .view()
        .attrs(layer)
        .expect("attrs を読めるはず")
        .expect("layer は居るはず");
    assert_eq!(
        attrs.label_color,
        Some(0),
        "1 undo で1歩だけ戻らない(1操作=1 undo 違反)"
    );
}

/// 末尾からは先頭へ一周する(palette 長は tokens の正本 `LABEL_PALETTE_LEN`)。
#[test]
fn the_cycle_wraps_from_the_last_palette_index_to_the_first() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            label_color: Some(Some((LABEL_PALETTE_LEN - 1) as u8)),
            ..Default::default()
        },
    })
    .expect("attrs を書けるはず");

    cycle_inspector_label_color(&mut doc, Some(layer)).expect("巡回は成功するはず");
    let attrs = doc
        .view()
        .attrs(layer)
        .expect("attrs を読めるはず")
        .expect("layer は居るはず");
    assert_eq!(attrs.label_color, Some(0), "末尾の次は先頭のはず");
}

/// 選択なしは黙って no-op(mask/effect 系と同じ安全側)。
#[test]
fn cycling_without_a_selection_is_a_silent_no_op() {
    let (mut doc, layer) = doc_with_layer();
    cycle_inspector_label_color(&mut doc, None).expect("選択なしは no-op のはず");
    // `doc_with_layer` は `SetAttrs` を一度も呼ばない — `StoreView::attrs` の
    // doc どおり「まだ一度も書かれていない」は `Ok(None)`(裁定37: 無いと空を
    // 同義にしない)。no-op の証拠は「`Intent::SetAttrs` が一切出ていない」
    // ことそのもの、すなわち attrs が最後まで未書き込み(`None`)であること。
    let attrs = doc.view().attrs(layer).expect("attrs を読めるはず");
    assert_eq!(
        attrs, None,
        "no-op のはずが Document が動いている(SetAttrs が出た形跡)"
    );
}

/// 純関数の境界([`next_label_color`]): 未割当→0・宣言順+1・末尾→0・
/// 範囲外 index(起こらないはず)も剰余で一覧内へ戻る。
#[test]
fn next_label_color_covers_the_unassigned_wrap_and_out_of_range_cases() {
    assert_eq!(next_label_color(None), 0);
    assert_eq!(next_label_color(Some(0)), 1);
    assert_eq!(
        next_label_color(Some((LABEL_PALETTE_LEN - 1) as u8)),
        0,
        "末尾は先頭へ一周するはず"
    );
    assert!(
        (next_label_color(Some(200)) as usize) < LABEL_PALETTE_LEN,
        "範囲外 index も一覧内へ戻るはず"
    );
}
