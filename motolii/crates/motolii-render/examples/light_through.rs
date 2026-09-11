//! Render a document with every Glass layer set to block light (shadow + colored light on the rest),
//! next to the untouched document. Nothing is saved back.
use motolii_render::{
    doc::store::{Document, Intent, LayerAttrsPatch, RationalTime},
    engine::Engine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: light_through document.rrd out_dir".into());
    }
    let mut doc = Document::load(&args[1])?;
    let comp = doc.view().composition()?.ok_or("no composition")?;
    let out = std::path::PathBuf::from(&args[2]);
    std::fs::create_dir_all(&out)?;
    let glass: Vec<_> = doc
        .view()
        .resolved_layers(RationalTime::ZERO)?
        .iter()
        .filter(|l| l.effects.iter().any(|e| e.plugin_id == "motolii.glass"))
        .map(|l| l.id)
        .collect();
    let mut engine = Engine::new()?;
    for (name, blocks) in [("plain", false), ("blocks-light", true)] {
        for &layer in &glass {
            doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { blocks_light: Some(blocks), ..Default::default() } })?;
        }
        engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        let started = std::time::Instant::now();
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        println!("{name}: {}us light_captures={}", started.elapsed().as_micros(), engine.surface_work().light_captures);
        if !engine.layer_failures().is_empty() {
            return Err(engine.layer_failures().join("; ").into());
        }
        image::RgbaImage::from_raw(comp.width, comp.height, pixels).ok_or("Wrong frame dimensions")?.save(out.join(format!("{name}.png")))?;
    }
    Ok(())
}
