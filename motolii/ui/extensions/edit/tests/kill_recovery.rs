//! Process kill → restart oracle (INF-6): whatever moment the process dies, the project file and the newest auto-save
//! generation are a complete earlier save — never a torn one. The child is this test binary run again with
//! `MOTOLII_KILL_SAVE_DIR` set; it saves generation after generation until the parent kills it (SIGKILL).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use motolii_edit::persist::AutoSaveConfig;
use motolii_edit::{blank_project, Document, Intent};

const CHILD: &str = "a_child_that_saves_until_it_is_killed";

/// Generation `n` is written as the composition width 64 + n, so a loaded file names the save it is; one document
/// edited on and on, as work is.
fn generation(doc: &mut Document, n: u32) {
    let mut comp = doc.view().composition().unwrap().unwrap();
    comp.width = 64 + n;
    doc.apply(Intent::SetComposition(comp)).unwrap();
}

fn width(path: &Path) -> u32 {
    Document::load(path).unwrap_or_else(|e| panic!("{} does not load: {e:?}", path.display())).view().composition().unwrap().unwrap().width
}

#[test]
fn a_child_that_saves_until_it_is_killed() {
    let Some(dir) = std::env::var_os("MOTOLII_KILL_SAVE_DIR") else { return };
    let dir = PathBuf::from(dir);
    let project = dir.join("work.rrd");
    let config = AutoSaveConfig { interval_secs: 0, generations: 3 };
    let mut doc = blank_project();
    let mut since = doc.revision();
    for n in 1.. {
        generation(&mut doc, n);
        doc.save(&project).unwrap();
        doc.auto_save(Some(&project), &since, &config).unwrap();
        since = doc.revision();
        println!("saved {n}");
    }
}

#[test]
fn a_killed_process_leaves_a_complete_earlier_save() {
    let exe = std::env::current_exe().unwrap();
    for round in 0..12u64 {
        let dir = std::env::temp_dir().join(format!("motolii-kill-{}-{round}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut child = Command::new(&exe)
            .args([CHILD, "--exact", "--nocapture", "--test-threads=1"])
            .env("MOTOLII_KILL_SAVE_DIR", &dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        // wait for a few complete saves, then kill at a moment that differs per round (mid-write most of the time)
        let mut last = 0u32;
        while last < 3 {
            let line = lines.next().expect("child ended early").unwrap();
            if let Some(n) = line.strip_prefix("saved ") {
                last = n.parse().unwrap();
            }
        }
        std::thread::sleep(Duration::from_micros(round * 700));
        child.kill().unwrap();
        child.wait().unwrap();
        let reported = lines.map_while(Result::ok).filter_map(|l| l.strip_prefix("saved ").map(|n| n.parse::<u32>().unwrap())).max().unwrap_or(last);

        let project = dir.join("work.rrd");
        let got = width(&project) - 64;
        assert!(got >= last && got <= reported + 1, "round {round}: the project is save {got}, saved {last}..={reported}");

        let autosaves = Document::auto_save_dir(&project);
        let mut kept: Vec<PathBuf> = std::fs::read_dir(&autosaves).unwrap().map(|e| e.unwrap().path()).filter(|p| p.to_string_lossy().contains(".autosave-") && !p.file_name().unwrap().to_string_lossy().starts_with('.')).collect();
        // generations are numbered in their names (work.autosave-N.rrd); the highest is the newest
        let seq = |p: &PathBuf| p.file_name().unwrap().to_string_lossy().split(".autosave-").nth(1).and_then(|r| r.split('.').next()?.parse::<u64>().ok()).unwrap_or(0);
        kept.sort_by_key(seq);
        let newest = kept.last().expect("an auto-save generation");
        let auto = width(newest) - 64;
        assert!(auto + 1 >= last && auto <= reported + 1, "round {round}: the newest auto-save is save {auto}, saved {last}..={reported}");
        // auto_save installs the new generation before it prunes the oldest: a kill between leaves one extra, which the
        // next auto_save prunes. Never more than that.
        assert!(kept.len() <= 3 + 1, "round {round}: {} generations kept", kept.len());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
