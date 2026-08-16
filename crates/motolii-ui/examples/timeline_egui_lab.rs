//! egui Timeline の Lab。**実際の Document を、実際の行モデルで、手で触る。**
//!
//! ここには偽のTimeline状態を置かない。行は `timeline_rows::rows()` が
//! 実 Document から毎フレーム作ったものであり、開閉はその `TimelineFoldState` である。
//! だから**この窓で見えていることは、そのまま製品の行モデルの挙動**である。
//!
//! 触れるもの:
//!   - グループ左端の `▸`/`▾` … 子レイヤーの開閉
//!   - `◇`/`◆`            … そのレイヤーのキーパラメータ行の開閉（子とは独立）
//!   - clip bar をドラッグ … 時間方向へ移動
//!
//!   - bar の左右端6px      … トリム（in / out）
//!   - `Cmd/Ctrl + Z` / `Shift+Z` … Undo / Redo
//!
//! **ドラッグは Document を実際に書き換える。** 経路は
//! `DocumentWriter::prepare_* -> apply_command(gesture, command)` で、
//! 1ドラッグ = 1 `GestureId` = 1 Undo 単位である。撤去された
//! `document_edit_runtime` を経由しない — D2 が既に薄い入口を持っていた。
//!
//! 実行: `cargo run --profile fast -p motolii-ui --example timeline_egui_lab`

use std::collections::HashMap;

use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter,
    GestureId, Group, ItemEnvelope, KeyframeId, LayerId, Track, TrackItem, Transform2D,
};
use std::sync::Arc;
use motolii_eval::Interp;
use motolii_ui::timeline_rows::{rows, ParamRef, RowKind, TimelineFoldState, TimelineRow};

// mock_tokens が timeline-library.html から出した値（面の大きさで動かないものだけ）
const BG: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const HEAD_BG: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const CELL: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
const CELL_CHILD: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
const CELL_PROP: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const TRACK_A: Color32 = Color32::from_rgb(0x37, 0x37, 0x37);
const TRACK_B: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
const RULE: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const INK: Color32 = Color32::from_rgb(0xd4, 0xd4, 0xd4);
const DIM: Color32 = Color32::from_rgb(0x8d, 0x8d, 0x8d);
const ACCENT: Color32 = Color32::from_rgb(0xe9, 0xcf, 0x72);
const KEY_IDLE: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);

const RAIL_W: f32 = 196.0;
const ROW_H: f32 = 24.0;
const PROP_H: f32 = 20.0;
const HEAD_H: f32 = 34.0;
const RULER_H: f32 = 27.0;

fn main() -> eframe::Result<()> {
    let shot = std::env::args().nth(1);
    eframe::run_native(
        "Timeline egui Lab — 実Documentを実行モデルで",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 460.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            motolii_ui::install_egui_symbol_fallback(&cc.egui_ctx);
            Ok(Box::new(Lab::new(shot)))
        }),
    )
}

/// 掴んでいるもの。**何を掴んだかで、出す command が変わる**
#[derive(Clone)]
enum Grab {
    /// **Group を掴んだら子も同じ差分で動く。**`targets` は動かす clip と、
    /// 掴んだ瞬間の開始時刻(秒)。Group 自身は clip ではないので入らない
    Move {
        layer: LayerId,
        grab_at: f32,
        targets: Vec<(LayerId, f32, f32)>,
    },
    TrimIn { layer: LayerId },
    TrimOut { layer: LayerId },
}

struct Lab {
    writer: DocumentWriter,
    document: Arc<Document>,
    fold: TimelineFoldState,
    drag: Option<(Grab, GestureId)>,
    names: HashMap<LayerId, String>,
    status: String,
    shot: Option<String>,
    frame: u32,
}

impl Lab {
    fn new(shot: Option<String>) -> Self {
        let (doc, names) = fixture();
        let catalog = Arc::new(
            motolii_plugin::reference::reference_catalog().expect("reference catalog"),
        );
        let writer = DocumentWriter::new(doc, catalog).expect("writer");
        let document = writer.snapshot();
        let mut fold = TimelineFoldState::default();
        // 最初から階層が見えているほうが、行モデルの確認になる
        for (&layer, name) in &names {
            match name.as_str() {
                // 子の軸
                "Title scene" => fold.open_children(layer),
                // パラメータの軸。両方が同時に効くことを最初の画面で見せる
                "Shared left" => fold.open_params(layer),
                _ => {}
            }
        }
        if let Some((&layer, _)) = names.iter().find(|(_, n)| n.as_str() == "Title scene") {
            fold.open_params(layer);
        }
        Self {
            writer,
            document,
            fold,
            drag: None,
            names,
            status: "arrow=children  diamond=key rows  drag=move  edges=trim  Cmd+Z=undo".to_owned(),
            shot,
            frame: 0,
        }
    }

    /// ポインタの時刻(秒)を、掴んでいるものに応じた D2 command にして適用する。
    ///
    /// **prepare_* が `Ok(None)` を返したら、それは「変化なし」であって失敗ではない。**
    /// 落ちた編集は status に出す。通ったことにしない。
    fn commit_drag(&mut self, at_seconds: f32) {
        let Some((grab, gesture)) = self.drag.clone() else {
            return;
        };
        let Some(time) = seconds_to_time(at_seconds) else {
            return;
        };

        // 掴んだものごとに、出す command の並びを作る。
        // **Move だけが複数になりうる**(Group は子をまとめて動かす)
        let mut prepared = Vec::new();
        match &grab {
            Grab::Move {
                grab_at, targets, ..
            } => {
                if targets.is_empty() {
                    self.status = "nothing to move (empty group)".to_owned();
                    return;
                }
                // **塊のまま動く。** 誰か1人でも端を越えるなら、全員そこで止まる。
                // 越えた者だけ置いていくと、Group の中の相対位置が壊れる
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                let floor = targets
                    .iter()
                    .map(|(_, start, _)| *start)
                    .fold(f32::INFINITY, f32::min);
                let headroom = targets
                    .iter()
                    .map(|(_, _, end)| comp - *end)
                    .fold(f32::INFINITY, f32::min);
                let delta = (at_seconds - grab_at).clamp(-floor, headroom.max(0.0));
                for (layer, start, _) in targets {
                    let Some(t) = seconds_to_time((start + delta).max(0.0)) else {
                        return;
                    };
                    prepared.push((*layer, self.writer.prepare_set_clip_start(*layer, t)));
                }
            }
            Grab::TrimIn { layer } => {
                prepared.push((*layer, self.writer.prepare_trim_clip_in(*layer, time)))
            }
            Grab::TrimOut { layer } => {
                prepared.push((*layer, self.writer.prepare_trim_clip_out(*layer, time)))
            }
        }

        let mut applied = 0usize;
        for (layer, result) in prepared {
            match result {
                Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                    Ok(()) => applied += 1,
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return;
                    }
                },
                Ok(None) => {}
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(layer));
                    return;
                }
            }
        }
        if applied > 0 {
            self.document = self.writer.snapshot();
            self.status = format!(
                "{} @ {at_seconds:.2}s  ({applied} clip)  undo {}",
                self.name(grab_layer(&grab)),
                self.writer.undo_len()
            );
        }
    }

    fn name(&self, layer: LayerId) -> &str {
        self.names.get(&layer).map(String::as_str).unwrap_or("?")
    }
}

impl eframe::App for Lab {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();

        // **毎フレーム、実Documentから行を作り直す。** 行のキャッシュを持たない
        let visible = rows(&self.document, &self.fold);
        let track_w_for_time = ui.available_rect_before_wrap().width() - RAIL_W;

        let full = ui.available_rect_before_wrap();
        let p = ui.painter().clone();
        p.rect_filled(full, CornerRadius::ZERO, BG);

        // ---- ヘッダ ----
        let head = Rect::from_min_size(full.min, Vec2::new(full.width(), HEAD_H));
        p.rect_filled(head, CornerRadius::ZERO, HEAD_BG);
        p.text(
            egui::pos2(head.left() + 10.0, head.center().y),
            Align2::LEFT_CENTER,
            "Timeline",
            FontId::proportional(15.0),
            INK,
        );
        p.text(
            egui::pos2(head.left() + 92.0, head.center().y),
            Align2::LEFT_CENTER,
            format!("{} rows", visible.len()),
            FontId::proportional(9.0),
            DIM,
        );
        p.text(
            egui::pos2(head.right() - 10.0, head.center().y),
            Align2::RIGHT_CENTER,
            &self.status,
            FontId::proportional(10.0),
            ACCENT,
        );

        // ---- ルーラ ----
        let ruler = Rect::from_min_size(
            egui::pos2(full.left(), head.bottom()),
            Vec2::new(full.width(), RULER_H),
        );
        let track_left = full.left() + RAIL_W;
        let track_w = full.right() - track_left;
        p.rect_filled(ruler, CornerRadius::ZERO, Color32::from_rgb(0x2a, 0x2a, 0x2a));
        for i in 0..=8 {
            let x = track_left + track_w * i as f32 / 8.0;
            p.line_segment(
                [egui::pos2(x, ruler.top()), egui::pos2(x, ruler.bottom())],
                Stroke::new(1.0, Color32::from_rgb(0x44, 0x44, 0x44)),
            );
            p.text(
                egui::pos2(x + 4.0, ruler.bottom() - 5.0),
                Align2::LEFT_BOTTOM,
                format!("0:{:02}", i * 2),
                FontId::monospace(9.0),
                DIM,
            );
        }
        p.line_segment(
            [ruler.left_bottom(), ruler.right_bottom()],
            Stroke::new(1.0, RULE),
        );

        // ---- 行 ----
        let mut y = ruler.bottom();
        let mut toggles: Vec<(LayerId, bool)> = Vec::new();

        for row in &visible {
            let h = match row.kind {
                RowKind::Object => ROW_H,
                RowKind::Property(_) => PROP_H,
            };
            let rect = Rect::from_min_size(egui::pos2(full.left(), y), Vec2::new(full.width(), h));
            let rail = Rect::from_min_size(rect.min, Vec2::new(RAIL_W, h));
            let track = Rect::from_min_max(egui::pos2(track_left, rect.top()), rect.max);

            // 左列
            let cell = match row.kind {
                RowKind::Property(_) => CELL_PROP,
                _ if row.depth > 0 => CELL_CHILD,
                _ => CELL,
            };
            p.rect_filled(rail, CornerRadius::ZERO, cell);
            p.line_segment(
                [rail.left_bottom(), rail.right_bottom()],
                Stroke::new(1.0, RULE),
            );
            p.line_segment(
                [rail.right_top(), rail.right_bottom()],
                Stroke::new(1.0, Color32::from_rgb(0x09, 0x09, 0x09)),
            );

            let indent = 8.0 + row.depth as f32 * 14.0;
            let cy = rail.center().y;

            // 子の開閉（三角）— 子を持つ行だけ
            if row.has_children {
                let hit = Rect::from_center_size(
                    egui::pos2(rail.left() + indent + 2.0, cy),
                    Vec2::splat(16.0),
                );
                let r = ui.interact(hit, ui.id().with(("fold", row.layer)), Sense::click());
                p.text(
                    hit.center(),
                    Align2::CENTER_CENTER,
                    if row.children_open { "▾" } else { "▸" },
                    FontId::proportional(11.0),
                    if r.hovered() { ACCENT } else { DIM },
                );
                if r.clicked() {
                    toggles.push((row.layer, true));
                }
            }

            match row.kind {
                RowKind::Object => {
                    let icon = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 20.0, cy),
                        Vec2::splat(9.0),
                    );
                    p.rect_filled(
                        icon,
                        CornerRadius::same(2),
                        if row.has_children {
                            ACCENT
                        } else {
                            Color32::from_rgb(0x72, 0x92, 0x98)
                        },
                    );
                    p.text(
                        egui::pos2(rail.left() + indent + 32.0, cy),
                        Align2::LEFT_CENTER,
                        self.name(row.layer),
                        FontId::proportional(11.0),
                        INK,
                    );

                    // キー行の開閉（◇/◆）— キーを持つ行だけ
                    let has_keys = !visible_params(&self.document, row.layer).is_empty();
                    if has_keys {
                        let hit = Rect::from_center_size(
                            egui::pos2(rail.right() - 52.0, cy),
                            Vec2::splat(16.0),
                        );
                        let r =
                            ui.interact(hit, ui.id().with(("params", row.layer)), Sense::click());
                        p.text(
                            hit.center(),
                            Align2::CENTER_CENTER,
                            if row.params_open { "◆" } else { "◇" },
                            FontId::proportional(11.0),
                            if row.params_open || r.hovered() {
                                ACCENT
                            } else {
                                DIM
                            },
                        );
                        if r.clicked() {
                            toggles.push((row.layer, false));
                        }
                    }

                    for (i, label) in ["M", "S"].iter().enumerate() {
                        let b = Rect::from_center_size(
                            egui::pos2(rail.right() - 30.0 + i as f32 * 18.0, cy),
                            Vec2::splat(16.0),
                        );
                        p.rect_stroke(
                            b,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, Color32::from_rgb(0x51, 0x51, 0x51)),
                            StrokeKind::Inside,
                        );
                        p.text(
                            b.center(),
                            Align2::CENTER_CENTER,
                            *label,
                            FontId::proportional(9.0),
                            Color32::from_rgb(0xaa, 0xaa, 0xaa),
                        );
                    }
                }
                RowKind::Property(param) => {
                    let chip = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 10.0, cy),
                        Vec2::new(4.0, 11.0),
                    );
                    p.rect_filled(chip, CornerRadius::same(2), param_color(param));
                    p.text(
                        egui::pos2(rail.left() + indent + 22.0, cy),
                        Align2::LEFT_CENTER,
                        param_label(param),
                        FontId::proportional(10.0),
                        Color32::from_rgb(0xa8, 0xa8, 0xa8),
                    );
                }
            }

            // 時間面
            p.rect_filled(track, CornerRadius::ZERO, TRACK_A);
            for i in 0..=8 {
                let x = track_left + track_w * i as f32 / 8.0;
                p.line_segment(
                    [egui::pos2(x, track.top()), egui::pos2(x, track.bottom())],
                    Stroke::new(1.0, TRACK_B),
                );
            }
            p.line_segment(
                [track.left_bottom(), track.right_bottom()],
                Stroke::new(1.0, RULE),
            );

            // **時間面の描画は track の中だけ。** 左列へはみ出すと M/S を覆う
            let p = p.with_clip_rect(track);
            match row.kind {
                RowKind::Object => {
                    if let Some((start, end)) = clip_span(&self.document, row.layer) {
                        let x0 = track_left + track_w * start / 16.0;
                        let x1 = track_left + track_w * end / 16.0;
                        let bar = Rect::from_min_max(
                            egui::pos2(x0, track.top() + 3.0),
                            egui::pos2(x1, track.bottom() - 4.0),
                        );
                        let r = ui.interact(
                            bar,
                            ui.id().with(("bar", row.layer)),
                            Sense::click_and_drag(),
                        );
                        let color = if row.has_children {
                            Color32::from_rgb(0x4c, 0x49, 0x3c)
                        } else {
                            Color32::from_rgb(0x65, 0x75, 0x8c)
                        };
                        p.rect_filled(
                            bar,
                            CornerRadius::ZERO,
                            if r.dragged() { ACCENT } else { color },
                        );
                        p.rect_stroke(
                            bar,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, Color32::from_rgb(0x17, 0x17, 0x17)),
                            StrokeKind::Inside,
                        );
                        if r.drag_started() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                // **何を掴んだかは端からの距離で決める。** 6px はモックの
                                // `.trimHandle{width:7px}` に合わせた値
                                let grab = if pos.x - bar.left() <= 6.0 {
                                    Grab::TrimIn { layer: row.layer }
                                } else if bar.right() - pos.x <= 6.0 {
                                    Grab::TrimOut { layer: row.layer }
                                } else {
                                    Grab::Move {
                                        layer: row.layer,
                                        grab_at: (pos.x - track_left) / track_w * 16.0,
                                        targets: movable_clips(&self.document, row.layer),
                                    }
                                };
                                let gesture = self.writer.begin_gesture();
                                self.drag = Some((grab, gesture));
                            }
                        }
                        if r.dragged() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                let at = ((pos.x - track_left) / track_w_for_time.max(1.0) * 16.0)
                                    .max(0.0);
                                self.commit_drag(at);
                            }
                        }
                        if r.drag_stopped() {
                            self.drag = None;
                        }
                    }
                }
                RowKind::Property(param) => {
                    for t in key_times(&self.document, row.layer, param) {
                        let x = track_left + track_w * t / 16.0;
                        let c = egui::pos2(x, track.center().y);
                        let d = 4.0;
                        p.add(egui::Shape::convex_polygon(
                            vec![
                                egui::pos2(c.x, c.y - d),
                                egui::pos2(c.x + d, c.y),
                                egui::pos2(c.x, c.y + d),
                                egui::pos2(c.x - d, c.y),
                            ],
                            KEY_IDLE,
                            Stroke::new(1.0, Color32::from_rgb(0xee, 0xee, 0xee)),
                        ));
                    }
                }
            }

            y += h;
        }

        for (layer, is_children) in toggles {
            if is_children {
                if self.fold.children_are_open(layer) {
                    self.fold.close_children(layer);
                } else {
                    self.fold.open_children(layer);
                }
            } else if self.fold.params_are_open(layer) {
                self.fold.close_params(layer);
            } else {
                self.fold.open_params(layer);
            }
        }

        // Undo / Redo。1ドラッグ = 1 GestureId なので、掴んで動かした分がまとめて戻る
        let (undo, redo) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift,
                i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift,
            )
        });
        if undo {
            match self.writer.undo() {
                Ok(()) => {
                    self.document = self.writer.snapshot();
                    self.status = format!("undo  ({} left)", self.writer.undo_len());
                }
                Err(error) => self.status = format!("undo rejected: {error}"),
            }
        } else if redo {
            match self.writer.redo() {
                Ok(()) => {
                    self.document = self.writer.snapshot();
                    self.status = format!("redo  ({} left)", self.writer.redo_len());
                }
                Err(error) => self.status = format!("redo rejected: {error}"),
            }
        }

        self.frame += 1;
        if let Some(path) = self.shot.clone() {
            if self.frame >= 20 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let image = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = image {
                write_bmp(&path, &image);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn grab_layer(grab: &Grab) -> LayerId {
    match grab {
        Grab::Move { layer, .. } | Grab::TrimIn { layer } | Grab::TrimOut { layer } => *layer,
    }
}

/// その layer を動かしたとき、実際に開始時刻が変わる clip を集める。
///
/// **Group 自身は clip ではないので、動くのは子孫の clip である。**
/// モックの「Group barを動かすと子barも同じ差分で追従する」をここで満たす。
fn movable_clips(document: &Document, layer: LayerId) -> Vec<(LayerId, f32, f32)> {
    fn collect(item: &TrackItem, out: &mut Vec<(LayerId, f32, f32)>) {
        match item {
            TrackItem::Clip(clip) => {
                let start = clip.start.as_seconds_f64() as f32;
                out.push((
                    clip.envelope.layer_id,
                    start,
                    start + clip.duration.as_seconds_f64() as f32,
                ));
            }
            TrackItem::Group(group) => {
                for child in &group.children {
                    collect(child, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(item) = find_item(document, layer) {
        collect(item, &mut out);
    }
    out
}

fn seconds_to_time(seconds: f32) -> Option<RationalTime> {
    RationalTime::try_new((seconds * 1000.0).round() as i64, 1000).ok()
}

fn param_label(p: ParamRef) -> &'static str {
    match p {
        ParamRef::Position => "Position",
        ParamRef::Anchor => "Anchor",
        ParamRef::Scale => "Scale",
        ParamRef::Rotation => "Rotation",
        ParamRef::Opacity => "Opacity",
    }
}

fn param_color(p: ParamRef) -> Color32 {
    match p {
        ParamRef::Position => Color32::from_rgb(0xcf, 0x75, 0x6d),
        ParamRef::Scale => Color32::from_rgb(0x75, 0xa9, 0x78),
        ParamRef::Rotation => Color32::from_rgb(0x77, 0x9b, 0xd0),
        ParamRef::Opacity => Color32::from_rgb(0xb1, 0x8e, 0xc0),
        ParamRef::Anchor => Color32::from_rgb(0xe1, 0xb8, 0x66),
    }
}

/// Document を歩いて、その layer の item を探す
fn find_item(document: &Document, layer: LayerId) -> Option<&TrackItem> {
    fn walk(items: &[TrackItem], layer: LayerId) -> Option<&TrackItem> {
        for item in items {
            let env = match item {
                TrackItem::Clip(c) => &c.envelope,
                TrackItem::Group(g) => &g.envelope,
            };
            if env.layer_id == layer {
                return Some(item);
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, layer) {
                    return Some(found);
                }
            }
        }
        None
    }
    document.tracks.iter().find_map(|t| walk(&t.items, layer))
}

fn visible_params(document: &Document, layer: LayerId) -> Vec<ParamRef> {
    find_item(document, layer)
        .map(motolii_ui::timeline_rows::keyed_params)
        .unwrap_or_default()
}

fn clip_span(document: &Document, layer: LayerId) -> Option<(f32, f32)> {
    match find_item(document, layer)? {
        TrackItem::Clip(c) => {
            let start = c.start.as_seconds_f64() as f32;
            Some((start, start + c.duration.as_seconds_f64() as f32))
        }
        // Group は子の範囲を包む
        TrackItem::Group(g) => {
            let mut span: Option<(f32, f32)> = None;
            for child in &g.children {
                if let TrackItem::Clip(c) = child {
                    let s = c.start.as_seconds_f64() as f32;
                    let e = s + c.duration.as_seconds_f64() as f32;
                    span = Some(match span {
                        Some((a, b)) => (a.min(s), b.max(e)),
                        None => (s, e),
                    });
                }
            }
            span
        }
    }
}

fn key_times(document: &Document, layer: LayerId, param: ParamRef) -> Vec<f32> {
    let Some(item) = find_item(document, layer) else {
        return Vec::new();
    };
    let env = match item {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    };
    let doc_param = match param {
        ParamRef::Position => &env.transform.position,
        ParamRef::Anchor => &env.transform.anchor,
        ParamRef::Scale => &env.transform.scale,
        ParamRef::Rotation => &env.transform.rotation,
        ParamRef::Opacity => &env.opacity,
    };
    match doc_param {
        DocParam::Keyframes(track) => track
            .keys()
            .iter()
            .map(|k| k.t.as_seconds_f64() as f32)
            .collect(),
        _ => Vec::new(),
    }
}

// ---- fixture: Group ひとつ、子3枚、兄弟2枚 ----

fn time(ms: i64) -> RationalTime {
    RationalTime::try_new(ms, 1000).expect("fixture time")
}

fn keys_at(document: &mut Document, seconds: &[f64], v: DocValue) -> DocParam {
    let mut track = DocKeyframeTrack::new();
    for s in seconds {
        let id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id"));
        track.insert(DocKeyframe {
            id,
            t: time((s * 1000.0) as i64),
            value: v.clone(),
            interp: Interp::Hold,
        });
    }
    DocParam::Keyframes(track)
}

fn make_clip(
    document: &mut Document,
    name: &str,
    start_s: f64,
    dur_s: f64,
    positions: &[f64],
    scales: &[f64],
) -> (TrackItem, LayerId, String) {
    let asset = document
        .assets
        .allocate(name, "video/mp4", &format!("{name}-hash"))
        .expect("asset");
    let layer = document.layers.allocate(name).expect("layer");
    let mut envelope = ItemEnvelope::new(layer);
    let mut transform = Transform2D::identity();
    if !positions.is_empty() {
        transform.position = keys_at(document, positions, DocValue::Vec2([0.0, 0.0]));
    }
    if !scales.is_empty() {
        transform.scale = keys_at(document, scales, DocValue::Vec2([1.0, 1.0]));
    }
    envelope.transform = transform;
    (
        TrackItem::Clip(Clip {
            envelope,
            start: time((start_s * 1000.0) as i64),
            duration: time((dur_s * 1000.0) as i64),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        }),
        layer,
        name.to_owned(),
    )
}

fn fixture() -> (Document, HashMap<LayerId, String>) {
    let mut document = Document::new_current();
    // ルーラが 0:00〜0:16 なので、composition もそこまで伸ばす
    document.composition.duration = time(16_000);
    let track = document.track_ids.allocate("V1").expect("track");
    let mut names = HashMap::new();

    let (a, la, na) = make_clip(&mut document, "Shared left", 0.6, 6.0, &[1.2, 5.0], &[2.4, 5.8]);
    let (b, lb, nb) = make_clip(&mut document, "Reference text", 5.4, 6.8, &[6.4, 11.2], &[]);
    let (c, lc, nc) = make_clip(&mut document, "Shared right", 11.0, 4.2, &[11.4, 13.1, 14.7], &[]);
    names.insert(la, na);
    names.insert(lb, nb);
    names.insert(lc, nc);

    let group_layer = document.layers.allocate("Title scene").expect("layer");
    names.insert(group_layer, "Title scene".to_owned());
    let mut group_envelope = ItemEnvelope::new(group_layer);
    group_envelope.transform = Transform2D {
        position: keys_at(&mut document, &[2.8, 10.1], DocValue::Vec2([0.0, 0.0])),
        ..Transform2D::identity()
    };
    group_envelope.opacity = keys_at(&mut document, &[1.9, 6.9, 12.5], DocValue::F64(1.0));
    let group = TrackItem::Group(Group {
        envelope: group_envelope,
        children: vec![a, b, c],
    });

    let (bg, lbg, nbg) = make_clip(&mut document, "Background", 0.0, 13.4, &[], &[]);
    let (audio, laudio, naudio) = make_clip(&mut document, "starter-tone.wav", 1.1, 12.5, &[], &[]);
    names.insert(lbg, nbg);
    names.insert(laudio, naudio);

    document.tracks.push(Track {
        id: track,
        items: vec![group, bg, audio],
    });
    document.validate().expect("valid fixture");
    (document, names)
}

fn write_bmp(path: &str, image: &egui::ColorImage) {
    let (w, h) = (image.size[0] as u32, image.size[1] as u32);
    let row_bytes = (w * 3).next_multiple_of(4);
    let mut out = Vec::with_capacity(54 + (row_bytes * h) as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54 + row_bytes * h).to_le_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_named(names: &HashMap<LayerId, String>, want: &str) -> LayerId {
        *names
            .iter()
            .find(|(_, n)| n.as_str() == want)
            .map(|(l, _)| l)
            .expect("fixture layer")
    }

    /// マウス無しで、ドラッグが出すのと同じ command を通す。
    /// **手で触る前に、編集が Document へ届くことをここで確かめる。**
    #[test]
    fn dragging_a_clip_moves_it_in_the_document_and_undo_puts_it_back() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let layer = layer_named(&names, "Background");

        let before = clip_span(&writer.snapshot(), layer).expect("span");
        assert_eq!(before.0, 0.0, "fixture の Background は 0s 始まり");

        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_clip_start(layer, seconds_to_time(2.0).expect("time"))
            .expect("prepare")
            .expect("変化があるので command が出る");
        writer.apply_command(gesture, command).expect("apply");

        let after = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((after.0 - 2.0).abs() < 1e-3, "clip が動いた: {after:?}");
        assert_eq!(
            after.1 - after.0,
            before.1 - before.0,
            "move は尺を変えない"
        );

        writer.undo().expect("undo");
        let restored = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((restored.0 - before.0).abs() < 1e-3, "1ドラッグ = 1 Undo");
    }

    /// **Group を掴むと、子が同じ差分で動く。** モックで決めた挙動。
    #[test]
    fn dragging_a_group_moves_every_child_by_the_same_delta() {
        let (doc, names) = fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let group = layer_named(&names, "Title scene");

        let targets = movable_clips(&writer.snapshot(), group);
        assert_eq!(targets.len(), 3, "Group の子 clip は3枚");

        // Shared right が 15.2s で終わり、composition は 16s。
        // **塊で動ける余地は 0.8s しかない** — それを超える delta は頭打ちになる
        let delta = 0.5_f32;
        let gesture = writer.begin_gesture();
        for (layer, start, _) in &targets {
            let command = writer
                .prepare_set_clip_start(*layer, seconds_to_time(start + delta).expect("time"))
                .expect("prepare")
                .expect("command");
            writer.apply_command(gesture, command).expect("apply");
        }

        let after = writer.snapshot();
        for (layer, start, _) in &targets {
            let moved = clip_span(&after, *layer).expect("span").0;
            assert!(
                (moved - (start + delta)).abs() < 1e-3,
                "子は同じ差分で動く: {moved} vs {}",
                start + delta
            );
        }

        // Group 行の bar は子を包むので、まとめて動いたように見える
        let group_span = clip_span(&after, group).expect("group span");
        assert!((group_span.0 - (0.6 + delta)).abs() < 1e-3);

        writer.undo().expect("undo");
        let restored = writer.snapshot();
        for (layer, start, _) in &targets {
            assert!((clip_span(&restored, *layer).expect("span").0 - start).abs() < 1e-3);
        }
    }
}
