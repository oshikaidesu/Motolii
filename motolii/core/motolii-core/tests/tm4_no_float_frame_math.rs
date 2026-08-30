
use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
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
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_float_math_between_time_and_frames_outside_core() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let core = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut files = Vec::new();
    collect_rs(&workspace, &mut files);
    assert!(!files.is_empty(), "走査対象が無い: {}", workspace.display());

    let mut violations = Vec::new();
    for path in files {
        if path.starts_with(core) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if path.file_name().and_then(|n| n.to_str()) == Some("tm4_no_float_frame_math.rs") {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let mentions_fps = code.contains("fps") || code.contains("Fps");
            let mentions_float = code.contains("f64") || code.contains("f32");
            if mentions_fps && mentions_float {
                violations.push(format!(
                    "{}:{}: {}",
                    path.strip_prefix(&workspace).unwrap_or(&path).display(),
                    index + 1,
                    code.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "時刻とフレームの写像を浮動小数で書いている箇所がある。\n\
         正準口(`RationalTime::try_to_frame_floor` / `try_to_frame_round` / \
         `try_from_frame`)を使うこと:\n{}",
        violations.join("\n")
    );
}
