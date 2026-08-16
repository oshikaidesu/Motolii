//! `spikes/blitz-probe/src/bin/ui_mock.rs` の `build_html` の**写し**。
//! HTML構造・class名・並び・宣言順を変えない。mockが固定値で持っていた
//! 寸法(W/H/SIDEBAR/ROW_H)と行データだけを `geometry.rs` / `rows.rs` 由来へ差し替える。
//!
//! ## CSSに直書きした値の出所
//!
//! `theme.rs` にある色はこのcrateの `timeline_blitz/theme.rs`(=写し)から差し込む。
//! `theme.rs` に無い色は現行egui実装の直書きリテラルであり、mockが既に写している。
//! 新しい値は1つも作らない。
//!
//! | CSS | 値 | 出所 |
//! |---|---|---|
//! **出所の `timeline_egui/*` は 2026-08-16 に撤去済み**。file:line は撤去前の原文に対するもので、
//! `git show f209da9d^:<path>` で読める。Timelineの正本は現在 `timeline_blitz`(2026-08-16裁定)。
//!
//! | `.ovborder` border | `#d8d8d8` | timeline_egui/ruler.rs:55 |
//! | `.rulerlabel` color | `#c0c0c0` | timeline_egui/ruler.rs:75 |
//! | `.tick` background | `#6a6a6a` | timeline_egui/ruler.rs:84 |
//! | `.sel` background | `#414141` | timeline_egui/clip_band.rs:40 |
//! | `.row` grid線 | `#262626` | timeline_egui/clip_band.rs:56 |
//! | `.tri` / `.dot` | `#929292` | timeline_egui/clip_band.rs:77 (mockが三角にも流用) |
//! | `.tg` 背景/枠/文字 | `#2f2f2f` / `#464646` / `#919191` | ui_mock.rs:212-213 |
//! | `.key` background | `#d6d6d6` | ui_mock.rs:217 (= INK) |
//! | `.selbar` border | `#ffffff` | timeline_egui/clip_band.rs:123 |
//! | `.ph` / `.phhead` | `#e7e7e7` | timeline_egui/ruler.rs:144,152 |
//!
//! ## CSS検証
//!
//! CSSの採用は既存probeのproperty一覧では制限しない。browser previewで設計を確認し、
//! 固定Blitz crate版のdumpで描画を確認する。共通方針は
//! `docs/reviews/2026-08-16-blitz-html-css-authoring-and-validation-decision.md`。

use motolii_core::RationalTime;
use motolii_doc::{Document, LayerId};

use super::rows::{rows_from_projection, TimelineRow};
use super::theme::{
    ACCENT, BAR_INK, CONTRAST, DESKTOP, DIM, INK, LOCATOR_H, OVERVIEW_H, PALETTE, ROW_H, RULER,
    RULER_H, SURFACE, SURFACE_HI, SURFACE_LO,
};
use crate::timeline_projection::TimelineProjection;

/// `timeline_egui::paint_timeline` と同じ入力から、同じ絵のHTML文書を作る。
/// 入力処理も編集意味もここには無い。Documentは読むだけ。
pub fn timeline_html(
    document: &Document,
    projection: Option<&TimelineProjection>,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> String {
    timeline_html_with_dense_mode(document, projection, primary, playhead, false)
}

/// custom widgetへ昇格する前の、clip/keyを通常DOMで出す参照実装。
///
/// 外側のHTML/CSS・Document投影・座標式は製品版と同じ。違うのは密な面を
/// DOM nodeで描く一点だけで、ブラウザとPNGでUXを詰めるために使う。
pub fn timeline_html_dom_prototype(
    document: &Document,
    projection: Option<&TimelineProjection>,
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> String {
    timeline_html_with_dense_mode(document, projection, primary, playhead, true)
}

fn timeline_html_with_dense_mode(
    document: &Document,
    projection: Option<&TimelineProjection>,
    primary: Option<LayerId>,
    playhead: RationalTime,
    dom_dense_surface: bool,
) -> String {
    let rows = rows_from_projection(document, projection);
    format!(
        "<html><head><style>{}</style></head><body>{}</body></html>",
        css(),
        body(
            &rows,
            primary,
            playhead,
            document.composition.duration.as_seconds_f64(),
            dom_dense_surface,
        )
    )
}

/// CSSの実体は `timeline.css`。**Rustを再ビルドせずに触れる**(`blitz_css`)。
/// 差し込む値の出所は `theme.rs`(色)と `geometry.rs`(寸法)で、ここでは決めない。
fn css() -> String {
    const EMBEDDED: &str = include_str!("timeline.css");
    crate::blitz_css::fill(
        &crate::blitz_css::template("timeline_blitz/timeline.css", EMBEDDED),
        &[
            ("overview_h", &OVERVIEW_H.to_string()),
            ("ov_border_h", &(OVERVIEW_H - 3.0).to_string()),
            ("locator_h", &LOCATOR_H.to_string()),
            ("ruler_h", &RULER_H.to_string()),
            ("phhead_top", &(OVERVIEW_H - 6.0).to_string()),
            (
                "rows_top",
                &(OVERVIEW_H + RULER_H + LOCATOR_H + 1.0).to_string(),
            ),
            ("row_h", &(ROW_H - 1.0).to_string()),
            ("bar_h", &(ROW_H - 4.0).to_string()),
            ("bar_ink", BAR_INK),
            ("surface_hi", SURFACE_HI),
            ("surface_lo", SURFACE_LO),
            ("surface", SURFACE),
            ("contrast", CONTRAST),
            ("desktop", DESKTOP),
            ("accent", ACCENT),
            ("ruler", RULER),
            ("dim", DIM),
            ("ink", INK),
        ],
    )
}

/// ui_mock.rs:75-186 の写し。
fn body(
    rows: &[TimelineRow],
    primary: Option<LayerId>,
    playhead: RationalTime,
    duration: f64,
    dom_dense_surface: bool,
) -> String {
    let rows_top = OVERVIEW_H + RULER_H + LOCATOR_H + 1.0;
    let mut b = String::new();

    // ---- 席全体の下地 ----
    // timeline_egui/mod.rs:79 の rect_filled(rect, DESKTOP) に相当。
    // mockは body の背景色で代用していたが、Blitzでは body の背景が
    // viewport全面を不透明で塗り、Stageの上へ重ねられなくなる
    // (blitz-paint-0.3.0-beta.1/src/render.rs:127-160)。要素側へ移す。
    b.push_str(r#"<div class="desktop">"#);
    b.push_str(&format!(
        r#"<div class="ruler" style="top:{}px"></div><div class="locator" style="top:{}px"></div>"#,
        OVERVIEW_H + 1.0,
        OVERVIEW_H + RULER_H + 1.0
    ));

    // 面高でDOMを増減させず、viewport のclipへ任せる。resizeでDocumentを作り直さないため。
    let selected_index = primary.as_ref().and_then(|layer| {
        rows.iter()
            .position(|row| row.property.is_none() && row.layer == *layer)
    });
    for (index, row) in rows.iter().enumerate() {
        let y = rows_top + index as f64 * ROW_H;
        let selected = row.property.is_none() && selected_index == Some(index);
        b.push_str(&format!(
            r#"<div class="row{}" style="top:{y}px"></div>"#,
            if selected { " sel" } else { "" }
        ));
    }

    // `.sidebar` の min/max/割合は TimelineGeometry::new と同じ clamp をCSSへ写したもの。
    b.push_str(r#"<div class="sidebar"><div class="ovlabel">overview</div><div class="rulerlabel">Inbox</div>"#);
    for (index, row) in rows.iter().enumerate() {
        let y = rows_top + index as f64 * ROW_H;
        let cy = y + ROW_H * 0.5;
        if row.property.is_some() {
            b.push_str(&format!(
                r#"<div class="dot" style="top:{}px"></div><div class="plabel" style="top:{}px">{}</div>"#,
                cy - 1.0,
                cy - 5.0,
                escape(&row.label)
            ));
        } else {
            b.push_str(&format!(
                r#"<div class="tri" style="top:{}px"></div><div class="llabel" style="top:{}px">{}</div><div class="tg tg-m" style="top:{}px">M</div><div class="tg tg-s" style="top:{}px">S</div>"#,
                cy - 4.0,
                cy - 5.0,
                escape(&row.label),
                cy - 6.0,
                cy - 6.0,
            ));
        }
    }
    b.push_str("</div>");

    // ---- 時間面 ----
    b.push_str(r#"<div class="track"><div class="ov"></div><div class="ovborder"></div>"#);
    for (i, r) in rows.iter().enumerate() {
        if r.property.is_some() {
            continue;
        }
        let (Some(start), Some(end)) = (r.start, r.end) else {
            continue;
        };
        if end <= start {
            continue;
        }
        b.push_str(&format!(
            r#"<div class="ovbar" style="left:{}%;top:{}px;width:{}%;background:{}"></div>"#,
            start.clamp(0.0, 1.0) * 100.0,
            4.0 + i as f64 * 4.0,
            ((end - start).clamp(0.0, 1.0) * 100.0).max(0.1),
            PALETTE[r.palette_slot]
        ));
    }

    // ruler と行のgridは時間面を親にして、横幅に追随させる。
    for i in 0..=10 {
        let percent = i as f64 * 10.0;
        b.push_str(&format!(
            r#"<div class="tick" style="left:{percent}%;top:{}px"></div>
               <div class="ticklabel" style="left:{percent}%;top:{}px">{i}s</div>
               <div class="gridline" style="left:{percent}%"></div>"#,
            OVERVIEW_H + RULER_H - 5.0,
            OVERVIEW_H + 3.0
        ));
    }

    // clip名だけはDOMに残す。箱の相対幅で位置を決めるのでviewport更新で追随する。
    let mut labels = String::new();
    for (i, r) in rows.iter().enumerate() {
        let y = rows_top + i as f64 * ROW_H;
        if r.property.is_none() {
            if let (Some(start), Some(end)) = (r.start, r.end) {
                if end > start {
                    labels.push_str(&format!(
                        r#"<div class="barlabel" style="left:{}%;top:{}px">{}</div>"#,
                        start.clamp(0.0, 1.0) * 100.0,
                        y + 1.0,
                        escape(&r.label)
                    ));
                }
            }
        }
    }

    if dom_dense_surface {
        // 参照実装: custom widget昇格前のHTML。座標・class・色をwidget版と揃える。
        for (index, row) in rows.iter().enumerate() {
            let y = rows_top + index as f64 * ROW_H + 1.0;
            if row.property.is_none() {
                if let (Some(start), Some(end)) = (row.start, row.end) {
                    if end > start {
                        let selected = primary == Some(row.layer);
                        b.push_str(&format!(
                            r#"<div class="bar{}" style="left:{}%;top:{y}px;width:{}%;background:{}">{}</div>"#,
                            if selected { " selbar" } else { "" },
                            start.clamp(0.0, 1.0) * 100.0,
                            (end - start).clamp(0.0, 1.0) * 100.0,
                            PALETTE[row.palette_slot],
                            escape(&row.label),
                        ));
                    }
                }
            } else if row.property == Some("Position") {
                for key in &row.keys {
                    b.push_str(&format!(
                        r#"<div class="key" style="left:{}%;top:{}px"></div>"#,
                        key.fraction.clamp(0.0, 1.0) * 100.0,
                        y + 4.0,
                    ));
                }
            }
        }
    } else {
        // 製品経路: 素材数に比例するDOMを避け、密な面を1 custom widgetで描く。
        b.push_str(r#"<div id="tl-surface" class="tlsurface"></div>"#);
    }
    b.push_str(&labels);

    // ---- surface 境界線と playhead ----
    let playhead_percent = if duration.is_finite() && duration > 0.0 {
        (playhead.as_seconds_f64() / duration).clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    };
    b.push_str(&format!(
        r#"<div class="vsep"></div><div class="ph" style="left:{playhead_percent}%"></div><div class="phhead" style="left:{playhead_percent}%"></div>"#,
    ));
    b.push_str("</div></div>");
    b
}

/// レイヤ名はDocument由来なのでDOMを壊せる。表示文字列としてだけ通す。
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline_blitz::project_for_blitz;

    fn sample_html() -> String {
        let document = Document::new_current();
        let projection = project_for_blitz(&document).expect("current document timeline");
        timeline_html(&document, Some(&projection), None, RationalTime::ZERO)
    }

    #[test]
    fn emits_every_theme_constant_as_a_literal() {
        let html = sample_html();
        for hex in [
            DESKTOP, SURFACE, SURFACE_HI, SURFACE_LO, CONTRAST, DIM, RULER, ACCENT, INK, BAR_INK,
        ] {
            assert!(html.contains(hex), "CSSに {hex} が出ていない");
        }
    }

    #[test]
    fn emits_the_same_class_vocabulary_as_the_probe_mock() {
        let html = sample_html();
        for class in [
            ".desktop",
            ".ov",
            ".ovlabel",
            ".ovborder",
            ".ovbar",
            ".ruler",
            ".rulerlabel",
            ".tick",
            ".ticklabel",
            ".locator",
            ".row",
            ".sel",
            ".tri",
            ".dot",
            ".llabel",
            ".plabel",
            ".tg",
            ".bar",
            ".selbar",
            ".key",
            ".selkey",
            ".vsep",
            ".ph",
            ".phhead",
        ] {
            assert!(html.contains(class), "mockのclass {class} が無い");
        }
    }

    #[test]
    fn uses_the_document_viewport_instead_of_baked_panel_dimensions() {
        let html = sample_html();
        assert!(html.contains(".desktop { left: 0; top: 0; width: 100%; height: 100%"));
        assert!(html.contains(".sidebar { left:0; top:0; width:25.5%; height:100%"));
        assert!(html.contains(".track { left:25.5%; top:0; right:0; height:100%"));
        assert!(!html.contains("width:1000px"));
        assert!(!html.contains("height:520px"));
    }

    /// `body` に背景色を置くとviewport全面が不透明になり、Stageの上へ重ねられない
    /// (blitz-paint-0.3.0-beta.1/src/render.rs:127-160)。下地は要素側に置く。
    #[test]
    fn keeps_the_panel_background_off_of_body() {
        let html = sample_html();
        let head_end = html.find("div {").expect("div rule");
        assert!(
            !html[..head_end].contains("background"),
            "html/body に背景色が残っている"
        );
        assert!(html.contains(".desktop"));
    }

    #[test]
    fn escapes_layer_names_so_they_cannot_break_the_dom() {
        assert_eq!(escape("a<b>&\"c\""), "a&lt;b&gt;&amp;&quot;c&quot;");
    }
}
