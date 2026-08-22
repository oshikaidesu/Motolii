//! B02 第1切片(裁定184): Inspector MASK section — mask を持つ layer での
//! **のみ** section が現れ、mode 巡回 / inverted トグル / opacity 値編集が
//! すべて既存文法(`Intent::SetMasks` / `Intent::SetTrack`)経由で書けること。
//!
//! 落ちるテスト先行(発注の型)。view の присут照合は
//! `value_cell_legibility.rs` と同じ iced_test 手口(`Target::Text` 列挙)。

use iced_test::selector::{Candidate, Target};

use motolii_core::{Fps, RationalTime};
use motolii_inspector_pane::{
    commit_inspector_field, cycle_inspector_mask_mode, project, property_id,
    toggle_inspector_mask_inverted, view, FieldDraft, Message, TransformField,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, Mask, MaskId,
    MaskMode, PropertyId, Value,
};
use motolii_tokens_rs::{Colors, Dimensions};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`motolii-store/tests/mask.rs` と同じ形)。
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

fn two_masks() -> Vec<Mask> {
    vec![
        Mask {
            id: MaskId(1),
            mode: MaskMode::Add,
            inverted: false,
        },
        Mask {
            id: MaskId(2),
            mode: MaskMode::Subtract,
            inverted: true,
        },
    ]
}

/// `value_cell_legibility.rs` と同じ「`find` を尽きるまで繰り返す」手口。
fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();
    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

fn has_text(targets: &[Target], content: &str) -> bool {
    targets
        .iter()
        .any(|t| matches!(t, Target::Text { content: c, .. } if c == content))
}

// ---------------------------------------------------------------------------
// 投影: mask を持つ layer でのみ section の中身が生える
// ---------------------------------------------------------------------------

/// mask の無い layer の投影は `masks` が空 — view は MASK section を出さない
/// (Q0: 効かない chrome を並べない、`empty_state` と同じ判断)。
#[test]
fn a_layer_without_masks_projects_no_mask_rows_and_shows_no_mask_section() {
    let (doc, layer) = doc_with_layer();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert!(
        projection.masks.is_empty(),
        "mask の無い layer に mask 行が生えている"
    );

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(
        !has_text(&targets, "MASK"),
        "mask の無い layer に MASK section header が出ている"
    );
}

/// mask 持ち layer は store の並びどおりに mask 行が生え、mode/inverted が
/// store の値を写す。opacity は track 未打ちなら既定 1.0 → 表示 100(%)。
#[test]
fn a_mask_bearing_layer_projects_one_row_per_mask_in_store_order() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    assert_eq!(projection.masks.len(), 2);
    assert_eq!(projection.masks[0].id, MaskId(1));
    assert_eq!(projection.masks[0].mode, MaskMode::Add);
    assert!(!projection.masks[0].inverted);
    assert_eq!(projection.masks[1].id, MaskId(2));
    assert_eq!(projection.masks[1].mode, MaskMode::Subtract);
    assert!(projection.masks[1].inverted);

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(has_text(&targets, "MASK"), "MASK section header が出ない");
    assert!(has_text(&targets, "Mask 1"), "mask 1 の行が出ない");
    assert!(has_text(&targets, "Mask 2"), "mask 2 の行が出ない");
    assert!(has_text(&targets, "Add"), "mode(Add)の表示が出ない");
    assert!(has_text(&targets, "Subtract"), "mode(Subtract)の表示が出ない");
}

// ---------------------------------------------------------------------------
// 編集: mode 巡回・inverted トグルは `Intent::SetMasks` 1回 = 1 undo
// ---------------------------------------------------------------------------

/// mode 巡回は対象 mask だけを1歩進め、1 undo で戻る。
#[test]
fn cycling_a_mask_mode_advances_only_that_mask_and_undoes_in_one_step() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");

    cycle_inspector_mask_mode(&mut doc, Some(layer), MaskId(1)).expect("mode 巡回は成功するはず");
    let masks = doc.view().masks(layer).expect("mask を読めるはず");
    assert_eq!(masks[0].mode, MaskMode::Subtract, "宣言順の次 mode のはず");
    assert_eq!(masks[1].mode, MaskMode::Subtract, "対象外の mask が動いている");
    assert!(masks[1].inverted, "対象外の mask の inverted が動いている");

    doc.undo();
    let masks = doc.view().masks(layer).expect("mask を読めるはず");
    assert_eq!(masks[0].mode, MaskMode::Add, "1 undo で戻らない(1操作=1 undo 違反)");
}

/// inverted トグルも同型 — 対象 mask だけが裏返り、1 undo で戻る。
#[test]
fn toggling_mask_inverted_flips_only_that_mask_and_undoes_in_one_step() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");

    toggle_inspector_mask_inverted(&mut doc, Some(layer), MaskId(1))
        .expect("inverted トグルは成功するはず");
    let masks = doc.view().masks(layer).expect("mask を読めるはず");
    assert!(masks[0].inverted, "対象 mask が裏返っていない");
    assert_eq!(masks[0].mode, MaskMode::Add, "トグルが mode を巻き込んでいる");
    assert!(masks[1].inverted, "対象外の mask が動いている");

    doc.undo();
    let masks = doc.view().masks(layer).expect("mask を読めるはず");
    assert!(!masks[0].inverted, "1 undo で戻らない(1操作=1 undo 違反)");
}

/// 選択なし・存在しない mask id は黙って no-op(`commit_inspector_field` と
/// 同じ安全側 — 選択が edit の合間に変わる稀なケースを捨てる)。
#[test]
fn mask_edits_without_a_selection_or_with_a_stale_id_are_silent_no_ops() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");
    let before = doc.view().masks(layer).expect("mask を読めるはず");

    cycle_inspector_mask_mode(&mut doc, None, MaskId(1)).expect("選択なしは no-op のはず");
    toggle_inspector_mask_inverted(&mut doc, Some(layer), MaskId(99))
        .expect("居ない mask id は no-op のはず");

    assert_eq!(
        doc.view().masks(layer).expect("mask を読めるはず"),
        before,
        "no-op のはずが Document が動いている"
    );
}

// ---------------------------------------------------------------------------
// 編集: opacity は既存の値セル文法(`TransformField` → `Intent::SetTrack`)
// ---------------------------------------------------------------------------

/// mask opacity の値セル Enter は `mask.{id}.opacity` track への
/// `Intent::SetTrack`(既存 `commit_inspector_field` がそのまま書く —
/// 表示 % → store 比 0..1)。
#[test]
fn committing_a_mask_opacity_draft_writes_the_mask_opacity_track_as_a_ratio() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");

    let field = TransformField::MaskOpacity(MaskId(1));
    let mut draft = Some(FieldDraft {
        field,
        text: "50".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, Some(layer), 0, fps30(), field)
        .expect("opacity の確定は成功するはず");

    let property = PropertyId::mask_opacity(MaskId(1));
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず")
        .expect("確定後は値が有るはず");
    assert_eq!(value, Value::F64(0.5), "表示 50(%)は store の比 0.5 のはず");
    assert_eq!(
        property_id(field).expect("mask opacity の property は作れるはず"),
        property,
        "field → property の対応が mask.{{id}}.opacity になっていない"
    );
}

/// 投影の opacity 行: 値セルは既存の editable 文法(field 付き・decimals 0・
/// % 表示)で、Key 列も既存3状態 oracle に乗る。
#[test]
fn the_projected_mask_opacity_row_reuses_the_value_cell_grammar() {
    use motolii_inspector_pane::{KeyCellState, KeyRow, RowValue};

    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetMasks {
        layer,
        masks: two_masks(),
    })
    .expect("mask を書けるはず");
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let row = &projection.masks[0].opacity;
    assert_eq!(row.label, "Opacity");
    assert_eq!(row.decimals, 0, "既存 Opacity 行と同じ精度のはず");
    assert_eq!(row.key.row, KeyRow::MaskOpacity(MaskId(1)));
    assert_eq!(row.key.state, KeyCellState::Static, "track 未打ちは Static のはず");
    match &row.value {
        RowValue::Scalar(slot) => {
            assert!(slot.present);
            assert!(slot.editable);
            assert_eq!(slot.value, 100.0, "既定 1.0(比)は表示 100(%)のはず");
            assert_eq!(slot.field, Some(TransformField::MaskOpacity(MaskId(1))));
        }
        other => panic!("opacity 行は Scalar のはず: {other:?}"),
    }
}
