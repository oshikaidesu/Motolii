use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{
    property, Composition, Fps, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, Value,
};

pub(super) const SIZE: u32 = 64;
/// 網の中(位置 32,32 から scale 12 の板が右下へ広がる)。
pub(super) const MESH_X: u32 = 44;
pub(super) const MESH_Y: u32 = 44;

pub(super) fn file_layer(doc: &mut Document, id: u64, order: i16, path: &std::path::Path) -> LayerId {
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
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            value: Value::Vec2([SIZE as f64 / 2.0, SIZE as f64 / 2.0]),
        },
    ])
    .unwrap();
    layer
}

/// 上半分 `top`、下半分 `bottom` の等距円筒図。
pub(super) fn sky_png(dir: &std::path::Path, name: &str, top: u8, bottom: u8) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut img = image::RgbaImage::new(8, 4);
    for (_, y, px) in img.enumerate_pixels_mut() {
        let v = if y < 2 { top } else { bottom };
        *px = image::Rgba([v, v, v, 255]);
    }
    img.save(&path).unwrap();
    path
}

pub(super) fn scene(dir: &std::path::Path, sky: &std::path::Path, environment: bool) -> Document {
    let obj = dir.join("quad.obj");
    // 法線はカメラ向き(世界の -z)。法線の無い obj は陰影が付かないので照明の test にならない。
    std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 0 -1\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: SIZE,
        height: SIZE,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let sky_layer = file_layer(&mut doc, 1, 0, sky);
    let mesh = file_layer(&mut doc, 2, 1, &obj);
    doc.apply(Intent::SetConstant {
        layer: mesh,
        property: PropertyId::new(property::SCALE).unwrap(),
        value: Value::Vec2([12.0, 12.0]),
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: sky_layer,
        patch: LayerAttrsPatch { environment: Some(environment), ..Default::default() },
    })
    .unwrap();
    doc
}

fn luma(pixels: &[u8], x: u32, y: u32) -> u8 {
    pixels[((y * SIZE + x) * 4) as usize]
}

fn ascii(pixels: &[u8]) -> String {
    (0..SIZE)
        .step_by(4)
        .map(|y| (0..SIZE).step_by(2).map(|x| b" .:-=+*#%@"[luma(pixels, x, y) as usize * 10 / 256] as char).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 環境層は板にならず、空として背景に敷かれ、網をその空で照らす。
/// 上が白・下が黒の空: 画面の上は白、下は黒、正面を向いた網は照度 1/2 の灰。
#[test]
fn an_environment_layer_lights_the_mesh_and_fills_the_background() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 255, 0);
    let mut engine = Engine::new().unwrap();

    let doc = scene(dir.path(), &sky, true);
    let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let art = ascii(&pixels);
    assert!(luma(&pixels, 2, 2) >= 250, "上端は空の白\n{art}");
    assert!(luma(&pixels, 2, SIZE - 3) <= 5, "下端は空の黒\n{art}");
    let center = luma(&pixels, MESH_X, MESH_Y);
    assert!((150..=215).contains(&center), "正面の網は照度 1/2 (sRGB ≈ 188)、got {center}\n{art}");

    let plain = scene(dir.path(), &sky, false);
    let pixels = engine.render_frame(&plain.view(), RationalTime::ZERO).unwrap();
    let art = ascii(&pixels);
    assert!(luma(&pixels, 2, SIZE - 3) <= 5 && luma(&pixels, 2, 2) <= 5, "属性を外せば空は敷かれない\n{art}");
    let unlit = luma(&pixels, MESH_X, MESH_Y);
    assert!(unlit > 5 && unlit != center, "属性を外せば固定の灯に戻る、got {unlit}\n{art}");
}

/// 面の応え方: 同じ空(下だけ白)と上を向いた面で、艶消しは薄く、鏡は上の黒を映し、
/// ガラスは屈折して下の白を通す。Glass は効果棚の 1 枚で、param は property。
#[test]
fn glass_and_mirror_answer_the_environment_differently_from_matte() {
    use crate::doc::store::{EffectId, EffectInstance};
    const GLASS: &str = "motolii.glass";
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "floor.png", 0, 255);
    let obj = dir.path().join("tilted.obj");
    // 法線は上(世界の -y)とカメラ(-z)の間。
    std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 -0.7071 -0.7071\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
    let mut engine = Engine::new().unwrap();
    let render = |engine: &mut Engine, surface: &[(&str, f64)]| -> u8 {
        let mut doc = scene(dir.path(), &sky, true);
        std::fs::copy(&obj, dir.path().join("quad.obj")).unwrap();
        let mesh = LayerId(2);
        if !surface.is_empty() {
            doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: GLASS.into() }] }).unwrap();
            for (name, value) in surface {
                doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
            }
        }
        engine.models.clear();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        luma(&pixels, MESH_X, MESH_Y)
    };
    let matte = render(&mut engine, &[]);
    let mirror = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)]);
    let glass = render(&mut engine, &[("ior", 1.5), ("roughness", 0.0), ("transmission", 1.0)]);
    assert!(mirror < matte && matte < glass, "mirror {mirror} matte {matte} glass {glass}");
    assert!(mirror <= 20, "鏡は上の黒を映す、got {mirror}");
    assert!(glass >= 150, "ガラスは下の白を通す、got {glass}");
}

/// ガラスは背後に描かれた層を屈折して通す。白い空の前に赤い板、その手前のガラス越しに赤が見える。
/// 鏡にすれば空の白を映して赤くならない。
#[test]
fn glass_refracts_the_layers_drawn_behind_it() {
    use crate::doc::store::{EffectId, EffectInstance};
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let red = dir.path().join("red.png");
    let mut img = image::RgbaImage::new(SIZE, SIZE);
    for px in img.pixels_mut() { *px = image::Rgba([255, 0, 0, 255]); }
    img.save(&red).unwrap();
    let mut engine = Engine::new().unwrap();
    let render = |engine: &mut Engine, surface: &[(&str, f64)], opacity: f64| -> [u8; 3] {
        let mut doc = scene(dir.path(), &sky, true);
        let board = file_layer(&mut doc, 3, 0, &red);
        doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
        for (name, value) in surface {
            doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
        }
        doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(property::OPACITY).unwrap(), value: Value::F64(opacity) }).unwrap();
        engine.models.clear();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2]]
    };
    let glass = render(&mut engine, &[("ior", 1.5), ("roughness", 0.0), ("transmission", 1.0), ("metallic", 0.0)], 1.0);
    assert!(glass[0] > 150 && glass[1] < 80, "ガラス越しに赤い板が見える、got {glass:?}");
    let mirror = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)], 1.0);
    assert!(mirror[1] > 150, "鏡は白い空を映す、got {mirror:?}");
    let half = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)], 0.5);
    assert!(half[1] > 80 && half[1] < mirror[1] - 10, "mesh coverage attenuates reflection, like the rectangle: half {half:?}, solid {mirror:?}");
}

/// 分散は背後の像を波長で分ける(KHR_materials_dispersion、three.js の読み)。灰色しか無い場面 —
/// 白い空、左黒右白の板、その手前の斜めのガラス — は dispersion 0 なら灰のまま、
/// 分散を入れると境目で赤と青が割れ、強めるほど伸びる。
#[test]
fn dispersion_splits_the_backdrop_edge_into_colors() {
    use crate::doc::store::{EffectId, EffectInstance};
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let edge = dir.path().join("edge.png");
    let mut img = image::RgbaImage::new(SIZE, SIZE);
    for (x, _, px) in img.enumerate_pixels_mut() {
        let v = if x < SIZE / 2 + 8 { 0 } else { 255 };
        *px = image::Rgba([v, v, v, 255]);
    }
    img.save(&edge).unwrap();
    let mut engine = Engine::new().unwrap();
    let mut spread = |dispersion: f64| -> u8 {
        let mut doc = scene(dir.path(), &sky, true);
        // 正面の板では屈折が曲がらず色も割れない。板を 45° に傾ける(法線は camera 側の -z 成分を持つ)。
        std::fs::write(dir.path().join("quad.obj"), "v -1 -1 -1\nv 1 -1 1\nv 1 1 1\nv -1 1 -1\nvn 0.7071 0 -0.7071\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
        let board = file_layer(&mut doc, 3, 0, &edge);
        doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
        for (name, value) in [("ior", 3.0), ("roughness", 0.0), ("transmission", 1.0), ("metallic", 0.0), ("dispersion", dispersion)] {
            doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(value) }).unwrap();
        }
        engine.models.clear();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels.chunks(4).map(|p| p[0].abs_diff(p[2])).max().unwrap()
    };
    let none = spread(0.0);
    let glass = spread(2.0);
    let exaggerated = spread(20.0);
    assert!(none <= 1, "灰色の場面は分散 0 で灰のまま、got {none}");
    // 64 px の場面では屈折角の差が画素の何分の一かにしかならない。割れが出て、値に比例して伸びることを見る。
    assert!(glass >= 3, "分散 2 で境目の赤と青が割れ始める、got {glass}");
    assert!(exaggerated > 40 && exaggerated > glass * 4, "分散を強めると割れが伸びる、got {glass} → {exaggerated}");
}

/// backdrop の mip は粗さが読む段までしか焼かない。粗さ 0 のガラスは写し 1 段、粗さ 1 は全段。
#[test]
fn backdrop_mips_stop_where_the_roughness_stops_reading() {
    use crate::doc::store::{EffectId, EffectInstance};
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let mut engine = Engine::new().unwrap();
    let mut levels_per_copy = |roughness: f64| -> u64 {
        let mut doc = scene(dir.path(), &sky, true);
        let board = file_layer(&mut doc, 3, 0, &red);
        doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
        doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new("effect.0.param.roughness").unwrap(), value: Value::F64(roughness) }).unwrap();
        engine.models.clear();
        let before = engine.surface_work();
        engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        let after = engine.surface_work();
        let copies = after.backdrop_copies - before.backdrop_copies;
        assert!(copies > 0, "ガラスは backdrop を写す");
        (after.backdrop_mip_levels - before.backdrop_mip_levels) / copies
    };
    let full = u64::from(re_renderer::resource_managers::MipmapGenerator::mip_level_count(SIZE, SIZE));
    assert_eq!(levels_per_copy(0.0), 1, "粗さ 0 は写しだけ");
    let rough = levels_per_copy(1.0);
    assert!(rough > 1 && rough <= full, "粗さ 1 は段を焼く、got {rough} of {full}");
}

/// 光は環境から来て、作者は奪うだけ(裁定 2026-09-10)。空の一点が明るい環境(太陽は camera 側の左上)の
/// 前に赤い板、その手前(camera 側)に板 1 枚。板に Cast Shadow の効果を掛けると、板の右下に影が落ちて
/// 赤が暗くなる。付けなければ変わらない。遠くの赤も変わらない。
#[test]
fn a_layer_with_cast_shadow_darkens_the_board_behind_it() {
    let dir = tempfile::tempdir().unwrap();
    let sky = dir.path().join("sun.png");
    let mut img = image::RgbaImage::from_pixel(8, 4, image::Rgba([40, 40, 40, 255]));
    img.put_pixel(0, 1, image::Rgba([255, 255, 255, 255]));
    img.save(&sky).unwrap();
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let mut engine = Engine::new().unwrap();
    let mut render = |blocks: bool| -> (Vec<u8>, u64) {
        let mut doc = scene(dir.path(), &sky, true);
        let board = file_layer(&mut doc, 3, 0, &red);
        doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        let blocker = LayerId(2);
        doc.apply(Intent::SetConstant { layer: blocker, property: PropertyId::new(property::POSITION_Z).unwrap(), value: Value::F64(-20.0) }).unwrap();
        let effects = if blocks { vec![crate::doc::store::EffectInstance { id: crate::doc::store::EffectId(900), plugin_id: "motolii.cast_shadow".into() }] } else { Vec::new() };
        doc.apply(Intent::SetEffects { layer: blocker, effects }).unwrap();
        engine.models.clear();
        let before = engine.surface_work().light_captures;
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        (pixels, engine.surface_work().light_captures - before)
    };
    let (lit, captures_lit) = render(false);
    let (shaded, captures) = render(true);
    assert_eq!(captures_lit, 0, "遮る層が無ければ型紙は描かない");
    assert_eq!(captures, 1, "遮る層があれば型紙 1 枚");
    let red_at = |p: &[u8], x: u32, y: u32| p[((y * SIZE + x) * 4) as usize];
    // 板(位置 32,32 から 12 倍、camera 側へ 20)の右下、板の外の赤。
    let (sx, sy) = (60, 60);
    assert!(red_at(&shaded, sx, sy) + 20 < red_at(&lit, sx, sy), "影で赤が暗くなる: lit {} shaded {}\n{}", red_at(&lit, sx, sy), red_at(&shaded, sx, sy), ascii(&shaded));
    assert_eq!(red_at(&shaded, 4, 4), red_at(&lit, 4, 4), "光線から外れた赤は変わらない");
}

/// The mesh Glass oracle, applied to a premultiplied 2D surface (GPU Gems 2 ch.19).
/// Coverage is independent of optical transmission, including the mix-mode route.
#[test]
fn shared_surface_glass_refracts_on_rectangles_and_preserves_coverage() {
    use crate::doc::store::{BlendMode, EffectId, EffectInstance};
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let cutout = dir.path().join("cutout.png");
    let mut img = image::RgbaImage::new(SIZE, SIZE);
    for (x, _, px) in img.enumerate_pixels_mut() {
        *px = image::Rgba([255, 255, 255, if x < 16 { 0 } else if x < 32 { 128 } else { 255 }]);
    }
    img.save(&cutout).unwrap();
    let mut doc = scene(dir.path(), &sky, true);
    doc.apply(Intent::RemoveLayer(LayerId(2))).unwrap();
    let board = file_layer(&mut doc, 3, 0, &red);
    let plate = file_layer(&mut doc, 4, 1, &cutout);
    for layer in [board, plate] {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
    }
    doc.apply(Intent::SetEffects { layer: plate, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
    let mut engine = Engine::new().unwrap();
    for blend_mode in [BlendMode::Normal, BlendMode::Screen] {
        doc.apply(Intent::SetAttrs { layer: plate, patch: LayerAttrsPatch { blend_mode: Some(blend_mode), ..Default::default() } }).unwrap();
        for (metallic, transmission) in [(0.0, 1.0), (1.0, 0.0)] {
            for (name, value) in [("ior", 1.5), ("roughness", 0.0), ("metallic", metallic), ("transmission", transmission)] {
                doc.apply(Intent::SetConstant { layer: plate, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(value) }).unwrap();
            }
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            let sample = |x: u32| { let i = ((32 * SIZE + x) * 4) as usize; &pixels[i..i+4] };
            let hole = sample(8);
            assert!(hole[0] > 150 && hole[1] < 10, "transparent hole retains red: {hole:?}, {blend_mode:?}");
            let solid = sample(44);
            if transmission > 0.0 {
                assert!(solid[0] > 150 && solid[1] < 80, "2D glass transmits red: {solid:?}, {blend_mode:?}");
            } else {
                assert!(solid[1] > 150, "2D mirror reflects white environment: {solid:?}, {blend_mode:?}");
                let edge = sample(24);
                assert!(edge[1] > 80 && edge[1] < solid[1] - 10, "half coverage blends reflected radiance: edge {edge:?}, solid {solid:?}");
            }
        }
    }
}

/// exr も hdr と同じ線形 f32 の道を通り、1.0 超が残る。
#[test]
fn exr_keeps_values_above_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sun.exr");
    let mut img = image::Rgb32FImage::new(2, 2);
    for px in img.pixels_mut() { *px = image::Rgb([4.0, 0.5, 1.0]); }
    img.save(&path).unwrap();
    let (rgb, w, h) = crate::render::engine::decode_still_linear_rgb(path.to_str().unwrap()).unwrap();
    assert_eq!((w, h), (2, 2));
    assert!((rgb[0] - 4.0).abs() < 0.05 && (rgb[1] - 0.5).abs() < 0.01, "{:?}", &rgb[..3]);
    assert!(crate::render::media::is_still_image_path(&path), "exr は画の門を通る");
}

/// Turbulent Displace は棚の 1 枚で、網の頂点を GPU で動かす: 掛けた絵は掛けない絵と違い、
/// Evolution を進めるとまた違う(時刻で流れる)。
#[test]
fn turbulent_displace_moves_mesh_vertices_and_evolves() {
    use crate::doc::store::{EffectId, EffectInstance};
    const TURBULENT_DISPLACE: &str = "motolii.turbulent_displace";
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 255, 0);
    let mut engine = Engine::new().unwrap();
    let mut render = |params: Option<&[(&str, f64)]>| -> Vec<u8> {
        let mut doc = scene(dir.path(), &sky, true);
        let mesh = LayerId(2);
        if let Some(params) = params {
            doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: TURBULENT_DISPLACE.into() }] }).unwrap();
            for (name, value) in params {
                doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
            }
        }
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    };
    let differing = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x[0].abs_diff(y[0]) > 8).count();
    let still = render(None);
    let space = render(Some(&[("amount", 30.0), ("size", 6.0), ("along", 1.0)]));
    let later = render(Some(&[("amount", 30.0), ("size", 6.0), ("along", 1.0), ("evolution", 2.0)]));
    let normal = render(Some(&[("amount", 8.0), ("size", 6.0), ("along", 0.0)]));
    assert!(differing(&still, &space) > 20, "Space の変位で絵が変わる: {}", differing(&still, &space));
    assert!(differing(&space, &later) > 20, "Evolution で流れる: {}", differing(&space, &later));
    assert!(differing(&still, &normal) > 20, "Normal の変位で陰影が変わる: {}", differing(&still, &normal));
}

/// 2D は 3D の部分集合: 同じ Turbulent Displace が板にも効く。板は場を「標本位置のずれ」として見せる
/// (面内の変位はそのまま、法線方向の変位は視差)。Space で絵が変わり、Evolution でまた変わり、
/// Amount 0 は掛けない絵と 1 階調も違わない。
#[test]
fn turbulent_displace_warps_a_flat_picture_too() {
    use crate::doc::store::{EffectId, EffectInstance};
    const TURBULENT_DISPLACE: &str = "motolii.turbulent_displace";
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 0, 0);
    let checker = dir.path().join("checker.png");
    let mut img = image::RgbaImage::new(SIZE, SIZE);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let v = if (x / 8 + y / 8) % 2 == 0 { 255 } else { 0 };
        *px = image::Rgba([v, v, v, 255]);
    }
    img.save(&checker).unwrap();
    let mut engine = Engine::new().unwrap();
    let mut render = |params: Option<&[(&str, f64)]>| -> Vec<u8> {
        let mut doc = scene(dir.path(), &sky, false);
        doc.apply(Intent::RemoveLayer(LayerId(2))).unwrap();
        let plate = file_layer(&mut doc, 3, 1, &checker);
        if let Some(params) = params {
            doc.apply(Intent::SetEffects { layer: plate, effects: vec![EffectInstance { id: EffectId(0), plugin_id: TURBULENT_DISPLACE.into() }] }).unwrap();
            for (name, value) in params {
                doc.apply(Intent::SetConstant { layer: plate, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
            }
        }
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    };
    let differing = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x[0].abs_diff(y[0]) > 8).count();
    let still = render(None);
    let zero = render(Some(&[("amount", 0.0)]));
    let space = render(Some(&[("amount", 6.0), ("size", 10.0), ("along", 1.0)]));
    let later = render(Some(&[("amount", 6.0), ("size", 10.0), ("along", 1.0), ("evolution", 2.0)]));
    assert_eq!(still, zero, "Amount 0 は掛けない絵と同じ");
    assert!(differing(&still, &space) > 20, "Space の変位で板の絵がずれる: {}", differing(&still, &space));
    assert!(differing(&space, &later) > 20, "Evolution で流れる: {}", differing(&space, &later));
}

/// hdr の 1.0 超は潰れない。環境の意味はここにある。
#[test]
fn hdr_keeps_values_above_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sun.hdr");
    let file = std::fs::File::create(&path).unwrap();
    image::codecs::hdr::HdrEncoder::new(file)
        .encode(&[image::Rgb([4.0f32, 0.5, 1.0]); 4], 2, 2)
        .unwrap();
    let (rgb, w, h) = crate::render::engine::decode_still_linear_rgb(path.to_str().unwrap()).unwrap();
    assert_eq!((w, h), (2, 2));
    assert!((rgb[0] - 4.0).abs() < 0.05 && (rgb[1] - 0.5).abs() < 0.01, "{:?}", &rgb[..3]);
    assert!(crate::render::media::is_still_image_path(&path), "hdr は画の門を通る");
}
