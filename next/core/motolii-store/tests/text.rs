//! text 束(75行)の第1・第2切片。
//!
//! **第1切片(静的組版の Document 意味)で固定したもの**:
//! - text-layer の中身(`Layer:text`)は content・組版既定値・フォント参照を持つ
//!   1個の component(裁定112(k) の後継)
//! - content(`t`)は**構造で Hold が保証された**時間変化(`animated-text-document k`)。
//!   線形補間もイージングも無い — 文字列に「中間」は無い
//! - font(`f`)の実体は **path + 指紋**(素材と同じ形、裁定79/97) — Lottie の名前参照は採らない
//! - line-height/tracking/fill/stroke/… は**スタイル表の既定行(index 0、裁定98)**として
//!   `TextDocument::style` に乗る。範囲ごとの複数行(runs)はまだ無い(次切片)
//! - `sz`(Wrap Size)は `None` = point text。Rive `text.width`/`text.height` と同じ静止設定
//! - `SetTextDocument` は**丸ごと差し替え**(`SetShapes`/`SetEffects` と同じ形)
//! - `sid`(Slot ID)は解決せずただ持てる(slots 機構自体は別発注単位、未着手)
//!
//! **第2切片(range selector とアニメータの Document 意味)で固定するもの**:
//! - [`TextRange`] は id で名前空間が決まるだけの入れ物。**動く量(selector の
//!   Start/End/Offset/Max Amount、style の fill/stroke/line-spacing/tracking)は
//!   フィールドとして持たず**、`PropertyId::text_range_*` の平坦な `KeyframeTrack`
//!   に乗る(マスク・effect と同じ形)
//! - `text-range-selector rn`(Randomize)は [`TextRandomize`] で **seed を必ず持つ**
//!   (Lottie の int-boolean のままでは Preview=Export と両立しない、裁定75/101)
//! - `TextDocument::ranges`(`Vec<TextRange>`)の**並び=適用順**。id の同一性は
//!   [`motolii_store`] 側の柵(重複 id は `SetTextDocument` が `Err`)で守る
//! - `TextDocument::alignment`(`text-data m`)はグループ内アンカーのオフセットと粒度
//!
//! **第3切片(Rive 由来の modifier/style span、text 束の最終切片)で固定するもの**:
//! - `TextDocument::style`(単数)は `styles`(`Vec<TextDocumentStyle>`)へ広がった —
//!   裁定98「Lottie の f/s/fc/lh/tr/sc/sw/of はスタイル表の既定行(id 0)」を型で体現する
//! - [`TextRun`]([`TextDocument::runs`])が「表 + 分割」の分割側(裁定85/87/88/89)。
//!   `len`(長さ、絶対オフセットではない)+ `style`([`TextStyleId`])。空は「既定行が
//!   全体を覆う」
//! - `text-modifier-group`(グリフの位置・回転・拡大・不透明度)は
//!   `PropertyId::text_range_origin`/`position`/`rotation`/`scale`/`opacity`
//! - `text-style-axis`/`text-variation-modifier` は可変フォント軸の**唯一動く例外**
//!   (裁定92/93) — スパン側の絶対値は `PropertyId::text_style_axis`、アニメーター側の
//!   Δ値は `PropertyId::text_range_variation_axis`。層が違うので二重帳簿ではない
//! - `text-style-feature`(OpenType feature)は [`TextStyleFeature`] で静止設定のまま

use motolii_store::{
    Composition, ContentKeyframe, ContentTrack, Document, FontRef, Fps, Interp, Intent, Keyframe,
    KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, SlotId,
    TextAlignmentOptions, TextBasedOn, TextDocument, TextDocumentStyle, TextGrouping, TextJustify,
    TextRandomize, TextRange, TextRangeId, TextRangeSelector, TextRangeUnits, TextRun, TextShape,
    TextStyleAxis, TextStyleFeature, TextStyleId, TextVariationAxis, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
    }))
    .unwrap();
    doc
}

fn place_text_layer(doc: &mut Document, layer: LayerId, start: i64, duration: i64) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Text,
                order: 0,
                timing: LayerTiming {
                    start,
                    duration,
                    source_in: 0,
                    ..Default::default()
                },
            },
        },
    ])
    .unwrap();
}

/// 分かりやすい既定スタイル(歌詞1行を想定)。**id 0 = スタイル表の既定行**(裁定98)。
fn lyric_style() -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef {
            path: "/fonts/NotoSansJP-Bold.otf".to_owned(),
            fingerprint: Some("motolii-source-v1:sha256:aa".repeat(2)),
            family: "Noto Sans JP".to_owned(),
            style: "Bold".to_owned(),
        },
        size: 64.0,
        fill: [1.0, 1.0, 1.0, 1.0],
        line_height: None,
        tracking: 0.0,
        stroke_color: Some([0.0, 0.0, 0.0, 1.0]),
        stroke_width: 4.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

fn lyric_document(content: &str) -> TextDocument {
    let mut content_track = ContentTrack::new();
    content_track.insert(ContentKeyframe {
        t: t(0),
        content: content.to_owned(),
    });
    TextDocument {
        content: content_track,
        justify: TextJustify::Center,
        wrap_size: None,
        styles: vec![lyric_style()],
        slot_id: None,
        ranges: Vec::new(),
        alignment: TextAlignmentOptions::default(),
        runs: Vec::new(),
    }
}

/// 分かりやすいアニメーター1個(カラオケワイプの selector を想定)。
fn karaoke_range(id: TextRangeId) -> TextRange {
    TextRange {
        id,
        name: "karaoke".to_owned(),
        selector: TextRangeSelector {
            based_on: TextBasedOn::CharactersExcludingSpaces,
            range_units: TextRangeUnits::Index,
            shape: TextShape::Square,
            randomize: None,
        },
        variation_axes: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Layer:text component — 丸ごと差し替え
// ---------------------------------------------------------------------------

#[test]
fn text_layer_holds_no_document_until_set() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    assert_eq!(doc.view().text_document(layer).unwrap(), None);
}

#[test]
fn set_text_document_round_trips_verbatim() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let document = lyric_document("歌詞1行目");
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document.clone(),
    })
    .unwrap();

    assert_eq!(doc.view().text_document(layer).unwrap(), Some(document));
}

/// `SetShapes`/`SetEffects` と同じ形 — **丸ごと差し替え**。`SetAttrs` のような部分更新
/// ではないので、2回目の `SetTextDocument` は前の内容を全部置き換える。
#[test]
fn set_text_document_replaces_the_whole_thing() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("1番"),
    })
    .unwrap();
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            justify: TextJustify::Left,
            ..lyric_document("2番")
        },
    })
    .unwrap();

    let current = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(current.content.eval(t(0)), "2番");
    assert_eq!(current.justify, TextJustify::Left);
}

// ---------------------------------------------------------------------------
// t / animated-text-document k — content の Hold track
// ---------------------------------------------------------------------------

#[test]
fn content_track_holds_across_multiple_keys_stored_in_the_document() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 300);

    let mut content = ContentTrack::new();
    content.insert(ContentKeyframe {
        t: t(0),
        content: "1番".to_owned(),
    });
    content.insert(ContentKeyframe {
        t: t(150),
        content: "2番".to_owned(),
    });

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            content,
            ..lyric_document("")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    // **線形補間もイージングも無い**。i/o を持たない Hold のみ(animated-text-document k)。
    assert_eq!(read_back.content.eval(t(0)), "1番");
    assert_eq!(read_back.content.eval(t(75)), "1番", "次のキーまで保持する");
    assert_eq!(read_back.content.eval(t(150)), "2番");
    assert_eq!(read_back.content.eval(t(299)), "2番");
}

// ---------------------------------------------------------------------------
// f / fPath / fFamily / fStyle — フォントは path + 指紋(素材と同じ形)
// ---------------------------------------------------------------------------

#[test]
fn font_ref_is_identified_by_path_and_fingerprint_not_by_name_alone() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞1行目"),
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.styles[0].font.path, "/fonts/NotoSansJP-Bold.otf");
    assert!(read_back.styles[0].font.fingerprint.is_some());
    assert_eq!(read_back.styles[0].font.family, "Noto Sans JP");
    assert_eq!(read_back.styles[0].font.style, "Bold");
}

// ---------------------------------------------------------------------------
// fc / lh / tr / sc / sw / of — スタイル表の既定行(裁定98)
// ---------------------------------------------------------------------------

#[test]
fn document_style_fields_are_the_default_row_not_animated_this_slice() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut style = lyric_style();
    style.fill = [1.0, 0.0, 0.0, 1.0];
    style.line_height = Some(72.0);
    style.tracking = 0.05;
    style.stroke_color = Some([0.0, 0.0, 0.0, 1.0]);
    style.stroke_width = 6.0;
    style.stroke_over_fill = true;

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            styles: vec![style],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.styles[0].fill, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(read_back.styles[0].line_height, Some(72.0));
    assert_eq!(read_back.styles[0].tracking, 0.05);
    assert_eq!(read_back.styles[0].stroke_color, Some([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(read_back.styles[0].stroke_width, 6.0);
    assert!(read_back.styles[0].stroke_over_fill);
}

/// `lh` 未指定はフォントのメトリクスから(store は `None` をそのまま持つだけ — 解決は
/// ラスタライザ側の仕事で、この切片ではやらない)。
#[test]
fn unset_line_height_stays_none_not_a_sentinel() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞"),
    })
    .unwrap();

    assert_eq!(
        doc.view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .styles[0]
            .line_height,
        None
    );
}

// ---------------------------------------------------------------------------
// j / sz — 段落レベルの静止設定(Rive text.alignValue / text.width+height と同型)
// ---------------------------------------------------------------------------

#[test]
fn justify_is_a_static_three_value_enum() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    for justify in [TextJustify::Left, TextJustify::Right, TextJustify::Center] {
        doc.apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                justify,
                ..lyric_document("歌詞")
            },
        })
        .unwrap();
        assert_eq!(
            doc.view().text_document(layer).unwrap().unwrap().justify,
            justify
        );
    }
}

/// `sz` = None は point text(折返し無し)。`Some` は箱の幅・高さ(Rive の `width`/`height`
/// 2成分に対応)。箱幅を動かすと毎フレーム行分割が要るので v1 は静止(地図の note どおり)。
#[test]
fn wrap_size_none_means_point_text_some_means_a_fixed_box() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            wrap_size: None,
            ..lyric_document("歌詞")
        },
    })
    .unwrap();
    assert_eq!(
        doc.view().text_document(layer).unwrap().unwrap().wrap_size,
        None
    );

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            wrap_size: Some([480.0, 200.0]),
            ..lyric_document("歌詞")
        },
    })
    .unwrap();
    assert_eq!(
        doc.view().text_document(layer).unwrap().unwrap().wrap_size,
        Some([480.0, 200.0])
    );
}

// ---------------------------------------------------------------------------
// sid — slots と同じ口([`SlotId`]、`slot` 発注単位が実装した型そのもの)
// ---------------------------------------------------------------------------

#[test]
fn slot_id_is_carried_as_the_shared_slot_id_type() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            slot_id: Some(SlotId("lyric_line_1".to_owned())),
            ..lyric_document("")
        },
    })
    .unwrap();

    assert_eq!(
        doc.view().text_document(layer).unwrap().unwrap().slot_id,
        Some(SlotId("lyric_line_1".to_owned())),
        "text-1 の sid が slot 束の SlotId と別の型になっている(第二の差し替え機構)"
    );
}

// ---------------------------------------------------------------------------
// a / nm / s — text-range(アニメーター)の列。並び=適用順、id は同一性を持つ
// ---------------------------------------------------------------------------

#[test]
fn text_ranges_are_empty_until_set() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞"),
    })
    .unwrap();

    assert!(doc.view().text_document(layer).unwrap().unwrap().ranges.is_empty());
}

#[test]
fn text_ranges_round_trip_with_stable_ids_and_order() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let ranges = vec![karaoke_range(TextRangeId(0)), karaoke_range(TextRangeId(1))];
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: ranges.clone(),
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.ranges, ranges, "並び=適用順がそのまま往復する");
}

/// 同じ id が2枚あると、selector/style の property track の持ち主が決まらない
/// ([`motolii_store::TextRange`] のドキュメント参照)。マスクの
/// `validate_unique_ids` と同型の柵。
#[test]
fn duplicate_text_range_ids_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                ranges: vec![karaoke_range(TextRangeId(0)), karaoke_range(TextRangeId(0))],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("text-range id 0 が2枚ある"));
}

// ---------------------------------------------------------------------------
// b / r / sh / rn — text-range-selector の静止部分
// ---------------------------------------------------------------------------

#[test]
fn selector_static_fields_round_trip() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let range = TextRange {
        id: TextRangeId(0),
        name: "wipe".to_owned(),
        selector: TextRangeSelector {
            based_on: TextBasedOn::Words,
            range_units: TextRangeUnits::Percent,
            shape: TextShape::RampUp,
            randomize: None,
        },
        variation_axes: Vec::new(),
    };
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![range.clone()],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let read_back = &doc.view().text_document(layer).unwrap().unwrap().ranges[0];
    assert_eq!(read_back.selector.based_on, TextBasedOn::Words);
    assert_eq!(read_back.selector.range_units, TextRangeUnits::Percent);
    assert_eq!(read_back.selector.shape, TextShape::RampUp);
    assert_eq!(read_back.selector.randomize, None);
}

/// `rn`(Randomize)は Lottie の int-boolean のままでは Preview=Export と両立しない
/// (裁定75/101)。**有効なら必ず seed を伴う**ことを型で保証する。
#[test]
fn randomize_carries_a_seed_when_enabled_and_is_absent_otherwise() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut range = karaoke_range(TextRangeId(0));
    range.selector.randomize = Some(TextRandomize { seed: 424242 });
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![range],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    assert_eq!(
        doc.view().text_document(layer).unwrap().unwrap().ranges[0]
            .selector
            .randomize,
        Some(TextRandomize { seed: 424242 }),
        "seed は番兵ではなく実値として保たれる"
    );
}

// ---------------------------------------------------------------------------
// s / e / o / a — text-range-selector の動く部分は普通の KeyframeTrack
// ---------------------------------------------------------------------------

/// 「カラオケワイプは Offset を時間駆動するだけに畳める」(地図の note)。
/// Offset がただの静止フィールドでは表現できないことを、実際に動かして確かめる。
#[test]
fn selector_offset_is_an_ordinary_animatable_property() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 300);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let mut wipe = KeyframeTrack::new();
    wipe.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    wipe.insert(Keyframe {
        t: t(150),
        value: Value::F64(100.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::text_range_selector_offset(range_id),
        track: wipe,
    })
    .unwrap();

    let track = doc
        .view()
        .track(layer, &PropertyId::text_range_selector_offset(range_id))
        .unwrap()
        .expect("offset track が読めない");
    assert_eq!(track.eval(t(0)), Value::F64(0.0));
    assert_eq!(track.eval(t(75)), Value::F64(50.0), "線形補間で中間値が出る");
    assert_eq!(track.eval(t(150)), Value::F64(100.0));
}

/// Start/End/Max Amount も同じ形の平坦な property。
#[test]
fn selector_start_end_and_max_amount_are_ordinary_properties() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_selector_start(range_id),
            track: still(Value::F64(0.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_selector_end(range_id),
            track: still(Value::F64(100.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_selector_max_amount(range_id),
            track: still(Value::F64(1.0)),
        },
    ])
    .unwrap();

    let view = doc.view();
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_selector_start(range_id), t(0))
            .unwrap(),
        Some(Value::F64(0.0))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_selector_end(range_id), t(0))
            .unwrap(),
        Some(Value::F64(100.0))
    );
    assert_eq!(
        view.value_at(
            layer,
            &PropertyId::text_range_selector_max_amount(range_id),
            t(0)
        )
        .unwrap(),
        Some(Value::F64(1.0))
    );
}

// ---------------------------------------------------------------------------
// fc / sc / sw / ls / t — text-style(アニメーター側)。track の有無が「触るか」を表す
// ---------------------------------------------------------------------------

/// **track が無い属性は、この animator が触らない属性**(裁定20 の応用)。
/// 一部の property だけ動かす animator を作れることを確かめる。
#[test]
fn animator_style_properties_are_present_only_when_the_animator_touches_them() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    // fill_color だけ触るアニメーター。stroke 系は触らない。
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::text_range_fill_color(range_id),
        track: still(Value::Color([1.0, 1.0, 0.0, 1.0])),
    })
    .unwrap();

    let view = doc.view();
    assert_eq!(
        view.track(layer, &PropertyId::text_range_fill_color(range_id))
            .unwrap()
            .map(|track| track.eval(t(0))),
        Some(Value::Color([1.0, 1.0, 0.0, 1.0]))
    );
    assert_eq!(
        view.track(layer, &PropertyId::text_range_stroke_color(range_id))
            .unwrap(),
        None,
        "触っていない属性は track 自体が無い"
    );
}

/// `ls`(Line Spacing)/`t`(Letter Spacing)。**組版に触るアニメーター**(裁定76)。
/// ここでは値がただの `KeyframeTrack` であることだけ確かめる(実際に組版を
/// 動かすのは engine の仕事、次切片以降)。
#[test]
fn line_spacing_and_tracking_animator_properties_round_trip() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_line_spacing(range_id),
            track: still(Value::F64(12.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_tracking(range_id),
            track: still(Value::F64(0.1)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_stroke_width(range_id),
            track: still(Value::F64(2.0)),
        },
    ])
    .unwrap();

    let view = doc.view();
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_line_spacing(range_id), t(0))
            .unwrap(),
        Some(Value::F64(12.0))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_tracking(range_id), t(0))
            .unwrap(),
        Some(Value::F64(0.1))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_stroke_width(range_id), t(0))
            .unwrap(),
        Some(Value::F64(2.0))
    );
}

// ---------------------------------------------------------------------------
// m — text-alignment-options(グループ内アンカーのオフセットと粒度)
// ---------------------------------------------------------------------------

#[test]
fn alignment_options_default_to_centered_characters() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞"),
    })
    .unwrap();

    let alignment = doc.view().text_document(layer).unwrap().unwrap().alignment;
    assert_eq!(alignment.anchor_offset, [0.0, 0.0], "既定=字面中心");
    assert_eq!(alignment.grouping, TextGrouping::Characters);
}

#[test]
fn alignment_options_round_trip() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            alignment: TextAlignmentOptions {
                anchor_offset: [-50.0, 25.0],
                grouping: TextGrouping::Line,
            },
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let alignment = doc.view().text_document(layer).unwrap().unwrap().alignment;
    assert_eq!(alignment.anchor_offset, [-50.0, 25.0]);
    assert_eq!(alignment.grouping, TextGrouping::Line);
}

// ---------------------------------------------------------------------------
// 保存/読込 — `flattened()`/`save()` は「store に聞く」形なので新 component も自動で運ぶ
// (裁定108(a))。ここでは text 束固有の中身が実際に往復することだけ確かめる。
// ---------------------------------------------------------------------------

#[test]
fn text_document_survives_save_and_load() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞1行目"),
    })
    .unwrap();

    let dir = std::env::temp_dir().join(format!("motolii-text-roundtrip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("text_roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");
    assert_eq!(
        loaded.view().text_document(layer).unwrap(),
        doc.view().text_document(layer).unwrap(),
        "text-layer の中身が保存/読込で往復しない"
    );
}

/// **第2切片固有**: ranges(アニメーターの列)と、その動く量(selector の property
/// track)の両方が保存/読込で往復する。`flattened()` は「store に聞く」形(裁定108(a))
/// なので、`text_range.*` の property component も他の property と同じ経路で運ばれる
/// — ここでは実際に運ばれることを確かめるだけ。
#[test]
fn text_ranges_and_their_property_tracks_survive_save_and_load() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 300);
    let range_id = TextRangeId(0);

    let mut range = karaoke_range(range_id);
    range.selector.randomize = Some(TextRandomize { seed: 7 });
    doc.apply_all([
        Intent::SetTextDocument {
            layer,
            document: TextDocument {
                ranges: vec![range],
                ..lyric_document("歌詞1行目")
            },
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_selector_offset(range_id),
            track: still(Value::F64(50.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_fill_color(range_id),
            track: still(Value::Color([1.0, 0.0, 1.0, 1.0])),
        },
    ])
    .unwrap();

    let dir = std::env::temp_dir().join(format!(
        "motolii-text-range-roundtrip-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("text_range_roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");
    assert_eq!(
        loaded.view().text_document(layer).unwrap(),
        doc.view().text_document(layer).unwrap(),
        "ranges が保存/読込で往復しない"
    );
    assert_eq!(
        loaded
            .view()
            .value_at(layer, &PropertyId::text_range_selector_offset(range_id), t(0))
            .unwrap(),
        Some(Value::F64(50.0)),
        "selector の property track が保存/読込で往復しない"
    );
    assert_eq!(
        loaded
            .view()
            .value_at(layer, &PropertyId::text_range_fill_color(range_id), t(0))
            .unwrap(),
        Some(Value::Color([1.0, 0.0, 1.0, 1.0])),
        "style の property track が保存/読込で往復しない"
    );
}

// ---------------------------------------------------------------------------
// text-modifier-group — アニメーターがグリフに適用する変形。裁定20 の応用で
// track の有無=触るかを表す(text-range-selector/text-style と同じ形)。
// ---------------------------------------------------------------------------

/// origin/opacity/position/rotation/scale は普通の平坦 property。
/// x/y・scaleX/scaleY は Rive の `"group"` 注記どおり Vec2 1個(裁定61 と衝突しない)。
#[test]
fn transform_group_properties_are_ordinary_animatable_vec2_and_scalar_tracks() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_origin(range_id),
            track: still(Value::Vec2([0.5, 0.5])),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_position(range_id),
            track: still(Value::Vec2([10.0, -4.0])),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_scale(range_id),
            track: still(Value::Vec2([1.5, 1.5])),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_rotation(range_id),
            track: still(Value::F64(45.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_opacity(range_id),
            track: still(Value::F64(0.5)),
        },
    ])
    .unwrap();

    let view = doc.view();
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_origin(range_id), t(0))
            .unwrap(),
        Some(Value::Vec2([0.5, 0.5]))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_position(range_id), t(0))
            .unwrap(),
        Some(Value::Vec2([10.0, -4.0]))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_scale(range_id), t(0))
            .unwrap(),
        Some(Value::Vec2([1.5, 1.5]))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_rotation(range_id), t(0))
            .unwrap(),
        Some(Value::F64(45.0))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_opacity(range_id), t(0))
            .unwrap(),
        Some(Value::F64(0.5))
    );
}

/// **触っていない変形は track 自体が無い**(`modifierFlags` を第二の帳簿として持たない
/// — 地図の note どおり、track の有無自体が有効フラグ)。
#[test]
fn transform_group_properties_are_present_only_when_the_animator_touches_them() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![karaoke_range(range_id)],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::text_range_opacity(range_id),
        track: still(Value::F64(0.2)),
    })
    .unwrap();

    let view = doc.view();
    assert!(view
        .track(layer, &PropertyId::text_range_opacity(range_id))
        .unwrap()
        .is_some());
    assert_eq!(
        view.track(layer, &PropertyId::text_range_position(range_id))
            .unwrap(),
        None,
        "位置を触っていないアニメーターは position track を持たない"
    );
}

// ---------------------------------------------------------------------------
// styles / runs — スタイル表 + 分割(裁定85/87/88/89)
// ---------------------------------------------------------------------------

#[test]
fn styles_default_to_a_single_default_row_and_runs_default_to_empty() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: lyric_document("歌詞"),
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.styles.len(), 1, "裁定98: 既定行が1枚だけある");
    assert_eq!(read_back.styles[0].id, TextStyleId(0));
    assert!(
        read_back.runs.is_empty(),
        "runs 空 = 既定行が本文全体を覆う(ranges 空と同じ形、裁定20 の応用)"
    );
}

/// 2行のスタイル表 + それを覆う runs が並びのまま往復する。
#[test]
fn styles_and_runs_round_trip_and_cover_the_content_without_gaps() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut second = lyric_style();
    second.id = TextStyleId(1);
    second.fill = [1.0, 0.0, 0.0, 1.0];

    let runs = vec![
        TextRun {
            len: 2,
            style: TextStyleId(0),
        },
        TextRun {
            len: 3,
            style: TextStyleId(1),
        },
    ];
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            styles: vec![lyric_style(), second],
            runs: runs.clone(),
            ..lyric_document("歌詞1行目")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.styles.len(), 2);
    assert_eq!(read_back.runs, runs, "並びと len がそのまま往復する");
}

/// 同じ id のスタイル行が2枚あると `text_style.{id}.axis.{tag}` の track や
/// [`TextRun::style`] の参照先が決まらない。
#[test]
fn duplicate_text_style_ids_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![lyric_style(), lyric_style()],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("text-style id 0 が2枚ある"));
}

/// run が存在しない styleId を指すと、どのスタイルも指さない宙ぶらりんの参照になる
/// (裁定37「無いと読めないを区別する」— 書けてしまうと後で気付けない)。
#[test]
fn a_run_referencing_a_missing_style_id_is_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                runs: vec![TextRun {
                    len: 3,
                    style: TextStyleId(99),
                }],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("styleId 99"));
}

/// `len == 0` の run は本文を1文字も覆わない空の分割で、意味を持たない。
#[test]
fn a_zero_length_run_is_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                runs: vec![TextRun {
                    len: 0,
                    style: TextStyleId(0),
                }],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("len が 0"));
}

/// **裁定89: 隣接同値ランの併合は最適化ではなく正しさの契約** — 併合しないと
/// 合字/カーニングがスパン境界で切れ、意味が同じ2つの Document が違う画を出す。
#[test]
fn adjacent_runs_pointing_at_the_same_style_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                runs: vec![
                    TextRun {
                        len: 2,
                        style: TextStyleId(0),
                    },
                    TextRun {
                        len: 3,
                        style: TextStyleId(0),
                    },
                ],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("隣接"));
}

// ---------------------------------------------------------------------------
// text-style-axis / text-variation-modifier — 可変フォント軸。裁定92 の唯一の例外
// (裁定93) — 軸値だけはスタイル層/アニメーター層でアニメ可。
// ---------------------------------------------------------------------------

/// tag は静止(どの軸に乗るか)、値(axisValue)は普通の `KeyframeTrack`。
#[test]
fn style_axis_tag_is_static_but_its_value_is_an_animatable_track() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut style = lyric_style();
    style.axes = vec![TextStyleAxis {
        tag: "wght".to_owned(),
    }];
    doc.apply_all([
        Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![style],
                ..lyric_document("歌詞")
            },
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_style_axis(TextStyleId(0), "wght"),
            track: still(Value::F64(700.0)),
        },
    ])
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.styles[0].axes[0].tag, "wght");
    assert_eq!(
        doc.view()
            .value_at(layer, &PropertyId::text_style_axis(TextStyleId(0), "wght"), t(0))
            .unwrap(),
        Some(Value::F64(700.0))
    );
}

#[test]
fn duplicate_axis_tags_within_one_style_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut style = lyric_style();
    style.axes = vec![
        TextStyleAxis {
            tag: "wght".to_owned(),
        },
        TextStyleAxis {
            tag: "wght".to_owned(),
        },
    ];
    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![style],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("axis タグ「wght」が2枚ある"));
}

/// **裁定76 の3層のうち「再シェープする層」の唯一の住人**。axisTag は静止、
/// Δ値(axisValue)は普通の `KeyframeTrack`。
#[test]
fn variation_modifier_axis_tag_is_static_but_its_delta_is_an_animatable_track() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 300);
    let range_id = TextRangeId(0);

    let mut range = karaoke_range(range_id);
    range.variation_axes = vec![TextVariationAxis {
        tag: "wght".to_owned(),
    }];
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            ranges: vec![range],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let mut delta = KeyframeTrack::new();
    delta.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    delta.insert(Keyframe {
        t: t(150),
        value: Value::F64(300.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::text_range_variation_axis(range_id, "wght"),
        track: delta,
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.ranges[0].variation_axes[0].tag, "wght");
    let view = doc.view();
    assert_eq!(
        view.value_at(
            layer,
            &PropertyId::text_range_variation_axis(range_id, "wght"),
            t(75)
        )
        .unwrap(),
        Some(Value::F64(150.0)),
        "線形補間で中間の Δ が出る"
    );
}

#[test]
fn duplicate_variation_axis_tags_within_one_range_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);

    let mut range = karaoke_range(range_id);
    range.variation_axes = vec![
        TextVariationAxis {
            tag: "wght".to_owned(),
        },
        TextVariationAxis {
            tag: "wght".to_owned(),
        },
    ];
    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                ranges: vec![range],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("variation axis タグ「wght」が2枚ある"));
}

/// スパン側の絶対値とアニメーター側のΔは**別 track**(層が違うので二重帳簿ではない、
/// 地図の note どおり) — 同じタグ "wght" でも独立に読み書きできることを確かめる。
#[test]
fn style_axis_and_range_variation_axis_use_independent_tracks_for_the_same_tag() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);
    let range_id = TextRangeId(0);

    let mut style = lyric_style();
    style.axes = vec![TextStyleAxis {
        tag: "wght".to_owned(),
    }];
    let mut range = karaoke_range(range_id);
    range.variation_axes = vec![TextVariationAxis {
        tag: "wght".to_owned(),
    }];

    doc.apply_all([
        Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![style],
                ranges: vec![range],
                ..lyric_document("歌詞")
            },
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_style_axis(TextStyleId(0), "wght"),
            track: still(Value::F64(400.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_variation_axis(range_id, "wght"),
            track: still(Value::F64(100.0)),
        },
    ])
    .unwrap();

    let view = doc.view();
    assert_eq!(
        view.value_at(layer, &PropertyId::text_style_axis(TextStyleId(0), "wght"), t(0))
            .unwrap(),
        Some(Value::F64(400.0)),
        "スパンの絶対値"
    );
    assert_eq!(
        view.value_at(
            layer,
            &PropertyId::text_range_variation_axis(range_id, "wght"),
            t(0)
        )
        .unwrap(),
        Some(Value::F64(100.0)),
        "アニメーターのΔ(独立)"
    );
}

// ---------------------------------------------------------------------------
// text-style-feature — OpenType feature。**v1 は静止**(裁定92のまま)。
// ---------------------------------------------------------------------------

/// 地図の穴を塞ぐ行(裁定99): `text-document ca`(Small Caps)不採用が「正本は
/// OpenType feature」と書いたのに置き場が無かった。ここがその置き場。
#[test]
fn style_features_round_trip_as_static_settings() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut style = lyric_style();
    style.features = vec![
        TextStyleFeature {
            tag: "palt".to_owned(),
            value: 1,
        },
        TextStyleFeature {
            tag: "smcp".to_owned(),
            value: 1,
        },
    ];
    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            styles: vec![style],
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(
        read_back.styles[0].features,
        vec![
            TextStyleFeature {
                tag: "palt".to_owned(),
                value: 1,
            },
            TextStyleFeature {
                tag: "smcp".to_owned(),
                value: 1,
            },
        ]
    );
}

#[test]
fn duplicate_feature_tags_within_one_style_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    let mut style = lyric_style();
    style.features = vec![
        TextStyleFeature {
            tag: "palt".to_owned(),
            value: 1,
        },
        TextStyleFeature {
            tag: "palt".to_owned(),
            value: 0,
        },
    ];
    let err = doc
        .apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![style],
                ..lyric_document("歌詞")
            },
        })
        .unwrap_err();
    assert!(format!("{err}").contains("feature タグ「palt」が2枚ある"));
}

// ---------------------------------------------------------------------------
// 保存/読込 — 第3切片固有: styles/runs と、その新しい property track
// ---------------------------------------------------------------------------

#[test]
fn styles_runs_and_their_property_tracks_survive_save_and_load() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 300);
    let range_id = TextRangeId(0);

    let mut style0 = lyric_style();
    style0.axes = vec![TextStyleAxis {
        tag: "wght".to_owned(),
    }];
    style0.features = vec![TextStyleFeature {
        tag: "palt".to_owned(),
        value: 1,
    }];
    let mut style1 = lyric_style();
    style1.id = TextStyleId(1);
    style1.fill = [1.0, 0.0, 0.0, 1.0];

    let mut range = karaoke_range(range_id);
    range.variation_axes = vec![TextVariationAxis {
        tag: "wght".to_owned(),
    }];

    doc.apply_all([
        Intent::SetTextDocument {
            layer,
            document: TextDocument {
                styles: vec![style0, style1],
                runs: vec![
                    TextRun {
                        len: 2,
                        style: TextStyleId(0),
                    },
                    TextRun {
                        len: 3,
                        style: TextStyleId(1),
                    },
                ],
                ranges: vec![range],
                ..lyric_document("歌詞1行目")
            },
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_style_axis(TextStyleId(0), "wght"),
            track: still(Value::F64(650.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_variation_axis(range_id, "wght"),
            track: still(Value::F64(50.0)),
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::text_range_position(range_id),
            track: still(Value::Vec2([0.0, -8.0])),
        },
    ])
    .unwrap();

    let dir = std::env::temp_dir().join(format!(
        "motolii-text-style-span-roundtrip-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("text_style_span_roundtrip.motolii");
    doc.save(&path).expect("保存できない");

    let loaded = Document::load(&path).expect("読み込めない");
    assert_eq!(
        loaded.view().text_document(layer).unwrap(),
        doc.view().text_document(layer).unwrap(),
        "styles/runs/variation_axes が保存/読込で往復しない"
    );
    let view = loaded.view();
    assert_eq!(
        view.value_at(layer, &PropertyId::text_style_axis(TextStyleId(0), "wght"), t(0))
            .unwrap(),
        Some(Value::F64(650.0))
    );
    assert_eq!(
        view.value_at(
            layer,
            &PropertyId::text_range_variation_axis(range_id, "wght"),
            t(0)
        )
        .unwrap(),
        Some(Value::F64(50.0))
    );
    assert_eq!(
        view.value_at(layer, &PropertyId::text_range_position(range_id), t(0))
            .unwrap(),
        Some(Value::Vec2([0.0, -8.0]))
    );
}
