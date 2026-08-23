//! TEXT section(B46 第1切片、裁定184)。
//!
//! **持つ**: [`TextField`]/[`TextFieldDraft`](`TransformField` とは別系統 ──
//! 対象は `KeyframeTrack` ではなく `TextDocumentStyle` の静止フィールド)・
//! font/size/line-height/tracking/justify の意味と書き口
//! ([`default_text_document`]/[`applied_text_field`]/`commit_text_field`/
//! `cycle_text_justify`/`reset_text_line_height`/`reset_text_tracking`)・
//! TEXT section の view([`text_section`]/`text_field_row`/`line_height_row`/
//! `tracking_row`/`justify_row`)。
//!
//! **持たない**: 型入力→Enter の即時 field(`TextField`/[`commit_text_field`])は
//! 今も丸ごと差し替えの `Intent::SetTextDocument` を使う(据え置き — RETURN
//! 参照。二重帳簿にはならない、下記 A-1b 節)。
//!
//! ## A-1b(裁定214 同日訂正版、evaluator overlay の後継発注)
//! A-1 の懸念(「`StoreView::text_document` はこの切片が置いた
//! `PropertyId::text_style_*`/`text_justify` track を一切評価しない」)は
//! **`view.rs`(store crate)側で解決済み** — `StoreView::resolved_text_document`
//! が「track があればその値、無ければ静的値」を返すようになり、
//! `motolii-engine` の描画経路もそちらを読むよう繋ぎ直した(write-set:
//! `next/core/motolii-store/src/{view,text}.rs`/`next/engine/motolii-engine/`、
//! 詳細は該当 crate の doc・RETURN 参照)。
//!
//! **静的値と track、どちらが正本か**: **track が正本**、`TextDocumentStyle`
//! の静的フィールドは「track が無い時の既定値」——`resolved_text_document` の
//! 読み優先順位そのもの。この切片の `TextField` 経由の型入力(Enter)は
//! **今も静的値だけを書く**(据え置き)——「無ければ既定」を書く経路として
//! 残る一方、track を書く経路([`commit_text_style_track_field`]/
//! [`toggle_text_style_key`]/drag 3関数)が**別腕として追加**された。両者は
//! 同じ値を取り合わない(track が有れば読み出し側が track を勝たせる)ので
//! 二重帳簿ではない。
//!
//! **Key 列/drag は D-1(2026-08-23)で結線済み** —
//! [`text_style_key_button`]/[`text_style_drag_handle`]([`size_row`]/
//! [`line_height_row`]/[`tracking_row`] が呼ぶ)が `crate::Message::
//! TextStyleKeyPressed`/`TextStyleValuePressed` を発火し、`motolii-shell`
//! (`inspector_ops.rs`、write-set 外だが D-1 の write-set には含む)が
//! [`toggle_text_style_key`]/[`start_text_style_drag`]/
//! [`continue_text_style_drag`]/[`finish_text_style_drag`] を呼ぶ。型入力
//! (Enter)は [`text_field_track_target`] を経由して既存の `TextFieldSubmit`
//! (`TextField::Size`/`LineHeight`/`Tracking`)からそのまま
//! [`commit_text_style_track_field`] へ橋渡しする(**新しい draft 型を view
//! へ追加で通す必要が無い**——`view.rs`(shell、write-set 外)を触らずに
//! track 化する唯一の道、詳細は `Shell::update_inspector` の
//! `TextFieldSubmit` 腕 doc 参照)。
//!
//! **3状態 Key oracle の見た目は E-3(2026-08-23)で結線済み** —
//! `TextSectionProjection`(`projection.rs`)は write-set 内(D-1 時点の
//! 誤記を訂正)なので `size_key`/`line_height_key`/`tracking_key`
//! ([`KeyCellState`])を持てる。`text_style_key_button` がそれを
//! `crate::chrome::key_glyph_for_state`(`key_glyph` から視覚だけを取り出した
//! 共有関数)へ渡す——click の意味は変わらず、見た目も Position/Scale 行と
//! 同じ ◇/◆薄/◆濃 の3状態になった。**drag 中の transient 反映も同時に
//! 直した** — `size`/`line_height`/`tracking` は `resolved_text_document`
//! 経由(`value_at` が `set_transient` overlay を最優先で読む)で projection
//! へ入るので、`continue_text_style_drag` が書く transient 値が次の再描画で
//! そのまま値欄に出る(投影を作り直す再描画の頻度に依存 — Shell 側の
//! subscription が drag 中も view を呼び直している前提、RETURN 参照)。


// ---------------------------------------------------------------------------
// SP-4(2026-08-23): 1,365行だった単一 `text.rs` をこのディレクトリモジュール
// へ割った(裁定220 検収)。**中身は無改変・移送だけ** — 割った線は crate doc
// 冒頭が既に名指ししている3層のとおり:
// - [`value`]: [`TextField`]/[`TextFieldDraft`] の識別・font/size/line-height/
//   tracking/justify の**静的値**の意味と書き口(`default_text_document`/
//   `applied_text_field`/`commit_text_field`/…)。
// - [`view`]: TEXT section の view([`text_section`]/`text_field_row`/
//   `line_height_row`/`tracking_row`/`justify_row`)。
// - [`style_track`]: A-1b(裁定214/217)— Size/Line Height/Tracking の
//   **track** 書き口([`TextStyleField`]/`commit_text_style_track_field`/
//   `toggle_text_style_key`/drag 3関数)。
//
// **公開 API は不変**: `pub use value::*;`/`pub use style_track::*;` が旧来の
// `text::X` フラットな経路をそのまま維持する(`lib.rs` の `pub use text::{..}`
// は無改修)。`view` は `pub(crate) use` — [`text_section`] 以外は元から
// 非公開のヘルパなので、可視性を広げない。ただし [`view::text_field_track_target`]
// だけは元から `pub fn`(`lib.rs` の `pub use text::{.., text_field_track_target,
// ..}` が crate 外へも公開している既存 API)なので、個別に `pub` 再輸出する。
// ---------------------------------------------------------------------------
mod value;
mod view;
mod style_track;

pub use value::*;
pub(crate) use view::*;
pub use view::text_field_track_target;
pub use style_track::*;
