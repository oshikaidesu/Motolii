//! 波及の物差し: 書類 1 つを数コマ描き、ブロックが物の component をいくつ動かしたかを数える。
//! `zz_reach <doc.rrd> [frames...]` → コマごとに 物の数 / 位置が動いた / 回った / 大きさが変わった / 色か不透明が変わった。
//! 「1 つの手が何に作用したか」を数で見る(利用者 2026-09-18「css はひとつのオブジェクトが数多くに作用した。物理もそう。測れるものにできるのでは」)。
use motolii_render::{doc::store::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("doc")?;
    let frames: Vec<i64> = { let v: Vec<i64> = args.filter_map(|a| a.parse().ok()).collect(); if v.is_empty() { vec![0, 15, 30, 45] } else { v } };
    let doc = Document::load(&path)?.with_programs(motolii_render::extensions::bundled());
    let comp = doc.view().composition()?.ok_or("comp")?;
    let mut engine = Engine::new()?;
    println!("frame\tthings\tmoved\tturned\tsized\ttinted\tvalues");
    for frame in frames {
        let t = RationalTime::try_from_frame(frame, comp.fps)?;
        engine.render_frame(&doc.view(), t)?;
        let o = engine.block_offsets();
        let eps = 1e-4f32;
        let moved = o.iter().filter(|s| s[0].abs() > eps || s[1].abs() > eps).count();
        let turned = o.iter().filter(|s| s[2].abs() > eps).count();
        let sized = o.iter().filter(|s| (s[3] - 1.0).abs() > eps).count();
        let tinted = o.iter().filter(|s| (s[4] - 1.0).abs() > eps || (s[5] - 1.0).abs() > eps || (s[6] - 1.0).abs() > eps || (s[7] - 1.0).abs() > eps).count();
        println!("{frame}\t{}\t{moved}\t{turned}\t{sized}\t{tinted}\t{}", o.len(), moved + turned + sized + tinted);
    }
    Ok(())
}
