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
//! ## 使ったCSSプロパティ
//!
//! すべて `spikes/blitz-probe/` で実際に効いたものだけ。新規プロパティは無い。
//! `margin` `padding` `padding-left` `width` `height` `background` `background-image`
//! `font-family` `font-size` `color` `position` `top` `left` `border` `border-left`
//! `border-top` `border-bottom` `text-align` `overflow` `white-space` `transform`

use motolii_core::RationalTime;
use motolii_doc::{Document, LayerId};

use super::geometry::TimelineGeometry;
use super::rows::{rows_from_projection, TimelineRow};
use super::theme::{
    ACCENT, BAR_INK, CONTRAST, DESKTOP, DIM, INK, LOCATOR_H, OVERVIEW_H, PALETTE, RULER, RULER_H,
    SURFACE, SURFACE_HI, SURFACE_LO,
};
use crate::timeline_projection::TimelineProjection;

/// `timeline_egui::paint_timeline` と同じ入力から、同じ絵のHTML文書を作る。
/// 入力処理も編集意味もここには無い。Documentは読むだけ。
pub fn timeline_html(
    document: &Document,
    projection: Option<&TimelineProjection>,
    primary: Option<LayerId>,
    playhead: RationalTime,
    width: f64,
    height: f64,
) -> String {
    let rows = rows_from_projection(document, projection);
    let geometry = TimelineGeometry::new(
        width,
        height,
        rows.len(),
        document.composition.duration.as_seconds_f64(),
    );
    format!(
        "<html><head><style>{}</style></head><body>{}</body></html>",
        css(&geometry),
        body(&geometry, &rows, primary, playhead)
    )
}

/// ui_mock.rs:188-229 の写し。
fn css(geometry: &TimelineGeometry) -> String {
    let w = geometry.width;
    let h = geometry.height;
    let ov_border_h = OVERVIEW_H - 3.0;
    let row_h = geometry.row_height - 1.0;
    let grid_step = geometry.surface_width / 10.0;
    let bar_h = geometry.row_height - 4.0;
    let ph_h = h - OVERVIEW_H;
    let phhead_top = OVERVIEW_H - 6.0;
    format!(
        "
  html,body {{ margin:0; padding:0; width:{w}px; height:{h}px;
              font-family:sans-serif; color:{INK}; }}
  div {{ position:absolute; }}
  .desktop {{ left:0; top:0; width:{w}px; height:{h}px; background:{DESKTOP}; }}
  .ov {{ top:0; height:{OVERVIEW_H}px; background:{SURFACE_LO}; }}
  .ovlabel {{ left:8px; top:4px; font-size:9px; color:{DIM}; }}
  .ovborder {{ top:1px; height:{ov_border_h}px; border:1px solid #d8d8d8; }}
  .ovbar {{ height:3px; }}
  .ruler {{ left:0; width:{w}px; height:{RULER_H}px; background:{SURFACE_HI}; }}
  .rulerlabel {{ left:8px; font-size:9px; color:#c0c0c0; }}
  .tick {{ width:1px; height:5px; background:#6a6a6a; }}
  .ticklabel {{ font-size:8px; color:{RULER}; }}
  .locator {{ left:0; width:{w}px; height:{LOCATOR_H}px; background:{SURFACE};
              border-bottom:1px solid {CONTRAST}; }}
  .row {{ left:0; width:{w}px; height:{row_h}px; background:{SURFACE};
          border-bottom:1px solid {CONTRAST};
          background-image:repeating-linear-gradient(to right,
            #262626 0px, #262626 1px, transparent 1px, transparent {grid_step}px); }}
  .sel {{ background:#414141; }}
  .tri {{ left:10px; width:0; height:0; border-left:5px solid #929292;
          border-top:4px solid transparent; border-bottom:4px solid transparent; }}
  .dot {{ left:11px; width:2px; height:2px; background:#929292; }}
  .llabel {{ left:20px; font-size:9px; color:{INK}; }}
  .plabel {{ left:31px; font-size:9px; color:{DIM}; }}
  .tg {{ width:13px; height:12px; background:#2f2f2f; color:{RULER};
         font-size:8px; text-align:center; border:1px solid {SURFACE_HI}; }}
  .bar {{ height:{bar_h}px; color:{BAR_INK}; font-size:9px; padding-left:6px;
          overflow:hidden; white-space:nowrap; }}
  .selbar {{ border:1px solid #ffffff; }}
  .key {{ width:8px; height:8px; background:{INK}; transform:rotate(45deg); }}
  .selkey {{ background:{ACCENT}; }}
  .vsep {{ top:0; width:1px; height:{h}px; background:{CONTRAST}; }}
  .ph {{ top:{OVERVIEW_H}px; width:1px; height:{ph_h}px; background:#e7e7e7; }}
  .phhead {{ top:{phhead_top}px; width:9px; height:6px; background:#e7e7e7; }}
"
    )
}

/// ui_mock.rs:75-186 の写し。
fn body(
    geometry: &TimelineGeometry,
    rows: &[TimelineRow],
    primary: Option<LayerId>,
    playhead: RationalTime,
) -> String {
    let sidebar = geometry.sidebar_width;
    let surface_w = geometry.surface_width;
    let mut b = String::new();

    // ---- 席全体の下地 ----
    // timeline_egui/mod.rs:79 の rect_filled(rect, DESKTOP) に相当。
    // mockは body の背景色で代用していたが、Blitzでは body の背景が
    // viewport全面を不透明で塗り、Stageの上へ重ねられなくなる
    // (blitz-paint-0.3.0-beta.1/src/render.rs:127-160)。要素側へ移す。
    b.push_str(r#"<div class="desktop"></div>"#);

    // ---- overview 帯 ----
    b.push_str(&format!(
        r#"<div class="ov" style="left:{sidebar}px;width:{surface_w}px"></div>
           <div class="ovlabel">overview</div>
           <div class="ovborder" style="left:{}px;width:{}px"></div>"#,
        sidebar + 1.0,
        surface_w - 3.0
    ));
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
            r#"<div class="ovbar" style="left:{}px;top:{}px;width:{}px;background:{}"></div>"#,
            geometry.x_at(start),
            4.0 + i as f64 * 4.0,
            (geometry.x_at(end) - geometry.x_at(start)).max(1.0),
            PALETTE[r.palette_slot]
        ));
    }

    // ---- ruler 帯 ----
    b.push_str(&format!(
        r#"<div class="ruler" style="top:{}px"></div><div class="rulerlabel" style="top:{}px">Inbox</div>"#,
        OVERVIEW_H + 1.0,
        OVERVIEW_H + 3.0
    ));
    for i in 0..=10 {
        let x = geometry.x_at(i as f64 / 10.0);
        b.push_str(&format!(
            r#"<div class="tick" style="left:{x}px;top:{}px"></div>
               <div class="ticklabel" style="left:{}px;top:{}px">{i}s</div>"#,
            OVERVIEW_H + RULER_H - 5.0,
            x + 3.0,
            OVERVIEW_H + 3.0
        ));
    }
    // ---- locator 帯 ----
    b.push_str(&format!(
        r#"<div class="locator" style="top:{}px"></div>"#,
        OVERVIEW_H + RULER_H + 1.0
    ));

    // ---- 行 ----
    // 選択行の決め方は timeline_egui/clip_band.rs:19-29 の写し。
    let selected_index = primary.as_ref().and_then(|layer| {
        rows.iter()
            .position(|row| row.property.is_none() && row.layer == *layer)
    });
    for (i, r) in rows.iter().enumerate() {
        let y = geometry.rows_top + i as f64 * geometry.row_height;
        if y >= geometry.height {
            break;
        }
        let cy = y + geometry.row_height * 0.5;
        let selected = r.property.is_none() && selected_index == Some(i);
        b.push_str(&format!(
            r#"<div class="row{}" style="top:{y}px"></div>"#,
            if selected { " sel" } else { "" }
        ));
        // sidebar 側
        if r.property.is_some() {
            b.push_str(&format!(
                r#"<div class="dot" style="top:{}px"></div>
                   <div class="plabel" style="top:{}px">{}</div>"#,
                cy - 1.0,
                cy - 5.0,
                escape(&r.label)
            ));
        } else {
            b.push_str(&format!(
                r#"<div class="tri" style="top:{}px"></div>
                   <div class="llabel" style="top:{}px">{}</div>
                   <div class="tg" style="left:{}px;top:{}px">M</div>
                   <div class="tg" style="left:{}px;top:{}px">S</div>"#,
                cy - 4.0,
                cy - 5.0,
                escape(&r.label),
                sidebar - 48.0,
                cy - 6.0,
                sidebar - 31.0,
                cy - 6.0
            ));
        }
        // clip バー
        if r.property.is_none() {
            if let (Some(start), Some(end)) = (r.start, r.end) {
                if end > start {
                    let x0 = geometry.x_at(start);
                    let x1 = geometry.x_at(end);
                    b.push_str(&format!(
                        r#"<div class="bar{}" style="left:{x0}px;top:{}px;width:{}px;background:{}">{}</div>"#,
                        if selected { " selbar" } else { "" },
                        y + 1.0,
                        (x1 - x0).max(1.0),
                        PALETTE[r.palette_slot],
                        escape(&r.label)
                    ));
                }
            }
        }
        // key ダイヤ
        for (k, kx) in r.keys.iter().enumerate() {
            b.push_str(&format!(
                r#"<div class="key{}" style="left:{}px;top:{}px"></div>"#,
                if selected && k == 0 { " selkey" } else { "" },
                geometry.x_at(*kx) - 4.0,
                cy - 4.0
            ));
        }
    }

    // ---- surface 境界線と playhead ----
    let playhead_x = geometry.x_at(geometry.playhead_fraction(playhead));
    b.push_str(&format!(
        r#"<div class="vsep" style="left:{sidebar}px"></div>
           <div class="ph" style="left:{playhead_x}px"></div>
           <div class="phhead" style="left:{}px"></div>"#,
        playhead_x - 4.0
    ));
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
        timeline_html(
            &document,
            Some(&projection),
            None,
            RationalTime::ZERO,
            1000.0,
            460.0,
        )
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
            ".desktop", ".ov", ".ovlabel", ".ovborder", ".ovbar", ".ruler", ".rulerlabel", ".tick",
            ".ticklabel", ".locator", ".row", ".sel", ".tri", ".dot", ".llabel", ".plabel", ".tg",
            ".bar", ".selbar", ".key", ".selkey", ".vsep", ".ph", ".phhead",
        ] {
            assert!(html.contains(class), "mockのclass {class} が無い");
        }
    }

    /// 出力に現れる `名前:` を全部拾う。CSS宣言もstyle属性も同じ形なので同じ走査で足りる。
    fn declared_properties(source: &str) -> Vec<String> {
        let bytes: Vec<char> = source.chars().collect();
        let mut found = Vec::new();
        for (index, ch) in bytes.iter().enumerate() {
            if *ch != ':' {
                continue;
            }
            let mut start = index;
            while start > 0 && (bytes[start - 1].is_ascii_lowercase() || bytes[start - 1] == '-') {
                start -= 1;
            }
            if start == index {
                continue;
            }
            let name: String = bytes[start..index].iter().collect();
            if !found.contains(&name) {
                found.push(name);
            }
        }
        found
    }

    /// NEGATIVE ORACLE: probeで一度も使っていないCSSプロパティを持ち込まない。
    #[test]
    fn uses_only_css_properties_proven_in_the_probe() {
        const PROBE: &str = concat!(
            include_str!("../../../../spikes/blitz-probe/src/bin/ui_mock.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/timeline_ux.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/browser_panel.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/diff_update.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/offscreen.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/texture_host.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/texture_mode.rs"),
            include_str!("../../../../spikes/blitz-probe/src/bin/custom_widget.rs"),
        );
        let html = sample_html();
        let probe_properties = declared_properties(PROBE);
        for property in declared_properties(&html) {
            assert!(
                probe_properties.contains(&property),
                "{property} が spikes/blitz-probe で一度も使われていない"
            );
        }
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
