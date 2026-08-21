//! owns: 「外部ツールが無い環境ではスキップし、CI では落とす」という試験の方針。
//!       上流にこの方針は無い(スキップの是非は製品の判断だから)。
//!
//! 旧 workspace `crates/motolii-testkit`(8,106行)から**使う分だけ**を移した。
//! 丸ごと移さないのは、旧 testkit が `motolii-plugin` に依存しており、拡張口が
//! まだ無い(裁定13)ためである。
//!
//! 手書きの `if which_ffmpeg().is_none() { eprintln!("skip"); return; }` を各試験に
//! 書かせないための口。**手書きスキップは方針を迂回する**ので、ここを通す。
//!
//! [`RgbaImageDesc`]/[`compare_rgba`]/[`tol`] は旧 crate の同名 API と**同じ形・
//! 同じ許容差定数**(`tol::GPU_RASTER` = max 1・mean 0.5)を next/ 側へ移した物
//! (旧 crate は `motolii-gpu`/`motolii-plugin` へ依存しており next/ から参照できない
//! — 2026-08-20 裁定の workspace 分離)。[`assert_rgba_matches_golden_file`] は
//! 参照 PNG をファイルとして持つ golden 用の口で、旧 crate には無かった形
//! (旧 crate は CPU 参照実装を都度計算する形が正本、`golden/README.md` 参照)。
//! 「無ければ初回に作り、意図した変更なら消して作り直す」規約は
//! `crates/motolii-shell-iced/tests/snapshot_start_screen.rs`(`iced_test`の
//! `Simulator::snapshot`/`matches_image`)がこのリポで既に持っている規約をそのまま
//! 借用した(GPU 生 RGBA を扱うぶん、許容差の突き合わせだけ `tol` 経由で足した)。

use std::path::{Path, PathBuf};

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

// ---------------------------------------------------------------------------
// image golden(旧 crates/motolii-testkit の compare_rgba/tol と同じ形・同じ許容差)
// ---------------------------------------------------------------------------

/// RGBA8 画像の寸法。`width*height*4` byte のバッファを指す(旧 crate と同じ形)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaImageDesc {
    pub width: u32,
    pub height: u32,
}

impl RgbaImageDesc {
    pub fn byte_len(self) -> usize {
        self.width as usize * self.height as usize * 4
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageDiffStats {
    pub max_abs_diff: u8,
    pub mean_abs_diff: f64,
    pub differing_bytes: usize,
    pub compared_bytes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageDiff {
    pub stats: ImageDiffStats,
    /// 差分を可視化する RGBA。RGB に差分を白く入れ、A は 255 固定(旧 crate と同じ)。
    pub diff_rgba: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum TestkitError {
    #[error("{label}: image size mismatch: actual={actual} expected={expected}")]
    SizeMismatch {
        label: String,
        actual: usize,
        expected: usize,
    },
    #[error("failed to create artifact directory {path}: {source}")]
    CreateArtifactDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write PNG to {path}: {source}")]
    PngWrite {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },
}

/// ゴールデン比較の許容誤差定数(旧 `crates/motolii-testkit::tol` と同じ名前・
/// 同じ値 — 発明しない)。
pub mod tol {
    /// ビット一致を要求する比較(CPU 参照同士・決定論パス)。
    pub const EXACT: u8 = 0;

    /// lavapipe 等の GPU ラスタライズで ±1 が出うる比較。
    pub const GPU_RASTER: u8 = 1;

    /// [`GPU_RASTER`] 使用時の `mean_abs_diff` 上限。
    ///
    /// max=1 だけ見ると「全画素が1ずれ」(mean≈1)の全体色ずれが合格してしまう。
    /// 縁の疎な±1は mean≪1 なので、1未満の上限で全域ずれだけを落とす。
    pub const GPU_RASTER_MEAN: f64 = 0.5;

    /// max 許容に対応する mean 上限。未知の max は定数外経路なので拒否する。
    pub fn mean_limit(max_tol: u8) -> f64 {
        match max_tol {
            EXACT => 0.0,
            GPU_RASTER => GPU_RASTER_MEAN,
            other => panic!(
                "unknown max tolerance {other}; use motolii_testkit::tol::EXACT or GPU_RASTER"
            ),
        }
    }
}

pub fn compare_rgba_labeled(
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
) -> Result<ImageDiff, TestkitError> {
    if actual.len() != desc.byte_len() || expected.len() != desc.byte_len() {
        return Err(TestkitError::SizeMismatch {
            label: label.into(),
            actual: actual.len(),
            expected: desc.byte_len(),
        });
    }

    let mut max_abs_diff = 0u8;
    let mut differing_bytes = 0usize;
    let mut sum = 0u64;
    let mut diff_rgba = vec![0u8; desc.byte_len()];

    for (i, (&a, &b)) in actual.iter().zip(expected).enumerate() {
        let d = a.abs_diff(b);
        max_abs_diff = max_abs_diff.max(d);
        if d != 0 {
            differing_bytes += 1;
        }
        sum += d as u64;

        let channel = i % 4;
        diff_rgba[i] = if channel == 3 { 255 } else { d.saturating_mul(4) };
    }

    let compared_bytes = actual.len();
    Ok(ImageDiff {
        stats: ImageDiffStats {
            max_abs_diff,
            mean_abs_diff: sum as f64 / compared_bytes as f64,
            differing_bytes,
            compared_bytes,
        },
        diff_rgba,
    })
}

pub fn save_rgba_png_labeled(
    path: impl AsRef<Path>,
    label: &str,
    desc: RgbaImageDesc,
    rgba: &[u8],
) -> Result<(), TestkitError> {
    let path = path.as_ref();
    if rgba.len() != desc.byte_len() {
        return Err(TestkitError::SizeMismatch {
            label: label.into(),
            actual: rgba.len(),
            expected: desc.byte_len(),
        });
    }

    let image =
        image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(desc.width, desc.height, rgba.to_vec())
            .ok_or_else(|| TestkitError::SizeMismatch {
                label: label.into(),
                actual: rgba.len(),
                expected: desc.byte_len(),
            })?;

    image.save(path).map_err(|source| TestkitError::PngWrite {
        path: path.to_path_buf(),
        source,
    })
}

/// 参照 PNG をファイルとして持つ golden 比較。
///
/// **無ければ初回に作り、意図した変更なら消して作り直す**
/// (`crates/motolii-shell-iced/tests/snapshot_start_screen.rs` の `iced_test`
/// snapshot 規約をそのまま踏襲 — module doc 参照)。GPU 生 RGBA はデバイス間で
/// 完全一致しないので、[`tol`] の許容差(`compare_rgba_labeled`)で突き合わせる。
///
/// 許容差を超えた場合は golden の隣に `<label>.actual.png`/`<label>.diff.png` を
/// 書き出す(旧 crate の `write_golden_artifacts` と同じ役割 — 目視で差分を追える形)。
pub fn assert_rgba_matches_golden_file(
    path: impl AsRef<Path>,
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    tolerance: u8,
) {
    let path = path.as_ref();

    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|source| {
                panic!(
                    "{}",
                    TestkitError::CreateArtifactDir {
                        path: parent.to_path_buf(),
                        source,
                    }
                )
            });
        }
        save_rgba_png_labeled(path, label, desc, actual)
            .unwrap_or_else(|err| panic!("{label}: golden を新規作成できない: {err}"));
        eprintln!(
            "{label}: golden が無かったので {path:?} へ新規作成した。\
             内容を目視で確認してからコミットすること。"
        );
        return;
    }

    let reference = image::open(path)
        .unwrap_or_else(|err| panic!("{label}: golden PNG {path:?} を読めない: {err}"))
        .to_rgba8();
    assert_eq!(
        (reference.width(), reference.height()),
        (desc.width, desc.height),
        "{label}: golden {path:?} の寸法が actual と一致しない"
    );
    let expected = reference.into_raw();

    let diff = compare_rgba_labeled(label, desc, actual, &expected)
        .unwrap_or_else(|err| panic!("{err}"));

    let mean_limit = tol::mean_limit(tolerance);
    if diff.stats.max_abs_diff > tolerance || diff.stats.mean_abs_diff > mean_limit {
        let actual_path = path.with_extension("actual.png");
        let diff_path = path.with_extension("diff.png");
        if let Err(err) = save_rgba_png_labeled(&actual_path, label, desc, actual) {
            eprintln!("{label}: actual アーティファクトを書けない: {err}");
        }
        if let Err(err) = save_rgba_png_labeled(&diff_path, label, desc, &diff.diff_rgba) {
            eprintln!("{label}: diff アーティファクトを書けない: {err}");
        }
        panic!(
            "{label}: golden {path:?} と一致しない: max={} (limit {tolerance}), \
             mean={:.3} (limit {mean_limit:.3}), differing={}/{}。\
             実際={actual_path:?} 差分={diff_path:?} を見る。\
             意図した変更なら golden を消して再実行すれば作り直せる。",
            diff.stats.max_abs_diff,
            diff.stats.mean_abs_diff,
            diff.stats.differing_bytes,
            diff.stats.compared_bytes,
        );
    }

    eprintln!(
        "{label}: max byte diff = {}, mean = {:.3}, differing = {}/{}",
        diff.stats.max_abs_diff,
        diff.stats.mean_abs_diff,
        diff.stats.differing_bytes,
        diff.stats.compared_bytes
    );
}
