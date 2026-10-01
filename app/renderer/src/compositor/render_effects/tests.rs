use super::*;
use motolii_edit::{Animate, Document, Intent};
use crate::doc::store::{
    Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};
use crate::render::engine::Engine;

fn document_with_effects(path: &std::path::Path, plugins: &[&str]) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply_all([
        Intent::SetComposition(Composition {
            width: 24,
            height: 24,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0; 4],
        }),
        Intent::AddLayer(LayerId(1)),
        Intent::SetMeta {
            layer: LayerId(1),
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetEffects {
            layer: LayerId(1),
            effects: plugins
                .iter()
                .enumerate()
                .map(|(index, plugin)| EffectInstance {
                    id: EffectId(index as u32),
                    plugin_id: (*plugin).into(),
                })
                .collect(),
        },
    ])
    .unwrap();
    doc
}

/// Radiance: 明るい所が光源、形が遮蔽。光は空気中に見え(Air)、遮蔽の裏は暗い。
#[test]
fn radiance_lights_the_air_around_emitters_and_occluders_cast_shadows() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("scene.png");
    let size = 96u32;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let emitter = (20..36).contains(&x) && (40..56).contains(&y);
            let wall = (50..54).contains(&x) && (20..76).contains(&y);
            pixels.extend_from_slice(&if emitter { [255, 255, 255, 255] } else if wall { [0, 0, 0, 255] } else { [0, 0, 0, 0] });
        }
    }
    image::save_buffer(&source, &pixels, size, size, image::ColorType::Rgba8).unwrap();
    let mut doc = document_with_effects(&source, &["motolii.radiance"]);
    doc.apply(Intent::SetComposition(Composition {
        width: size,
        height: size,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    for (name, value) in [("air", 1.0), ("intensity", 4.0), ("radius", 16.0)] {
        doc.apply(Intent::SetConstant {
            layer: LayerId(1),
            property: PropertyId::effect_param(EffectId(0), name).unwrap(),
            value: Value::F64(value),
        })
        .unwrap();
    }
    let mut engine = Engine::new().unwrap();
    let scope = engine.gpu_device().push_error_scope(wgpu::ErrorFilter::Validation);
    let lit = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    let error = pollster::block_on(scope.pop());
    assert!(error.is_none(), "radiance passes must validate: {error:?}");
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let at = |frame: &[u8], x: u32, y: u32| frame[((y * size + x) * 4) as usize];
    let beside = at(&lit, 42, 48);
    let behind_wall = at(&lit, 60, 48);
    let far_corner = at(&lit, 90, 6);
    assert!(beside > 20, "air next to the emitter must be lit: {beside}");
    assert!(behind_wall < beside / 2, "the wall must shadow the far side: beside {beside}, behind {behind_wall}");
    assert!(far_corner < beside, "light falls off with distance: corner {far_corner}, beside {beside}");

    doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "intensity").unwrap(), value: Value::F64(0.0) }).unwrap();
    let dark = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(doc.undo());
    let relit = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(relit, lit, "GPU reduction must not retain a stale no-emitter result");
    doc.apply(Intent::SetEffects { layer: LayerId(1), effects: Vec::new() }).unwrap();
    let plain = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(dark, plain, "without emission no light can be added");
    assert_eq!(at(&plain, 42, 48), 0, "without the effect the air is dark");
}

#[test]
fn radiance_preaverage_preserves_reference_pixels() {
    use crate::render::compositor::effects::{self, VismSource};
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.png");
    let size = 96u32;
    let mut pixels = vec![0; (size * size * 4) as usize];
    for y in 32..48 { for x in 20..36 { let p = ((y*size+x)*4) as usize; pixels[p..p+4].copy_from_slice(&[255,255,255,255]); } }
    image::save_buffer(&source, &pixels, size, size, image::ColorType::Rgba8).unwrap();
    let doc = if let Ok(path) = std::env::var("MOTOLII_RADIANCE_BENCH_DOCUMENT") {
        let mut doc = Document::load(path).unwrap().with_programs(crate::extensions::bundled());
        let ids = crate::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO).unwrap().iter().map(|l| l.id).collect::<Vec<_>>();
        for layer in ids { for effect in doc.view().effects(layer).unwrap() { doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_enabled(effect.id), value: Value::Bool(true) }).unwrap(); } }
        doc
    } else {
        let mut doc = document_with_effects(&source, &["motolii.radiance"]);
        doc.apply(Intent::SetComposition(Composition { width:size, height:size, fps:Fps::try_new(30,1).unwrap(), duration_frames:150, background:[0.0,0.0,0.0,1.0] })).unwrap();
        doc
    };
    let fps = doc.view().composition().unwrap().unwrap().fps;
    let old = include_str!("../../../../../reference/radiance-before-2026-09-12.wgsl");
    let mut reference = Engine::new().unwrap();
    let mut definition = reference.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == "motolii.radiance").unwrap().clone();
    let (manifest, body) = effects::isf::parse_isf_source(old).unwrap();
    definition.source = VismSource { name:"radiance-reference".into(), extension:"wgsl".into(), source:old.into() };
    definition.manifest = manifest;
    definition.vertex_text = body.clone(); definition.fragment_text = body;
    definition.stage().unwrap();
    let program = effects::EffectProgram::compile(&reference.compositor.ctx, &definition);
    reference.compositor.effect_programs.insert("motolii.radiance".into(), program);
    let mut candidate = Engine::new().unwrap();
    let mut times = [0.0f64; 2];
    for frame in [0,30,60,90] {
        let t = RationalTime::try_from_frame(frame, fps).unwrap();
        let expected = reference.render_frame(&doc.view(), t).unwrap();
        let actual = candidate.render_frame(&doc.view(), t).unwrap();
        let bad = actual.iter().zip(&expected).filter(|(a,b)| a.abs_diff(**b)>3).count();
        assert_eq!(bad, 0, "pre-averaging must retain image values, frame {frame}");
        for (i, engine) in [&mut reference, &mut candidate].into_iter().enumerate() {
            let start = std::time::Instant::now();
            for _ in 0..3 { engine.render_frame(&doc.view(), t).unwrap(); }
            times[i] += start.elapsed().as_secs_f64()*1000.0;
        }
    }
    eprintln!("radiance reference {:.3} ms, optimized {:.3} ms (including readback)",times[0]/12.0,times[1]/12.0);
}

#[test]
fn mixed_format_effects_consume_previous_output_and_recover_after_undo() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.png");
    let pixels = [32u8, 64, 96, 255]
        .into_iter()
        .cycle()
        .take(24 * 24 * 4)
        .collect::<Vec<_>>();
    image::save_buffer(&source, &pixels, 24, 24, image::ColorType::Rgba8).unwrap();
    let mut doc = document_with_effects(&source, &["motolii.gradient"]);
    let mut engine = Engine::new().unwrap();
    let mut checked_render = |doc: &Document| {
        let scope = engine
            .gpu_device()
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let pixels = engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        let error = pollster::block_on(scope.pop());
        assert!(
            error.is_none(),
            "effect attachments must match pipeline formats: {error:?}"
        );
        pixels
    };
    let gradient = checked_render(&doc);
    assert!(
        gradient.chunks_exact(4).any(|pixel| pixel[0] != pixel[1]),
        "the reference is visibly nonuniform"
    );
    for plugins in [
        vec!["motolii.gradient", "motolii.blur"],
        vec!["motolii.blur", "motolii.gradient"],
        vec!["motolii.gradient", "motolii.blur", "motolii.blur"],
    ] {
        let mut edits = vec![Intent::SetEffects {
            layer: LayerId(1),
            effects: plugins
                .iter()
                .enumerate()
                .map(|(index, plugin)| EffectInstance {
                    id: EffectId(index as u32),
                    plugin_id: (*plugin).into(),
                })
                .collect(),
        }];
        for (index, plugin) in plugins.iter().enumerate() {
            if *plugin == "motolii.blur" {
                edits.push(Intent::SetConstant {
                    layer: LayerId(1),
                    property: PropertyId::effect_param(EffectId(index as u32), "radius")
                        .unwrap(),
                    value: Value::F64(0.0),
                });
            }
        }
        doc.apply_all(edits).unwrap();
        let actual = checked_render(&doc);
        assert_eq!(actual.len(), gradient.len());
        assert!(actual.iter().zip(&gradient).all(|(actual, expected)| actual.abs_diff(*expected) <= 2),
        "a zero-radius blur must preserve the previous Vism output through 8-bit/float format transitions: {plugins:?}");
        assert!(doc.undo());
        assert_eq!(
            checked_render(&doc),
            gradient,
            "removing the chain restores rendering in the same Engine"
        );
    }
}
