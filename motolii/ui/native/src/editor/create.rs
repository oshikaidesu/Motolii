use crate::doc::store::*;
use crate::doc::vector::{Brush,Contour,Fill,FillRule,Rgb,Vertex};
use crate::editor::fixture;
#[derive(Clone)]
pub(crate) enum NewKind {
    Text,
    Camera,
    Rectangle,
    Bezier,
    Cube { path: String },
    Media { path: String, name: String },
}

pub(crate) fn cube() -> Result<NewKind, String> {
    let home = std::env::var_os("HOME").ok_or("User data directory unavailable")?;
    let directory = std::path::PathBuf::from(home).join(".local/share/motolii/builtins");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join("cube-v1.obj");
    let bytes = include_bytes!("../../assets/cube.obj");
    if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(NewKind::Cube { path: path.to_string_lossy().into_owned() })
}

fn spatial_fit_intents(layer: LayerId, path: &str, comp: (f64, f64)) -> Vec<Intent> {
    let bounds = if crate::render::media::is_mesh_path(path) {
        crate::render::media::load_mesh_bounds(path).ok()
    } else {
        crate::render::media::load_point_cloud(std::path::Path::new(path))
            .ok()
            .map(|data| data.bounds())
    };
    let Some(bounds) = bounds else {
        return Vec::new();
    };
    let radius = bounds.radius();
    if radius <= 0.0 {
        return Vec::new();
    }
    let fit = (comp.1 * 0.6) / (radius as f64 * 2.0);
    let size = bounds.size_xy();
    // 置いた時の初期値は**素の値**。0秒のキーにすると、利用者が ◇ を
    // 押していないのに時間の世界が開いてしまう。
    let put = |name: &str, value: Value| Intent::SetConstant {
        layer,
        property: PropertyId::new(name).expect("既知の属性"),
        value,
    };
    vec![
        put(property::SCALE, Value::Vec2([fit, fit])),
        put(
            property::POSITION,
            Value::Vec2([
                (comp.0 - size[0] as f64 * fit) * 0.5,
                (comp.1 - size[1] as f64 * fit) * 0.5,
            ]),
        ),
    ]
}
fn center_intents(layer: LayerId, natural: (f64, f64), comp: (f64, f64)) -> Vec<Intent> {
    let Ok(property) = crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION)
    else {
        return Vec::new();
    };
    let value =
        crate::doc::store::Value::Vec2([(comp.0 - natural.0) * 0.5, (comp.1 - natural.1) * 0.5]);
    vec![Intent::SetConstant {
        layer,
        property,
        value,
    }]
}
fn shape_natural(shapes: &[ShapeNode]) -> (f64, f64) {
    crate::doc::vector::content_bounds(shapes)
        .ok()
        .flatten()
        .map(|b| (b[2] - b[0], b[3] - b[1]))
        .unwrap_or((0.0, 0.0))
}
fn source_frames_in(
    info: &crate::render::media::MediaInfo,
    fps: crate::doc::store::Fps,
) -> Option<i64> {
    let secs = info.duration.map(|d| d.as_seconds_f64()).or_else(|| {
        info.nb_frames
            .map(|n| n as f64 / info.fps.as_f64().max(1e-9))
    })?;
    if secs < 0.2 {
        return None;
    }
    Some((secs * fps.as_f64()).round().max(1.0) as i64)
}
fn rect_side(comp: (f64, f64)) -> f64 {
    (comp.0.min(comp.1) * 0.25).round().max(1.0)
}
pub(crate) fn numbered(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|n| n == base) {
        return base.to_owned();
    }
    (2..)
        .map(|k| format!("{base} {k}"))
        .find(|candidate| !taken.iter().any(|n| n == candidate))
        .expect("an unbounded range always yields")
}
pub(crate) fn new_layer_intents(
    layer: LayerId,
    order: i16,
    playhead: i64,
    duration_frames: i64,
    fps: crate::doc::store::Fps,
    comp: (f64, f64),
    kind: NewKind,
) -> Vec<Intent> {
    let label_color = Some(Some((layer.0 % fixture::LABEL_PALETTE.len() as u64) as u8));
    match kind {
        NewKind::Camera => vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Camera, order, timing: LayerTiming::place(playhead,None,duration_frames) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("Camera".into()), label_color, ..Default::default() } },
        ],
        NewKind::Cube { path } => {
            let mut out = new_layer_intents(layer, order, playhead, duration_frames, fps, comp, NewKind::Media { path, name: "Cube".into() });
            for (name, value) in [
                (property::POSITION, Value::Vec2([comp.0 * 0.5, comp.1 * 0.5])),
                (property::ANCHOR, Value::Vec2([135.0, 135.0])),
                (property::SCALE, Value::Vec2([1.0, 1.0])),
                (property::ROTATION_X, Value::F64(-20.0)),
                (property::ROTATION_Y, Value::F64(30.0)),
            ] {
                out.push(Intent::SetConstant { layer, property: PropertyId::new(name).expect("known property"), value });
            }
            out
        }
        NewKind::Media { path, name } => {
            let spatial = crate::render::media::is_point_cloud_path(&path)
                || crate::render::media::is_mesh_path(&path);
            let info = if spatial {
                None
            } else {
                crate::render::media::probe(&path).ok()
            };
            let fit = if spatial {
                spatial_fit_intents(layer, &path, comp)
            } else {
                let natural = info
                    .as_ref()
                    .map(|i| (i.width as f64, i.height as f64))
                    .unwrap_or((0.0, 0.0));
                center_intents(layer, natural, comp)
            };
            // 動画は**素材の尺**で入る(Premiere・Resolve)。静止画と尺の無い物は comp の終わりまで。
            let source_frames = info.as_ref().and_then(|i| source_frames_in(i, fps));
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::File {
                            path,
                            fingerprint: None,
                        },
                        order,
                        timing: LayerTiming::place(playhead, source_frames, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some(name),
                        label_color,
                        projection: Some(if spatial {
                            LayerProjection::ThreeD
                        } else {
                            LayerProjection::TwoPointFiveD
                        }),
                        ..Default::default()
                    },
                },
            ];
            out.extend(fit);
            out
        }
        NewKind::Rectangle => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Rectangle".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetShapes {
                    layer,
                    shapes: vec![ShapeNode::Leaf(Shape {
                        source: PathSource::Rectangle {
                            size: VectorPoint {
                                x: rect_side(comp),
                                y: rect_side(comp),
                            },
                        },
                        ops: Vec::new(),
                        fill: Some(Fill {
                            brush: Brush::Solid(Rgb {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                            }),
                            rule: FillRule::NonZero,
                            opacity: 1.0,
                            hidden: false,
                        }),
                        stroke: None,
                    })],
                },
            ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Bezier => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Bezier".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetShapes {
                    layer,
                    shapes: vec![ShapeNode::Leaf(Shape {
                        source: PathSource::Bezier(vec![Contour {
                            closed: false,
                            vertices: vec![
                                Vertex {
                                    point: VectorPoint { x: -150.0, y: 0.0 },
                                    in_tangent: VectorPoint { x: 0.0, y: 0.0 },
                                    out_tangent: VectorPoint {
                                        x: 100.0,
                                        y: -150.0,
                                    },
                                },
                                Vertex {
                                    point: VectorPoint { x: 150.0, y: 0.0 },
                                    in_tangent: VectorPoint {
                                        x: -100.0,
                                        y: 150.0,
                                    },
                                    out_tangent: VectorPoint { x: 0.0, y: 0.0 },
                                },
                            ],
                        }]),
                        ops: Vec::new(),
                        fill: None,
                        stroke: Some(crate::doc::vector::Stroke {
                            brush: Brush::Solid(Rgb {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                            }),
                            width: 6.0,
                            cap: crate::doc::vector::LineCap::Round,
                            join: crate::doc::vector::LineJoin::Round,
                            miter_limit: 4.0,
                            opacity: 1.0,
                            hidden: false,
                            dash: None,
                        }),
                    })],
                },
            ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Text => {
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Text,
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        name: Some("Text".to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
                Intent::SetTextDocument {
                    layer,
                    document: TextDocument {
                        content: {
                            let mut track = ContentTrack::new();
                            track.insert(ContentKeyframe {
                                t: RationalTime::try_from_frame(playhead, fps)
                                    .unwrap_or(RationalTime::ZERO),
                                content: "Text".to_owned(),
                            });
                            track
                        },
                        justify: TextJustify::Center,
                        wrap_size: None,
                        styles: vec![TextDocumentStyle {
                            id: TextStyleId(0),
                            font: FontRef {
                                path: "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".to_owned(),
                                fingerprint: None,
                                family: "Hiragino Sans".to_owned(),
                                style: "W3".to_owned(),
                            },
                            size: 96.0,
                            fill: [1.0, 1.0, 1.0, 1.0],
                            // 日本語の歌詞の既定: 行送り 1.5、約物を詰める(palt)、黒の縁取り(背景が動画でも読める)。
                            line_height: Some(96.0 * 1.5),
                            tracking: 0.0,
                            stroke_color: Some([0.0, 0.0, 0.0, 1.0]),
                            stroke_width: 96.0 * 0.08,
                            stroke_over_fill: false,
                            axes: Vec::new(),
                            features: vec![crate::doc::store::TextStyleFeature {
                                tag: "palt".to_owned(),
                                value: 1,
                            }],
                        }],
                        slot_id: None,
                        ranges: Vec::new(),
                        alignment: TextAlignmentOptions::default(),
                        runs: Vec::new(),
                    },
                },
            ];
            // 文字は組んでみるまで大きさが決まらない。実寸が要らない形で
            // 真ん中へ置く —— 左上を枠の中心に合わせる。
            out.extend(
                // 文字の箱は枠と同じ幅・左上起点。中央揃えが枠の中心軸に乗る(揃えは箱の幅で決まる)。
                center_intents(layer, comp, comp),
            );
            out
        }
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;
    #[test]
    fn camera_layer_uses_normal_properties_lifetime_and_undo() {
        let mut doc = blank_project();
        let layer = LayerId(1);
        let fps = Fps::try_new(30,1).unwrap();
        doc.apply_all(new_layer_intents(layer,0,0,60,fps,(1920.0,1080.0),NewKind::Camera)).unwrap();
        let property = PropertyId::new(property::CAMERA_ZOOM).unwrap();
        doc.apply(Intent::SetConstant { layer, property:property.clone(), value:Value::F64(2.0) }).unwrap();
        assert_eq!(doc.view().resolve_camera(RationalTime::ZERO).unwrap().zoom,2.0);
        assert_eq!(doc.view().resolve_camera(RationalTime::from_seconds(3)).unwrap().zoom,1.0);
        doc.apply(Intent::RemoveLayer(layer)).unwrap();
        assert_eq!(doc.view().resolve_camera(RationalTime::ZERO).unwrap().zoom,1.0);
        assert!(doc.undo());
        assert_eq!(doc.view().resolve_camera(RationalTime::ZERO).unwrap().zoom,2.0);
    }
}
