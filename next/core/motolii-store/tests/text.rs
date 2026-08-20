//! text 束(75行)の第1切片 — **静的組版の Document 意味**。
//!
//! ここで固定するのは:
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

use motolii_store::{
    Composition, ContentKeyframe, ContentTrack, Document, FontRef, Fps, Intent, LayerId, LayerMeta,
    LayerSource, LayerTiming, RationalTime, TextDocument, TextDocumentStyle, TextJustify,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
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

/// 分かりやすい既定スタイル(歌詞1行を想定)。
fn lyric_style() -> TextDocumentStyle {
    TextDocumentStyle {
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
        style: lyric_style(),
        slot_id: None,
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
    assert_eq!(read_back.style.font.path, "/fonts/NotoSansJP-Bold.otf");
    assert!(read_back.style.font.fingerprint.is_some());
    assert_eq!(read_back.style.font.family, "Noto Sans JP");
    assert_eq!(read_back.style.font.style, "Bold");
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
            style,
            ..lyric_document("歌詞")
        },
    })
    .unwrap();

    let read_back = doc.view().text_document(layer).unwrap().unwrap();
    assert_eq!(read_back.style.fill, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(read_back.style.line_height, Some(72.0));
    assert_eq!(read_back.style.tracking, 0.05);
    assert_eq!(read_back.style.stroke_color, Some([0.0, 0.0, 0.0, 1.0]));
    assert_eq!(read_back.style.stroke_width, 6.0);
    assert!(read_back.style.stroke_over_fill);
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
            .style
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
// sid — slots と同じ口(まだ解決しない、参照識別子だけ持てる)
// ---------------------------------------------------------------------------

#[test]
fn slot_id_is_carried_but_not_resolved_this_slice() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer, 0, 100);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: TextDocument {
            slot_id: Some("lyric_line_1".to_owned()),
            ..lyric_document("")
        },
    })
    .unwrap();

    assert_eq!(
        doc.view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .slot_id
            .as_deref(),
        Some("lyric_line_1")
    );
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
