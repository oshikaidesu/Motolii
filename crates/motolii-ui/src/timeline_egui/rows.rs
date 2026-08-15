//! Document投影からTimelineの行モデルを起こす。描画も入力もここでは行わない。

use motolii_doc::{Document, LayerId};

use super::theme::PALETTE;
use crate::timeline_projection::TimelineProjection;

#[derive(Debug, Clone)]
pub(super) struct TimelineRow {
    pub(super) layer: LayerId,
    pub(super) label: String,
    pub(super) property: Option<&'static str>,
    pub(super) start: Option<f32>,
    pub(super) end: Option<f32>,
    pub(super) keys: Vec<f32>,
    pub(super) palette_slot: usize,
}

pub(super) fn rows_from_projection(
    document: &Document,
    projection: Option<&TimelineProjection>,
) -> Vec<TimelineRow> {
    let Some(projection) = projection else {
        return Vec::new();
    };
    let mut layers = Vec::<LayerId>::new();
    for bar in projection.bars() {
        if !layers.contains(&bar.layer) {
            layers.push(bar.layer);
        }
    }
    for key in projection.keys() {
        if !layers.contains(&key.layer) {
            layers.push(key.layer);
        }
    }
    layers.sort();

    layers
        .into_iter()
        .flat_map(|layer| {
            let bars = projection
                .bars()
                .iter()
                .filter(|bar| bar.layer == layer)
                .collect::<Vec<_>>();
            let start = bars.iter().map(|bar| bar.x_start as f32).reduce(f32::min);
            let end = bars.iter().map(|bar| bar.x_end as f32).reduce(f32::max);
            let keys = projection
                .keys()
                .iter()
                .filter(|key| key.layer == layer)
                .map(|key| key.center_x as f32)
                .collect::<Vec<_>>();
            let label = document
                .layers
                .display_name(layer)
                .unwrap_or("layer")
                .to_owned();
            vec![
                TimelineRow {
                    layer,
                    label: label.clone(),
                    property: None,
                    start,
                    end,
                    keys: Vec::new(),
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
                TimelineRow {
                    layer,
                    label: "Position".to_owned(),
                    property: Some("Position"),
                    start: None,
                    end: None,
                    keys,
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
                TimelineRow {
                    layer,
                    label: "Parameters".to_owned(),
                    property: Some("Parameters"),
                    start: None,
                    end: None,
                    keys: Vec::new(),
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
            ]
        })
        .collect()
}
