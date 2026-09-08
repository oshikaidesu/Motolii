use crate::doc::store::*;
use crate::doc::vector::{Brush,Contour,Fill,FillRule,Rgb,Vertex};
use crate::editor::fixture;
#[derive(Clone)]
pub(crate) enum NewKind {
    Text,
    Camera,
    Stage,
    Rectangle,
    Bezier,
    Cube { path: String },
    Media { path: String, name: String },
}

/// 同梱の素材を `~/.local/share/motolii/builtins` へ 1 回だけ書き出し、その path を返す。
fn builtin(file: &str, bytes: &[u8]) -> Result<String, String> {
    let home = std::env::var_os("HOME").ok_or("User data directory unavailable")?;
    let directory = std::path::PathBuf::from(home).join(".local/share/motolii/builtins");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join(file);
    if std::fs::metadata(&path).ok().map(|m| m.len() as usize) != Some(bytes.len()) {
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn cube() -> Result<NewKind, String> {
    Ok(NewKind::Cube { path: builtin("cube-v1.obj", include_bytes!("../../assets/cube.obj"))? })
}

pub(crate) struct Background { pub id: &'static str, pub name: &'static str, pub path: String }

/// 同梱の背景(Poly Haven の HDRI、CC0、1k)。置くと環境層になり、空として描かれ、光にもなる。
pub(crate) fn backgrounds() -> &'static [Background] {
    static MADE: std::sync::OnceLock<Vec<Background>> = std::sync::OnceLock::new();
    MADE.get_or_init(|| {
        let table: [(&str, &str, &[u8]); 6] = [
            ("partly-cloudy-sky", "Partly cloudy sky", include_bytes!("../../assets/backgrounds/kloofendal_48d_partly_cloudy_puresky.hdr")),
            ("sunset-sky", "Sunset sky", include_bytes!("../../assets/backgrounds/the_sky_is_on_fire.hdr")),
            ("night-sky", "Night sky", include_bytes!("../../assets/backgrounds/moonless_golf.hdr")),
            ("meadow", "Meadow", include_bytes!("../../assets/backgrounds/meadow_2.hdr")),
            ("city-night", "City at night", include_bytes!("../../assets/backgrounds/shanghai_bund.hdr")),
            ("photo-studio", "Photo studio", include_bytes!("../../assets/backgrounds/brown_photostudio_02.hdr")),
        ];
        table.into_iter().filter_map(|(id, name, bytes)| {
            let path = builtin(&format!("background-{id}-v1.hdr"), bytes).ok()?;
            Some(Background { id, name, path })
        }).collect()
    })
}

pub(crate) fn background(id: &str) -> Result<NewKind, String> {
    let b = backgrounds().iter().find(|b| b.id == id).ok_or("Unknown background")?;
    Ok(NewKind::Media { path: b.path.clone(), name: b.name.into() })
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
/// 尺の無い物(静止画・文字・図形)の既定の長さ = **見えている Timeline の幅の 3/4**。
/// comp の終わりまで伸ばすと終端のトリムが画面外に出て扱えない(2026-09-07 利用者)。
/// 動画・音のように尺のある物は素材の尺が勝つ。
pub(crate) const UNBOUNDED_SPAN_OF_VIEW: (i64, i64) = (3, 4);

/// UI が見えているコマ数を渡した時だけ短くする。渡さなければ従来どおり comp の終わりまで。
pub(crate) fn unbounded_frames(visible_frames: Option<i64>) -> Option<i64> {
    visible_frames
        .filter(|v| *v > 0)
        .map(|v| (v * UNBOUNDED_SPAN_OF_VIEW.0 / UNBOUNDED_SPAN_OF_VIEW.1).max(1))
}

pub(crate) fn new_layer_intents(
    layer: LayerId,
    order: i16,
    playhead: i64,
    duration_frames: i64,
    fps: crate::doc::store::Fps,
    comp: (f64, f64),
    kind: NewKind,
    unbounded: Option<i64>,
) -> Vec<Intent> {
    let label_color = Some(Some((layer.0 % fixture::LABEL_PALETTE.len() as u64) as u8));
    match kind {
        NewKind::Camera => vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Camera, order, timing: LayerTiming::place(playhead,None,duration_frames) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("Camera".into()), label_color, ..Default::default() } },
        ],
        NewKind::Stage => vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Stage, order, timing: LayerTiming::place(playhead,None,duration_frames) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("Stage".into()), label_color, ..Default::default() } },
        ],
        NewKind::Cube { path } => {
            let mut out = new_layer_intents(layer, order, playhead, duration_frames, fps, comp, NewKind::Media { path, name: "Cube".into() }, unbounded);
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
            // HDRI は空として置く。板として使いたければ Inspector で Environment を切る。
            let environment = crate::render::media::is_environment_image_path(&path);
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
                        timing: LayerTiming::place(playhead, source_frames.or(unbounded), duration_frames),
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
                        environment: Some(environment),
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
                        timing: LayerTiming::place(playhead, unbounded, duration_frames),
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
                        timing: LayerTiming::place(playhead, unbounded, duration_frames),
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
                        timing: LayerTiming::place(playhead, unbounded, duration_frames),
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
    /// 尺の無い物は見えている幅の 3/4、尺のある物と、幅を渡さない時は従来どおり。
    #[test]
    fn unbounded_layers_take_three_quarters_of_the_visible_span() {
        use crate::doc::store::{Document, Fps, Intent, LayerId, LayerMeta, Composition};
        assert_eq!(super::unbounded_frames(Some(120)), Some(90));
        assert_eq!(super::unbounded_frames(Some(1)), Some(1));
        assert_eq!(super::unbounded_frames(None), None);
        let fps = Fps::try_new(30, 1).unwrap();
        let timing_of = |visible: Option<i64>| {
            let intents = super::new_layer_intents(LayerId(1), 0, 10, 300, fps, (1920.0, 1080.0), super::NewKind::Rectangle, visible);
            intents.into_iter().find_map(|i| match i { Intent::SetMeta { meta: LayerMeta { timing, .. }, .. } => Some(timing), _ => None }).unwrap()
        };
        assert_eq!(timing_of(Some(90)).duration, 90, "90 = 120 の 3/4");
        assert_eq!(timing_of(None).duration, 290, "渡さなければ comp の終わりまで");
        let _ = (Document::new(), Composition::default_background());
    }

    #[test]
    fn camera_layer_uses_normal_properties_lifetime_and_undo() {
        let mut doc = blank_project();
        let layer = LayerId(1);
        let fps = Fps::try_new(30,1).unwrap();
        doc.apply_all(new_layer_intents(layer,0,0,60,fps,(1920.0,1080.0),NewKind::Camera,None)).unwrap();
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

#[cfg(test)]
mod environment_media {
    use super::*;

    /// `.hdr` / `.exr` を置くと最初から環境層。普通の画は板のまま。
    #[test]
    fn hdr_is_placed_as_the_environment() {
        let fps = crate::doc::store::Fps::try_new(30, 1).unwrap();
        let environment_of = |path: &str| {
            let intents = new_layer_intents(LayerId(7), 0, 0, 90, fps, (1280.0, 720.0), NewKind::Media { path: path.into(), name: "x".into() }, None);
            intents.iter().find_map(|i| match i {
                Intent::SetAttrs { patch, .. } => patch.environment,
                _ => None,
            })
        };
        assert_eq!(environment_of("/nowhere/sky.hdr"), Some(true));
        assert_eq!(environment_of("/nowhere/sky.EXR"), Some(true));
        assert_eq!(environment_of("/nowhere/photo.png"), Some(false));
    }

    /// 同梱の背景は書き出された実ファイルを指し、置くと環境層になる。
    #[test]
    fn bundled_backgrounds_exist_on_disk_and_land_as_environments() {
        assert_eq!(backgrounds().len(), 6);
        for b in backgrounds() {
            assert!(std::path::Path::new(&b.path).exists(), "{}", b.path);
            let NewKind::Media { path, name } = background(b.id).unwrap() else { panic!("media") };
            assert_eq!((path.as_str(), name.as_str()), (b.path.as_str(), b.name));
            assert!(crate::render::media::is_environment_image_path(&path));
        }
        assert!(background("nope").is_err());
    }
}

