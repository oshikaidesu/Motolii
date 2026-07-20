use crate::theme;
use eframe::egui::Color32;

#[derive(Debug, Clone, Copy)]
pub struct TimelineBar {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub start: f32,
    pub end: f32,
    pub row: usize,
    pub color: Color32,
}

pub const BARS: [TimelineBar; 6] = [
    TimelineBar {
        id: "audio",
        label: "night_drive.wav",
        kind: "A",
        start: 0.013,
        end: 0.98,
        row: 0,
        color: theme::DATA,
    },
    TimelineBar {
        id: "pulse",
        label: "Pulse rings",
        kind: "G",
        start: 0.073,
        end: 0.98,
        row: 1,
        color: theme::SHAPE,
    },
    TimelineBar {
        id: "city",
        label: "City grid",
        kind: "S",
        start: 0.117,
        end: 0.98,
        row: 2,
        color: Color32::from_rgb(157, 185, 163),
    },
    TimelineBar {
        id: "title",
        label: "NIGHT DRIVE",
        kind: "T",
        start: 0.394,
        end: 0.943,
        row: 3,
        color: Color32::from_rgb(190, 164, 123),
    },
    TimelineBar {
        id: "city-loop",
        label: "city_loop.mp4",
        kind: "V",
        start: 0.233,
        end: 0.681,
        row: 4,
        color: Color32::from_rgb(190, 143, 130),
    },
    TimelineBar {
        id: "traffic",
        label: "traffic_pass.mp4",
        kind: "V",
        start: 0.741,
        end: 1.0,
        row: 4,
        color: Color32::from_rgb(126, 163, 194),
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
