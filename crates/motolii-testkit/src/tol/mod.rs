//! ゴールデン比較の許容誤差定数(M2E-2保護領域 / M2E-4で呼び出し側を定数経由へ)。
//!
//! 生リテラルで閾値を上げる1文字diffは「テスト改変」に見えにくい(監査E-2)。
//! 閾値変更はこのモジュールに局在させ、CODEOWNERS+diffゲートで保護する。

/// ビット一致を要求する比較(CPU参照同士・決定論パス)。
pub const EXACT: u8 = 0;

/// lavapipe等のGPUラスタライズで±1が出うる比較。
pub const GPU_RASTER: u8 = 1;

/// [`GPU_RASTER`] 使用時の `mean_abs_diff` 上限。
///
/// max=1だけ見ると「全画素が1ずれ」(mean≈1)の全体色ずれが合格してしまう(監査E-2)。
/// 縁の疎な±1は mean≪1 なので、1未満の上限で全域ずれだけを落とす。
pub const GPU_RASTER_MEAN: f64 = 0.5;

/// max許容に対応する mean 上限。未知の max は定数外経路なので拒否する。
pub fn mean_limit(max_tol: u8) -> f64 {
    match max_tol {
        EXACT => 0.0,
        GPU_RASTER => GPU_RASTER_MEAN,
        other => {
            panic!("unknown max tolerance {other}; use motolii_testkit::tol::EXACT or GPU_RASTER")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_documented_values() {
        assert_eq!(EXACT, 0);
        assert_eq!(GPU_RASTER, 1);
        assert_eq!(GPU_RASTER_MEAN, 0.5);
        assert_eq!(mean_limit(EXACT), 0.0);
        assert_eq!(mean_limit(GPU_RASTER), GPU_RASTER_MEAN);
    }

    #[test]
    #[should_panic(expected = "unknown max tolerance")]
    fn mean_limit_rejects_unknown_max() {
        let _ = mean_limit(2);
    }
}

// M2E-2 ruleset live verify: CODEOWNERS review required (agent-run 2026-07-11T22:58Z)
