use super::environment_tests::{file_layer, scene, sky_png, MESH_X, MESH_Y, SIZE};
use super::*;
use crate::doc::store::{
    placement, property, Document, EffectId, EffectInstance, Intent, LayerId, LayerSource,
    PropertyId, Value,
};

fn set(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::new(name).unwrap(),
        value,
    })
    .unwrap();
}
fn effect(doc: &mut Document, layer: LayerId, id: u32, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::effect_param(EffectId(id), name).unwrap(),
        value,
    })
    .unwrap();
}

/// Local probes include geometry behind the main camera; changing it must invalidate the result.
#[test]
fn shared_reflection_sees_offscreen_objects_and_refreshes_without_history() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let red = dir.path().join("red.png");
    let blue = dir.path().join("blue.png");
    let plate = dir.path().join("plate.png");
    for (path, color, size) in [
        (&red, [255, 0, 0, 255], SIZE),
        (&blue, [0, 0, 255, 255], SIZE),
        (&plate, [255, 255, 255, 255], 24),
    ] {
        image::RgbaImage::from_pixel(size, size, image::Rgba(color))
            .save(path)
            .unwrap();
    }
    let mut engine = Engine::new().unwrap();
    for rectangle in [false, true] {
        let mut doc = scene(dir.path(), &sky, true);
        let receiver = LayerId(2);
        if rectangle {
            doc.apply(Intent::SetSource {
                layer: receiver,
                source: LayerSource::File {
                    path: plate.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
            })
            .unwrap();
            set(&mut doc, receiver, property::SCALE, Value::Vec2([1.0, 1.0]));
        }
        doc.apply(Intent::SetEffects {
            layer: receiver,
            effects: vec![EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.glass".into(),
            }],
        })
        .unwrap();
        for (name, value) in [("roughness", 0.0), ("metallic", 1.0), ("transmission", 0.0)] {
            effect(&mut doc, receiver, 0, name, Value::F64(value));
        }
        let sender = file_layer(&mut doc, 3, 0, &red);
        set(
            &mut doc,
            sender,
            property::POSITION,
            Value::Vec2([-300.0, -300.0]),
        );
        set(&mut doc, sender, property::SCALE, Value::Vec2([10.0, 10.0]));
        set(&mut doc, sender, "position.z", Value::F64(-200.0));
        let mut first = None;
        for (step, path) in [&red, &blue, &red].into_iter().enumerate() {
            doc.apply(Intent::SetSource {
                layer: sender,
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
            })
            .unwrap();
            let pixels = engine
                .render_frame(&doc.view(), RationalTime::ZERO)
                .unwrap();
            assert!(
                engine.layer_failures().is_empty(),
                "{:?}",
                engine.layer_failures()
            );
            let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
            let color = &pixels[i..i + 3];
            if step == 1 {
                assert!(
                    color[2] > 150 && color[0] < 80,
                    "offscreen blue, rectangle={rectangle}: {color:?}"
                );
            } else {
                assert!(
                    color[0] > 150 && color[2] < 80,
                    "offscreen red, rectangle={rectangle}: {color:?}"
                );
            }
            if step == 0 {
                first = Some(pixels);
            } else if step == 2 {
                assert_eq!(
                    first.as_ref().unwrap(),
                    &pixels,
                    "same source state must have identical output"
                );
                let observed = engine
                    .render_with_camera_override(
                        &doc.view(),
                        RationalTime::ZERO,
                        true,
                        Some(ResolvedCamera {
                            orbit_degrees: [15.0, 25.0],
                            ..Default::default()
                        }),
                    )
                    .unwrap();
                assert_ne!(
                    &pixels, &observed,
                    "authored view change must remain visible"
                );
                let restored = engine
                    .render_frame(&doc.view(), RationalTime::ZERO)
                    .unwrap();
                assert_eq!(
                    pixels, restored,
                    "returning to a camera must not retain old reflection history"
                );
            }
        }
    }
}

/// The capture/draw budget belongs to the scene, not to each Repeater copy.
#[test]
fn repeated_mirrors_share_captures_batches_and_do_not_copy_the_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut engine = Engine::new().unwrap();
    engine.set_reflection_cache_enabled(false);
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
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
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
            after.scene_captures - before.scene_captures,
            12,
            "two shared probes, count={count}"
        );
        assert_eq!(
            after.main_runs - before.main_runs,
            1,
            "mirror copies need no backdrop barrier"
        );
        assert_eq!(after.backdrop_copies - before.backdrop_copies, 0);
        assert_eq!(
            after.mesh_batches - before.mesh_batches,
            3,
            "two capture batches and one main batch"
        );
        eprintln!("SHARED_REFLECTION count={count} total_with_readback_us={} captures=12 main_runs=1 backdrop_copies=0 mesh_batches=3",started.elapsed().as_micros());
    }
}

/// Ordered transmission keeps its barriers, but its full-size mip texture is reused.
#[test]
fn overlapping_glass_reuses_one_backdrop_without_removing_transmission_steps() {
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
    assert_eq!(first.backdrop_copies, 3);
    assert_eq!(first.backdrop_allocations, 1);
    let b = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let second = engine.surface_work();
    assert_eq!(second.backdrop_copies - first.backdrop_copies, 3);
    assert_eq!(second.backdrop_allocations, 1);
    assert_eq!(a, b);
    let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
    assert!(
        a[i] > 240 && a[i + 1] > 240,
        "unit-index glass preserves the white background: {:?}",
        &a[i..i + 4]
    );
}

/// Primary-view culling must not discard a lateral reflection sender.
#[test]
fn reflection_keeps_a_sender_outside_the_primary_frustum() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255]))
        .save(&red)
        .unwrap();
    let mut doc = scene(dir.path(), &sky, true);
    let receiver = LayerId(2);
    doc.apply(Intent::SetEffects {
        layer: receiver,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.glass".into(),
        }],
    })
    .unwrap();
    for (name, value) in [("roughness", 0.0), ("metallic", 1.0), ("transmission", 0.0)] {
        effect(&mut doc, receiver, 0, name, Value::F64(value));
    }
    set(&mut doc, receiver, property::ROTATION_Y, Value::F64(45.0));
    let sender = file_layer(&mut doc, 3, 0, &red);
    set(
        &mut doc,
        sender,
        property::POSITION,
        Value::Vec2([-200.0, -100.0]),
    );
    set(
        &mut doc,
        sender,
        property::SCALE,
        Value::Vec2([1.5625, 4.0]),
    );
    set(&mut doc, sender, "position.z", Value::F64(50.0));
    set(&mut doc, sender, property::ROTATION_Y, Value::F64(90.0));
    let mut engine = Engine::new().unwrap();
    let pixels = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let red_pixels = |pixels: &[u8]| {
        pixels
            .chunks_exact(4)
            .filter(|p| p[0] > 150 && p[1] < 80 && p[2] < 80)
            .count()
    };
    assert!(
        red_pixels(&pixels) > 10,
        "mirror must see the lateral red sender"
    );
    doc.apply(Intent::SetEffects {
        layer: receiver,
        effects: vec![],
    })
    .unwrap();
    let plain = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(
        red_pixels(&plain),
        0,
        "the sender is not directly visible in the primary view"
    );
    assert!(
        engine.layer_failures().is_empty(),
        "{:?}",
        engine.layer_failures()
    );
}


/// A cache hit, a miss and eviction must agree with the uncached renderer at the same inputs.
#[test]
fn reflection_cache_matches_uncached_after_edits_undo_and_eviction() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let red = dir.path().join("cache-sender.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let sender = file_layer(&mut doc, 3, 0, &red);
    set(&mut doc, sender, property::POSITION, Value::Vec2([-300.0, -300.0]));
    set(&mut doc, sender, property::SCALE, Value::Vec2([10.0, 10.0]));
    set(&mut doc, sender, "position.z", Value::F64(-200.0));
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance {
        id: EffectId(0), plugin_id: "motolii.glass".into(),
    }] }).unwrap();
    effect(&mut doc, mesh, 0, "transmission", Value::F64(0.0));
    effect(&mut doc, mesh, 0, "metallic", Value::F64(1.0));
    let mut cached = Engine::new().unwrap();
    let mut oracle = Engine::new().unwrap();
    oracle.set_reflection_cache_enabled(false);
    for step in 0..12 {
        match step {
            1 => effect(&mut doc, mesh, 0, "roughness", Value::F64(0.5)),
            2 => set(&mut doc, mesh, "rotation.y", Value::F64(35.0)),
            3 => { assert!(doc.undo()); },
            4 => { assert!(doc.redo()); },
            5 => cached.clear_reflection_cache(),
            6 => set(&mut doc, mesh, property::POSITION, Value::Vec2([30.0, 24.0])),
            7 => set(&mut doc, sender, property::OPACITY, Value::F64(0.5)),
            8 => {
                doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
                    EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
                    EffectInstance { id: EffectId(1), plugin_id: "motolii.turbulent_displace".into() },
                ] }).unwrap();
                effect(&mut doc, mesh, 1, "amount", Value::F64(2.0));
            },
            9 => effect(&mut doc, mesh, 1, "evolution", Value::F64(0.5)),
            10 => {
                doc.apply(Intent::SetEffects { layer: sender, effects: vec![
                    EffectInstance { id: EffectId(2), plugin_id: "motolii.blur".into() },
                ] }).unwrap();
            },
            11 => { doc.apply(Intent::SetEffects { layer: sender, effects: vec![] }).unwrap(); },
            _ => (),
        }
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let miss = cached.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(expected, miss, "edit/eviction step {step}");
        let before = cached.surface_work();
        let hit = cached.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = cached.surface_work();
        assert_eq!(expected, hit, "cache hit step {step}");
        if step == 10 {
            assert!(after.cache_bypasses > before.cache_bypasses, "mutable pass output must bypass");
            continue;
        }
        assert_eq!(after.scene_captures, before.scene_captures);
        assert_eq!(after.cache_hits, before.cache_hits + 1, "step {step}: {after:?}");
        assert!(after.cache_retained_texture_bytes <= 128 * 1024 * 1024);
    }
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
    oracle.set_reflection_cache_enabled(false);
    shared.set_reflection_cache_enabled(false);
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
        assert_eq!(after.scene_captures - before.scene_captures, 12);
        assert_eq!(shared.drawn_layers(), count as usize + 1);
    }
    for opacity in [0.5, 1.0] {
        set(&mut doc, mesh, property::OPACITY, Value::F64(opacity));
        effect(&mut doc, mesh, 1, "count", Value::F64(10.0));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(actual, expected, "transparency fallback {opacity}");
        if opacity < 1.0 { assert!(shared.surface_work().mesh_batches - before.mesh_batches > 1); }
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
    oracle.set_reflection_cache_enabled(false);
    shared.set_reflection_cache_enabled(false);
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

/// MSAA samples geometry coverage; it must not blur fully covered image interiors.
#[test]
fn upstream_msaa_smooths_mesh_and_rectangle_coverage_without_blurring_interiors() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 255, 255);
    let white = dir.path().join("white.png");
    image::RgbaImage::from_pixel(24, 24, image::Rgba([255, 255, 255, 255])).save(&white).unwrap();
    let mut aa = Engine::new().unwrap();
    let mut single = Engine::new().unwrap();
    single.compositor = crate::render::compositor::Compositor::with_device(
        single.gpu_device().clone(), single.gpu_queue().clone(),
        crate::render::compositor::PRESENTABLE_FORMAT,
        |_| re_renderer::RenderConfig { msaa_mode: re_renderer::MsaaMode::Off },
    ).unwrap();
    for rectangle in [false, true] {
        let mut doc = scene(dir.path(), &sky, false);
        doc.apply(Intent::SetAttrs { layer: LayerId(1), patch: crate::doc::store::LayerAttrsPatch {
            hidden: Some(true), ..Default::default()
        } }).unwrap();
        let layer = LayerId(2);
        if rectangle {
            doc.apply(Intent::SetSource { layer, source: LayerSource::File {
                path: white.to_string_lossy().into_owned(), fingerprint: None,
            } }).unwrap();
        }
        set(&mut doc, layer, "anchor", Value::Vec2(if rectangle { [12.0, 12.0] } else { [1.0, 1.0] }));
        set(&mut doc, layer, property::SCALE, Value::Vec2(if rectangle { [1.0,1.0] } else { [12.0,12.0] }));
        set(&mut doc, layer, "rotation", Value::F64(17.0));
        let before = single.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = aa.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(aa.layer_failures().is_empty());
        let peak = before.chunks_exact(4).map(|p| p[0]).max().unwrap();
        assert!(peak > 64);
        let partial = |pixels: &[u8]| pixels.chunks_exact(4).filter(|p| p[0] > 4 && p[0] < peak - 4).count();
        assert!(partial(&after) > partial(&before) + 8, "rectangle={rectangle}: {} -> {} partial pixels", partial(&before), partial(&after));
        for y in 28..36 { for x in 28..36 {
            let i = (y * SIZE as usize + x) * 4;
            assert_eq!(&before[i..i+4], &after[i..i+4], "interior remains sharp");
        } }
    }
}
