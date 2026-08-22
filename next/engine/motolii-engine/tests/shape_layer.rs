//! shape layer の `render_frame` 結線(発注「シェイプが画に出るようにする」、
//! 2026-08-22)の通し試験。
//!
//! `crate::shape::rasterize_shapes` 自体の単体試験(矩形/楕円で非空画素、
//! fill/stroke の実測、`Canvas::centered` の証拠)は `src/shape.rs` の
//! `#[cfg(test)]` が既に縛っている。ここで固定するのは
//! **`render_frame` からその関数へ実際に届くこと**——`text_layer.rs` の
//! shape 版(同じ形の通し試験)。

use motolii_engine::Engine;
use motolii_store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Shape, ShapeNode, Value,
};
use motolii_vector::{Brush, Fill, FillRule, PathSource, Point as VPoint, Rgb, StarType, Stroke};

const W: u32 = 128;
const H: u32 = 128;

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

fn place_shape_layer(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Shape,
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn red_rect(size: f64) -> Shape {
    Shape {
        source: PathSource::Rectangle {
            size: VPoint { x: size, y: size },
        },
        ops: Vec::new(),
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }),
        stroke: None,
    }
}

fn blue_stroked_ellipse(size: f64, width: f64) -> Shape {
    Shape {
        source: PathSource::Ellipse {
            size: VPoint { x: size, y: size },
        },
        ops: Vec::new(),
        fill: None,
        stroke: Some(Stroke {
            brush: Brush::Solid(Rgb {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            }),
            width,
            opacity: 1.0,
            ..Stroke::default()
        }),
    }
}

/// 背景は不透明黒(`doc_with_comp` の既定)なので、赤か青チャンネルが立っている画素を
/// 「shape が出た画素」として数える(`text_layer.rs::colored_pixel_count` と同じ手口)。
fn colored_pixel_count(frame: &[u8]) -> usize {
    frame
        .chunks_exact(4)
        .filter(|p| p[0] > 40 || p[2] > 40)
        .count()
}

/// **矩形**。`render_frame` → `Engine::texture_for_layer` → `Engine::shape_texture_for`
/// → `crate::shape::rasterize_shapes` の配線が実際に画素を出すことを確かめる。
#[test]
fn rectangle_shape_renders_visible_pixels_through_render_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(60.0))],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert!(
        colored_pixel_count(&frame) > 500,
        "矩形シェイプが render_frame の出力に画素として出ているはず"
    );
}

/// **楕円 + 塗りなし線のみ**。塗りが無くても stroke だけで画素が出ることを確かめる。
#[test]
fn ellipse_shape_with_stroke_only_renders_visible_pixels_through_render_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(blue_stroked_ellipse(60.0, 6.0))],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert!(
        colored_pixel_count(&frame) > 50,
        "stroke だけの楕円シェイプが render_frame の出力に画素として出ているはず"
    );
}

/// 塗り色が実際に画素チャンネルへ届いている(赤 fill が赤チャンネルとして出る)。
#[test]
fn fill_color_reaches_the_rendered_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(60.0))],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");
    let center = ((H / 2 * W + W / 2) * 4) as usize;
    assert!(
        frame[center] > 200 && frame[center + 1] < 20 && frame[center + 2] < 20,
        "comp 中央は赤 fill で塗られているはず: {:?}",
        &frame[center..center + 4]
    );
}

/// `SetShapes` を一度も呼んでいない(または空配列)shape layer は「無い」
/// (エラーではない、`StoreView::shapes` の doc 参照)——`render_frame` 自体は
/// 普通に成功し、何も描かれない。
#[test]
fn shape_layer_with_no_shapes_renders_nothing_but_does_not_error() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert_eq!(
        colored_pixel_count(&frame),
        0,
        "shapes が無い shape layer は何も描かないはず"
    );
    assert_eq!(
        engine.cached_shape_texture_count(),
        0,
        "描く物が無い時は shape texture キャッシュへ何も積まれないはず"
    );
}

/// **キャッシュ鍵の設計(`ShapeCacheKey`)の直接固定**: 同じ内容の shape layer を
/// 別フレーム(別の `t`)で描いても、キャッシュ行は増えない(= texture が
/// 再利用される)——`ShapeCacheKey` が `t` を鍵に含めないことの直接証拠
/// (`text_texture_cache_reuses_same_content_across_frames_within_a_hold_span` の
/// shape 版)。
#[test]
fn shape_texture_cache_reuses_same_content_across_different_frames() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(40.0))],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(engine.cached_shape_texture_count(), 1);

    engine.render_frame(&doc.view(), t(5)).expect("render");
    assert_eq!(
        engine.cached_shape_texture_count(),
        1,
        "同じ内容の別フレームはキャッシュを再利用するはず(新規エントリが増えてはいけない)"
    );
}

/// 内容が変わればキャッシュ行が増える(再利用ではなく再ラスタライズが起きる)——
/// 上の試験と対になる、`ShapeCacheKey` が中身の変化を正しく拾うことの固定
/// (`text_texture_cache_grows_when_content_changes` の shape 版)。
#[test]
fn shape_texture_cache_grows_when_content_changes() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(40.0))],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(engine.cached_shape_texture_count(), 1);

    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(80.0))],
    })
    .unwrap();
    engine.render_frame(&doc.view(), t(0)).expect("render");
    assert_eq!(
        engine.cached_shape_texture_count(),
        2,
        "内容が変わったら新しいキャッシュ行が増えるはず"
    );
}

/// **「時刻で頂点が動く」**: shape 自身は時間評価を持たないが(`shape.rs` module
/// doc 参照)、layer の transform(`POSITION` track、既存のキーフレーム機構)は
/// 他の layer 種別と同じように shape layer にも効く——`frame.rs::keyframes_
/// move_the_picture_over_time` と同じ手口で、shape が実際に画面上を動くことを
/// 固定する。
#[test]
fn keyframed_position_moves_the_rendered_shape_over_time() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(30.0))],
    })
    .unwrap();

    // shape の板は comp 全域(128x128、`Canvas::centered` で局所原点は板の中心
    // (64,64))なので、既定の position([0,0])では板がそのまま comp に重なり、
    // 30x30 の矩形(半径15)は comp 中心に居る。position を x 方向へ ±30 動かすと、
    // 矩形も comp 内(幅128)に収まったまま左右へ動く
    // (±30 ± 半径15 = [19,49]/[79,109] — どちらも [0,128) の内側)。
    let mut position = KeyframeTrack::new();
    position.insert(Keyframe {
        t: t(0),
        value: Value::Vec2([-30.0, 0.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    position.insert(Keyframe {
        t: t(30),
        value: Value::Vec2([30.0, 0.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION).unwrap(),
        track: position,
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let at_start = engine.render_frame(&doc.view(), t(0)).unwrap();
    let at_end = engine.render_frame(&doc.view(), t(30)).unwrap();

    assert_ne!(
        at_start, at_end,
        "position track が動けば shape の絵も動くはず(動画ソフトの芯、frame.rs と同じ主張)"
    );
    assert!(
        colored_pixel_count(&at_start) > 0,
        "0フレームでも shape の画素は出ているはず"
    );
    assert!(
        colored_pixel_count(&at_end) > 0,
        "30フレームでも shape の画素は出ているはず"
    );

    // 具体的な画素位置でも「動いた」ことを固定する — comp 中心 y=64 の行で、
    // 0フレームは左寄り(x=34 付近)、30フレームは右寄り(x=94 付近)に赤が居る。
    let pixel = |frame: &[u8], x: u32, y: u32| -> [u8; 4] {
        let i = ((y * W + x) * 4) as usize;
        [frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]
    };
    assert!(
        pixel(&at_start, 34, 64)[0] > 200,
        "0フレームでは矩形が左寄り(x=34)に居るはず: {:?}",
        pixel(&at_start, 34, 64)
    );
    assert!(
        pixel(&at_start, 94, 64)[0] < 20,
        "0フレームでは矩形はまだ右寄り(x=94)に居ないはず: {:?}",
        pixel(&at_start, 94, 64)
    );
    assert!(
        pixel(&at_end, 94, 64)[0] > 200,
        "30フレームでは矩形が右寄り(x=94)に居るはず: {:?}",
        pixel(&at_end, 94, 64)
    );
    assert!(
        pixel(&at_end, 34, 64)[0] < 20,
        "30フレームでは矩形はもう左寄り(x=34)に居ないはず: {:?}",
        pixel(&at_end, 34, 64)
    );
}

/// 星形(`PathSource::PolyStar`)も同じ経路で描けることの確認 — `motolii-vector`
/// の3つのパス源(`Rectangle`/`Ellipse`/`PolyStar`)のうち矩形・楕円は上の試験で
/// 押さえたので、ここで星形も engine 経由で非空画素になることを確かめておく
/// (パス源の種類で分岐が漏れていないことの oracle)。
#[test]
fn polystar_shape_renders_visible_pixels_through_render_frame() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    let star = Shape {
        source: PathSource::PolyStar {
            points: 5.0,
            outer_radius: 40.0,
            inner_radius: 20.0,
            star_type: StarType::Star,
        },
        ops: Vec::new(),
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }),
        stroke: None,
    };
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(star)],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).expect("render");

    assert!(
        colored_pixel_count(&frame) > 500,
        "星形シェイプが render_frame の出力に画素として出ているはず"
    );
}
