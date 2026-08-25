use makepad_widgets::*;
use motolii_shell::timeline_pane;
use motolii_store::Fps;

const RULER_HEIGHT: f64 = 22.0;
const RAIL_WIDTH: f64 = 150.0;
const PROPERTY_ROW_HEIGHT: f64 = 18.0;
const MIN_VISIBLE_SPAN_SECONDS: f64 = 2.0;

fn fitted_lane_height(total_height: f64, lane_count: usize, property_count: usize) -> f64 {
    let body_height = (total_height - RULER_HEIGHT).max(1.0);
    let properties_height = property_count as f64 * PROPERTY_ROW_HEIGHT;
    if lane_count == 0 {
        body_height
    } else {
        ((body_height - properties_height).max(lane_count as f64) / lane_count as f64).max(1.0)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TimelineLane {
    pub id: u64,
    pub name: String,
    pub hidden: bool,
    pub solo: bool,
    pub locked: bool,
    pub label_color: usize,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TimelinePropertyLane {
    pub layer_id: u64,
    pub name: String,
    pub keys: Vec<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct TimelineModel {
    /// Front-to-back. The backend derives this from `LayerMeta.order`; the widget
    /// never owns a second persistent ordering model.
    pub lanes: Vec<TimelineLane>,
    pub property_lanes: Vec<TimelinePropertyLane>,
    pub duration_frames: i64,
    pub playhead: i64,
    pub fps_num: i64,
    pub fps_den: i64,
}

#[derive(Clone, Debug, Default)]
pub enum TimelineSurfaceAction {
    #[default]
    None,
    Scrub(i64),
    /// Destination is a front-to-back lane index. The backend translates this
    /// once into a Document stacking edit (one gesture = one undo).
    Restack {
        layer_id: u64,
        target_from_front: usize,
    },
    ZoomChanged {
        start_frame: i64,
        visible_frames: i64,
    },
}

#[derive(Clone, Copy, Debug, Default)]
enum DragState {
    #[default]
    None,
    Playhead,
    Lane {
        layer_id: u64,
        from_front: usize,
        target_from_front: usize,
    },
}

#[derive(Clone, Copy, Debug)]
enum VisualRowKind {
    Lane(usize),
    Property(usize),
}

#[derive(Clone, Copy, Debug)]
struct VisualRow {
    kind: VisualRowKind,
    y: f64,
    height: f64,
}

#[derive(Script, ScriptHook, Widget)]
pub struct TimelineSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,

    /// This draw object owns the full hit area. All later rectangles use a
    /// separate draw object so their smaller areas cannot replace it.
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_item: DrawColor,
    #[live]
    draw_text: DrawText,

    #[rust]
    rect: Rect,
    #[rust]
    lanes: Vec<TimelineLane>,
    #[rust]
    property_lanes: Vec<TimelinePropertyLane>,
    #[rust]
    duration_frames: i64,
    #[rust]
    playhead: i64,
    #[rust]
    fps_num: i64,
    #[rust]
    fps_den: i64,
    /// Horizontal viewport only. There is intentionally no vertical scale.
    #[rust]
    view_start: f64,
    #[rust]
    view_span: f64,
    #[rust]
    drag: DragState,
}

impl TimelineSurface {
    pub fn set_model(&mut self, cx: &mut Cx, model: TimelineModel) {
        let first_model = self.duration_frames <= 0 || self.view_span <= 0.0;
        self.lanes = model.lanes;
        self.property_lanes = model.property_lanes;
        self.duration_frames = model.duration_frames.max(1);
        self.playhead = model
            .playhead
            .clamp(0, self.duration_frames.saturating_sub(1));
        self.fps_num = model.fps_num.max(1);
        self.fps_den = model.fps_den.max(1);

        if first_model {
            self.view_start = 0.0;
            self.view_span = self.duration_frames as f64;
        } else {
            self.view_span = self
                .view_span
                .clamp(self.min_view_span(), self.duration_frames as f64);
            self.clamp_view_start();
        }
        self.redraw(cx);
    }

    fn fps(&self) -> f64 {
        self.fps_num.max(1) as f64 / self.fps_den.max(1) as f64
    }

    fn min_view_span(&self) -> f64 {
        (self.fps() * MIN_VISIBLE_SPAN_SECONDS)
            .max(10.0)
            .min(self.duration_frames.max(1) as f64)
    }

    fn clamp_view_start(&mut self) {
        let max_start = (self.duration_frames as f64 - self.view_span).max(0.0);
        self.view_start = self.view_start.clamp(0.0, max_start);
    }

    fn time_rect(&self) -> Rect {
        Rect {
            pos: dvec2(self.rect.pos.x + RAIL_WIDTH, self.rect.pos.y),
            size: dvec2((self.rect.size.x - RAIL_WIDTH).max(1.0), self.rect.size.y),
        }
    }

    fn property_count_for_visible_lanes(&self) -> usize {
        self.property_lanes
            .iter()
            .filter(|property| self.lanes.iter().any(|lane| lane.id == property.layer_id))
            .count()
    }

    fn lane_height(&self) -> f64 {
        fitted_lane_height(
            self.rect.size.y,
            self.lanes.len(),
            self.property_count_for_visible_lanes(),
        )
    }

    fn visual_rows(&self) -> Vec<VisualRow> {
        let mut rows = Vec::with_capacity(self.lanes.len() + self.property_lanes.len());
        let lane_height = self.lane_height();
        let mut y = self.rect.pos.y + RULER_HEIGHT;
        for (lane_index, lane) in self.lanes.iter().enumerate() {
            rows.push(VisualRow {
                kind: VisualRowKind::Lane(lane_index),
                y,
                height: lane_height,
            });
            y += lane_height;
            for (property_index, property) in self.property_lanes.iter().enumerate() {
                if property.layer_id == lane.id {
                    rows.push(VisualRow {
                        kind: VisualRowKind::Property(property_index),
                        y,
                        height: PROPERTY_ROW_HEIGHT,
                    });
                    y += PROPERTY_ROW_HEIGHT;
                }
            }
        }
        rows
    }

    fn lane_at_y(&self, abs_y: f64) -> Option<(u64, usize)> {
        self.visual_rows().into_iter().find_map(|row| {
            if abs_y < row.y || abs_y >= row.y + row.height {
                return None;
            }
            match row.kind {
                VisualRowKind::Lane(index) => Some((self.lanes[index].id, index)),
                VisualRowKind::Property(_) => None,
            }
        })
    }

    fn drop_index_at_y(&self, abs_y: f64) -> usize {
        let lane_rows: Vec<VisualRow> = self
            .visual_rows()
            .into_iter()
            .filter(|row| matches!(row.kind, VisualRowKind::Lane(_)))
            .collect();
        if lane_rows.is_empty() {
            return 0;
        }
        for (index, row) in lane_rows.iter().enumerate() {
            if abs_y < row.y + row.height * 0.5 {
                return index;
            }
        }
        lane_rows.len() - 1
    }

    fn frame_at_x(&self, abs_x: f64) -> i64 {
        let time = self.time_rect();
        let fraction = ((abs_x - time.pos.x) / time.size.x).clamp(0.0, 1.0);
        (self.view_start + fraction * self.view_span)
            .round()
            .clamp(0.0, self.duration_frames.saturating_sub(1) as f64) as i64
    }

    fn x_at_frame(&self, frame: f64) -> f64 {
        let time = self.time_rect();
        time.pos.x + (frame - self.view_start) / self.view_span.max(1.0) * time.size.x
    }

    fn emit_scrub(&mut self, cx: &mut Cx, abs_x: f64) {
        let frame = self.frame_at_x(abs_x);
        if frame != self.playhead {
            self.playhead = frame;
            self.redraw(cx);
        }
        cx.widget_action(self.uid, TimelineSurfaceAction::Scrub(frame));
    }

    fn zoom_at(&mut self, cx: &mut Cx, scroll: f64, abs_x: f64) {
        if scroll.abs() <= f64::EPSILON || abs_x < self.time_rect().pos.x {
            return;
        }
        let time = self.time_rect();
        let anchor_fraction = ((abs_x - time.pos.x) / time.size.x).clamp(0.0, 1.0);
        let anchor_frame = self.view_start + anchor_fraction * self.view_span;
        let zoom_power = (-scroll / 240.0).clamp(-1.0, 1.0);
        let next_span = (self.view_span * 2.0_f64.powf(zoom_power))
            .clamp(self.min_view_span(), self.duration_frames as f64);
        if (next_span - self.view_span).abs() < 0.01 {
            return;
        }
        self.view_span = next_span;
        self.view_start = anchor_frame - anchor_fraction * self.view_span;
        self.clamp_view_start();
        self.redraw(cx);
        cx.widget_action(
            self.uid,
            TimelineSurfaceAction::ZoomChanged {
                start_frame: self.view_start.round() as i64,
                visible_frames: self.view_span.round() as i64,
            },
        );
    }

    fn set_hover_cursor(&self, cx: &mut Cx, abs: DVec2) {
        if abs.x >= self.time_rect().pos.x {
            cx.set_cursor(MouseCursor::EwResize);
        } else if self.lane_at_y(abs.y).is_some() {
            cx.set_cursor(MouseCursor::Grab);
        } else {
            cx.set_cursor(MouseCursor::Default);
        }
    }

    fn draw_rect(&mut self, cx: &mut Cx2d, rect: Rect, color: Vec4f) {
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        self.draw_item.color = color;
        self.draw_item.draw_abs(cx, rect);
    }

    fn draw_label(&mut self, cx: &mut Cx2d, pos: DVec2, text: &str, color: Vec4f, size: f32) {
        self.draw_text.color = color;
        self.draw_text.text_style.font_size = size;
        self.draw_text.draw_abs(cx, pos, text);
    }

    fn lane_color(index: usize) -> Vec4f {
        const COLORS: [[f32; 4]; 12] = [
            [0.92, 0.92, 0.88, 1.0],
            [0.71, 0.55, 0.47, 1.0],
            [0.47, 0.59, 0.67, 1.0],
            [0.82, 0.78, 0.92, 1.0],
            [0.63, 0.47, 0.59, 1.0],
            [0.92, 0.78, 0.59, 1.0],
            [0.86, 0.35, 0.35, 1.0],
            [0.35, 0.71, 0.67, 1.0],
            [0.55, 0.43, 0.67, 1.0],
            [0.29, 0.35, 0.51, 1.0],
            [0.78, 0.78, 0.78, 1.0],
            [0.55, 0.55, 0.51, 1.0],
        ];
        let color = COLORS[index % COLORS.len()];
        vec4(color[0], color[1], color[2], color[3])
    }

    fn draw_lane(&mut self, cx: &mut Cx2d, lane: &TimelineLane, row: VisualRow, zebra: bool) {
        let bg = if lane.selected {
            vec4(0.34, 0.31, 0.28, 1.0)
        } else if zebra {
            vec4(0.205, 0.205, 0.205, 1.0)
        } else {
            vec4(0.225, 0.225, 0.225, 1.0)
        };
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(self.rect.size.x, row.height),
            },
            bg,
        );

        let color = Self::lane_color(lane.label_color);
        // Sticky-note tab: full lane height, left aligned. It labels the row
        // without adding a second, misleading 8x8 "content height" signal.
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(4.0, row.height),
            },
            color,
        );

        let text_y = row.y + ((row.height - 9.0) * 0.5).max(0.0);
        self.draw_label(
            cx,
            dvec2(self.rect.pos.x + 9.0, text_y),
            &lane.name,
            if lane.selected {
                vec4(0.93, 0.91, 0.84, 1.0)
            } else {
                vec4(0.72, 0.72, 0.72, 1.0)
            },
            7.8,
        );

        let control_h = (row.height - 4.0).clamp(8.0, 13.0);
        let control_y = row.y + (row.height - control_h) * 0.5;
        let control_x = self.rect.pos.x + RAIL_WIDTH - 45.0;
        for (index, (label, active)) in [("M", lane.hidden), ("S", lane.solo), ("L", lane.locked)]
            .into_iter()
            .enumerate()
        {
            let x = control_x + index as f64 * 15.0;
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, control_y),
                    size: dvec2(12.0, control_h),
                },
                if active {
                    vec4(0.66, 0.53, 0.33, 1.0)
                } else {
                    vec4(0.16, 0.16, 0.16, 1.0)
                },
            );
            self.draw_label(
                cx,
                dvec2(x + 3.2, control_y + ((control_h - 7.0) * 0.5).max(0.0)),
                label,
                if active {
                    vec4(0.95, 0.92, 0.84, 1.0)
                } else {
                    vec4(0.55, 0.55, 0.55, 1.0)
                },
                6.4,
            );
        }

        let visible_start = self.view_start;
        let visible_end = self.view_start + self.view_span;
        let clip_start = lane.start as f64;
        let clip_end = (lane.start + lane.duration) as f64;
        let left = clip_start.max(visible_start);
        let right = clip_end.min(visible_end);
        if right > left {
            let x0 = self.x_at_frame(left);
            let x1 = self.x_at_frame(right);
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x0, row.y),
                    // One separator pixel is the only vertical gap. The clip
                    // otherwise fits the lane instead of floating inside it.
                    size: dvec2((x1 - x0).max(1.0), (row.height - 1.0).max(1.0)),
                },
                color,
            );
        }
    }

    fn draw_property(&mut self, cx: &mut Cx2d, property: &TimelinePropertyLane, row: VisualRow) {
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(self.rect.size.x, row.height),
            },
            vec4(0.18, 0.18, 0.18, 1.0),
        );
        self.draw_label(
            cx,
            dvec2(self.rect.pos.x + 20.0, row.y + (row.height - 8.0) * 0.5),
            &property.name,
            vec4(0.57, 0.57, 0.57, 1.0),
            7.2,
        );
        let key_color = self
            .lanes
            .iter()
            .find(|lane| lane.id == property.layer_id)
            .map(|lane| Self::lane_color(lane.label_color))
            .unwrap_or_else(|| vec4(0.92, 0.78, 0.59, 1.0));
        let key_size = 8.0;
        let key_y = row.y + (row.height - key_size) * 0.5;
        for &frame in &property.keys {
            let frame = frame as f64;
            if frame < self.view_start || frame > self.view_start + self.view_span {
                continue;
            }
            let x = self.x_at_frame(frame) - key_size * 0.5;
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, key_y),
                    size: dvec2(key_size, key_size),
                },
                key_color,
            );
        }
    }

    fn draw_grid_and_ruler(&mut self, cx: &mut Cx2d, lane_height: f64) {
        let time = self.time_rect();
        let fps = Fps::try_new(self.fps_num.max(1), self.fps_den.max(1)).ok();
        let (minor, major) = timeline_pane::tick_steps_with_target(
            fps,
            self.view_span.round().max(1.0) as i64,
            time.size.x as f32,
            lane_height.max(1.0) as f32,
            5.0,
        );
        let first_minor = (self.view_start / minor as f64).ceil() as i64 * minor;
        let last = (self.view_start + self.view_span).ceil() as i64;
        let mut frame = first_minor;
        while frame <= last {
            let is_major = frame.rem_euclid(major) == 0;
            let x = self.x_at_frame(frame as f64);
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, self.rect.pos.y + RULER_HEIGHT),
                    size: dvec2(1.0, (self.rect.size.y - RULER_HEIGHT).max(1.0)),
                },
                if is_major {
                    vec4(0.05, 0.05, 0.05, 0.38)
                } else {
                    vec4(0.02, 0.02, 0.02, 0.20)
                },
            );
            frame = frame.saturating_add(minor.max(1));
        }

        // Ruler is deliberately emitted in a fresh foreground draw call after
        // clips and body grid, so bars can never overwrite its ticks again.
        self.draw_item.new_draw_call(cx);
        self.draw_rect(
            cx,
            Rect {
                pos: self.rect.pos,
                size: dvec2(self.rect.size.x, RULER_HEIGHT),
            },
            vec4(0.245, 0.245, 0.245, 1.0),
        );
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x + RAIL_WIDTH - 1.0, self.rect.pos.y),
                size: dvec2(1.0, self.rect.size.y),
            },
            vec4(0.10, 0.10, 0.10, 1.0),
        );

        let zoom_percent = (self.duration_frames as f64 / self.view_span.max(1.0) * 100.0).round();
        self.draw_label(
            cx,
            dvec2(self.rect.pos.x + 9.0, self.rect.pos.y + 5.0),
            &format!("TIME  {zoom_percent:.0}%"),
            vec4(0.50, 0.50, 0.50, 1.0),
            7.3,
        );

        let first_minor = (self.view_start / minor as f64).ceil() as i64 * minor;
        let mut frame = first_minor;
        while frame <= last {
            let is_major = frame.rem_euclid(major) == 0;
            let x = self.x_at_frame(frame as f64);
            let tick_height = if is_major { 11.0 } else { 5.0 };
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, self.rect.pos.y + RULER_HEIGHT - tick_height),
                    size: dvec2(1.0, tick_height),
                },
                if is_major {
                    vec4(0.47, 0.47, 0.47, 1.0)
                } else {
                    vec4(0.12, 0.12, 0.12, 0.55)
                },
            );
            if is_major {
                let seconds = frame as f64 / self.fps();
                let label = if major as f64 >= self.fps() {
                    format!("{seconds:.0}")
                } else {
                    format!("{seconds:.1}")
                };
                self.draw_label(
                    cx,
                    dvec2(x + 2.0, self.rect.pos.y + 1.0),
                    &label,
                    vec4(0.55, 0.55, 0.55, 1.0),
                    7.0,
                );
            }
            frame = frame.saturating_add(minor.max(1));
        }
    }

    fn draw_playhead_and_drop_target(&mut self, cx: &mut Cx2d) {
        let time = self.time_rect();
        let playhead_x = self.x_at_frame(self.playhead as f64);
        if playhead_x >= time.pos.x && playhead_x <= time.pos.x + time.size.x {
            self.draw_item.new_draw_call(cx);
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(playhead_x, self.rect.pos.y),
                    size: dvec2(1.5, self.rect.size.y),
                },
                vec4(0.85, 0.71, 0.45, 1.0),
            );
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(playhead_x - 3.0, self.rect.pos.y),
                    size: dvec2(7.0, 7.0),
                },
                vec4(0.85, 0.71, 0.45, 1.0),
            );
        }

        if let DragState::Lane {
            target_from_front, ..
        } = self.drag
        {
            if let Some(row) = self
                .visual_rows()
                .into_iter()
                .filter(|row| matches!(row.kind, VisualRowKind::Lane(_)))
                .nth(target_from_front)
            {
                self.draw_item.new_draw_call(cx);
                self.draw_rect(
                    cx,
                    Rect {
                        pos: dvec2(self.rect.pos.x, row.y),
                        size: dvec2(self.rect.size.x, 2.0),
                    },
                    vec4(0.85, 0.71, 0.45, 1.0),
                );
            }
        }
    }
}

impl Widget for TimelineSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits_with_capture_overload(cx, self.draw_bg.area(), true) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                if fe.abs.x >= self.time_rect().pos.x {
                    self.drag = DragState::Playhead;
                    cx.set_cursor(MouseCursor::EwResize);
                    self.emit_scrub(cx, fe.abs.x);
                } else if let Some((layer_id, from_front)) = self.lane_at_y(fe.abs.y) {
                    self.drag = DragState::Lane {
                        layer_id,
                        from_front,
                        target_from_front: from_front,
                    };
                    cx.set_cursor(MouseCursor::Grabbing);
                    self.redraw(cx);
                }
            }
            Hit::FingerMove(fe) => match self.drag {
                DragState::Playhead => self.emit_scrub(cx, fe.abs.x),
                DragState::Lane {
                    layer_id,
                    from_front,
                    target_from_front: _,
                } => {
                    let target_from_front = self.drop_index_at_y(fe.abs.y);
                    self.drag = DragState::Lane {
                        layer_id,
                        from_front,
                        target_from_front,
                    };
                    self.redraw(cx);
                }
                DragState::None => {}
            },
            Hit::FingerUp(_) => {
                if let DragState::Lane {
                    layer_id,
                    from_front,
                    target_from_front,
                } = self.drag
                {
                    if from_front != target_from_front {
                        cx.widget_action(
                            self.uid,
                            TimelineSurfaceAction::Restack {
                                layer_id,
                                target_from_front,
                            },
                        );
                    }
                }
                self.drag = DragState::None;
                cx.set_cursor(MouseCursor::Default);
                self.redraw(cx);
            }
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                self.set_hover_cursor(cx, fe.abs);
            }
            Hit::FingerHoverOut(_) => cx.set_cursor(MouseCursor::Default),
            Hit::FingerScroll(fs) => {
                let scroll = if fs.scroll.y.abs() > f64::EPSILON {
                    fs.scroll.y
                } else {
                    fs.scroll.x
                };
                self.zoom_at(cx, scroll, fs.abs.x);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.rect = cx.walk_turtle(walk);
        self.draw_bg.draw_abs(cx, self.rect);
        self.draw_item.new_draw_call(cx);

        let lanes = self.lanes.clone();
        let properties = self.property_lanes.clone();
        let rows = self.visual_rows();
        let mut lane_number = 0usize;
        for row in rows {
            match row.kind {
                VisualRowKind::Lane(index) => {
                    self.draw_lane(cx, &lanes[index], row, lane_number % 2 == 1);
                    lane_number += 1;
                }
                VisualRowKind::Property(index) => {
                    self.draw_property(cx, &properties[index], row);
                }
            }
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(self.rect.pos.x, row.y + row.height - 1.0),
                    size: dvec2(self.rect.size.x, 1.0),
                },
                vec4(0.13, 0.13, 0.13, 1.0),
            );
        }

        self.draw_grid_and_ruler(cx, self.lane_height());
        self.draw_playhead_and_drop_target(cx);
        DrawStep::done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_height_fits_all_lanes_and_keeps_property_height_fixed() {
        let occupied = fitted_lane_height(300.0, 15, 1) * 15.0 + PROPERTY_ROW_HEIGHT + RULER_HEIGHT;
        assert!((occupied - 300.0).abs() < 0.001);
    }
}
