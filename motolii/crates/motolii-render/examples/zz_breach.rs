//! 壁の抜けを輪郭基準で測る: 各物の当たりの外接が部屋(comp)をどれだけ越えるか。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = motolii_render::doc::store::Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let last: i64 = args.next().ok_or("frame")?.parse()?;
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let mut engine = motolii_render::engine::Engine::new()?;
    for frame in 0..=last {
        engine.render_frame(&view, motolii_render::doc::store::RationalTime::try_from_frame(frame, comp.fps)?)?;
    }
    let (w, h) = (comp.width as f32, comp.height as f32);
    for (k, b) in engine.physics_outline_bounds().iter().enumerate() {
        let breach = [(-b[0]).max(0.0), (-b[1]).max(0.0), (b[2] - w).max(0.0), (b[3] - h).max(0.0)];
        println!("物 {k}: 輪郭 x {:.1}..{:.1} y {:.1}..{:.1}  越え 左{:.1} 上{:.1} 右{:.1} 下{:.1}", b[0], b[2], b[1], b[3], breach[0], breach[1], breach[2], breach[3]);
    }
    Ok(())
}
