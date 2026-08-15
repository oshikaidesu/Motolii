//! 時間面の密な部分(clip バーと key ダイヤ)を **DOM 1ノード**で描く custom widget。
//!
//! スプレッドシートの構造 — **密な面は自前描画、DOM は chrome だけ**。
//! P8 実測(`docs/reviews/2026-08-15-blitz-ui-runtime-probe.md`)がその根拠:
//!
//! | 描き方 | 天井 | 単価 |
//! |---|---|---|
//! | DOM ノード | 約3,600 | `resolve` 4.0µs/個 |
//! | 描画プリミティブ | 約20,000 | 0.73µs/個 |
//!
//! 行(`.row`)と playhead は **DOM に残す**。本数がトラック数で有界であり、
//! CSS で触れる利点(`timeline.css`)の方が大きいため。ここへ移したのは
//! **clip と key** — 素材が増えた分だけ増える2種類だけ。
//!
//! 座標はこの widget の箱の左上が原点。箱は `html.rs` が
//! `left = sidebar_width / top = rows_top / width = surface_width` で置く。

use anyrender::{PaintScene, Scene};
use blitz_dom::node::{ComputedStyles, Widget};
use blitz_traits::events::UiEvent;
use vello_common::kurbo::{Affine, Point, Rect};
use vello_common::peniko::{Color, Fill};

use super::rows::TimelineRow;
use super::theme;

/// `#rrggbb` を色へ。**値の出所は `theme.rs`(CSSと同じ文字列)** で、ここでは決めない。
/// 読めない文字列は白へ落とす(描画を止めない。色が違えば絵で分かる)。
fn rgb(hex: &str) -> Color {
    let raw = hex.trim_start_matches('#');
    let parse = |range: std::ops::Range<usize>| -> Option<u8> {
        u8::from_str_radix(raw.get(range)?, 16).ok()
    };
    match (parse(0..2), parse(2..4), parse(4..6)) {
        (Some(r), Some(g), Some(b)) => Color::from_rgb8(r, g, b),
        _ => Color::from_rgb8(255, 255, 255),
    }
}

pub(super) struct TimelineSurface {
    rows: Vec<TimelineRow>,
    row_height: f64,
    /// 選択中の clip 行。`html.rs` が出した index をそのまま受ける。
    selected_index: Option<usize>,
}

impl TimelineSurface {
    pub(super) fn new(
        rows: Vec<TimelineRow>,
        row_height: f64,
        selected_index: Option<usize>,
    ) -> Self {
        Self {
            rows,
            row_height,
            selected_index,
        }
    }
}

impl Widget for TimelineSurface {
    /// 面の中の hit と掴みはここで受ける(C2で配線する)。
    /// **面の中は DOM から見て1ノード**なので、内側の当たり判定は
    /// 描画と同じ式(この file の座標)を共有する。二重持ちにはならない。
    fn handle_event(&mut self, _event: &UiEvent) {}

    fn paint(
        &mut self,
        _ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene {
        let mut scene = Scene::new();
        let width = width as f64;
        let height = height as f64;
        // **`width`/`height` は物理px、`row_height` は CSS px。** 掛け合わせないと
        // hidpi の面で行がずれる(scale=1 の dump では一致するので気付けない)。
        // 文書側の `Viewport::hidpi_scale` と同じ値がここへ来る。
        let row_height = self.row_height * scale;
        // bar の高さは `html.rs` の `.bar` と同じ式(row_height - 4)。
        let bar_h = (row_height - 4.0 * scale).max(1.0);
        let ink = rgb(theme::INK);
        let accent = rgb(theme::ACCENT);
        let white = Color::from_rgb8(255, 255, 255);

        for (index, row) in self.rows.iter().enumerate() {
            let y = index as f64 * row_height;
            if y >= height {
                break;
            }
            let cy = y + row_height * 0.5;
            let selected = row.property.is_none() && self.selected_index == Some(index);

            // ---- clip バー ----
            if row.property.is_none() {
                if let (Some(start), Some(end)) = (row.start, row.end) {
                    if end > start {
                        let x0 = start.clamp(0.0, 1.0) * width;
                        let x1 = end.clamp(0.0, 1.0) * width;
                        let top = y + scale;
                        let rect = Rect::new(x0, top, x1.max(x0 + 1.0), top + bar_h);
                        scene.fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            rgb(theme::PALETTE[row.palette_slot]),
                            None,
                            &rect,
                        );
                        // 選択枠。`.selbar` の 1px 白枠と同じ意味。
                        if selected {
                            for edge in [
                                Rect::new(rect.x0, rect.y0, rect.x1, rect.y0 + scale),
                                Rect::new(rect.x0, rect.y1 - scale, rect.x1, rect.y1),
                                Rect::new(rect.x0, rect.y0, rect.x0 + scale, rect.y1),
                                Rect::new(rect.x1 - scale, rect.y0, rect.x1, rect.y1),
                            ] {
                                scene.fill(Fill::NonZero, Affine::IDENTITY, white, None, &edge);
                            }
                        }
                    }
                }
            }

            // ---- key ダイヤ ----
            // `.key` は 8x8 を 45° 回した形。同じものを回転行列で描く。
            for (k, fraction) in row.keys.iter().enumerate() {
                let cx = fraction.clamp(0.0, 1.0) * width;
                let color = if selected && k == 0 { accent } else { ink };
                let half = 4.0 * scale;
                let square = Rect::new(cx - half, cy - half, cx + half, cy + half);
                scene.fill(
                    Fill::NonZero,
                    Affine::rotate_about(std::f64::consts::FRAC_PI_4, Point::new(cx, cy)),
                    color,
                    None,
                    &square,
                );
            }
        }
        scene
    }
}
