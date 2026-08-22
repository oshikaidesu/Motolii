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
    Composition, ContentKeyframe, ContentTrack, Document, FontRef, Fps, Intent, LayerId,
    LayerMeta, LayerSource, LayerTiming, RationalTime, TextAlignmentOptions, TextDocument,
    TextDocumentStyle, TextJustify, TextStyleId,
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
