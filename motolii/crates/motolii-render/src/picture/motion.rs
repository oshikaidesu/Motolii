//! Where a layer travels, as the picture places it. Each point is `world_affine`
//! of the pivot at that frame. A line through the position keys is a different curve.

use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::store::{property, LayerId, PropertyId, StoreError, StoreView};

const MAX_SAMPLES: i64 = 240;

/// Comp-space points of the pivot across the keyed span. Empty when Position
/// does not move: a still value, or travel under half a pixel.
pub fn samples(view: &StoreView<'_>, layer: LayerId) -> Result<Vec<[f64; 2]>, StoreError> {
    let Some(comp) = view.composition()? else { return Ok(Vec::new()) };
    let Some((start, end)) = span(view, layer, comp.fps)? else { return Ok(Vec::new()) };
    let step = ((end - start) / MAX_SAMPLES).max(1);
    let present: HashSet<LayerId> = view.layers().into_iter().collect();
    let mut points = Vec::new();
    let mut frame = start;
    loop {
        points.push(at(view, layer, frame, comp.fps, &present)?);
        if frame >= end { break; }
        let next = (frame + step).min(end);
        if next == frame { break; }
        frame = next;
    }
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for p in &points {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    if (max_x - min_x).max(max_y - min_y) < 0.5 { return Ok(Vec::new()); }
    Ok(points)
}

fn span(view: &StoreView<'_>, layer: LayerId, fps: crate::doc::core::Fps) -> Result<Option<(i64, i64)>, StoreError> {
    let mut frames = Vec::new();
    for name in [property::POSITION, property::POSITION_X, property::POSITION_Y] {
        frames.extend(key_frames(view, layer, name, fps)?);
    }
    if frames.len() < 2 {
        let Some(meta) = view.meta(layer)? else { return Ok(None) };
        let start = meta.timing.start;
        let end = start + meta.timing.duration;
        if end <= start { return Ok(None) }
        let t = RationalTime::try_from_frame(start, fps).map_err(|err| StoreError::Property(err.to_string()))?;
        if crate::picture::path::on_offset_path(view, layer, t)?.is_none() { return Ok(None) }
        return Ok(Some((start, end)));
    }
    frames.sort_unstable();
    Ok(Some((frames[0], *frames.last().unwrap())))
}

fn key_frames(view: &StoreView<'_>, layer: LayerId, name: &str, fps: crate::doc::core::Fps) -> Result<Vec<i64>, StoreError> {
    let Some(track) = view.track(layer, &PropertyId::new(name)?)? else { return Ok(Vec::new()) };
    track.keys().iter().map(|key| key.t.try_to_frame_round(fps).map_err(|err| StoreError::Property(err.to_string()))).collect()
}

fn at(view: &StoreView<'_>, layer: LayerId, frame: i64, fps: crate::doc::core::Fps, present: &HashSet<LayerId>) -> Result<[f64; 2], StoreError> {
    let t = RationalTime::try_from_frame(frame, fps).map_err(|err| StoreError::Property(err.to_string()))?;
    let world = crate::picture::resolve::transform::world_affine(view, layer, t, present, &mut HashMap::new(), &mut HashSet::new())?;
    let pivot = crate::picture::resolve::transform::layer_pivot(view, layer, t)?;
    let point = world.transform_point2(glam::Vec2::new(pivot[0], pivot[1]));
    Ok([point.x as f64, point.y as f64])
}

#[cfg(test)]
mod tests {
    use motolii_edit::{Document, Intent};

    use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Value};
    use crate::doc::store::{property, Composition, Fps, LayerAttrsPatch, LayerId, PropertyId, RationalTime};

    use super::samples;

    fn fps() -> Fps { Fps::try_new(30, 1).unwrap() }

    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps: fps(),
            duration_frames: 300,
            background: Composition::default_background(),
        })).unwrap();
        doc
    }

    fn track(keys: &[(i64, [f64; 2])]) -> KeyframeTrack {
        KeyframeTrack::try_from_keys(keys.iter().map(|(frame, value)| Keyframe {
            t: RationalTime::try_from_frame(*frame, fps()).unwrap(),
            value: Value::Vec2(*value),
            interp: Interp::Linear,
            spatial: None,
        }).collect()).unwrap()
    }

    #[test]
    fn a_still_position_has_no_path() {
        let mut doc = document();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([10.0, 20.0]) }).unwrap();
        assert!(samples(&doc.view(), layer).unwrap().is_empty());
    }

    #[test]
    fn samples_follow_the_eased_position_including_a_static_parent() {
        let mut doc = document();
        let parent = LayerId(1);
        let child = LayerId(2);
        doc.apply_all([
            Intent::AddLayer(parent),
            Intent::AddLayer(child),
            Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() } },
            Intent::SetConstant { layer: parent, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([100.0, 0.0]) },
            Intent::SetTrack { layer: child, property: PropertyId::new(property::POSITION).unwrap(), track: track(&[(0, [0.0, 0.0]), (30, [300.0, 0.0])]) },
        ]).unwrap();
        let points = samples(&doc.view(), child).unwrap();
        assert!(points.len() > 2, "{}", points.len());
        assert!((points[0][0] - 100.0).abs() < 0.01, "{:?}", points[0]);
        assert!((points.last().unwrap()[0] - 400.0).abs() < 0.01, "{:?}", points.last());
        let mid = points[points.len() / 2][0];
        assert!(mid > 150.0 && mid < 350.0, "{mid}");
    }
}
