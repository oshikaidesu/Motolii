//! 可視 Layer の Stage 幾何を Document から読み取り専用投影する（R2-STAGE-GEOMETRY-READ）。
//! AABB へ潰さず、局所 rect + world / camera view Affine2D を並べる。第二 writer を作らない。

use motolii_core::{CanonicalPoint, CanonicalSize, CompCamera, CompCameraError, RationalTime};
use motolii_doc::{
    param_eval, resolve_document_spaces, resolve_transform, visible_layers_at, Affine2D, Clip,
    ClipSource, CompCameraDoc, Document, EvaluationTime, LayerId, ParamEvalError,
    ResolvedLayerParams, TrackItem, Transform2D, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageLocalRect {
    pub center: CanonicalPoint,
    pub size: CanonicalSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageLayerGeometry {
    pub local_rect: StageLocalRect,
    pub world: Affine2D,
    pub camera_view: Affine2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageGeometryUnavailable {
    Group { layer: LayerId },
    VideoSource { layer: LayerId },
    VectorSource { layer: LayerId },
    PluginSource { layer: LayerId },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageLayerProjection {
    Available(StageLayerGeometry),
    Unavailable(StageGeometryUnavailable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageGeometryProjection {
    layers: Vec<(LayerId, StageLayerProjection)>,
    camera_view: Affine2D,
}

impl StageGeometryProjection {
    pub fn layers(&self) -> &[(LayerId, StageLayerProjection)] {
        &self.layers
    }

    pub fn camera_view(&self) -> Affine2D {
        self.camera_view
    }

    pub fn get(&self, layer: LayerId) -> Option<&StageLayerProjection> {
        self.layers
            .iter()
            .find(|(id, _)| *id == layer)
            .map(|(_, p)| p)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum StageGeometryError {
    #[error(transparent)]
    Param(#[from] ParamEvalError),
    #[error(transparent)]
    Camera(#[from] CompCameraError),
    #[error("singular transform (non-invertible) on layer {layer:?}")]
    SingularTransform { layer: LayerId },
    #[error("rect layer source missing param `{param}` (layer {layer:?})")]
    MissingRectParam { layer: LayerId, param: &'static str },
    #[error("visible layer {layer:?} missing from document tree")]
    LayerMissing { layer: LayerId },
}

/// published Document と評価時刻から、可視 layer の幾何を型付きで投影する。
pub fn project_stage_geometry(
    document: &Document,
    eval: EvaluationTime,
    tracks: &DataTracks,
) -> Result<StageGeometryProjection, StageGeometryError> {
    let t = eval.timeline_time;
    let camera = eval_camera(document, eval, tracks)?;
    let camera_view = camera_view_affine(camera);
    // camera_view 自体が特異なら後続の screen↔局所換算が成立しない。
    if camera_view.try_invert().is_none() {
        return Err(StageGeometryError::SingularTransform {
            layer: LayerId::from_raw(0),
        });
    }

    let (resolved, _) = resolve_document_spaces(document, t, tracks)?;
    let visible = visible_layers_at(document, t);
    let mut layers = Vec::with_capacity(visible.len());

    for layer in visible {
        let item =
            find_track_item(document, layer).ok_or(StageGeometryError::LayerMissing { layer })?;
        match item {
            TrackItem::Group(_) => {
                layers.push((
                    layer,
                    StageLayerProjection::Unavailable(StageGeometryUnavailable::Group { layer }),
                ));
            }
            TrackItem::Clip(clip) => {
                match project_clip(document, clip, layer, t, tracks, &resolved, camera_view)? {
                    Some(projection) => layers.push((layer, projection)),
                    None => {}
                }
            }
        }
    }

    Ok(StageGeometryProjection {
        layers,
        camera_view,
    })
}

fn project_clip(
    document: &Document,
    clip: &Clip,
    layer: LayerId,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    camera_view: Affine2D,
) -> Result<Option<StageLayerProjection>, StageGeometryError> {
    match &clip.source {
        // AG-1: audio-only は visual に参加しない。正しい不在。
        ClipSource::Asset { video: None, .. } => Ok(None),
        ClipSource::Asset { video: Some(_), .. } => Ok(Some(StageLayerProjection::Unavailable(
            StageGeometryUnavailable::VideoSource { layer },
        ))),
        ClipSource::Vector { .. } => Ok(Some(StageLayerProjection::Unavailable(
            StageGeometryUnavailable::VectorSource { layer },
        ))),
        ClipSource::Plugin {
            plugin_id, params, ..
        } if plugin_id == RECT_LAYER_SOURCE => {
            let center_p = params
                .get("center")
                .ok_or(StageGeometryError::MissingRectParam {
                    layer,
                    param: "center",
                })?;
            let size_p = params
                .get("size")
                .ok_or(StageGeometryError::MissingRectParam {
                    layer,
                    param: "size",
                })?;
            let center = param_eval::eval_vec2(center_p, t, tracks, resolved)?;
            let size = param_eval::eval_vec2(size_p, t, tracks, resolved)?;
            let world = resolve_layer_world(
                document,
                &clip.envelope.transform,
                layer,
                t,
                tracks,
                resolved,
            )?;
            if world.try_invert().is_none() || (camera_view * world).try_invert().is_none() {
                return Err(StageGeometryError::SingularTransform { layer });
            }
            Ok(Some(StageLayerProjection::Available(StageLayerGeometry {
                local_rect: StageLocalRect {
                    center: CanonicalPoint {
                        x: center[0],
                        y: center[1],
                    },
                    size: CanonicalSize {
                        width: size[0],
                        height: size[1],
                    },
                },
                world,
                camera_view,
            })))
        }
        ClipSource::Plugin { .. } => Ok(Some(StageLayerProjection::Unavailable(
            StageGeometryUnavailable::PluginSource { layer },
        ))),
    }
}

fn resolve_layer_world(
    document: &Document,
    xform: &Transform2D,
    layer: LayerId,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<Affine2D, StageGeometryError> {
    let lookup = |id: LayerId| -> Option<&Transform2D> { find_transform(document, id) };
    Ok(resolve_transform(
        xform,
        t,
        tracks,
        resolved,
        &lookup,
        Some(layer),
    )?)
}

/// `camera_view_affine` と同一構成: S(1/h)·R(-roll)·T(-center)。
fn camera_view_affine(camera: CompCamera) -> Affine2D {
    let center = camera.center();
    let inv_h = 1.0 / camera.height();
    Affine2D::scale(inv_h, inv_h)
        * Affine2D::rotation(-camera.roll_radians())
        * Affine2D::translation(-center.x, -center.y)
}

/// `eval_comp_camera_doc` は crate 外へ未再exportのため、公開部品で同値を構成する。
fn eval_camera(
    doc: &Document,
    eval: EvaluationTime,
    tracks: &DataTracks,
) -> Result<CompCamera, StageGeometryError> {
    let resolved = ResolvedLayerParams::default();
    let t = eval.timeline_time;
    let CompCameraDoc::PlanarOrthographic {
        center,
        roll_radians,
        height,
    } = &doc.composition.camera;

    let center_v = param_eval::eval_vec2(center, t, tracks, &resolved)?;
    let roll = param_eval::eval_f64(roll_radians, t, tracks, &resolved)?;
    let h = param_eval::eval_f64(height, t, tracks, &resolved)?;

    Ok(CompCamera::try_new(
        CanonicalPoint {
            x: center_v[0],
            y: center_v[1],
        },
        roll,
        h,
        doc.composition.aspect_num(),
        doc.composition.aspect_den(),
    )?)
}

fn find_track_item(doc: &Document, target: LayerId) -> Option<&TrackItem> {
    fn walk(items: &[TrackItem], target: LayerId) -> Option<&TrackItem> {
        for item in items {
            let layer = match item {
                TrackItem::Clip(c) => c.envelope.layer_id,
                TrackItem::Group(g) => g.envelope.layer_id,
            };
            if layer == target {
                return Some(item);
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, target) {
                    return Some(found);
                }
            }
        }
        None
    }
    doc.tracks.iter().find_map(|t| walk(&t.items, target))
}

fn find_transform(doc: &Document, target: LayerId) -> Option<&Transform2D> {
    find_track_item(doc, target).map(|item| match item {
        TrackItem::Clip(c) => &c.envelope.transform,
        TrackItem::Group(g) => &g.envelope.transform,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_doc::{
        Clip, DocParam, Group, ItemEnvelope, LookAtAxis, Track, TrackItem, Transform2D,
    };
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
        start: RationalTime,
        duration: RationalTime,
        center: [f64; 2],
        size: [f64; 2],
        transform: Transform2D,
    ) -> LayerId {
        let layer = doc.layers.allocate(name).unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform = transform;
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start,
            duration,
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
    fn default_camera_non_rotated_rect_matches_overlay_center_size() {
        let mut doc = base_doc();
        let center = [0.1, -0.2];
        let size = [0.3, 0.4];
        let layer = push_rect(
            &mut doc,
            "r",
            RationalTime::ZERO,
            sec(10),
            center,
            size,
            Transform2D::identity(),
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        let StageLayerProjection::Available(geo) = proj.get(layer).unwrap() else {
            panic!("expected available");
        };
        assert_eq!(geo.local_rect.center.x, center[0]);
        assert_eq!(geo.local_rect.center.y, center[1]);
        assert_eq!(geo.local_rect.size.width, size[0]);
        assert_eq!(geo.local_rect.size.height, size[1]);
        assert_eq!(geo.world, Affine2D::IDENTITY);
        assert_eq!(geo.camera_view, Affine2D::IDENTITY);
    }

    #[test]
    fn parented_layer_world_matches_resolve_transform() {
        let mut doc = base_doc();
        let parent = doc.layers.allocate("parent").unwrap();
        let mut parent_env = ItemEnvelope::new(parent);
        parent_env.transform.position = DocParam::const_vec2([10.0, 0.0]);
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope: parent_env,
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params([0.0, 0.0], [0.1, 0.1]),
                extra: Default::default(),
            },
        }));
        let child_xform = Transform2D {
            position: DocParam::const_vec2([1.0, 2.0]),
            parent: Some(parent),
            ..Transform2D::identity()
        };
        let child = push_rect(
            &mut doc,
            "child",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [0.2, 0.2],
            child_xform.clone(),
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let t = RationalTime::ZERO;
        let (resolved, _) = resolve_document_spaces(&doc, t, &tracks).unwrap();
        let lookup = |id: LayerId| find_transform(&doc, id);
        let expected =
            resolve_transform(&child_xform, t, &tracks, &resolved, &lookup, Some(child)).unwrap();
        let proj = project_stage_geometry(&doc, EvaluationTime::new(t), &tracks).unwrap();
        let StageLayerProjection::Available(geo) = proj.get(child).unwrap() else {
            panic!("expected available");
        };
        assert_eq!(geo.world, expected);
    }

    #[test]
    fn rotation_and_look_at_corners_match_camera_view_times_world() {
        let mut doc = base_doc();
        let target = push_rect(
            &mut doc,
            "target",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [0.1, 0.1],
            Transform2D {
                position: DocParam::const_vec2([1.0, 0.0]),
                ..Transform2D::identity()
            },
        );
        let look = Transform2D {
            position: DocParam::const_vec2([0.0, 0.0]),
            rotation: DocParam::LookAt {
                target,
                axis: LookAtAxis::PlusX,
            },
            ..Transform2D::identity()
        };
        let layer = push_rect(
            &mut doc,
            "look",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [0.5, 0.25],
            look,
        );
        doc.composition.camera = CompCameraDoc::PlanarOrthographic {
            center: DocParam::const_vec2([0.1, 0.2]),
            roll_radians: DocParam::const_f64(0.3),
            height: DocParam::const_f64(2.0),
        };
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        let StageLayerProjection::Available(geo) = proj.get(layer).unwrap() else {
            panic!("expected available");
        };
        let composed = geo.camera_view * geo.world;
        let hw = geo.local_rect.size.width * 0.5;
        let hh = geo.local_rect.size.height * 0.5;
        let cx = geo.local_rect.center.x;
        let cy = geo.local_rect.center.y;
        let corners = [
            [cx - hw, cy - hh],
            [cx + hw, cy - hh],
            [cx + hw, cy + hh],
            [cx - hw, cy + hh],
        ];
        for c in corners {
            let local = geo.world.transform_point(c[0], c[1]);
            let via_view = geo.camera_view.transform_point(local[0], local[1]);
            let direct = composed.transform_point(c[0], c[1]);
            assert!((via_view[0] - direct[0]).abs() < 1e-12);
            assert!((via_view[1] - direct[1]).abs() < 1e-12);
        }
    }

    #[test]
    fn camera_center_roll_height_affect_projection() {
        let mut doc = base_doc();
        let layer = push_rect(
            &mut doc,
            "r",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        doc.composition.camera = CompCameraDoc::PlanarOrthographic {
            center: DocParam::const_vec2([0.5, -0.25]),
            roll_radians: DocParam::const_f64(std::f64::consts::FRAC_PI_4),
            height: DocParam::const_f64(2.0),
        };
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        let expected = camera_view_affine(
            CompCamera::try_new(
                CanonicalPoint { x: 0.5, y: -0.25 },
                std::f64::consts::FRAC_PI_4,
                2.0,
                doc.composition.aspect_num(),
                doc.composition.aspect_den(),
            )
            .unwrap(),
        );
        assert_eq!(proj.camera_view(), expected);
        let StageLayerProjection::Available(geo) = proj.get(layer).unwrap() else {
            panic!("expected available");
        };
        assert_eq!(geo.camera_view, expected);
    }

    #[test]
    fn audio_only_clip_absent_from_projection() {
        let mut doc = base_doc();
        let asset = doc.assets.allocate("a", "audio/wav", "hash").unwrap();
        let layer = doc.layers.allocate("audio").unwrap();
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::Asset {
                asset,
                video: None,
                audio: vec![motolii_doc::AudioComponent::ordinal(0)],
            },
        }));
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        assert!(proj.get(layer).is_none());
        assert!(proj.layers().is_empty());
    }

    #[test]
    fn inactive_clip_time_yields_no_geometry() {
        let mut doc = base_doc();
        let layer = push_rect(
            &mut doc,
            "r",
            sec(5),
            sec(2),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj = project_stage_geometry(&doc, EvaluationTime::new(sec(1)), &tracks).unwrap();
        assert!(proj.get(layer).is_none());
    }

    #[test]
    fn invisible_or_unsoloed_item_yields_no_geometry() {
        let mut doc = base_doc();
        let hidden = push_rect(
            &mut doc,
            "hidden",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        match &mut doc.tracks[0].items[0] {
            TrackItem::Clip(c) => c.envelope.visible = false,
            _ => panic!("clip"),
        }
        let solo = push_rect(
            &mut doc,
            "solo",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        match &mut doc.tracks[0].items[1] {
            TrackItem::Clip(c) => c.envelope.solo = true,
            _ => panic!("clip"),
        }
        let other = push_rect(
            &mut doc,
            "other",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D::identity(),
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        assert!(proj.get(hidden).is_none());
        assert!(proj.get(other).is_none());
        assert!(matches!(
            proj.get(solo),
            Some(StageLayerProjection::Available(_))
        ));
    }

    #[test]
    fn group_video_vector_plugin_are_typed_unavailable() {
        let mut doc = base_doc();
        let asset = doc.assets.allocate("v", "video/mp4", "hash").unwrap();
        let group_id = doc.layers.allocate("g").unwrap();
        let video_id = doc.layers.allocate("video").unwrap();
        let vector_id = doc.layers.allocate("vector").unwrap();
        let plugin_id = doc.layers.allocate("plugin").unwrap();
        doc.tracks[0].items.push(TrackItem::Group(Group {
            envelope: ItemEnvelope::new(group_id),
            children: vec![],
        }));
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(video_id),
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        }));
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(vector_id),
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::Vector {
                recipe: motolii_doc::VectorRecipe {
                    content: motolii_doc::VectorContent::StandardShape {
                        shape: motolii_doc::StandardShape::Rect {
                            width: DocParam::const_f64(1.0),
                            height: DocParam::const_f64(1.0),
                        },
                    },
                    modifiers: vec![],
                },
            },
        }));
        doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(plugin_id),
            start: RationalTime::ZERO,
            duration: sec(10),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: "other.plugin".into(),
                effect_version: 1,
                params: BTreeMap::new(),
                extra: Default::default(),
            },
        }));
        // other.plugin は validate/prepare で落ちうるので validate 無しで投影だけ見る。
        let tracks = DataTracks::new();
        let proj =
            project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks).unwrap();
        assert!(matches!(
            proj.get(group_id),
            Some(StageLayerProjection::Unavailable(
                StageGeometryUnavailable::Group { .. }
            ))
        ));
        assert!(matches!(
            proj.get(video_id),
            Some(StageLayerProjection::Unavailable(
                StageGeometryUnavailable::VideoSource { .. }
            ))
        ));
        assert!(matches!(
            proj.get(vector_id),
            Some(StageLayerProjection::Unavailable(
                StageGeometryUnavailable::VectorSource { .. }
            ))
        ));
        assert!(matches!(
            proj.get(plugin_id),
            Some(StageLayerProjection::Unavailable(
                StageGeometryUnavailable::PluginSource { .. }
            ))
        ));
    }

    #[test]
    fn singular_scale_returns_typed_error_without_panic() {
        let mut doc = base_doc();
        let layer = push_rect(
            &mut doc,
            "r",
            RationalTime::ZERO,
            sec(10),
            [0.0, 0.0],
            [1.0, 1.0],
            Transform2D {
                scale: DocParam::const_vec2([0.0, 1.0]),
                ..Transform2D::identity()
            },
        );
        doc.validate().unwrap();
        let tracks = DataTracks::new();
        let err = project_stage_geometry(&doc, EvaluationTime::new(RationalTime::ZERO), &tracks)
            .unwrap_err();
        assert_eq!(err, StageGeometryError::SingularTransform { layer });
    }
}
