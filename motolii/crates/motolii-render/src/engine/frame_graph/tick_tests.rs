//! The tick's invariants, fixed before any feature is ported onto it: one renderer frame and one
//! submission per tick, at most one preparation per document frame, however many views.

use super::tick::{TickStats, ViewRequest};
use crate::doc::core::RationalTime;
use crate::frame_graph::ViewProjection;
use crate::render::compositor::{Window, PRESENTABLE_FORMAT};
use crate::render::engine::environment_tests::SIZE;
use super::tick_oracle_tests::pictures;
use crate::render::engine::Engine;

fn target(engine: &Engine, window: Window) -> wgpu::Texture {
    engine.compositor.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("tick-test-view"),
        size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// A Camera view (the output) and a Stage view (wider, half as dense, from the default camera).
fn windows(n: usize) -> Vec<(Window, ViewProjection)> {
    let comp = SIZE as f32;
    (0..n).map(|i| if i % 2 == 0 {
        (Window { width: SIZE, height: SIZE, roi: [0.0, 0.0, comp, comp], projection_camera: None }, ViewProjection::Camera)
    } else {
        (Window { width: SIZE, height: SIZE / 2, roi: [-comp / 2.0, 0.0, comp * 2.0, comp], projection_camera: Some(Default::default()) }, ViewProjection::Stage)
    }).collect()
}

fn tick(engine: &mut Engine, doc: &motolii_edit::Document, time: RationalTime, n: usize) -> TickStats {
    let windows = windows(n);
    let targets: Vec<_> = windows.iter().map(|(w, _)| target(engine, *w)).collect();
    let views: Vec<_> = windows.iter().zip(&targets).map(|((window, projection), target)| ViewRequest {
        target, window: *window, camera: (*projection == ViewProjection::Stage).then(Default::default), projection: *projection, include_background: true, outline: &[],
    }).collect();
    engine.tick(&doc.view(), time, &views).unwrap()
}

#[test]
fn a_tick_is_one_frame_one_preparation_and_one_submission_however_many_views() {
    let dir = tempfile::tempdir().unwrap();
    let doc = pictures(dir.path());
    for n in [0, 1, 2, 5] {
        let mut engine = Engine::new().unwrap();
        let stats = tick(&mut engine, &doc, RationalTime::ZERO, n);
        assert_eq!(stats, TickStats { begin_frames: 1, preparations: 1, views: n as u32, submits: 1 }, "{n} view(s)");
        // The same document frame again (a paused playhead, a redraw): nothing is prepared.
        let again = tick(&mut engine, &doc, RationalTime::ZERO, n);
        assert_eq!(again, TickStats { begin_frames: 1, preparations: 0, views: n as u32, submits: 1 }, "{n} view(s), same frame");
    }
}

#[test]
fn every_view_of_a_tick_reads_the_same_prepared_frame() {
    let dir = tempfile::tempdir().unwrap();
    let doc = pictures(dir.path());
    let mut engine = Engine::new().unwrap();
    tick(&mut engine, &doc, RationalTime::ZERO, 3);
    let first = engine.tick_frame.clone().unwrap();
    tick(&mut engine, &doc, RationalTime::ZERO, 3);
    let second = engine.tick_frame.clone().unwrap();
    assert!(std::sync::Arc::ptr_eq(&first.scene, &second.scene), "the document frame is prepared once and only read by views");
}

/// The world's light is captured once per document frame, however many views look at it; the
/// views' own work (a glass layer's backdrop copy) grows with the views.
#[test]
fn world_captures_do_not_grow_with_views() {
    use crate::doc::store::{EffectId, EffectInstance, LayerId, PropertyId, Value};
    use crate::render::engine::environment_tests::{scene, sky_png};
    use motolii_edit::Intent;
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 40, 220);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(crate::doc::store::property::POSITION_Z).unwrap(), value: Value::F64(-20.0) }).unwrap();
    // Something for the glass to reflect: a second mesh beside it.
    let other = crate::render::engine::environment_tests::file_layer(&mut doc, 3, 2, &dir.path().join("quad.obj"));
    doc.apply(Intent::SetConstant { layer: other, property: PropertyId::new(crate::doc::store::property::POSITION).unwrap(), value: Value::Vec2([10.0, 10.0]) }).unwrap();
    doc.apply(Intent::SetConstant { layer: other, property: PropertyId::new(crate::doc::store::property::SCALE).unwrap(), value: Value::Vec2([8.0, 8.0]) }).unwrap();
    doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
        EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
        EffectInstance { id: EffectId(1), plugin_id: "motolii.cast_shadow".into() },
    ] }).unwrap();
    let work = |n: usize| {
        let mut engine = Engine::new().unwrap();
        let before = engine.surface_work();
        let stats = tick(&mut engine, &doc, RationalTime::ZERO, n);
        let after = engine.surface_work();
        (stats, after.scene_captures - before.scene_captures, after.light_captures - before.light_captures, after.backdrop_copies - before.backdrop_copies)
    };
    let (one, captures, lights, copies) = work(1);
    assert!(captures > 0 && lights == 1, "the scene reflects and casts a shadow: {captures} {lights}");
    for n in [2, 5] {
        let (stats, c, l, b) = work(n);
        assert_eq!((stats.preparations, stats.submits, c, l), (one.preparations, one.submits, captures, lights), "{n} views: the world is made once");
        assert_eq!(b, copies * n as u64, "{n} views: each view copies its own backdrop");
    }
}

/// A plate is made inside the preparation and goes out with the tick's one submission.
#[test]
fn a_plate_is_recorded_with_the_tick() {
    let dir = tempfile::tempdir().unwrap();
    let doc = super::tick_oracle_tests::plate(dir.path());
    for n in [1, 2] {
        let mut engine = Engine::new().unwrap();
        let stats = tick(&mut engine, &doc, RationalTime::ZERO, n);
        assert_eq!(stats, TickStats { begin_frames: 1, preparations: 1, views: n as u32, submits: 1 }, "{n} view(s)");
    }
}

/// Standard glass is one group per stretch of glass in the stack: separate glass layers and a
/// Repeater's copies alike read one backdrop, however many there are; a picture between glass
/// starts another group (it is what the upper glass refracts).
#[test]
fn glass_layers_share_a_backdrop_however_many() {
    use crate::doc::store::{EffectId, EffectInstance, LayerId, PropertyId, Value, property};
    use crate::render::engine::environment_tests::{file_layer, scene, sky_png};
    use motolii_edit::Intent;
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 40, 220);
    let obj = dir.path().join("quad.obj");
    let copies = |doc: &mut motolii_edit::Document, glass: &[u64], picture_between: bool| {
        for (k, id) in glass.iter().enumerate() {
            let layer = if *id == 2 { LayerId(2) } else { file_layer(doc, *id, 2 + k as i16 * 2, &obj) };
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([10.0 + 8.0 * k as f64, 12.0]) }).unwrap();
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([8.0, 8.0]) }).unwrap();
            doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId((100 + *id) as _), plugin_id: "motolii.glass".into() }] }).unwrap();
        }
        if picture_between {
            let red = dir.path().join("red.png");
            image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
            file_layer(doc, 50, 3, &red);
        }
    };
    let backdrops = |glass: &[u64], picture_between: bool| {
        let mut doc = scene(dir.path(), &sky, true);
        copies(&mut doc, glass, picture_between);
        let mut engine = Engine::new().unwrap();
        let before = engine.surface_work();
        let _ = tick(&mut engine, &doc, RationalTime::ZERO, 1);
        engine.surface_work().backdrop_copies - before.backdrop_copies
    };
    assert_eq!(backdrops(&[2], false), 1);
    assert_eq!(backdrops(&[2, 3], false), 1, "two glass layers next to each other: one backdrop");
    assert_eq!(backdrops(&[2, 3, 4, 5, 6], false), 1, "five: still one");
    assert_eq!(backdrops(&[2, 3], true), 2, "a picture between them: the upper glass refracts it");
}

/// Where two glass layers next to each other overlap, the upper one refracts the picture below
/// the pair, not the lower glass: the overlap is what the upper glass alone would show there.
#[test]
fn overlapping_glass_refracts_the_picture_below_not_the_other_glass() {
    use crate::doc::store::{EffectId, EffectInstance, LayerId, PropertyId, Value, property};
    use crate::render::engine::environment_tests::{file_layer, scene, sky_png, SIZE};
    use motolii_edit::Intent;
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 30, 230);
    let obj = dir.path().join("quad.obj");
    let render = |with_lower: bool| {
        let mut doc = scene(dir.path(), &sky, true);
        // Something to refract: a red board behind.
        let red = dir.path().join("board.png");
        image::RgbaImage::from_fn(SIZE, SIZE, |x, _| if (x / 3) % 2 == 0 { image::Rgba([220, 30, 30, 255]) } else { image::Rgba([240, 240, 240, 255]) }).save(&red).unwrap();
        let board = file_layer(&mut doc, 20, 1, &red);
        doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        let glass = |doc: &mut motolii_edit::Document, layer: LayerId, x: f64, ior: f64| {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([x, 20.0]) }).unwrap();
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([12.0, 12.0]) }).unwrap();
            doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(layer.0 as _), plugin_id: "motolii.glass".into() }] }).unwrap();
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(&format!("effect.{}.param.ior", layer.0)).unwrap(), value: Value::F64(ior) }).unwrap();
        };
        // The scene's own mesh stays out; the lower and upper glass are next to each other above the board.
        doc.apply(Intent::SetAttrs { layer: LayerId(2), patch: crate::doc::store::LayerAttrsPatch { hidden: Some(true), ..Default::default() } }).unwrap();
        if with_lower {
            let lower = file_layer(&mut doc, 25, 4, &obj);
            glass(&mut doc, lower, 16.0, 1.8);
        }
        let upper = file_layer(&mut doc, 30, 5, &obj);
        glass(&mut doc, upper, 28.0, 1.3);
        let mut engine = Engine::new().unwrap();
        let pixels = engine.export_frame(&doc.view(), RationalTime::ZERO, true, None).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    };
    let both = render(true);
    let upper_alone = render(false);
    // A pixel inside the upper glass and over the lower one.
    let at = |p: &[u8], x: u32, y: u32| p[((y * SIZE + x) * 4) as usize..((y * SIZE + x) * 4 + 4) as usize].to_vec();
    let (x, y) = (32, 30);
    let worst = at(&both, x, y).iter().zip(at(&upper_alone, x, y)).map(|(a, b)| a.abs_diff(b)).max().unwrap();
    // The glass is really there: across its row it differs from the bare board.
    let row = |p: &[u8], y: u32| (8..40).map(|x| at(p, x, y)).collect::<Vec<_>>();
    assert_ne!(row(&both, y), row(&upper_alone, y), "the lower glass changes the picture where it is alone");
    assert!(worst <= 2, "the upper glass shows the picture below the pair: {:?} vs {:?}", at(&both, x, y), at(&upper_alone, x, y));
}

/// A Repeater's glass copies are standard glass like any other: one transmission input for 1, 10 or
/// 100 copies, drawn as one instanced batch; preview (the Camera view) and export are the same picture.
#[test]
fn repeated_glass_is_one_transmission_input_and_one_batch_and_previews_as_it_exports() {
    use crate::doc::store::{EffectId, EffectInstance, LayerId, PropertyId, Value, property};
    use crate::render::engine::environment_tests::{scene, sky_png, SIZE};
    use motolii_edit::Intent;
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 40, 220);
    let doc_with = |count: u32| {
        let mut doc = scene(dir.path(), &sky, true);
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
            EffectInstance { id: EffectId(1), plugin_id: crate::extensions::placement::REPEAT.into() },
        ] }).unwrap();
        doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([2.0, 2.0]) }).unwrap();
        doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::effect_param(EffectId(1), "count").unwrap(), value: Value::F64(count as f64) }).unwrap();
        doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::effect_param(EffectId(1), "position_each").unwrap(), value: Value::Vec2([0.5, 0.5]) }).unwrap();
        doc
    };
    for count in [1, 10, 100] {
        let doc = doc_with(count);
        let mut engine = Engine::new().unwrap();
        let before = engine.surface_work();
        let _ = tick(&mut engine, &doc, RationalTime::ZERO, 1);
        let after = engine.surface_work();
        assert_eq!(after.backdrop_copies - before.backdrop_copies, 1, "{count} glass copies, one transmission input");
        assert!(after.main_runs - before.main_runs <= 3, "{count} copies are not a run each: {}", after.main_runs - before.main_runs);

        // Preview and export: the same meaning, the same pixels.
        let comp = SIZE as f32;
        let window = crate::render::compositor::Window { width: SIZE, height: SIZE, roi: [0.0, 0.0, comp, comp], projection_camera: None };
        let preview = target(&engine, window);
        engine.tick(&doc.view(), RationalTime::ZERO, &[ViewRequest { target: &preview, window, camera: None, projection: ViewProjection::Camera, include_background: true, outline: &[] }]).unwrap();
        let shown = engine.compositor.read_texture_bytes(&preview).unwrap();
        let exported = Engine::new().unwrap().export_frame(&doc.view(), RationalTime::ZERO, true, None).unwrap();
        let worst = shown.iter().zip(&exported).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
        assert!(worst <= 1, "{count} copies: preview and export differ by {worst}");
    }
}
