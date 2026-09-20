//! 物ごとの位置をコマ順に出す(震えを見るため)。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = motolii_edit::Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let (from, to): (i64, i64) = (args.next().ok_or("from")?.parse()?, args.next().ok_or("to")?.parse()?);
    let view = doc.view();
    let fps = view.composition()?.ok_or("comp")?.fps;
    let mut engine = motolii_render::engine::Engine::new()?;
    for frame in from..=to {
        let t = motolii_render::doc::store::RationalTime::try_from_frame(frame, fps)?;
        engine.render_frame(&view, t)?;
        let states = engine.block_states();
        let line: Vec<String> = states.iter().map(|o| format!("{:.2},{:.2},{:.2}", o[0], o[1], o[2])).collect();
        println!("{frame} contacts={} {}", engine.physics_contacts(), line.join(" "));
    }
    Ok(())
}
