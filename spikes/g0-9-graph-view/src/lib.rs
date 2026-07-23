//! 固定React Graph ViewをBlender-likeなnative surfaceへ投影する隔離fixture。
//! Blender sourceは使用せず、製品Document/D2の代替にもならない。

use std::collections::BTreeSet;

use serde::Serialize;
use understory_view2d::Viewport1D;

pub const WIDTH: f32 = 1100.0;
pub const HEIGHT: f32 = 650.0;
pub const HEADER_HEIGHT: f32 = 56.0;
pub const STATUS_HEIGHT: f32 = 25.0;
pub const CHANNEL_WIDTH: f32 = 224.0;
pub const PLOT: [f32; 4] = [278.0, 82.0, 794.0, 514.0];
pub const TIME_RANGE: [f32; 2] = [52.0, 56.0];
pub const VALUE_RANGE: [f32; 2] = [0.0, 100.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphPoint {
    pub time: f32,
    pub value: f32,
}

impl GraphPoint {
    pub const fn new(time: f32, value: f32) -> Self {
        Self { time, value }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphKey {
    pub id: &'static str,
    pub point: GraphPoint,
    pub incoming: Option<GraphPoint>,
    pub outgoing: Option<GraphPoint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphChannel {
    pub id: &'static str,
    pub object: &'static str,
    pub parameter: &'static str,
    pub unit: &'static str,
    pub color: [f32; 4],
    pub keys: Vec<GraphKey>,
}

pub fn fixture_channels() -> Vec<GraphChannel> {
    vec![
        GraphChannel {
            id: "intensity",
            object: "Pulse rings",
            parameter: "Intensity",
            unit: "%",
            color: [0.89, 0.48, 0.12, 1.0],
            keys: vec![
                GraphKey {
                    id: "i0",
                    point: GraphPoint::new(52.18, 18.0),
                    incoming: None,
                    outgoing: Some(GraphPoint::new(52.56, 18.0)),
                },
                GraphKey {
                    id: "i1",
                    point: GraphPoint::new(53.24, 82.0),
                    incoming: Some(GraphPoint::new(52.86, 82.0)),
                    outgoing: Some(GraphPoint::new(53.62, 82.0)),
                },
                GraphKey {
                    id: "i2",
                    point: GraphPoint::new(54.48, 36.0),
                    incoming: Some(GraphPoint::new(54.08, 36.0)),
                    outgoing: Some(GraphPoint::new(54.86, 36.0)),
                },
                GraphKey {
                    id: "i3",
                    point: GraphPoint::new(55.62, 78.0),
                    incoming: Some(GraphPoint::new(55.22, 78.0)),
                    outgoing: None,
                },
            ],
        },
        GraphChannel {
            id: "spread",
            object: "Pulse rings",
            parameter: "Spread",
            unit: "%",
            color: [0.25, 0.64, 0.86, 1.0],
            keys: vec![
                GraphKey {
                    id: "s0",
                    point: GraphPoint::new(52.18, 34.0),
                    incoming: None,
                    outgoing: Some(GraphPoint::new(52.72, 34.0)),
                },
                GraphKey {
                    id: "s1",
                    point: GraphPoint::new(54.06, 64.0),
                    incoming: Some(GraphPoint::new(53.52, 64.0)),
                    outgoing: Some(GraphPoint::new(54.42, 64.0)),
                },
                GraphKey {
                    id: "s2",
                    point: GraphPoint::new(55.62, 42.0),
                    incoming: Some(GraphPoint::new(55.18, 42.0)),
                    outgoing: None,
                },
            ],
        },
        GraphChannel {
            id: "depth",
            object: "City grid",
            parameter: "Depth",
            unit: "z",
            color: [0.48, 0.76, 0.35, 1.0],
            keys: vec![
                GraphKey {
                    id: "d0",
                    point: GraphPoint::new(52.18, 58.0),
                    incoming: None,
                    outgoing: Some(GraphPoint::new(52.86, 58.0)),
                },
                GraphKey {
                    id: "d1",
                    point: GraphPoint::new(55.62, 28.0),
                    incoming: Some(GraphPoint::new(54.94, 28.0)),
                    outgoing: None,
                },
            ],
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyId {
    pub channel: &'static str,
    pub key: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTarget {
    Key { channel: usize, key: usize },
    Incoming { channel: usize, key: usize },
    Outgoing { channel: usize, key: usize },
}

impl HitTarget {
    const fn indices(self) -> (usize, usize) {
        match self {
            Self::Key { channel, key }
            | Self::Incoming { channel, key }
            | Self::Outgoing { channel, key } => (channel, key),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GraphViewport {
    time: Viewport1D,
    value_down: Viewport1D,
}

impl Default for GraphViewport {
    fn default() -> Self {
        let mut time = Viewport1D::new(PLOT[0] as f64..(PLOT[0] + PLOT[2]) as f64);
        time.set_visible_world_range(TIME_RANGE[0] as f64..TIME_RANGE[1] as f64);
        let time_zoom = time.zoom();
        time.set_zoom_limits(time_zoom * 0.25, time_zoom * 64.0);

        let mut value_down = Viewport1D::new(PLOT[1] as f64..(PLOT[1] + PLOT[3]) as f64);
        value_down.set_visible_world_range(0.0..100.0);
        let value_zoom = value_down.zoom();
        value_down.set_zoom_limits(value_zoom * 0.25, value_zoom * 64.0);
        Self { time, value_down }
    }
}

impl GraphViewport {
    pub fn screen_from_graph(&self, point: GraphPoint) -> [f32; 2] {
        [
            self.time.world_to_view_x(point.time as f64) as f32,
            self.value_down
                .world_to_view_x((VALUE_RANGE[1] - point.value) as f64) as f32,
        ]
    }

    pub fn graph_from_screen(&self, point: [f32; 2]) -> GraphPoint {
        GraphPoint::new(
            self.time.view_to_world_x(point[0] as f64) as f32,
            VALUE_RANGE[1] - self.value_down.view_to_world_x(point[1] as f64) as f32,
        )
    }

    pub fn pan_by_view(&mut self, delta: [f32; 2]) {
        self.time.pan_by_view(delta[0] as f64);
        self.value_down.pan_by_view(delta[1] as f64);
    }

    pub fn zoom_about_screen(&mut self, anchor: [f32; 2], factor: f64) {
        self.time.zoom_about_view_point(anchor[0] as f64, factor);
        self.value_down
            .zoom_about_view_point(anchor[1] as f64, factor);
    }

    pub fn fit_all(&mut self) {
        self.time
            .set_visible_world_range(TIME_RANGE[0] as f64..TIME_RANGE[1] as f64);
        self.value_down.set_visible_world_range(0.0..100.0);
    }

    pub fn fit_points(&mut self, points: impl Iterator<Item = GraphPoint>) -> bool {
        let mut min_time = f32::INFINITY;
        let mut max_time = f32::NEG_INFINITY;
        let mut min_value = f32::INFINITY;
        let mut max_value = f32::NEG_INFINITY;
        let mut count = 0;
        for point in points {
            if !point.time.is_finite() || !point.value.is_finite() {
                continue;
            }
            min_time = min_time.min(point.time);
            max_time = max_time.max(point.time);
            min_value = min_value.min(point.value);
            max_value = max_value.max(point.value);
            count += 1;
        }
        if count == 0 {
            return false;
        }
        let time_padding = ((max_time - min_time) * 0.12).max(0.2);
        let value_padding = ((max_value - min_value) * 0.12).max(8.0);
        self.time.set_visible_world_range(
            (min_time - time_padding) as f64..(max_time + time_padding) as f64,
        );
        self.value_down.set_visible_world_range(
            (VALUE_RANGE[1] - max_value - value_padding) as f64
                ..(VALUE_RANGE[1] - min_value + value_padding) as f64,
        );
        true
    }

    pub fn visible_ranges(&self) -> ([f32; 2], [f32; 2]) {
        let time = self.time.visible_world_range();
        let value_down = self.value_down.visible_world_range();
        (
            [time.start as f32, time.end as f32],
            [
                VALUE_RANGE[1] - value_down.end as f32,
                VALUE_RANGE[1] - value_down.start as f32,
            ],
        )
    }
}

#[derive(Clone, Debug)]
struct DragState {
    token: u64,
    target: HitTarget,
    start_keys: Vec<GraphKey>,
    start_screen: Option<[f32; 2]>,
    crossed_threshold: bool,
    changed: bool,
}

#[derive(Clone, Debug)]
struct MarqueeState {
    start: [f32; 2],
    current: [f32; 2],
    start_selection: BTreeSet<KeyId>,
    additive: bool,
    crossed_threshold: bool,
}

#[derive(Clone, Debug)]
pub struct GraphSession {
    pub channels: Vec<GraphChannel>,
    pub active_channel: usize,
    pub selected_key: usize,
    drag: Option<DragState>,
    marquee: Option<MarqueeState>,
    pub viewport: GraphViewport,
    pub selection: BTreeSet<KeyId>,
    pub snap_enabled: bool,
    pub semantic_commit_count: u32,
    pub navigation_change_count: u32,
    pub selection_change_count: u32,
    pub readback_count: u32,
    pub hot_resource_creation_count: u32,
    pub status: &'static str,
}

impl Default for GraphSession {
    fn default() -> Self {
        let mut selection = BTreeSet::new();
        selection.insert(KeyId {
            channel: "intensity",
            key: "i1",
        });
        Self {
            channels: fixture_channels(),
            active_channel: 0,
            selected_key: 1,
            drag: None,
            marquee: None,
            viewport: GraphViewport::default(),
            selection,
            snap_enabled: true,
            semantic_commit_count: 0,
            navigation_change_count: 0,
            selection_change_count: 0,
            readback_count: 0,
            hot_resource_creation_count: 0,
            status: "Intensity · 4 keys · selected i1",
        }
    }
}

impl GraphSession {
    pub fn begin_drag(&mut self, target: HitTarget, token: u64) -> bool {
        self.begin_drag_at(target, token, None, false)
    }

    pub fn begin_drag_at(
        &mut self,
        target: HitTarget,
        token: u64,
        start_screen: Option<[f32; 2]>,
        additive: bool,
    ) -> bool {
        if self.drag.is_some() {
            return false;
        }
        let (channel, key) = target.indices();
        self.active_channel = channel;
        self.selected_key = key;
        self.select_key(channel, key, additive);
        self.drag = Some(DragState {
            token,
            target,
            start_keys: self.channels[channel].keys.clone(),
            start_screen,
            crossed_threshold: start_screen.is_none(),
            changed: false,
        });
        self.status = "Preview · release commits once · Esc cancels";
        true
    }

    pub fn update_drag(&mut self, point: GraphPoint) -> bool {
        let Some(drag) = self.drag.as_ref() else {
            return false;
        };
        let source = &drag.start_keys;
        let (channel, index) = drag.target.indices();
        let mut next = source.clone();
        let clamped = GraphPoint::new(
            point.time.clamp(TIME_RANGE[0], TIME_RANGE[1]),
            point.value.clamp(VALUE_RANGE[0], VALUE_RANGE[1]),
        );
        match drag.target {
            HitTarget::Key { .. } => {
                let min_time = if index == 0 {
                    TIME_RANGE[0]
                } else {
                    source[index - 1].point.time + 0.02
                };
                let max_time = if index + 1 == source.len() {
                    TIME_RANGE[1]
                } else {
                    source[index + 1].point.time - 0.02
                };
                let time = if self.snap_enabled {
                    (clamped.time * 10.0).round() / 10.0
                } else {
                    clamped.time
                };
                let moved = GraphPoint::new(time.clamp(min_time, max_time), clamped.value);
                let dt = moved.time - source[index].point.time;
                let dv = moved.value - source[index].point.value;
                next[index].point = moved;
                next[index].incoming = source[index]
                    .incoming
                    .map(|p| GraphPoint::new(p.time + dt, p.value + dv));
                next[index].outgoing = source[index]
                    .outgoing
                    .map(|p| GraphPoint::new(p.time + dt, p.value + dv));
            }
            HitTarget::Incoming { .. } => next[index].incoming = Some(clamped),
            HitTarget::Outgoing { .. } => next[index].outgoing = Some(clamped),
        }
        self.channels[channel].keys = next;
        if let Some(drag) = self.drag.as_mut() {
            drag.changed = true;
        }
        true
    }

    pub fn update_drag_screen(&mut self, screen: [f32; 2]) -> bool {
        let Some(drag) = self.drag.as_mut() else {
            return false;
        };
        if !drag.crossed_threshold {
            let Some(start) = drag.start_screen else {
                drag.crossed_threshold = true;
                return self.update_drag(self.viewport.graph_from_screen(screen));
            };
            if distance(start, screen) < 4.0 {
                return false;
            }
            drag.crossed_threshold = true;
        }
        self.update_drag(self.viewport.graph_from_screen(screen))
    }

    pub fn release(&mut self, token: u64) -> bool {
        if self.drag.as_ref().is_none_or(|drag| drag.token != token) {
            return false;
        }
        let changed = self.drag.take().is_some_and(|drag| drag.changed);
        if changed {
            self.semantic_commit_count += 1;
            self.status = "Committed · Undo 1";
        } else {
            self.status = "Selected · Document unchanged";
        }
        true
    }

    pub fn cancel(&mut self) -> bool {
        if let Some(drag) = self.drag.take() {
            let (channel, _) = drag.target.indices();
            self.channels[channel].keys = drag.start_keys;
            self.status = "Cancelled · Document unchanged";
            return true;
        }
        if let Some(marquee) = self.marquee.take() {
            self.selection = marquee.start_selection;
            self.status = "Selection cancelled · Document unchanged";
            return true;
        }
        false
    }

    pub fn active_target(&self) -> Option<HitTarget> {
        self.drag.as_ref().map(|drag| drag.target)
    }

    pub fn begin_marquee(&mut self, start: [f32; 2], additive: bool) -> bool {
        if self.drag.is_some() || self.marquee.is_some() || !plot_contains(start) {
            return false;
        }
        self.marquee = Some(MarqueeState {
            start,
            current: start,
            start_selection: self.selection.clone(),
            additive,
            crossed_threshold: false,
        });
        self.status = "Marquee preview · release selects · Esc cancels";
        true
    }

    pub fn update_marquee(&mut self, current: [f32; 2]) -> bool {
        let Some(mut marquee) = self.marquee.take() else {
            return false;
        };
        marquee.current = clamp_to_plot(current);
        if !marquee.crossed_threshold && distance(marquee.start, marquee.current) >= 4.0 {
            marquee.crossed_threshold = true;
        }
        if marquee.crossed_threshold {
            let rect = normalized_rect(marquee.start, marquee.current);
            let mut next = if marquee.additive {
                marquee.start_selection.clone()
            } else {
                BTreeSet::new()
            };
            for (channel_index, channel) in self.channels.iter().enumerate() {
                for (key_index, key) in channel.keys.iter().enumerate() {
                    if rect_contains(rect, self.viewport.screen_from_graph(key.point)) {
                        next.insert(self.key_id(channel_index, key_index));
                    }
                }
            }
            self.selection = next;
        }
        self.marquee = Some(marquee);
        true
    }

    pub fn release_marquee(&mut self) -> bool {
        let Some(marquee) = self.marquee.take() else {
            return false;
        };
        if !marquee.crossed_threshold && !marquee.additive {
            self.selection.clear();
        }
        self.selection_change_count += 1;
        self.status = "Selection changed · Document unchanged";
        true
    }

    pub fn marquee_rect(&self) -> Option<[f32; 4]> {
        self.marquee
            .as_ref()
            .filter(|state| state.crossed_threshold)
            .map(|state| normalized_rect(state.start, state.current))
    }

    pub fn pan_by_view(&mut self, delta: [f32; 2]) -> bool {
        if !delta[0].is_finite() || !delta[1].is_finite() {
            return false;
        }
        self.viewport.pan_by_view(delta);
        self.navigation_change_count += 1;
        self.status = "View panned · Document unchanged";
        true
    }

    pub fn zoom_about_screen(&mut self, anchor: [f32; 2], factor: f64) -> bool {
        if !plot_contains(anchor) || !factor.is_finite() || factor <= 0.0 {
            return false;
        }
        self.viewport.zoom_about_screen(anchor, factor);
        self.navigation_change_count += 1;
        self.status = "View zoomed · cursor anchor preserved";
        true
    }

    pub fn fit_all(&mut self) {
        self.viewport.fit_all();
        self.navigation_change_count += 1;
        self.status = "Frame All · Document unchanged";
    }

    pub fn fit_selection(&mut self) -> bool {
        let points = self
            .channels
            .iter()
            .flat_map(|channel| {
                channel.keys.iter().filter_map(|key| {
                    self.selection
                        .contains(&KeyId {
                            channel: channel.id,
                            key: key.id,
                        })
                        .then_some(key.point)
                })
            })
            .collect::<Vec<_>>();
        if !self.viewport.fit_points(points.into_iter()) {
            return false;
        }
        self.navigation_change_count += 1;
        self.status = "Frame Selection · Document unchanged";
        true
    }

    pub fn toggle_snap(&mut self) {
        self.snap_enabled = !self.snap_enabled;
        self.status = if self.snap_enabled {
            "Snap: Frame enabled"
        } else {
            "Snap disabled"
        };
    }

    pub fn is_selected(&self, channel: usize, key: usize) -> bool {
        self.selection.contains(&self.key_id(channel, key))
    }

    fn select_key(&mut self, channel: usize, key: usize, additive: bool) {
        let id = self.key_id(channel, key);
        if additive {
            if !self.selection.insert(id) {
                self.selection.remove(&id);
            }
        } else {
            self.selection.clear();
            self.selection.insert(id);
        }
        self.selection_change_count += 1;
    }

    fn key_id(&self, channel: usize, key: usize) -> KeyId {
        KeyId {
            channel: self.channels[channel].id,
            key: self.channels[channel].keys[key].id,
        }
    }
}

pub fn screen_from_graph(point: GraphPoint) -> [f32; 2] {
    GraphViewport::default().screen_from_graph(point)
}

pub fn graph_from_screen(point: [f32; 2]) -> GraphPoint {
    GraphViewport::default().graph_from_screen(point)
}

pub fn hit_target(session: &GraphSession, cursor: [f32; 2]) -> Option<HitTarget> {
    let keys = &session.channels[session.active_channel].keys;
    let selected = &keys[session.selected_key];
    for (target, point) in [
        (
            HitTarget::Incoming {
                channel: session.active_channel,
                key: session.selected_key,
            },
            selected.incoming,
        ),
        (
            HitTarget::Outgoing {
                channel: session.active_channel,
                key: session.selected_key,
            },
            selected.outgoing,
        ),
    ] {
        if point.is_some_and(|point| {
            distance(session.viewport.screen_from_graph(point), cursor) <= 12.0
        }) {
            return Some(target);
        }
    }
    session
        .channels
        .iter()
        .enumerate()
        .flat_map(|(channel_index, channel)| {
            channel
                .keys
                .iter()
                .enumerate()
                .map(move |(key_index, key)| (channel_index, key_index, key))
        })
        .filter_map(|(channel, key_index, key)| {
            let distance = distance(session.viewport.screen_from_graph(key.point), cursor);
            (distance <= 12.0).then_some((
                distance,
                HitTarget::Key {
                    channel,
                    key: key_index,
                },
            ))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, target)| target)
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

pub fn plot_contains(point: [f32; 2]) -> bool {
    point[0] >= PLOT[0]
        && point[0] <= PLOT[0] + PLOT[2]
        && point[1] >= PLOT[1]
        && point[1] <= PLOT[1] + PLOT[3]
}

fn clamp_to_plot(point: [f32; 2]) -> [f32; 2] {
    [
        point[0].clamp(PLOT[0], PLOT[0] + PLOT[2]),
        point[1].clamp(PLOT[1], PLOT[1] + PLOT[3]),
    ]
}

fn normalized_rect(a: [f32; 2], b: [f32; 2]) -> [f32; 4] {
    let x = a[0].min(b[0]);
    let y = a[1].min(b[1]);
    [x, y, (a[0] - b[0]).abs(), (a[1] - b[1]).abs()]
}

fn rect_contains(rect: [f32; 4], point: [f32; 2]) -> bool {
    point[0] >= rect[0]
        && point[0] <= rect[0] + rect[2]
        && point[1] >= rect[1]
        && point[1] <= rect[1] + rect[3]
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Primitive {
    pub bounds: [f32; 4],
    pub color: [f32; 4],
    pub extra: [f32; 4],
    pub shape: u32,
    pub _padding: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct TextPrimitive {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub size: f32,
    pub color: [u8; 4],
    pub monospace: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub primitives: Vec<Primitive>,
    pub texts: Vec<TextPrimitive>,
}

impl Scene {
    fn rect(&mut self, bounds: [f32; 4], color: [f32; 4]) {
        self.primitives.push(Primitive {
            bounds,
            color,
            extra: [0.0; 4],
            shape: 0,
            _padding: [0; 3],
        });
    }

    fn shape(&mut self, bounds: [f32; 4], color: [f32; 4], shape: u32) {
        self.primitives.push(Primitive {
            bounds,
            color,
            extra: [0.0; 4],
            shape,
            _padding: [0; 3],
        });
    }

    fn line(&mut self, start: [f32; 2], end: [f32; 2], width: f32, color: [f32; 4]) {
        self.primitives.push(Primitive {
            bounds: [start[0], start[1], 0.0, 0.0],
            color,
            extra: [end[0], end[1], width * 0.5, 0.0],
            shape: 3,
            _padding: [0; 3],
        });
    }

    fn plot_line(&mut self, start: [f32; 2], end: [f32; 2], width: f32, color: [f32; 4]) {
        if let Some((start, end)) = clip_line_to_plot(start, end) {
            self.line(start, end, width, color);
        }
    }

    fn text(
        &mut self,
        text: impl Into<String>,
        at: [f32; 3],
        size: f32,
        color: [u8; 4],
        monospace: bool,
    ) {
        self.texts.push(TextPrimitive {
            text: text.into(),
            left: at[0],
            top: at[1],
            width: at[2],
            size,
            color,
            monospace,
        });
    }
}

fn cubic(a: GraphPoint, b: GraphPoint, c: GraphPoint, d: GraphPoint, t: f32) -> GraphPoint {
    let u = 1.0 - t;
    GraphPoint::new(
        u * u * u * a.time
            + 3.0 * u * u * t * b.time
            + 3.0 * u * t * t * c.time
            + t * t * t * d.time,
        u * u * u * a.value
            + 3.0 * u * u * t * b.value
            + 3.0 * u * t * t * c.value
            + t * t * t * d.value,
    )
}

fn draw_curve(
    scene: &mut Scene,
    viewport: &GraphViewport,
    channel: &GraphChannel,
    color: [f32; 4],
    width: f32,
) {
    for pair in channel.keys.windows(2) {
        let a = pair[0].point;
        let b = pair[0].outgoing.unwrap_or(a);
        let d = pair[1].point;
        let c = pair[1].incoming.unwrap_or(d);
        let mut previous = viewport.screen_from_graph(a);
        for index in 1..=28 {
            let next = viewport.screen_from_graph(cubic(a, b, c, d, index as f32 / 28.0));
            scene.plot_line(previous, next, width, color);
            previous = next;
        }
    }
}

fn clip_line_to_plot(start: [f32; 2], end: [f32; 2]) -> Option<([f32; 2], [f32; 2])> {
    let min_x = PLOT[0];
    let max_x = PLOT[0] + PLOT[2];
    let min_y = PLOT[1];
    let max_y = PLOT[1] + PLOT[3];
    let delta = [end[0] - start[0], end[1] - start[1]];
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for (p, q) in [
        (-delta[0], start[0] - min_x),
        (delta[0], max_x - start[0]),
        (-delta[1], start[1] - min_y),
        (delta[1], max_y - start[1]),
    ] {
        if p.abs() <= f32::EPSILON {
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = q / p;
        if p < 0.0 {
            low = low.max(ratio);
        } else {
            high = high.min(ratio);
        }
        if low > high {
            return None;
        }
    }
    Some((
        [start[0] + low * delta[0], start[1] + low * delta[1]],
        [start[0] + high * delta[0], start[1] + high * delta[1]],
    ))
}

pub fn build_scene(session: &GraphSession) -> Scene {
    let bg = [0.018, 0.019, 0.021, 1.0];
    let panel = [0.035, 0.037, 0.041, 1.0];
    let raised = [0.055, 0.058, 0.064, 1.0];
    let line = [0.105, 0.11, 0.12, 1.0];
    let line2 = [0.22, 0.23, 0.25, 1.0];
    let ink = [0.83, 0.84, 0.86, 1.0];
    let active = [0.89, 0.48, 0.12, 1.0];
    let playhead = [0.22, 0.56, 0.92, 1.0];
    let mut scene = Scene::default();
    scene.rect([0.0, 0.0, WIDTH, HEIGHT], bg);
    scene.rect([0.0, 0.0, WIDTH, 28.0], panel);
    scene.rect([0.0, 28.0, WIDTH, 28.0], raised);
    scene.rect(
        [
            0.0,
            HEADER_HEIGHT,
            CHANNEL_WIDTH,
            HEIGHT - HEADER_HEIGHT - STATUS_HEIGHT,
        ],
        panel,
    );
    scene.rect([0.0, HEIGHT - STATUS_HEIGHT, WIDTH, STATUS_HEIGHT], panel);
    scene.line([0.0, 28.0], [WIDTH, 28.0], 1.0, line2);
    scene.line([0.0, HEADER_HEIGHT], [WIDTH, HEADER_HEIGHT], 1.0, line2);
    scene.line(
        [CHANNEL_WIDTH, HEADER_HEIGHT],
        [CHANNEL_WIDTH, HEIGHT - STATUS_HEIGHT],
        1.0,
        line2,
    );
    scene.line(
        [0.0, HEIGHT - STATUS_HEIGHT],
        [WIDTH, HEIGHT - STATUS_HEIGHT],
        1.0,
        line2,
    );

    scene.text(
        "Graph Editor",
        [9.0, 8.0, 105.0],
        10.0,
        [225, 226, 230, 255],
        false,
    );
    scene.text(
        "View    Select    Channel    Key",
        [134.0, 9.0, 300.0],
        9.0,
        [190, 192, 198, 255],
        false,
    );
    scene.text(
        "Pivot: Median",
        [720.0, 9.0, 110.0],
        8.0,
        [155, 158, 166, 255],
        false,
    );
    scene.text(
        "Snap: Frame",
        [846.0, 9.0, 95.0],
        8.0,
        [155, 158, 166, 255],
        false,
    );
    scene.text(
        "Normalize",
        [970.0, 9.0, 90.0],
        8.0,
        [155, 158, 166, 255],
        false,
    );
    for (index, label) in ["Select", "Cursor", "Handle", "Frame All", "Frame Selected"]
        .iter()
        .enumerate()
    {
        let x = 9.0 + index as f32 * 91.0;
        scene.rect(
            [x, 32.0, 83.0, 20.0],
            if index == 0 {
                [0.10, 0.072, 0.042, 1.0]
            } else {
                panel
            },
        );
        scene.line(
            [x, 52.0],
            [x + 83.0, 52.0],
            1.0,
            if index == 0 { active } else { line },
        );
        scene.text(
            *label,
            [x + 7.0, 38.0, 72.0],
            8.0,
            if index == 0 {
                [238, 205, 168, 255]
            } else {
                [165, 168, 176, 255]
            },
            false,
        );
    }
    scene.text(
        "CHANNELS",
        [9.0, 67.0, 100.0],
        8.0,
        [125, 128, 136, 255],
        true,
    );
    scene.text(
        "Filter",
        [166.0, 67.0, 48.0],
        8.0,
        [125, 128, 136, 255],
        false,
    );

    for (index, channel) in session.channels.iter().enumerate() {
        let y = 84.0 + index as f32 * 58.0;
        let selected = index == session.active_channel;
        scene.rect(
            [0.0, y, CHANNEL_WIDTH, 57.0],
            if selected { raised } else { panel },
        );
        scene.rect([8.0, y + 8.0, 4.0, 40.0], channel.color);
        scene.text(
            "▾  ◉  ◇",
            [20.0, y + 19.0, 62.0],
            8.0,
            [145, 148, 156, 255],
            false,
        );
        scene.text(
            channel.object,
            [82.0, y + 10.0, 118.0],
            7.0,
            [125, 128, 136, 255],
            false,
        );
        scene.text(
            channel.parameter,
            [82.0, y + 27.0, 118.0],
            9.0,
            if selected {
                [232, 233, 236, 255]
            } else {
                [174, 177, 184, 255]
            },
            false,
        );
        scene.text(
            channel.keys.len().to_string(),
            [202.0, y + 24.0, 16.0],
            7.0,
            [145, 148, 156, 255],
            true,
        );
        scene.line([0.0, y + 57.0], [CHANNEL_WIDTH, y + 57.0], 1.0, line);
    }

    scene.rect(PLOT, bg);
    for tick in 0..8 {
        let x = PLOT[0] + (tick as f32 + 0.5) * PLOT[2] / 8.0;
        scene.line(
            [x, PLOT[1]],
            [x, PLOT[1] + PLOT[3]],
            0.45,
            [0.067, 0.07, 0.076, 1.0],
        );
    }
    for tick in 0..10 {
        let y = PLOT[1] + (tick as f32 + 0.5) * PLOT[3] / 10.0;
        scene.line(
            [PLOT[0], y],
            [PLOT[0] + PLOT[2], y],
            0.45,
            [0.067, 0.07, 0.076, 1.0],
        );
    }
    for tick in 0..=8 {
        let x = PLOT[0] + tick as f32 * PLOT[2] / 8.0;
        scene.line(
            [x, PLOT[1]],
            [x, PLOT[1] + PLOT[3]],
            if tick % 2 == 0 { 1.0 } else { 0.6 },
            if tick % 2 == 0 { line2 } else { line },
        );
        let time = session.viewport.graph_from_screen([x, PLOT[1]]).time;
        scene.text(
            format!("{time:.1}"),
            [x - 17.0, PLOT[1] + PLOT[3] + 7.0, 36.0],
            7.0,
            [118, 121, 129, 255],
            true,
        );
    }
    for tick in 0..=10 {
        let y = PLOT[1] + tick as f32 * PLOT[3] / 10.0;
        scene.line(
            [PLOT[0], y],
            [PLOT[0] + PLOT[2], y],
            if tick % 5 == 0 { 1.0 } else { 0.55 },
            if tick % 5 == 0 { line2 } else { line },
        );
        scene.text(
            format!(
                "{:.0}",
                session.viewport.graph_from_screen([PLOT[0], y]).value
            ),
            [PLOT[0] - 43.0, y - 4.0, 34.0],
            7.0,
            [118, 121, 129, 255],
            true,
        );
    }
    for (index, channel) in session.channels.iter().enumerate().rev() {
        let color = if index == session.active_channel {
            channel.color
        } else {
            [
                channel.color[0] * 0.55,
                channel.color[1] * 0.55,
                channel.color[2] * 0.55,
                0.72,
            ]
        };
        draw_curve(
            &mut scene,
            &session.viewport,
            channel,
            color,
            if index == session.active_channel {
                2.8
            } else {
                1.3
            },
        );
    }

    let channel = &session.channels[session.active_channel];
    let selected = &channel.keys[session.selected_key];
    if session.is_selected(session.active_channel, session.selected_key) {
        for handle in [selected.incoming, selected.outgoing].into_iter().flatten() {
            let origin = session.viewport.screen_from_graph(selected.point);
            let point = session.viewport.screen_from_graph(handle);
            scene.plot_line(origin, point, 1.2, ink);
            if plot_contains(point) {
                scene.shape([point[0] - 5.5, point[1] - 5.5, 11.0, 11.0], bg, 1);
                scene.shape([point[0] - 4.1, point[1] - 4.1, 8.2, 8.2], ink, 1);
            }
        }
    }
    for (channel_index, graph_channel) in session.channels.iter().enumerate() {
        for (key_index, key) in graph_channel.keys.iter().enumerate() {
            let point = session.viewport.screen_from_graph(key.point);
            if !plot_contains(point) {
                continue;
            }
            let is_selected = session.is_selected(channel_index, key_index);
            scene.shape(
                [point[0] - 6.0, point[1] - 6.0, 12.0, 12.0],
                if is_selected { active } else { panel },
                2,
            );
            if !is_selected {
                scene.shape(
                    [point[0] - 4.0, point[1] - 4.0, 8.0, 8.0],
                    graph_channel.color,
                    2,
                );
            }
        }
    }
    let playhead_x = session
        .viewport
        .screen_from_graph(GraphPoint::new(54.2, 0.0))[0];
    if (PLOT[0]..=PLOT[0] + PLOT[2]).contains(&playhead_x) {
        scene.line(
            [playhead_x, PLOT[1]],
            [playhead_x, PLOT[1] + PLOT[3]],
            1.2,
            playhead,
        );
        scene.shape([playhead_x - 7.0, PLOT[1] - 1.0, 14.0, 11.0], playhead, 2);
    }
    if let Some(rect) = session.marquee_rect() {
        scene.rect(rect, [0.22, 0.56, 0.92, 0.12]);
        scene.line(
            [rect[0], rect[1]],
            [rect[0] + rect[2], rect[1]],
            1.0,
            playhead,
        );
        scene.line(
            [rect[0] + rect[2], rect[1]],
            [rect[0] + rect[2], rect[1] + rect[3]],
            1.0,
            playhead,
        );
        scene.line(
            [rect[0] + rect[2], rect[1] + rect[3]],
            [rect[0], rect[1] + rect[3]],
            1.0,
            playhead,
        );
        scene.line(
            [rect[0], rect[1] + rect[3]],
            [rect[0], rect[1]],
            1.0,
            playhead,
        );
    }
    scene.text(
        channel.unit,
        [PLOT[0] + 8.0, PLOT[1] + 8.0, 22.0],
        8.0,
        [125, 128, 136, 255],
        true,
    );

    scene.text(
        session.status,
        [9.0, HEIGHT - 17.0, 420.0],
        8.0,
        [175, 178, 185, 255],
        false,
    );
    scene.text(
        format!(
            "Time {:.2}   Value {:.1}{}   Selected {}",
            selected.point.time,
            selected.point.value,
            channel.unit,
            session.selection.len()
        ),
        [690.0, HEIGHT - 17.0, 260.0],
        8.0,
        [175, 178, 185, 255],
        true,
    );
    scene.text(
        format!("Commits {}", session.semantic_commit_count),
        [983.0, HEIGHT - 17.0, 105.0],
        8.0,
        [220, 222, 226, 255],
        true,
    );
    scene
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub ticket: &'static str,
    pub status: &'static str,
    pub adapter: String,
    pub backend: String,
    pub channel_count: usize,
    pub key_count: usize,
    pub primitive_count: usize,
    pub text_run_count: usize,
    pub semantic_commit_count: u32,
    pub selected_key_count: usize,
    pub navigation_change_count: u32,
    pub selection_change_count: u32,
    pub readback_count: u32,
    pub hot_drag_resource_creation_count: u32,
    pub present_count: u32,
    pub pass: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_matches_fixed_react_channels_and_keys() {
        let channels = fixture_channels();
        assert_eq!(channels.len(), 3);
        assert_eq!(
            channels
                .iter()
                .map(|channel| channel.keys.len())
                .sum::<usize>(),
            9
        );
        assert_eq!(channels[0].parameter, "Intensity");
        assert_eq!(channels[1].parameter, "Spread");
        assert_eq!(channels[2].parameter, "Depth");
    }

    #[test]
    fn drag_is_transient_release_exactly_once() {
        let mut session = GraphSession::default();
        let before = session.channels[0].keys.clone();
        assert!(session.begin_drag(HitTarget::Key { channel: 0, key: 1 }, 7));
        assert!(session.update_drag(GraphPoint::new(53.5, 64.0)));
        assert_ne!(session.channels[0].keys, before);
        assert_eq!(session.semantic_commit_count, 0);
        assert!(session.release(7));
        assert_eq!(session.semantic_commit_count, 1);
        assert!(!session.release(7));
        assert_eq!(session.semantic_commit_count, 1);
    }

    #[test]
    fn cancel_restores_and_time_never_crosses_neighbor() {
        let mut session = GraphSession::default();
        let before = session.channels[0].keys.clone();
        session.begin_drag(HitTarget::Key { channel: 0, key: 1 }, 9);
        session.update_drag(GraphPoint::new(99.0, -99.0));
        assert!(session.channels[0].keys[1].point.time < session.channels[0].keys[2].point.time);
        assert_eq!(session.channels[0].keys[1].point.value, 0.0);
        assert!(session.cancel());
        assert_eq!(session.channels[0].keys, before);
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn scene_has_blender_like_information_regions_without_blender_assets() {
        let scene = build_scene(&GraphSession::default());
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>();
        for required in [
            "Graph Editor",
            "View    Select    Channel    Key",
            "CHANNELS",
            "Intensity",
            "Spread",
            "Depth",
        ] {
            assert!(labels.contains(&required));
        }
        assert!(
            scene.primitives.len() > 200,
            "primitive count was {}",
            scene.primitives.len()
        );
    }

    #[test]
    fn headless_zoom_preserves_cursor_anchor_and_never_commits() {
        let mut session = GraphSession::default();
        let anchor = [612.0, 318.0];
        let before = session.viewport.graph_from_screen(anchor);
        assert!(session.zoom_about_screen(anchor, 1.7));
        let after = session.viewport.graph_from_screen(anchor);
        assert!((before.time - after.time).abs() < 0.0001);
        assert!((before.value - after.value).abs() < 0.0001);
        assert_eq!(session.semantic_commit_count, 0);
        assert_eq!(session.navigation_change_count, 1);
    }

    #[test]
    fn headless_pan_changes_only_the_project_session_view() {
        let mut session = GraphSession::default();
        let before = session.viewport.visible_ranges();
        assert!(session.pan_by_view([96.0, -48.0]));
        let after = session.viewport.visible_ranges();
        assert_ne!(after, before);
        assert_eq!(session.semantic_commit_count, 0);
        assert_eq!(session.navigation_change_count, 1);
    }

    #[test]
    fn marquee_and_additive_selection_use_stable_ids_without_document_commit() {
        let mut session = GraphSession::default();
        let i2 = session
            .viewport
            .screen_from_graph(session.channels[0].keys[2].point);
        let s1 = session
            .viewport
            .screen_from_graph(session.channels[1].keys[1].point);
        let start = [i2[0].min(s1[0]) - 8.0, i2[1].min(s1[1]) - 8.0];
        let end = [i2[0].max(s1[0]) + 8.0, i2[1].max(s1[1]) + 8.0];
        assert!(session.begin_marquee(start, false));
        assert!(session.update_marquee(end));
        assert_eq!(session.semantic_commit_count, 0);
        assert!(session.release_marquee());
        assert!(session.selection.contains(&KeyId {
            channel: "intensity",
            key: "i2"
        }));
        assert!(session.selection.contains(&KeyId {
            channel: "spread",
            key: "s1"
        }));

        assert!(session.begin_drag_at(
            HitTarget::Key { channel: 2, key: 0 },
            41,
            Some(
                session
                    .viewport
                    .screen_from_graph(session.channels[2].keys[0].point)
            ),
            true,
        ));
        assert!(session.release(41));
        assert!(session.selection.contains(&KeyId {
            channel: "depth",
            key: "d0"
        }));
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn drag_threshold_and_frame_snap_are_deterministic() {
        let mut session = GraphSession::default();
        let before = session.channels[0].keys[1].point;
        let start = session.viewport.screen_from_graph(before);
        assert!(session.begin_drag_at(
            HitTarget::Key { channel: 0, key: 1 },
            51,
            Some(start),
            false,
        ));
        assert!(!session.update_drag_screen([start[0] + 2.0, start[1] + 1.0]));
        assert_eq!(session.channels[0].keys[1].point, before);
        let target = session
            .viewport
            .screen_from_graph(GraphPoint::new(53.47, 64.0));
        assert!(session.update_drag_screen(target));
        assert_eq!(session.channels[0].keys[1].point.time, 53.5);
        assert!(session.release(51));
        assert_eq!(session.semantic_commit_count, 1);
    }

    #[test]
    fn cancel_restores_marquee_and_non_finite_navigation_is_ignored() {
        let mut session = GraphSession::default();
        let before = session.selection.clone();
        assert!(session.begin_marquee([400.0, 150.0], false));
        assert!(session.update_marquee([800.0, 500.0]));
        assert_ne!(session.selection, before);
        assert!(session.cancel());
        assert_eq!(session.selection, before);
        assert!(!session.pan_by_view([f32::NAN, 0.0]));
        assert!(!session.zoom_about_screen([500.0, 300.0], f64::INFINITY));
        assert_eq!(session.navigation_change_count, 0);
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn fit_selection_changes_only_project_session_view() {
        let mut session = GraphSession::default();
        let keys_before = session.channels.clone();
        assert!(session.fit_selection());
        let (time, value) = session.viewport.visible_ranges();
        assert!(time[0] < 53.24 && time[1] > 53.24);
        assert!(value[0] < 82.0 && value[1] > 82.0);
        assert_eq!(session.channels, keys_before);
        assert_eq!(session.semantic_commit_count, 0);
    }
}
