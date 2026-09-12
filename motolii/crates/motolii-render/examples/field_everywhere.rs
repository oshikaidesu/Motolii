//! 1 つの場(Field)が、板・網・点群を同時に動かす証拠の絵を描く。
//!
//! `cargo run -p motolii-render --example field_everywhere -- <flag.png> <out_dir>`
//!
//! 旗は Wikimedia Commons の公共領域の国旗 PNG を想定(縁がどう動いたか輪郭で判る素材)。
//! 取説: docs/vism-field-model.md
use motolii_render::{doc::store::*, engine::Engine};

const W: u32 = 1600;
const H: u32 = 900;

fn sphere_obj() -> String {
    let (nu, nv) = (48, 24);
    let mut s = String::new();
    for j in 0..=nv {
        let phi = std::f64::consts::PI * j as f64 / nv as f64;
        for i in 0..=nu {
            let th = 2.0 * std::f64::consts::PI * i as f64 / nu as f64;
            let (x, y, z) = (phi.sin() * th.cos(), phi.cos(), phi.sin() * th.sin());
            s += &format!("v {x} {y} {z}\nvn {x} {y} {z}\n");
        }
    }
    let idx = |i: usize, j: usize| j * (nu + 1) + i + 1;
    for j in 0..nv {
        for i in 0..nu {
            let (a, b, c, d) = (idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1));
            s += &format!("f {a}//{a} {b}//{b} {c}//{c}\nf {a}//{a} {c}//{c} {d}//{d}\n");
        }
    }
    s
}

fn grid_ply() -> String {
    let n = 80;
    let mut s = format!("ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n", (n + 1) * (n + 1));
    for y in 0..=n {
        for x in 0..=n {
            let (u, v) = (x as f64 / n as f64, y as f64 / n as f64);
            s += &format!("{} {} 0 {} {} {}\n", x * 4, y * 4, (40.0 + 215.0 * u) as u8, (120.0 + 100.0 * v) as u8, 255u8);
        }
    }
    s
}

fn layer(doc: &mut Document, id: u64, path: &str, order: i16, pos: [f64; 2], scale: Option<f64>, amount: f64, world: bool) -> Result<(), Box<dyn std::error::Error>> {
    let layer = LayerId(id);
    let fx = EffectId(0);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta {
            source: LayerSource::File { path: path.into(), fingerprint: None },
            order, timing: LayerTiming::place(0, None, 1),
        }},
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(pos) },
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: fx, plugin_id: "motolii.turbulent_displace".into() }] },
        // 同じ場 — 同じ大きさ、同じ量、同じ種。違うのは素材だけ。
        Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "along").unwrap(), value: Value::F64(1.0) },
        Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "amount").unwrap(), value: Value::F64(amount) },
        Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "size").unwrap(), value: Value::F64(280.0) },
        // 場は層の座標で評価される。層の居場所を offset に入れると、式の座標が世界の座標になり、
        // 隣り合う層の波が一続きになる(裁定 2026-09-02「変位は世界を動かす」の手触りを見るため)。
        Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "offset_x").unwrap(), value: Value::F64(if world { pos[0] } else { 0.0 }) },
        Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "offset_y").unwrap(), value: Value::F64(if world { pos[1] } else { 0.0 }) },
    ])?;
    if let Some(scale) = scale {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale, scale]) })?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let mut args = std::env::args().skip(1);
    let flag = args.next().ok_or("flag png required")?;
    let dir = args.next().unwrap_or_else(|| ".".into());
    let obj = format!("{dir}/sphere.obj");
    let ply = format!("{dir}/grid.ply");
    std::fs::write(&obj, sphere_obj())?;
    std::fs::write(&ply, grid_ply())?;

    let mut engine = Engine::new()?;
    for (name, amount, world) in [("world-still", 0.0, false), ("world-moved", 70.0, false), ("world-linked", 70.0, true)] {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.07, 0.07, 0.09, 1.0] }))?;
        layer(&mut doc, 1, &flag, 0, [90.0, 290.0], Some(0.45), amount, world)?;
        layer(&mut doc, 2, &ply, 1, [540.0, 270.0], Some(1.25), amount, world)?;
        layer(&mut doc, 3, &obj, 2, [1130.0, 290.0], Some(160.0), amount, world)?;
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
        let out = format!("{dir}/{name}.png");
        image::save_buffer(&out, &pixels, W, H, image::ColorType::Rgba8)?;
        eprintln!("wrote {out} (drawn {})", engine.drawn_layers());
    }

    // 旗 1 枚だけ: 面内(XYZ)と法線(Normal)。AE ならマスク + Repeat Edge Pixels が要る所。
    for (name, amount, along) in [("flag-still", 0.0, 1.0), ("flag-xyz", 60.0, 1.0), ("flag-normal", 60.0, 0.0)] {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.07, 0.07, 0.09, 1.0] }))?;
        let layer = LayerId(1);
        let fx = EffectId(0);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: flag.clone(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([320.0, 160.0]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: fx, plugin_id: "motolii.turbulent_displace".into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "along").unwrap(), value: Value::F64(along) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "amount").unwrap(), value: Value::F64(amount) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "size").unwrap(), value: Value::F64(320.0) },
        ])?;
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
        let out = format!("{dir}/{name}.png");
        image::save_buffer(&out, &pixels, W, H, image::ColorType::Rgba8)?;
        eprintln!("wrote {out}");
    }

    // 絵の効果(Pass)も素材を選ばない。板は層の絵へ焼かれ、網と点群は描いた後の窓で効く。
    {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.07, 0.07, 0.09, 1.0] }))?;
        for (id, path, order, pos, scale) in [
            (1u64, flag.clone(), 0i16, [90.0, 290.0], Some(0.45)),
            (2, ply.clone(), 1, [540.0, 270.0], Some(1.25)),
            (3, obj.clone(), 2, [1130.0, 290.0], Some(160.0)),
        ] {
            let layer = LayerId(id);
            let fx = EffectId(0);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path, fingerprint: None }, order, timing: LayerTiming::place(0, None, 1) } },
                Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(pos) },
                Intent::SetEffects { layer, effects: vec![EffectInstance { id: fx, plugin_id: "motolii.blur".into() }] },
                Intent::SetConstant { layer, property: PropertyId::effect_param(fx, "radius").unwrap(), value: Value::F64(14.0) },
            ])?;
            if let Some(scale) = scale {
                doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale, scale]) })?;
            }
        }
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
        let out = format!("{dir}/pass-everywhere.png");
        image::save_buffer(&out, &pixels, W, H, image::ColorType::Rgba8)?;
        eprintln!("wrote {out}");
    }

    // 決定的な絵: 点群を旗の真上へ重ねる。場が世界で繋がっていれば、点は旗の波に乗る。
    for (name, world) in [("overlap-loose", false), ("overlap-linked", true)] {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.07, 0.07, 0.09, 1.0] }))?;
        let at = [580.0, 300.0];
        layer(&mut doc, 1, &flag, 0, at, Some(0.9), 70.0, world)?;
        layer(&mut doc, 2, &ply, 1, at, Some(2.7), 70.0, world)?;
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
        let out = format!("{dir}/{name}.png");
        image::save_buffer(&out, &pixels, W, H, image::ColorType::Rgba8)?;
        eprintln!("wrote {out}");
    }
    Ok(())
}
