//! wraps: iced+motolii-store — Browser pane 骨格(B0)+一覧 projection(B1)。
//! **まだ何も描かない**(B1 も view はまだ空 — projection の純関数だけ)。
//!
//! ζ 縫い目調査(`docs/reviews/2026-08-21-browser-seam-survey.md`)+裁定162 の
//! 切片割り: B0 骨格(挙動ゼロ・PNG 一致)→ B1 素材列挙(この crate、`model.rs`)
//! → B2 rail/filter → B3 view(視覚、`browser-library.html` 構造のみ借用し
//! トンマナは tokens 読み替え)→ B4 静止画サムネ ∥ B5 動画サムネ → B6 ドラッグ
//! 配置。第一波は MEDIA 種別のみ(EFFECTS/CREATE は意味起草タスク#14 が空席)。
//!
//! ## この crate の依存は骨格の間だけ最小のまま
//! pane split 流儀(`docs/reviews/2026-08-21-pane-split-survey.md`)が許す
//! 構成は `iced+motolii-core+motolii-store+motolii-tokens-rs+
//! motolii-shell-state(+motolii-media、`motolii-stage-pane` の
//! `motolii-engine` 依存と同型の単独例外)` — B1 で素材列挙(store)を足した。
//! B2 以降、rail/filter(shell-state)・視覚(tokens)・サムネ(media)を実装する
//! 腕から順にこの一覧へ足していく。**`view()` は B1 でもまだ空 `column![]`**
//! (視覚配線は B3)なので、`iced`+`motolii-store` 以外はまだ引かない
//! (汚染ゼロ — 使わない依存を先取りで抱えない)。
//!
//! ## `motolii-shell` への組み込み(B1 時点)
//! root `motolii_shell::Message::Browser(Message)` が1本で畳む想定
//! (`Settings`/`Stage`/`Timeline` と同型)。**B1 でも `Shell::view` に組み込まない**
//! — 描画ゼロ = レイアウト不変が挙動ゼロ変更の最も確実な証明(view 配線は絵と
//! 一緒に B3 で行う)。`motolii_shell::Message::Browser` の match 腕も中身が
//! 無い(`Message` がまだ空 enum のため `match msg {}` で尽くせる)。台帳への
//! 記帳自体(`Intent::AdmitAsset` の発行)は `Shell::admit`(`motolii-shell`
//! 側)が持つ — この crate は読み専用の projection しか持たない。

pub mod model;

pub use model::AssetListItem;

use iced::widget::column;
use iced::Element;

/// pane ローカル Message。**B0 時点では空**(B1 以降、素材列挙/rail・filter/
/// ドラッグ配置の腕をここへ足していく — 腕が増えたら `motolii-shell` 側の
/// `Message::Browser` の match 腕も追随させる)。
#[derive(Clone, Debug, PartialEq)]
pub enum Message {}

/// まだ何も描かない骨格 — 空の `column![]`。B3 で `browser-library.html` の
/// 構造(rail 3席・grid)を足すまでのプレースホルダ。
pub fn view<'a>() -> Element<'a, Message> {
    column![].into()
}
