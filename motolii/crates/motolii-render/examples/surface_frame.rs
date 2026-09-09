//! Render a saved surface comparison through the same Engine as preview/export.
use motolii_render::{
    doc::store::{Document, RationalTime},
    engine::Engine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let args: Vec<_> = std::env::args().collect();
    if !(3..=4).contains(&args.len()) {
        return Err("usage: surface_frame document.rrd output.png [measured_frames]".into());
    }
    let doc = Document::load(&args[1])?;
    let view = doc.view();
    let comp = view.composition()?.ok_or("Missing composition")?;
    let mut engine = Engine::new()?;
    let frames = args
        .get(3)
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(1)
        .max(1);
    engine.render_frame(&view, RationalTime::ZERO)?;
    let before = engine.surface_work();
    let mut pixels = Vec::new();
    let mut elapsed = Vec::new();
    for _ in 0..frames {
        let started = std::time::Instant::now();
        pixels = engine.render_frame(&view, RationalTime::ZERO)?;
        elapsed.push(started.elapsed().as_micros());
    }
    let after = engine.surface_work();
    println!("total_with_readback_us={elapsed:?} captures={} main_runs={} backdrop_copies={} mesh_data_batches={}",
        (after.scene_captures-before.scene_captures)/frames as u64,
        (after.main_runs-before.main_runs)/frames as u64,
        (after.backdrop_copies-before.backdrop_copies)/frames as u64,
        (after.mesh_batches-before.mesh_batches)/frames as u64);
    if !engine.layer_failures().is_empty() {
        return Err(engine.layer_failures().join("; ").into());
    }
    image::RgbaImage::from_raw(comp.width, comp.height, pixels)
        .ok_or("Wrong frame dimensions")?
        .save(&args[2])?;
    Ok(())
}
