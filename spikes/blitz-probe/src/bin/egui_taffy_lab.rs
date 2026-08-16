//! P13: モックのTimeline構造を、Blitz抜きで egui + egui_taffy に組めるか。
//!
//! 目的は見た目の完成ではなく、**構造が表現できるかどうか**の1点。
//! 確かめること:
//!   1. `grid-template-columns: 196px minmax(0,1fr)` 相当（固定列＋可変列）
//!   2. 行の積み上げ（layer 24px / property 20px の混在）
//!   3. 密な面をleafとして受け取り、その矩形へ直接描けるか（clip/keyの居場所）
//!   4. grid（KEY TOOLSの3列）
//!
//! 色と寸法はモックのCSSから**手で写している**。これがまさに、後で
//! HTML/CSSから生成させたい「値」の部分。ここでは構造の検証が目的なので手写しでよい。
//!
//! 使い方:
//!   egui_taffy_lab                 … 窓を開けて触る
//!   egui_taffy_lab out.bmp         … 4フレーム目を撮って終了（比較用）

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind, Vec2};
use egui_taffy::{taffy, tui, TuiBuilderLogic};
use taffy::prelude::{auto, length, percent};

// ---- モックのCSSから写した値 ----
const BG: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const HEAD: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const OVERVIEW: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
const CELL: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
const CELL_CHILD: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
const CELL_PROP: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const CELL_SELECTED: Color32 = Color32::from_rgb(0x42, 0x42, 0x42);
const TRACK_A: Color32 = Color32::from_rgb(0x37, 0x37, 0x37);
const TRACK_B: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
const RULE: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const INK: Color32 = Color32::from_rgb(0xd4, 0xd4, 0xd4);
const DIM: Color32 = Color32::from_rgb(0x8d, 0x8d, 0x8d);
const ACCENT: Color32 = Color32::from_rgb(0xe9, 0xcf, 0x72);
const TOOLS: Color32 = Color32::from_rgb(0x34, 0x34, 0x34);
const KEY_IDLE: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);

const RAIL_W: f32 = 196.0;
const ROW_H: f32 = 24.0;
const PROP_H: f32 = 20.0;
const TOOLS_W: f32 = 150.0;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Group,
    Child,
    Root,
    Property,
}

struct Row {
    kind: Kind,
    depth: usize,
    name: &'static str,
    /// (left%, width%, 色) — clip帯。propertyRowは持たない
    bar: Option<(f32, f32, Color32)>,
    /// キーの位置(%)
    keys: &'static [f32],
    selected: bool,
}

fn rows() -> Vec<Row> {
    vec![
        Row { kind: Kind::Group, depth: 0, name: "Title scene", bar: Some((4.0, 91.0, Color32::from_rgb(0x45, 0x42, 0x38))), keys: &[], selected: true },
        Row { kind: Kind::Property, depth: 1, name: "Transform", bar: None, keys: &[18.0, 63.0], selected: false },
        Row { kind: Kind::Property, depth: 1, name: "Opacity", bar: None, keys: &[12.0, 43.0, 78.0], selected: false },
        Row { kind: Kind::Child, depth: 1, name: "Shared left", bar: Some((4.0, 38.0, Color32::from_rgb(0x65, 0x75, 0x8c))), keys: &[], selected: false },
        Row { kind: Kind::Property, depth: 2, name: "Position", bar: None, keys: &[8.0, 32.0], selected: false },
        Row { kind: Kind::Property, depth: 2, name: "Scale", bar: None, keys: &[16.0, 37.0], selected: false },
        Row { kind: Kind::Child, depth: 1, name: "Reference text", bar: Some((34.0, 43.0, Color32::from_rgb(0x79, 0x67, 0x7d))), keys: &[], selected: false },
        Row { kind: Kind::Property, depth: 2, name: "Opacity", bar: None, keys: &[40.0, 71.0], selected: false },
        Row { kind: Kind::Child, depth: 1, name: "Shared right", bar: Some((69.0, 26.0, Color32::from_rgb(0x85, 0x64, 0x5f))), keys: &[], selected: false },
        Row { kind: Kind::Property, depth: 2, name: "Rotation", bar: None, keys: &[71.0, 82.0, 92.0], selected: false },
        Row { kind: Kind::Root, depth: 0, name: "Background", bar: Some((0.0, 84.0, Color32::from_rgb(0x65, 0x7b, 0x65))), keys: &[], selected: false },
        Row { kind: Kind::Root, depth: 0, name: "starter-tone.wav", bar: Some((7.0, 78.0, Color32::from_rgb(0x88, 0x76, 0x4e))), keys: &[], selected: false },
    ]
}

fn main() -> eframe::Result<()> {
    let shot = std::env::args().nth(1);
    eframe::run_native(
        "P13: egui + egui_taffy でモックの構造を組む",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 500.0]),
            ..Default::default()
        },
        Box::new(move |_cc| {
            // 記号の豆腐を消す。製品側 `motolii-ui/src/egui_fonts.rs` と同じことを、
            // 隔離workspaceなので写している(このcrateはmotolii-uiへ依存しない)。
            let mut fonts = egui::FontDefinitions::default();
            if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                if !f.iter().any(|n| n == "Hack") {
                    f.push("Hack".to_owned());
                }
            }
            _cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(App {
                shot,
                frame: 0,
                hovered: None,
            }))
        }),
    )
}

struct App {
    shot: Option<String>,
    frame: u32,
    hovered: Option<String>,
}

/// 行の左セル（OBJECT列）。ここは普通のUIなのでtaffyに任せる。
fn layer_cell(tui: &mut egui_taffy::Tui, row: &Row) {
    let bg = match row.kind {
        Kind::Property => CELL_PROP,
        Kind::Child => CELL_CHILD,
        _ if row.selected => CELL_SELECTED,
        _ => CELL,
    };
    tui.style(taffy::Style {
        size: taffy::Size {
            width: length(RAIL_W),
            height: percent(1.0),
        },
        ..Default::default()
    })
    .ui(|ui| {
        let rect = ui.max_rect();
        let p = ui.painter();
        p.rect_filled(rect, CornerRadius::ZERO, bg);
        p.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, RULE),
        );
        p.line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(0x09, 0x09, 0x09)),
        );
        if row.selected {
            p.rect_filled(
                Rect::from_min_size(rect.left_top(), Vec2::new(3.0, rect.height())),
                CornerRadius::ZERO,
                ACCENT,
            );
        }

        let indent = 8.0 + row.depth as f32 * 14.0;
        let cy = rect.center().y;

        // 開閉の三角(groupだけ)
        if row.kind == Kind::Group {
            p.text(
                egui::pos2(rect.left() + 6.0, cy),
                Align2::LEFT_CENTER,
                "▾",
                FontId::proportional(11.0),
                DIM,
            );
        }
        // 種別のしるし
        if row.kind != Kind::Property {
            let icon = Rect::from_center_size(
                egui::pos2(rect.left() + indent + 12.0, cy),
                Vec2::splat(9.0),
            );
            let color = match row.kind {
                Kind::Group => ACCENT,
                _ => Color32::from_rgb(0x72, 0x92, 0x98),
            };
            p.rect_filled(icon, CornerRadius::same(2), color);
        } else {
            let chip = Rect::from_center_size(
                egui::pos2(rect.left() + indent + 10.0, cy),
                Vec2::new(4.0, 11.0),
            );
            p.rect_filled(chip, CornerRadius::same(2), Color32::from_rgb(0xcf, 0x75, 0x6d));
        }

        let (font, color) = if row.kind == Kind::Property {
            (FontId::proportional(10.0), Color32::from_rgb(0xa8, 0xa8, 0xa8))
        } else {
            (FontId::proportional(11.0), INK)
        };
        p.text(
            egui::pos2(rect.left() + indent + 26.0, cy),
            Align2::LEFT_CENTER,
            row.name,
            font,
            color,
        );

        // M / S
        if row.kind != Kind::Property {
            for (i, label) in ["M", "S"].iter().enumerate() {
                let b = Rect::from_center_size(
                    egui::pos2(rect.right() - 30.0 + i as f32 * 18.0, cy),
                    Vec2::new(16.0, 16.0),
                );
                p.rect_filled(b, CornerRadius::ZERO, BG);
                p.rect_stroke(
                    b,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::from_rgb(0x51, 0x51, 0x51)),
                    StrokeKind::Inside,
                );
                p.text(
                    b.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(9.0),
                    Color32::from_rgb(0xaa, 0xaa, 0xaa),
                );
            }
        }
    });
}

/// 行の時間面。**ここが密な面**。leafの矩形を受け取って自分で描く。
fn row_track(tui: &mut egui_taffy::Tui, row: &Row) -> Option<String> {
    tui.style(taffy::Style {
        flex_grow: 1.0,
        size: taffy::Size {
            width: auto(),
            height: percent(1.0),
        },
        ..Default::default()
    })
    .ui(|ui| {
        let rect = ui.max_rect();
        let p = ui.painter();

        // 時間グリッド(CSSのrepeating-linear-gradient相当)
        p.rect_filled(rect, CornerRadius::ZERO, TRACK_A);
        let step = rect.width() / 8.0;
        for i in 0..=8 {
            let x = rect.left() + i as f32 * step;
            p.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(1.0, TRACK_B),
            );
        }
        p.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, RULE),
        );

        let mut hit = None;
        let pointer = ui.ctx().pointer_latest_pos();

        if let Some((left, width, color)) = row.bar {
            let bar = Rect::from_min_size(
                egui::pos2(
                    rect.left() + rect.width() * left / 100.0,
                    rect.top() + 3.0,
                ),
                Vec2::new(rect.width() * width / 100.0, rect.height() - 7.0),
            );
            p.rect_filled(bar, CornerRadius::ZERO, color);
            p.rect_stroke(
                bar,
                CornerRadius::ZERO,
                Stroke::new(1.0, Color32::from_rgb(0x17, 0x17, 0x17)),
                StrokeKind::Inside,
            );
            if row.selected {
                p.rect_stroke(
                    bar.shrink(1.0),
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::from_rgb(0xf3, 0xe4, 0xa2)),
                    StrokeKind::Inside,
                );
            }
            if pointer.is_some_and(|q| bar.contains(q)) {
                hit = Some(format!("clip: {}", row.name));
            }
        }

        for k in row.keys {
            let c = egui::pos2(rect.left() + rect.width() * k / 100.0, rect.center().y);
            let d = 4.0;
            let pts = vec![
                egui::pos2(c.x, c.y - d),
                egui::pos2(c.x + d, c.y),
                egui::pos2(c.x, c.y + d),
                egui::pos2(c.x - d, c.y),
            ];
            let near = pointer.is_some_and(|q| (q - c).length() < d + 2.0);
            p.add(egui::Shape::convex_polygon(
                pts,
                if near { ACCENT } else { KEY_IDLE },
                Stroke::new(1.0, Color32::from_rgb(0xee, 0xee, 0xee)),
            ));
            if near {
                hit = Some(format!("key: {} @{k}%", row.name));
            }
        }
        hit
    })
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();
        ui.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(3).unwrap());
        ui.global_style_mut(|s| s.wrap_mode = Some(egui::TextWrapMode::Extend));

        let data = rows();
        let mut hovered = None;

        tui(ui, ui.id().with("timeline"))
            .reserve_available_space()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                size: taffy::Size {
                    width: percent(1.0),
                    height: percent(1.0),
                },
                ..Default::default()
            })
            .show(|tui| {
                // ---- ヘッダ 34px ----
                tui.style(bar_style(34.0)).ui(|ui| {
                    let rect = ui.max_rect();
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::ZERO, HEAD);
                    p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, RULE));
                    p.text(egui::pos2(rect.left() + 10.0, rect.center().y), Align2::LEFT_CENTER,
                        "Timeline", FontId::proportional(15.0), INK);
                    p.text(egui::pos2(rect.left() + 82.0, rect.center().y), Align2::LEFT_CENTER,
                        "LAYERS", FontId::proportional(9.0), DIM);
                    p.text(egui::pos2(rect.right() - 10.0, rect.center().y), Align2::RIGHT_CENTER,
                        "00:00:04", FontId::monospace(11.0), ACCENT);
                });

                // ---- overview 30px ----
                tui.style(bar_style(30.0)).ui(|ui| {
                    let rect = ui.max_rect();
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::ZERO, OVERVIEW);
                    p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, RULE));
                    p.text(egui::pos2(rect.left() + 10.0, rect.center().y), Align2::LEFT_CENTER,
                        "ARRANGEMENT", FontId::proportional(9.0), DIM);
                    let track = Rect::from_min_max(
                        egui::pos2(rect.left() + 100.0, rect.top() + 8.0),
                        egui::pos2(rect.right() - 10.0, rect.bottom() - 8.0));
                    p.rect_stroke(track, CornerRadius::ZERO,
                        Stroke::new(1.0, Color32::from_rgb(0x70, 0x70, 0x70)), StrokeKind::Inside);
                });

                // ---- workspace: arrangement + KEY TOOLS ----
                tui.style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Row,
                    flex_grow: 1.0,
                    size: taffy::Size { width: percent(1.0), height: auto() },
                    ..Default::default()
                })
                .add(|tui| {
                    // arrangement
                    tui.style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        flex_grow: 1.0,
                        ..Default::default()
                    })
                    .add(|tui| {
                        // 見出し行: 固定196px + 可変
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            size: taffy::Size { width: percent(1.0), height: length(27.0) },
                            flex_shrink: 0.0,
                            ..Default::default()
                        })
                        .add(|tui| {
                            tui.style(taffy::Style {
                                size: taffy::Size { width: length(RAIL_W), height: percent(1.0) },
                                flex_shrink: 0.0,
                                ..Default::default()
                            })
                            .ui(|ui| {
                                let rect = ui.max_rect();
                                let p = ui.painter();
                                p.rect_filled(rect, CornerRadius::ZERO, CELL);
                                p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, RULE));
                                p.text(egui::pos2(rect.left() + 8.0, rect.center().y), Align2::LEFT_CENTER,
                                    "OBJECT", FontId::proportional(8.0), Color32::from_rgb(0x8f,0x8f,0x8f));
                                p.text(egui::pos2(rect.right() - 8.0, rect.center().y), Align2::RIGHT_CENTER,
                                    "M  S", FontId::proportional(8.0), Color32::from_rgb(0x8f,0x8f,0x8f));
                            });
                            tui.style(taffy::Style { flex_grow: 1.0, ..Default::default() }).ui(|ui| {
                                let rect = ui.max_rect();
                                let p = ui.painter();
                                p.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(0x2a,0x2a,0x2a));
                                p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, RULE));
                                for i in 0..=8 {
                                    let x = rect.left() + rect.width() * i as f32 / 8.0;
                                    p.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                        Stroke::new(1.0, Color32::from_rgb(0x44,0x44,0x44)));
                                    p.text(egui::pos2(x + 4.0, rect.bottom() - 6.0), Align2::LEFT_BOTTOM,
                                        format!("0:{:02}", i * 2), FontId::monospace(9.0), DIM);
                                }
                            });
                        });

                        // 行の積み上げ
                        for row in &data {
                            let h = if row.kind == Kind::Property { PROP_H } else { ROW_H };
                            let hit = tui
                                .style(taffy::Style {
                                    flex_direction: taffy::FlexDirection::Row,
                                    size: taffy::Size { width: percent(1.0), height: length(h) },
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                })
                                .add(|tui| {
                                    layer_cell(tui, row);
                                    row_track(tui, row)
                                });
                            if hit.is_some() {
                                hovered = hit;
                            }
                        }

                        // 余白
                        tui.style(taffy::Style { flex_grow: 1.0, ..Default::default() }).ui(|ui| {
                            ui.painter().rect_filled(ui.max_rect(), CornerRadius::ZERO, BG);
                        });
                    });

                    // KEY TOOLS: gridを使う
                    tui.style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        size: taffy::Size { width: length(TOOLS_W), height: auto() },
                        flex_shrink: 0.0,
                        padding: taffy::Rect { left: length(8.0), right: length(8.0), top: length(8.0), bottom: length(8.0) },
                        gap: taffy::Size { width: length(0.0), height: length(6.0) },
                        ..Default::default()
                    })
                    .add_with_background_color(|tui| {
                        let tools_rect = tui.egui_ui().max_rect();
                        tui.egui_ui_mut()
                            .painter()
                            .rect_filled(tools_rect, CornerRadius::ZERO, TOOLS);
                        tui.style(taffy::Style { size: taffy::Size { width: percent(1.0), height: length(14.0) }, ..Default::default() })
                            .ui(|ui| {
                                ui.painter().text(ui.max_rect().left_center(), Align2::LEFT_CENTER,
                                    "KEY TOOLS", FontId::proportional(8.0), Color32::from_rgb(0x77,0x77,0x77));
                            });
                        // 3列grid
                        tui.style(taffy::Style {
                            display: taffy::Display::Grid,
                            grid_template_columns: vec![taffy::prelude::fr(1.0), taffy::prelude::fr(1.0), taffy::prelude::fr(1.0)],
                            gap: taffy::Size { width: length(3.0), height: length(3.0) },
                            size: taffy::Size { width: percent(1.0), height: auto() },
                            ..Default::default()
                        })
                        .add(|tui| {
                            for label in ["←", "↔", "→", "Hold", "Smooth"] {
                                tui.style(taffy::Style {
                                    size: taffy::Size { width: auto(), height: length(24.0) },
                                    ..Default::default()
                                })
                                .ui(|ui| {
                                    let rect = ui.max_rect();
                                    let p = ui.painter();
                                    p.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(0x33,0x33,0x33));
                                    p.rect_stroke(rect, CornerRadius::ZERO,
                                        Stroke::new(1.0, Color32::from_rgb(0x4a,0x4a,0x4a)), StrokeKind::Inside);
                                    p.text(rect.center(), Align2::CENTER_CENTER, label,
                                        FontId::proportional(9.0), Color32::from_rgb(0xcf,0xcf,0xcf));
                                });
                            }
                        });
                    });
                });

                // ---- footer 26px ----
                tui.style(bar_style(26.0)).ui(|ui| {
                    let rect = ui.max_rect();
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::ZERO, OVERVIEW);
                    p.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, RULE));
                    p.text(egui::pos2(rect.left() + 10.0, rect.center().y), Align2::LEFT_CENTER,
                        "Selected Group: Title scene · 3 children", FontId::proportional(9.0), Color32::from_rgb(0x77,0x77,0x77));
                });
            });

        self.hovered = hovered;
        if let Some(h) = &self.hovered {
            ctx.debug_painter().text(
                egui::pos2(300.0, 8.0),
                Align2::LEFT_TOP,
                h,
                FontId::proportional(11.0),
                ACCENT,
            );
        }

        // ---- 比較用のスクリーンショット ----
        self.frame += 1;
        if self.frame % 60 == 1 {
            eprintln!("P13 frame={} shot={:?}", self.frame, self.shot);
        }
        if let Some(path) = self.shot.clone() {
            // egui_taffy はレイアウト測定のために pass を discard する。
            // discard された pass の viewport command は捨てられるので、
            // 1回だけ送ると届かない。落ち着いてから毎フレーム送る。
            if self.frame >= 30 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let shot = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = shot {
                write_bmp(&path, &image);
                eprintln!("P13 出力: {path} ({}x{})", image.size[0], image.size[1]);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn bar_style(h: f32) -> taffy::Style {
    taffy::Style {
        size: taffy::Size {
            width: percent(1.0),
            height: length(h),
        },
        flex_shrink: 0.0,
        ..Default::default()
    }
}

fn write_bmp(path: &str, image: &egui::ColorImage) {
    let (w, h) = (image.size[0] as u32, image.size[1] as u32);
    let row_bytes = (w * 3).next_multiple_of(4);
    let pixels = (row_bytes * h) as usize;
    let mut out = Vec::with_capacity(54 + pixels);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54u32 + pixels as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..6 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    for y in (0..h).rev() {
        let mut written = 0;
        for x in 0..w {
            let c = image.pixels[(y * w + x) as usize];
            out.push(c.b());
            out.push(c.g());
            out.push(c.r());
            written += 3;
        }
        while written < row_bytes {
            out.push(0);
            written += 1;
        }
    }
    std::fs::write(path, out).expect("write bmp");
}
