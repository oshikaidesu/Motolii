//! 常駐の見張り・描く側: 書類(`.rrd`)の保存と棚(vism/)の保存を見張り、変わる度にコマを描いて敷き詰め PNG を吐く。
//! 描く側(GPU・書体・棚)は開いたまま。台本の側の見張り(ui の `watch_shot`、debug)と組で使う。
//! 組み方は `cargo build --profile watch -p motolii-render --example zz_watch`(release の最適化 + disk の棚。
//! `--release` だと棚は焼き込みで、shader の保存は次の build まで載らない)。
//! `motolii/target/watch/examples/zz_watch <doc.rrd> <out_dir>`、`MOTOLII_LAST` / `MOTOLII_STEP` / `MOTOLII_SHRINK`。
//! 手順と計測は docs/reviews/2026-09-17-build-placement.md。
use motolii_edit::Document;
use motolii_render::{doc::store::*, engine::Engine};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("doc")?;
    let out = args.next().ok_or("out")?;
    std::fs::create_dir_all(&out)?;
    let read = |name: &str, default: u32| std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default);
    let mut engine = Engine::new()?;
    // 棚の見張り(port.rs と同じ): 起きたら旗、描く前に読み直す。焼き込み build では見張りは空。
    let shelf = Arc::new(AtomicBool::new(false));
    let flag = shelf.clone();
    let _watch = motolii_render::engine::watch_effect_catalog(move || flag.store(true, Ordering::Release))?;
    eprintln!("shelf: {}", if motolii_render::engine::catalog_reads_disk() { "disk (vism/ の保存が次のコマに載る)" } else { "baked (shader は build 時のまま)" });
    let mut seen = None;
    loop {
        let stamp = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let shelf_changed = shelf.swap(false, Ordering::AcqRel);
        if (stamp == seen && !shelf_changed) || stamp.is_none() {
            std::thread::sleep(std::time::Duration::from_millis(300));
            continue;
        }
        seen = stamp;
        if shelf_changed {
            let refresh = motolii_render::engine::refresh_effect_catalog();
            for error in &refresh.errors { eprintln!("shelf: {error}"); }
            eprintln!("shelf: generation {}", refresh.generation);
        }
        let started = std::time::Instant::now();
        let (last, step, shrink) = (read("MOTOLII_LAST", 120) as i64, read("MOTOLII_STEP", 1).max(1) as i64, read("MOTOLII_SHRINK", 1).max(1));
        let Ok(doc) = Document::load(&path) else { eprintln!("load failed"); continue };
        let Some(comp) = doc.view().composition().ok().flatten() else { continue };
        let mut picked: Vec<image::RgbaImage> = Vec::new();
        let mut frame = 0i64;
        while frame <= last {
            let Ok(t) = RationalTime::try_from_frame(frame, comp.fps) else { break };
            match engine.render_frame(&doc.view(), t) {
                Ok(pixels) => {
                    let image = image::RgbaImage::from_raw(comp.width, comp.height, pixels).ok_or("pixels")?;
                    let saved = if shrink > 1 { image::imageops::resize(&image, comp.width / shrink, comp.height / shrink, image::imageops::FilterType::Triangle) } else { image };
                    let _ = saved.save(format!("{out}/{frame:04}.png"));
                    picked.push(saved);
                }
                Err(e) => { eprintln!("frame {frame}: {e:?}"); break }
            }
            for skipped in frame + 1..(frame + step).min(last + 1) {
                if let Ok(t) = RationalTime::try_from_frame(skipped, comp.fps) { let _ = engine.render_frame(&doc.view(), t); }
            }
            frame += step;
        }
        if !picked.is_empty() {
            let n = picked.len();
            let idx: Vec<usize> = { let mut v = vec![0, n / 3, 2 * n / 3, n - 1]; v.dedup(); v };
            let (w, h) = (picked[0].width(), picked[0].height());
            let scale = 300.0 / w.max(h) as f32;
            let (tw, th) = (((w as f32 * scale) as u32).max(1), ((h as f32 * scale) as u32).max(1));
            let mut sheet = image::RgbaImage::from_pixel(tw * idx.len() as u32 + 10 * (idx.len() as u32 - 1), th, image::Rgba([16, 18, 22, 255]));
            for (k, i) in idx.iter().enumerate() {
                let thumb = image::imageops::resize(&picked[*i], tw, th, image::imageops::FilterType::Triangle);
                image::imageops::overlay(&mut sheet, &thumb, (k as u32 * (tw + 10)) as i64, 0);
            }
            let _ = sheet.save(format!("{out}/sheet.png"));
        }
        eprintln!("shot: {} frames in {:.1}s -> {out}/sheet.png", picked.len(), started.elapsed().as_secs_f32());
    }
}
