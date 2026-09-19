//! 連続性の物差し(関係の動き): 作品をコマごとに解き、層の箱の角と文字の字の位置の跳びを報告する。
//! `cargo run -p motolii-render --example continuity -- <doc.rrd> <from> <to> [track filter]`(filter を渡すとその軌跡を出す)
//! 跳び = 1 コマの移動が前後 3 コマの中央値の 4 倍を越え、かつ 6 px を越えるコマ(範囲の最後のコマは比べる相手が片側なので除く)。尖り = 速度の変化が 6 px を越え、前後の 4 倍を越えるコマ。
use std::collections::BTreeMap;

use motolii_render::{doc::store::*, engine::Engine};

fn median(mut v: Vec<f32>) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let doc = Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let (from, to): (i64, i64) = (args.next().ok_or("from")?.parse()?, args.next().ok_or("to")?.parse()?);
    let comp = doc.view().composition()?.ok_or("comp")?;
    let mut engine = Engine::new()?;
    let mut tracks: BTreeMap<String, Vec<Option<[f32; 2]>>> = BTreeMap::new();
    for (i, frame) in (from..=to).enumerate() {
        for (key, p) in engine.continuity_samples(&doc.view(), RationalTime::try_from_frame(frame, comp.fps)?)? {
            let track = tracks.entry(key).or_insert_with(|| vec![None; (to - from + 1) as usize]);
            track[i] = Some(p);
        }
    }
    if let Some(filter) = args.next() {
        for (key, track) in tracks.iter().filter(|(k, _)| k.contains(&filter)) {
            println!("{key}");
            for (i, p) in track.iter().enumerate() {
                if let Some(p) = p {
                    println!("  {:4} {:8.1} {:8.1}", from + i as i64, p[0], p[1]);
                }
            }
        }
        return Ok(());
    }
    let mut jumps_total = 0;
    let mut spikes_total = 0;
    let mut worst: Vec<(f32, String, i64, &str)> = Vec::new();
    let mut by_track: BTreeMap<String, usize> = BTreeMap::new();
    for (key, track) in &tracks {
        let speed: Vec<Option<f32>> = (0..track.len()).map(|i| match (i.checked_sub(1).and_then(|j| track[j]), track[i]) {
            (Some(a), Some(b)) => Some(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()),
            _ => None,
        }).collect();
        for i in 0..speed.len() {
            // 範囲の端(片側しか比べる相手が無い)は数えない。
            let Some(v) = speed[i] else { continue };
            if i + 1 >= speed.len() {
                continue;
            }
            let around: Vec<f32> = (i.saturating_sub(3)..(i + 4).min(speed.len())).filter(|j| *j != i).filter_map(|j| speed[j]).collect();
            let m = median(around);
            if v > 6.0 && v > 4.0 * m.max(0.5) {
                jumps_total += 1;
                *by_track.entry(key.split(' ').next().unwrap_or("").to_string()).or_default() += 1;
                worst.push((v, key.clone(), from + i as i64, "jump"));
            }
            if i >= 1 {
                if let (Some(a), Some(b), Some(c)) = (i.checked_sub(1).and_then(|j| track[j]), track[i], i.checked_sub(2).and_then(|j| track[j])) {
                    let accel = ((b[0] - 2.0 * a[0] + c[0]).powi(2) + (b[1] - 2.0 * a[1] + c[1]).powi(2)).sqrt();
                    let around: Vec<f32> = (i.saturating_sub(3)..(i + 4).min(track.len())).filter(|j| *j != i && *j >= 2).filter_map(|j| match (track[j - 1], track[j], track[j - 2]) {
                        (Some(a), Some(b), Some(c)) => Some(((b[0] - 2.0 * a[0] + c[0]).powi(2) + (b[1] - 2.0 * a[1] + c[1]).powi(2)).sqrt()),
                        _ => None,
                    }).collect();
                    if accel > 6.0 && accel > 4.0 * median(around).max(0.5) {
                        spikes_total += 1;
                        worst.push((accel, key.clone(), from + i as i64, "spike"));
                    }
                }
            }
        }
    }
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("tracks {} frames {}..{} jumps {} spikes {}", tracks.len(), from, to, jumps_total, spikes_total);
    let mut ranked: Vec<_> = by_track.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    println!("  jumps by layer: {}", ranked.iter().take(8).map(|(k, n)| format!("{k}={n}")).collect::<Vec<_>>().join(" "));
    for (v, key, frame, kind) in worst.iter().filter(|w| std::env::var("SKIP").map_or(true, |s| !w.1.starts_with(&s))).take(20) {
        println!("  {kind:5} {v:8.1} px  frame {frame:4}  {key}");
    }
    Ok(())
}
