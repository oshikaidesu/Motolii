//! 配置効果(motolii.repeat)は他の効果と同じ口から入り、素材を N 個置く。
//! 既定は通り抜け。配置効果の**下**に効果を積んだ時だけ、配置を 1 枚に合わせてから掛かる。
use super::*;
use motolii_edit::{Animate, Document, Intent};
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use crate::doc::store::{
    property, Composition, EffectId, EffectInstance, EffectScope, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};
use crate::extensions::{placement};
use crate::render::engine::{known_effects, Engine};

const SIZE: u32 = 48;
const DOT: u32 = 4;

fn document(path: &std::path::Path, count: f64, below: &[&str]) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: SIZE,
        height: SIZE,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    let repeat = EffectId(0);
    let mut effects = vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }];
    effects.extend(below.iter().enumerate().map(|(i, id)| EffectInstance {
        id: EffectId(i as u32 + 1),
        plugin_id: (*id).to_owned(),
    }));
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                order: 0,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            value: Value::Vec2([4.0, 4.0]),
        },
        Intent::SetEffects { layer, effects },
        Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(count) },
        Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
    ])
    .unwrap();
    doc
}

fn covered(pixels: &[u8]) -> usize {
    pixels.chunks(4).filter(|px| px[3] > 0).count()
}

fn png(dir: &std::path::Path, name: &str, rgba: [u8; 4]) -> std::path::PathBuf {
    let path = dir.join(name);
    let pixels = rgba.into_iter().cycle().take((DOT * DOT * 4) as usize).collect::<Vec<_>>();
    image::save_buffer(&path, &pixels, DOT, DOT, image::ColorType::Rgba8).unwrap();
    path
}

fn add_file_layer(doc: &mut Document, id: u64, order: i16, path: &std::path::Path, parent: Option<LayerId>, clip: bool) -> LayerId {
    use crate::doc::store::LayerAttrsPatch;
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                order,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), clip_to_below: Some(clip), ..Default::default() } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([4.0, 4.0]) },
    ])
    .unwrap();
    layer
}

/// 半透明の緑を、赤の複製に clip した時の画素。二重に描かれていれば赤が 64 まで落ちる。
fn red_floor(pixels: &[u8]) -> u8 {
    pixels.chunks(4).filter(|px| px[3] > 0).map(|px| px[0]).min().unwrap_or(255)
}

#[test]
fn clipping_onto_repeated_copies_is_screen_overlap_from_outside_and_per_copy_inside() {
    let dir = tempfile::tempdir().unwrap();
    let red = png(dir.path(), "red.png", [255, 0, 0, 255]);
    let green = png(dir.path(), "green.png", [0, 255, 0, 128]);
    let mut engine = Engine::new().unwrap();

    // 外から: 赤 3 枚(2 px ずつ重なる)の上に緑を clip。緑は和に 1 回だけ乗るので、重なりでも赤は半分より落ちない。
    let mut outside = document(&red, 3.0, &[]);
    outside.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([2.0, 0.0]) }).unwrap();
    add_file_layer(&mut outside, 2, 1, &green, None, true);
    let pixels = engine.render_frame(&outside.view(), RationalTime::ZERO).unwrap();
    assert_eq!(covered(&pixels), (DOT * (DOT + 4)) as usize, "the clip paints only where the union of copies is");
    assert!(red_floor(&pixels) >= 120, "a half-transparent clip is drawn once over overlapping copies, got red {}", red_floor(&pixels));

    // 中で: グループに赤と(赤へ clip した)緑を入れて丸ごと 2 枚に増やす。緑は自分の番号の赤にだけ切られる。
    let mut inside = Document::new().with_programs(crate::extensions::bundled());
    inside.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
    let group = LayerId(10);
    inside.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(0), plugin_id: placement::REPEAT.to_owned() }] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "count").unwrap(), value: Value::F64(2.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
    ]).unwrap();
    add_file_layer(&mut inside, 11, 0, &red, Some(group), false);
    add_file_layer(&mut inside, 12, 1, &green, Some(group), true);
    let pixels = engine.render_frame(&inside.view(), RationalTime::ZERO).unwrap();
    assert_eq!(covered(&pixels), 2 * (DOT * DOT) as usize, "each copy carries its own clipped lyric, nothing leaks outside the copies");
    assert!(red_floor(&pixels) >= 120, "inside a copy the clip is drawn once, got red {}", red_floor(&pixels));
}

/// グループの効果は host が解く。Each なら子がそのまま重なり(重なりは 2 回描かれる)、
/// Whole を 1 つ積むと子は 1 枚に焼かれてからグループの不透明度で 1 回だけ乗る。効果(gain=1)は何も知らない。
#[test]
fn a_whole_effect_bakes_the_group_into_one_plate() {
    let dir = tempfile::tempdir().unwrap();
    let red = png(dir.path(), "red.png", [255, 0, 0, 255]);
    let mut engine = Engine::new().unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
    let group = LayerId(10);
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetConstant { layer: group, property: PropertyId::new(property::OPACITY).unwrap(), value: Value::F64(0.5) },
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.gain".to_owned() }] },
    ]).unwrap();
    add_file_layer(&mut doc, 11, 0, &red, Some(group), false);
    let second = add_file_layer(&mut doc, 12, 1, &red, Some(group), false);
    doc.apply(Intent::SetConstant { layer: second, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([6.0, 4.0]) }).unwrap();
    let alpha_max = |pixels: &[u8]| pixels.chunks(4).map(|px| px[3]).max().unwrap();

    // Each(既定): グループの不透明度は子に降りず、重なりも 1 枚ずつ。全部不透明。
    let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(covered(&pixels), (DOT * (DOT + 2)) as usize);
    assert_eq!(alpha_max(&pixels), 255, "pass-through children keep their own opacity");

    // Whole: 2 枚を 1 枚に焼き、グループの 50 % で 1 回乗る。重なりも 50 % のまま。
    doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
    let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(covered(&pixels), (DOT * (DOT + 2)) as usize, "the plate covers the same pixels");
    let alphas: std::collections::BTreeSet<u8> = pixels.chunks(4).filter(|px| px[3] > 0).map(|px| px[3]).collect();
    assert!(alphas.iter().all(|a| (120..=136).contains(a)), "one plate at 50 %, overlap included, got {alphas:?}");
}

/// 背景は世界の板でなく出力の地。カメラを回しても書き出しの全画素が背景色で、素材の無い所に穴は開かない。
#[test]
fn the_background_fills_every_pixel_under_a_tilted_camera() {
    let dir = tempfile::tempdir().unwrap();
    let green = png(dir.path(), "green.png", [0, 255, 0, 255]);
    let mut engine = Engine::new().unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
    add_file_layer(&mut doc, 1, 0, &green, None, false);
    // 注視点(comp 中心)に置く。寄せたカメラでも枠の中に残る。
    doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([SIZE as f64 * 0.5 - DOT as f64 * 0.5; 2]) }).unwrap();
    let camera = LayerId(2);
    doc.apply_all([
        Intent::AddLayer(camera),
        Intent::SetMeta { layer: camera, meta: LayerMeta { source: LayerSource::Camera, order: 1, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_ORBIT).unwrap(), value: Value::Vec2([-25.0, 60.0]) },
        Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_DISTANCE).unwrap(), value: Value::F64(0.5) },
    ]).unwrap();
    let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert_eq!(covered(&pixels), (SIZE * SIZE) as usize, "no hole in the frame");
    let green_px = pixels.chunks(4).filter(|px| px[1] > 200 && px[0] < 50).count();
    assert!(green_px > 0, "the layer is still in view");
    // 縁の画素は赤と緑の混ざり。どの画素も背景か素材の色で、黒や灰(板の外・深度の穴)は無い。
    let stray = pixels.chunks(4).filter(|px| (u32::from(px[0]) + u32::from(px[1])) < 200 || px[2] > 50).count();
    assert_eq!(stray, 0, "every pixel is background or layer");
}

/// 生成器(gradient のように image 入力の無い効果)は素材の形の中に閉じ込められる。
/// Repeat の上なら各複製に、下なら増えた後の全体に付くが、どちらも形は消えない。
#[test]
fn a_generator_stays_inside_the_source_shape_above_and_below_the_placement() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("disc.png");
    let mut pixels = Vec::new();
    for y in 0..DOT * 3 {
        for x in 0..DOT * 3 {
            let inside = (x as f32 - 5.5).powi(2) + (y as f32 - 5.5).powi(2) < 25.0;
            pixels.extend_from_slice(&[255, 255, 255, if inside { 255 } else { 0 }]);
        }
    }
    image::save_buffer(&source, &pixels, DOT * 3, DOT * 3, image::ColorType::Rgba8).unwrap();
    let disc = pixels.chunks(4).filter(|px| px[3] > 0).count();
    let mut engine = Engine::new().unwrap();
    let mut render = |doc: &Document| engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();

    let below = render(&document(&source, 3.0, &["motolii.gradient"]));
    assert_eq!(covered(&below), 3 * disc, "below the placement the gradient fills only the three discs");

    let mut above = document(&source, 3.0, &[]);
    above
        .apply(Intent::SetEffects {
            layer: LayerId(1),
            effects: vec![
                EffectInstance { id: EffectId(1), plugin_id: "motolii.gradient".to_owned() },
                EffectInstance { id: EffectId(0), plugin_id: placement::REPEAT.to_owned() },
            ],
        })
        .unwrap();
    let above = render(&above);
    assert_eq!(covered(&above), 3 * disc, "above the placement each copy is a gradient disc");
    assert!(above.chunks(4).filter(|px| px[3] > 0).any(|px| px[0] != px[1]), "the gradient is visibly painted");
}

#[test]
fn the_repeat_effect_places_the_source_count_times_through_the_effect_stack() {
    assert!(
        known_effects().iter().any(|e| e.plugin_id == placement::REPEAT),
        "the placement effect must sit in the same catalog as the shader effects"
    );
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("dot.png");
    let pixels = [255u8, 0, 0, 255].into_iter().cycle().take((DOT * DOT * 4) as usize).collect::<Vec<_>>();
    image::save_buffer(&source, &pixels, DOT, DOT, image::ColorType::Rgba8).unwrap();
    let mut engine = Engine::new().unwrap();
    let mut render = |doc: &Document| engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();

    let one = render(&document(&source, 1.0, &[]));
    let three = render(&document(&source, 3.0, &[]));
    let area = (DOT * DOT) as usize;
    assert_eq!(covered(&one), area, "one copy is the plain layer");
    assert_eq!(covered(&three), 3 * area, "three copies at step 10 do not overlap and are all drawn");

    // 10 copies at step 10 px in a 48 px comp: the last ones fall outside and are not built.
    let far = render(&document(&source, 10.0, &[]));
    assert_eq!(covered(&far), 5 * area, "only the copies inside the frame paint (x = 4 … 44; 54 and beyond are outside)");
    let mut counting = Engine::new().unwrap();
    counting.render_frame(&document(&source, 10.0, &[]).view(), RationalTime::ZERO).unwrap();
    assert_eq!(counting.drawn_layers(), 5, "copies wholly outside the frame are culled before they are built");

    let mut blurred = document(&source, 3.0, &["motolii.blur"]);
    blurred
        .apply(Intent::SetConstant {
            layer: LayerId(1),
            property: PropertyId::effect_param(EffectId(1), "radius").unwrap(),
            value: Value::F64(2.0),
        })
        .unwrap();
    let three_then_blur = render(&blurred);
    assert!(
        covered(&three_then_blur) > 3 * area,
        "a blur below the placement runs on the assembled picture and spreads past every copy"
    );
    assert!(
        three.chunks(4).zip(three_then_blur.chunks(4)).all(|(sharp, soft)| sharp[3] == 0 || soft[3] > 0),
        "every copy is still present under the blur"
    );
}
