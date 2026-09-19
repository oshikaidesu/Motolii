use motolii_render::doc::store::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Document::load(&std::env::args().nth(1).ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let view = doc.view();
    let t = RationalTime::ZERO;
    let frame = view.layout_frame(t)?;
    for id in view.layers().into_iter().take(6) {
        println!("{id:?} size={:?} slot={:?} box={:?}", frame.sizes.get(&id), frame.slots.get(&id).map(|s| format!("{s:?}")), view.layer_box(id, t)?);
    }
    Ok(())
}
