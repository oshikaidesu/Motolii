use crate::theme;

#[derive(Debug, Clone, Copy)]
pub struct TimelineBar {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub start: f32,
    pub end: f32,
    pub row: usize,
    pub color: eframe::egui::Color32,
}

pub const BARS: [TimelineBar; 6] = [
    TimelineBar {
        id: "audio",
        label: "night_drive.wav",
        kind: "A",
        start: 0.0142,
        end: 1.0,
        row: 0,
        color: theme::OBJECT_AUDIO,
    },
    TimelineBar {
        id: "pulse",
        label: "Pulse rings",
        kind: "G",
        start: 0.0567,
        end: 1.0,
        row: 1,
        color: theme::OBJECT_GROUP,
    },
    TimelineBar {
        id: "city",
        label: "City grid",
        kind: "S",
        start: 0.1133,
        end: 1.0,
        row: 2,
        color: theme::OBJECT_CHILD,
    },
    TimelineBar {
        id: "title",
        label: "NIGHT DRIVE",
        kind: "T",
        start: 0.3825,
        end: 0.9207,
        row: 3,
        color: theme::OBJECT_TITLE,
    },
    TimelineBar {
        id: "city-loop",
        label: "city_loop.mp4",
        kind: "V",
        start: 0.2266,
        end: 0.6656,
        row: 4,
        color: theme::OBJECT_VIDEO_A,
    },
    TimelineBar {
        id: "traffic",
        label: "traffic_pass.mp4",
        kind: "V",
        start: 0.7224,
        end: 1.0,
        row: 4,
        color: theme::OBJECT_VIDEO_B,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_fixture_has_valid_normalized_ranges() {
        assert!(BARS.iter().all(|bar| {
            (0.0..=1.0).contains(&bar.start)
                && (0.0..=1.0).contains(&bar.end)
                && bar.start < bar.end
        }));
    }
}
