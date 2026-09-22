use motolii_edit::{Document, Intent, blank_project};
use super::*;
use crate::doc::store::*;
use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};

fn circle(size: f64, scale: f64, pass: bool) -> Document {
    let shapes = vec![ShapeNode::Leaf(Shape {
        source: PathSource::Ellipse { size: Point { x: size, y: size } },
        ops: Vec::new(), stroke: None,
        fill: Some(Fill { brush: Brush::Solid(Rgb { r: 0.0, g: 0.0, b: 0.0 }), ..Default::default() }),
    })];
    let canvas = content_canvas(&shapes).unwrap().unwrap();
    let mut doc = blank_project();
    let mut comp = doc.view().composition().unwrap().unwrap();
    comp.width = 512; comp.height = 512; comp.background = [1.0; 4];
    let id = LayerId(1);
    doc.apply_all([
        Intent::SetComposition(comp), Intent::AddLayer(id),
        Intent::SetMeta { layer: id, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 60) } },
        Intent::SetShapes { layer: id, shapes },
        Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
        Intent::SetConstant { layer: id, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([256.0,256.0]) },
        Intent::SetConstant { layer: id, property: PropertyId::new(property::ANCHOR).unwrap(), value: Value::Vec2([canvas.origin_x as f64,canvas.origin_y as f64]) },
        Intent::SetConstant { layer: id, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale,scale]) },
    ]).unwrap();
    if pass {
        doc.apply_all([
            Intent::SetEffects { layer: id, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.gain".into() }] },
            Intent::SetConstant { layer: id, property: PropertyId::new("effect.0.param.gain").unwrap(), value: Value::F64(1.0) },
        ]).unwrap();
    }
    doc
}

/// 広がりの法: 効果は素材全体に素材座標で評価する。comp からはみ出した円の Blur が、
/// comp の縁で切れない(comp を広げても、重なる範囲の絵は同じ)。
#[test]
fn blur_reaches_past_the_composition_edge() {
    let scene = |width: u32| {
        let mut doc = circle(48.0, 1.0, false);
        let mut comp = doc.view().composition().unwrap().unwrap();
        comp.width = width; comp.height = 64;
        doc.apply_all([
            Intent::SetComposition(comp),
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([64.0, 32.0]) },
            Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.radius").unwrap(), value: Value::F64(8.0) },
        ]).unwrap();
        doc
    };
    let mut engine = Engine::new().unwrap();
    let narrow = engine.render_frame(&scene(64).view(), RationalTime::ZERO).unwrap();
    let wide = engine.render_frame(&scene(128).view(), RationalTime::ZERO).unwrap();
    let mut differing = 0;
    let mut blurred_edge = 0;
    for y in 0..64usize {
        for x in 0..64usize {
            let a = &narrow[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
            let b = &wide[(y * 128 + x) * 4..(y * 128 + x) * 4 + 4];
            if a.iter().zip(b).any(|(a, b)| a.abs_diff(*b) > 3) { differing += 1; }
            // 縁の 1 列: 円の中(白地に黒)がぼけて灰になっている画素
            if x == 63 && a[0] > 8 && a[0] < 247 { blurred_edge += 1; }
        }
    }
    assert!(blurred_edge > 4, "the circle must straddle the right edge and be blurred there");
    assert!(differing < 20, "the composition edge cut the blur: {differing} pixels differ from the wider composition");
}

/// 広がりの法(密度): 効果の radius・reach は論理 px。20 倍に置いた小さな円の Blur 4 は、
/// 大きな円の Blur 80 と同じ絵になる(余白の置き方も密度で割れている)。
#[test]
fn blur_radius_is_measured_in_logical_pixels_at_any_raster_density() {
    let blur = |mut doc: Document, radius: f64| {
        doc.apply_all([
            Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.radius").unwrap(), value: Value::F64(radius) },
        ]).unwrap();
        doc
    };
    let mut engine = Engine::new().unwrap();
    let a = engine.render_frame(&blur(circle(16.0, 20.0, false), 4.0).view(), RationalTime::ZERO).unwrap();
    let b = engine.render_frame(&blur(circle(320.0, 1.0, false), 80.0).view(), RationalTime::ZERO).unwrap();
    if let Some(out) = std::env::var_os("MOTOLII_VECTOR_EVIDENCE") {
        let dir = std::path::PathBuf::from(out); std::fs::create_dir_all(&dir).unwrap();
        image::save_buffer(dir.join("blur-density20.png"), &a, 512,512,image::ColorType::Rgba8).unwrap();
        image::save_buffer(dir.join("blur-density1.png"), &b, 512,512,image::ColorType::Rgba8).unwrap();
    }
    let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a, b)| a[0].abs_diff(b[0]) > 12).count();
    let soft = a.chunks_exact(4).filter(|p| p[0] > 16 && p[0] < 240).count();
    assert!(soft > 20000, "the magnified circle must be visibly blurred: {soft} soft pixels");
    assert!(bad < 1500, "blur width or padding changed with raster density: {bad} differing pixels");
}

#[test]
fn a_slow_field_does_not_tear_the_fill_and_stroke_of_one_plane() {
    let scene = |opacity: f64, evolution: f64| {
        let mut doc = circle(320.0, 1.0, false);
        let mut shapes = doc.view().shapes(LayerId(1)).unwrap();
        if let ShapeNode::Leaf(shape) = &mut shapes[0] {
            shape.fill.as_mut().unwrap().brush = Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 });
        }
        shapes.insert(0, ShapeNode::Leaf(Shape {
            source: PathSource::Rectangle { size: Point { x: 400.0, y: 400.0 } }, ops: Vec::new(), stroke: None,
            fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), opacity, ..Default::default() }),
        }));
        let mut comp = doc.view().composition().unwrap().unwrap(); comp.background = [0.0,0.0,0.0,1.0];
        doc.apply_all([
            Intent::SetComposition(comp), Intent::SetShapes { layer: LayerId(1), shapes },
            Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.turbulent_displace".into() }] },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.amount").unwrap(), value: Value::F64(400.0) },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.size").unwrap(), value: Value::F64(10000.0) },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.evolution").unwrap(), value: Value::F64(evolution) },
        ]).unwrap();
        doc
    };
    let mut engine = Engine::new().unwrap();
    let white = |pixels: &[u8]| pixels.chunks_exact(4).filter(|p| p[0]>230 && p[1]>230 && p[2]>230).count();
    for evolution in [0.0,1.0,2.0] {
        let reference = engine.render_frame(&scene(0.5,evolution).view(), RationalTime::ZERO).unwrap();
        let opaque = engine.render_frame(&scene(1.0,evolution).view(), RationalTime::ZERO).unwrap();
        let (expected,actual) = (white(&reference),white(&opaque));
        assert!(expected>5000 && actual*100 >= expected*98, "underpaint opacity must not occlude opaque foreground paint: {expected} -> {actual}, evolution={evolution}");
    }
}

#[test]
fn a_clipped_vector_selection_uses_only_the_visible_half() {
    let mut doc = circle(320.0, 1.0, false);
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.clip".into() }] },
        Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.axis").unwrap(), value: Value::F64(0.0) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let texture = engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label: Some("clipped vector selection"), size: wgpu::Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: crate::render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
    });
    engine.render_frame_into_with_camera(&doc.view(), RationalTime::ZERO, &texture, Default::default(), true, &[LayerId(1)]).unwrap();
    crate::compositor::wait_for_gpu(engine.gpu_device(), "vector-selection-test").unwrap();
    let bounds = engine.take_selection_bounds().unwrap();
    let (_, b) = bounds.iter().find(|(id,_)| *id == LayerId(1)).unwrap();
    assert!((95.0..=97.0).contains(&b[0]) && (255.0..=257.0).contains(&b[2]), "the clipped circle's mask must stop at its center: {b:?}");
}

#[test]
fn camera_magnification_keeps_the_contour_at_output_precision() {
    let mut engine = Engine::new().unwrap();
    let mut doc = circle(16.0, 1.0, false);
    doc.apply(Intent::SetAttrs { layer: LayerId(1), patch: LayerAttrsPatch { projection: Some(LayerProjection::ThreeD), ..Default::default() } }).unwrap();
    let camera = crate::doc::core::ResolvedCamera { zoom: 20.0, ..Default::default() };
    let a = engine.render_with_camera_override(&doc.view(), RationalTime::ZERO, true, Some(camera)).unwrap();
    let b = engine.render_frame(&circle(320.0, 1.0, false).view(), RationalTime::ZERO).unwrap();
    let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a,b)| a[0].abs_diff(b[0]) > 16).count();
    assert!(bad < 300, "camera magnification introduced {bad} differing pixels");
}

#[test]
fn translucent_vector_paint_blends_over_the_background() {
    let mut doc = circle(320.0, 1.0, false);
    let mut shapes = doc.view().shapes(LayerId(1)).unwrap();
    if let ShapeNode::Leaf(shape) = &mut shapes[0] { shape.fill.as_mut().unwrap().opacity = 0.5; }
    doc.apply(Intent::SetShapes { layer: LayerId(1), shapes }).unwrap();
    let pixels = Engine::new().unwrap().render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    let center = pixels[(256*512+256)*4];
    assert!((180..=195).contains(&center), "half black over linear white, encoded as sRGB: {center}");
}

#[test]
fn enlarging_a_path_matches_drawing_the_large_contour_even_at_an_image_effect_boundary() {
    let mut engine = Engine::new().unwrap();
    for pass in [false,true] {
        let small = circle(16.0, 20.0, pass);
        let a = engine.render_frame(&small.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(engine.shape_textures.values().any(|c| matches!(c.texture, LayerContent::Model(ref m) if m.planar_size.is_some())), "the contour must remain geometry");
        let b = engine.render_frame(&circle(320.0,1.0,pass).view(), RationalTime::ZERO).unwrap();
        let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a,b)| a[0].abs_diff(b[0]) > 16).count();
        let ink = a.chunks_exact(4).filter(|p| p[0] < 128).count();
        if let Some(out) = std::env::var_os("MOTOLII_VECTOR_EVIDENCE") {
            let dir = std::path::PathBuf::from(out); std::fs::create_dir_all(&dir).unwrap();
            image::save_buffer(dir.join(format!("circle-scale20-pass{pass}.png")), &a, 512,512,image::ColorType::Rgba8).unwrap();
            image::save_buffer(dir.join(format!("circle-reference-pass{pass}.png")), &b, 512,512,image::ColorType::Rgba8).unwrap();
            std::fs::write(dir.join(format!("circle-comparison-pass{pass}.json")), serde_json::json!({"different_pixels_over_16":bad,"ink_pixels":ink,"pixels":512*512,"scale":20,"image_effect":pass}).to_string()).unwrap();
        }
        assert!(ink > 70000 && ink < 90000, "the unlit circle keeps its size and paint: {ink}");
        assert!(bad < 300, "magnifying a contour introduced {bad} differing pixels (image effect={pass})");
    }
}
