//! oc-testkit: ゴールデン画像テストの共通部品。
//!
//! ここはGPUやメディア実装へ依存しない。各クレートのテストがRGBAバッファを渡し、
//! 許容誤差・平均誤差・差分画像を同じ規則で扱うための薄い基盤。

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
}

pub fn compare_rgba(
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
) -> Result<ImageDiff, TestkitError> {
    if actual.len() != desc.byte_len() || expected.len() != desc.byte_len() {
        return Err(TestkitError::SizeMismatch {
            label: "compare_rgba".into(),
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

pub fn assert_rgba_close(
    label: &str,
    desc: RgbaImageDesc,
    actual: &[u8],
    expected: &[u8],
    tolerance: u8,
) {
    let diff = compare_rgba(desc, actual, expected).unwrap_or_else(|err| panic!("{label}: {err}"));
    assert!(
        diff.stats.max_abs_diff <= tolerance,
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
    eprintln!(
        "{label}: max byte diff = {}, mean = {:.3}, differing = {}/{}",
        diff.stats.max_abs_diff,
        diff.stats.mean_abs_diff,
        diff.stats.differing_bytes,
        diff.stats.compared_bytes
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
