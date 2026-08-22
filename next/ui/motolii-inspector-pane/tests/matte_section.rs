//! MATTE 行(2026-08-22 発注「レイヤーを指す」文法 第1号): `LayerAttrs.matte`
//! を選ぶ pick_list とその意味・書き口。engine は本日 `MatteMode`/`Matte` の
//! 消費を開始済み(`next/engine/motolii-engine/src/lib.rs::Engine::apply_matte`)
//! ──ここがその初めての呼び手。
//!
//! 落ちるテスト先行(発注の型)。**候補の絞り込み**(自分自身を選べない・
//! matte 連鎖の循環を選べない)が本発注の核 ── store 側に書き込み時の循環
//! 拒否が無い(`LayerAttrs::matte` の doc 参照 ── `parent`/`PropertyLink` と
//! 違う)ので、UI 側([`project`])の絞り込みが唯一の防波堤であることをここで
//! 検証する。

use motolii_core::Fps;
use motolii_inspector_pane::{
    clear_inspector_matte, cycle_inspector_matte_mode, next_matte_mode, project,
    set_inspector_matte_source,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming,
    Matte, MatteMode,
};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`tests/mask_section.rs::doc_with_layer`
/// と同じ形)。
fn doc_with_layer(id: u64) -> LayerId {
    LayerId(id)
}

fn seed_composition(doc: &mut Document) {
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp を置けるはず");
}

fn seed_layer(doc: &mut Document, layer: LayerId) {
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
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

// ---------------------------------------------------------------------------
// mode 巡回の意味
// ---------------------------------------------------------------------------

/// 宣言順どおり4値を一周して戻る。
#[test]
fn cycles_through_all_four_modes_and_wraps() {
    assert_eq!(next_matte_mode(MatteMode::Alpha), MatteMode::InvertedAlpha);
    assert_eq!(next_matte_mode(MatteMode::InvertedAlpha), MatteMode::Luma);
    assert_eq!(next_matte_mode(MatteMode::Luma), MatteMode::InvertedLuma);
    assert_eq!(next_matte_mode(MatteMode::InvertedLuma), MatteMode::Alpha);
}

// ---------------------------------------------------------------------------
// 投影: 候補の絞り込み(発注書の核)
// ---------------------------------------------------------------------------

/// **自分自身を選べない**(発注書「マット元は自分自身を選べてはいけない」)。
#[test]
fn matte_candidates_exclude_the_selected_layer_itself() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    let session = session_selecting(a);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let ids: Vec<LayerId> = projection
        .attrs
        .matte_candidates
        .iter()
        .map(|c| c.id)
        .collect();
    assert!(ids.contains(&b), "他 layer は候補に居るはず");
    assert!(!ids.contains(&a), "自分自身が候補に混ざっている");
}

/// **matte 連鎖の循環を選べない**(発注書「循環してもいけない」)。B が既に A
/// を matte 元にしている時、A の候補から B を除く(A→B→A の循環になる)。
#[test]
fn matte_candidates_exclude_a_layer_that_would_close_a_cycle() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    let c = doc_with_layer(3);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);
    seed_layer(&mut doc, c);

    // B の matte 元 = A(B→A)。
    set_inspector_matte_source(&mut doc, Some(b), a).expect("matte を書けるはず");

    // A の候補から B を除く(A→B→A の循環)。C は無関係なので候補に残る。
    let session = session_selecting(a);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    let ids: Vec<LayerId> = projection
        .attrs
        .matte_candidates
        .iter()
        .map(|candidate| candidate.id)
        .collect();
    assert!(!ids.contains(&b), "循環になる候補が絞り込まれていない");
    assert!(ids.contains(&c), "無関係な layer まで絞り込まれている");
}

// ---------------------------------------------------------------------------
// 書き口: 元の選択・mode 巡回・解除
// ---------------------------------------------------------------------------

/// 元を選ぶと `MatteMode::Alpha` で初期化され、1回の `Intent` = 1 undo。
#[test]
fn setting_a_matte_source_initializes_alpha_mode_and_undoes_in_one_step() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    set_inspector_matte_source(&mut doc, Some(a), b).expect("matte を書けるはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, Some(Matte { layer: b, mode: MatteMode::Alpha }));

    doc.undo();
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, None, "1 undo で戻らない(1操作=1 undo 違反)");
}

/// 既に matte が有る状態で元を差し替えても mode は保たれる。
#[test]
fn setting_a_new_source_keeps_the_existing_mode() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    let c = doc_with_layer(3);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);
    seed_layer(&mut doc, c);

    doc.apply(Intent::SetAttrs {
        layer: a,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte { layer: b, mode: MatteMode::Luma })),
            ..Default::default()
        },
    })
    .expect("matte を書けるはず");

    set_inspector_matte_source(&mut doc, Some(a), c).expect("元の差し替えは成功するはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, Some(Matte { layer: c, mode: MatteMode::Luma }), "mode が保たれていない");
}

/// mode 巡回は matte が有る時だけ効く。無ければ no-op。
#[test]
fn cycling_mode_is_a_no_op_without_a_matte_source() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    seed_layer(&mut doc, a);

    cycle_inspector_matte_mode(&mut doc, Some(a)).expect("matte 無しは no-op のはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, None);
}

/// mode 巡回は1回の `Intent` = 1 undo。
#[test]
fn cycling_mode_advances_one_step_and_undoes_in_one_step() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);
    set_inspector_matte_source(&mut doc, Some(a), b).expect("matte を書けるはず");

    cycle_inspector_matte_mode(&mut doc, Some(a)).expect("mode 巡回は成功するはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, Some(Matte { layer: b, mode: MatteMode::InvertedAlpha }));

    doc.undo();
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte.map(|m| m.mode), Some(MatteMode::Alpha), "1 undo で戻らない");
}

/// Clear は matte を外す。既に無ければ no-op(決定7 と同じ判断)。
#[test]
fn clearing_removes_the_matte_and_is_a_no_op_when_already_absent() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);
    set_inspector_matte_source(&mut doc, Some(a), b).expect("matte を書けるはず");

    clear_inspector_matte(&mut doc, Some(a)).expect("clear は成功するはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, None);

    let history_len_before = format!("{:?}", doc.view().attrs(a));
    clear_inspector_matte(&mut doc, Some(a)).expect("既に無ければ no-op のはず");
    let history_len_after = format!("{:?}", doc.view().attrs(a));
    assert_eq!(history_len_before, history_len_after, "no-op で値が変わっている");
}

/// 選択なしは黙って no-op(`commit_inspector_field` と同じ安全側)。
#[test]
fn matte_edits_without_a_selection_are_silent_no_ops() {
    let mut doc = Document::new();
    seed_composition(&mut doc);
    let a = doc_with_layer(1);
    let b = doc_with_layer(2);
    seed_layer(&mut doc, a);
    seed_layer(&mut doc, b);

    set_inspector_matte_source(&mut doc, None, b).expect("選択なしは no-op のはず");
    cycle_inspector_matte_mode(&mut doc, None).expect("選択なしは no-op のはず");
    clear_inspector_matte(&mut doc, None).expect("選択なしは no-op のはず");
    let attrs = doc.view().attrs(a).expect("attrs を読めるはず").unwrap_or_default();
    assert_eq!(attrs.matte, None);
}
