//! owns: 「外部ツールが無い環境ではスキップし、CI では落とす」という試験の方針。
//!       上流にこの方針は無い(スキップの是非は製品の判断だから)。
//!
//! 旧 workspace `crates/motolii-testkit`(8,106行)から**使う分だけ**を移した。
//! 丸ごと移さないのは、旧 testkit が `motolii-plugin` に依存しており、拡張口が
//! まだ無い(裁定13)ためである。
//!
//! 手書きの `if which_ffmpeg().is_none() { eprintln!("skip"); return; }` を各試験に
//! 書かせないための口。**手書きスキップは方針を迂回する**ので、ここを通す。

use std::path::PathBuf;

/// 外部ツールの状態。「未導入」と「導入済みだが実行失敗」を区別する
/// (区別しないと、壊れた ffmpeg を「無い」と誤診して静かに通してしまう)。
#[derive(Debug)]
pub enum ToolStatus {
    Ok,
    NotInstalled,
    Failed(String),
}

pub fn tool_status(bin: &str) -> ToolStatus {
    match std::process::Command::new(bin).arg("-version").output() {
        Ok(out) if out.status.success() => ToolStatus::Ok,
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let head = stderr.lines().next().unwrap_or("");
            ToolStatus::Failed(if head.is_empty() {
                format!("`{bin} -version` exited with {}", out.status)
            } else {
                format!("`{bin} -version` exited with {} — {head}", out.status)
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => ToolStatus::NotInstalled,
        Err(e) => ToolStatus::Failed(format!("failed to spawn `{bin}`: {e}")),
    }
}

/// CI では依存の欠落を**スキップさせない**。手元では黙って飛ばす。
fn deps_required() -> bool {
    std::env::var("MOTOLII_REQUIRE_DEPS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn unavailable_dep(dep: &str, detail: &str) -> bool {
    if deps_required() {
        panic!("MOTOLII_REQUIRE_DEPS=1 だが {dep} が使えない: {detail}");
    }
    eprintln!("skip: {dep} が使えないので飛ばす({detail})");
    false
}

/// ffmpeg / ffprobe が両方使えるか。使えなければ試験をスキップする(戻り値 false)。
pub fn ffmpeg_or_skip() -> bool {
    for bin in ["ffmpeg", "ffprobe"] {
        match tool_status(bin) {
            ToolStatus::Ok => {}
            ToolStatus::NotInstalled => {
                return unavailable_dep(bin, "PATH に無い(未導入)");
            }
            ToolStatus::Failed(detail) => return unavailable_dep(bin, &detail),
        }
    }
    true
}

pub fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmp_dir: create_dir_all");
    dir
}

/// CPU 側の正解。GPU/ffmpeg の結果をこれと突き合わせる。
pub mod cpu_reference {
    /// limited range(BT.601/709 共通)の Y 値。
    pub fn expected_luma(gray: u8) -> i32 {
        (16.0 + 219.0 * gray as f64 / 255.0).round() as i32
    }
}
