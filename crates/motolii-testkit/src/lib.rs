//! motolii-testkit: ゴールデン画像テストの共通部品。
//!
//! 許容誤差・平均誤差・差分画像を同じ規則で扱うための薄い基盤。
//! GPU/一時ディレクトリのテストヘルパーは `gpu_or_skip` / `tmp_dir` に集約する。

use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgba};

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
    /// 差分を可視化するRGBA。RGBに差分を白く入れ、Aは255固定。
    pub diff_rgba: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldenArtifacts {
    pub reference: PathBuf,
    pub actual: PathBuf,
    pub diff: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum TestkitError {
    #[error("{label}: image size mismatch: actual={actual} expected={expected}")]
    SizeMismatch {
        label: String,
        actual: usize,
        expected: usize,
    },
    #[error(
        "{label}: image diff exceeded tolerance: max={max} > {tolerance}, mean={mean:.3}, differing={differing}/{compared}"
    )]
    ToleranceExceeded {
        label: String,
        max: u8,
        mean: f64,
        differing: usize,
        compared: usize,
        tolerance: u8,
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

/// ゴールデン失敗時の差分PNG出力先。未設定ならアーティファクト保存は行わない。
pub fn artifact_dir_from_env() -> Option<PathBuf> {
    std::env::var_os("OC_TESTKIT_ARTIFACT_DIR").map(PathBuf::from)
}

pub fn compare_rgba(
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
) -> Result<ImageDiff, TestkitError> {
    compare_rgba_labeled("compare_rgba", desc, actual, expected)
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
            expected: expected.len(),
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
        diff_rgba[i] = if channel == 3 {
            255
        } else {
            d.saturating_mul(4)
        };
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

pub fn save_rgba_png(
    path: impl AsRef<Path>,
    desc: RgbaImageDesc,
    rgba: &[u8],
) -> Result<(), TestkitError> {
    save_rgba_png_labeled(path, "save_rgba_png", desc, rgba)
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

    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(desc.width, desc.height, rgba.to_vec())
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

pub fn write_golden_artifacts(
    dir: impl AsRef<Path>,
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
    diff: &ImageDiff,
) -> Result<GoldenArtifacts, TestkitError> {
    let dir = dir.as_ref();
    std::fs::create_dir_all(dir).map_err(|source| TestkitError::CreateArtifactDir {
        path: dir.to_path_buf(),
        source,
    })?;

    let paths = golden_artifact_paths(dir, label);
    save_rgba_png_labeled(&paths.reference, label, desc, expected)?;
    save_rgba_png_labeled(&paths.actual, label, desc, actual)?;
    save_rgba_png_labeled(&paths.diff, label, desc, &diff.diff_rgba)?;

    Ok(paths)
}

pub fn write_golden_artifacts_if_configured(
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
    diff: &ImageDiff,
    artifact_dir: Option<&Path>,
) -> Result<Option<GoldenArtifacts>, TestkitError> {
    let Some(dir) = artifact_dir
        .map(Path::to_path_buf)
        .or_else(artifact_dir_from_env)
    else {
        return Ok(None);
    };
    write_golden_artifacts(&dir, label, desc, actual, expected, diff).map(Some)
}

pub fn assert_rgba_close(
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
    tolerance: u8,
) {
    assert_rgba_close_with_artifacts(label, desc, actual, expected, tolerance, None);
}

pub fn assert_rgba_close_with_artifacts(
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
    tolerance: u8,
    artifact_dir: Option<&Path>,
) {
    let diff =
        compare_rgba_labeled(label, desc, actual, expected).unwrap_or_else(|err| panic!("{err}"));

    if diff.stats.max_abs_diff > tolerance {
        if let Err(err) =
            write_golden_artifacts_if_configured(label, desc, actual, expected, &diff, artifact_dir)
        {
            eprintln!("{label}: failed to write golden artifacts: {err}");
        }

        panic!(
            "{}",
            TestkitError::ToleranceExceeded {
                label: label.into(),
                max: diff.stats.max_abs_diff,
                mean: diff.stats.mean_abs_diff,
                differing: diff.stats.differing_bytes,
                compared: diff.stats.compared_bytes,
                tolerance,
            }
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

fn golden_artifact_paths(dir: &Path, label: &str) -> GoldenArtifacts {
    let safe = label.replace(['/', '\\', ':'], "_");
    GoldenArtifacts {
        reference: dir.join(format!("{safe}-reference.png")),
        actual: dir.join(format!("{safe}-actual.png")),
        diff: dir.join(format!("{safe}-diff.png")),
    }
}

/// lavapipe等が無い環境ではテストをスキップする。
pub fn gpu_or_skip() -> Option<motolii_gpu::GpuCtx> {
    match motolii_gpu::GpuCtx::new_headless() {
        Ok(gpu) => Some(gpu),
        Err(e) => {
            eprintln!("SKIP: no GPU adapter: {e}");
            None
        }
    }
}

/// プロセス固有の一時ディレクトリ。統合テストの入出力用。
pub fn tmp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("tmp_dir: create_dir_all");
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn temp_artifact_dir(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "motolii-testkit-{test_name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn exact_images_have_zero_diff() {
        let desc = RgbaImageDesc {
            width: 1,
            height: 1,
        };
        let img = [1, 2, 3, 255];
        let diff = compare_rgba(desc, &img, &img).unwrap();
        assert_eq!(diff.stats.max_abs_diff, 0);
        assert_eq!(diff.stats.differing_bytes, 0);
    }

    #[test]
    fn reports_max_mean_and_visual_diff() {
        let desc = RgbaImageDesc {
            width: 1,
            height: 1,
        };
        let diff = compare_rgba(desc, &[10, 20, 30, 255], &[8, 25, 30, 255]).unwrap();
        assert_eq!(diff.stats.max_abs_diff, 5);
        assert_eq!(diff.stats.differing_bytes, 2);
        assert_eq!(diff.diff_rgba, vec![8, 20, 0, 255]);
    }

    #[test]
    fn mismatch_writes_reference_actual_and_diff_png() {
        let dir = temp_artifact_dir("mismatch_writes_png");
        let _ = std::fs::remove_dir_all(&dir);

        let desc = RgbaImageDesc {
            width: 2,
            height: 1,
        };
        let actual = [255, 0, 0, 255, 0, 255, 0, 255];
        let expected = [0, 0, 0, 255, 0, 0, 0, 255];
        let diff = compare_rgba(desc, &actual, &expected).unwrap();

        let artifacts =
            write_golden_artifacts(&dir, "golden", desc, &actual, &expected, &diff).unwrap();

        assert!(artifacts.reference.is_file());
        assert!(artifacts.actual.is_file());
        assert!(artifacts.diff.is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn assert_close_writes_diff_png_on_intentional_failure() {
        let _lock = env_lock();
        let dir = temp_artifact_dir("assert_failure");
        let _ = std::fs::remove_dir_all(&dir);

        let previous = std::env::var_os("OC_TESTKIT_ARTIFACT_DIR");
        std::env::set_var("OC_TESTKIT_ARTIFACT_DIR", &dir);

        let desc = RgbaImageDesc {
            width: 1,
            height: 1,
        };
        let actual = [255, 0, 0, 255];
        let expected = [0, 0, 0, 255];

        let panic = std::panic::catch_unwind(|| {
            assert_rgba_close("broken-golden", desc, &actual, &expected, 0);
        });
        assert!(panic.is_err());

        let diff_path = dir.join("broken-golden-diff.png");
        assert!(diff_path.is_file());
        assert!(dir.join("broken-golden-reference.png").is_file());
        assert!(dir.join("broken-golden-actual.png").is_file());

        match previous {
            Some(value) => std::env::set_var("OC_TESTKIT_ARTIFACT_DIR", value),
            None => std::env::remove_var("OC_TESTKIT_ARTIFACT_DIR"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
