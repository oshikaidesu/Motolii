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
///
/// **`Add` は Lottie の16値には無い**(裁定67「Add は velato も落としているので
/// 後回し」— `motolii_compositor::BlendMode` のモジュール doc 参照)。合成器側は
/// 無改造で出せる(`multiplicative_tint.a = 0` で `out = src + dst`)ことが先に
/// わかっていたので、engine が対応できる分だけ後追いでここへ足す(BL2 縦一本)。
/// 並びは Normal の直後(AE のメニュー順同型)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
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
/// **格納形は丸ごと1 component**(`SetMasks`/`SetMarkers` と同じ形)だが、
/// **書き口([`crate::Intent::SetAttrs`])は丸ごと差し替えではない** — 受け取るのは
/// [`LayerAttrsPatch`](フィールドごとに「触るか」を持つ部分更新)で、`write` が現在の
/// `attrs` を読んでから `Some` なフィールドだけ重ねる。`meta` の `timing`/`source`/
/// `order` を一切含まないので他 component は構造的に巻き込めないが、**`attrs` 自身の
/// 中の他フィールド**(name/blend_mode/auto_orient 等)は同じ component なので、
/// 丸ごと差し替えのままだと1階層下で同じ事故が起きる(2026-08-20 の敵対的レビュー、
/// `LayerAttrsPatch` のドキュメント参照)。
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
    /// solo。**描画に効く** — comp 内のいずれかの layer が `solo=true` のとき、
    /// solo でない layer は(hidden と同じ経路で)描かれない([`crate::view`] の
    /// `resolve` 参照)。**hidden が勝つ**: hidden な layer は自分が solo でも出ない
    /// (隠した層を solo で復活させない)。将来のグループ AND 導出(裁定119)に矛盾しない
    /// よう、単層の bool のまま素直に持つ(グループの solo は子の solo から導出する側の
    /// 仕事であって、ここに特別な意味を足さない)。既定 false。
    pub solo: bool,
    /// locked。**描画には無関係**(合成器・resolve は読まない) — 効くのは書き口。
    /// `Document::write` が locked な layer への層変更 Intent(`SetTiming`/`SetSource`/
    /// `SetOrder`/`SetMasks`/`SetEffects`/`SetShapes`/`SetTextDocument`/`SetTrack`/
    /// `SetPropertySlot`、および `SetAttrs` の `locked` 以外のフィールド)を理由つき
    /// `Err` で拒む。**`locked` 自身の解除だけは常に通す** — 自分をロックしたら二度と
    /// 触れなくなる、という詰みを作らない。既定 false。
    pub locked: bool,
    /// レイヤー差し色(利用者裁定2026-08-21「色が足りない。Abletonはレイヤー全部に
    /// 色」)。**RGB ではなくパレット index を保存**(AE ラベル色と同型 — テーマ
    /// (パレットの実体色)を後から差し替えても既存ドキュメントの意味が保たれる)。
    /// `None` = 未割当(旧ドキュメントの読み戻し・まだ一度も書かれていない layer)で、
    /// 表示側は既定色(`way_timeline`)へ落ちる。生成時の決定論自動割当は
    /// `motolii_shell` 側(`LayerId % パレット長`)が行う — ここは受け皿のみ。
    /// `#[serde(default)]`: 旧ドキュメントの JSON にこのキーは無いので、無いと
    /// `None` に読み戻す(欠落を `Err` にしない、他の任意フィールドと同じ扱い)。
    #[serde(default)]
    pub label_color: Option<u8>,
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
            solo: false,
            locked: false,
            label_color: None,
        }
    }
}

/// [`crate::Intent::SetAttrs`] が受け取る部分更新。**全フィールド `Option`** —
/// `None` = 「このフィールドは触らない」。
///
/// 2026-08-20 の敵対的レビューが実証した事故: `SetAttrs` が `LayerAttrs` を丸ごと
/// 差し替えていた頃、`SetAttrs{ hidden: true, ..Default::default() }` を書くと
/// name/blend_mode/auto_orient が**黙って既定値へ戻った**。`SetMeta` を新規配置専用に
/// 締めた(裁定108(c))のと同じ事故が、1階層下の `attrs` の中で再生していた。
///
/// `LayerAttrsPatch::default()` は全フィールド `None`(= 無変更)なので、
/// `..Default::default()` を使っても**他のフィールドは型的に戻らない** — struct-update
/// 構文がそのまま安全になる。`write` は `SetTiming`/`SetSource`/`SetOrder` が `meta` に
/// 対して既にやっている read-modify-write を `attrs` にも適用し、`Some` なフィールドだけ
/// 現在値の上へ重ねる。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerAttrsPatch {
    pub hidden: Option<bool>,
    /// 外側の `Option` が「触るか」、内側が「触るなら何にするか」(`None` で親を外す)。
    pub parent: Option<Option<LayerId>>,
    pub blend_mode: Option<BlendMode>,
    pub matte: Option<Option<Matte>>,
    pub name: Option<String>,
    pub auto_orient: Option<bool>,
    pub pinned: Option<bool>,
    pub solo: Option<bool>,
    pub locked: Option<bool>,
    /// 外側の `Option` が「触るか」、内側が「触るなら何にするか」(`None` で
    /// 未割当へ戻す) — `parent` と同じ二重 `Option` の形。
    pub label_color: Option<Option<u8>>,
}

impl LayerAttrsPatch {
    /// `current` へこの patch を重ねる。`Some` なフィールドだけ上書きする。
    pub(crate) fn apply_to(self, mut current: LayerAttrs) -> LayerAttrs {
        if let Some(v) = self.hidden {
            current.hidden = v;
        }
        if let Some(v) = self.parent {
            current.parent = v;
        }
        if let Some(v) = self.blend_mode {
            current.blend_mode = v;
        }
        if let Some(v) = self.matte {
            current.matte = v;
        }
        if let Some(v) = self.name {
            current.name = v;
        }
        if let Some(v) = self.auto_orient {
            current.auto_orient = v;
        }
        if let Some(v) = self.pinned {
            current.pinned = v;
        }
        if let Some(v) = self.solo {
            current.solo = v;
        }
        if let Some(v) = self.locked {
            current.locked = v;
        }
        if let Some(v) = self.label_color {
            current.label_color = v;
        }
        current
    }
}
