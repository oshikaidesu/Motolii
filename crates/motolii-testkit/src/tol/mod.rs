//! ゴールデン比較の許容誤差定数(M2E-2保護領域 / M2E-4で呼び出し側を定数経由へ)。
//!
//! 生リテラルで閾値を上げる1文字diffは「テスト改変」に見えにくい(監査E-2)。
//! 閾値変更はこのモジュールに局在させ、CODEOWNERS+diffゲートで保護する。

/// ビット一致を要求する比較(CPU参照同士・決定論パス)。
pub const EXACT: u8 = 0;

/// lavapipe等のGPUラスタライズで±1が出うる比較。
pub const GPU_RASTER: u8 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_documented_values() {
        assert_eq!(EXACT, 0);
        assert_eq!(GPU_RASTER, 1);
    }
}
