//! G0-9 native Easing popupのwindow/GPU非依存受入model。
//! 製品Document、D2、User settings codecの代替ではない。

use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const POPUP_WIDTH: f64 = 510.0;
pub const POPUP_HEIGHT: f64 = 284.0;
pub const GRAPH_RECT: [f32; 4] = [174.0, 37.0, 226.0, 238.0];
pub const SAVE_RECT: [f32; 4] = [61.67, 87.0, 50.67, 37.0];
pub const FAVORITE_RECT: [f32; 4] = [457.0, 4.0, 26.0, 21.0];
pub const CLOSE_RECT: [f32; 4] = [484.0, 4.0, 21.0, 21.0];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Bezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Bezier {
    pub const LINEAR: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const SMOOTH: Self = Self::new(0.4, 0.0, 0.2, 1.0);
    pub const EASE_IN: Self = Self::new(0.42, 0.0, 1.0, 1.0);
    pub const EASE_OUT: Self = Self::new(0.0, 0.0, 0.58, 1.0);

    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn with_handle(self, handle: Handle, point: [f32; 2]) -> Self {
        let x = point[0].clamp(0.0, 1.0);
        let y = point[1].clamp(-1.0, 2.0);
        match handle {
            Handle::Start => Self::new(x, y, self.x2, self.y2),
            Handle::End => Self::new(self.x1, self.y1, x, y),
        }
    }

    pub fn point(self, t: f32) -> [f32; 2] {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let c1 = 3.0 * u * u * t;
        let c2 = 3.0 * u * t * t;
        [
            c1 * self.x1 + c2 * self.x2 + t * t * t,
            c1 * self.y1 + c2 * self.y2 + t * t * t,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogicalRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalPlacement {
    Below,
    Above,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub vertical: VerticalPlacement,
}

pub fn place_popup(
    host_content_origin: [i32; 2],
    anchor: LogicalRect,
    scale_factor: f64,
    work_area: PhysicalRect,
) -> Option<Placement> {
    if !scale_factor.is_finite()
        || scale_factor <= 0.0
        || anchor.width < 0.0
        || anchor.height < 0.0
        || work_area.width == 0
        || work_area.height == 0
    {
        return None;
    }
    let width = (POPUP_WIDTH * scale_factor).round().max(1.0) as u32;
    let height = (POPUP_HEIGHT * scale_factor).round().max(1.0) as u32;
    let anchor_left = host_content_origin[0] + (anchor.x * scale_factor).round() as i32;
    let anchor_top = host_content_origin[1] + (anchor.y * scale_factor).round() as i32;
    let anchor_bottom = anchor_top + (anchor.height * scale_factor).round() as i32;
    let work_right = work_area.x.saturating_add(work_area.width as i32);
    let work_bottom = work_area.y.saturating_add(work_area.height as i32);
    let below_fits = anchor_bottom.saturating_add(height as i32) <= work_bottom;
    let vertical = if below_fits {
        VerticalPlacement::Below
    } else {
        VerticalPlacement::Above
    };
    let desired_y = if below_fits {
        anchor_bottom
    } else {
        anchor_top.saturating_sub(height as i32)
    };
    let max_x = work_right.saturating_sub(width as i32).max(work_area.x);
    let max_y = work_bottom.saturating_sub(height as i32).max(work_area.y);
    Some(Placement {
        x: anchor_left.clamp(work_area.x, max_x),
        y: desired_y.clamp(work_area.y, max_y),
        width,
        height,
        vertical,
    })
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserPreset {
    pub id: String,
    pub name: String,
    pub curve: Bezier,
    pub order: u32,
    pub favorite: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SpikePresetStore {
    pub presets: Vec<UserPreset>,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("preset store IO failed")]
    Io(#[from] std::io::Error),
    #[error("preset store JSON failed")]
    Json(#[from] serde_json::Error),
}

impl SpikePresetStore {
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    pub fn save(&self, path: &Path) -> Result<(), StoreError> {
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    pub fn save_curve(&mut self, name: impl Into<String>, curve: Bezier) -> &UserPreset {
        let order = self.presets.len() as u32;
        let id = format!("spike-user-{}", order + 1);
        self.presets.push(UserPreset {
            id,
            name: name.into(),
            curve,
            order,
            favorite: false,
        });
        self.presets.last().expect("just pushed")
    }

    pub fn set_favorite(&mut self, id: &str) -> bool {
        let mut found = false;
        for preset in &mut self.presets {
            preset.favorite = preset.id == id;
            found |= preset.favorite;
        }
        found
    }
}

pub fn thumbnail_points(curve: Bezier, sample_count: usize) -> Vec<[f32; 2]> {
    let count = sample_count.max(2);
    (0..count)
        .map(|index| curve.point(index as f32 / (count - 1) as f32))
        .collect()
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
pub struct PopupScene {
    pub primitives: Vec<Primitive>,
    pub texts: Vec<TextPrimitive>,
}

impl PopupScene {
    fn rect(&mut self, rect: [f32; 4], color: [f32; 4]) {
        self.primitives.push(Primitive {
            bounds: rect,
            color,
            extra: [0.0; 4],
            shape: 0,
            _padding: [0; 3],
        });
    }

    fn shape(&mut self, rect: [f32; 4], color: [f32; 4], shape: u32) {
        self.primitives.push(Primitive {
            bounds: rect,
            color,
            extra: [0.0; 4],
            shape,
            _padding: [0; 3],
        });
    }

    fn line(&mut self, from: [f32; 2], to: [f32; 2], width: f32, color: [f32; 4]) {
        self.primitives.push(Primitive {
            bounds: [from[0], from[1], 0.0, 0.0],
            color,
            extra: [to[0], to[1], width * 0.5, 0.0],
            shape: 3,
            _padding: [0; 3],
        });
    }

    fn dashed_line(
        &mut self,
        from: [f32; 2],
        to: [f32; 2],
        width: f32,
        dash: f32,
        gap: f32,
        color: [f32; 4],
    ) {
        let delta = [to[0] - from[0], to[1] - from[1]];
        let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
        if length <= f32::EPSILON {
            return;
        }
        let direction = [delta[0] / length, delta[1] / length];
        let mut cursor = 0.0;
        while cursor < length {
            let end = (cursor + dash).min(length);
            self.line(
                [
                    from[0] + direction[0] * cursor,
                    from[1] + direction[1] * cursor,
                ],
                [from[0] + direction[0] * end, from[1] + direction[1] * end],
                width,
                color,
            );
            cursor += dash + gap;
        }
    }

    fn outline(&mut self, rect: [f32; 4], color: [f32; 4]) {
        let [x, y, w, h] = rect;
        self.rect([x, y, w, 1.0], color);
        self.rect([x, y + h - 1.0, w, 1.0], color);
        self.rect([x, y, 1.0, h], color);
        self.rect([x + w - 1.0, y, 1.0, h], color);
    }

    fn text(
        &mut self,
        value: impl Into<String>,
        bounds: [f32; 3],
        size: f32,
        color: [u8; 4],
        monospace: bool,
    ) {
        let [left, top, width] = bounds;
        self.texts.push(TextPrimitive {
            text: value.into(),
            left,
            top,
            width,
            size,
            color,
            monospace,
        });
    }
}

const BUILTIN_PRESETS: [(&str, Bezier); 4] = [
    ("Linear", Bezier::LINEAR),
    ("Smooth", Bezier::SMOOTH),
    ("Ease In", Bezier::EASE_IN),
    ("Ease Out", Bezier::EASE_OUT),
];

pub fn builtin_presets() -> &'static [(&'static str, Bezier)] {
    &BUILTIN_PRESETS
}

pub fn preset_rect(index: usize) -> [f32; 4] {
    let column = index % 3;
    let row = index / 3;
    [
        8.0 + column as f32 * 53.67,
        47.0 + row as f32 * 40.0,
        50.67,
        37.0,
    ]
}

pub fn hit_preset_index(point: [f32; 2]) -> Option<usize> {
    (0..BUILTIN_PRESETS.len()).find(|index| contains(preset_rect(*index), point))
}

pub fn hit_preset(point: [f32; 2]) -> Option<Bezier> {
    BUILTIN_PRESETS
        .iter()
        .enumerate()
        .find(|(index, _)| contains(preset_rect(*index), point))
        .map(|(_, (_, curve))| *curve)
}

pub fn hit_action(point: [f32; 2]) -> Option<PopupAction> {
    if contains(SAVE_RECT, point) {
        Some(PopupAction::SavePreset)
    } else if contains(FAVORITE_RECT, point) {
        Some(PopupAction::FavoriteLatest)
    } else if contains(CLOSE_RECT, point) {
        Some(PopupAction::Close)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupAction {
    SavePreset,
    FavoriteLatest,
    Close,
}

pub fn graph_point(curve_point: [f32; 2]) -> [f32; 2] {
    let [x, y, width, height] = GRAPH_RECT;
    let y_min = -0.35;
    let y_max = 1.35;
    [
        x + curve_point[0] * width,
        y + (y_max - curve_point[1]) / (y_max - y_min) * height,
    ]
}

pub fn curve_point_from_graph(point: [f32; 2]) -> [f32; 2] {
    let [x, y, width, height] = GRAPH_RECT;
    let y_min = -0.35;
    let y_max = 1.35;
    [
        ((point[0] - x) / width).clamp(0.0, 1.0),
        (y_max - (point[1] - y) / height * (y_max - y_min)).clamp(-1.0, 2.0),
    ]
}

pub fn hit_handle(curve: Bezier, point: [f32; 2]) -> Option<Handle> {
    let first = graph_point([curve.x1, curve.y1]);
    let second = graph_point([curve.x2, curve.y2]);
    let distance_sq = |target: [f32; 2]| {
        let dx = target[0] - point[0];
        let dy = target[1] - point[1];
        dx * dx + dy * dy
    };
    if distance_sq(first) <= 100.0 {
        Some(Handle::Start)
    } else if distance_sq(second) <= 100.0 {
        Some(Handle::End)
    } else {
        None
    }
}

fn contains(rect: [f32; 4], point: [f32; 2]) -> bool {
    point[0] >= rect[0]
        && point[0] <= rect[0] + rect[2]
        && point[1] >= rect[1]
        && point[1] <= rect[1] + rect[3]
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopupVisualState {
    pub hovered_preset: Option<usize>,
    pub hovered_handle: Option<Handle>,
    pub focused_handle: Option<Handle>,
}

const ADVANCED_THUMBNAILS: [(&str, &[[f32; 2]]); 6] = [
    (
        "Bounce",
        &[
            [0.0, 0.0],
            [0.30, 1.0],
            [0.55, 0.48],
            [0.72, 1.0],
            [0.86, 0.76],
            [1.0, 1.0],
        ],
    ),
    (
        "Elastic",
        &[
            [0.0, 0.0],
            [0.26, 1.18],
            [0.46, 0.78],
            [0.64, 1.08],
            [0.82, 0.94],
            [1.0, 1.0],
        ],
    ),
    (
        "CYCLIC / SIN",
        &[
            [0.0, 0.0],
            [0.16, 1.0],
            [0.32, 0.0],
            [0.48, 1.0],
            [0.64, 0.0],
            [0.82, 1.0],
            [1.0, 1.0],
        ],
    ),
    (
        "Random",
        &[
            [0.0, 0.0],
            [0.14, 0.28],
            [0.27, 0.12],
            [0.42, 0.62],
            [0.57, 0.36],
            [0.72, 0.82],
            [0.86, 0.66],
            [1.0, 1.0],
        ],
    ),
    (
        "Steps",
        &[
            [0.0, 0.0],
            [0.24, 0.0],
            [0.24, 0.25],
            [0.49, 0.25],
            [0.49, 0.52],
            [0.74, 0.52],
            [0.74, 0.78],
            [1.0, 0.78],
            [1.0, 1.0],
        ],
    ),
    (
        "Elastic Steps",
        &[
            [0.0, 0.0],
            [0.24, 0.0],
            [0.24, 0.43],
            [0.38, 0.28],
            [0.5, 0.28],
            [0.5, 0.78],
            [0.64, 0.6],
            [0.75, 0.6],
            [0.75, 1.08],
            [0.88, 0.94],
            [1.0, 1.0],
        ],
    ),
];

fn advanced_rect(index: usize) -> [f32; 4] {
    let column = index % 3;
    let row = index / 3;
    [
        8.0 + column as f32 * 53.67,
        140.0 + row as f32 * 40.0,
        50.67,
        37.0,
    ]
}

fn draw_thumbnail(scene: &mut PopupScene, rect: [f32; 4], points: &[[f32; 2]], color: [f32; 4]) {
    let map = |point: [f32; 2]| {
        [
            rect[0] + 5.0 + point[0] * (rect[2] - 10.0),
            rect[1] + 4.0 + (1.0 - point[1].clamp(-0.2, 1.2)) / 1.4 * 22.0,
        ]
    };
    for pair in points.windows(2) {
        scene.line(map(pair[0]), map(pair[1]), 1.1, color);
    }
}

fn draw_value_card(
    scene: &mut PopupScene,
    index: usize,
    point: [f32; 2],
    active: bool,
    panel: [f32; 4],
    line: [f32; 4],
    accent: [f32; 4],
) {
    let rect = [408.0, 96.0 + (index - 1) as f32 * 64.0, 92.0, 56.0];
    scene.rect(rect, panel);
    scene.outline(rect, if active { accent } else { line });
    let center = [rect[0] + 15.0, rect[1] + 28.0];
    scene.shape([center[0] - 8.0, center[1] - 8.0, 16.0, 16.0], accent, 1);
    scene.shape([center[0] - 6.8, center[1] - 6.8, 13.6, 13.6], panel, 1);
    scene.text(
        index.to_string(),
        [center[0] - 2.3, center[1] - 4.2, 7.0],
        7.0,
        [216, 181, 116, 255],
        true,
    );
    scene.text(
        "x",
        [rect[0] + 29.0, rect[1] + 12.0, 8.0],
        6.0,
        [146, 146, 146, 255],
        true,
    );
    scene.text(
        format!("{:.2}", point[0]),
        [rect[0] + 41.0, rect[1] + 10.0, 42.0],
        7.0,
        [216, 181, 116, 255],
        true,
    );
    scene.text(
        "y",
        [rect[0] + 29.0, rect[1] + 32.0, 8.0],
        6.0,
        [146, 146, 146, 255],
        true,
    );
    scene.text(
        format!("{:.2}", point[1]),
        [rect[0] + 41.0, rect[1] + 30.0, 42.0],
        7.0,
        [216, 181, 116, 255],
        true,
    );
}

pub fn build_popup_scene(
    session: &PopupSession,
    store: &SpikePresetStore,
    visual: PopupVisualState,
) -> PopupScene {
    // 固定React oracle (56c318ed)のsRGB tokenをlinear surface値へ変換した比較色。
    let bg = [0.0070, 0.0070, 0.0070, 1.0];
    let panel = [0.0103, 0.0103, 0.0103, 1.0];
    let raised = [0.0160, 0.0160, 0.0160, 1.0];
    let hover = [0.0252, 0.0252, 0.0252, 1.0];
    let line = [0.0437, 0.0437, 0.0437, 1.0];
    let line2 = [0.1384, 0.1384, 0.1384, 1.0];
    let ink = [0.8714, 0.8714, 0.8714, 1.0];
    let muted = [0.2874, 0.2874, 0.2874, 1.0];
    let accent = [0.6867, 0.4621, 0.1746, 1.0];

    let mut scene = PopupScene::default();
    scene.rect([0.0, 0.0, POPUP_WIDTH as f32, POPUP_HEIGHT as f32], panel);
    scene.outline([0.0, 0.0, POPUP_WIDTH as f32, POPUP_HEIGHT as f32], line2);
    scene.rect([1.0, 1.0, 508.0, 28.0], raised);
    scene.rect([1.0, 28.0, 508.0, 1.0], line);
    scene.text(
        "Pulse rings · Intensity",
        [8.0, 9.0, 350.0],
        8.0,
        [240, 240, 240, 255],
        true,
    );
    scene.text("•••", [461.0, 7.0, 22.0], 8.0, [146, 146, 146, 255], true);
    scene.text("×", [490.0, 6.0, 13.0], 10.0, [146, 146, 146, 255], false);

    scene.text("BEZIER", [8.0, 36.0, 80.0], 6.0, [146, 146, 146, 255], true);
    for (index, (name, curve)) in BUILTIN_PRESETS.iter().enumerate() {
        let rect = preset_rect(index);
        let selected = session.curve() == *curve;
        scene.rect(
            rect,
            if visual.hovered_preset == Some(index) {
                hover
            } else {
                bg
            },
        );
        scene.outline(rect, if selected { accent } else { line });
        if selected {
            scene.rect(
                [rect[0] + 1.0, rect[1] + rect[3] - 3.0, rect[2] - 2.0, 2.0],
                accent,
            );
        }
        let points = thumbnail_points(*curve, 20);
        draw_thumbnail(
            &mut scene,
            rect,
            &points,
            if selected { accent } else { muted },
        );
        scene.text(
            *name,
            [rect[0] + 2.0, rect[1] + 29.0, rect[2] - 4.0],
            5.0,
            if selected {
                [216, 181, 116, 255]
            } else {
                [146, 146, 146, 255]
            },
            true,
        );
        if index == 1 && !store.presets.iter().any(|preset| preset.favorite) {
            scene.text(
                "◎",
                [rect[0] + rect[2] - 10.0, rect[1] + 1.0, 8.0],
                7.0,
                [240, 240, 240, 255],
                true,
            );
        }
    }

    let my_rect = SAVE_RECT;
    scene.rect(my_rect, bg);
    scene.outline(
        my_rect,
        if store.presets.last().is_some_and(|preset| preset.favorite) {
            accent
        } else {
            line
        },
    );
    if let Some(preset) = store.presets.last() {
        let points = thumbnail_points(preset.curve, 20);
        draw_thumbnail(&mut scene, my_rect, &points, accent);
        if preset.favorite {
            scene.text(
                "◎",
                [my_rect[0] + my_rect[2] - 10.0, my_rect[1] + 1.0, 8.0],
                7.0,
                [240, 240, 240, 255],
                true,
            );
        }
    } else {
        for offset in [0.0, 4.0, 8.0] {
            scene.line(
                [my_rect[0] + 7.0, my_rect[1] + 22.0 - offset],
                [my_rect[0] + 42.0, my_rect[1] + 8.0 - offset * 0.3],
                0.8,
                muted,
            );
        }
    }
    scene.text(
        format!(
            "MY{}",
            if store.presets.is_empty() {
                String::new()
            } else {
                format!(" · {}", store.presets.len())
            }
        ),
        [my_rect[0] + 2.0, my_rect[1] + 29.0, my_rect[2] - 4.0],
        5.0,
        [146, 146, 146, 255],
        true,
    );

    scene.text(
        "ADVANCED · INTERVAL",
        [8.0, 130.0, 148.0],
        6.0,
        [146, 146, 146, 255],
        true,
    );
    for (index, (name, points)) in ADVANCED_THUMBNAILS.iter().enumerate() {
        let rect = advanced_rect(index);
        scene.rect(rect, bg);
        scene.outline(rect, line);
        draw_thumbnail(&mut scene, rect, points, muted);
        scene.text(
            *name,
            [rect[0] + 2.0, rect[1] + 29.0, rect[2] - 4.0],
            4.6,
            [146, 146, 146, 255],
            true,
        );
    }

    scene.rect(GRAPH_RECT, bg);
    scene.outline(GRAPH_RECT, line);
    for division in 1..4 {
        let x = GRAPH_RECT[0] + GRAPH_RECT[2] * division as f32 / 4.0;
        scene.line(
            [x, GRAPH_RECT[1]],
            [x, GRAPH_RECT[1] + GRAPH_RECT[3]],
            0.55,
            line,
        );
    }
    for division in 1..5 {
        let y = GRAPH_RECT[1] + GRAPH_RECT[3] * division as f32 / 5.0;
        scene.line(
            [GRAPH_RECT[0], y],
            [GRAPH_RECT[0] + GRAPH_RECT[2], y],
            0.55,
            line,
        );
    }
    for value in [0.0, 1.0] {
        let y = graph_point([0.0, value])[1];
        scene.dashed_line(
            [GRAPH_RECT[0], y],
            [GRAPH_RECT[0] + GRAPH_RECT[2], y],
            0.8,
            4.0,
            5.0,
            line2,
        );
    }
    let playhead_x = GRAPH_RECT[0] + GRAPH_RECT[2] * 0.46;
    let top = graph_point([0.0, 1.0])[1];
    let bottom = graph_point([0.0, 0.0])[1];
    scene.dashed_line(
        [playhead_x, top],
        [playhead_x, bottom],
        0.65,
        2.0,
        6.0,
        line2,
    );

    let curve = session.curve();
    let start = graph_point([0.0, 0.0]);
    let end = graph_point([1.0, 1.0]);
    let handle1 = graph_point([curve.x1, curve.y1]);
    let handle2 = graph_point([curve.x2, curve.y2]);
    scene.line(start, handle1, 1.25, [accent[0], accent[1], accent[2], 0.5]);
    scene.line(end, handle2, 1.25, [accent[0], accent[1], accent[2], 0.5]);
    let points = thumbnail_points(curve, 96);
    for pair in points.windows(2) {
        scene.line(graph_point(pair[0]), graph_point(pair[1]), 2.75, accent);
    }
    for point in [start, end] {
        scene.shape([point[0] - 3.0, point[1] - 3.0, 6.0, 6.0], accent, 1);
    }
    for (handle, point) in [(Handle::Start, handle1), (Handle::End, handle2)] {
        let engaged = session.active_handle() == Some(handle)
            || visual.hovered_handle == Some(handle)
            || visual.focused_handle == Some(handle);
        if engaged {
            scene.shape([point[0] - 7.0, point[1] - 7.0, 14.0, 14.0], accent, 1);
        }
        scene.shape([point[0] - 5.0, point[1] - 5.0, 10.0, 10.0], ink, 1);
        scene.shape([point[0] - 3.7, point[1] - 3.7, 7.4, 7.4], bg, 1);
    }

    draw_value_card(
        &mut scene,
        1,
        [curve.x1, curve.y1],
        session.active_handle() == Some(Handle::Start)
            || visual.hovered_handle == Some(Handle::Start)
            || visual.focused_handle == Some(Handle::Start),
        bg,
        line,
        accent,
    );
    draw_value_card(
        &mut scene,
        2,
        [curve.x2, curve.y2],
        session.active_handle() == Some(Handle::End)
            || visual.hovered_handle == Some(Handle::End)
            || visual.focused_handle == Some(Handle::End),
        bg,
        line,
        accent,
    );
    scene
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveCommit {
    pub curve: Bezier,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug)]
struct ActiveDrag {
    handle: Handle,
    token: u64,
}

#[derive(Clone, Debug)]
pub struct PopupSession {
    initial_curve: Bezier,
    curve: Bezier,
    revision: u64,
    layout_epoch: u64,
    active_drag: Option<ActiveDrag>,
    released_tokens: HashSet<u64>,
    pub semantic_commit_count: u32,
    pub settings_write_count: u32,
    pub readback_count: u32,
    pub hot_resource_creation_count: u32,
}

impl PopupSession {
    pub fn new(curve: Bezier, revision: u64, layout_epoch: u64) -> Self {
        Self {
            initial_curve: curve,
            curve,
            revision,
            layout_epoch,
            active_drag: None,
            released_tokens: HashSet::new(),
            semantic_commit_count: 0,
            settings_write_count: 0,
            readback_count: 0,
            hot_resource_creation_count: 0,
        }
    }

    pub fn curve(&self) -> Bezier {
        self.curve
    }

    pub fn active_handle(&self) -> Option<Handle> {
        self.active_drag.map(|drag| drag.handle)
    }

    pub fn begin_drag(&mut self, handle: Handle, token: u64) -> bool {
        if self.active_drag.is_some() || self.released_tokens.contains(&token) {
            return false;
        }
        self.active_drag = Some(ActiveDrag { handle, token });
        true
    }

    pub fn update_drag(&mut self, point: [f32; 2]) -> bool {
        let Some(drag) = self.active_drag else {
            return false;
        };
        self.curve = self.curve.with_handle(drag.handle, point);
        true
    }

    pub fn release(
        &mut self,
        token: u64,
        current_revision: u64,
        current_layout_epoch: u64,
    ) -> Option<CurveCommit> {
        let drag = self.active_drag?;
        if drag.token != token
            || self.released_tokens.contains(&token)
            || current_revision != self.revision
            || current_layout_epoch != self.layout_epoch
        {
            self.cancel();
            return None;
        }
        self.active_drag = None;
        self.released_tokens.insert(token);
        self.semantic_commit_count += 1;
        self.initial_curve = self.curve;
        Some(CurveCommit {
            curve: self.curve,
            revision: current_revision,
        })
    }

    pub fn cancel(&mut self) {
        self.curve = self.initial_curve;
        self.active_drag = None;
    }

    pub fn apply_preset(&mut self, curve: Bezier) -> CurveCommit {
        self.curve = curve;
        self.initial_curve = curve;
        self.semantic_commit_count += 1;
        CurveCommit {
            curve,
            revision: self.revision,
        }
    }

    pub fn record_settings_write(&mut self) {
        self.settings_write_count += 1;
    }

    pub fn rebase(&mut self, revision: u64, layout_epoch: u64) {
        self.revision = revision;
        self.layout_epoch = layout_epoch;
        self.initial_curve = self.curve;
        self.active_drag = None;
    }

    pub fn bounded_accessibility(&self) -> Vec<AccessibilityNode> {
        let curve = self.curve;
        vec![
            AccessibilityNode::button("Linear preset"),
            AccessibilityNode::button("Smooth preset"),
            AccessibilityNode::button("Ease In preset"),
            AccessibilityNode::button("Ease Out preset"),
            AccessibilityNode::button("Save current curve"),
            AccessibilityNode::adjustable("Bezier handle 1", curve.x1, curve.y1),
            AccessibilityNode::adjustable("Bezier handle 2", curve.x2, curve.y2),
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityNode {
    pub role: &'static str,
    pub label: &'static str,
    pub value: Option<[f32; 2]>,
}

impl AccessibilityNode {
    fn button(label: &'static str) -> Self {
        Self {
            role: "button",
            label,
            value: None,
        }
    }

    fn adjustable(label: &'static str, x: f32, y: f32) -> Self {
        Self {
            role: "adjustable",
            label,
            value: Some([x, y]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_flips_and_clamps_inside_work_area() {
        let work = PhysicalRect {
            x: 100,
            y: 50,
            width: 1200,
            height: 800,
        };
        let below = place_popup(
            [100, 50],
            LogicalRect {
                x: 40.0,
                y: 20.0,
                width: 24.0,
                height: 24.0,
            },
            1.0,
            work,
        )
        .unwrap();
        assert_eq!(below.vertical, VerticalPlacement::Below);
        assert_eq!((below.x, below.y), (140, 94));

        let above = place_popup(
            [100, 50],
            LogicalRect {
                x: 1180.0,
                y: 760.0,
                width: 24.0,
                height: 24.0,
            },
            1.0,
            work,
        )
        .unwrap();
        assert_eq!(above.vertical, VerticalPlacement::Above);
        assert_eq!(above.x, 790);
        assert!(above.y >= work.y);
        assert!(above.y + above.height as i32 <= work.y + work.height as i32);
    }

    #[test]
    fn drag_is_transient_until_exactly_one_release() {
        let mut session = PopupSession::new(Bezier::SMOOTH, 7, 3);
        assert!(session.begin_drag(Handle::Start, 41));
        assert!(session.update_drag([0.25, -0.4]));
        assert_eq!(session.semantic_commit_count, 0);
        let commit = session.release(41, 7, 3).unwrap();
        assert_eq!(commit.curve, Bezier::new(0.25, -0.4, 0.2, 1.0));
        assert_eq!(session.semantic_commit_count, 1);
        assert!(session.release(41, 7, 3).is_none());
        assert_eq!(session.semantic_commit_count, 1);
    }

    #[test]
    fn escape_and_stale_release_restore_without_commit() {
        let mut escape = PopupSession::new(Bezier::SMOOTH, 7, 3);
        escape.begin_drag(Handle::End, 9);
        escape.update_drag([0.8, 1.6]);
        escape.cancel();
        assert_eq!(escape.curve(), Bezier::SMOOTH);
        assert_eq!(escape.semantic_commit_count, 0);

        let mut stale = PopupSession::new(Bezier::SMOOTH, 7, 3);
        stale.begin_drag(Handle::End, 10);
        stale.update_drag([0.8, 1.6]);
        assert!(stale.release(10, 8, 3).is_none());
        assert_eq!(stale.curve(), Bezier::SMOOTH);
        assert_eq!(stale.semantic_commit_count, 0);
    }

    #[test]
    fn preset_restart_and_thumbnail_share_curve_projection() {
        let dir = std::env::temp_dir().join(format!(
            "motolii-easing-popup-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("spike-presets.json");
        let curve = Bezier::new(0.18, -0.25, 0.72, 1.4);
        let mut store = SpikePresetStore::default();
        let id = store.save_curve("My snap", curve).id.clone();
        assert!(store.set_favorite(&id));
        store.save(&path).unwrap();
        let restarted = SpikePresetStore::load(&path).unwrap();
        assert_eq!(restarted.presets[0].curve, curve);
        assert!(restarted.presets[0].favorite);
        assert_eq!(
            thumbnail_points(restarted.presets[0].curve, 24),
            thumbnail_points(curve, 24)
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn theme_and_dpi_do_not_change_saved_curve_or_thumbnail_geometry() {
        let curve = Bezier::new(0.21, -0.1, 0.78, 1.25);
        let dark_1x = thumbnail_points(curve, 32);
        let light_2x = thumbnail_points(curve, 32);
        assert_eq!(dark_1x, light_2x);
    }

    #[test]
    fn accessibility_is_bounded_and_resources_stay_out_of_hot_drag() {
        let mut session = PopupSession::new(Bezier::SMOOTH, 1, 1);
        session.begin_drag(Handle::Start, 1);
        for index in 0..1_000 {
            session.update_drag([index as f32 / 1_000.0, 0.25]);
        }
        assert!(session.bounded_accessibility().len() <= 16);
        assert_eq!(session.readback_count, 0);
        assert_eq!(session.hot_resource_creation_count, 0);
    }

    #[test]
    fn scene_reproduces_fixed_react_information_hierarchy() {
        let session = PopupSession::new(Bezier::SMOOTH, 1, 1);
        let scene = build_popup_scene(
            &session,
            &SpikePresetStore::default(),
            PopupVisualState::default(),
        );
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>();
        for required in [
            "Pulse rings · Intensity",
            "BEZIER",
            "ADVANCED · INTERVAL",
            "Linear",
            "Smooth",
            "Ease In",
            "Ease Out",
            "MY",
            "Bounce",
            "Elastic",
            "CYCLIC / SIN",
            "Random",
            "Steps",
            "Elastic Steps",
        ] {
            assert!(
                labels.contains(&required),
                "missing oracle label: {required}"
            );
        }
        assert_eq!((POPUP_WIDTH, POPUP_HEIGHT), (510.0, 284.0));
        assert_eq!(preset_rect(3), [8.0, 87.0, 50.67, 37.0]);
        assert_eq!(GRAPH_RECT, [174.0, 37.0, 226.0, 238.0]);
    }
}
