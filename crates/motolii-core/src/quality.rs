use serde::{Deserialize, Serialize};

use crate::FrameDesc;

/// プレビュー/書き出しで共有する品質パラメータ。
///
/// 同一の `render_frame(..., quality)` を通し、差分はここの値だけにする(落とし穴B-4)。
/// v1の実効: `resolution_scale`(解像度)と`precise_color`(合成分岐選択 — M2E-18、実装は恒等)。
/// `effect_samples`は口のみ。`render_desc`は解像度のみ写し、色空間は変えない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Quality {
    /// 1 = full, 2 = 1/2, 4 = 1/4。内部レンダ解像度を width/scale × height/scale にする。
    pub resolution_scale: u32,
    /// false のとき sRGB 空間ブレンド等の近似を許容。
    /// 合成分岐(`select_composite_color_path`)まで配線済み。v1実装は両枝とも同一WGSL(恒等)。
    pub precise_color: bool,
    /// モーションブラー等プラグインのサンプル数の口(v1では未使用)。
    pub effect_samples: SampleTier,
}

/// エフェクトのサンプル段。Quality の口として先行定義(実装は後続)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SampleTier {
    Draft,
    Full,
}

impl Quality {
    /// 書き出し / 最終品質プレビュー / ゴールデン比較用。
    pub const FINAL: Self = Self {
        resolution_scale: 1,
        precise_color: true,
        effect_samples: SampleTier::Full,
    };

    /// 既定プレビュー(半解像度)。
    pub const DRAFT: Self = Self {
        resolution_scale: 2,
        precise_color: false,
        effect_samples: SampleTier::Draft,
    };

    /// `FrameDesc` を内部レンダ解像度へ写す。scale=1 では入力と同一。
    /// 色空間は触らない(`precise_color`の合成分岐は`select_composite_color_path`側)。
    pub fn render_desc(self, desc: FrameDesc) -> FrameDesc {
        let scale = self.resolution_scale.max(1);
        if scale == 1 {
            return desc;
        }
        let width = (desc.width / scale).max(1);
        let height = (desc.height / scale).max(1);
        FrameDesc::packed(
            width,
            height,
            desc.format,
            desc.color_space,
            desc.premultiplied,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSpace, PixelFormat};

    #[test]
    fn final_render_desc_is_identity() {
        let desc = FrameDesc::packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        assert_eq!(Quality::FINAL.render_desc(desc), desc);
    }

    #[test]
    fn draft_halves_resolution() {
        let desc = FrameDesc::packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let scaled = Quality::DRAFT.render_desc(desc);
        assert_eq!(scaled.width, 960);
        assert_eq!(scaled.height, 540);
        assert_eq!(scaled.format, desc.format);
        assert_eq!(scaled.premultiplied, desc.premultiplied);
    }

    #[test]
    fn render_desc_keeps_at_least_one_pixel() {
        let desc = FrameDesc::packed(1, 1, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let q = Quality {
            resolution_scale: 4,
            ..Quality::DRAFT
        };
        let scaled = q.render_desc(desc);
        assert_eq!(scaled.width, 1);
        assert_eq!(scaled.height, 1);
    }
}
