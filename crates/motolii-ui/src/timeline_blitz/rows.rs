//! 移植元 `timeline_egui/rows.rs` の**写し**。行の起こし方を変えないため、
//! 構造体のfield名・行の並び・palette_slotの決め方を1対1で保つ。
//!
//! Documentは読むだけで、ここでも描画側でも書き換えない。
//!
//! **移植元は 2026-08-16 に撤去済み**(旧eguiアプリの畳み込み)。file:line の引用は
//! 撤去前の原文に対するもので、原文は `git show f209da9d^:<path>` で読める。
//! 新しい値をここで決めないという契約は生きている。

use motolii_doc::{Document, KeyframeId, LayerId};

use super::theme::PALETTE;
use crate::timeline_projection::TimelineProjection;

#[derive(Debug, Clone)]
pub(super) struct TimelineRow {
    pub(super) layer: LayerId,
    pub(super) label: String,
    pub(super) property: Option<&'static str>,
    pub(super) start: Option<f64>,
    pub(super) end: Option<f64>,
    pub(super) keys: Vec<TimelineRowKey>,
    pub(super) palette_slot: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TimelineRowKey {
    pub(super) id: KeyframeId,
    pub(super) fraction: f64,
}

/// timeline_egui/rows.rs:19-97 の写し。f32ではなくf64で持つのはCSSへ出すため。
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
            let start = bars.iter().map(|bar| bar.x_start).reduce(f64::min);
            let end = bars.iter().map(|bar| bar.x_end).reduce(f64::max);
            let keys = projection
                .keys()
                .iter()
                .filter(|key| key.layer == layer)
                .map(|key| TimelineRowKey {
                    id: key.key,
                    fraction: key.center_x,
                })
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
