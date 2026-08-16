//! egui の fallback chain。**新しいフォントは足さない。**
//!
//! egui の既定 proportional は Ubuntu-Light で、Timeline が使う記号を持っていない。
//! 実測(2026-08-16、cmap 直読み。モックが使う13コードポイントに対して):
//!
//! | 同梱フォント | 収録 | 不足 |
//! |---|---|---|
//! | `Ubuntu-Light.ttf` | 1,194字 | 8個 — `◆ ◇ ▶ ← ↔ → ⌘ ⌄` |
//! | `Hack-Regular.ttf` | 1,548字 | **1個 — `⌘`(U+2318)だけ** |
//!
//! つまり **Hack を proportional の fallback へ連ねるだけで、記号の豆腐は消える**。
//! Hack は `epaint_default_fonts` に既に入っているので、
//! 追加のフォントファイルも license notice も増えない。
//!
//! **CJK はこれでは直らない。** Hack も Ubuntu-Light も CJK を持たない。
//! そちらは再配布可能な CJK フォント(Noto Sans JP 等)の取得が要り、
//! 取得と `docs/references.md` への登録は別の作業である。
//!
//! 等幅が要る列(timecode)は `FontFamily::Monospace` を使う。
//! **egui は OpenType feature を持たないので `tabular-nums` は再現できない** —
//! 等幅書体で解くのが唯一の道であり、[UI視覚言語](../../../docs/ui-visual-language.md)の
//! 「等幅が必要な列はmonospaced書体またはtabular lining機能を使う」もそれを許している。

use egui::{Context, FontFamily};

/// 記号が豆腐にならないよう、proportional の後ろに Hack を連ねる。
///
/// 呼ぶのは `eframe::CreationContext` を持つ入口で1回だけ。
pub fn install_symbol_fallback(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 既定の定義に "Hack" が居る前提で、順序だけを変える。
    // 名前が変わったらここで気づけるよう、黙って挿入せず存在を確かめる。
    debug_assert!(
        fonts.font_data.contains_key("Hack"),
        "epaint の既定フォント名が変わった。fallback chain を見直すこと"
    );

    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        if !family.iter().any(|name| name == "Hack") {
            family.push("Hack".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}
