//! B42(裁定184 型別 section 第4号): Inspector AUDIO section — 音声を持ち得る
//! layer(`LayerSource::Media`)選択時**のみ** section が現れ、Level/Pan/
//! Fade In/Fade Out が既存の値セル文法(`TransformField`/`Intent::SetTrack`)
//! 経由でそのまま書けること。
//!
//! **新しい `Message` は無い** — mask/effect と違い per-id の一覧
//! (mode 巡回・reorder・bypass のような構造編集)を持たない、layer につき
//! 常に高々1組の固定4行なので、Position/Rotation/Opacity と同じ
//! `TransformField`/`KeyRow` の静的 variant として足すだけで
//! `FieldInput`/`FieldSubmit`/`ValuePressed`/`KeyPressed` の既存経路が
//! そのまま動く(`lib.rs` の `TransformField::Level` 等の doc 参照)。
//! そのため `motolii-shell` 側にも新しい match 腕は要らない
//! (`update_inspector` は `TransformField`/`KeyRow` を素通しする glue のまま
//! — RETURN 参照)。
//!
//! 落ちるテスト先行(発注の型)。**このファイルは `cargo check --tests` の
//! 対象であって `cargo test` では実行しない**(発注の規律どおり)。
//!
//! view の存在照合は `mask_section.rs`/`text_section.rs` と同じ iced_test 手口
//! (`Target::Text` 列挙)。

use iced_test::selector::{Candidate, Target};

use motolii_core::{Fps, RationalTime};
use motolii_inspector_pane::{
    commit_inspector_field, project, property_id, view, FieldDraft, KeyCellState, KeyRow,
    Message, RowValue, TransformField,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId,
    Value,
};
use motolii_tokens_rs::{Colors, Dimensions};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30fps は正値")
}

/// comp と layer を1つ置いた Document(`source` を呼び手が選べる —
/// `text_section.rs::doc_with_layer` と同じ派生形)。
fn doc_with_layer(source: LayerSource) -> (Document, LayerId) {
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
                source,
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置けるはず");
    (doc, layer)
}

fn media_layer() -> (Document, LayerId) {
    doc_with_layer(LayerSource::Media {
        path: "clip.mp4".to_owned(),
        fingerprint: None,
    })
}

fn session_selecting(layer: LayerId) -> Session {
    Session {
        selection: Some(layer),
        ..Session::default()
    }
}

/// `mask_section.rs::collect_targets` と同じ「`find` を尽きるまで繰り返す」手口。
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
// 投影: `LayerSource::Media` の layer でのみ section の中身が生える(Q0)
// ---------------------------------------------------------------------------

/// 音声を持ち得ない layer(solid)の投影は `audio` が `None` — view は
/// AUDIO section を出さない(`mask_section.rs`/`text_section.rs` の
/// 「型が合わない layer」テストと同型)。
#[test]
fn a_non_media_layer_projects_no_audio_section_and_shows_no_audio_header() {
    let (doc, layer) = doc_with_layer(LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    });
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");
    assert!(
        projection.audio.is_none(),
        "音声を持ち得ない layer に AUDIO 投影が生えている"
    );

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(
        !has_text(&targets, "AUDIO"),
        "音声を持ち得ない layer に AUDIO section header が出ている"
    );
}

/// `LayerSource::Media` の layer は track 未着手でも4行(Level=100%・
/// Pan=0.0・Fade In/Out=0.0秒)を store 既定から投影する(Position 等の
/// 「track が無ければ既定値」と同じ、裁定20)。
#[test]
fn a_media_layer_projects_the_four_audio_rows_with_store_defaults() {
    let (doc, layer) = media_layer();
    let session = session_selecting(layer);
    let projection = project(&doc.view(), &session)
        .expect("投影は組めるはず")
        .expect("選択ありなので Some のはず");

    let audio = projection
        .audio
        .as_ref()
        .expect("media layer は Some のはず");

    match &audio.level.value {
        RowValue::Scalar(slot) => {
            assert!(slot.present);
            assert!(slot.editable);
            assert_eq!(slot.value, 100.0, "既定 1.0(倍率)は表示 100(%)のはず");
            assert_eq!(slot.field, Some(TransformField::Level));
        }
        other => panic!("Level 行は Scalar のはず: {other:?}"),
    }
    match &audio.pan.value {
        RowValue::Scalar(slot) => assert_eq!(slot.value, 0.0, "既定 Pan は中央(0.0)のはず"),
        other => panic!("Pan 行は Scalar のはず: {other:?}"),
    }
    match &audio.fade_in.value {
        RowValue::Scalar(slot) => assert_eq!(slot.value, 0.0, "既定 Fade In は無効(0秒)のはず"),
        other => panic!("Fade In 行は Scalar のはず: {other:?}"),
    }
    match &audio.fade_out.value {
        RowValue::Scalar(slot) => assert_eq!(slot.value, 0.0, "既定 Fade Out は無効(0秒)のはず"),
        other => panic!("Fade Out 行は Scalar のはず: {other:?}"),
    }
    assert_eq!(audio.level.decimals, 1);
    assert_eq!(audio.pan.decimals, 2);
    assert_eq!(audio.fade_in.decimals, 2);
    assert_eq!(audio.fade_out.decimals, 2);
    assert_eq!(audio.level.key.row, KeyRow::Level);
    assert_eq!(audio.level.key.state, KeyCellState::Static, "track 未打ちは Static のはず");

    let targets = collect_targets(view(
        Some(&projection),
        None,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(has_text(&targets, "AUDIO"), "AUDIO section header が出ない");
    assert!(has_text(&targets, "Level"), "Level 行が出ない");
    assert!(has_text(&targets, "Pan"), "Pan 行が出ない");
    assert!(has_text(&targets, "Fade In"), "Fade In 行が出ない");
    assert!(has_text(&targets, "Fade Out"), "Fade Out 行が出ない");
}

// ---------------------------------------------------------------------------
// 編集: 既存の値セル文法(`FieldDraft` → `commit_inspector_field` →
// `Intent::SetTrack`)がそのまま4行を書ける
// ---------------------------------------------------------------------------

/// Level の Enter(150%)は `property::LEVEL` へ倍率 1.5 を書く
/// (`mask_section.rs::committing_a_mask_opacity_draft_writes_...` と同型)。
#[test]
fn committing_a_level_draft_writes_the_level_track_as_a_ratio() {
    let (mut doc, layer) = media_layer();
    let field = TransformField::Level;
    let mut draft = Some(FieldDraft {
        field,
        text: "150".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, Some(layer), 0, fps30(), field)
        .expect("Level の確定は成功するはず");

    let property = property_id(field).expect("Level の property は作れるはず");
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず")
        .expect("確定後は値が有るはず");
    assert_eq!(value, Value::F64(1.5), "表示 150(%)は store の倍率 1.5 のはず");
    assert_eq!(property, PropertyId::new(motolii_store::property::LEVEL).unwrap());
}

/// Pan の Enter は store 単位のまま変換なしで書く(-1.0..1.0、`property::PAN`
/// doc の「W3C StereoPannerNode と同じ人間可読単位」をそのまま転写した設計 —
/// RETURN の「単位と表示」決定参照)。範囲外の入力は commit 側で clamp する
/// (Opacity の `.clamp(0.0, 1.0)` と同じ「commit 側で範囲を守る」判断)。
#[test]
fn committing_a_pan_draft_writes_the_raw_value_and_clamps_out_of_range_input() {
    let (mut doc, layer) = media_layer();
    let field = TransformField::Pan;

    let mut draft = Some(FieldDraft {
        field,
        text: "-0.5".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, Some(layer), 0, fps30(), field)
        .expect("Pan の確定は成功するはず");
    let property = property_id(field).expect("Pan の property は作れるはず");
    let value = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず")
        .expect("確定後は値が有るはず");
    assert_eq!(value, Value::F64(-0.5), "Pan は変換なしでそのまま書くはず");

    let mut out_of_range = Some(FieldDraft {
        field,
        text: "3".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut out_of_range, Some(layer), 0, fps30(), field)
        .expect("Pan の確定は成功するはず");
    let clamped = doc
        .view()
        .value_at(layer, &property, RationalTime::ZERO)
        .expect("track を読めるはず")
        .expect("確定後は値が有るはず");
    assert_eq!(clamped, Value::F64(1.0), "範囲外の Pan 入力は 1.0 へ clamp されるはず");
}

/// Fade In/Out の Enter は秒をそのまま書く。負の入力は 0 へ clamp する
/// (負の尺は意味を持たないため)。
#[test]
fn committing_fade_in_and_fade_out_drafts_writes_seconds_and_clamps_negative_input() {
    let (mut doc, layer) = media_layer();

    let mut fade_in_draft = Some(FieldDraft {
        field: TransformField::FadeIn,
        text: "1.5".to_owned(),
    });
    commit_inspector_field(
        &mut doc,
        &mut fade_in_draft,
        Some(layer),
        0,
        fps30(),
        TransformField::FadeIn,
    )
    .expect("Fade In の確定は成功するはず");
    let fade_in_property =
        property_id(TransformField::FadeIn).expect("Fade In の property は作れるはず");
    assert_eq!(
        doc.view()
            .value_at(layer, &fade_in_property, RationalTime::ZERO)
            .expect("track を読めるはず"),
        Some(Value::F64(1.5))
    );

    let mut fade_out_draft = Some(FieldDraft {
        field: TransformField::FadeOut,
        text: "-2".to_owned(),
    });
    commit_inspector_field(
        &mut doc,
        &mut fade_out_draft,
        Some(layer),
        0,
        fps30(),
        TransformField::FadeOut,
    )
    .expect("Fade Out の確定は成功するはず");
    let fade_out_property =
        property_id(TransformField::FadeOut).expect("Fade Out の property は作れるはず");
    assert_eq!(
        doc.view()
            .value_at(layer, &fade_out_property, RationalTime::ZERO)
            .expect("track を読めるはず"),
        Some(Value::F64(0.0)),
        "負の Fade Out 入力は 0 へ clamp されるはず"
    );
}

/// 選択なしは黙って no-op(`commit_inspector_field` の既存柵 — mask/text と
/// 同じ安全側、AUDIO 固有のロジックを新設していないことの確認)。
#[test]
fn audio_field_edits_without_a_selection_are_silent_no_ops() {
    let mut doc = Document::new();
    let field = TransformField::Level;
    let mut draft = Some(FieldDraft {
        field,
        text: "150".to_owned(),
    });
    commit_inspector_field(&mut doc, &mut draft, None, 0, fps30(), field)
        .expect("選択なしは no-op のはず");
}

// ---------------------------------------------------------------------------
// ±1px 柵: AUDIO section の行も既存の行グラマー(`bordered_row`/
// `row_band_style`)をそのまま再利用しており、新しい寸法・border 幅を発明
// していない(`text_section_rows_reuse_the_existing_row_geometry_contract` と
// 同じ確認)。
// ---------------------------------------------------------------------------

#[test]
fn audio_section_rows_reuse_the_existing_row_geometry_contract() {
    use motolii_inspector_pane::row_band_style;

    let dims = Dimensions::default();
    let style = row_band_style(dims);
    assert_eq!(
        style.border.width, dims.theme().stroke.hairline,
        "AUDIO section が依拠する行 border 幅が既存トークンから動いた(幾何不変違反)"
    );
    assert_eq!(
        style.border.color.a, 0.0,
        "AUDIO section が依拠する行 border が透明でない(D5 違反)"
    );
}
