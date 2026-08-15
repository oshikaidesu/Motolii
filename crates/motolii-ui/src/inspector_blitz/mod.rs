//! Blitz(HTML/CSS)版Inspectorのskin。
//!
//! `ui/motolii-rn/src/Inspector.tsx` の**見た目だけ**をHTML/CSSへ移す。
//! 色・寸法・間隔の出所は `ui/motolii-rn/src/productStyles.ts` のままで、
//! `theme.rs` はその写しを持つだけ。ここで新しい値を1つも決めない。
//!
//! C7 の範囲はここまで。**データには繋がない。**
//! `Document` も `rn_product_host` も `inspector_host_runtime` も参照しない。
//! 表示する値は `sample.rs` の固定値。入力ルーティングと投影の配線は別capsule。
//!
//! 絵にするのは `dump_main.rs`(`[[bin]] motolii-inspector-blitz-dump`)。

// C7は描画文字列の生成までで、呼び出し側の配線は後続capsule。
// それまで未使用になるためここで明示的に許容する。
#![allow(dead_code)]

mod html;
mod sample;
mod theme;

pub use html::inspector_html;
pub use sample::{DialSample, EffectSample, InspectorSample, KeyState, ParamSample, SAMPLE};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_sample_inspector_as_a_blitz_document() {
        let html = inspector_html(&SAMPLE, 340.0, 760.0);
        assert!(html.starts_with("<html><head><style>"));
        assert!(html.ends_with("</body></html>"));
        assert!(html.contains(r#"class="inspector""#));
        assert!(html.contains(r#"class="parameterRow""#));
    }
}
