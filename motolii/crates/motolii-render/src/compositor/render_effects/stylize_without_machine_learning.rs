use super::*;
use motolii_edit::{Document, Intent};
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 96;

fn render(pixels: Vec<u8>, effect: &str, params: &[(&str, f64)]) -> Vec<u8> {
    let stem = effect.trim_start_matches("motolii.");
    let errors: Vec<_> = crate::render::compositor::refresh_effect_catalog().errors.into_iter().filter(|e| e.contains(stem)).collect();
    assert!(errors.is_empty(), "{effect} が棚に載らない: {errors:?}");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("in.png");
    image::save_buffer(&path, &pixels, SIZE, SIZE, image::ColorType::Rgba8).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: effect.into() }] },
    ]).unwrap();
    for (name, value) in params {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(*value) }).unwrap();
    }
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{effect}: {:?}", engine.layer_failures());
    frame
}

fn image(f: impl Fn(u32, u32) -> u8) -> Vec<u8> {
    (0..SIZE * SIZE).flat_map(|i| { let v = f(i % SIZE, i / SIZE); [v, v, v, 255] }).collect()
}
fn at(frame: &[u8], x: u32, y: u32) -> u8 { frame[((y * SIZE + x) * 4) as usize] }
fn spread(frame: &[u8], xs: std::ops::Range<u32>) -> f64 {
    let values: Vec<f64> = (20..76).flat_map(|y| xs.clone().map(move |x| (y, x))).map(|(y, x)| at(frame, x, y) as f64).collect();
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

#[test]
fn depth_map_is_nearer_lower_in_the_frame() {
    let flat = image(|_, _| 128);
    let frame = render(flat.clone(), "motolii.depth_map", &[]);
    let (top, bottom) = (at(&frame, 48, 6), at(&frame, 48, 90));
    assert!(bottom > top + 40, "下ほど近い(白い): 上 {top} 下 {bottom}");
    let inverted = render(flat, "motolii.depth_map", &[("invert", 1.0)]);
    assert!(at(&inverted, 48, 6) > at(&inverted, 48, 90) + 40, "Invert で上下が逆");
}

#[test]
fn kuwahara_flattens_texture_and_keeps_the_edge() {
    let noisy = image(|x, y| {
        let n = ((x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)) % 81) as i32 - 40;
        let base = if x < 48 { 60 } else { 190 };
        (base + n).clamp(0, 255) as u8
    });
    let frame = render(noisy.clone(), "motolii.kuwahara", &[]);
    for (half, xs) in [("暗い側", 8..40), ("明るい側", 56..88)] {
        let (before, after) = (spread(&noisy, xs.clone()), spread(&frame, xs));
        assert!(after < before * 0.5, "{half}: ざらつきが減る {before:.1} → {after:.1}");
    }
    assert!(at(&frame, 44, 48) < 110 && at(&frame, 52, 48) > 150, "縁は切り立ったまま: {} | {}", at(&frame, 44, 48), at(&frame, 52, 48));
}

fn rgba(f: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
    (0..SIZE * SIZE).flat_map(|i| { let [r, g, b] = f(i % SIZE, i / SIZE); [r, g, b, 255] }).collect()
}
fn px(frame: &[u8], x: u32, y: u32) -> [u8; 3] { let i = ((y * SIZE + x) * 4) as usize; [frame[i], frame[i + 1], frame[i + 2]] }

#[test]
fn hue_saturation_turns_red_blue_and_drains_to_grey() {
    let red = rgba(|_, _| [220, 40, 40]);
    let blue = px(&render(red.clone(), "motolii.hue_saturation", &[("hue", -120.0)]), 48, 48);
    assert!(blue[2] > 180 && blue[0] < 80, "Master Hue -120: red becomes blue {blue:?}");
    let grey = px(&render(red, "motolii.hue_saturation", &[("saturation", -100.0)]), 48, 48);
    assert!(grey[0].abs_diff(grey[1]) < 4 && grey[1].abs_diff(grey[2]) < 4, "Master Saturation -100: grey {grey:?}");
}

#[test]
fn tint_maps_dark_and_light_to_two_colours_and_invert_flips() {
    let ramp = image(|x, _| if x < 48 { 0 } else { 255 });
    let blueprint = render(ramp.clone(), "motolii.tint", &[]);
    assert!(px(&blueprint, 10, 48)[0] < 20 && px(&blueprint, 80, 48)[0] > 235, "default tint keeps black and white");
    let inverted = render(ramp, "motolii.invert", &[]);
    assert!(px(&inverted, 10, 48)[0] > 235 && px(&inverted, 80, 48)[0] < 20, "Invert: black becomes white");
}

#[test]
fn mosaic_flattens_blocks_and_block_dissolve_drops_some() {
    let stripes = image(|x, _| if x % 4 < 2 { 0 } else { 255 });
    let blocks = render(stripes.clone(), "motolii.mosaic", &[("columns", 8.0), ("rows", 8.0), ("sharp", 1.0)]);
    let cell = SIZE / 8;
    assert!((0..cell).all(|x| at(&blocks, x, 5) == at(&blocks, 0, 5)), "one block, one colour");
    let white = image(|_, _| 255);
    let half = render(white.clone(), "motolii.block_dissolve", &[("completion", 50.0), ("block_width", 12.0), ("block_height", 12.0)]);
    let gone = (0..SIZE).step_by(12).flat_map(|y| (0..SIZE).step_by(12).map(move |x| (x + 6, y + 6))).filter(|&(x, y)| x < SIZE && y < SIZE && at(&half, x, y) < 20).count();
    assert!((12..52).contains(&gone), "about half the blocks are gone: {gone} of 64");
    assert_eq!(half, render(white, "motolii.block_dissolve", &[("completion", 50.0), ("block_width", 12.0), ("block_height", 12.0)]), "the same blocks every time");
}

#[test]
fn linear_wipe_clears_from_the_left_at_ninety_degrees() {
    let white = image(|_, _| 255);
    let wiped = render(white, "motolii.linear_wipe", &[("completion", 50.0), ("angle", 90.0)]);
    assert!(at(&wiped, 10, 48) < 20 && at(&wiped, 86, 48) > 235, "left half gone, right half kept: {} {}", at(&wiped, 10, 48), at(&wiped, 86, 48));
}

#[test]
fn noise_roughens_a_flat_picture() {
    let flat = image(|_, _| 128);
    let grainy = render(flat.clone(), "motolii.noise", &[("amount", 40.0)]);
    assert!(spread(&grainy, 20..76) > spread(&flat, 20..76) + 10.0, "grain appears");
}

#[test]
fn xdog_draws_a_line_on_the_edge_and_leaves_flat_areas_white() {
    let square = image(|x, y| if (28..68).contains(&x) && (28..68).contains(&y) { 180 } else { 255 });
    let frame = render(square, "motolii.xdog", &[]);
    let (outside, inside, edge) = (at(&frame, 8, 48), at(&frame, 48, 48), (26..31).map(|x| at(&frame, x, 48)).min().unwrap());
    assert!(outside > 240 && inside > 240, "平らな所は白: 外 {outside} 中 {inside}");
    assert!(edge < 150, "縁に線: {edge}");
}
