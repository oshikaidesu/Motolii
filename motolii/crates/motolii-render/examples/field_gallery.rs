//! 見せる絵。1 つの場が点群の絹を流し、同じ場で歪んだガラスがその向こうを透かす。
//! `cargo run -p motolii-render --example field_gallery -- <out_dir>`
use motolii_render::{doc::store::*, engine::Engine};

const W: u32 = 1920;
const H: u32 = 1080;

/// 細かい点の板。色は位置で深青 → 紫 → 桃へ。
fn silk_ply(n: usize, step: f64) -> String {
    let mut s = format!("ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n", (n + 1) * (n + 1));
    for y in 0..=n {
        for x in 0..=n {
            let (u, v) = (x as f64 / n as f64, y as f64 / n as f64);
            let t = (u * 0.65 + v * 0.35).clamp(0.0, 1.0);
            let (r, g, b) = (
                30.0 + 225.0 * t.powf(1.6),
                40.0 + 90.0 * (1.0 - (t - 0.5).abs() * 2.0).max(0.0),
                120.0 + 135.0 * (1.0 - t * 0.5),
            );
            s += &format!("{} {} 0 {} {} {}\n", x as f64 * step, y as f64 * step, r as u8, g as u8, b as u8);
        }
    }
    s
}

fn sphere_obj(nu: usize, nv: usize) -> String {
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

struct Fx {
    plugin: &'static str,
    params: Vec<(&'static str, f64)>,
}

fn place(doc: &mut Document, id: u64, path: &str, order: i16, pos: [f64; 2], scale: f64, effects: Vec<Fx>) -> Result<(), Box<dyn std::error::Error>> {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta {
            source: LayerSource::File { path: path.into(), fingerprint: None },
            order, timing: LayerTiming::place(0, None, 1),
        }},
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(pos) },
        Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale, scale]) },
        Intent::SetEffects { layer, effects: effects.iter().enumerate()
            .map(|(i, fx)| EffectInstance { id: EffectId(i as u32), plugin_id: fx.plugin.into() }).collect() },
    ])?;
    for (i, fx) in effects.iter().enumerate() {
        for (name, value) in &fx.params {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(i as u32), name).unwrap(), value: Value::F64(*value) })?;
        }
    }
    Ok(())
}

/// 場は 1 つ。offset に層の居場所を入れて、世界の座標で評価させる。
fn flow(amount: f64, size: f64, at: [f64; 2]) -> Fx {
    Fx { plugin: "motolii.turbulent_displace", params: vec![
        ("along", 1.0), ("amount", amount), ("size", size), ("complexity", 4.0),
        ("offset_x", at[0]), ("offset_y", at[1]),
    ]}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let ply = format!("{dir}/silk.ply");
    let obj = format!("{dir}/ball.obj");
    std::fs::write(&ply, silk_ply(300, 6.0))?;
    std::fs::write(&obj, sphere_obj(96, 48))?;

    let mut engine = Engine::new()?;
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1,
        background: [0.03, 0.03, 0.05, 1.0],
    }))?;

    let silk_at = [120.0, 60.0];
    place(&mut doc, 1, &ply, 0, silk_at, 1.0, vec![flow(150.0, 620.0, silk_at)])?;
    let ball_at = [1060.0, 400.0];
    place(&mut doc, 2, &obj, 1, ball_at, 240.0, vec![
        flow(60.0, 620.0, ball_at),
        Fx { plugin: "motolii.glass", params: vec![
            ("ior", 1.45), ("roughness", 0.06), ("transmission", 1.0), ("dispersion", 0.9),
        ]},
    ])?;

    let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
    for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
    let out = format!("{dir}/gallery.png");
    image::save_buffer(&out, &pixels, W, H, image::ColorType::Rgba8)?;
    eprintln!("wrote {out} (drawn {})", engine.drawn_layers());
    Ok(())
}
