//! 3 つの札の法(2026-09-12): 2D は世界に居ない(深度に参加せず積み順だけ)、2.5D は z で並ぶ、
//! 既定カメラでは 3 つの札が同じ場所に映る。
use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::*;
use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
use crate::render::engine::Engine;

fn square(color: Rgb, size: f64) -> ShapeNode {
    ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(color), ..Default::default() }) })
}
fn document() -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: 256, height: 256, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [1.0, 1.0, 1.0, 1.0] })).unwrap();
    doc
}
fn layer(doc: &mut Document, id: u64, order: i16, color: Rgb, projection: LayerProjection, position: [f64; 2], z: f64) {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetShapes { layer, shapes: vec![square(color, 80.0)] },
        Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(projection), ..Default::default() } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(position) },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION_Z).unwrap(), value: Value::F64(z) },
    ]).unwrap();
}
fn count(px: &[u8], red: bool) -> usize {
    px.chunks_exact(4).filter(|p| if red { p[0] > 128 && p[2] < 100 } else { p[2] > 128 && p[0] < 100 }).count()
}

/// 下の段に手前(z=-200)の 3D の板、上の段に 2D の板。2D は積み順で勝ち、3D に隠されない。
/// 同じ配置で上の段が 2.5D(z=0)なら、深度で 3D の奥になる。
#[test]
fn a_two_d_layer_above_in_the_stack_is_never_hidden_by_closer_three_d() {
    let mut engine = Engine::new().unwrap();
    let mut doc = document();
    layer(&mut doc, 1, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::ThreeD, [128.0, 128.0], -200.0);
    layer(&mut doc, 2, 1, Rgb { r: 0.0, g: 0.2, b: 1.0 }, LayerProjection::TwoD, [148.0, 148.0], 0.0);
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(count(&px, false) > 6000, "the 2D layer on top must be fully visible: {} blue pixels", count(&px, false));
    assert!(count(&px, true) > 1000, "the 3D layer still shows around it");
    doc.apply(Intent::SetAttrs { layer: LayerId(2), patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoPointFiveD), ..Default::default() } }).unwrap();
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(count(&px, false) < 100, "a 2.5D layer at z=0 sits behind a 3D layer at z=-200: {} blue pixels", count(&px, false));
}

/// カメラの Near Fade: 奥行きがその 1/3 より近い 2.5D・3D の物は消え、2D は画面の物なので残る。0 なら何もしない。
#[test]
fn the_camera_near_fade_hides_close_world_layers_but_not_two_d() {
    let mut engine = Engine::new().unwrap();
    let mut doc = document();
    layer(&mut doc, 1, 0, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::ThreeD, [90.0, 128.0], 0.0);
    layer(&mut doc, 2, 1, Rgb { r: 0.0, g: 0.2, b: 1.0 }, LayerProjection::TwoD, [170.0, 128.0], 0.0);
    doc.apply_all([
        Intent::AddLayer(LayerId(9)),
        Intent::SetMeta { layer: LayerId(9), meta: LayerMeta { source: LayerSource::Camera, order: 2, timing: LayerTiming::place(0, None, 1) } },
    ]).unwrap();
    let fade = |doc: &mut Document, distance: f64| doc.apply(Intent::SetConstant { layer: LayerId(9), property: PropertyId::new(property::CAMERA_NEAR_FADE).unwrap(), value: Value::F64(distance) }).unwrap();
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    let (red, blue) = (count(&px, true), count(&px, false));
    assert!(red > 4000 && blue > 4000, "no fade by default: red {red} blue {blue}");
    // 既定の距離は 256 の comp で約 246。1000 の 1/3 = 333 より近いので消える。
    fade(&mut doc, 1000.0);
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(count(&px, true) < 50, "the 3D layer fades out: {} red pixels", count(&px, true));
    assert!(count(&px, false) > 4000, "2D is not in the world: {} blue pixels", count(&px, false));
    layer(&mut doc, 3, 3, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::TwoPointFiveD, [128.0, 60.0], 0.0);
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(count(&px, true) < 50, "2.5D fades too: {} red pixels", count(&px, true));
}

/// 2D 同士は積み順。3D の間に挟まれても変わらない。
#[test]
fn two_d_layers_stack_in_timeline_order() {
    let mut engine = Engine::new().unwrap();
    let mut doc = document();
    layer(&mut doc, 1, 0, Rgb { r: 0.0, g: 0.2, b: 1.0 }, LayerProjection::TwoD, [128.0, 128.0], 0.0);
    layer(&mut doc, 2, 1, Rgb { r: 1.0, g: 0.0, b: 0.0 }, LayerProjection::TwoD, [128.0, 128.0], -500.0);
    let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(count(&px, false) < 50, "z does not lift a 2D layer above a later one: {} blue pixels", count(&px, false));
    assert!(count(&px, true) > 6000);
}

/// 既定カメラでは同じ Position の板が 2D / 2.5D / 3D で同じ場所に映る。
#[test]
fn the_three_projections_coincide_under_the_default_camera() {
    let mut engine = Engine::new().unwrap();
    let mut centroids = Vec::new();
    for projection in [LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::ThreeD] {
        let mut doc = document();
        layer(&mut doc, 1, 0, Rgb { r: 0.0, g: 0.2, b: 1.0 }, projection, [60.0, 60.0], 0.0);
        let px = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let (mut sx, mut sy, mut n) = (0.0, 0.0, 0usize);
        for (i, p) in px.chunks_exact(4).enumerate() { if p[2] > 128 && p[0] < 100 { sx += (i % 256) as f64; sy += (i / 256) as f64; n += 1; } }
        centroids.push((sx / n as f64, sy / n as f64, n));
    }
    for c in &centroids[1..] {
        assert!((c.0 - centroids[0].0).abs() < 0.5 && (c.1 - centroids[0].1).abs() < 0.5 && c.2 == centroids[0].2, "{centroids:?}");
    }
}
