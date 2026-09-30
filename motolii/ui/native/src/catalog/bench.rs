//! Measurements at 100 / 1,000 / 10,000 files (run on demand):
//!   cargo test -p motolii-ui --lib catalog::bench -- --ignored --nocapture
//! Each row is wall time of one operation over a fresh temporary tree (small files: the cost measured is the walk, the
//! diff and the database, not disk throughput; the cheap fingerprint reads at most 2 MB per file on top).

use std::fs;
use std::time::Instant;

use super::*;

fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps").args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().ok();
    out.and_then(|o| String::from_utf8(o.stdout).ok()).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0) / 1024.0
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn build(root: &std::path::Path, n: usize) {
    // 10 groups x 10 folders; files spread evenly; four kinds of extension
    for i in 0..n {
        let dir = root.join(format!("g{}/d{}", i % 10, (i / 10) % 10));
        fs::create_dir_all(&dir).unwrap();
        let ext = ["png", "mov", "wav", "glb"][i % 4];
        fs::write(dir.join(format!("file{i}.{ext}")), format!("bytes of file {i}").repeat(8)).unwrap();
    }
}

#[test]
#[ignore]
fn measure() {
    println!("\n{:>7} | {:>9} | {:>9} | {:>8} | {:>8} | {:>10} | {:>8} | {:>10} | {:>7} | {:>7} | {:>8} | {:>9} | {:>7}", "files", "index ms", "noop ms", "rename", "move", "folder mv", "delete", "relocate", "query", "search", "folder", "enrich ms", "rss MB");
    for n in [100usize, 1_000, 10_000] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Vol");
        build(&root, n);
        let mut c = Catalog::open(&dir.path().join("state/catalog.sqlite")).unwrap();
        let s = c.add_source(&root, None).unwrap();

        let t = Instant::now();
        let r = c.refresh(None).unwrap();
        let index = ms(t);
        assert_eq!(r[0].added, n);

        let t = Instant::now();
        c.refresh(None).unwrap();
        let noop = ms(t);

        fs::rename(root.join("g0/d0/file0.png"), root.join("g0/d0/renamed.png")).unwrap();
        let t = Instant::now();
        let r = c.refresh(None).unwrap();
        let rename = ms(t);
        assert_eq!(r[0].moved, 1);

        fs::create_dir_all(root.join("g1/moved-to")).unwrap();
        fs::rename(root.join("g1/d0/file10.png").exists().then(|| root.join("g1/d0/file10.png")).unwrap_or_else(|| root.join("g1/d0/file11.mov")), root.join("g1/moved-to/one.bin.png")).ok();
        let t = Instant::now();
        let r = c.refresh(None).unwrap();
        let mv = ms(t);
        assert!(r[0].moved <= 1);

        // a top-level group (a tenth of the tree) carried into another folder
        fs::create_dir_all(root.join("Archive")).unwrap();
        fs::rename(root.join("g2"), root.join("Archive/g2")).unwrap();
        let t = Instant::now();
        let r = c.refresh(None).unwrap();
        let folder = ms(t);
        assert_eq!(r[0].moved, n / 10);
        assert_eq!(r[0].missing, 0);

        let victim = root.join("g3/d0").read_dir().unwrap().next().unwrap().unwrap().path();
        fs::remove_file(victim).unwrap();
        let t = Instant::now();
        let r = c.refresh(None).unwrap();
        let delete = ms(t);
        assert_eq!(r[0].missing, 1);

        let moved_root = dir.path().join("Vol2");
        fs::rename(&root, &moved_root).unwrap();
        let t = Instant::now();
        c.refresh(None).unwrap();
        c.relocate_source(&s.id, &moved_root).unwrap();
        let r = c.refresh(None).unwrap();
        let relocate = ms(t);
        assert_eq!((r[0].added, r[0].moved, r[0].missing), (0, 0, 0));

        let t = Instant::now();
        let a = c.query(&Query { kinds: Some(vec![MediaKind::Video]), limit: 100, ..Default::default() }).unwrap();
        let query = ms(t);
        assert!(a.total > 0);
        let t = Instant::now();
        let b = c.query(&Query { text: Some("file7".into()), limit: 100, ..Default::default() }).unwrap();
        let search = ms(t);
        assert!(b.total > 0);
        let t = Instant::now();
        c.folders(&s.id, "").unwrap();
        c.query(&Query { folder: Some((s.id.clone(), "Archive/g2".into())), direct: false, limit: 100, ..Default::default() }).unwrap();
        let folder_q = ms(t);

        let t = Instant::now();
        while c.enrich(500) > 0 {}
        let enrich = ms(t);

        println!("{n:>7} | {index:>9.1} | {noop:>9.1} | {rename:>8.1} | {mv:>8.1} | {folder:>10.1} | {delete:>8.1} | {relocate:>10.1} | {query:>7.2} | {search:>7.2} | {folder_q:>8.2} | {enrich:>9.1} | {:>7.0}", rss_mb());
    }
}
