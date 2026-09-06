mod testkit;

use motolii::doc::store::{
    Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, Matte, MatteMode, RationalTime,
};
use motolii::render::engine::Engine;

fn add_image_layer(doc: &mut Document, id: u64, path: &std::path::Path, order: i16) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                name: Some(format!("Layer {id}")),
                ..Default::default()
            },
        },
    ])
    .unwrap();
    layer
}

#[test]
fn choosing_a_matte_changes_the_product_pixels() {
    let dir = testkit::tmp_dir("matte-relationship");
    let target_path = dir.join("target.png");
    let matte_path = dir.join("matte.png");
    let width = 8u32;
    let height = 8u32;

    let target_pixels = vec![255u8, 0, 0, 255]
        .into_iter()
        .cycle()
        .take((width * height * 4) as usize)
        .collect::<Vec<_>>();
    let mut matte_pixels = Vec::with_capacity((width * height * 4) as usize);
    for _y in 0..height {
        for x in 0..width {
            matte_pixels.extend_from_slice(&[255, 255, 255, if x < width / 2 { 255 } else { 0 }]);
        }
    }
    image::save_buffer(
        &target_path,
        &target_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    image::save_buffer(
        &matte_path,
        &matte_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width,
        height,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();
    let target = add_image_layer(&mut doc, 1, &target_path, 0);
    let source = add_image_layer(&mut doc, 2, &matte_path, 1);
    let at = RationalTime::ZERO;

    let before = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), at)
        .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: target,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: source,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();
    let after = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), at)
        .unwrap();

    assert_ne!(
        before, after,
        "matte relation did not reach the rendered picture"
    );
    let left_alpha = after[((height / 2 * width + width / 4) * 4 + 3) as usize];
    let right_alpha = after[((height / 2 * width + width * 3 / 4) * 4 + 3) as usize];
    assert!(
        left_alpha > right_alpha,
        "matte coverage was not applied: left={left_alpha} right={right_alpha}"
    );
}

#[test]
fn target_effect_stays_inside_the_alpha_matte() {
    let dir = testkit::tmp_dir("effect-before-matte");
    let target_path = dir.join("target.png");
    let matte_path = dir.join("matte.png");
    let width = 8u32;
    let height = 8u32;

    let target_pixels = [255u8, 0, 0, 255]
        .into_iter()
        .cycle()
        .take((width * height * 4) as usize)
        .collect::<Vec<_>>();
    let mut matte_pixels = Vec::with_capacity((width * height * 4) as usize);
    for _y in 0..height {
        for x in 0..width {
            matte_pixels.extend_from_slice(&[255, 255, 255, if x < width / 2 { 255 } else { 0 }]);
        }
    }
    image::save_buffer(
        &target_path,
        &target_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    image::save_buffer(
        &matte_path,
        &matte_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width,
        height,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();
    let target = add_image_layer(&mut doc, 1, &target_path, 0);
    let source = add_image_layer(&mut doc, 2, &matte_path, 1);
    doc.apply(Intent::SetEffects {
        layer: target,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.gradient".to_owned(),
        }],
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: target,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: source,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let frame = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let inside_alpha = frame[((height / 2 * width + width / 4) * 4 + 3) as usize];
    let outside_alpha = frame[((height / 2 * width + width * 3 / 4) * 4 + 3) as usize];
    assert!(
        inside_alpha > 0,
        "the effected target vanished inside its matte"
    );
    assert_eq!(
        outside_alpha, 0,
        "target effect escaped the matte coverage: outside alpha={outside_alpha}"
    );
}

#[test]
fn matte_source_effect_contributes_to_coverage() {
    let dir = testkit::tmp_dir("matte-source-effect");
    let target_path = dir.join("target.png");
    let matte_path = dir.join("matte.png");
    let width = 8u32;
    let height = 8u32;

    let target_pixels = [255u8, 0, 0, 255]
        .into_iter()
        .cycle()
        .take((width * height * 4) as usize)
        .collect::<Vec<_>>();
    let mut matte_pixels = Vec::with_capacity((width * height * 4) as usize);
    for _y in 0..height {
        for x in 0..width {
            matte_pixels.extend_from_slice(&[255, 255, 255, if x < width / 2 { 255 } else { 0 }]);
        }
    }
    image::save_buffer(
        &target_path,
        &target_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();
    image::save_buffer(
        &matte_path,
        &matte_pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width,
        height,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();
    let target = add_image_layer(&mut doc, 1, &target_path, 0);
    let source = add_image_layer(&mut doc, 2, &matte_path, 1);
    doc.apply(Intent::SetEffects {
        layer: source,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.gradient".to_owned(),
        }],
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: target,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: source,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let frame = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let former_transparent_side = frame[((height / 2 * width + width * 3 / 4) * 4 + 3) as usize];
    assert!(
        former_transparent_side > 0,
        "the matte source Effect was ignored: right alpha={former_transparent_side}"
    );
}

#[test]
fn clipping_preserves_its_visible_base_and_never_draws_without_coverage() {
    let dir = testkit::tmp_dir("clip-to-below");
    let base_path = dir.join("base.png");
    let upper_path = dir.join("upper.png");
    let (width, height) = (16u32, 16u32);
    let mut base_pixels = Vec::new();
    let mut upper_pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            base_pixels.extend_from_slice(&[0u8, 255, 0, if x < width / 2 { 128 } else { 0 }]);
            upper_pixels.extend_from_slice(&[255u8, 0, 0, if y < height / 2 { 255 } else { 0 }]);
        }
    }
    for (path, pixels) in [(&base_path, &base_pixels), (&upper_path, &upper_pixels)] {
        image::save_buffer(path, pixels, width, height, image::ColorType::Rgba8).unwrap();
    }
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width, height, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1,
        background: [0.0; 4],
    })).unwrap();
    let base = add_image_layer(&mut doc, 1, &base_path, 0);
    let upper = add_image_layer(&mut doc, 2, &upper_path, 1);
    doc.apply(Intent::SetAttrs {
        layer: upper,
        patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() },
    }).unwrap();
    let mut engine = Engine::new().unwrap();
    let clipped = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    let pixel = |x: u32, y: u32| {
        let start = ((y * width + x) * 4) as usize;
        &clipped[start..start + 4]
    };
    assert!(pixel(3, 3)[0] > 60 && pixel(3, 3)[1] < 15, "upper color missing inside base");
    assert!((120..=136).contains(&pixel(3, 3)[3]), "clipping inflated the base alpha: {:?}", pixel(3, 3));
    assert!(pixel(3, 12)[1] > 60 && (120..=136).contains(&pixel(3, 12)[3]), "base was consumed like a legacy matte");
    assert_eq!(pixel(12, 3)[3], 0, "upper escaped base coverage");

    let exported = motolii::render::export::export_lottie(&doc.view()).unwrap();
    assert!(exported.unsupported.iter().any(|issue| issue.layer == Some(upper) && issue.category == "clipping"));
    assert!(exported.json["layers"].as_array().unwrap().iter().all(|layer| layer.get("td").is_none()));

    let backdrop_path = dir.join("backdrop.png");
    let backdrop_pixels: Vec<u8> = [0u8, 0, 255, 255].into_iter().cycle().take((width * height * 4) as usize).collect();
    image::save_buffer(&backdrop_path, &backdrop_pixels, width, height, image::ColorType::Rgba8).unwrap();
    let backdrop = add_image_layer(&mut doc, 3, &backdrop_path, -1);
    let with_backdrop = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    let outside = ((3 * width + 12) * 4) as usize;
    assert!(with_backdrop[outside] < 15 && with_backdrop[outside + 2] > 240, "unrelated backdrop became clipping coverage");
    let inside = ((3 * width + 3) * 4) as usize;
    assert!(with_backdrop[inside] > 60 && with_backdrop[inside + 2] > 60, "clipping group failed to composite translucently over backdrop");
    doc.apply(Intent::RemoveLayer(backdrop)).unwrap();

    doc.apply(Intent::SetAttrs {
        layer: base,
        patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() },
    }).unwrap();
    let hidden = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(hidden.chunks_exact(4).all(|pixel| pixel[3] == 0), "hidden base left unbounded clipping");
    doc.apply(Intent::RemoveLayer(base)).unwrap();
    let orphan = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(orphan.chunks_exact(4).all(|pixel| pixel[3] == 0), "missing base left unbounded clipping");
}
