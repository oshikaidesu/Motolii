//! 動画が進まない時の切り分け: コマを描く度に frame cache の当たり・数を出す。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = motolii_render::doc::store::Document::load(&std::env::args().nth(1).ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let mut engine = motolii_render::engine::Engine::new()?;
    for frame in [0i64, 1, 2, 10, 50, 100] {
        engine.render_frame(&view, motolii_render::doc::store::RationalTime::try_from_frame(frame, comp.fps)?)?;
        let (hits, entries, bytes) = engine.video_frame_cache_stats();
        println!("frame {frame}: hits {hits} entries {entries} bytes {bytes} failures {:?}", engine.layer_failures());
    }
    Ok(())
}
