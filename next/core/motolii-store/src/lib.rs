//! owns: Document の意味(layer の同一性・素材の指紋・comp 時刻での解決)。
//!
//! OWNS-JUSTIFICATION(A): 意見1(`next/reference/OPINIONS.md` #1・有理時間)が強制する。
//!       rerun の store は `TimeInt`(整数)前提で `30000/1001` を表せないため、
//!       有理時間を採った時点で上流 store をそのまま使う道が消えた — これが
//!       約19,000行の自前実装の出所(意見1のコスト列に記載)。併せて意見3
//!       (`Intent` が唯一の書き口)と意見12(undo は `edit` timeline の時間移動)も
//!       この crate に住む。**上流不在の裏取りも別途済み**: 下記の通り
//!       `fingerprint.rs` と `resolve`/`ResolvedLayer` は敵対的レビュー(2026-08-20)で
//!       「上流に無い物」と確認されている。
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

mod asset;
mod attrs;
mod components;
mod document;
mod effect;
mod fingerprint;
mod marker;
mod mask;
mod persist;
mod slot;
mod text;
mod view;

pub use asset::{Asset, AssetDraft, AssetError, AssetId, AssetStatus, AssetTable};
pub use attrs::{BlendMode, LayerAttrs, LayerAttrsPatch, Matte, MatteMode};
pub use document::{DisplayRevision, Document, Intent, LayerId, PropertyId, Revision};
pub use effect::{EffectId, EffectInstance, ResolvedEffect};
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use marker::Marker;
pub use mask::{Mask, MaskId, MaskMode, ResolvedMask};
pub use persist::AutoSaveConfig;
pub use slot::{PropertyBase, PropertyLink, PropertySource, Slot, SlotId};
pub use text::{
    ContentKeyframe, ContentTrack, FontRef, TextAlignmentOptions, TextBasedOn, TextDocument,
    TextDocumentStyle, TextGrouping, TextJustify, TextRandomize, TextRange, TextRangeId,
    TextRangeSelector, TextRangeUnits, TextRun, TextShape, TextStyleAxis, TextStyleFeature,
    TextStyleId, TextVariationAxis,
};
pub use view::StoreView;

pub use motolii_core::{CompSpec, Fps, LayerPlacement, RationalTime, ResolvedCamera};
pub use motolii_eval::{Interp, Keyframe, KeyframeTrack, Path, PathVertex, SpatialTangent, Value};
/// shape-layer(`layers/shape-layer/shapes`)の中身。語彙の正本は `motolii-vector`
/// (shape-1/2/3 が既に決めた)— ここでは作り直さない(裁定10)。`Path` は再輸出しない
/// (`motolii_eval::Path` と名前が衝突する — マスク形状の `Value::Path` が正本のまま)。
///
/// `ShapeNode`/`ShapeGroup`/`RepeaterTransform` は裁定173 H4(シェイプ内階層)で
/// `Layer:shapes` の中身が `Vec<Shape>` から `Vec<ShapeNode>` へ広がった分。
/// `Shape` 自体はそのまま(旧 flat JSON の着地点 = `ShapeNode::Leaf`)。
pub use motolii_vector::{
    PathSource, Point as VectorPoint, RepeaterTransform, Shape, ShapeGroup, ShapeNode,
};

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

    /// テキストアニメーター([`crate::TextRangeId`])の selector/style/transform/variation
    /// トラックの名前は `text_range.{id}.…` で始まる。マスクと同じ平坦な流儀
    /// (裁定92 が text.style で先に見つけた形と同じ)。
    pub const TEXT_RANGE_PREFIX: &str = "text_range.";

    /// スタイル表の行([`crate::TextStyleId`])が持つ動く量(今のところ可変フォント軸
    /// の絶対値だけ、裁定92の唯一の例外)の名前は `text_style.{id}.…` で始まる。
    pub const TEXT_STYLE_PREFIX: &str = "text_style.";

    /// effect インスタンス([`crate::EffectId`])の param トラックの名前は
    /// `effect.{id}.param.{name}` で始まる。マスク/テキストと同じ平坦な流儀
    /// (裁定92 が先に見つけた形と同じ、裁定72「新機構ゼロ」)。
    pub const EFFECT_PREFIX: &str = "effect.";

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
    /// ステレオ pan。-1.0 が全振り左、0.0 が中央(既定)、1.0 が全振り右。
    /// **Lottie 圏外**——`lottie-coverage.tsv` の `layers/audio-settings` 束は `lv`
    /// (Level)1行だけで、pan に対応する語彙が無い(Lottie は音声の空間配置を
    /// 持たない仕様)。GOALS を見ても他エディタの標準機能なので、`LEVEL` と同じ
    /// 「専用 component を作らず普通の property track に乗せる」形をそのまま踏襲する
    /// (裁定20 がここでも効く——track が無ければ中央 0.0)。
    pub const PAN: &str = "pan";
    /// clip 先頭からのフェードイン尺(秒、`RationalTime` ではなく `f64` — 他の
    /// property と同じ生の数値。`KeyframeTrack` に乗せられる型に合わせた)。
    /// 0.0(既定・無効)ならフェードなし。**Lottie 圏外・motolii 独自**——`lv` 以外の
    /// audio-settings 語彙が地図に無いので、発明ではなく Lottie が扱わない領域を
    /// 埋める必然(音声は Lottie の対象外、`docs` 側の音声整形要求から逆算した
    /// GOALS 標準)。engine 側の適用は AUD レーンの仕事——ここは値の置き場だけ。
    pub const FADE_IN: &str = "fade_in";
    /// clip 末尾までのフェードアウト尺(秒)。0.0(既定・無効)ならフェードなし。
    /// [`FADE_IN`] と対称、同じ理由で Lottie 圏外・motolii 独自。
    pub const FADE_OUT: &str = "fade_out";
    /// `layers/precomposition-layer/tm`(Time Remap)。値がそのまま**素材のフレーム番号**
    /// (comp のフレームではない)。timing に混ぜない(裁定65 が `tm` を落とした理由の
    /// 裏返し — Time Remap は timing ではなく property)。track が無ければ通常どおり
    /// `LayerTiming::source_frame` の写像を使う。
    pub const TIME_REMAP: &str = "time_remap";
    /// `layers/layer/sr`(Time Stretch)の track 版。裁定63 が空けた穴 —
    /// `LayerTiming.speed` は静的な1点(`Intent::SetTiming` の read-modify-write)
    /// しか持てないので、時間で変わる速度は別 property track として持つ。
    /// track が無ければ従来どおり `LayerTiming.speed`(静的値)を使う。`TIME_REMAP`
    /// と同時に存在しても構わないが **`TIME_REMAP` が勝つ**(`resolve_with_solo` の
    /// 適用順そのまま — remap は「素材フレーム番号を直接指定する」より強い意味
    /// なので、速度の積算より後に上書きする現行順を変えない)。
    pub const SPEED: &str = "speed";
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
    /// ベクタ生成物(`layers/shape-layer/ty`)。中身(パス源+演算子スタック+fill/stroke、
    /// 裁定173 H4 で入れ子グループも)は `Layer:shapes` component が
    /// `Vec<motolii_vector::ShapeNode>` として持つ(`layers/shape-layer/shapes`)。
    /// 語彙の正本は `motolii-vector`(shape-1/2/3 が既に決めた)— ここで作り直さない
    /// (裁定10)。`ShapeNode` は shape 粒度の入れ子で、layer 粒度の [`LayerSource::Group`]
    /// (この enum の別 variant)とは別概念 — 旧世界 `VectorContent::Group` が
    /// タイムライン `TrackItem::Group` と意図的に別概念だったのと同じ区別
    /// (`docs/reviews/2026-08-22-transform-hierarchy-seam-survey.md` §3.1)。
    Shape,
    /// テキスト生成物(`layers/text-layer/ty`)。中身は `Layer:text` component。
    /// **今は素の文字列1本だけ**(`layers/text-layer/t`)— 範囲スタイル・アニメーターの
    /// 語彙(裁定82/85 等)は `text` 発注単位(75行)の仕事で、ここでは作らない。
    Text,
    /// **グループの印**(裁定173 (c))。Group は「子を持てる」という印だけを持つ特殊な
    /// layer で、絵を持たない(`Null` と同じく engine は texture を焼かない —
    /// `motolii-engine::texture_for` 参照)。
    ///
    /// **member 列を持たせない**(裁定173 §4.4 の二重帳簿回避)。所属は既存の
    /// `LayerAttrs.parent: Option<LayerId>` 1本槍で表現する — ある layer が
    /// このGroupの子かどうかは `attrs(child).parent == Some(group_id)` で毎回導出する
    /// (正本は常に子側の1フィールド)。旧世界(`crates/motolii-doc::TrackItem::Group`)が
    /// 持っていた `children: Vec<TrackItem>` に相当する列はここには**存在しない** —
    /// 将来これを足すと「Group.members に入っている」と「子の parent が Group を指す」
    /// という2つの正本が同時に生まれる(H-survey §4.4)。
    Group,
}

impl LayerSource {
    /// Document が知っている大きさ。実素材は probe しないと分からないので `None`。
    /// `Null`/`Shape`/`Text`/`Group` も `None` — 寸法は素材ではなく中身(演算子・組版)
    /// が決める(`Group` は中身を子 layer が持つので、そもそも寸法という概念を持たない)。
    pub fn declared_size(&self) -> Option<[f32; 2]> {
        match self {
            Self::Solid { width, height, .. } => Some([*width as f32, *height as f32]),
            Self::Media { .. } | Self::Null | Self::Shape | Self::Text | Self::Group => None,
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
    /// comp の背景色(RGBA、0.0〜1.0)。**静的値でキーフレーム化しない** — 動く背景は
    /// 層でやる(裁定20 の精神。KeyframeTrack にすると「層と comp の2箇所に動く値の
    /// 経路ができる」という同じ問題を繰り返す)。
    ///
    /// 既定は不透明黒(現行の見た目のまま)。`#[serde(default)]` は**新規保存だけの
    /// ためではない** — 既存の保存ファイルにはこの component が無いので、読み込み時に
    /// 必ずこの既定を通る(persist の後方互換)。
    #[serde(default = "Composition::default_background")]
    pub background: [f32; 4],
}

impl Composition {
    /// [`Composition::background`] の既定値。旧描画(合成器の clear 色)と同じ見た目に
    /// なる不透明黒 — 利用者が明示的に変えるまで絵は変わらない。
    pub fn default_background() -> [f32; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

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

    /// [`Self::source_frame`] の speed track 版(A03「Speed(ATTRS)」、裁定63 の穴)。
    ///
    /// **`self.speed`(静的値)ではなく積算で決める** — `source_frame` は comp
    /// フレームだけの純粋な関数だが、speed が時間で変わる時は「start からここまでの
    /// speed の積算」でなければ正しくない(1点の `value_at` 上書きでは足りない、
    /// 発注文書の指摘どおり)。
    ///
    /// **O(K)**(K = track のキー数)であって **O(comp_frame) ではない** — 毎フレーム
    /// 積算を1フレームずつ足すと `resolve` が呼ばれる回数($export$ なら総フレーム数)
    /// だけ効いて O(N²) の性能事故になる(裁定140 が潰した `track()` の
    /// serde_json 解析コストと同じ形の事故を積算側で再現しないための設計)。
    /// 区間ごとの閉形式(Hold=定数×区間長、Linear=等差級数の和)で解くことで、
    /// 区間内のフレーム数によらずキー数だけに比例させる。
    ///
    /// `Interp::Bezier` の区間をまたぐ場合は明示的に `Err` を返す — 三次ベジェの
    /// 積分は閉形式で解けるが、この発注の検収条件(Hold のみ)を超えて黙って
    /// 近似する判断をここでは下さない(裁定218「根拠を示してからテスト」の裏
    /// — 根拠を示せない近似は実装しない)。
    pub fn source_frame_with_speed_track(
        &self,
        comp_frame: i64,
        track: &crate::KeyframeTrack,
        fps: motolii_core::Fps,
    ) -> Result<Option<i64>, StoreError> {
        if !self.covers(comp_frame) {
            return Ok(None);
        }
        let accumulated = accumulate_speed_offset(track, fps, self.start, comp_frame)?;
        Ok(Some(self.source_in + accumulated))
    }
}

/// [`LayerTiming::source_frame_with_speed_track`] 本体。`track` の値(F64、比率)を
/// comp フレーム `start..comp_frame` の範囲で積算し、素材側の進み幅(フレーム、
/// 床関数)を返す。
///
/// `motolii_eval::KeyframeTrack::eval` と同じ端の規約を使う — 最初のキーより前は
/// `keys[0].value` で一定、最後のキーより後は `keys[last].value` で一定
/// (`KeyframeTrack::eval` のクランプと同じ)。
fn accumulate_speed_offset(
    track: &crate::KeyframeTrack,
    fps: motolii_core::Fps,
    start: i64,
    comp_frame: i64,
) -> Result<i64, StoreError> {
    use crate::{Interp, RationalTime};

    if comp_frame <= start {
        return Ok(0);
    }
    let keys = track.keys();
    if keys.is_empty() {
        // `KeyframeTrack::eval` はキー無しを F64(0.0) 一定として扱う — ここも
        // 同じ規約(速度0 = 素材が一切進まない)。
        return Ok(0);
    }

    let frame_time = |f: i64| -> Result<RationalTime, StoreError> {
        RationalTime::try_from_frame(f, fps).map_err(|e| StoreError::Property(e.to_string()))
    };
    // `t` 以上になる最小の整数フレーム番号(`try_to_frame_floor` の上向き版 —
    // 正準口には ceil が無いので、floor を作ってから復元値と比べて直す)。
    let ceil_frame = |t: RationalTime| -> Result<i64, StoreError> {
        let f = t
            .try_to_frame_floor(fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let recon = frame_time(f)?;
        Ok(if recon < t { f + 1 } else { f })
    };
    let value_f64 = |v: &crate::Value| -> Result<f64, StoreError> {
        match v {
            crate::Value::F64(x) => Ok(*x),
            other => Err(StoreError::Property(format!(
                "{} に数値でない値が入っている: {other:?}",
                crate::property::SPEED
            ))),
        }
    };

    let mut total = 0.0f64;
    let n = keys.len();

    // 区間 [lo, hi) に定数 v を積む(Hold、および最初/最後のキーの外側)。
    let mut add_hold = |lo: i64, hi: i64, v: f64| {
        let lo = lo.max(start);
        let hi = hi.min(comp_frame);
        if hi > lo {
            total += v * (hi - lo) as f64;
        }
    };

    // 先頭キーより前: 一定 keys[0].value。
    {
        let hi = ceil_frame(keys[0].t)?;
        add_hold(start, hi, value_f64(&keys[0].value)?);
    }
    // 末尾キー以降: 一定 keys[last].value。
    {
        let lo = ceil_frame(keys[n - 1].t)?;
        add_hold(lo, comp_frame, value_f64(&keys[n - 1].value)?);
    }
    // キー間の各区間。
    for i in 0..n.saturating_sub(1) {
        let (a, b) = (&keys[i], &keys[i + 1]);
        let lo = ceil_frame(a.t)?.max(start);
        let hi = ceil_frame(b.t)?.min(comp_frame);
        if hi <= lo {
            continue;
        }
        match a.interp {
            Interp::Hold => {
                total += value_f64(&a.value)? * (hi - lo) as f64;
            }
            Interp::Linear => {
                let va = value_f64(&a.value)?;
                let vb = value_f64(&b.value)?;
                // u(f) = この区間内での正規化位置(0..1)。両端は線形なので、
                // 区間内の整数フレーム点での u は等差数列 — 平均 × 個数で
                // 和が閉形式に求まる(1フレームずつ足さない、O(1))。
                let u = |f: i64| -> Result<f64, StoreError> {
                    let num = seconds_since(frame_time(f)?, a.t);
                    let den = seconds_since(b.t, a.t);
                    Ok(if den == 0.0 { 0.0 } else { num / den })
                };
                let u_lo = u(lo)?;
                let u_hi_last = u(hi - 1)?;
                let count = (hi - lo) as f64;
                let sum_u = count * (u_lo + u_hi_last) / 2.0;
                total += va * count + (vb - va) * sum_u;
            }
            Interp::Bezier { .. } => {
                return Err(StoreError::Property(format!(
                    "{} の Bezier 補間区間は積算未対応(発注の検収条件は Hold のみ、\
                     黙って近似しない — 対応するなら別発注で判断すること)",
                    crate::property::SPEED
                )));
            }
        }
    }

    Ok(total.floor() as i64)
}

/// `t - origin` の秒(`motolii_eval::track` の同名の内部関数と同じ規約 —
/// 差分が有理数として厳密に取れれば厳密経路、溢れ時は f64 秒差へフォールバック)。
fn seconds_since(t: motolii_core::RationalTime, origin: motolii_core::RationalTime) -> f64 {
    match t.try_sub(origin) {
        Ok(rel) => rel.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - origin.as_seconds_f64(),
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
    /// この layer 自身の同一性。**BL4(track matte)で新設**——`matte: Some(Matte {
    /// layer, .. })` が指す先は `LayerId` なので、`resolved_layers()` が返す
    /// `Vec<ResolvedLayer>` の中から「マット元の layer」を引くには、各要素が
    /// 自分の `LayerId` を持っている必要がある(そうでないと `resolve_with_solo` を
    /// もう一度 `LayerId` 付きで呼び直す=同じ layer を二重に resolve することになる)。
    /// masks が「別の口にすると `LayerId` が引けず辿り着けない」という同じ理由で
    /// 内側に埋め込まれているのと対称の話——今回は逆に「`LayerId` の方を運ぶ」形で
    /// 同じ問題を塞ぐ。
    pub id: LayerId,
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
    /// この時刻の effect スタック。**スタックの順**(手前へ畳んでいく mask と違い、
    /// こちらは「上から下へ適用する」順、裁定70)で並ぶ。disabled な effect と
    /// track の無い param は含まれない(型の doc、`ResolvedEffect` 参照)。
    /// **合成器/engine まで消費済み**(裁定153 S1 が resolve() の外へ出す穴を塞ぎ、
    /// S2/S3 で `motolii-compositor`/`motolii-engine` 側の消費が繋がった —
    /// `motolii_engine` の `translate_effect_passes` 参照。ただし対応する pass は
    /// `"motolii.glow"` 1本だけで、他の plugin_id は無音で skip される)。
    pub effects: Vec<ResolvedEffect>,
    /// `layers/visual-layer/bm`。**合成器/engine まで消費済み**(`motolii_engine` の
    /// `translate_blend_mode` 参照)。ただし対応するのは `Normal`/`Add` の2値だけ
    /// (`motolii-compositor` が固定式 blend equation で表現できる範囲、
    /// モジュール doc 参照)— 対応外は engine が `EngineError::UnsupportedBlendMode`
    /// で明示的に拒む。
    pub blend_mode: BlendMode,
    /// matte(裁定66)。**まだ合成器は読んでいない**——`Matte.layer` が指す先を
    /// `resolved_layers()` の結果から引けるようになった(上の `id` フィールド)分だけ
    /// store 側の穴は塞いだが、`motolii-compositor` に2枚目の texture を読む
    /// shader 拡張がまだ無いので、engine は今も `EngineError::UnsupportedMatte`
    /// で明示的に拒む(`next/engine/motolii-engine/tests/blend_matte.rs` 参照)。
    pub matte: Option<Matte>,
    /// `LayerAttrs::pinned`(裁定113)。true ならカメラ変換を受けず画面に張り付く —
    /// 合成器が `placement.transform`/`z` をカメラで動かす前に打ち消す(裁定116 実装)。
    pub pinned: bool,
}
