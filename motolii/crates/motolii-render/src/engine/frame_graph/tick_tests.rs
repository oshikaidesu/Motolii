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
        target, window: *window, camera: Default::default(), projection: *projection, include_background: true,
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
