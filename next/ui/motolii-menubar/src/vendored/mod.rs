//! iced_aw(rev 924be285、MIT)の menu widget の vendoring 移植。
//!
//! 上流 crate の `src/widget/menu/`(widget 実体)と `src/style/{menu_bar,status}.rs`
//! (menu が使う style catalog)だけを持ち込み、それ以外の iced_aw は一切引かない。
//! 各ファイル冒頭に出典(リポ URL+rev+upstream path)と MIT 表示、motolii 変更点を
//! 明記してある — 変更は fork API 追随(`Shell::local` の `Bus` 化)の2箇所のみ。
//!
//! 上流が fork の iced へ追随したらこのディレクトリごと捨てて crate 依存へ戻す
//! (crate marker 参照)。

pub mod menu;
pub mod style;
