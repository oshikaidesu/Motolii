//! コマの範囲を PNG に書き出す。往復を速くする口: `MOTOLII_STEP=n` で n コマおき、
//! `MOTOLII_SHRINK=k` で 1/k に縮めて保存(描く側は comp の大きさのまま、保存だけ縮める)。
use motolii_render::{doc::store::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let (from, to): (i64, i64) = (args.next().ok_or("from")?.parse()?, args.next().ok_or("to")?.parse()?);
    let dir = args.next().ok_or("dir")?;
    let step: i64 = std::env::var("MOTOLII_STEP").ok().and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
    let shrink: u32 = std::env::var("MOTOLII_SHRINK").ok().and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
    let comp = doc.view().composition()?.ok_or("comp")?;
    let mut engine = Engine::new()?;
    let mut frame = from;
    while frame <= to {
        // 物理は列なので、飛ばすコマも解くが描かない(描く側の費用だけを削る)。
        let pixels = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, comp.fps)?)?;
        for f in engine.layer_failures() {
            eprintln!("frame {frame}: {f}");
        }
        let image = image::RgbaImage::from_raw(comp.width, comp.height, pixels).ok_or("pixels")?;
        let saved = if shrink > 1 { image::imageops::resize(&image, comp.width / shrink, comp.height / shrink, image::imageops::FilterType::Triangle) } else { image };
        saved.save(format!("{dir}/{frame:04}.png"))?;
        for skipped in frame + 1..(frame + step).min(to + 1) {
            engine.render_frame(&doc.view(), RationalTime::try_from_frame(skipped, comp.fps)?)?;
        }
        frame += step;
    }
    Ok(())
}
