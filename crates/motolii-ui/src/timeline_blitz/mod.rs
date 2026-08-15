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
mod surface;
pub mod theme;

use motolii_core::RationalTime;
use motolii_doc::Document;

use crate::timeline_projection::{
    project_timeline, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};

pub use html::timeline_html;

/// 時間面の密な部分(clip と key)を描く custom widget を文書へ挿す。
///
/// `timeline_html` が置いた `#tl-surface` の箱へ結ぶ。**この呼び出しをしないと
/// clip と key は描かれない**(DOM 側には置いていないため)。
/// 引数は `timeline_html` と同じものを渡すこと — 同じ行と同じ選択から組む。
///
/// 挿せなかった場合は `false`。呼び手はそれを絵の欠落として扱う
/// (勝手に DOM へ戻さない — 二重に描く経路を作らないため)。
pub fn attach_surface(
    blitz_document: &mut blitz_html::HtmlDocument,
    document: &Document,
    projection: Option<&TimelineProjection>,
    primary: Option<motolii_doc::LayerId>,
    width: f64,
    height: f64,
) -> bool {
    let rows = rows::rows_from_projection(document, projection);
    let geometry = geometry::TimelineGeometry::new(
        width,
        height,
        rows.len(),
        document.composition.duration.as_seconds_f64(),
    );
    let selected_index = primary.as_ref().and_then(|layer| {
        rows.iter()
            .position(|row| row.property.is_none() && row.layer == *layer)
    });
    let Some(node_id) = blitz_document.query_selector("#tl-surface").ok().flatten() else {
        return false;
    };
    blitz_document.set_custom_widget(
        node_id,
        Box::new(surface::TimelineSurface::new(
            rows,
            geometry.row_height,
            selected_index,
        )),
    );
    true
}

/// timeline_egui/mod.rs:39-56 の写し。投影の作り方を変えないため。
pub fn project_for_blitz(
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
