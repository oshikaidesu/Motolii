//! Blitz(HTML/CSS)版Timelineのskin。
//!
//! 現行Timelineの**見た目だけ**をHTML/CSSへ移す。意味・寸法・色の出所は
//! `timeline_egui/theme.rs` と `timeline_egui/geometry.rs` のままであり、
//! ここはその写しを持つだけで新しい値を決めない。
//!
//! 入力ルーティング(C2)、key帯のcustom widget化(C3)、
//! 自前wgpuテクスチャへの出力はこのmoduleの担当外。
//! テクスチャ出力は `crates/motolii-ui/Cargo.toml` へ blitz-* / vello_hybrid 依存を
//! 足さないと書けず、それはC1のALLOWLISTの外にある。

// C1は描画文字列の生成までで、呼び出し側の配線はC2以降。
// それまで未使用になるためここで明示的に許容する。
#![allow(dead_code)]

mod geometry;
mod html;
mod rows;
mod theme;

use motolii_core::RationalTime;
use motolii_doc::Document;

use crate::timeline_projection::{
    project_timeline, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};

pub(crate) use html::timeline_html;

/// timeline_egui/mod.rs:39-56 の写し。投影の作り方を変えないため。
pub(crate) fn project_for_blitz(
    document: &Document,
) -> Result<TimelineProjection, TimelineProjectionError> {
    let duration = document.composition.duration;
    let duration_seconds = duration.as_seconds_f64();
    project_timeline(
        document,
        &TimelineMetrics {
            band_height: 1.0,
            units_per_second: duration_seconds.recip(),
            key_half_extent: 1.0,
        },
        &TimelineViewport {
            start: RationalTime::ZERO,
            end: duration,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_current_document_timeline_as_a_blitz_document() {
        let document = Document::new_current();
        let projection = project_for_blitz(&document).expect("current document timeline");
        let html = timeline_html(
            &document,
            Some(&projection),
            None,
            RationalTime::ZERO,
            1000.0,
            460.0,
        );
        assert!(html.starts_with("<html><head><style>"));
        assert!(html.ends_with("</body></html>"));
        assert!(html.contains("class=\"row\""));
        assert!(html.contains("class=\"bar\""));
    }

    /// 投影が無い時に落ちないこと。egui側 rows_from_projection と同じ空扱い。
    #[test]
    fn renders_an_empty_surface_without_a_projection() {
        let document = Document::new_current();
        let html = timeline_html(&document, None, None, RationalTime::ZERO, 1000.0, 460.0);
        assert!(html.contains("class=\"vsep\""));
        assert!(!html.contains("class=\"row\""));
    }
}
