//! wraps: re_entity_db::EntityDb — Document の実体。
//!
//! **undo / redo は `edit` timeline の latest-at 移動そのもの**であり、自前の履歴機構を
//! 持たない(rerun blueprint の undo と同じ機構。R0-2 で1000編集跨ぎを実測)。
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
mod view;

pub use document::{Document, Intent, LayerId, PropertyId};
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use view::StoreView;

pub use motolii_core::RationalTime;
pub use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value};

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
}

/// 標準 property の名前。**ここに無い名前も置けるが、標準面はこれを見る**。
pub mod property {
    pub const POSITION_X: &str = "position.x";
    pub const POSITION_Y: &str = "position.y";
    pub const WIDTH: &str = "size.width";
    pub const HEIGHT: &str = "size.height";
    pub const OPACITY: &str = "opacity";
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

/// layer の非アニメーション属性。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerMeta {
    pub source: LayerSource,
    /// 大きいほど手前。
    pub order: i32,
}

/// ある comp 時刻に解決済みの layer。**合成器が要るのはこれだけ**。
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLayer {
    pub source: LayerSource,
    pub order: i32,
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub opacity: f32,
}
