//! token 照合 — `theme::Tokens` は生成正本の写しであって、写し損ねを黙らせない。
//!
//! 比較先は `ui/motolii-tokens/generated/tokens.css`(単一 DTCG 正本
//! `ui/motolii-tokens/sources/motolii-dark.json` からの決定生成物、CU-0B02T)。
//! `tokens.rs` ではなく CSS を読むのは、生成 Rust が `egui::Color32` 型で書かれて
//! いて、この crate は egui を依存に持てないからである
//! (柵: `crates/motolii-testkit/src/ui_toolkit_dep_policy.rs`)。CSS は toolkit
//! 非依存の同一生成物で、同じ `input-sha256` を持つ。
//!
//! 照合は**両方向**: 生成物に在って `Tokens` に無い role も、`Tokens` に在って
//! 生成物に無い role も落とす。値のずれは role 名ごとに言う。

use std::collections::BTreeMap;
use std::fs;

use motolii_shell_iced::theme::{self, Tokens};

/// 生成 CSS の `--motolii-color-<name>: #rrggbbaa;` を全部読む。
fn generated_css() -> BTreeMap<String, iced::Color> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../ui/motolii-tokens/generated/tokens.css"
    );
    let css = fs::read_to_string(path).expect("generated tokens.css lives in the repo");

    let mut out = BTreeMap::new();
    for line in css.lines() {
        let Some(rest) = line.trim().strip_prefix("--motolii-color-") else {
            continue;
        };
        let (name, value) = rest.split_once(':').expect("token line has a value");
        let value = value.trim().trim_end_matches(';');
        let hex = value
            .strip_prefix('#')
            .expect("generator v2 writes #rrggbbaa");
        assert_eq!(hex.len(), 8, "generator v2 writes #rrggbbaa: {line}");
        let byte =
            |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("hex byte in generated css");
        assert_eq!(byte(6), 0xff, "全21 roleは不透明のはず: {line}");
        out.insert(
            name.to_string(),
            iced::Color::from_rgb8(byte(0), byte(2), byte(4)),
        );
    }
    out
}

/// 21 role 全部が、名前も値も生成物と1対1で一致する。
#[test]
fn tokens_dark_matches_the_generated_css_one_to_one() {
    let css = generated_css();
    assert_eq!(css.len(), 21, "manifest.json の role 数は 21");

    let entries = Tokens::DARK.entries();
    assert_eq!(
        entries.len(),
        css.len(),
        "Tokens の role 数が生成物と一致しない"
    );

    for (name, color) in entries {
        let generated = css
            .get(name)
            .unwrap_or_else(|| panic!("`--motolii-color-{name}` が生成物に無い — 発明された role"));
        assert_eq!(color, *generated, "`{name}` の値が生成物とずれた(写し損ね)");
    }
}

/// iced への結線も token を運ぶ — custom palette の種は `Tokens::DARK` そのもの。
///
/// `Palette::generate` は weak/strong 派生を足すが、`base` の色は種を素通しする
/// (`Background::new` / `Swatch::generate` は base の色を変えない)。ここが
/// ずれたら「widget 既定 style だけ別の色」という嘘が起きる。
#[test]
fn the_custom_iced_theme_is_seeded_from_the_tokens() {
    let theme = theme::product();
    let palette = theme.palette();
    let t = &Tokens::DARK;

    assert_eq!(palette.background.base.color, t.surface_app);
    assert_eq!(palette.background.base.text, t.text_primary);
    assert_eq!(palette.primary.base.color, t.action_active);
    assert_eq!(palette.success.base.color, t.status_ok);
    assert_eq!(palette.warning.base.color, t.status_warning);
    assert!(palette.is_dark, "初回既定は Dark(2026-07-14 裁定)");
}
