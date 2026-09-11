use crate::doc::store::*;
use crate::doc::vector::{Brush,Contour,Fill,FillRule,Rgb,Vertex};
use crate::editor::fixture;
#[derive(Clone)]
pub(crate) enum NewKind {
    Text,
    Camera,
    Stage,
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Star,
    Polygon,
    Line,
    Bezier,
    Null,
    Primitive { path: String, name: String },
    Media { path: String, name: String },
}

/// 同梱の素材を `~/.local/share/motolii/builtins` へ 1 回だけ書き出し、その path を返す。
pub(crate) fn builtin(file: &str, bytes: &[u8]) -> Result<String, String> {
    let home = std::env::var_os("HOME").ok_or("User data directory unavailable")?;
    let directory = std::path::PathBuf::from(home).join(".local/share/motolii/builtins");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join(file);
    if std::fs::metadata(&path).ok().map(|m| m.len() as usize) != Some(bytes.len()) {
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) struct Primitive { pub id: &'static str, pub name: &'static str, pub path: String }

/// 同梱の基本形(Blender の Add ▸ Mesh の並び)。どれも cube と同じ 270 の箱に収まり、置くと 3D 層になる。
pub(crate) fn primitives() -> &'static [Primitive] {
    static MADE: std::sync::OnceLock<Vec<Primitive>> = std::sync::OnceLock::new();
    MADE.get_or_init(|| {
        let table: [(&str, &str, &[u8]); 6] = [
            ("plane", "Plane", include_bytes!("../../assets/primitives/plane.obj")),
            ("cube", "Cube", include_bytes!("../../assets/primitives/cube.obj")),
            ("sphere", "Sphere", include_bytes!("../../assets/primitives/sphere.obj")),
            ("cylinder", "Cylinder", include_bytes!("../../assets/primitives/cylinder.obj")),
            ("cone", "Cone", include_bytes!("../../assets/primitives/cone.obj")),
            ("torus", "Torus", include_bytes!("../../assets/primitives/torus.obj")),
        ];
        table.into_iter().filter_map(|(id, name, bytes)| {
            let path = builtin(&format!("{id}-v1.obj"), bytes).ok()?;
            Some(Primitive { id, name, path })
        }).collect()
    })
}

pub(crate) fn primitive(id: &str) -> Result<NewKind, String> {
    let p = primitives().iter().find(|p| p.id == id).ok_or("Unsupported create kind")?;
    Ok(NewKind::Primitive { path: p.path.clone(), name: p.name.into() })
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
    let mut intents = center_intents(layer, [size[0] as f64 * 0.5, size[1] as f64 * 0.5], comp);
    intents.push(put(property::SCALE, Value::Vec2([fit, fit])));
    intents
}
fn center_intents(layer: LayerId, anchor: [f64; 2], comp: (f64, f64)) -> Vec<Intent> {
    [
        (property::ANCHOR, anchor),
        (property::POSITION, [comp.0 * 0.5, comp.1 * 0.5]),
    ].into_iter().map(|(name, value)| Intent::SetConstant {
        layer,
        property: PropertyId::new(name).expect("known property"),
        value: Value::Vec2(value),
    }).collect()
}

fn shape_anchor(shapes: &[ShapeNode]) -> [f64; 2] {
    let bounds = crate::doc::vector::content_bounds(shapes).ok().flatten();
    let canvas = crate::render::engine::content_canvas(shapes).ok().flatten();
    match (bounds, canvas) {
        (Some(b), Some(c)) => [(b[0] + b[2]) * 0.5 + c.origin_x as f64, (b[1] + b[3]) * 0.5 + c.origin_y as f64],
        _ => [0.0, 0.0],
    }
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
/// 図形 1 枚のレシピ: 名前・形・塗りか線・最初から積む効果。AE の shape ツールの既定に合わせる。
struct ShapeRecipe {
    name: &'static str,
    shapes: Vec<ShapeNode>,
    effects: &'static [&'static str],
}

fn white_fill() -> Option<Fill> {
    Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), rule: FillRule::NonZero, opacity: 1.0, hidden: false })
}

fn white_stroke(cap: crate::doc::vector::LineCap) -> Option<crate::doc::vector::Stroke> {
    Some(crate::doc::vector::Stroke {
        brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }),
        width: 6.0,
        cap,
        join: crate::doc::vector::LineJoin::Round,
        miter_limit: 4.0,
        opacity: 1.0,
        hidden: false,
        dash: None,
    })
}

fn vertex(point: VectorPoint, in_tangent: VectorPoint, out_tangent: VectorPoint) -> Vertex {
    Vertex { point, in_tangent, out_tangent }
}

fn shape_recipe(kind: &NewKind, comp: (f64, f64)) -> ShapeRecipe {
    use crate::doc::vector::{LineCap, PathSource, StarType};
    let side = rect_side(comp);
    let square = VectorPoint { x: side, y: side };
    let point = |x: f64, y: f64| VectorPoint { x, y };
    let filled = |name, source| ShapeRecipe { name, shapes: vec![ShapeNode::Leaf(Shape { source, ops: Vec::new(), fill: white_fill(), stroke: None })], effects: &[] };
    let stroked = |name, contour, cap| ShapeRecipe { name, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Bezier(vec![contour]), ops: Vec::new(), fill: None, stroke: white_stroke(cap) })], effects: &[] };
    let star = |name, star_type, inner| filled(name, PathSource::PolyStar { points: 5.0, inner_radius: side * 0.5 * inner, outer_radius: side * 0.5, star_type });
    match kind {
        NewKind::Rectangle => filled("Rectangle", PathSource::Rectangle { size: square }),
        NewKind::RoundedRectangle => ShapeRecipe { effects: &[crate::doc::store::pathop::ROUNDED_CORNERS], ..filled("Rounded Rectangle", PathSource::Rectangle { size: square }) },
        NewKind::Ellipse => filled("Ellipse", PathSource::Ellipse { size: square }),
        NewKind::Star => star("Star", StarType::Star, 0.5),
        NewKind::Polygon => star("Polygon", StarType::Polygon, 1.0),
        NewKind::Line => stroked("Line", Contour { closed: false, vertices: vec![
            vertex(point(-side, 0.0), point(0.0, 0.0), point(0.0, 0.0)),
            vertex(point(side, 0.0), point(0.0, 0.0), point(0.0, 0.0)),
        ] }, LineCap::Butt),
        _ => stroked("Bezier", Contour { closed: false, vertices: vec![
            vertex(point(-150.0, 0.0), point(0.0, 0.0), point(100.0, -150.0)),
            vertex(point(150.0, 0.0), point(-100.0, 150.0), point(0.0, 0.0)),
        ] }, LineCap::Round),
    }
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

/// 描かれる素材は設定の投影で生まれる。Camera・Stageには素材の投影を与えない。
pub(crate) fn prefer_projection(intents: &mut [Intent], projection: LayerProjection) {
    for intent in intents {
        if let Intent::SetAttrs { patch, .. } = intent {
            if patch.projection.is_some() { patch.projection = Some(projection); }
        }
    }
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
        NewKind::Primitive { path, name } => {
            let mut out = new_layer_intents(layer, order, playhead, duration_frames, fps, comp, NewKind::Media { path, name }, unbounded);
            for (name, value) in [
                (property::POSITION, Value::Vec2([comp.0 * 0.5, comp.1 * 0.5])),
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
                center_intents(layer, [natural.0 * 0.5, natural.1 * 0.5], comp)
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
        NewKind::Rectangle | NewKind::RoundedRectangle | NewKind::Ellipse | NewKind::Star | NewKind::Polygon | NewKind::Line | NewKind::Bezier => {
            let recipe = shape_recipe(&kind, comp);
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
                        name: Some(recipe.name.to_owned()),
                        label_color,
                        projection: Some(LayerProjection::TwoPointFiveD),
                        ..Default::default()
                    },
                },
            ];
            if !recipe.effects.is_empty() {
                out.push(Intent::SetEffects {
                    layer,
                    effects: recipe.effects.iter().enumerate().map(|(i, id)| EffectInstance { id: EffectId(i as u32), plugin_id: (*id).to_owned() }).collect(),
                });
            }
            let anchor = shape_anchor(&recipe.shapes);
            out.push(Intent::SetShapes { layer, shapes: recipe.shapes });
            out.extend(center_intents(layer, anchor, comp));
            out
        }
        NewKind::Null => vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order, timing: LayerTiming::place(playhead, unbounded, duration_frames) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("Null".into()), label_color, ..Default::default() } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).expect("known property"), value: Value::Vec2([comp.0 * 0.5, comp.1 * 0.5]) },
        ],
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
            let canvas = crate::doc::vector::Canvas { width: comp.0 as u32, height: comp.1 as u32, origin_x: 0, origin_y: 0 };
            let anchor = out.iter().find_map(|intent| {
                let Intent::SetTextDocument { document, .. } = intent else { return None };
                let t = RationalTime::try_from_frame(playhead, fps).ok()?;
                let shapes = crate::render::engine::text::text_shapes(document, t, &canvas).ok()??;
                let b = crate::doc::vector::content_bounds(&shapes).ok()??;
                Some([(b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5])
            }).unwrap_or([comp.0 * 0.5, comp.1 * 0.5]);
            out.extend(center_intents(layer, anchor, comp));
            out
        }
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;
    #[test]
    fn new_shape_centres_stay_fixed_when_rotated() {
        let fps = Fps::try_new(30, 1).unwrap();
        let comp = (1920.0, 1080.0);
        for kind in [NewKind::Rectangle, NewKind::RoundedRectangle, NewKind::Ellipse,
            NewKind::Star, NewKind::Polygon, NewKind::Line, NewKind::Bezier] {
            let shapes = shape_recipe(&kind, comp).shapes;
            let b = crate::doc::vector::content_bounds(&shapes).unwrap().unwrap();
            let canvas = crate::render::engine::content_canvas(&shapes).unwrap().unwrap();
            let centre = glam::vec2(((b[0] + b[2]) * 0.5 + canvas.origin_x as f64) as f32,
                ((b[1] + b[3]) * 0.5 + canvas.origin_y as f64) as f32);
            let mut doc = blank_project();
            let layer = LayerId(1);
            doc.apply_all(new_layer_intents(layer, 0, 0, 90, fps, comp, kind, None)).unwrap();
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::ROTATION).unwrap(), value: Value::F64(73.0) }).unwrap();
            let world = doc.view().world_transform3d(layer, RationalTime::ZERO).unwrap();
            let actual = world.transform_point3(centre.extend(0.0));
            assert!((actual - glam::vec3(960.0, 540.0, 0.0)).length() < 0.001, "{actual:?}");
        }
    }

    /// 設定「New layers」は平面素材にも空間素材にも同じように効く。
    #[test]
    fn the_projection_preference_applies_to_flat_and_spatial_material() {
        let fps = Fps::try_new(30, 1).unwrap();
        let projection_of = |kind: NewKind, flat: LayerProjection| {
            let mut intents = new_layer_intents(LayerId(1), 0, 0, 90, fps, (1280.0, 720.0), kind, None);
            prefer_projection(&mut intents, flat);
            intents.iter().find_map(|i| match i { Intent::SetAttrs { patch, .. } => patch.projection, _ => None }).unwrap()
        };
        assert_eq!(projection_of(NewKind::Rectangle, LayerProjection::ThreeD), LayerProjection::ThreeD);
        assert_eq!(projection_of(NewKind::Text, LayerProjection::ThreeD), LayerProjection::ThreeD);
        assert_eq!(projection_of(NewKind::Rectangle, LayerProjection::TwoPointFiveD), LayerProjection::TwoPointFiveD);
        let mut spatial = vec![Intent::SetAttrs { layer: LayerId(2), patch: LayerAttrsPatch { projection: Some(LayerProjection::ThreeD), ..Default::default() } }];
        prefer_projection(&mut spatial, LayerProjection::TwoPointFiveD);
        assert!(matches!(&spatial[0], Intent::SetAttrs { patch, .. } if patch.projection == Some(LayerProjection::TwoPointFiveD)));
        assert_eq!(projection_of(primitive("torus").unwrap(), LayerProjection::TwoPointFiveD), LayerProjection::TwoPointFiveD);
        assert_eq!(projection_of(primitive("torus").unwrap(), LayerProjection::ThreeD), LayerProjection::ThreeD);
    }

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

    /// 同梱の基本形は書き出された実ファイルを指し、置くと 3D 層になって cube と同じ箱の中心に立つ。
    #[test]
    fn bundled_primitives_exist_on_disk_and_land_as_3d_layers() {
        assert_eq!(primitives().len(), 6);
        let fps = crate::doc::store::Fps::try_new(30, 1).unwrap();
        for p in primitives() {
            assert!(std::path::Path::new(&p.path).exists(), "{}", p.path);
            let bounds = crate::render::media::load_mesh_bounds(&p.path).unwrap();
            assert!(bounds.radius() > 0.0 && bounds.radius() <= 135.0 * 3f32.sqrt() + 0.1, "{}: {}", p.id, bounds.radius());
            let intents = new_layer_intents(LayerId(3), 0, 0, 90, fps, (1280.0, 720.0), primitive(p.id).unwrap(), None);
            let (mut name, mut projection, mut anchor) = (None, None, None);
            for i in &intents {
                match i {
                    Intent::SetAttrs { patch, .. } => { name = patch.name.clone(); projection = patch.projection.clone(); }
                    Intent::SetConstant { property, value: Value::Vec2(v), .. } if *property == PropertyId::new(property::ANCHOR).unwrap() => anchor = Some(*v),
                    _ => {}
                }
            }
            assert_eq!(name.as_deref(), Some(p.name));
            assert_eq!(projection, Some(LayerProjection::ThreeD));
            let size = bounds.size_xy();
            assert_eq!(anchor, Some([size[0] as f64 * 0.5, size[1] as f64 * 0.5]));
        }
        assert!(primitive("nope").is_err());
    }
}
