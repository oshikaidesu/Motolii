//! TM-4: 時刻→フレームの独自 f64 経路がクレート内に残っていないことを走査で固定する。

use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &[
    "as_f64()).round()",
    "fps.as_f64()).round()",
    "secs * fps.as_f64()",
    "start_frame as f64 - 0.5",
    "floor(secs*",
];

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn is_allowed(path: &Path) -> bool {
    path.ends_with("motolii-core/src/time.rs")
        || path.ends_with("motolii-core/tests/tm4_no_scattered_frame_conversion.rs")
}

#[test]
fn workspace_has_no_scattered_time_to_frame_f64_paths() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates_root = workspace.join("crates");
    let mut files = Vec::new();
    collect_rs_files(&crates_root, &mut files);
    assert!(
        !files.is_empty(),
        "expected Rust sources under {}",
        crates_root.display()
    );

    let mut violations = Vec::new();
    for path in files {
        if is_allowed(&path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for pattern in FORBIDDEN {
            if text.contains(pattern) {
                violations.push(format!(
                    "{}: forbidden pattern {:?}",
                    path.strip_prefix(&workspace).unwrap_or(&path).display(),
                    pattern
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "scattered time→frame f64 paths:\n{}",
        violations.join("\n")
    );
}
