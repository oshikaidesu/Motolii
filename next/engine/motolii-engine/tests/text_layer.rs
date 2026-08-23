//! text layer の `render_frame` 結線(裁定190 切片3)の通し試験。
//!
//! `crate::text::rasterize_text_document` 自体の単体試験(英字/日本語で非空画素、
//! lh/tr の実測)は `src/text.rs` の `#[cfg(test)]` が既に縛っている。ここで
//! 固定するのは **`render_frame` からその関数へ実際に届くこと**——
//! `LayerSource::Text` は中身を持たない unit variant なので、`ResolvedLayer.id`
//! (BL4)から `StoreView::text_document` を引く配線(`Engine::text_texture_for`)が
//! 実際に通ることと、その texture cache(`TextCacheKey`、`lib.rs` 参照)が正しく
//! 効くこと(内容が変わらなければ再利用・変われば新規)。

use motolii_engine::Engine;
use motolii_store::{
    Composition, ContentKeyframe, ContentTrack, Document, FontRef, Fps, Intent, Interp, Keyframe,
    KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime,
    TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId, Value,
};

const ARIAL: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
const HIRAGINO: &str = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc";

const W: u32 = 512;
const H: u32 = 200;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn place_text_layer(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Text,
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn style(font_path: &str, family: &str, size: f32, fill: [f64; 4]) -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef {
            path: font_path.to_owned(),
            fingerprint: None,
            family: family.to_owned(),
            style: "Regular".to_owned(),
        },
        size,
        fill,
        line_height: None,
        tracking: 0.0,
        stroke_color: None,
        stroke_width: 0.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

fn document_with(content: &str, style_row: TextDocumentStyle) -> TextDocument {
    let mut content_track = ContentTrack::new();
    content_track.insert(ContentKeyframe {
        t: t(0),
        content: content.to_owned(),
    });
    TextDocument {
        content: content_track,
        justify: TextJustify::Left,
        wrap_size: None,
        styles: vec![style_row],
        slot_id: None,
        ranges: Vec::new(),
        alignment: TextAlignmentOptions::default(),
        runs: Vec::new(),
    }
}

/// 背景は不透明黒(`doc_with_comp` の既定)なので、赤か青チャンネルが立っている画素を
/// 「文字が出た画素」として数える(白文字は使わない——白は背景の黒とだけ違うので
/// 判定できるが、赤/青にしておけば「matte の漏れ検査」等の他の試験と同じ手口で
/// 読める)。
fn colored_pixel_count(frame: &[u8]) -> usize {
    frame
        .chunks_exact(4)
        .filter(|p| p[0] > 40 || p[2] > 40)
        .count()
}

/// **英字**。`render_frame` → `Engine::texture_for_layer` → `Engine::text_texture_for`
/// → `crate::text::rasterize_text_document` の配線が実際に画素を出すことを確かめる
/// (`text.rs::text_layer_produces_non_empty_pixels_english` の render_frame 版)。
#[test]
fn text_layer_renders_visible_pixels_through_render_frame_english() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("Motolii", style(ARIAL, "Arial", 96.0, [1.0, 0.0, 0.0, 1.0])),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert!(
        colored_pixel_count(&frame) > 500,
        "英字テキストが render_frame の出力に画素として出ているはず"
    );
}

/// **日本語**(`text.rs::text_layer_produces_non_empty_pixels_japanese` の
/// render_frame 版)。
#[test]
fn text_layer_renders_visible_pixels_through_render_frame_japanese() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with(
            "文字が画素になる",
            style(HIRAGINO, "Hiragino Sans", 64.0, [0.0, 0.0, 1.0, 1.0]),
        ),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert!(
        colored_pixel_count(&frame) > 500,
        "日本語テキストが render_frame の出力に画素として出ているはず"
    );
}

/// `SetTextDocument` を一度も呼んでいない text layer は「無い」(エラーではない、
/// `StoreView::text_document` の doc 参照)——`render_frame` 自体は普通に成功し、
/// 何も描かれない。
#[test]
fn text_layer_with_no_document_renders_nothing_but_does_not_error() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert_eq!(
        colored_pixel_count(&frame),
        0,
        "文書が無い text layer は何も描かないはず"
    );
    assert_eq!(
        engine.cached_text_texture_count(),
        0,
        "描く物が無い時は text texture キャッシュへ何も積まれないはず"
    );
}

/// **キャッシュ鍵の設計(`TextCacheKey`)の直接固定**: 同じ内容の text layer を
/// 同じ Hold 区間内の別フレームで描いても、キャッシュ行は増えない(= texture が
/// 再利用される)。`content` は Hold 評価(裁定92)なので、キーフレームが1本だけの
/// この document は t(0) でも t(5) でも同じ評価結果になる——`TextCacheKey` が
/// `t` そのものではなく評価後の内容を鍵にしていることの直接証拠。
#[test]
fn text_texture_cache_reuses_same_content_across_frames_within_a_hold_span() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("Motolii", style(ARIAL, "Arial", 64.0, [1.0, 1.0, 1.0, 1.0])),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(engine.cached_text_texture_count(), 1);

    engine.render_frame(&doc.view(), t(5)).expect("render");
    assert_eq!(
        engine.cached_text_texture_count(),
        1,
        "同じ内容の別フレームはキャッシュを再利用するはず(新規エントリが増えてはいけない)"
    );
}

/// 内容が変わればキャッシュ行が増える(再利用ではなく再ラスタライズが起きる)——
/// 上の試験と対になる、`TextCacheKey` が中身の変化を正しく拾うことの固定。
#[test]
fn text_texture_cache_grows_when_content_changes() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("A", style(ARIAL, "Arial", 64.0, [1.0, 1.0, 1.0, 1.0])),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(engine.cached_text_texture_count(), 1);

    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("B", style(ARIAL, "Arial", 64.0, [1.0, 1.0, 1.0, 1.0])),
    })
    .unwrap();
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(
        engine.cached_text_texture_count(),
        2,
        "内容が変わったら新しいキャッシュ行が増えるはず"
    );
}

// ---------------------------------------------------------------------------
// A-1b(裁定214 同日訂正版): `text_style.*` track が実際に画素を変える。
//
// A-1 の実測(`view.rs::text_document` doc 参照)は「store に `PropertyId::
// text_style_*` は在るが、`StoreView::text_document` が丸ごと static
// deserialize するだけで track を一切読まない」だった——`bm`/`matte`/`ao` と
// 同型の「在るが未消費」。この2試験は**その穴が実際に閉じたこと**を画素で
// 固定する: track を書く前は静的値の画、書いた後は track の値の画になる
// (`StoreView::resolved_text_document`/`Engine::text_texture_for` 経由)。
// この2試験は本発注の前(`resolved_text_document` 追加前)は共に赤だった
// (`view.text_document` のまま静的値しか読まないので track の色/寸法が
// 画に反映されない)。
// ---------------------------------------------------------------------------

fn red_pixel_count(frame: &[u8]) -> usize {
    frame
        .chunks_exact(4)
        .filter(|p| p[0] > 40 && p[2] <= 40)
        .count()
}

fn blue_pixel_count(frame: &[u8]) -> usize {
    frame
        .chunks_exact(4)
        .filter(|p| p[2] > 40 && p[0] <= 40)
        .count()
}

/// `PropertyId::text_style_fill_color` の track が静的 `fill`(赤)を上書きして
/// 画素の色そのものを変える。
#[test]
fn text_style_fill_color_track_overrides_the_static_fill_and_changes_pixel_color() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("Motolii", style(ARIAL, "Arial", 96.0, [1.0, 0.0, 0.0, 1.0])),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let red_frame = engine.render_frame(&doc.view(), t(0)).expect("render");
    assert!(
        red_pixel_count(&red_frame) > 500,
        "track を書く前は静的値どおり赤で出ているはず"
    );

    let property = PropertyId::text_style_fill_color(TextStyleId(0));
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::Color([0.0, 0.0, 1.0, 1.0]),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track,
    })
    .unwrap();

    let blue_frame = engine.render_frame(&doc.view(), t(0)).expect("render");
    assert!(
        blue_pixel_count(&blue_frame) > 500,
        "text_style_fill_color の track が実際の画素色に反映されているはず"
    );
    assert_eq!(
        red_pixel_count(&blue_frame),
        0,
        "track を書いた後は静的値(赤)の画素が残っていてはいけない(残っていれば\
         track が評価側で未消費のまま = A-1 の穴が閉じていない証拠)"
    );
}

/// `PropertyId::text_style_size` の track が静的 `size` を上書きして文字の
/// 大きさ(=色付き画素数)そのものを変える。
#[test]
fn text_style_size_track_overrides_the_static_size_and_changes_pixel_count() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with("Motolii", style(ARIAL, "Arial", 32.0, [1.0, 1.0, 1.0, 1.0])),
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let small_frame = engine.render_frame(&doc.view(), t(0)).expect("render");
    let small_count = colored_pixel_count(&small_frame);
    assert!(small_count > 100, "静的 size(32)でもまず画素が出ているはず");

    let property = PropertyId::text_style_size(TextStyleId(0));
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(128.0),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track,
    })
    .unwrap();

    let large_frame = engine.render_frame(&doc.view(), t(0)).expect("render");
    let large_count = colored_pixel_count(&large_frame);
    assert!(
        large_count > small_count * 2,
        "text_style_size の track(128)が静的値(32)より大きく画素数へ反映\
         されているはず(small={small_count} large={large_count})"
    );
}
