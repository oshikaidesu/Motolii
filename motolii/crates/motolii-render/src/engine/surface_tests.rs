use motolii_edit::{Document, Intent};
use super::environment_tests::{file_layer, scene, sky_png, MESH_X, MESH_Y, SIZE};
use super::*;
use crate::doc::store::{
    property, EffectId, EffectInstance, LayerId, LayerSource,
    PropertyId, Value,
};
use crate::extensions::{placement};

pub(super) fn set(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::new(name).unwrap(),
        value,
    })
    .unwrap();
}
pub(super) fn effect(doc: &mut Document, layer: LayerId, id: u32, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::effect_param(EffectId(id), name).unwrap(),
        value,
    })
    .unwrap();
}

/// The capture/draw budget belongs to the scene, not to each Repeater copy.
#[test]
fn repeated_mirrors_batch_and_do_not_copy_the_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut engine = Engine::new().unwrap();
    engine.set_gpu_instance_sharing_enabled(false);
    for count in [10, 100, 1000] {
        let mut doc = scene(dir.path(), &sky, true);
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects {
            layer: mesh,
            effects: vec![
                EffectInstance {
                    id: EffectId(0),
                    plugin_id: "motolii.glass".into(),
                },
                EffectInstance {
                    id: EffectId(1),
                    plugin_id: placement::REPEAT.into(),
                },
            ],
        })
        .unwrap();
        set(&mut doc, mesh, property::POSITION, Value::Vec2([0.0, 0.0]));
        set(&mut doc, mesh, property::SCALE, Value::Vec2([0.7, 0.7]));
        for (id, name, value) in [
            (0, "transmission", Value::F64(0.0)),
            (0, "metallic", Value::F64(1.0)),
            (1, "count", Value::F64(count as f64)),
            (1, "mode", Value::F64(2.0)),
            (1, "columns", Value::F64(32.0)),
            (1, "position_each", Value::Vec2([2.0, 2.0])),
        ] {
            effect(&mut doc, mesh, id, name, value);
        }
        // The world is made once per document frame: a redraw of the same frame draws only the view.
        let before = engine.surface_work();
        let started = std::time::Instant::now();
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        assert!(
            engine.layer_failures().is_empty(),
            "{:?}",
            engine.layer_failures()
        );
        assert_eq!(
            engine.drawn_layers(),
            count as usize + 1,
            "all requested mesh copies reach rendering"
        );
        let after = engine.surface_work();
        assert_eq!(
            after.main_runs - before.main_runs,
            1,
            "mirror copies need no backdrop barrier"
        );
        assert_eq!(after.backdrop_copies - before.backdrop_copies, 0);
        assert_eq!(
            after.mesh_batches - before.mesh_batches,
            1,
            "one main batch"
        );
        eprintln!("REPEATED_MIRRORS count={count} total_with_readback_us={} main_runs=1 backdrop_copies=0 mesh_batches=1",started.elapsed().as_micros());
        let before = engine.surface_work();
        engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = engine.surface_work();
        assert_eq!(after.main_runs - before.main_runs, 1, "the view is drawn again, count={count}");
    }
}

/// Standard glass refracts what is below it (2026-09-23): overlapping copies of one glass layer
/// are one group reading one backdrop; they do not refract each other.
#[test]
fn overlapping_glass_copies_read_one_shared_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects {
        layer: mesh,
        effects: vec![
            EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.glass".into(),
            },
            EffectInstance {
                id: EffectId(1),
                plugin_id: placement::REPEAT.into(),
            },
        ],
    })
    .unwrap();
    effect(&mut doc, mesh, 0, "ior", Value::F64(1.0));
    effect(&mut doc, mesh, 0, "roughness", Value::F64(0.0));
    effect(&mut doc, mesh, 1, "count", Value::F64(3.0));
    effect(&mut doc, mesh, 1, "position_each", Value::Vec2([0.0, 0.0]));
    let mut engine = Engine::new().unwrap();
    let a = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert!(
        engine.layer_failures().is_empty(),
        "{:?}",
        engine.layer_failures()
    );
    let first = engine.surface_work();
    assert_eq!(first.backdrop_copies, 1, "three glass copies, one backdrop");
    let b = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let second = engine.surface_work();
    // The backdrop is the view's picture: each draw of the view copies it again.
    assert_eq!(second.backdrop_copies - first.backdrop_copies, 1);
    assert_eq!(a, b);
    let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
    assert!(
        a[i] > 240 && a[i + 1] > 240,
        "unit-index glass preserves the white background: {:?}",
        &a[i..i + 4]
    );
}

#[test]
fn gpu_instance_sharing_matches_rebuilds_and_uploads_each_copy_once() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
        EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
        EffectInstance { id: EffectId(1), plugin_id: placement::REPEAT.into() },
    ] }).unwrap();
    set(&mut doc, mesh, property::POSITION, Value::Vec2([0.0, 0.0]));
    set(&mut doc, mesh, property::SCALE, Value::Vec2([0.7, 0.7]));
    effect(&mut doc, mesh, 0, "transmission", Value::F64(0.0));
    effect(&mut doc, mesh, 0, "metallic", Value::F64(1.0));
    effect(&mut doc, mesh, 1, "mode", Value::F64(2.0));
    effect(&mut doc, mesh, 1, "columns", Value::F64(32.0));
    effect(&mut doc, mesh, 1, "position_each", Value::Vec2([2.0, 2.0]));
    let mut oracle = Engine::new().unwrap();
    let mut shared = Engine::new().unwrap();
    oracle.set_gpu_instance_sharing_enabled(false);
    for count in [10, 100, 1000] {
        effect(&mut doc, mesh, 1, "count", Value::F64(count as f64));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = shared.surface_work();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(actual, expected, "shared GPU subset count={count}");
        assert_eq!(after.mesh_instances_uploaded - before.mesh_instances_uploaded, count);
        assert_eq!(after.mesh_batches - before.mesh_batches, 1);
        assert_eq!(shared.drawn_layers(), count as usize + 1);
    }
    for opacity in [0.5, 1.0] {
        set(&mut doc, mesh, property::OPACITY, Value::F64(opacity));
        effect(&mut doc, mesh, 1, "count", Value::F64(10.0));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(actual, expected, "transparency fallback {opacity}");
    }
    for clipped in [false, true] {
        let mut effects = vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
            EffectInstance { id: EffectId(2), plugin_id: "motolii.turbulent_displace".into() },
        ];
        if clipped { effects.push(EffectInstance { id: EffectId(3), plugin_id: "motolii.clip".into() }); }
        effects.push(EffectInstance { id: EffectId(1), plugin_id: placement::REPEAT.into() });
        doc.apply(Intent::SetEffects { layer: mesh, effects }).unwrap();
        effect(&mut doc, mesh, 2, "amount", Value::F64(0.2));
        effect(&mut doc, mesh, 2, "evolution", Value::F64(0.5));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(expected, actual, "field with clip={clipped}");
        if clipped { assert!(shared.surface_work().mesh_batches - before.mesh_batches > 1); }
    }

}

#[test]
fn gpu_instance_subsets_preserve_rect_mesh_boundaries_and_surface_parameters() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let sender = file_layer(&mut doc, 3, 2, &red);
    set(&mut doc, sender, property::POSITION, Value::Vec2([-300.0, -300.0]));
    set(&mut doc, sender, property::SCALE, Value::Vec2([10.0, 10.0]));
    set(&mut doc, sender, "position.z", Value::F64(-200.0));
    let second = file_layer(&mut doc, 4, 3, &dir.path().join("quad.obj"));
    set(&mut doc, second, property::POSITION, Value::Vec2([35.0, 35.0]));
    set(&mut doc, second, property::SCALE, Value::Vec2([8.0, 8.0]));
    for mesh in [LayerId(2), second] {
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
        ] }).unwrap();
        effect(&mut doc, mesh, 0, "transmission", Value::F64(0.0));
        effect(&mut doc, mesh, 0, "metallic", Value::F64(1.0));
    }
    let mut oracle = Engine::new().unwrap();
    let mut shared = Engine::new().unwrap();
    oracle.set_gpu_instance_sharing_enabled(false);
    for roughness in [0.0, 0.4, 0.0] {
        effect(&mut doc, second, 0, "roughness", Value::F64(roughness));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = shared.surface_work();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(expected, actual, "interleaved main runs roughness={roughness}");
        assert_eq!(after.mesh_instances_uploaded - before.mesh_instances_uploaded, 2);
        assert!(after.main_runs - before.main_runs >= 2);
    }
}

/// A Vism's `VIEWS` are drawn by the host once for the layer's prepared frame, and what they saw
/// reaches the surface: a picture only the Views can see changes the mirror. A Vism asking for no
/// Views (Glass) gets none drawn.
#[test]
fn a_vism_asks_for_views_and_the_host_draws_them() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let sender = file_layer(&mut doc, 3, 0, &red);
    set(&mut doc, sender, "position", Value::Vec2([-300.0, -300.0]));
    set(&mut doc, sender, "scale", Value::Vec2([10.0, 10.0]));
    set(&mut doc, sender, "position.z", Value::F64(-200.0));
    let mesh = LayerId(2);
    let use_effect = |doc: &mut Document, id: &str| {
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: id.into() }] }).unwrap();
    };
    use_effect(&mut doc, "motolii.cube_mirror");
    let mut engine = Engine::new().unwrap();
    let before = engine.surface_work();
    let seen = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    assert_eq!(engine.surface_work().layer_views - before.layer_views, 6, "the six Views the Vism asked for");
    set(&mut doc, sender, "opacity", Value::F64(0.0));
    let unseen = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_ne!(seen, unseen, "what only the Views see reaches the mirror");
    use_effect(&mut doc, "motolii.glass");
    let before = engine.surface_work();
    engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(engine.surface_work().layer_views - before.layer_views, 0, "Glass asks for no View");
}

/// A Repeater's copies are one layer's: its Views are drawn once, however many copies there are,
/// and the copies are left out of what the Views see.
#[test]
fn a_repeaters_copies_share_their_layers_views() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
        EffectInstance { id: EffectId(0), plugin_id: "motolii.cube_mirror".into() },
        EffectInstance { id: EffectId(1), plugin_id: placement::REPEAT.into() },
    ] }).unwrap();
    set(&mut doc, mesh, property::SCALE, Value::Vec2([0.3, 0.3]));
    effect(&mut doc, mesh, 1, "position_each", Value::Vec2([3.0, 0.0]));
    let mut engine = Engine::new().unwrap();
    for count in [1, 10, 200] {
        effect(&mut doc, mesh, 1, "count", Value::F64(count as f64));
        let before = engine.surface_work().layer_views;
        engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert_eq!(engine.drawn_layers(), count as usize + 1, "every copy is drawn");
        assert_eq!(engine.surface_work().layer_views - before, 6, "{count} copies, one layer's six Views");
    }
}

/// `SurfaceIn::uv` means the same on every picture: a Checker of 2×2 cells lights the same corners
/// of an image, a shape (a path mesh, whose texcoords are path points) and an extruded solid.
#[test]
fn a_surfaces_uv_is_the_same_on_an_image_a_shape_and_a_solid() {
    use crate::doc::store::{Composition, Fps, LayerMeta, LayerTiming, PathSource, Shape, ShapeNode};
    use crate::doc::vector::{Brush, Fill, Point, Rgb};
    let dir = tempfile::tempdir().unwrap();
    let white = dir.path().join("white.png");
    image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 255, 255, 255])).save(&white).unwrap();
    let corners = |kind: &str| -> [bool; 4] {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        let source = if kind == "image" { LayerSource::File { path: white.to_string_lossy().into_owned(), fingerprint: None } } else { LayerSource::Shape };
        doc.apply_all([Intent::AddLayer(layer), Intent::SetMeta { layer, meta: LayerMeta { source, order: 0, timing: LayerTiming::place(0, None, 1) } }]).unwrap();
        if kind != "image" {
            let fill = Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() };
            doc.apply(Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 32.0, y: 32.0 } }, ops: Vec::new(), stroke: None, fill: Some(fill) })] }).unwrap();
        }
        set(&mut doc, layer, property::POSITION, Value::Vec2([16.0, 16.0]));
        // Off the picture plane, as a layer placed in depth is: drawn as itself, not as a picture of it.
        set(&mut doc, layer, "position.z", Value::F64(-8.0));
        let mut effects = Vec::new();
        if kind == "solid" { effects.push(EffectInstance { id: EffectId(1), plugin_id: crate::extensions::solid::EXTRUDE.into() }); }
        effects.push(EffectInstance { id: EffectId(0), plugin_id: "motolii.checker".into() });
        doc.apply(Intent::SetEffects { layer, effects }).unwrap();
        effect(&mut doc, layer, 0, "cells", Value::F64(2.0));
        effect(&mut doc, layer, 0, "dark", Value::F64(0.4));
        let mut engine = Engine::new().unwrap();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{kind}: {:?}", engine.layer_failures());
        // Each kind places its box its own way; the cells are read within the box it drew.
        let value = |x: u32, y: u32| pixels[((y * 64 + x) * 4 + 1) as usize];
        let drawn: Vec<(u32, u32)> = (0..64).flat_map(|y| (0..64).map(move |x| (x, y))).filter(|&(x, y)| value(x, y) > 20).collect();
        let (x0, x1) = (drawn.iter().map(|p| p.0).min().unwrap(), drawn.iter().map(|p| p.0).max().unwrap());
        let (y0, y1) = (drawn.iter().map(|p| p.1).min().unwrap(), drawn.iter().map(|p| p.1).max().unwrap());
        // A cell is one colour throughout (not stripes of some other unit): lit or dark as a whole.
        let cell = |cx: u32, cy: u32| {
            let (w, h) = ((x1 - x0) / 2, (y1 - y0) / 2);
            let inside: Vec<bool> = (y0 + cy * h + 2..y0 + (cy + 1) * h - 2)
                .flat_map(|y| (x0 + cx * w + 2..x0 + (cx + 1) * w - 2).map(move |x| (x, y)))
                .map(|(x, y)| value(x, y) > 180).collect();
            let lit = inside.iter().filter(|l| **l).count() as f32 / inside.len() as f32;
            assert!(lit > 0.95 || lit < 0.05, "{kind}: cell ({cx}, {cy}) is {:.0}% lit — not one cell", lit * 100.0);
            lit > 0.5
        };
        [cell(0, 0), cell(1, 0), cell(0, 1), cell(1, 1)]
    };
    let image = corners("image");
    assert_eq!(image, [true, false, false, true], "the image's cells");
    assert_eq!(corners("shape"), image, "a shape's uv is its box's, as an image's");
    assert_eq!(corners("solid"), image, "a solid's front is its box's too");
}
