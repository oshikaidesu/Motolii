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

mod components;
mod document;
mod fingerprint;
mod marker;
mod mask;
mod persist;
mod view;

pub use document::{Document, Intent, LayerId, PropertyId, Revision};
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use marker::Marker;
pub use mask::{Mask, MaskId, MaskMode, ResolvedMask};
pub use view::StoreView;

pub use motolii_core::{CompSpec, Fps, LayerPlacement, RationalTime};
pub use motolii_eval::{Interp, Keyframe, KeyframeTrack, Path, PathVertex, Value};

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
    /// layer の素材と重ね順を上書きする。
    pub const RESERVED: &[&str] = &["meta", "present", "masks"];

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
}

/// layer の素材。media が入るまでは単色だけ。
///
/// **variant を足すのが素材種を増やす唯一の道**にしてある(動画・静止画・生成物が
/// 別々の経路を持たないようにするため)。
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
}

impl LayerSource {
    /// Document が知っている大きさ。実素材は probe しないと分からないので `None`。
    pub fn declared_size(&self) -> Option<[f32; 2]> {
        match self {
            Self::Solid { width, height, .. } => Some([*width as f32, *height as f32]),
            Self::Media { .. } => None,
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
}

impl Default for LayerTiming {
    fn default() -> Self {
        Self {
            start: 0,
            // 0 は「まだ決まっていない」ではなく「尺ゼロ」なので、既定は置かない。
            // `LayerMeta::new` が素材の実尺から埋める。
            duration: 0,
            source_in: 0,
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
        }
    }

    /// comp フレーム → 素材のフレーム。居ない時刻なら `None`。
    ///
    /// **素材の終端でフリーズさせない**(M4)。居ない時刻は描かない。
    pub fn source_frame(&self, comp_frame: i64) -> Option<i64> {
        self.covers(comp_frame)
            .then(|| self.source_in + (comp_frame - self.start))
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
}
