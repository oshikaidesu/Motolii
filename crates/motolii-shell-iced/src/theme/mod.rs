//! 見た目の写し — token 正本から iced へ。**ここで値を1つも決めない。**
//!
//! 全色は `ui/motolii-tokens/generated/tokens.css`(単一 DTCG 正本
//! `ui/motolii-tokens/sources/motolii-dark.json` からの決定生成物、CU-0B02T)の
//! 写しである。egui 側 `crates/motolii-ui/src/inspector_panel/theme.rs` が
//! `inspector-library.css` を file:line で写しているのと同じ流儀で、各 field に
//! 出所の行を添える。写し先に egui 型の `tokens.rs` でなく CSS を使うのは、
//! この crate が egui を依存に持てないからである(柵:
//! `crates/motolii-testkit/src/ui_toolkit_dep_policy.rs`)。
//! 照合は `tests/theme_matches_generated_tokens.rs` が両方向で固定する。
//!
//! ## Light がまだ無い理由(発明しないため)
//!
//! [UI視覚言語](../../../../docs/ui-visual-language.md)は Dark / Light 同格を決めて
//! いるが、具体値の正本(`ui/motolii-tokens/sources/`)には `motolii-dark.json`
//! しか無い。CU-0B02T が明言する通り「Light、custom、high-contrast、…は、
//! 二supplier間の一意な照合または既存4型の根拠が無いため追加していない」。
//! 仮 hue を新規に決めない規律により、ここで Light を発明しない。正本に light
//! source が入った日、[`ThemeId`] と [`Tokens::of`] に腕が1本増え、
//! [`Tokens::resolve`] が theme 名で分岐するだけの形にしてある。
//!
//! ## 仮パレット(別レーンの `widgets/palette.rs`)との対応
//!
//! 別レーンが置く仮パレットの語彙は、ここの語彙で全て置き換えられる
//! (統合 wave 用の対応表):
//!
//! | 仮パレット | ここ |
//! |---|---|
//! | `bg_panel` | [`Tokens::surface_panel`] |
//! | `bg_control` | [`Tokens::surface_raised`] |
//! | `text_primary` | [`Tokens::text_primary`] |
//! | `text_secondary` | [`Tokens::text_secondary`] |
//! | `accent` | [`Tokens::action_active`] |
//! | `outline` | [`Tokens::border_default`] |

pub mod style;

/// 文字サイズの中央スケール — **これも「発明しない」対象**。値の正本は
/// `docs/reviews/2026-08-19-flat-grammar-canon-revision.md` の定数表
/// (「文字 — 1バンド3段」)。第3波レーンIでこの定数表へ改定した — 旧
/// `BODY 13 / CAPTION 11 / MICRO 10` は Ableton 実測(主文字帯 11px)と揃って
/// いなかった暫定値で、ここに新しい大きさを1つも発明していない(4値とも
/// 定数表からの写し)。
///
/// 型は `f32`(`u16` ではない) — iced の `Text::size` / `Row::spacing` /
/// `Column::spacing` は `impl Into<iced::Pixels>` を取るが、`Pixels` は
/// `From<f32>` と `From<u32>` しか持たず `From<u16>` が無い
/// (`iced::core::pixels` 実測)。`u16` は `padding()`(`Into<Padding>`、
/// こちらは `From<u16>` あり)でしか通らず、両方で使うこの中央スケールは
/// `f32` で揃える。
pub mod type_scale {
    /// 本文・行ラベル — 帯のボタン・project 名・書き出しの一言(旧 `BODY`
    /// を吸収。Ableton実測の主文字帯と同値)。
    pub const BASE: f32 = 11.0;
    /// 補助・メタ行 — legend 行の状況・ヒント(旧 `CAPTION` を吸収)。
    pub const DENSE: f32 = 9.0;
    /// 単位・添字 — これ未満は禁止(定数表の下限フロア)。現状 view.rs に
    /// 出番は無いが、4値の対称性を保つための予約 role(旧 `MICRO` の後継)。
    pub const MICRO: f32 = 8.0;
    /// panel title(旧 14 から1段圧縮)。現状 view.rs に出番は無い —
    /// browser_pane / inspector_pane 側が自前の title を持つため
    /// (このレーンの allowlist 外)。
    pub const TITLE: f32 = 12.0;
}

/// 余白の中央スケール — 値の正本は同じく上記定数表の「spacing scale」。
/// 型は `f32`([`type_scale`] と同じ理由 — `spacing()` も `Into<Pixels>`)。
pub mod space {
    /// status 帯の中で project 行と legend 行を継ぐ、行間の最小 spacing。
    /// scale の最小値 — 帯の合計高さを増やさない(campaign D2 の制約)。
    pub const XS: f32 = 2.0;
    /// 帯 panel の padding・start 画面ボタン間の spacing。
    pub const S: f32 = 4.0;
    /// scale の中央値 — 現状 view.rs に単独の出番は無いが、4値の対称性を
    /// 保つための予約 role。
    pub const M: f32 = 6.0;
    /// 帯の項目間 spacing・action_button 内の名前と近道の間(旧 `M=12`
    /// だった箇所を吸収 — scale の上限を超えるので L へ付け替えた)。
    pub const L: f32 = 8.0;
}

#[cfg(test)]
mod scale_matches_canon {
    //! `docs/reviews/2026-08-19-flat-grammar-canon-revision.md` の定数表
    //! (文字 1バンド3段+title、spacing scale {2,4,6,8})が正本。ここが
    //! 定数表からずれたら red — 旧値(BODY13/CAPTION11/MICRO10、
    //! XS2/S8/M12)へ戻すとどちらのテストも fail する。
    use super::{space, type_scale};

    #[test]
    fn type_scale_is_the_canon_band() {
        assert_eq!(type_scale::BASE, 11.0);
        assert_eq!(type_scale::DENSE, 9.0);
        assert_eq!(type_scale::MICRO, 8.0);
        assert_eq!(type_scale::TITLE, 12.0);
    }

    #[test]
    fn space_is_the_canon_scale() {
        assert_eq!(space::XS, 2.0);
        assert_eq!(space::S, 4.0);
        assert_eq!(space::M, 6.0);
        assert_eq!(space::L, 8.0);
    }
}

use iced::Color;

/// `#rrggbb`(不透明)を [`Color`] へ。生成 CSS の `#rrggbbaa` は全 role `ff` な
/// ので alpha は持たない(照合テストが `ff` であることも確かめる)。
const fn rgb(v: u32) -> Color {
    Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// 組み込み theme の名札。生成 manifest(`ui/motolii-tokens/generated/manifest.json`)
/// の `themes: ["motolii-dark"]` と同数 — 生成物に無い theme をここに立てない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    /// 初回既定(2026-07-14 裁定: 既定=Dark、ライトは正本が来た日に同格で並ぶ)。
    MotoliiDark,
}

/// 21 の意味 role — `motolii-dark.json` の全色。egui 側の生成
/// `GeneratedTheme`(`ui/motolii-tokens/generated/tokens.rs`)と同じ語彙を
/// iced の [`Color`] で持つ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    /// tokens.css:13 `--motolii-color-surface-app: #141414ff`
    pub surface_app: Color,
    /// tokens.css:15 `--motolii-color-surface-panel: #1a1a1aff`
    pub surface_panel: Color,
    /// tokens.css:16 `--motolii-color-surface-raised: #222222ff`
    pub surface_raised: Color,
    /// tokens.css:14 `--motolii-color-surface-hover: #2c2c2cff`
    pub surface_hover: Color,
    /// tokens.css:6 `--motolii-color-border-default: #3b3b3bff`
    pub border_default: Color,
    /// tokens.css:7 `--motolii-color-border-strong: #686868ff`
    pub border_strong: Color,
    /// tokens.css:18 `--motolii-color-text-primary: #f0f0f0ff`
    pub text_primary: Color,
    /// tokens.css:19 `--motolii-color-text-secondary: #c6c6c6ff`
    pub text_secondary: Color,
    /// tokens.css:17 `--motolii-color-text-muted: #929292ff`
    pub text_muted: Color,
    /// tokens.css:9 `--motolii-color-focus: #f0f0f0ff`
    pub focus: Color,
    /// tokens.css:5 `--motolii-color-action-active: #d8b574ff`
    pub action_active: Color,
    /// tokens.css:8 `--motolii-color-data: #78b5b0ff`
    pub data: Color,
    /// tokens.css:10 `--motolii-color-shape: #aaa0d0ff`
    pub shape: Color,
    /// tokens.css:11 `--motolii-color-status-ok: #90b287ff`
    pub status_ok: Color,
    /// tokens.css:12 `--motolii-color-status-warning: #e18a6dff`
    pub status_warning: Color,
    /// tokens.css:23 `--motolii-color-way-project: #6eb3aeff`
    pub way_project: Color,
    /// tokens.css:20 `--motolii-color-way-files: #83a8cfff`
    pub way_files: Color,
    /// tokens.css:22 `--motolii-color-way-plugins: #9f9fcfff`
    pub way_plugins: Color,
    /// tokens.css:24 `--motolii-color-way-stage: #bca072ff`
    pub way_stage: Color,
    /// tokens.css:21 `--motolii-color-way-inspector: #8eb086ff`
    pub way_inspector: Color,
    /// tokens.css:25 `--motolii-color-way-timeline: #cc9587ff`
    pub way_timeline: Color,
}

impl Tokens {
    /// `motolii-dark` の全 role。各値の出所は field の doc(tokens.css の行)。
    pub const DARK: Self = Self {
        surface_app: rgb(0x141414),
        surface_panel: rgb(0x1a1a1a),
        surface_raised: rgb(0x222222),
        surface_hover: rgb(0x2c2c2c),
        border_default: rgb(0x3b3b3b),
        border_strong: rgb(0x686868),
        text_primary: rgb(0xf0f0f0),
        text_secondary: rgb(0xc6c6c6),
        text_muted: rgb(0x929292),
        focus: rgb(0xf0f0f0),
        action_active: rgb(0xd8b574),
        data: rgb(0x78b5b0),
        shape: rgb(0xaaa0d0),
        status_ok: rgb(0x90b287),
        status_warning: rgb(0xe18a6d),
        way_project: rgb(0x6eb3ae),
        way_files: rgb(0x83a8cf),
        way_plugins: rgb(0x9f9fcf),
        way_stage: rgb(0xbca072),
        way_inspector: rgb(0x8eb086),
        way_timeline: rgb(0xcc9587),
    };

    /// 名札 → token 束。
    pub const fn of(id: ThemeId) -> &'static Self {
        match id {
            ThemeId::MotoliiDark => &Self::DARK,
        }
    }

    /// 実行中の [`iced::Theme`] から token 束を引く。widget style ヘルパは
    /// 全部ここを通る — 組み込み theme が1つの今は常に Dark だが、light が
    /// 正本に入ったらここが theme 名で分岐する(component 側は変わらない。
    /// UI視覚言語「UI実装は light/dark の名称や具体色を参照せず、解決済み
    /// semantic tokenだけを読む」)。
    pub fn resolve(_theme: &iced::Theme) -> &'static Self {
        Self::of(ThemeId::MotoliiDark)
    }

    /// 生成 CSS の変数名(`--motolii-color-` の後ろ)との対応表。
    /// 照合テストが「発明された role が無いこと」を両方向で見るための口。
    pub fn entries(&self) -> [(&'static str, Color); 21] {
        [
            ("action-active", self.action_active),
            ("border-default", self.border_default),
            ("border-strong", self.border_strong),
            ("data", self.data),
            ("focus", self.focus),
            ("shape", self.shape),
            ("status-ok", self.status_ok),
            ("status-warning", self.status_warning),
            ("surface-app", self.surface_app),
            ("surface-hover", self.surface_hover),
            ("surface-panel", self.surface_panel),
            ("surface-raised", self.surface_raised),
            ("text-muted", self.text_muted),
            ("text-primary", self.text_primary),
            ("text-secondary", self.text_secondary),
            ("way-files", self.way_files),
            ("way-inspector", self.way_inspector),
            ("way-plugins", self.way_plugins),
            ("way-project", self.way_project),
            ("way-stage", self.way_stage),
            ("way-timeline", self.way_timeline),
        ]
    }
}

/// 窓に出る theme 名。UI視覚言語の組み込み theme 名 `Motolii Dark` と同じ。
pub const THEME_NAME: &str = "Motolii Dark";

/// 製品 theme — iced の custom palette へ [`Tokens::DARK`] を種として結線する。
///
/// `Palette::generate` は weak/strong の派生を(`deviate`/`mix` で)足すが、
/// それは**このレーンで style を書いていない widget の既定色**にしか使われない。
/// M-1 の画面は全部 [`style`] のヘルパで token を直に読む。派生色は正本値の
/// 決定的変換であり、手書き hex ではない(egui 側 `theme.rs` の `mix` と同じ扱い)。
pub fn product() -> iced::Theme {
    let t = Tokens::of(ThemeId::MotoliiDark);
    iced::Theme::custom(
        THEME_NAME,
        iced::theme::palette::Seed {
            background: t.surface_app,
            text: t.text_primary,
            primary: t.action_active,
            success: t.status_ok,
            warning: t.status_warning,
            // 正本に error/danger role がまだ無い(CU-0B02T)。M-1 に danger を
            // 使う widget は無く、ここに新しい hue を発明しない — warning と同じ
            // 値を置き、error token が正本に入った日に差し替える。
            danger: t.status_warning,
        },
    )
}

/// CSS `color-mix(in srgb, A p%, B (100-p)%)` の写し(egui 版 `theme::mix` と同じ)。
/// 葉 widget(`crate::widgets`)の hover / press 段階はここで**token の2色から**
/// 導く — 新しい色定数を作らない。統合 wave で `widgets::palette::mix` の代わりに
/// ここへ差し替えた。
pub fn mix(a: Color, pct_a: f32, b: Color) -> Color {
    let t = (pct_a.clamp(0.0, 100.0)) / 100.0;
    let ch = |x: f32, y: f32| x * t + y * (1.0 - t);
    Color {
        r: ch(a.r, b.r),
        g: ch(a.g, b.g),
        b: ch(a.b, b.b),
        a: 1.0,
    }
}
