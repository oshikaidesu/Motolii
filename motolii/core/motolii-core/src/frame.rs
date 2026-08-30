use serde::{Deserialize, Serialize};

use crate::RationalTime;

/// ピクセルフォーマット。パック系はwgpu::TextureFormatに1:1で対応させる前提の命名。
/// YUV系はデコード直後のCPUフレーム用(GPUへはアップロード時に変換シェーダを通す)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Rgba16Float,
    Rgba32Float,
    /// planar 4:2:0 (Y面 + U面 + V面)
    Yuv420p,
    /// semi-planar 4:2:0 (Y面 + UVインターリーブ面)
    Nv12,
}

impl PixelFormat {
    /// パック系フォーマットの1ピクセルあたりバイト数。YUV系(プレーナ)はNone。
    pub fn bytes_per_pixel(&self) -> Option<u32> {
        match self {
            PixelFormat::Rgba8Unorm | PixelFormat::Rgba8UnormSrgb | PixelFormat::Bgra8Unorm => {
                Some(4)
            }
            PixelFormat::Rgba16Float => Some(8),
            PixelFormat::Rgba32Float => Some(16),
            PixelFormat::Yuv420p | PixelFormat::Nv12 => None,
        }
    }

    pub fn is_yuv(&self) -> bool {
        matches!(self, PixelFormat::Yuv420p | PixelFormat::Nv12)
    }
}

/// 色空間タグ。「タグを持つ」だけでなく変換の正しさはゴールデンテストで守る(落とし穴B-3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorSpace {
    /// リニアRGB。precise_color時の合成空間の受け皿(M2E-18)。v1合成はsRGB空間ブレンド(M2E-13③)
    LinearRgb,
    /// sRGB伝達関数つきRGB
    Srgb,
    /// BT.709 limited range (16-235)。動画デコード出力の既定
    Rec709Limited,
    /// BT.709 full range (0-255)
    Rec709Full,
    /// BT.601 limited range(SD素材・古い素材)
    Rec601Limited,
}

/// プラグイン/クレート間で受け渡すフレームの記述子。
/// docs/concept.md「フレーム記述子」決定(2026-07-06)をコード化したもの。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameDesc {
    pub width: u32,
    pub height: u32,
    /// 行バイト数(パック系のみ意味を持つ。YUV系は各プレーンがタイトパッキング前提)
    pub stride: u32,
    pub format: PixelFormat,
    pub color_space: ColorSpace,
    /// アルファがプリマルチプライ済みか(取り違えは黒フリンジとして出る)
    pub premultiplied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FrameDescError {
    #[error("FrameDesc::packed: packed format required")]
    PackedFormatRequired,
    #[error("FrameDesc::yuv: yuv format required")]
    YuvFormatRequired,
    #[error("FrameDesc::yuv: 4:2:0 requires even dimensions")]
    OddDimensions,
}

impl FrameDesc {
    /// パディングなし(stride = width * bpp)のパック系記述子を作る。
    pub fn packed(
        width: u32,
        height: u32,
        format: PixelFormat,
        color_space: ColorSpace,
        premultiplied: bool,
    ) -> Self {
        Self::try_packed(width, height, format, color_space, premultiplied)
            .expect("FrameDesc::packed: invalid arguments")
    }

    pub fn try_packed(
        width: u32,
        height: u32,
        format: PixelFormat,
        color_space: ColorSpace,
        premultiplied: bool,
    ) -> Result<Self, FrameDescError> {
        let bpp = format
            .bytes_per_pixel()
            .ok_or(FrameDescError::PackedFormatRequired)?;
        Ok(Self {
            width,
            height,
            stride: width * bpp,
            format,
            color_space,
            premultiplied,
        })
    }

    /// YUV系(タイトパッキング)の記述子を作る。4:2:0のため偶数サイズのみ。
    pub fn yuv(width: u32, height: u32, format: PixelFormat, color_space: ColorSpace) -> Self {
        Self::try_yuv(width, height, format, color_space)
            .expect("FrameDesc::yuv: invalid arguments")
    }

    pub fn try_yuv(
        width: u32,
        height: u32,
        format: PixelFormat,
        color_space: ColorSpace,
    ) -> Result<Self, FrameDescError> {
        if !format.is_yuv() {
            return Err(FrameDescError::YuvFormatRequired);
        }
        if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
            return Err(FrameDescError::OddDimensions);
        }
        Ok(Self {
            width,
            height,
            stride: width,
            format,
            color_space,
            premultiplied: false,
        })
    }

    /// フレーム全体のバイト数。
    pub fn data_size(&self) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        match self.format {
            PixelFormat::Yuv420p | PixelFormat::Nv12 => w * h + 2 * (w / 2) * (h / 2),
            _ => self.stride as usize * h,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 {
            return Err("zero dimension".into());
        }
        if let Some(bpp) = self.format.bytes_per_pixel() {
            if self.stride < self.width * bpp {
                return Err(format!(
                    "stride {} < width {} * bpp {}",
                    self.stride, self.width, bpp
                ));
            }
        }
        if self.format.is_yuv() && (!self.width.is_multiple_of(2) || !self.height.is_multiple_of(2))
        {
            return Err("4:2:0 requires even dimensions".into());
        }
        Ok(())
    }

    /// 同一アスペクト比かつ、どちらかが他方の整数倍解像度か(Draft縮小の動画背景など)。
    pub fn same_aspect_integer_scale(self, other: Self) -> bool {
        if self.width == 0 || self.height == 0 || other.width == 0 || other.height == 0 {
            return false;
        }
        if u64::from(self.width) * u64::from(other.height)
            != u64::from(self.height) * u64::from(other.width)
        {
            return false;
        }
        let (large_w, large_h, small_w, small_h) = if self.width >= other.width {
            (self.width, self.height, other.width, other.height)
        } else {
            (other.width, other.height, self.width, self.height)
        };
        if large_w % small_w != 0 || large_h % small_h != 0 {
            return false;
        }
        large_w / small_w == large_h / small_h
    }
}

/// CPU側メモリに載ったフレーム。デコード出力・書き出し入力・テストで使う。
/// GPU上のフレームはmotolii-gpuのTextureHandleが担う(このクレートはGPU非依存)。
#[derive(Debug, Clone)]
pub struct CpuFrame {
    pub desc: FrameDesc,
    /// フレームの表示時刻(素材内ローカル時刻)
    pub pts: RationalTime,
    pub data: Vec<u8>,
}

impl CpuFrame {
    pub fn new(desc: FrameDesc, pts: RationalTime, data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), desc.data_size());
        Self { desc, pts, data }
    }
}

/// Straight RGBAをpremultiplied RGBAへ変換する。
///
/// UIやユーザー入力の色はstraightとして受け取り、GPU合成境界へ入る前にこの関数へ集約する。
pub fn premultiply_rgba_f32(mut rgba: [f32; 4]) -> [f32; 4] {
    rgba[0] *= rgba[3];
    rgba[1] *= rgba[3];
    rgba[2] *= rgba[3];
    rgba
}

/// u8 straight RGBAをpremultiplied RGBAへ変換する。テスト・CPU素材の境界用。
pub fn premultiply_rgba_u8(rgba: [u8; 4]) -> [u8; 4] {
    let a = rgba[3] as u16;
    [
        ((rgba[0] as u16 * a + 127) / 255) as u8,
        ((rgba[1] as u16 * a + 127) / 255) as u8,
        ((rgba[2] as u16 * a + 127) / 255) as u8,
        rgba[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_desc_and_size() {
        let d = FrameDesc::packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        assert_eq!(d.stride, 1920 * 4);
        assert_eq!(d.data_size(), 1920 * 1080 * 4);
        assert!(d.validate().is_ok());
    }

    #[test]
    fn yuv420_size() {
        let d = FrameDesc::yuv(64, 48, PixelFormat::Yuv420p, ColorSpace::Rec709Limited);
        assert_eq!(d.data_size(), 64 * 48 * 3 / 2);
        assert!(d.validate().is_ok());
    }

    #[test]
    fn validate_rejects_bad_stride() {
        let mut d = FrameDesc::packed(16, 16, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        d.stride = 16; // 16*4より小さい
        assert!(d.validate().is_err());
    }

    #[test]
    fn premultiplies_straight_color() {
        assert_eq!(premultiply_rgba_u8([255, 128, 0, 128]), [128, 64, 0, 128]);
        assert_eq!(
            premultiply_rgba_f32([1.0, 0.5, 0.25, 0.5]),
            [0.5, 0.25, 0.125, 0.5]
        );
    }

    #[test]
    fn same_aspect_integer_scale_accepts_draft_halving() {
        let full = FrameDesc::packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let draft = FrameDesc::packed(960, 540, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        assert!(full.same_aspect_integer_scale(draft));
    }

    #[test]
    fn same_aspect_integer_scale_rejects_mismatched_aspect() {
        let a = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let b = FrameDesc::packed(8, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        assert!(!a.same_aspect_integer_scale(b));
    }

    #[test]
    fn same_aspect_integer_scale_rejects_non_integer_scale() {
        let a = FrameDesc::packed(640, 360, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let b = FrameDesc::packed(480, 270, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        assert!(!a.same_aspect_integer_scale(b));
    }

    #[test]
    fn same_aspect_integer_scale_rejects_zero_dimension() {
        let valid = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let zero_w = FrameDesc::packed(0, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        let zero_h = FrameDesc::packed(8, 0, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
        assert!(!valid.same_aspect_integer_scale(zero_w));
        assert!(!valid.same_aspect_integer_scale(zero_h));
        assert!(!zero_w.same_aspect_integer_scale(valid));
    }

    #[test]
    fn same_aspect_integer_scale_handles_large_dimensions_without_overflow() {
        let a = FrameDesc::packed(
            65536,
            65536,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            true,
        );
        let b = FrameDesc::packed(
            65536,
            65536,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            true,
        );
        assert!(a.same_aspect_integer_scale(b));
        let c = FrameDesc::packed(
            65536,
            65537,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            true,
        );
        assert!(!a.same_aspect_integer_scale(c));
    }

    #[test]
    fn try_yuv_rejects_odd_dimensions() {
        assert!(matches!(
            FrameDesc::try_yuv(63, 48, PixelFormat::Yuv420p, ColorSpace::Rec709Limited),
            Err(FrameDescError::OddDimensions)
        ));
    }

    #[test]
    #[should_panic]
    fn yuv_rejects_odd_dimensions() {
        FrameDesc::yuv(63, 48, PixelFormat::Yuv420p, ColorSpace::Rec709Limited);
    }
}

/// 合成の器。comp の解像度だけを持つ。
///
/// **`motolii-compositor` ではなく core に置いてある**。ここに置く前は
/// `motolii-export` が `CompSpec` を取るためだけに `motolii-compositor` を直接
/// 引いており、**export が第二の `Compositor` を建てられる**状態だった
/// (2026-08-20 の敵対的レビュー。背骨2「第二経路を作れる公開 API を置かない」に反する)。
/// 型を core へ出すと依存ごと切れて、**建てられなくなる**。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompSpec {
    pub width: u32,
    pub height: u32,
}

/// comp 座標(ピクセル・左上原点)での layer の置き方。
///
/// **意味は Lottie(= 実質 OSS の AE 解析)から取っている**(裁定58)。
/// 公式仕様 + velato + zimond/lottie-rs の3つで裏を取った適用順序:
///
/// ```text
/// translate(position) · rotate(rotation) · skew · scale · translate(-anchor)
/// ```
///
/// 読み方: anchor をローカル原点へ引き寄せる → 拡大 → skew → 回転 → position へ置く。
/// **anchor は 0..1 の正規化ピボットではなく、レイヤ自身の座標単位の点**。
///
/// 以前は `top_left` + `size` を持っていたが両方とも捨てた(裁定59・60):
/// - `top_left` は回転を入れた瞬間に保存値として意味を失う。**position は anchor が
///   着地する点**である
/// - 「レイヤの寸法」property は Lottie に存在しない。寸法は素材固有で、大きさは
///   `scale` でしか動かない。両方持つと**同じことを言う正本が2つ**になる
///
/// `opacity` は**行列の外**にある。Lottie は形式上 transform の中に置くが、行列に
/// 合成せず親からも継承しない(velato も剥がして返す)。
/// **継承しないものを継承する箱に入れない**(裁定62)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerPlacement {
    /// レイヤ座標 → comp 座標。上の順序で組んである。
    pub transform: glam::Affine2,
    /// 重ね順。大きいほど手前。上流の `re_renderer::DepthOffset` と同じ `i16`。
    pub order: i16,
    /// 0.0〜1.0。
    pub opacity: f32,
    /// `position.z`(裁定113/116)。**世界は1つ・全員 z=0 既定**。単位はピクセルと同じ
    /// world 単位。AE と同じ符号(大きいほどカメラから遠ざかる、奥)。
    /// `transform`(`Affine2`)は z 軸まわりの回転しか表せないので、奥行きと傾きは
    /// このフィールドと `rotation_x`/`rotation_y` が別に持つ。
    pub z: f32,
    /// 板を傾ける回転(度)。`transform` の回転は z 軸まわりで、この2つはその外。
    pub rotation_x: f32,
    pub rotation_y: f32,
}

impl Default for LayerPlacement {
    fn default() -> Self {
        Self {
            transform: glam::Affine2::IDENTITY,
            order: 0,
            opacity: 1.0,
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
        }
    }
}

impl LayerPlacement {
    /// Lottie の適用順序で行列を組む。**ここが transform の意味の正本**。
    ///
    /// - `anchor` / `position`: comp 座標のピクセル
    /// - `scale`: 1.0 が等倍(Lottie のパーセントは採らない、裁定58)
    /// - `rotation_degrees`: 時計回りの度(AE と同じ。ラジアンは人が読めない)
    /// - `skew_degrees` / `skew_axis_degrees`: skew の量と軸。**軸に沿った点は動かない**
    ///   (`skew_axis_degrees=0` なら x 軸上、`90` なら y 軸上の点が不動点になる)。
    ///   式は `velato`(linebender/velato, `src/runtime/model/animated.rs`)の
    ///   `Transform::evaluate` と一致させてある: `R(-axis) · shear_x(tan(skew)) · R(axis)`。
    pub fn from_transform(
        anchor: [f32; 2],
        position: [f32; 2],
        scale: [f32; 2],
        rotation_degrees: f32,
        skew_degrees: f32,
        skew_axis_degrees: f32,
    ) -> glam::Affine2 {
        use glam::{Affine2, Mat2, Vec2};

        let skew_matrix = if skew_degrees == 0.0 {
            Affine2::IDENTITY
        } else {
            let skew = skew_degrees.to_radians();
            let axis = skew_axis_degrees.to_radians();
            // x 方向だけの shear。x' = x + tan(skew)*y, y' = y。
            let shear = Affine2::from_mat2(Mat2::from_cols(
                Vec2::new(1.0, 0.0),
                Vec2::new(skew.tan(), 1.0),
            ));
            Affine2::from_angle(-axis) * shear * Affine2::from_angle(axis)
        };

        // 列ベクトル規約。右から順に「anchor を引く → 拡大 → skew → 回転 → position へ」。
        Affine2::from_translation(Vec2::new(position[0], position[1]))
            * Affine2::from_angle(rotation_degrees.to_radians())
            * skew_matrix
            * Affine2::from_scale(Vec2::new(scale[0], scale[1]))
            * Affine2::from_translation(Vec2::new(-anchor[0], -anchor[1]))
    }
}

#[cfg(test)]
mod layer_placement_tests {
    use super::LayerPlacement;
    use glam::Vec2;

    fn approx(a: Vec2, b: Vec2) {
        assert!(
            (a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5,
            "{a:?} != {b:?}"
        );
    }

    /// skew=0 は恒等。skew_axis がどんな値でも、量が0なら何も動かさない。
    #[test]
    fn zero_skew_does_not_move_points() {
        let t = LayerPlacement::from_transform([0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 0.0, 37.0);
        approx(t.transform_point2(Vec2::new(1.0, 0.0)), Vec2::new(1.0, 0.0));
        approx(t.transform_point2(Vec2::new(0.0, 1.0)), Vec2::new(0.0, 1.0));
    }

    /// skew_axis=0(x軸)で skew すると、x軸上の点は不動、y方向の点は x へずれる。
    #[test]
    fn skew_along_x_axis_leaves_the_x_axis_fixed() {
        let t = LayerPlacement::from_transform([0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 45.0, 0.0);
        approx(t.transform_point2(Vec2::new(1.0, 0.0)), Vec2::new(1.0, 0.0));
        approx(t.transform_point2(Vec2::new(0.0, 1.0)), Vec2::new(1.0, 1.0));
    }

    /// skew_axis=90(y軸)で skew すると、y軸上の点は不動、x方向の点が y へずれる。
    /// **軸が変わると不動点も変わる** — sk/sa が独立変数であることの実証。
    #[test]
    fn skew_along_y_axis_leaves_the_y_axis_fixed() {
        let t = LayerPlacement::from_transform([0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 45.0, 90.0);
        approx(t.transform_point2(Vec2::new(0.0, 1.0)), Vec2::new(0.0, 1.0));
        approx(t.transform_point2(Vec2::new(1.0, 0.0)), Vec2::new(1.0, -1.0));
    }
}
