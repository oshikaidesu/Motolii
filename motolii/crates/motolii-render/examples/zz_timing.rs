//! 層ごとの出自と時間(start / duration / source_in / speed)を並べる: 動画が進まない時の切り分け。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = motolii_edit::Document::load(&std::env::args().nth(1).ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    println!("comp {}x{} fps {:?} frames {}", comp.width, comp.height, comp.fps, comp.duration_frames);
    for id in view.layers() {
        let Some(m) = view.meta(id)? else { continue };
        println!("{id:?} {:?} start {} dur {} src_in {} speed {:?}", m.source, m.timing.start, m.timing.duration, m.timing.source_in, m.timing.speed);
    }
    Ok(())
}
