//! Timeline の投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//! `time_band_segment_frames`/`property_rows`/`layer_row_top`)。**読むだけ** —
//! Document/Session を書き換えない。
//!
//! ## SP-2 分割(1232行 → 800行以下、中身は無改変・純粋な移送)
//!
//! 単一ファイルだった頃、**layer 木の flatten(`rows`)・音声の有無投影・
//! フレーム⇄px の座標変換とルーラー目盛り・property 行(キー行)投影・
//! ドラッグ中のライブプレビュー**という5つの独立した責任が同居していた。
//! 互いの呼び出し関係が薄い(ほぼ一方向: `preview` だけが `tree::row_selected`
//! を要る)ので、責任ごとに子モジュールへ分けた:
//! - [`tree`] … `rows`/`mark_reachable`/`push_layer`/`row_selected`
//! - [`audio`] … `AudioRowProjection`/`audio_rows`
//! - [`geometry`] … `frame_to_x`/`frame_at_x`/`tick_steps`/`time_band_segment_frames`
//! - [`properties`] … `PropertyKeyProjection`/`PropertyRowProjection`/
//!   `property_rows`/`selected_row_index`/`layer_row_top`/`layer_row_at_y`/`key_order`
//! - [`preview`] … `apply_clip_preview`/`apply_key_preview`
//!
//! `RowProjection`(全モジュールが読む共通型)だけはここ(`mod.rs`)に残した。
//! **crate 外の呼び手を1つも壊さない**: `frame_to_x`/`frame_at_x`/`tick_steps`/
//! `time_band_segment_frames`/`rows`/`property_rows`/`audio_rows`/
//! `selected_row_index`/`layer_row_top` 等はすべて `pub`(裁定160 切片7で
//! `motolii_shell::screenshot` の cross-crate 参照のために緩めた物)のまま、
//! ここで `pub use 子モジュール::X;` として `projection::X` へ再輸出する —
//! `lib.rs` の `pub use projection::{...}` は無改修。crate 内から
//! `super::projection::X`/`crate::projection::X` で直接参照している箇所
//! (`hit.rs`/`canvas.rs`/`key_rows.rs` 等)も同じ再輸出で無改修のまま通る。
//! `apply_clip_preview`/`apply_key_preview`(元 `pub(super)` = crate root まで
//! 到達)は `pub(crate)` として同じ到達範囲を保った(`preview` モジュール
//! doc 参照)。

use std::collections::{HashMap, HashSet};

use motolii_store::{Fps, LayerId, LayerSource, LayerTiming, PropertyId, StoreView};

use crate::state::Session;

/// `KeySelector`/`KeySelectionOp` は裁定160 切片6 で `crate::state` へ移設済み
/// (pane split survey §2.3: `Session ⇄ timeline` の型循環解消 — `state` は
/// leaf、`timeline` はそこへ依存する片方向)。`pub use` は `timeline::mod` の
/// `pub use projection::{..., KeySelectionOp, KeySelector, ...}` を無改修で
/// 保つための re-export(型 alias で外部参照を壊さない手口)。
pub use crate::state::{KeySelectionOp, KeySelector};

/// 1層分の読み取り投影。**Document の写しではなく、1度描くための使い捨て値**。
#[derive(Clone, Debug, PartialEq)]
pub struct RowProjection {
    pub id: LayerId,
    pub name: String,
    pub hidden: bool,
    /// solo(`LayerAttrs.solo`)。レーンバーの S トグルが読む(裁定147)。
    pub solo: bool,
    /// locked(`LayerAttrs.locked`)。レーンバーの L トグルが読む(裁定147)。
    pub locked: bool,
    /// レイヤー差し色の index(`LayerAttrs.label_color`)。`None` = 未割当 —
    /// bar は既定色(`way_timeline`)のまま(`canvas::draw` 参照)。
    pub label_color: Option<u8>,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
    /// クリップ drag のプレビュー中(第2波T5、正典 §2「ドラッグ中の bar は
    /// ACCENT」)。`rows()` は常に `false` — [`apply_clip_preview`] だけが
    /// preview 列に含まれる各行へ立てる。`selected` とは別ロール(trim は
    /// 選択を変えないので、選択と drag 中は独立に真偽が分かれ得る)。
    pub dragging: bool,
    /// **裁定173 H2**: `attrs.parent` を辺として読んだ木の深さ。0 = 最上位
    /// (parent が無い、または parent が存在しない/循環している孤立行)。
    /// rail のインデントの出典(`rail::layer_row` 参照)。
    pub depth: u16,
    /// この行が子(`attrs.parent` がこの layer を指す present な layer)を
    /// 1つでも持つか。fold 三角を出すかどうかの出典 — 子を持たない行は
    /// 矢印を出さない(旧世界 `timeline_rows.rs` と同じ規則)。
    pub has_children: bool,
    /// 子が展開されているか(`!session.timeline_fold.is_folded(id)`)。
    /// `has_children == false` の行では意味を持たない(矢印自体が無い)。
    pub children_open: bool,
}

mod audio;
mod geometry;
mod preview;
mod properties;
mod tree;

pub use audio::{audio_rows, AudioRowProjection};
pub use geometry::{frame_at_x, frame_to_x, tick_steps, time_band_segment_frames};
pub(crate) use preview::{apply_clip_preview, apply_key_preview};
pub use properties::{
    key_order, layer_row_top, property_rows, selected_row_index, PropertyKeyProjection,
    PropertyRowProjection,
};
pub(crate) use properties::layer_row_at_y;
pub use tree::rows;
