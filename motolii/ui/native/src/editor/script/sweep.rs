//! ささくれの総当たり: 効果 × 素材 × 投影をスクリプトで組み、2 コマ描いて機械の目で 3 つだけ見る。
//! `MOTOLII_SWEEP_OUT=<dir> cargo test -p motolii-ui --lib -- --ignored sweep_every_effect`
//! 赤は `<dir>/report.tsv` と縮小画(効果なしの絵と並べた 1 枚)。赤は候補で、判定は人の目。
use crate::doc::store::RationalTime;

const BACKGROUND: [u8; 3] = [16, 16, 16];
const FRAMES: [i64; 2] = [0, 15];

struct Look {
    drawn: usize,
    bbox: Option<[usize; 4]>,
    /// 絵の外枠の 4 辺のうち、一番「絵が辺いっぱいに詰まっている」辺の割合。柔らかい光は低く、切れた板は高い。
    hard_edge: f64,
    failures: Vec<String>,
    pixels: Vec<u8>,
}

fn differs(p: &[u8]) -> bool {
    p[..3].iter().zip(BACKGROUND).map(|(a, b)| (*a as i32 - b as i32).abs()).sum::<i32>() > 24
}

fn look(pixels: Vec<u8>, width: usize, height: usize, failures: Vec<String>) -> Look {
    let at = |x: usize, y: usize| &pixels[(y * width + x) * 4..(y * width + x) * 4 + 4];
    let (mut x0, mut y0, mut x1, mut y1, mut drawn) = (usize::MAX, usize::MAX, 0, 0, 0);
    for y in 0..height {
        for x in 0..width {
            if differs(at(x, y)) {
                drawn += 1;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if drawn == 0 {
        return Look { drawn, bbox: None, hard_edge: 0.0, failures, pixels };
    }
    // 画面の縁に接している辺は、切れていても comp の枠なので数えない。
    let mut hard = 0.0f64;
    let row = |y: usize| (x0..=x1).filter(|x| differs(at(*x, y))).count() as f64 / (x1 - x0 + 1) as f64;
    let column = |x: usize| (y0..=y1).filter(|y| differs(at(x, *y))).count() as f64 / (y1 - y0 + 1) as f64;
    if x1 - x0 > 60 {
        if y0 > 0 { hard = hard.max(row(y0)); }
        if y1 + 1 < height { hard = hard.max(row(y1)); }
    }
    if y1 - y0 > 60 {
        if x0 > 0 { hard = hard.max(column(x0)); }
        if x1 + 1 < width { hard = hard.max(column(x1)); }
    }
    Look { drawn, bbox: Some([x0, y0, x1, y1]), hard_edge: hard, failures, pixels }
}

fn material_png(dir: &std::path::Path) -> String {
    let path = dir.join("sweep-material.png");
    let image = image::RgbaImage::from_fn(360, 240, |x, y| image::Rgba([(x * 255 / 360) as u8, (y * 255 / 240) as u8, 180, 255]));
    image.save(&path).unwrap();
    path.to_str().unwrap().to_owned()
}

fn side_by_side(before: &Look, after: &Look, width: usize, height: usize, path: &std::path::Path) {
    let frame = |l: &Look| image::RgbaImage::from_raw(width as u32, height as u32, l.pixels.clone()).unwrap();
    let small = |l: &Look| image::imageops::thumbnail(&frame(l), 480, 270);
    let mut sheet = image::RgbaImage::new(960, 270);
    image::imageops::overlay(&mut sheet, &small(before), 0, 0);
    image::imageops::overlay(&mut sheet, &small(after), 480, 0);
    sheet.save(path).unwrap();
}

#[test]
#[ignore]
fn sweep_every_effect() {
    let out = std::path::PathBuf::from(std::env::var("MOTOLII_SWEEP_OUT").unwrap_or_else(|_| std::env::temp_dir().join("motolii-sweep").to_string_lossy().into_owned()));
    std::fs::create_dir_all(&out).unwrap();
    let picture = material_png(&out);
    let moving = r#".key("Position", 0, [900, 540], "Bezier").key("Position", 1, [1020, 540])"#;
    let materials: Vec<(&str, String)> = vec![
        ("ellipse", format!(r##"ellipse({{ Scale: [0.2, 0.2] }}).fill("#ffcc33"){moving}"##)),
        ("star", format!(r##"star({{ Scale: [0.6, 0.6] }}).fill("#33ccff"){moving}"##)),
        ("text", format!(r##"text("Sweep").fill("#ffffff"){moving}"##)),
        ("image", format!(r#"media({picture:?}){moving}"#)),
        ("particles", r#"particles({ Rate: 200, Speed: 300, Size: 10, Spread: 360 })"#.to_owned()),
    ];
    let effects: Vec<String> = crate::render::engine::known_effects().iter().map(|d| d.label.clone()).collect();
    let mut rt = crate::EditorRuntime::open("").unwrap();
    let mut report = vec!["verdict\tmaterial\tprojection\teffect\tframe\tdrawn\tbaseline_drawn\thard_edge\tbaseline_hard_edge\tdetail".to_owned()];
    let mut render = |rt: &mut crate::EditorRuntime, source: &str| -> Result<Vec<Look>, String> {
        rt.request(serde_json::json!({ "op": "new" }))?;
        rt.run_script(source, "sweep.js")?;
        let comp = rt.doc.view().composition().map_err(|e| e.to_string())?.ok_or("no composition")?;
        FRAMES.iter().map(|frame| {
            let t = RationalTime::try_from_frame(*frame, comp.fps).map_err(|e| e.to_string())?;
            let pixels = rt.engine.render_frame(&rt.doc.view(), t).map_err(|e| e.to_string())?;
            Ok(look(pixels, comp.width as usize, comp.height as usize, rt.engine.layer_failures().to_vec()))
        }).collect()
    };
    let (mut red, mut total) = (0, 0);
    for (material, make) in &materials {
        for projection in ["2D", "2.5D", "3D"] {
            let prefix = format!(r##"comp({{ seconds: 2, background: "#101010" }}); const L = {make}; L.projection("{projection}");"##);
            let baseline = match render(&mut rt, &prefix) {
                Ok(looks) => looks,
                Err(message) => {
                    report.push(format!("BASELINE\t{material}\t{projection}\t-\t-\t-\t-\t-\t-\t{}", message.replace(['\n', '\t'], " ")));
                    continue;
                }
            };
            for effect in &effects {
                total += 1;
                let source = format!("{prefix} L.effect({effect:?});");
                let looks = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render(&mut rt, &source))) {
                    Ok(Ok(looks)) => looks,
                    Ok(Err(message)) if message.contains("apply to") || message.contains("requires") => continue,
                    Ok(Err(message)) => {
                        red += 1;
                        report.push(format!("ERROR\t{material}\t{projection}\t{effect}\t-\t-\t-\t-\t-\t{}", message.lines().next().unwrap_or_default()));
                        continue;
                    }
                    Err(_) => {
                        red += 1;
                        report.push(format!("PANIC\t{material}\t{projection}\t{effect}\t-\t-\t-\t-\t-\t"));
                        rt = crate::EditorRuntime::open("").unwrap();
                        continue;
                    }
                };
                for ((frame, after), before) in FRAMES.iter().zip(&looks).zip(&baseline) {
                    let verdict = if !after.failures.is_empty() {
                        Some("FAILED")
                    } else if before.drawn > 500 && after.drawn < 50 {
                        Some("EMPTY")
                    } else if after.hard_edge > 0.5 && before.hard_edge < 0.3 {
                        Some("BOX")
                    } else if after.hard_edge > 0.5 && after.bbox.zip(before.bbox).is_some_and(|(a, b)| a[0] + 8 < b[0] || a[2] > b[2] + 8) {
                        Some("BOX")
                    } else {
                        None
                    };
                    if let Some(verdict) = verdict {
                        red += 1;
                        let name = format!("{verdict}-{material}-{projection}-{effect}-{frame}.png").replace([' ', '/', '&', '(', ')', ','], "_");
                        let comp = rt.doc.view().composition().unwrap().unwrap();
                        side_by_side(before, after, comp.width as usize, comp.height as usize, &out.join(&name));
                        report.push(format!("{verdict}\t{material}\t{projection}\t{effect}\t{frame}\t{}\t{}\t{:.2}\t{:.2}\t{}", after.drawn, before.drawn, after.hard_edge, before.hard_edge, after.failures.join(" | ").replace(['\n', '\t'], " ")));
                    }
                }
            }
        }
    }
    std::fs::write(out.join("report.tsv"), report.join("\n")).unwrap();
    eprintln!("sweep: {red} red of {total} combinations → {}", out.display());
}
