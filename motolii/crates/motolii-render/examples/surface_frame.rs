//! Render a saved surface comparison through the same Engine as preview/export.
use motolii_render::{
    doc::store::{Document, RationalTime},
    engine::Engine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: surface_frame document.rrd output.png".into());
    }
    let doc = Document::load(&args[1])?;
    let view = doc.view();
    let comp = view.composition()?.ok_or("Missing composition")?;
    let mut engine = Engine::new()?;
    let pixels = engine.render_frame(&view, RationalTime::ZERO)?;
    if !engine.layer_failures().is_empty() {
        return Err(engine.layer_failures().join("; ").into());
    }
    image::RgbaImage::from_raw(comp.width, comp.height, pixels)
        .ok_or("Wrong frame dimensions")?
        .save(&args[2])?;
    Ok(())
}
