use motolii_render::doc::store::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = Document::load(&args.next().ok_or("doc")?)?;
    let frame: i64 = args.next().ok_or("frame")?.parse()?;
    let filter = args.next().unwrap_or_default();
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let t = RationalTime::try_from_frame(frame, comp.fps)?;
    for l in view.resolved_layers(t)? {
        let name = view.attrs(l.id)?.unwrap_or_default().name;
        if !name.contains(&filter) { continue; }
        println!("{:>5} proj={:?} order={:>6} op={:.2} src={:?} masks={} matte={:?} plate={:?} t={:?}", l.id.0, l.projection, l.placement.order, l.placement.opacity, l.source, l.masks.len(), l.matte.map(|m| m.layer.0), l.plate.map(|p| p.0), l.placement.transform.translation);
        println!("       world={:?} box={:?}", l.placement.world_transform.map(|w| w.translation), view.layer_box(l.id, t)?);
        println!("       {name}");
    }
    Ok(())
}
