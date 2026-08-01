//! Documentのtop-level Clipをheadless Timeline座標へ読み取り専用投影する。
//! band packingはviewportより前に全clipへ適用する契約のため、cull後もbandは広いviewportと一致させる。
//! 入力検証と座標の有限性もここで閉じ、部分layoutや非有限座標を成功返却しない。

use motolii_core::RationalTime;
use motolii_doc::{DocParam, Document, KeyframeId, LayerId, TrackItem};
use motolii_eval::Interp;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineMetrics {
    pub band_height: f64,
    pub units_per_second: f64,
    pub key_half_extent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineViewport {
    pub start: RationalTime,
    pub end: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineBar {
    pub layer: LayerId,
    pub start: RationalTime,
    pub end: RationalTime,
    pub band: u32,
    pub x_start: f64,
    pub x_end: f64,
    pub y_top: f64,
    pub y_bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineKey {
    pub layer: LayerId,
    pub key: KeyframeId,
    pub t: RationalTime,
    pub band: u32,
    pub center_x: f64,
    pub center_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelinePositionInterval {
    pub layer: LayerId,
    pub left_key: KeyframeId,
    pub right_key: KeyframeId,
    pub start: RationalTime,
    pub end: RationalTime,
    pub interp: Interp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineUnsupported {
    GroupItem { layer: LayerId },
    DataParam { layer: LayerId },
    Vec2AxesParam { layer: LayerId },
    LookAtParam { layer: LayerId },
    FollowParam { layer: LayerId },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineProjection {
    bars: Vec<TimelineBar>,
    keys: Vec<TimelineKey>,
    unsupported: Vec<TimelineUnsupported>,
    key_half_extent: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineHit {
    Key { layer: LayerId, key: KeyframeId },
    Bar { layer: LayerId },
    None,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TimelineProjectionError {
    #[error("timeline metric is not finite")]
    NonFiniteMetric,
    #[error("timeline metric is not positive")]
    NonPositiveMetric,
    #[error("viewport end must be after viewport start")]
    InvalidViewport,
    #[error("clip duration must be positive (layer {layer:?})")]
    InvalidDuration { layer: LayerId },
    #[error("rational time overflow (layer {layer:?})")]
    TimeOverflow { layer: LayerId },
}

struct ClipRecord {
    layer: LayerId,
    start: RationalTime,
    end: RationalTime,
    keys: Vec<(KeyframeId, RationalTime)>,
}

pub fn project_timeline(
    document: &Document,
    metrics: &TimelineMetrics,
    viewport: &TimelineViewport,
) -> Result<TimelineProjection, TimelineProjectionError> {
    validate_metrics(metrics)?;
    validate_viewport(viewport)?;

    let mut records = Vec::new();
    let mut unsupported = Vec::new();

    for track in &document.tracks {
        for item in &track.items {
            match item {
                TrackItem::Group(group) => {
                    unsupported.push(TimelineUnsupported::GroupItem {
                        layer: group.envelope.layer_id,
                    });
                }
                TrackItem::Clip(clip) => {
                    let layer = clip.envelope.layer_id;
                    if clip.duration <= RationalTime::ZERO {
                        return Err(TimelineProjectionError::InvalidDuration { layer });
                    }
                    let end = clip
                        .start
                        .try_add(clip.duration)
                        .map_err(|_| TimelineProjectionError::TimeOverflow { layer })?;
                    scan_position(&clip.envelope.transform.position, layer, &mut unsupported);
                    let keys = collect_keys(&clip.envelope.transform.position);
                    records.push(ClipRecord {
                        layer,
                        start: clip.start,
                        end,
                        keys,
                    });
                }
            }
        }
    }

    records.sort_by_key(|r| (r.start, r.end, r.layer));

    let bands = assign_bands(&records);

    let mut bars = Vec::new();
    let mut keys = Vec::new();

    for (record, &band) in records.iter().zip(bands.iter()) {
        if bar_visible(record.start, record.end, viewport) {
            let bar = make_bar(record, band, metrics, viewport)?;
            bars.push(bar);
        }
        for &(key_id, t) in &record.keys {
            if key_visible(t, viewport) {
                let key = make_key(record.layer, key_id, t, band, metrics, viewport)?;
                keys.push(key);
            }
        }
    }

    bars.sort_by_key(|a| (a.band, a.start, a.end, a.layer));
    keys.sort_by_key(|a| (a.layer, a.key));
    unsupported.sort_by_key(|a| (unsupported_layer(a), unsupported_order(a)));

    Ok(TimelineProjection {
        bars,
        keys,
        unsupported,
        key_half_extent: metrics.key_half_extent,
    })
}

/// U4b-1: native Timelineで選ばれた左Position keyから直後区間を導出する。
pub fn project_position_interval(
    document: &Document,
    layer: LayerId,
    left_key: KeyframeId,
) -> Option<TimelinePositionInterval> {
    for track in &document.tracks {
        for item in &track.items {
            let TrackItem::Clip(clip) = item else {
                continue;
            };
            if clip.envelope.layer_id != layer {
                continue;
            }
            let DocParam::Keyframes(keys) = &clip.envelope.transform.position else {
                return None;
            };
            let left_index = keys.keys().iter().position(|key| key.id == left_key)?;
            let left = &keys.keys()[left_index];
            let right = keys.keys().get(left_index + 1)?;
            return Some(TimelinePositionInterval {
                layer,
                left_key,
                right_key: right.id,
                start: left.t,
                end: right.t,
                interp: left.interp,
            });
        }
    }
    None
}

impl TimelineProjection {
    pub fn bars(&self) -> &[TimelineBar] {
        &self.bars
    }

    pub fn keys(&self) -> &[TimelineKey] {
        &self.keys
    }

    pub fn unsupported(&self) -> &[TimelineUnsupported] {
        &self.unsupported
    }

    pub fn hit_test(&self, x: f64, y: f64) -> TimelineHit {
        let mut best_key: Option<(LayerId, KeyframeId)> = None;
        for key in &self.keys {
            if !manhattan_hit(x, y, key.center_x, key.center_y, self.key_half_extent) {
                continue;
            }
            let candidate = (key.layer, key.key);
            if best_key.is_none_or(|best| candidate < best) {
                best_key = Some(candidate);
            }
        }
        if let Some((layer, key)) = best_key {
            return TimelineHit::Key { layer, key };
        }

        let mut best_bar: Option<LayerId> = None;
        for bar in &self.bars {
            if bar_hit(x, y, bar) && best_bar.is_none_or(|best| bar.layer < best) {
                best_bar = Some(bar.layer);
            }
        }
        if let Some(layer) = best_bar {
            return TimelineHit::Bar { layer };
        }

        TimelineHit::None
    }
}

fn validate_metrics(metrics: &TimelineMetrics) -> Result<(), TimelineProjectionError> {
    let fields = [
        metrics.band_height,
        metrics.units_per_second,
        metrics.key_half_extent,
    ];
    if fields.iter().any(|v| !v.is_finite()) {
        return Err(TimelineProjectionError::NonFiniteMetric);
    }
    if fields.iter().any(|&v| v <= 0.0) {
        return Err(TimelineProjectionError::NonPositiveMetric);
    }
    Ok(())
}

fn validate_viewport(viewport: &TimelineViewport) -> Result<(), TimelineProjectionError> {
    if viewport.end <= viewport.start {
        return Err(TimelineProjectionError::InvalidViewport);
    }
    Ok(())
}

fn scan_position(position: &DocParam, layer: LayerId, unsupported: &mut Vec<TimelineUnsupported>) {
    match position {
        DocParam::Const(_) | DocParam::Keyframes(_) => {}
        DocParam::Data { .. } => unsupported.push(TimelineUnsupported::DataParam { layer }),
        DocParam::Vec2Axes { .. } => unsupported.push(TimelineUnsupported::Vec2AxesParam { layer }),
        DocParam::LookAt { .. } => unsupported.push(TimelineUnsupported::LookAtParam { layer }),
        DocParam::Follow { .. } => unsupported.push(TimelineUnsupported::FollowParam { layer }),
    }
}

fn collect_keys(position: &DocParam) -> Vec<(KeyframeId, RationalTime)> {
    match position {
        DocParam::Keyframes(track) => track.keys().iter().map(|k| (k.id, k.t)).collect(),
        _ => Vec::new(),
    }
}

fn assign_bands(records: &[ClipRecord]) -> Vec<u32> {
    let mut band_ends: Vec<RationalTime> = Vec::new();
    let mut out = Vec::with_capacity(records.len());
    for record in records {
        let mut placed = false;
        for (idx, end) in band_ends.iter_mut().enumerate() {
            if *end <= record.start {
                *end = record.end;
                out.push(idx as u32);
                placed = true;
                break;
            }
        }
        if !placed {
            band_ends.push(record.end);
            out.push((band_ends.len() - 1) as u32);
        }
    }
    out
}

fn bar_visible(start: RationalTime, end: RationalTime, viewport: &TimelineViewport) -> bool {
    start < viewport.end && viewport.start < end
}

fn key_visible(t: RationalTime, viewport: &TimelineViewport) -> bool {
    viewport.start <= t && t < viewport.end
}

fn time_to_x(
    t: RationalTime,
    layer: LayerId,
    viewport: &TimelineViewport,
    units_per_second: f64,
) -> Result<f64, TimelineProjectionError> {
    let dt = t
        .try_sub(viewport.start)
        .map_err(|_| TimelineProjectionError::TimeOverflow { layer })?;
    let x = dt.as_seconds_f64() * units_per_second;
    if !x.is_finite() {
        return Err(TimelineProjectionError::TimeOverflow { layer });
    }
    Ok(x)
}

fn finite_derived_coords(layer: LayerId, coords: &[f64]) -> Result<(), TimelineProjectionError> {
    if coords.iter().any(|v| !v.is_finite()) {
        return Err(TimelineProjectionError::TimeOverflow { layer });
    }
    Ok(())
}

fn make_bar(
    record: &ClipRecord,
    band: u32,
    metrics: &TimelineMetrics,
    viewport: &TimelineViewport,
) -> Result<TimelineBar, TimelineProjectionError> {
    let x_start = time_to_x(
        record.start,
        record.layer,
        viewport,
        metrics.units_per_second,
    )?;
    let x_end = time_to_x(record.end, record.layer, viewport, metrics.units_per_second)?;
    let y_top = band as f64 * metrics.band_height;
    let y_bottom = y_top + metrics.band_height;
    finite_derived_coords(record.layer, &[x_start, x_end, y_top, y_bottom])?;
    Ok(TimelineBar {
        layer: record.layer,
        start: record.start,
        end: record.end,
        band,
        x_start,
        x_end,
        y_top,
        y_bottom,
    })
}

fn make_key(
    layer: LayerId,
    key: KeyframeId,
    t: RationalTime,
    band: u32,
    metrics: &TimelineMetrics,
    viewport: &TimelineViewport,
) -> Result<TimelineKey, TimelineProjectionError> {
    let center_x = time_to_x(t, layer, viewport, metrics.units_per_second)?;
    let y_top = band as f64 * metrics.band_height;
    let center_y = y_top + metrics.band_height / 2.0;
    finite_derived_coords(layer, &[center_x, center_y])?;
    Ok(TimelineKey {
        layer,
        key,
        t,
        band,
        center_x,
        center_y,
    })
}

fn manhattan_hit(x: f64, y: f64, cx: f64, cy: f64, half: f64) -> bool {
    (x - cx).abs() + (y - cy).abs() <= half
}

fn bar_hit(x: f64, y: f64, bar: &TimelineBar) -> bool {
    x >= bar.x_start && x < bar.x_end && y >= bar.y_top && y < bar.y_bottom
}

fn unsupported_layer(u: &TimelineUnsupported) -> LayerId {
    match u {
        TimelineUnsupported::GroupItem { layer }
        | TimelineUnsupported::DataParam { layer }
        | TimelineUnsupported::Vec2AxesParam { layer }
        | TimelineUnsupported::LookAtParam { layer }
        | TimelineUnsupported::FollowParam { layer } => *layer,
    }
}

fn unsupported_order(u: &TimelineUnsupported) -> u8 {
    match u {
        TimelineUnsupported::GroupItem { .. } => 0,
        TimelineUnsupported::DataParam { .. } => 1,
        TimelineUnsupported::Vec2AxesParam { .. } => 2,
        TimelineUnsupported::LookAtParam { .. } => 3,
        TimelineUnsupported::FollowParam { .. } => 4,
    }
}
