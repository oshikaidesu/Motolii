//! layer の小さな非アニメーション属性 — hidden / parent / blend mode / matte / name /
//! auto-orient。
//!
//! **`LayerMeta` の中へ入れない**。`meta` は `Intent::SetMeta` が新規配置でしか
//! 書けない(裁定108(c) の構造修正 — `document.rs` 参照)。属性をここに同居させると、
//! 「属性を1つ変えるたびに新規配置用の口を使う」ことになり、結局そこから
//! `timing`/`source` を巻き込む道が復活する。マスクを `meta` の外へ出したのと
//! 同じ理由(裁定108(a))で、こちらも別 component にする。

use serde::{Deserialize, Serialize};

use crate::LayerId;

/// 手前までの覆いと、このマスクの覆いをどう重ねるか、ではなく——
/// blend mode の16値(Lottie `constants/blend-mode` = AE / Photoshop / peniko / wgpu で
/// 共通の語彙、裁定67)。発明の余地が無いのでそのまま採る。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// matte の重ね方(Lottie `constants/matte-mode` の4値、AE の語彙、裁定66)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

/// 「このレイヤは、あの layer を、この mode でマットにする」を**1フィールドに畳む**
/// (裁定66)。`tp` 省略時に「1つ上のレイヤ」を暗黙参照する規則は採らない —
/// 並べ替えが合成結果を黙って変えるのは編集ソフトとして致命的なので、参照は常に明示。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matte {
    pub layer: LayerId,
    pub mode: MatteMode,
}

/// layer の非アニメーション属性のうち、`meta`(素材・重ね順・配置)ではない残り全部。
///
/// **`Intent::SetAttrs` は丸ごと差し替え**(`SetMasks`/`SetMarkers` と同じ形)。
/// これは安全 — `meta` の `timing`/`source`/`order` を一切含まないので、属性を
/// 差し替えても他の component は触れない(component が分かれている以上、構造的に
/// 巻き込めない)。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerAttrs {
    /// `layers/layer/hd`(Hidden)。GOALS 標準の mute。tombstone の `present`
    /// (削除)とは別物 — hidden な layer は Undo で戻る「編集」で、present=false は
    /// 「無かったことにする」墓標。
    pub hidden: bool,
    /// `layers/layer/parent`(Parent)。参照は `ind` ではなく `LayerId`(裁定65)。
    /// 循環参照は書き口([`crate::Intent::SetAttrs`])が拒む。
    pub parent: Option<LayerId>,
    /// `layers/visual-layer/bm`(Blend Mode)。
    pub blend_mode: BlendMode,
    /// matte(裁定66)。`None` = マットにされていない。
    pub matte: Option<Matte>,
    /// `helpers/visual-object/nm`(Name)。人が付ける表示名。同一性は `LayerId` のまま
    /// (裁定65 と同じ理由 — 名前で layer を探させない)。
    pub name: String,
    /// `layers/visual-layer/ao`(Auto Orient)。`position` が Vec2 単一 track なので
    /// 定義できる(裁定61 の余得)。
    pub auto_orient: bool,
    /// **カメラ変換を受けず画面に張り付く**(裁定113)。AE の「2Dレイヤはカメラを
    /// 無視」という空間分割の形は採らず、明示属性1つに畳んである。既定 false
    /// (裁定113「pinned は明示属性」)。
    pub pinned: bool,
}

impl Default for LayerAttrs {
    fn default() -> Self {
        Self {
            hidden: false,
            parent: None,
            blend_mode: BlendMode::default(),
            matte: None,
            name: String::new(),
            auto_orient: false,
            pinned: false,
        }
    }
}
