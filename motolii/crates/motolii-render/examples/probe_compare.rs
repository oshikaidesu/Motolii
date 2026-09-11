//! Same document, receiver moved through a few poses: receiver-following probes (current) beside
//! the scene-fixed probe (Arm's local cubemap). Writes PNGs and prints captures per frame.
use motolii_render::{
    doc::store::{Document, Intent, PropertyId, RationalTime, Value},
    engine::Engine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: probe_compare document.rrd out_dir".into());
    }
    let mut doc = Document::load(&args[1])?;
    let comp = doc.view().composition()?.ok_or("no composition")?;
    let out = std::path::PathBuf::from(&args[2]);
    std::fs::create_dir_all(&out)?;
    let receiver = doc
        .view()
        .resolved_layers(RationalTime::ZERO)?
        .iter()
        .find(|l| l.effects.iter().any(|e| e.plugin_id == "motolii.glass"))
        .ok_or("no glass receiver")?
        .id;
    for scene_probe in [false, true] {
        let mut engine = Engine::new()?;
        engine.set_reflection_scene_probe(scene_probe);
        let name = if scene_probe { "scene" } else { "receiver" };
        for (i, rotation) in [20.0, 40.0, 60.0].into_iter().enumerate() {
            doc.apply(Intent::SetConstant {
                layer: receiver,
                property: PropertyId::new("rotation.y")?,
                value: Value::F64(rotation),
            })?;
            engine.render_frame(&doc.view(), RationalTime::ZERO)?;
            let before = engine.surface_work();
            let started = std::time::Instant::now();
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
            let repeat_us = started.elapsed().as_micros();
            let after = engine.surface_work();
            // Move, then render once: how many faces does a moved receiver cost?
            doc.apply(Intent::SetConstant { layer: receiver, property: PropertyId::new("rotation.y")?, value: Value::F64(rotation + 1.0) })?;
            let started = std::time::Instant::now();
            engine.render_frame(&doc.view(), RationalTime::ZERO)?;
            let moved_us = started.elapsed().as_micros();
            let moved = engine.surface_work();
            if !engine.layer_failures().is_empty() {
                return Err(engine.layer_failures().join("; ").into());
            }
            println!(
                "{name} pose{i} rotation={rotation}: same-frame captures={} {repeat_us}us; after move captures={} {moved_us}us",
                after.scene_captures - before.scene_captures,
                moved.scene_captures - after.scene_captures
            );
            image::RgbaImage::from_raw(comp.width, comp.height, pixels)
                .ok_or("Wrong frame dimensions")?
                .save(out.join(format!("{name}-pose{i}.png")))?;
        }
    }
    Ok(())
}
