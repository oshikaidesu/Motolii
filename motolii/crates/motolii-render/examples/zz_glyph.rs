//! 文字の当たり(輪郭の外接)と親の箱を、コマを追って並べる: 壁を抜けるコマを見つける。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = motolii_render::doc::store::Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let last: i64 = args.next().unwrap_or("150".into()).parse()?;
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let mut engine = motolii_render::engine::Engine::new()?;
    let groups: Vec<_> = view.layers().into_iter().filter(|id| view.meta(*id).ok().flatten().is_some_and(|m| matches!(m.source, motolii_render::doc::store::LayerSource::Group))).collect();
    for frame in 0..=last {
        let t = motolii_render::doc::store::RationalTime::try_from_frame(frame, comp.fps)?;
        engine.render_frame(&view, t)?;
        if frame % 10 != 0 { continue; }
        let hulls = engine.physics_outline_bounds();
        let boxes: Vec<String> = groups.iter().filter_map(|g| view.layer_box(*g, t).ok().flatten()).map(|b| format!("{:.0}..{:.0}", b[0], b[2])).collect();
        let sel: Vec<String> = hulls.iter().skip(21).take(4).map(|b| format!("{:.0}..{:.0}", b[0], b[2])).collect();
        println!("f{frame:3} boxes {} | glyphs {}", boxes.join(" "), sel.join(" "));
    }
    Ok(())
}
