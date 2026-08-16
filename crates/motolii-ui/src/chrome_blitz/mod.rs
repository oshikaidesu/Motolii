//! Blitz(HTML/CSS)版 chrome と extension panel の skin。
//!
//! `ui/motolii-rn/src/chrome.tsx`(172行)、`ui/motolii-rn/src/panels/registry.tsx`(24行)、
//! `ui/motolii-rn/src/panels/AssetTaggingPanel.tsx`(56行)の**見た目だけ**をHTML/CSSへ移す。
//!
//! 色・寸法・間隔の出所は3つある。**どれもここで決めない。**
//!
//! | 出所 | 対象 | `theme.rs` の表 |
//! |---|---|---|
//! | `productStyles.ts` | `chrome.tsx` が `styles.X` で引くもの | `STYLES` |
//! | `AssetTaggingPanel.tsx:39-56` のローカル `StyleSheet` | Tagsパネル | `ASSET_TAGS_STYLES` |
//! | `registry.tsx:20-24` のローカル `StyleSheet` | Notesパネル | `EXPORT_NOTES_STYLES` |
//!
//! 後ろ2つは `productStyles.ts` に**無い**。移植元がそこに書いていないだけで、
//! 新しく決めた値ではない。`theme.rs` のテストがそれぞれの原文と逐語で突き合わせる。
//!
//! C8 の範囲はここまで。**データにも入力にも繋がない。**
//! `Document` も `rn_product_host` も参照しない。表示する値は `sample.rs` の固定値。
//!
//! 絵にするのは `blitz_dump`(`[[bin]] motolii-blitz-dump`)。C7 と違って専用の bin は作らない
//! — Timeline / Browser / dock と同じ道具から出せるほうが、4枚を並べて見るときに都合がよい。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-dump -- all out/
//! ```

// C8は描画文字列の生成までで、呼び出し側の配線は後続capsule。
// それまで未使用になるためここで明示的に許容する。
#![allow(dead_code)]

mod html;
mod sample;
mod theme;

pub use html::{export_html, panels_html, parts_html, settings_html};
pub use sample::{ExportPhase, ExportSample, EXPORT_SAMPLE};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_every_chrome_screen_as_a_blitz_document() {
        for html in [
            export_html(&EXPORT_SAMPLE),
            settings_html(),
            panels_html(),
            parts_html(),
        ] {
            assert!(html.starts_with("<html><head><style>"));
            assert!(html.ends_with("</body></html>"));
        }
    }

    /// `body` に背景色を置くと viewport 全面が不透明になり、Stage の上へ重ねられない
    /// (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。4枚とも置いていないこと。
    #[test]
    fn never_paints_the_body() {
        for html in [
            export_html(&EXPORT_SAMPLE),
            settings_html(),
            panels_html(),
            parts_html(),
        ] {
            let head = html.split("</style>").next().expect("style block");
            for line in head.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("body") || trimmed.starts_with("html,body") {
                    assert!(
                        !trimmed.contains("background"),
                        "body に背景色が置かれている: {trimmed}"
                    );
                }
            }
        }
    }
}
