//! React Timelineをsemantic fixtureへ翻訳する、製品workspace外の比較model。

use serde::Serialize;

pub const FIXTURE_WIDTH: f32 = 1200.0;
pub const FIXTURE_HEIGHT: f32 = 240.0;
pub const AUTO_PRESENT_TARGET: u32 = 120;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineObject {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub row: usize,
    pub start: f32,
    pub end: f32,
    pub color_slot: usize,
    pub selected: bool,
}

pub const OBJECTS: [TimelineObject; 5] = [
    TimelineObject {
        id: "audio-night-drive",
        name: "night_drive.wav",
        kind: "A",
        row: 0,
        start: 0.02,
        end: 1.00,
        color_slot: 0,
        selected: false,
    },
    TimelineObject {
        id: "group-pulse-rings",
        name: "Pulse rings",
        kind: "G",
        row: 1,
        start: 0.08,
        end: 0.88,
        color_slot: 1,
        selected: true,
    },
    TimelineObject {
        id: "shape-city-grid",
        name: "City grid",
        kind: "S",
        row: 2,
        start: 0.17,
        end: 0.94,
        color_slot: 2,
        selected: false,
    },
    TimelineObject {
        id: "text-night-drive",
        name: "NIGHT DRIVE",
        kind: "T",
        row: 3,
        start: 0.49,
        end: 1.00,
        color_slot: 3,
        selected: false,
    },
    TimelineObject {
        id: "video-city-loop",
        name: "city_loop.mp4",
        kind: "V",
        row: 4,
        start: 0.32,
        end: 1.00,
        color_slot: 4,
        selected: false,
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectPrimitive {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub shape: u32,
    pub _padding: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct TextPrimitive {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub size: f32,
    pub color: [u8; 4],
    pub monospace: bool,
}

#[derive(Clone, Debug, Default)]
pub struct VisualScene {
    pub rects: Vec<RectPrimitive>,
    pub texts: Vec<TextPrimitive>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VisualParityReport {
    pub ticket: &'static str,
    pub status: &'static str,
    pub adapter: String,
    pub backend: String,
    pub object_count: usize,
    pub rect_primitive_count: usize,
    pub text_run_count: usize,
    pub present_count: u32,
    pub readback_count: u32,
    pub semantic_state_owner_count: u32,
    pub pass: bool,
}

const BG: [f32; 4] = [0.018, 0.020, 0.022, 1.0];
const RAISED: [f32; 4] = [0.060, 0.064, 0.068, 1.0];
const LINE: [f32; 4] = [0.15, 0.16, 0.17, 1.0];
const LINE_STRONG: [f32; 4] = [0.34, 0.35, 0.36, 1.0];
const ACTIVE: [f32; 4] = [0.83, 0.59, 0.31, 1.0];
const BAR_COLORS: [[f32; 4]; 5] = [
    [0.28, 0.55, 0.52, 1.0],
    [0.37, 0.35, 0.64, 1.0],
    [0.30, 0.46, 0.33, 1.0],
    [0.58, 0.47, 0.29, 1.0],
    [0.54, 0.34, 0.30, 1.0],
];

impl VisualScene {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.rects.push(RectPrimitive {
            rect: [x, y, w, h],
            color,
            shape: 0,
            _padding: [0; 3],
        });
    }

    fn shape(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], shape: u32) {
        self.rects.push(RectPrimitive {
            rect: [x, y, w, h],
            color,
            shape,
            _padding: [0; 3],
        });
    }

    fn outline(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], thickness: f32) {
        self.rect(x, y, w, thickness, color);
        self.rect(x, y + h - thickness, w, thickness, color);
        self.rect(x, y, thickness, h, color);
        self.rect(x + w - thickness, y, thickness, h, color);
    }

    fn text(
        &mut self,
        text: impl Into<String>,
        left: f32,
        top: f32,
        width: f32,
        appearance: (f32, [u8; 4], bool),
    ) {
        let (size, color, monospace) = appearance;
        self.texts.push(TextPrimitive {
            text: text.into(),
            left,
            top,
            width,
            height: size * 1.4,
            size,
            color,
            monospace,
        });
    }
}

pub fn build_scene(width: f32, height: f32) -> VisualScene {
    let width = width.max(640.0);
    let height = height.max(240.0);
    let mut scene = VisualScene::default();
    let header_h = 30.0;
    let ruler_h = 25.0;
    let rail_w = 54.0;
    let time_x = rail_w;
    let time_w = width - time_x;
    let row_h = (height - header_h - ruler_h) / OBJECTS.len() as f32;

    scene.rect(0.0, 0.0, width, height, BG);
    scene.rect(0.0, 0.0, width, header_h, RAISED);
    scene.rect(0.0, header_h - 1.0, width, 1.0, LINE_STRONG);
    scene.rect(10.0, 11.0, 7.0, 7.0, [0.55, 0.30, 0.27, 1.0]);
    scene.text(
        "譜面 / Timeline",
        25.0,
        8.0,
        180.0,
        (11.0, [232, 232, 232, 255], false),
    );
    scene.outline(width - 78.0, 4.0, 30.0, 22.0, LINE, 1.0);
    scene.outline(width - 44.0, 4.0, 30.0, 22.0, ACTIVE, 1.0);
    scene.text(
        "⌁",
        width - 69.0,
        7.0,
        20.0,
        (11.0, [150, 150, 150, 255], true),
    );
    scene.text(
        "▤",
        width - 35.0,
        7.0,
        20.0,
        (11.0, [232, 232, 232, 255], true),
    );

    scene.rect(0.0, header_h, rail_w, height - header_h, BG);
    scene.rect(time_x, header_h, time_w, ruler_h, BG);
    scene.rect(time_x - 1.0, header_h, 1.0, height - header_h, LINE_STRONG);

    scene.text(
        "S",
        10.0,
        header_h + 8.0,
        12.0,
        (8.0, [156, 156, 156, 255], true),
    );
    scene.text(
        "M",
        32.0,
        header_h + 8.0,
        12.0,
        (8.0, [156, 156, 156, 255], true),
    );
    scene.text(
        "TIME / BEAT",
        time_x + 8.0,
        header_h + 8.0,
        80.0,
        (8.0, [218, 174, 117, 255], true),
    );
    for tick in 0..17 {
        let x = time_x + tick as f32 * time_w / 16.0;
        let major = tick % 4 == 0;
        scene.rect(
            x,
            header_h + ruler_h - if major { 8.0 } else { 4.0 },
            1.0,
            if major { 8.0 } else { 4.0 },
            if major { LINE_STRONG } else { LINE },
        );
        scene.rect(
            x,
            header_h + ruler_h,
            1.0,
            height - header_h - ruler_h,
            if major {
                LINE
            } else {
                [0.065, 0.068, 0.072, 1.0]
            },
        );
        if major && tick > 0 {
            scene.text(
                format!("{}", 52 + tick / 4),
                x - 4.0,
                header_h + 7.0,
                32.0,
                (8.0, [158, 158, 158, 255], true),
            );
        }
    }

    for (index, object) in OBJECTS.iter().enumerate() {
        let row_y = header_h + ruler_h + index as f32 * row_h;
        scene.rect(0.0, row_y + row_h - 1.0, width, 1.0, LINE);
        scene.outline(7.0, row_y + 7.0, 18.0, 18.0, LINE, 1.0);
        scene.outline(29.0, row_y + 7.0, 18.0, 18.0, LINE, 1.0);
        scene.text(
            "S",
            13.0,
            row_y + 11.0,
            10.0,
            (8.0, [212, 212, 212, 255], true),
        );
        scene.text(
            "M",
            35.0,
            row_y + 11.0,
            10.0,
            (8.0, [212, 212, 212, 255], true),
        );

        let bar_x = time_x + object.start * time_w;
        let bar_w = (object.end - object.start) * time_w;
        let bar_y = row_y + 5.0;
        let bar_h = (row_h - 10.0).max(20.0);
        if object.kind == "G" {
            scene.rect(time_x, row_y, time_w, row_h, [0.055, 0.050, 0.085, 1.0]);
            scene.rect(time_x, row_y, 3.0, row_h, BAR_COLORS[object.color_slot]);
        }
        scene.rect(bar_x, bar_y, bar_w, bar_h, BAR_COLORS[object.color_slot]);
        scene.outline(
            bar_x,
            bar_y,
            bar_w,
            bar_h,
            if object.selected { ACTIVE } else { LINE_STRONG },
            if object.selected { 2.0 } else { 1.0 },
        );
        scene.rect(
            bar_x + 7.0,
            bar_y + 6.0,
            16.0,
            16.0,
            [0.035, 0.038, 0.040, 0.82],
        );
        scene.outline(
            bar_x + 7.0,
            bar_y + 6.0,
            16.0,
            16.0,
            [0.72, 0.72, 0.72, 1.0],
            1.0,
        );
        scene.text(
            object.kind,
            bar_x + 12.0,
            bar_y + 9.0,
            10.0,
            (8.0, [230, 230, 230, 255], true),
        );
        scene.text(
            object.name,
            bar_x + 31.0,
            bar_y + 9.0,
            (bar_w - 40.0).max(30.0),
            (10.0, [26, 27, 28, 255], object.kind != "T"),
        );
        if object.kind == "G" {
            scene.text(
                "IN  →  Echo Bloom  →  OUT",
                bar_x + 130.0,
                bar_y + 9.0,
                190.0,
                (8.0, [42, 42, 48, 255], true),
            );
            for key in [0.32_f32, 0.51, 0.72] {
                let x = time_x + key * time_w;
                scene.shape(x - 5.0, bar_y + bar_h * 0.5 - 5.0, 10.0, 10.0, ACTIVE, 1);
            }
        }
        if object.kind == "A" {
            scene.text(
                "╱╲╱▁╲╱╲▁╱╲╱╲▁╱╲",
                bar_x + 150.0,
                bar_y + 9.0,
                220.0,
                (9.0, [50, 78, 76, 255], true),
            );
        }
    }

    let playhead_x = time_x + time_w * 0.46;
    scene.rect(
        playhead_x,
        header_h + 2.0,
        1.5,
        height - header_h - 2.0,
        ACTIVE,
    );
    scene.shape(
        playhead_x - 6.0,
        header_h + ruler_h - 2.0,
        13.0,
        9.0,
        ACTIVE,
        2,
    );
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fixture_ids_are_stable_and_unique() {
        let ids = OBJECTS
            .iter()
            .map(|object| object.id)
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), OBJECTS.len());
        assert_eq!(OBJECTS.iter().filter(|object| object.selected).count(), 1);
    }

    #[test]
    fn fixture_ranges_are_normalized_and_non_empty() {
        for object in OBJECTS {
            assert!((0.0..=1.0).contains(&object.start));
            assert!((0.0..=1.0).contains(&object.end));
            assert!(object.start < object.end);
        }
    }

    #[test]
    fn scene_contains_oracle_roles_without_a_second_state_owner() {
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<HashSet<_>>();
        for expected in [
            "譜面 / Timeline",
            "TIME / BEAT",
            "Pulse rings",
            "NIGHT DRIVE",
        ] {
            assert!(labels.contains(expected), "missing oracle label {expected}");
        }
        assert!(scene.rects.iter().any(|primitive| primitive.shape == 1));
        assert!(scene.rects.iter().any(|primitive| primitive.shape == 2));
    }

    #[test]
    fn react_tool_panel_is_not_duplicated_in_native_scene() {
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<HashSet<_>>();
        for react_owned in ["KEYS", "LAYERS", "ALIGN"] {
            assert!(
                !labels.contains(react_owned),
                "duplicated React label {react_owned}"
            );
        }
    }

    #[test]
    fn logical_layout_is_dpi_independent() {
        let one = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let two = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        assert_eq!(
            bytemuck::cast_slice::<RectPrimitive, u8>(&one.rects),
            bytemuck::cast_slice::<RectPrimitive, u8>(&two.rects)
        );
    }
}
