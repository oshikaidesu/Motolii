//! 使い捨て: 貼った Shadertoy を実写へ乗せる。
use motolii_render::{doc::store::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let mut args = std::env::args().skip(1);
    let clip = args.next().ok_or("clip required")?;
    let dir = args.next().unwrap_or_else(|| ".".into());
    let (w, h) = (1280u32, 534u32);
    let refresh = motolii_render::compositor::refresh_effect_catalog();
    eprintln!("catalog errors = {:#?}", refresh.errors);
    let mut engine = Engine::new()?;
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("city-plain", vec![]),
        ("city-neon", vec!["import.neon_city"]),
        ("city-plasma", vec!["import.ceil_plasma"]),
        ("city-edge", vec!["import.ceil_edge"]),
        ("city-bloom", vec!["import.ceil_bloom"]),
        ("city-trail", vec!["import.ceil_trail"]),
        ("city-timediff", vec!["motolii.time_difference"]),
    ];
    // 利用者が回した欄: ずれ 0 なら差は無く真っ黒になるはず。
    let dialed: Vec<(&str, Vec<&str>, Vec<(&str, f64)>)> = vec![("city-timediff0", vec!["motolii.time_difference"], vec![("offset", 0.0)])];
    let cases: Vec<(&str, Vec<&str>, Vec<(&str, f64)>)> = cases.into_iter().map(|(n, e)| (n, e, vec![])).chain(dialed).collect();
    for (name, effects, params) in cases {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: w, height: h, fps: Fps::try_new(24, 1).unwrap(), duration_frames: 240, background: [0.0, 0.0, 0.0, 1.0] }))?;
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.clone(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 240) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
            Intent::SetEffects { layer, effects: effects.iter().enumerate().map(|(i, id)| EffectInstance { id: EffectId(i as u32), plugin_id: (*id).into() }).collect() },
        ])?;
        for (pname, value) in &params {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), pname).unwrap(), value: Value::F64(*value) })?;
        }
        let t = RationalTime::try_from_frame(72, Fps::try_new(24, 1).unwrap())?;
        let pixels = match engine.render_frame(&doc.view(), t) { Ok(p) => p, Err(e) => { eprintln!("  {name}: RENDER FAILED: {e}"); continue; } };
        for f in engine.layer_failures() { eprintln!("  {name}: layer failure: {f}"); }
        let out = format!("{dir}/{name}.png");
        image::save_buffer(&out, &pixels, w, h, image::ColorType::Rgba8)?;
        eprintln!("wrote {out}");
    }
    Ok(())
}
