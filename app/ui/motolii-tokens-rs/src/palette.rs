//! ラベルパレットのうち、**色を持たない事実**だけを置く場所。
//!
//! 長さは生成側(`LayerId % LABEL_PALETTE_LEN` の決定論割当)と表示側の index
//! 境界チェックの両方が読む。色そのもの(`Colors::label_palette`)は view の
//! 型(`iced::Color`)を抱えるので `colors` 側に残し、feature の後ろへ置いた。
//! 長さだけはどちらの柱からも読めるようここに居る。

/// [`crate::Colors::label_palette`] の長さ。値を2箇所に持たない。
pub const LABEL_PALETTE_LEN: usize = 12;
