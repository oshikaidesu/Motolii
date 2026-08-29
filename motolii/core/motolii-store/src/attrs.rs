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

impl BlendMode {
    /// [`crate::PropertyId::blend_mode`] の `Value::Enum` 表現(裁定214 — 出力に
    /// 現れる property は時間軸に乗る。blend mode は合成に直接効くのでその1つ)。
    /// **補間は Hold**(裁定213 の境界 — `Enum` は変調不可・単一 source が勝つ)。
    /// 添字は保存形の一部になるので、既存の並び([`TextJustify::to_enum_value`]と
    /// 同じ流儀)を変えてはいけない — enum 宣言順そのまま。
    pub fn to_enum_value(self) -> i64 {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Add => 1,
            BlendMode::Multiply => 2,
            BlendMode::Screen => 3,
            BlendMode::Overlay => 4,
            BlendMode::Darken => 5,
            BlendMode::Lighten => 6,
            BlendMode::ColorDodge => 7,
            BlendMode::ColorBurn => 8,
            BlendMode::HardLight => 9,
            BlendMode::SoftLight => 10,
            BlendMode::Difference => 11,
            BlendMode::Exclusion => 12,
            BlendMode::Hue => 13,
            BlendMode::Saturation => 14,
            BlendMode::Color => 15,
            BlendMode::Luminosity => 16,
        }
    }

    /// [`Self::to_enum_value`] の逆写像。未知の値は `None`(壊れた track を近似しない
    /// — 呼び手が `StoreError::Property` へ変える、`TextJustify::from_enum_value` と
    /// 同じ規約)。
    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(BlendMode::Normal),
            1 => Some(BlendMode::Add),
            2 => Some(BlendMode::Multiply),
            3 => Some(BlendMode::Screen),
            4 => Some(BlendMode::Overlay),
            5 => Some(BlendMode::Darken),
            6 => Some(BlendMode::Lighten),
            7 => Some(BlendMode::ColorDodge),
            8 => Some(BlendMode::ColorBurn),
            9 => Some(BlendMode::HardLight),
            10 => Some(BlendMode::SoftLight),
            11 => Some(BlendMode::Difference),
            12 => Some(BlendMode::Exclusion),
            13 => Some(BlendMode::Hue),
            14 => Some(BlendMode::Saturation),
            15 => Some(BlendMode::Color),
            16 => Some(BlendMode::Luminosity),
            _ => None,
        }
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

impl MatteMode {
    /// [`crate::PropertyId::matte_mode`] の `Value::Enum` 表現(裁定214)。**補間は
    /// Hold**(裁定213)。添字は保存形の一部になるので、既存の並びを変えてはいけない。
    pub fn to_enum_value(self) -> i64 {
        match self {
            MatteMode::Alpha => 0,
            MatteMode::InvertedAlpha => 1,
            MatteMode::Luma => 2,
            MatteMode::InvertedLuma => 3,
        }
    }

    /// [`Self::to_enum_value`] の逆写像。未知の値は `None`(壊れた track を近似しない)。
    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(MatteMode::Alpha),
            1 => Some(MatteMode::InvertedAlpha),
            2 => Some(MatteMode::Luma),
            3 => Some(MatteMode::InvertedLuma),
            _ => None,
        }
    }
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
    /// **`LayerSource::Group` の layer にのみ意味を持つ**(裁定119
    /// `docs/reviews/2026-08-20-group-layer-semantics-decision.md` §4)。true の間、
    /// この Group の部分木(子孫)への編集 Intent は理由つき `Err` になる
    /// (`crate::document::check_not_frozen` — `Document::write` 参照)。
    ///
    /// **`LayerAttrsPatch` には frozen フィールドを持たせない** — 汎用の `SetAttrs`
    /// 経由では触れず、専用の [`crate::Intent::Freeze`]/[`crate::Intent::Unfreeze`]
    /// だけが書く(`hidden`/`solo`/`locked` のような汎用属性と違い、freeze は
    /// 意味の重い操作なので専用の口を持たせる — 裁定119 の「明示動詞」の精神、
    /// 裁定174「意図優先の原則」と同じ形)。
    ///
    /// Group でない layer に立っていても(型では防げない — Group 判定は `meta.source`
    /// 側にあるため)描画・書き口のどちらにも一切効かない(`check_not_frozen` は
    /// `meta.source == LayerSource::Group` の祖先だけを見る)。**Document の意味は
    /// 1bit も変わらない**(裁定119 OUTCOME) — 凍結は導出キャッシュの許可証であって
    /// Document データではない。解凍 = この bit を戻すだけで何も失われない。
    /// `#[serde(default)]`: 旧ドキュメントに無いキーは `false`(未凍結)に読み戻す。
    #[serde(default)]
    pub frozen: bool,
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
            frozen: false,
        }
    }
}

/// **裁定214(2026-08-23)**: solo は「Inspector に映る物」の一種として時間軸に
/// 乗るべき(`Value::Bool` は補間 Hold、キーフレームは打てる — 裁定213 が確認した
/// 一般則)。**配線は完了**(A03副監督A発注、同日): [`crate::view::StoreView::
/// resolve_with_solo`]/`any_solo`(`view.rs`)がこの track を `value_at` 経由で
/// 読み、track が無ければ静的 `LayerAttrs::solo` が既定値になる
/// (`resolved_masks`/`resolved_text_document` と同じ overlay の形、裁定20)。
/// Inspector 側の Key 列(書き口を `Intent::SetTrack` へ切り替える枝)はまだ無い
/// (裁定212 の入口ゼロ問題と同型、本発注の write-set 外)。
impl crate::PropertyId {
    pub fn solo() -> Self {
        Self::new("solo").expect("`solo` は予約語でも空でもない")
    }

    /// **裁定214**: hidden も同じ理由で時間軸に乗る。`text_justify`/`solo` と同じ
    /// 「素の property 名」の形(prefix 無し、layer 自身が名前空間)。**配線済み**
    /// (`resolve_with_solo` が `value_at` 経由で読む、track 無しは静的
    /// `LayerAttrs::hidden` が既定値)。
    pub fn hidden() -> Self {
        Self::new("hidden").expect("`hidden` は予約語でも空でもない")
    }

    /// **裁定214**: blend mode も出力に直接効くので時間軸に乗る。`Value::Enum`
    /// (`BlendMode::to_enum_value`/`from_enum_value`)、補間は Hold(裁定213)。
    /// **配線済み**(`resolve_with_solo` が読む、track 無しは静的
    /// `LayerAttrs::blend_mode` が既定値)。
    pub fn blend_mode() -> Self {
        Self::new("blend_mode").expect("`blend_mode` は予約語でも空でもない")
    }

    /// **裁定214**: matte の重ね方(mode)も時間軸に乗る。**`Matte.layer`(参照先)は
    /// 対象外**(裁定214 修正版「LayerId は Hold でキーフレーム自体は打てるが実装は
    /// 後回し」— A03副監督A発注が明示的に後回しにした枝)。`matte` 自体が `None`
    /// (このレイヤはマットにされていない)なら track があっても上書き先が無いので
    /// 効かない — mode だけを time-vary させる話であって、track 単独で matte を
    /// 新設することはできない(`resolve_with_solo` 参照)。`Value::Enum`、補間は
    /// Hold(裁定213)。
    pub fn matte_mode() -> Self {
        Self::new("matte_mode").expect("`matte_mode` は予約語でも空でもない")
    }
}

/// **裁定214**: Speed(`crate::Speed`、`LayerTiming::speed`)も同じ理由で時間軸に
/// 乗るべき対象(A03 棚卸し行「Speed(ATTRS)」)。この `PropertyId` も器だけ。
///
/// **配線は未完**: Speed の書き口は `motolii-shell::Shell::apply_speed`
/// (`Intent::SetTiming` の read-modify-write + duration 再計算)にあり、
/// `next/shell/motolii-shell/src/lib.rs` は本発注で明示的に「触らない」と
/// 指定されたファイル。そこを `Intent::SetTrack` 経由へ切り替えない限り、
/// この track を Inspector が書いても再生速度は変わらない。
impl crate::PropertyId {
    pub fn speed() -> Self {
        Self::new("speed").expect("`speed` は予約語でも空でもない")
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
