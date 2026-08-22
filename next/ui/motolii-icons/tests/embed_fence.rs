//! 落ちるテスト先行(裁定186): 全 [`Icon`] variant の SVG が embed され
//! parse 可能であること。variant を足せば `Icon::ALL` 経由で自動的に全数へ
//! 効く(裁定187 の追加24語彙にも個別テスト追記なしで効いている)。

use motolii_icons::{frame_px_for_glyph_px, icon, Icon};

/// 受入条件「SVG は最適化済み・1個あたり数 KB 以下」の機械化。
/// Material Symbols の素の 24px outlined は全て 1KB 未満(実測 135〜752B)。
const MAX_BYTES: usize = 4096;

#[test]
fn every_icon_embeds_a_parseable_svg() {
    let options = usvg::Options::default();
    for &icon in Icon::ALL {
        let bytes = icon.svg_bytes();
        assert!(!bytes.is_empty(), "{} の SVG が空", icon.name());
        let tree = usvg::Tree::from_data(bytes, &options)
            .unwrap_or_else(|e| panic!("{} の SVG が parse 不能: {e}", icon.name()));
        let size = tree.size();
        assert!(
            size.width() > 0.0 && size.height() > 0.0,
            "{} の SVG が寸法ゼロ",
            icon.name()
        );
    }
}

#[test]
fn every_icon_stays_under_the_size_budget() {
    for &icon in Icon::ALL {
        let len = icon.svg_bytes().len();
        assert!(
            len <= MAX_BYTES,
            "{} が {}B — 受入条件(数 KB 以下)超過",
            icon.name(),
            len
        );
    }
}

/// 名前(= vendoring ファイル名)は全 variant で一意 — 台帳の重複検出。
#[test]
fn icon_names_are_unique() {
    let mut names: Vec<&str> = Icon::ALL.iter().map(|i| i.name()).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "Icon の名前が重複している");
}

/// 公開 API のスモーク: 全 variant が widget(Element)へ組める。
/// 色は呼び手が渡す契約(module doc)— ここでは任意の色で良い。
#[test]
fn the_icon_api_builds_an_element_for_every_variant() {
    for &variant in Icon::ALL {
        let _element: iced::Element<'static, ()> =
            icon(variant, 11.0, iced::Color::from_rgb(0.5, 0.5, 0.5));
    }
}

/// handle の id は内容 hash(fork core/src/svg.rs)— 同じアイコンなら毎回
/// 同じ id(rasterize cache が1エントリへ畳まれる前提の検分)。
#[test]
fn handles_are_stable_across_calls() {
    for &variant in Icon::ALL {
        assert_eq!(variant.handle().id(), variant.handle().id());
    }
}

/// グリフ字寸→枠寸の転写は Material の live area 比 24/20(module doc)。
#[test]
fn the_glyph_to_frame_transfer_uses_the_material_live_area_ratio() {
    assert_eq!(frame_px_for_glyph_px(20.0), 24.0);
    assert!((frame_px_for_glyph_px(11.0) - 13.2).abs() < 1e-4);
}
