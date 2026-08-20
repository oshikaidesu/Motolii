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
mod view;

pub use document::{Document, Intent, LayerId, PropertyId, Revision};
pub use fingerprint::{SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1};
pub use view::StoreView;

pub use motolii_core::{CompSpec, Fps, LayerPlacement, RationalTime};
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
    /// component 識別子は `Layer:{name}` なので、**layer 自身の component と衝突する
    /// 名前は禁止**(`PropertyId::new` が弾く)。弾かないと `PropertyId::new("meta")` が
    /// layer の素材と重ね順を上書きする。
    pub const RESERVED: &[&str] = &["meta", "present"];

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

/// layer の非アニメーション属性。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LayerMeta {
    pub source: LayerSource,
    /// 大きいほど手前。上流の `re_renderer::DepthOffset` と同じ `i16`。
    pub order: i16,
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
}
