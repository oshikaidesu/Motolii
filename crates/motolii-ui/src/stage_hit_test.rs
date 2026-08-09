//! Stage pointer の純関数 hit-test（R2-STAGE-SELECTION-PRODUCER）。
//! consumer-local な selection owner は作らず、幾何投影列への読み取り判定だけを返す。

use motolii_core::CanonicalPoint;
use motolii_doc::LayerId;

use crate::stage_geometry_projection::{
    StageGeometryProjection, StageLayerProjection, StageLocalRect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageHitTestReject {
    /// width または height が 0。変換の分母にせず typed 拒否する。
    ZeroStageExtent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageHit {
    Layer(LayerId),
    Miss,
}

/// view-local（原点左下・Y-up・logical points）→ 正準（原点中央・Y-up・高さ1.0）。
/// Y 反転は入れない。分母は常に height（aspect は x の範囲に出る）。
pub(crate) fn view_local_to_canonical(
    view_local_x: f64,
    view_local_y: f64,
    width: u32,
    height: u32,
) -> Result<CanonicalPoint, StageHitTestReject> {
    if width == 0 || height == 0 {
        return Err(StageHitTestReject::ZeroStageExtent);
    }
    let w = f64::from(width);
    let h = f64::from(height);
    Ok(CanonicalPoint {
        x: (view_local_x - w * 0.5) / h,
        y: (view_local_y - h * 0.5) / h,
    })
}

/// stage 論理矩形の閉区間内か。外側は hit なし（clear）へ送る。
pub(crate) fn view_local_in_stage(
    view_local_x: f64,
    view_local_y: f64,
    width: u32,
    height: u32,
) -> bool {
    let w = f64::from(width);
    let h = f64::from(height);
    view_local_x >= 0.0 && view_local_x <= w && view_local_y >= 0.0 && view_local_y <= h
}

fn point_in_closed_local_rect(local: CanonicalPoint, rect: StageLocalRect) -> bool {
    let size = rect.size;
    // 退化 rect は hit 対象外。
    if size.width <= 0.0 || size.height <= 0.0 {
        return false;
    }
    let half_w = size.width * 0.5;
    let half_h = size.height * 0.5;
    let cx = rect.center.x;
    let cy = rect.center.y;
    local.x >= cx - half_w
        && local.x <= cx + half_w
        && local.y >= cy - half_h
        && local.y <= cy + half_h
}

/// 正準点を投影列へ当てる。
///
/// `build_group` は子を順に composite し後の子が手前へ重なるため、
/// 投影列の後方ほど手前である。最後に hit した層を勝者とする。
pub(crate) fn hit_test_projected_layers(
    canonical: CanonicalPoint,
    projection: &StageGeometryProjection,
) -> StageHit {
    let mut winner = None;
    for (layer_id, layer_proj) in projection.layers() {
        let StageLayerProjection::Available(geo) = layer_proj else {
            // Unavailable は hit 対象外。
            continue;
        };
        if geo.local_rect.size.width <= 0.0 || geo.local_rect.size.height <= 0.0 {
            continue;
        }
        let Some(inverse) = (geo.camera_view * geo.world).try_invert() else {
            // project_stage_geometry は特異を全体 Err にするが、防御的に除外する。
            continue;
        };
        let [lx, ly] = inverse.transform_point(canonical.x, canonical.y);
        if point_in_closed_local_rect(CanonicalPoint { x: lx, y: ly }, geo.local_rect) {
            winner = Some(*layer_id);
        }
    }
    match winner {
        Some(layer) => StageHit::Layer(layer),
        None => StageHit::Miss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage_geometry_projection::project_stage_geometry;
    use motolii_core::{CanonicalPoint, CanonicalSize, RationalTime};
    use motolii_doc::{
        Clip, ClipSource, DocParam, Document, EvaluationTime, ItemEnvelope, LayerId, Track,
        TrackItem, Transform2D, RECT_LAYER_SOURCE,
    };
    use motolii_eval::DataTracks;
    use std::collections::BTreeMap;

    fn sec(n: i64) -> RationalTime {
        RationalTime::try_new(n, 1).unwrap()
    }

    fn rect_params(center: [f64; 2], size: [f64; 2]) -> BTreeMap<String, DocParam> {
        BTreeMap::from([
            ("center".into(), DocParam::const_vec2(center)),
            ("size".into(), DocParam::const_vec2(size)),
            ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
        ])
    }

    fn push_rect(
        doc: &mut Document,
        name: &str,
        center: [f64; 2],
        size: [f64; 2],
        transform: Transform2D,
    ) -> LayerId {
        let layer = doc.layers.allocate(name).unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform = transform;
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params(center, size),
                extra: Default::default(),
            },
        }));
        layer
    }

    fn base_doc() -> Document {
        let mut doc = Document::new_current();
        let track = doc.track_ids.allocate("V1").unwrap();
        doc.tracks.push(Track {
            id: track,
            items: vec![],
        });
        doc
    }

    #[test]
    fn view_local_to_canonical_uses_height_denominator_without_y_flip() {
        let p = view_local_to_canonical(800.0, 450.0, 1600, 900).unwrap();
        assert_eq!(p, CanonicalPoint { x: 0.0, y: 0.0 });
        let upper = view_local_to_canonical(800.0, 900.0, 1600, 900).unwrap();
        assert!((upper.y - 0.5).abs() < 1e-12);
        let right = view_local_to_canonical(1250.0, 450.0, 1600, 900).unwrap();
        assert!((right.x - 0.5).abs() < 1e-12);
    }

    #[test]
    fn zero_extent_rejects_without_division() {
        assert_eq!(
            view_local_to_canonical(1.0, 1.0, 0, 900),
            Err(StageHitTestReject::ZeroStageExtent)
        );
        assert_eq!(
            view_local_to_canonical(1.0, 1.0, 1600, 0),
            Err(StageHitTestReject::ZeroStageExtent)
        );
    }

    #[test]
    fn later_projection_layer_wins_overlap() {
        let mut doc = base_doc();
        let back = push_rect(
            &mut doc,
            "back",
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        let front = push_rect(
            &mut doc,
            "front",
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        assert_eq!(
            proj.layers().iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![back, front]
        );
        let hit = hit_test_projected_layers(CanonicalPoint::CENTER, &proj);
        assert_eq!(hit, StageHit::Layer(front));
    }

    #[test]
    fn degenerate_rect_is_not_contained() {
        assert!(!point_in_closed_local_rect(
            CanonicalPoint::CENTER,
            StageLocalRect {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 0.0,
                    height: 1.0,
                },
            }
        ));
        assert!(!point_in_closed_local_rect(
            CanonicalPoint::CENTER,
            StageLocalRect {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 1.0,
                    height: -0.1,
                },
            }
        ));
    }
}
