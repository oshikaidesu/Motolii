//! `TextWeight`/`Ink` — 文字の太さ3段とink3段(裁定137)。
//! `lib.rs` から分割(SP-8、中身は移送のみ)。

use iced::Color;

use crate::Colors;

/// 文字の太さ3段(裁定137「文字の階層はサイズでなく weight(400/600/800)と
/// ink 3段で作る」)。CSS `font-weight` の値とそのまま対応させ、視覚正本
/// `next/reference/mocks/ui-scale-and-z.html` の実使用箇所を名指しする:
/// `.glyph`(M/S/Key マーカー全般)= 800、`.ident b`(identity 名の強調)= 600、
/// それ以外の本文は既定 400(明示しなくても iced の既定と同じ)。
/// `iced::font::Weight` は 100刻みの9段(Thin..Black)を持つので、CSS の
/// 400/600/800 は `Normal`/`Semibold`/`ExtraBold` に1:1で対応する
/// (`iced_core::font::Weight` 実測、上流に per-CSS-value のズレは無い)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWeight {
    /// 400 — 本文既定。
    Regular,
    /// 600 — mock `.ident b`(identity 名)。
    Semibold,
    /// 800 — mock `.glyph`(M/S/Key マーカー)。
    Bold,
}

impl TextWeight {
    /// `text`/`text_input` の `.font(..)` へそのまま渡せる `iced::Font`。
    pub fn font(self) -> iced::Font {
        iced::Font {
            weight: match self {
                TextWeight::Regular => iced::font::Weight::Normal,
                TextWeight::Semibold => iced::font::Weight::Semibold,
                TextWeight::Bold => iced::font::Weight::ExtraBold,
            },
            ..iced::Font::DEFAULT
        }
    }
}

/// ink 3段(裁定137)。**新色は発明しない** — 既存 [`Colors`] の `text_*` を
/// そのまま返す薄いラッパー。呼び出し側が raw な `colors.text_muted` 等を
/// 直書きする代わりに、mock の `--ink`/`--ink2`/`--ink3` と同じ語彙(意味段)
/// で選べるようにするだけで、色の実体は増えない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    /// mock `--ink`(既定の本文色。`.prow .n` 等)。
    Primary,
    /// mock `--ink2`(`.glyph`/`.ident s` — 二次的な情報)。
    Secondary,
    /// mock `--ink3`(`.sec`/`.cols`/`.hint` — 最も控えめな注記)。
    Muted,
}

impl Ink {
    pub fn resolve(self, colors: &Colors) -> Color {
        match self {
            Ink::Primary => colors.text_primary,
            Ink::Secondary => colors.text_secondary,
            Ink::Muted => colors.text_muted,
        }
    }
}

#[cfg(test)]
mod text_weight_and_ink_tests {
    use super::{Colors, Ink, TextWeight};

    /// 裁定137の3段(400/600/800)が `iced::font::Weight` の対応する段に
    /// 正しく写ること。CSS の値そのものが変数名に現れる上流列挙
    /// (`Normal`=400/`Semibold`=600/`ExtraBold`=800)へ1:1で繋がっているかの柵。
    #[test]
    fn text_weight_maps_to_the_canonical_css_bands() {
        assert_eq!(
            TextWeight::Regular.font().weight,
            iced::font::Weight::Normal
        );
        assert_eq!(
            TextWeight::Semibold.font().weight,
            iced::font::Weight::Semibold
        );
        assert_eq!(
            TextWeight::Bold.font().weight,
            iced::font::Weight::ExtraBold
        );
    }

    /// ink 3段は既存 `Colors::text_*` をそのまま返すだけ(新色を発明しない、
    /// 裁定139)。
    #[test]
    fn ink_resolves_to_the_existing_colors_without_inventing_new_ones() {
        let colors = Colors::default();
        assert_eq!(Ink::Primary.resolve(&colors), colors.text_primary);
        assert_eq!(Ink::Secondary.resolve(&colors), colors.text_secondary);
        assert_eq!(Ink::Muted.resolve(&colors), colors.text_muted);
    }
}

