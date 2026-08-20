//! owns: Document の意味(layer の同一性・素材の指紋・comp 時刻での解決)。
//!
//! **`wraps:` ではない**。当初 `wraps: re_entity_db::EntityDb` と名乗っていたが、
//! 敵対的レビュー(2026-08-20)で「`fingerprint.rs` と `resolve`/`ResolvedLayer` は
//! 上流に無い物 = `owns:` の中身」と指摘され、訂正した。**marker は crate の根しか
//! 見ないので、`wraps:` を名乗った crate の中に `owns:` の中身が入ると規律が空振りする**。
//!
//! 上流に**寄せている**もの(ここで再実装していないもの):
//!
//! - 保存と検索: `re_entity_db::EntityDb` / `re_chunk_store`
//! - **undo / redo は `edit` timeline の latest-at 移動そのもの**で、自前の履歴機構を
//!   持たない(rerun blueprint の undo と同じ機構。R0-2 で1000編集跨ぎを実測)
//! 「新しい編集をする前に redo 空間を落とす」も rerun の規則をそのまま踏襲する。
//!
//! ここに書いてよいのは「store の口をどう開けるか」だけである。時刻→値の意味は
//! `motolii-eval`(移植した正本)が持ち、この crate は評価を呼ぶだけで再実装しない。
//!
//! 設計上の柵:
//! - 読み手が受け取るのは [`StoreView`] だけで、可変ハンドルは外へ出ない
//! - 書き口は [`Document::apply`] 1本だけ
//! - **削除も append**(tombstone)。`drop_entity_path` を使うと undo で戻せなくなる

mod attrs;
mod components;
mod document;
mod effect;
mod fingerprint;
mod marker;
mod mask;
mod persist;
mod text;
mod view;

pub use attrs::{BlendMode, LayerAttrs, LayerAttrsPatch, Matte, MatteMode};
pub use document::{Document, Intent, LayerId, PropertyId, Revision};
pub use effect::{EffectId, EffectInstance};
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use marker::Marker;
pub use mask::{Mask, MaskId, MaskMode, ResolvedMask};
pub use text::{
    ContentKeyframe, ContentTrack, FontRef, TextDocument, TextDocumentStyle, TextJustify,
};
pub use view::StoreView;

pub use motolii_core::{CompSpec, Fps, LayerPlacement, RationalTime, ResolvedCamera};
pub use motolii_eval::{Interp, Keyframe, KeyframeTrack, Path, PathVertex, Value};
/// shape-layer(`layers/shape-layer/shapes`)の中身。語彙の正本は `motolii-vector`
/// (shape-1/2/3 が既に決めた)— ここでは作り直さない(裁定10)。`Path` は再輸出しない
/// (`motolii_eval::Path` と名前が衝突する — マスク形状の `Value::Path` が正本のまま)。
pub use motolii_vector::{PathSource, Point as VectorPoint, Shape};

/// `edit` timeline の名前。undo/redo はこの軸の移動である。
pub const EDIT_TIMELINE: &str = "edit";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("chunk の組み立てに失敗した: {0}")]
    Chunk(String),
    #[error("store への追加に失敗した: {0}")]
    Ingest(String),
    #[error("track の符号化に失敗した: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("property 名が不正: {0}")]
    Property(String),
    #[error("file の読み書きに失敗した: {0}")]
    Io(String),
}

/// 標準 property の名前。**ここに無い名前も置けるが、標準面はこれを見る**。
pub mod property {
    /// component 識別子は `Layer:{name}` なので、**layer 自身の component と衝突する
    /// 名前は禁止**(`PropertyId::new` が弾く)。弾かないと `PropertyId::new("meta")` が
    /// layer の素材と重ね順を上書きする。`attrs`/`effects`/`shapes`/`text` は
    /// layer-meta 束が足した component(裁定108(c) の構造修正)。
    pub const RESERVED: &[&str] = &[
        "meta", "present", "masks", "attrs", "effects", "shapes", "text",
    ];

    /// マスクの形状・不透明度トラックの名前は `mask.{id}.…` で始まる。
    /// **平坦な名前**にしてあるので、新しい機構を足さずに `KeyframeTrack` へ乗る
    /// (裁定92 が text.style で先に見つけた形と同じ)。
    pub const MASK_PREFIX: &str = "mask.";

    /// 変換の中心。**レイヤ自身の座標単位の点**であって 0..1 の正規化ピボットではない。
    pub const ANCHOR: &str = "anchor";
    /// **anchor が着地する点**。`top_left` ではない(裁定60)。
    pub const POSITION: &str = "position";
    /// split(x/y 別 track)の X 側。`position` 本体が無い時だけ読む(裁定61: 別 track が
    /// 既定なのではなく、後から選べる variant)。
    pub const POSITION_X: &str = "position.x";
    /// 同 Y 側。
    pub const POSITION_Y: &str = "position.y";
    /// 1.0 が等倍(Lottie のパーセントは採らない、裁定58)。
    pub const SCALE: &str = "scale";
    /// 度・時計回り(AE と同じ。ラジアンは人が読めない)。
    pub const ROTATION: &str = "rotation";
    pub const OPACITY: &str = "opacity";
    /// skew の量(度)。`LayerPlacement::from_transform` の穴だった箇所(裁定69)。
    pub const SKEW: &str = "skew";
    /// skew の軸(度)。0 なら x 軸、90 なら y 軸に沿った点が不動点になる。
    pub const SKEW_AXIS: &str = "skew_axis";
    /// `layers/audio-settings/lv`(Level)。clip の音量(gain)。1.0 が等倍。
    /// GOALS 標準。専用の component にしない — 普通の property track で十分
    /// (裁定20「キーを打っていない property は静止値」がそのまま効く)。
    pub const LEVEL: &str = "level";
    /// `layers/precomposition-layer/tm`(Time Remap)。値がそのまま**素材のフレーム番号**
    /// (comp のフレームではない)。timing に混ぜない(裁定65 が `tm` を落とした理由の
    /// 裏返し — Time Remap は timing ではなく property)。track が無ければ通常どおり
    /// `LayerTiming::source_frame` の写像を使う。
    pub const TIME_REMAP: &str = "time_remap";
    /// layer の奥行き(裁定113/116)。**既定 0**(全員 z=0)。`position.x`/`position.y`
    /// (split-position 束、裁定111(b))の隣に同じ流儀で置く。単位は `position` と同じ
    /// world = ピクセル。AE と同じ符号(大きいほどカメラから遠い)。
    pub const POSITION_Z: &str = "position.z";

    /// カメラの property(裁定113/115、裁定116 で実装)。`layer` ではなく `/composition`
    /// entity へ書く(`PropertyId::camera` が別の component 名前空間を作る)。
    /// comp 中心からのパン量(ピクセル)。既定 [0,0](パン無し)。
    pub const CAMERA_CENTER: &str = "camera.center";
    /// 既定 1.0(zoom 無し)。
    pub const CAMERA_ZOOM: &str = "camera.zoom";
    /// 度・時計回り。既定 0.0(roll 無し)。
    pub const CAMERA_ROLL: &str = "camera.roll";
}

/// layer の素材。media が入るまでは単色だけ。
///
/// **variant を足すのが素材種を増やす唯一の道**にしてある(動画・静止画・生成物が
/// 別々の経路を持たないようにするため)。
///
/// `Null` / `Shape` / `Text` に中身のフィールドを持たせていないのは、この enum が
/// `Eq + Hash`(engine の texture cache のキー)である必要があるため — 図形の頂点や
/// テキストの内容は `f64` を含み `Eq` になれない。中身は layer 自身の別 component
/// (`Layer:shapes` / `Layer:text`)が持ち、ここは「この層の素材はどの種類か」の印だけ
/// (mask を `meta` の外に置いたのと同じ理由、裁定108(a))。
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LayerSource {
    Solid {
        rgba: [u8; 4],
        width: u32,
        height: u32,
    },
    /// 実素材。**動画も静止画も同じ variant**を通す — 経路を分けると、
    /// 片方だけ直る欠陥が生まれる(初回タッチ観察の再発防止)。
    ///
    /// 大きさは probe が決めるので Document は持たない。`fingerprint` はパスが
    /// 動いても同じ物だと言えるようにするための内容識別で、無くても描ける。
    Media {
        path: String,
        fingerprint: Option<String>,
    },
    /// 絵を持たず transform だけ持つ(AE の Null Object)。親子の受け皿
    /// (`layers/null-layer/ty`、layer-meta 束)。
    Null,
    /// ベクタ生成物(`layers/shape-layer/ty`)。中身(パス源+演算子スタック+fill/stroke)は
    /// `Layer:shapes` component が `Vec<motolii_vector::Shape>` として持つ
    /// (`layers/shape-layer/shapes`)。語彙の正本は `motolii-vector`(shape-1/2/3 が既に決めた)
    /// — ここで作り直さない(裁定10)。
    Shape,
    /// テキスト生成物(`layers/text-layer/ty`)。中身は `Layer:text` component。
    /// **今は素の文字列1本だけ**(`layers/text-layer/t`)— 範囲スタイル・アニメーターの
    /// 語彙(裁定82/85 等)は `text` 発注単位(75行)の仕事で、ここでは作らない。
    Text,
}

impl LayerSource {
    /// Document が知っている大きさ。実素材は probe しないと分からないので `None`。
    /// `Null`/`Shape`/`Text` も `None` — 寸法は素材ではなく中身(演算子・組版)が決める。
    pub fn declared_size(&self) -> Option<[f32; 2]> {
        match self {
            Self::Solid { width, height, .. } => Some([*width as f32, *height as f32]),
            Self::Media { .. } | Self::Null | Self::Shape | Self::Text => None,
        }
    }
}

/// comp の設定。**Document が持つ**。
///
/// ここに置く理由(2026-08-20 の敵対的レビュー): 以前は `render_frame(view, t, comp)` と
/// `ExportJob { comp, fps }` が別々に持っていたので、**preview と export が違う入力を
/// 渡せた**。「評価経路が1本」は入力が同じ時だけの保証であり、その入力の正本が
/// どこにも無かった。
///
/// 上流の `EntityDb::set_recording_property` は `TimePoint::STATIC` で書くので
/// **undo が効かない**。解像度や fps の変更は戻せるべきなので、layer と同じく
/// `edit` timeline 上の普通の entity として置く(新しい機構を足さない)。
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Composition {
    pub width: u32,
    pub height: u32,
    pub fps: motolii_core::Fps,
    /// 尺(フレーム数)。半開 `[0, duration_frames)`。
    pub duration_frames: i64,
}

impl Composition {
    pub fn spec(&self) -> motolii_core::CompSpec {
        motolii_core::CompSpec {
            width: self.width,
            height: self.height,
        }
    }
}

/// layer が comp 上のどこに、素材のどこから乗るか。
///
/// **これが無いと「常に存在し、素材フレーム = comp フレーム」しか表現できない** —
/// 配置も trim も split も速度も、全部この型の上に乗る。
///
/// 上流に相当物は無い。rerun の `AbsoluteTimeRange` は store の時間範囲であって
/// 「素材のどこを使うか」を持たないので、これは Motolii の意味である。
///
/// 単位はフレーム。comp の fps で数える(`RationalTime` を持たないのは、
/// 配置が fps に紐づく整数だからで、時刻へ写す時は正準口を通る)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerTiming {
    /// comp 上の開始フレーム。
    pub start: i64,
    /// 尺(フレーム数)。半開 `[start, start + duration)`。
    pub duration: i64,
    /// 素材の何フレーム目から使うか。
    pub source_in: i64,
    /// `layers/layer/sr`(Time Stretch)。comp が1フレーム進む間に素材が何フレーム
    /// 進むかの比。`Speed::NORMAL`(1/1)が等速。**穴は裁定63 で空けてあった** —
    /// 元の `source_in + (comp_frame - start)` は 1:1 固定しか表せなかった。
    pub speed: Speed,
}

impl Default for LayerTiming {
    fn default() -> Self {
        Self {
            start: 0,
            // 0 は「まだ決まっていない」ではなく「尺ゼロ」なので、既定は置かない。
            // `LayerMeta::new` が素材の実尺から埋める。
            duration: 0,
            source_in: 0,
            speed: Speed::NORMAL,
        }
    }
}

impl LayerTiming {
    /// この comp フレームで layer は居るか。
    pub fn covers(&self, comp_frame: i64) -> bool {
        comp_frame >= self.start && comp_frame < self.start + self.duration
    }

    /// 素材を置く時の尺 = **min(素材の尺, comp の残り)**(M4)。
    ///
    /// 素材の尺が分からない場合(静止画など)は comp の残り全部。
    /// **この規則を shell に書かせない** — 書かせると面ごとに違う置き方が生まれる。
    pub fn place(start: i64, source_frames: Option<i64>, comp_duration: i64) -> Self {
        let remaining = (comp_duration - start).max(0);
        let duration = match source_frames {
            Some(frames) => frames.min(remaining),
            None => remaining,
        };
        Self {
            start,
            duration,
            source_in: 0,
            speed: Speed::NORMAL,
        }
    }

    /// comp フレーム → 素材のフレーム。居ない時刻なら `None`。
    ///
    /// **素材の終端でフリーズさせない**(M4)。居ない時刻は描かない。
    /// `speed` が等速でない場合、進み幅を比でスケールする(裁定63)。
    ///
    /// **未クランプ**(2026-08-20 の敵対的レビューが指摘): `speed` が負(逆再生)で
    /// `source_in` が小さいと、返る値が負になりうる — `source_in=0` の逆速度 layer は
    /// `covers` している間ずっと負の素材フレームを返す。store はここで 0 へ丸めない
    /// (`0` は「素材の先頭」という意味のある値なので、それとの区別が付かなくなる方が
    /// 危険)。**負の `source_frame` をどう扱うか(0 でクランプ / ループ / エラー)は
    /// engine 側の判断で、この store の仕事ではない** — ここでは実測どおりの数値を
    /// そのまま返す。
    pub fn source_frame(&self, comp_frame: i64) -> Option<i64> {
        self.covers(comp_frame).then(|| {
            let offset = comp_frame - self.start;
            self.source_in + self.speed.scale_frame_offset(offset)
        })
    }
}

/// `sr`(Time Stretch)。**比率であって時刻ではない** — `motolii-core::RationalTime` を
/// 再利用しない(あちらは秒の正本で、意味を混ぜると「速度なのか時刻なのか」が
/// 型から読めなくなる)。f64 を経由しないので TM-4 の柵(`fps` と `f64` が同じ式に
/// 出ないこと)の対象外のまま保てる — この型はそもそも `fps` を参照しない。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Speed {
    num: i64,
    den: i64,
}

impl Speed {
    pub const NORMAL: Speed = Speed { num: 1, den: 1 };

    /// `den` は正でなければならない(符号は `num` が持つ — 負の速度 = 逆再生)。
    pub fn try_new(num: i64, den: i64) -> Result<Self, StoreError> {
        if den <= 0 {
            return Err(StoreError::Property(
                "speed の分母は正でなければならない".to_owned(),
            ));
        }
        Ok(Self { num, den })
    }

    pub const fn num(self) -> i64 {
        self.num
    }

    pub const fn den(self) -> i64 {
        self.den
    }

    /// comp 上のオフセット(フレーム)を、この速度で素材側のオフセットへ写す(床関数)。
    /// `den > 0` は構築時に保証済みなので `div_euclid` がそのまま床除算になる
    /// (逆速度で `offset` が実質負になる場合も含めて)。
    fn scale_frame_offset(self, offset: i64) -> i64 {
        let num = offset as i128 * self.num as i128;
        let den = self.den as i128;
        (num.div_euclid(den)) as i64
    }
}

/// layer の非アニメーション属性。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerMeta {
    pub source: LayerSource,
    /// 大きいほど手前。上流の `re_renderer::DepthOffset` と同じ `i16`。
    pub order: i16,
    /// comp 上の配置と、素材のどこを使うか。
    pub timing: LayerTiming,
}

/// ある comp 時刻に解決済みの layer。**合成器が要るのはこれだけ**。
///
/// 置き方は `motolii-core::LayerPlacement` を**そのまま持つ**(フィールドを並べ直さない)。
/// 並べ直すと、property を1つ足すたびに store と合成器の両方を触ることになり、
/// それが翻訳層の始まりになる。
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLayer {
    pub source: LayerSource,
    pub placement: LayerPlacement,
    /// Document が知っている素材の寸法。`[0,0]` = **probe しないと分からない**ので
    /// engine が実寸で埋める。
    pub declared_size: [f32; 2],
    /// この comp 時刻に対応する**素材のフレーム**。
    /// 解決済みなので、engine はもう時間の計算をしない。
    pub source_frame: i64,
    /// この時刻のマスク。**スタックの順**(手前のマスクへ畳んでいく順)で並ぶ。
    ///
    /// ここに置くのは、`ResolvedLayer` が「この時刻のこの layer の姿」の全部だからである。
    /// 別の口にすると `ResolvedLayer` から `LayerId` が引けず、描く側がマスクへ辿り着けない。
    pub masks: Vec<ResolvedMask>,
    /// `layers/visual-layer/bm`。**まだ合成器は読んでいない**(`motolii-compositor` は
    /// `re_renderer` の `multiplicative_tint` しか使っておらず、blend mode の合成式は
    /// 未実装)。Document 側の意味はここで解決済みにしておき、engine が繋ぐ日を待つ。
    pub blend_mode: BlendMode,
    /// matte(裁定66)。**同上、まだ合成器は読んでいない**。
    pub matte: Option<Matte>,
    /// `LayerAttrs::pinned`(裁定113)。true ならカメラ変換を受けず画面に張り付く —
    /// 合成器が `placement.transform`/`z` をカメラで動かす前に打ち消す(裁定116 実装)。
    pub pinned: bool,
}
