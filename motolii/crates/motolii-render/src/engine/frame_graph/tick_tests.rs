//! The tick's invariants, fixed before any feature is ported onto it: one renderer frame and one
//! submission per tick, at most one preparation per document frame, however many views.

use super::tick::{TickStats, ViewRequest};
use crate::doc::core::RationalTime;
use crate::frame_graph::ViewProjection;
use crate::render::compositor::{Window, PRESENTABLE_FORMAT};
use crate::render::engine::environment_tests::SIZE;
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
        (Window { width: SIZE, height: SIZE, roi: [0.0, 0.0, comp, comp] }, ViewProjection::Camera)
    } else {
        (Window { width: SIZE, height: SIZE / 2, roi: [-comp / 2.0, 0.0, comp * 2.0, comp] }, ViewProjection::Stage)
    }).collect()
}

fn tick(engine: &mut Engine, doc: &motolii_edit::Document, time: RationalTime, n: usize) -> TickStats {
    let windows = windows(n);
    let targets: Vec<_> = windows.iter().map(|(w, _)| target(engine, *w)).collect();
    let views: Vec<_> = windows.iter().zip(&targets).map(|((window, projection), target)| ViewRequest {
        target, window: *window, camera: (*projection == ViewProjection::Stage).then(Default::default), projection: *projection, include_background: true, read_back: false,
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

/// A view's window and zoom are the view's: resizing a Stage (any aspect) neither evaluates the
/// semantic scene again nor prepares the world again; zooming in so far that the views ask for
/// finer outlines prepares the world again at that precision — on the same semantic scene.
#[test]
fn a_resize_or_a_zoom_never_evaluates_the_scene_again() {
    let dir = tempfile::tempdir().unwrap();
    let doc = pictures(dir.path());
    let mut engine = Engine::new().unwrap();
    let comp = SIZE as f32;
    let stage = |engine: &mut Engine, width: u32, height: u32, roi: [f32; 4]| {
        let window = Window { width, height, roi };
        let target = target(engine, window);
        let view = ViewRequest { target: &target, window, camera: Some(Default::default()), projection: ViewProjection::Stage, include_background: true, read_back: false };
        let stats = engine.tick(&doc.view(), RationalTime::ZERO, &[view]).unwrap();
        let state = engine.frame_graph.as_ref().unwrap();
        (stats.preparations, state.generation, engine.tick_frame.clone().unwrap())
    };
    let (_, scene, first) = stage(&mut engine, SIZE, SIZE, [0.0, 0.0, comp, comp]);
    // A resize at the same zoom: the region of interest grows and shrinks with the window.
    for (width, height) in [(SIZE / 2, SIZE), (SIZE, SIZE / 3), (SIZE * 3 / 4, SIZE / 2), (SIZE * 2, SIZE)] {
        let roi = [-(width as f32) / 4.0, 0.0, width as f32, height as f32];
        let (prepared, generation, frame) = stage(&mut engine, width, height, roi);
        assert_eq!((prepared, generation), (0, scene), "{width}x{height}: a resize is the view's alone");
        assert!(std::sync::Arc::ptr_eq(&frame.scene, &first.scene), "{width}x{height}: the same prepared world");
    }
    let (prepared, generation, frame) = stage(&mut engine, SIZE * 4, SIZE * 4, [0.0, 0.0, comp, comp]);
    assert_eq!((prepared, generation), (1, scene), "a 4x zoom asks for finer outlines: prepared again, not evaluated again");
    for (a, b) in frame.scene.layers.iter().zip(&first.scene.layers) {
        assert_eq!(a.layer.placement.transform, b.layer.placement.transform, "precision never moves a layer");
    }
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
    // A second mesh beside it, under the same sun.
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
        (stats, after.light_captures - before.light_captures, after.backdrop_copies - before.backdrop_copies)
    };
    let (one, lights, copies) = work(1);
    assert_eq!(lights, 1, "the scene casts a shadow");
    for n in [2, 5] {
        let (stats, l, b) = work(n);
        assert_eq!((stats.preparations, stats.submits, l), (one.preparations, one.submits, lights), "{n} views: the world is made once");
        assert_eq!(b, copies * n as u64, "{n} views: each view copies its own backdrop");
    }
}

/// The Views a layer's Vism asks for are the world's: drawn once per document frame, however
/// many views show it.
#[test]
fn layer_views_do_not_grow_with_views() {
    use crate::doc::store::{EffectId, EffectInstance, LayerId};
    use crate::render::engine::environment_tests::{scene, sky_png};
    use motolii_edit::Intent;
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 40, 220);
    let mut doc = scene(dir.path(), &sky, true);
    doc.apply(Intent::SetEffects { layer: LayerId(2), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.cube_mirror".into() }] }).unwrap();
    for n in [1, 2, 5] {
        let mut engine = Engine::new().unwrap();
        let before = engine.surface_work().layer_views;
        let stats = tick(&mut engine, &doc, RationalTime::ZERO, n);
        assert_eq!((stats.preparations, engine.surface_work().layer_views - before), (1, 6), "{n} views");
    }
}

/// A plate is made inside the preparation and goes out with the tick's one submission.
#[test]
fn a_plate_is_recorded_with_the_tick() {
    let dir = tempfile::tempdir().unwrap();
    let doc = plate(dir.path());
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
        let window = crate::render::compositor::Window { width: SIZE, height: SIZE, roi: [0.0, 0.0, comp, comp] };
        let preview = target(&engine, window);
        engine.tick(&doc.view(), RationalTime::ZERO, &[ViewRequest { target: &preview, window, camera: None, projection: ViewProjection::Camera, include_background: true, read_back: false }]).unwrap();
        let shown = engine.read_texture_offline(&preview).unwrap();
        let exported = Engine::new().unwrap().export_frame(&doc.view(), RationalTime::ZERO, true, None).unwrap();
        let worst = shown.iter().zip(&exported).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
        assert!(worst <= 1, "{count} copies: preview and export differ by {worst}");
    }
}

/// The frame's reflection, checked on the preparation's own record (no pictures): one capture per
/// evaluated frame, taken before any plate is baked, seeing every plate's members, and the light
/// every plate is baked with.
mod frame_reflection {
    use super::*;
    use crate::doc::store::{EffectId, EffectInstance, EffectScope, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, Value};
    use crate::render::engine::environment_tests::{file_layer, scene, sky_png};
    use crate::render::engine::frame_graph_scene::PreparationEvent::{Bake, Capture};
    use motolii_edit::{Document, Intent};

    /// `count` groups, each a plate (a Repeater on the whole and a cheap effect after it) holding
    /// two mesh members.
    fn plates(dir: &std::path::Path, count: u64) -> (Document, Vec<LayerId>) {
        let sky = sky_png(dir, "sky.png", 40, 220);
        let mut doc = scene(dir, &sky, true);
        let obj = dir.join("quad.obj");
        let mut members = Vec::new();
        for k in 0..count {
            let group = LayerId(100 + k * 10);
            let (repeat, gain) = (EffectId(1000 + k as u32), EffectId(1500 + k as u32));
            doc.apply_all([
                Intent::AddLayer(group),
                Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 10 + k as i16 * 3, timing: LayerTiming::place(0, None, 1) } },
                Intent::SetEffects { layer: group, effects: vec![
                    EffectInstance { id: repeat, plugin_id: crate::extensions::placement::REPEAT.to_owned() },
                    EffectInstance { id: gain, plugin_id: "motolii.gain".to_owned() },
                ] },
                Intent::SetConstant { layer: group, property: PropertyId::effect_scope(gain), value: Value::Enum(EffectScope::Whole.enum_value()) },
            ]).unwrap();
            for m in 1..=2 {
                let member = file_layer(&mut doc, 100 + k * 10 + m, 10 + k as i16 * 3 + m as i16, &obj);
                doc.apply(Intent::SetAttrs { layer: member, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
                doc.apply(Intent::SetEffects { layer: member, effects: vec![EffectInstance { id: EffectId(2000 + (k * 10 + m) as u32), plugin_id: "motolii.glass".into() }] }).unwrap();
                members.push(member);
            }
        }
        (doc, members)
    }

    fn prepare(doc: &Document, views: usize) -> Engine {
        let mut engine = Engine::new().unwrap();
        let _ = tick(&mut engine, doc, RationalTime::ZERO, views);
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        engine
    }

    /// One capture, before every bake, and every bake lit by it — however many plates or views.
    #[test]
    fn one_capture_lights_every_plate_however_many_plates_or_views() {
        let dir = tempfile::tempdir().unwrap();
        for (count, views) in [(1, 1), (3, 1), (3, 5)] {
            let (doc, _) = plates(dir.path(), count);
            let engine = prepare(&doc, views);
            let events = &engine.preparation_events;
            let captures: Vec<_> = events.iter().filter_map(|e| match e { Capture(s) => Some(*s), _ => None }).collect();
            assert_eq!(captures.len(), 1, "{count} plates, {views} views: one frame-level capture: {events:?}");
            // Their members are glass: each view materializes the plate over its own picture below
            // (2026-09-23), so the preparation bakes no picture of it.
            assert_eq!(events.iter().filter(|e| matches!(e, Bake(_))).count(), 0, "a plate holding glass is the views': {events:?}");
            assert_eq!(events.first(), Some(&Capture(captures[0])), "the capture comes first: {events:?}");
        }
    }

    /// A plate without glass reads nothing of a view: its picture is baked once in the preparation,
    /// however many views draw it (the plate as an optimization).
    #[test]
    fn a_plate_without_glass_is_baked_once_for_every_view() {
        let dir = tempfile::tempdir().unwrap();
        let (mut doc, members) = plates(dir.path(), 2);
        for member in members {
            doc.apply(Intent::SetEffects { layer: member, effects: vec![] }).unwrap();
        }
        let engine = prepare(&doc, 3);
        let events = &engine.preparation_events;
        assert_eq!(events.iter().filter(|e| matches!(e, Bake(_))).count(), 2, "each plate baked once: {events:?}");
    }

    /// Every plate's members are in the scene the frame's one light capture sees; no plate
    /// captures light of its own.
    #[test]
    fn plate_members_are_the_light_scene_and_no_plate_captures_its_own() {
        let dir = tempfile::tempdir().unwrap();
        let (one, _) = plates(dir.path(), 1);
        let (three, _) = plates(dir.path(), 3);
        let faces = |doc: &Document| { let engine = prepare(doc, 1); engine.surface_work().light_captures };
        let mut engine = prepare(&three, 1);
        let members = engine.plate_member_ids(&three, RationalTime::ZERO);
        assert!(!members.is_empty());
        for member in &members {
            assert!(engine.light_scene_ids.contains(member), "member {member:?} is in the captured scene: {:?}", engine.light_scene_ids);
        }
        assert_eq!(faces(&one), faces(&three), "three plates capture no more light than one");
    }

    /// A matte reading a plate needs it during preparation: still one capture, and the plate is lit
    /// by it, not by one of its own.
    #[test]
    fn a_plate_needed_early_is_lit_by_the_frames_capture() {
        use crate::doc::store::{Matte, MatteMode};
        let dir = tempfile::tempdir().unwrap();
        let (mut doc, _) = plates(dir.path(), 1);
        let board = file_layer(&mut doc, 500, 40, &dir.path().join("sky.png"));
        doc.apply(Intent::SetAttrs { layer: board, patch: LayerAttrsPatch { matte: Some(Some(Matte { layer: LayerId(100), mode: MatteMode::Alpha })), ..Default::default() } }).unwrap();
        let engine = prepare(&doc, 1);
        let events = &engine.preparation_events;
        let captures: Vec<_> = events.iter().filter_map(|e| match e { Capture(s) => Some(*s), _ => None }).collect();
        assert_eq!(captures.len(), 1, "{events:?}");
        assert!(events.iter().any(|e| matches!(e, Bake(_))) && events.iter().all(|e| !matches!(e, Bake(s) if *s != captures[0])), "{events:?}");
    }

    /// Reads for analysis belong to the frame: however many, no capture of their own; an
    /// evaluation of its own (a frozen frame) is lit once.
    #[test]
    fn reads_within_a_frame_make_no_capture_of_their_own() {
        let dir = tempfile::tempdir().unwrap();
        let (doc, _) = plates(dir.path(), 2);
        let mut engine = prepare(&doc, 1);
        let scene = engine.frame_graph_editor_scene(&doc.view(), RationalTime::ZERO).unwrap();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let before = engine.world_light_captures;
        let prep = crate::render::engine::frame_graph_scene::Preparation::new(comp, Default::default(), crate::render::engine::frame_graph_scene::LegacyCameraSeam::new(Default::default()));
        for _ in 0..3 { engine.prepare_gpu_pictures(&scene, &prep, false).unwrap(); }
        assert_eq!(engine.world_light_captures, before, "three reads within the frame, no capture");
        engine.prepare_gpu_pictures(&scene, &prep, true).unwrap();
        assert_eq!(engine.world_light_captures, before + 1, "an evaluation of its own is lit once");
        engine.compositor.next_frame();
    }
}

/// A plate with an effect after it shows its members: its picture is read after it is baked (a
/// snapshot taken while it waited for the frame's light would be empty).
#[test]
fn a_plate_with_an_effect_after_it_shows_its_members() {
    let dir = tempfile::tempdir().unwrap();
    let doc = plate(dir.path());
    let pixels = Engine::new().unwrap().export_frame(&doc.view(), RationalTime::ZERO, true, None).unwrap();
    let dots = pixels.chunks(4).filter(|p| p[0] > 200 && p[1] > 150 && p[2] < 120).count();
    assert!(dots > 0, "the plate's orange dot is in the picture");
}


/// A plate is an intent, not a raster (2026-09-23): glass in a Repeater's plate (`Whole`) refracts
/// the view's picture below the plate — here a red picture — as the same copies drawn each
/// (`Each`) do, instead of finding nothing below and showing the sky.
#[test]
fn glass_in_a_plate_refracts_the_views_picture_below_it() {
    use crate::doc::store::{EffectId, EffectInstance, EffectScope, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};
    use crate::render::engine::environment_tests::{scene, sky_png, SIZE};
    let render = |scope: EffectScope| {
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "sky.png", 40, 220);
        let mut doc = scene(dir.path(), &sky, true);
        let red = file_layer(&mut doc, 20, 3, &png(dir.path(), "red.png", [230, 20, 20, 255]));
        doc.apply(Intent::SetConstant { layer: red, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([2.0, 2.0]) }).unwrap();
        let group = LayerId(30);
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 5, timing: LayerTiming::place(0, None, 1) } },
            // The copies, then an effect on them: `Whole` puts it on one plate of the copies, `Each`
            // on every copy (the effect is neutral, so the two pictures are the same).
            Intent::SetEffects { layer: group, effects: vec![
                EffectInstance { id: EffectId(0), plugin_id: crate::extensions::placement::REPEAT.to_owned() },
                EffectInstance { id: EffectId(2), plugin_id: "motolii.gain".to_owned() },
            ] },
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "count").unwrap(), value: Value::F64(2.0) },
            Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([8.0, 0.0]) },
            Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(2)), value: Value::Enum(scope.enum_value()) },
        ]).unwrap();
        let glass = file_layer(&mut doc, 31, 6, &dir.path().join("quad.obj"));
        doc.apply(Intent::SetAttrs { layer: glass, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
        doc.apply(Intent::SetConstant { layer: glass, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([6.0, 6.0]) }).unwrap();
        doc.apply(Intent::SetEffects { layer: glass, effects: vec![EffectInstance { id: EffectId(1), plugin_id: "motolii.glass".into() }] }).unwrap();
        doc.apply(Intent::SetConstant { layer: glass, property: PropertyId::effect_param(EffectId(1), "transmission").unwrap(), value: Value::F64(1.0) }).unwrap();
        let mut engine = Engine::new().unwrap();
        let pixels = engine.export_frame(&doc.view(), RationalTime::ZERO, true, None).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    };
    let (whole, each) = (render(EffectScope::Whole), render(EffectScope::Each));
    let center = ((SIZE / 2 * SIZE + SIZE / 2) * 4) as usize;
    let p = &whole[center..center + 4];
    assert!(i32::from(p[0]) - i32::from(p[2]) > 60, "the glass in the plate shows the red picture below it: {p:?}");
    let differ = whole.chunks(4).zip(each.chunks(4)).filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 8)).count();
    assert!(differ * 100 < (SIZE * SIZE) as usize, "a plate looks as its copies drawn each: {differ} pixels differ");
}

// Fixtures: two overlapping pictures; a repeated group baked into a glowing plate.
use crate::doc::store::{property, LayerId, PropertyId, Value};
use crate::render::engine::environment_tests::file_layer;
use motolii_edit::{Document, Intent};

fn png(dir: &std::path::Path, name: &str, rgba: [u8; 4]) -> std::path::PathBuf {
    let path = dir.join(name);
    image::RgbaImage::from_pixel(16, 16, image::Rgba(rgba)).save(&path).unwrap();
    path
}

fn comp(doc: &mut Document) {
    doc.apply(Intent::SetComposition(crate::doc::store::Composition {
        width: SIZE, height: SIZE, fps: crate::doc::store::Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.1, 0.2, 0.3, 1.0],
    })).unwrap();
}

/// Two overlapping pictures: one scaled, one turned and see-through.
fn pictures(dir: &std::path::Path) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    comp(&mut doc);
    let red = file_layer(&mut doc, 1, 0, &png(dir, "red.png", [255, 0, 0, 255]));
    let green = file_layer(&mut doc, 2, 1, &png(dir, "green.png", [0, 255, 0, 160]));
    let set = |doc: &mut Document, layer: LayerId, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    set(&mut doc, red, property::SCALE, Value::Vec2([2.0, 2.0]));
    set(&mut doc, green, property::POSITION, Value::Vec2([40.0, 36.0]));
    set(&mut doc, green, property::ROTATION, Value::F64(30.0));
    set(&mut doc, green, property::OPACITY, Value::F64(70.0));
    doc
}

/// A group repeated as a whole with a glow on the whole (Glass Garden's rings): the copies are
/// baked into one plate inside the preparation, the glow reads the plate. One member is a glass
/// mesh, so the plate has its own light.
fn plate(dir: &std::path::Path) -> Document {
    use crate::doc::store::{EffectId, EffectInstance, EffectScope, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};
    use crate::render::engine::environment_tests::{scene, sky_png};
    let sky = sky_png(dir, "sky.png", 40, 220);
    let mut doc = scene(dir, &sky, true);
    let group = LayerId(10);
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 5, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetEffects { layer: group, effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: crate::extensions::placement::REPEAT.to_owned() },
            EffectInstance { id: EffectId(1), plugin_id: "motolii.glow".to_owned() },
        ] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "count").unwrap(), value: Value::F64(3.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([12.0, 6.0]) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) },
    ]).unwrap();
    let dot = file_layer(&mut doc, 11, 6, &png(dir, "dot.png", [255, 200, 40, 255]));
    let glass = crate::render::engine::environment_tests::file_layer(&mut doc, 12, 7, &dir.join("quad.obj"));
    for member in [dot, glass] {
        doc.apply(Intent::SetAttrs { layer: member, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
    }
    doc.apply(Intent::SetConstant { layer: glass, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([6.0, 6.0]) }).unwrap();
    doc.apply(Intent::SetEffects { layer: glass, effects: vec![EffectInstance { id: EffectId(2), plugin_id: "motolii.glass".into() }] }).unwrap();
    doc
}
