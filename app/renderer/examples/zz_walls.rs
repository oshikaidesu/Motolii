use motolii_edit::Document;
use motolii_render::doc::store::*;
use motolii_render::picture::{boxes::layer_box, frame::layout_frame};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Document::load(&std::env::args().nth(1).ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let view = doc.view();
    let t = RationalTime::ZERO;
    let frame = layout_frame(&view, t)?;
    for id in view.layers().into_iter().take(6) {
        println!("{id:?} size={:?} slot={:?} box={:?}", frame.sizes.get(&id), frame.slots.get(&id).map(|s| format!("{s:?}")), layer_box(&view, id, t)?);
    }
    Ok(())
}
