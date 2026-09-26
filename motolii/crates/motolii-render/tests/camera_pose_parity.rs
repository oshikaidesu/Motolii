//! Camera pose parity: the same document seen through every camera evaluator the product has.
//! Resolver = `picture::resolve::camera` (document only). Graph = the frame-graph Camera node (what render, export and the
//! Stage read). Bounds = the engine variant that aims a Target layer at the bounds centre (frustum gizmos, feedback replay).
//! Poses are compared, not pixels: eye, orientation, vertical fov, aspect, near plane and near fade.

use motolii_doc::core::{camera_projection, CompSpec, Fps, ResolvedCamera};
use motolii_doc::store::*;
use motolii_edit::{blank_project, Document, Intent};
use motolii_render::frame_graph::{CompiledGraph, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, NodeExecutor, NodeInputs, NodeValue, SceneProgram, SceneProgramError};

struct Executor<'a>(&'a SceneProgram);
impl NodeExecutor for Executor<'_> {
    type Error = SceneProgramError;
    fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
}

/// The production camera: the frame-graph Camera node evaluated for this document at `t`.
fn graph(doc: &Document, t: RationalTime) -> ResolvedCamera {
    let program = SceneProgram::compile(&doc.view()).unwrap();
    let root = program.camera();
    let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
    let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
    let mut executor = Executor(&program);
    let frame = graph.evaluate(&mut executor, t, FrameQuality::Export, Generation::new(1)).unwrap();
    *frame.value(root).and_then(|value| value.downcast_ref::<ResolvedCamera>()).unwrap()
}

fn resolver(doc: &Document, t: RationalTime) -> ResolvedCamera {
    motolii_render::picture::resolve::camera::resolve_camera(&doc.view(), t).unwrap()
}

#[derive(Clone, Copy, Debug)]
struct Pose { eye: glam::Vec3, rotation: glam::Quat, fov: f32, aspect: f32, near_plane: f32, near_fade: f32 }

fn pose(comp: CompSpec, camera: ResolvedCamera) -> Pose {
    let p = camera_projection(comp, camera);
    Pose { eye: p.eye, rotation: p.rotation, fov: p.vertical_fov_radians, aspect: p.aspect_ratio, near_plane: p.near_plane_distance, near_fade: camera.near_fade }
}

fn differences(a: &Pose, b: &Pose) -> Vec<String> {
    let mut out = Vec::new();
    if a.eye.distance(b.eye) > 1e-3 { out.push(format!("eye {:?} vs {:?}", a.eye, b.eye)); }
    // orientation, sign-insensitive: q and -q are the same turn (angle_between is too noisy near equality in f32)
    let (qa, qb) = (a.rotation.normalize(), b.rotation.normalize());
    let d = (qa - qb).length().min((qa + qb).length());
    if d > 1e-5 { out.push(format!("rotation differs by {d}")); }
    if (a.fov - b.fov).abs() > 1e-6 { out.push(format!("fov {} vs {}", a.fov, b.fov)); }
    if (a.aspect - b.aspect).abs() > 1e-6 { out.push(format!("aspect {} vs {}", a.aspect, b.aspect)); }
    if (a.near_plane - b.near_plane).abs() > 1e-9 { out.push("near plane".into()); }
    if (a.near_fade - b.near_fade).abs() > 1e-6 { out.push(format!("near fade {} vs {}", a.near_fade, b.near_fade)); }
    out
}

fn comp(doc: &Document) -> CompSpec { doc.view().composition().unwrap().unwrap().spec() }
fn fps(doc: &Document) -> Fps { doc.view().composition().unwrap().unwrap().fps }
fn frame(doc: &Document, n: i64) -> RationalTime { RationalTime::try_from_frame(n, fps(doc)).unwrap() }
/// A time between two frames.
fn sub_frame(doc: &Document, n: i64) -> RationalTime {
    let f = fps(doc);
    RationalTime::try_new((2 * n + 1) * f.den() as i64, 2 * f.num() as i64).unwrap()
}

fn add(doc: &mut Document, id: u64, source: LayerSource, order: i16, start: i64, frames: i64) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all(vec![Intent::AddLayer(layer), Intent::SetMeta { layer, meta: LayerMeta { source, order, timing: LayerTiming::place(start, None, frames) } }]).unwrap();
    layer
}
fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) { doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap(); }
fn key(doc: &mut Document, layer: LayerId, name: &str, keys: &[(i64, Value)]) {
    let f = fps(doc);
    let mut track = KeyframeTrack::new();
    for (n, value) in keys { track.insert(Keyframe { t: RationalTime::try_from_frame(*n, f).unwrap(), value: value.clone(), interp: Interp::Linear, spatial: None }); }
    doc.apply(Intent::SetTrack { layer, property: PropertyId::new(name).unwrap(), track }).unwrap();
}

/// Every scenario: (name, document, times to sample).
fn scenarios() -> Vec<(&'static str, Document)> {
    let mut all: Vec<(&'static str, Document)> = Vec::new();
    all.push(("no camera at all", blank_project()));
    let mut with_camera = |name: &'static str, build: &dyn Fn(&mut Document, LayerId)| {
        let mut doc = blank_project();
        let cam = add(&mut doc, 1, LayerSource::Camera, 0, 0, 100);
        build(&mut doc, cam);
        all.push((name, doc));
    };
    with_camera("camera layer, defaults", &|_, _| {});
    with_camera("center and target z", &|d, c| { put(d, c, property::CAMERA_CENTER, Value::Vec2([120.0, -40.0])); put(d, c, property::CAMERA_TARGET_Z, Value::F64(300.0)); });
    with_camera("orbit", &|d, c| put(d, c, property::CAMERA_ORBIT, Value::Vec2([-20.0, 35.0])));
    with_camera("distance", &|d, c| put(d, c, property::CAMERA_DISTANCE, Value::F64(2.0)));
    with_camera("zoom", &|d, c| put(d, c, property::CAMERA_ZOOM, Value::F64(1.7)));
    with_camera("roll", &|d, c| put(d, c, property::CAMERA_ROLL, Value::F64(47.0)));
    with_camera("near fade", &|d, c| put(d, c, property::CAMERA_NEAR_FADE, Value::F64(800.0)));
    with_camera("everything at once", &|d, c| {
        put(d, c, property::CAMERA_CENTER, Value::Vec2([-90.0, 60.0]));
        put(d, c, property::CAMERA_TARGET_Z, Value::F64(-150.0));
        put(d, c, property::CAMERA_ORBIT, Value::Vec2([25.0, -140.0]));
        put(d, c, property::CAMERA_DISTANCE, Value::F64(0.6));
        put(d, c, property::CAMERA_ZOOM, Value::F64(2.2));
        put(d, c, property::CAMERA_ROLL, Value::F64(-33.0));
        put(d, c, property::CAMERA_NEAR_FADE, Value::F64(500.0));
    });
    with_camera("keyed rig", &|d, c| {
        key(d, c, property::CAMERA_ORBIT, &[(0, Value::Vec2([0.0, 0.0])), (30, Value::Vec2([40.0, 170.0]))]);
        key(d, c, property::CAMERA_DISTANCE, &[(0, Value::F64(1.0)), (30, Value::F64(3.0))]);
        key(d, c, property::CAMERA_ZOOM, &[(0, Value::F64(1.0)), (30, Value::F64(2.5))]);
        key(d, c, property::CAMERA_ROLL, &[(0, Value::F64(0.0)), (30, Value::F64(90.0))]);
        key(d, c, property::CAMERA_CENTER, &[(0, Value::Vec2([0.0, 0.0])), (30, Value::Vec2([300.0, -200.0]))]);
    });
    // ---- targets ----
    let target_doc = |shape: bool, build: &dyn Fn(&mut Document, LayerId, LayerId)| {
        let mut doc = blank_project();
        let cam = add(&mut doc, 1, LayerSource::Camera, 0, 0, 100);
        let target = add(&mut doc, 2, if shape { LayerSource::Shape } else { LayerSource::Null }, 1, 0, 100);
        if shape { doc.apply(Intent::SetShapes { layer: target, shapes: vec![rect_shape([255; 4], [200.0, 100.0])] }).unwrap(); }
        put(&mut doc, cam, property::CAMERA_TARGET, Value::LayerId(target.0));
        build(&mut doc, cam, target);
        doc
    };
    all.push(("target null with position and z", target_doc(false, &|d, _, t| { put(d, t, property::POSITION, Value::Vec2([400.0, 300.0])); put(d, t, property::POSITION_Z, Value::F64(250.0)); })));
    all.push(("target null keyed", target_doc(false, &|d, _, t| key(d, t, property::POSITION, &[(0, Value::Vec2([100.0, 200.0])), (30, Value::Vec2([700.0, 500.0]))]))));
    all.push(("target null with orbit and distance", target_doc(false, &|d, c, t| { put(d, t, property::POSITION, Value::Vec2([400.0, 300.0])); put(d, c, property::CAMERA_ORBIT, Value::Vec2([15.0, 60.0])); put(d, c, property::CAMERA_DISTANCE, Value::F64(1.5)); })));
    all.push(("target shape, default anchor", target_doc(true, &|d, _, t| put(d, t, property::POSITION, Value::Vec2([300.0, 200.0])))));
    all.push(("target shape, anchor at its centre", target_doc(true, &|d, _, t| { put(d, t, property::POSITION, Value::Vec2([300.0, 200.0])); put(d, t, property::ANCHOR, Value::Vec2([100.0, 50.0])); })));
    all.push(("target shape, moved anchor", target_doc(true, &|d, _, t| { put(d, t, property::POSITION, Value::Vec2([300.0, 200.0])); put(d, t, property::ANCHOR, Value::Vec2([200.0, -80.0])); })));
    all.push(("target shape, scaled and rotated", target_doc(true, &|d, _, t| { put(d, t, property::POSITION, Value::Vec2([600.0, 400.0])); put(d, t, property::SCALE, Value::Vec2([1.5, 0.7])); put(d, t, property::ROTATION, Value::F64(30.0)); })));
    all.push(("target with a parent", target_doc(false, &|d, _, t| {
        let parent = add(d, 3, LayerSource::Null, 2, 0, 100);
        put(d, parent, property::POSITION, Value::Vec2([1000.0, 0.0]));
        put(d, t, property::POSITION, Value::Vec2([100.0, 200.0]));
        d.apply(Intent::SetAttrs { layer: t, patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() } }).unwrap();
    })));
    all.push(("target is the camera itself", target_doc(false, &|d, c, _| put(d, c, property::CAMERA_TARGET, Value::LayerId(1)))));
    all.push(("target id 0", target_doc(false, &|d, c, _| put(d, c, property::CAMERA_TARGET, Value::LayerId(0)))));
    all.push(("target removed", target_doc(false, &|d, _, t| d.apply(Intent::RemoveLayer(t)).unwrap())));
    // ---- which camera is active ----
    let cameras = |a: (i16, i64, i64, f64, bool, bool), b: (i16, i64, i64, f64, bool, bool)| {
        let mut doc = blank_project();
        for (id, (order, start, frames, zoom, hidden, solo)) in [(1u64, a), (2, b)] {
            let cam = add(&mut doc, id, LayerSource::Camera, order, start, frames);
            put(&mut doc, cam, property::CAMERA_ZOOM, Value::F64(zoom));
            doc.apply(Intent::SetAttrs { layer: cam, patch: LayerAttrsPatch { hidden: Some(hidden), solo: Some(solo), ..Default::default() } }).unwrap();
        }
        doc
    };
    all.push(("two cameras, topmost wins", cameras((0, 0, 50, 2.0, false, false), (5, 20, 80, 3.0, false, false))));
    all.push(("two cameras, the top one hidden", cameras((0, 0, 100, 2.0, false, false), (5, 0, 100, 3.0, true, false))));
    all.push(("two cameras, the lower one solo", cameras((0, 0, 100, 2.0, false, true), (5, 0, 100, 3.0, false, false))));
    // hidden and solo can be keyed properties; the resolver reads them at the time, the graph path reads the static flag
    let keyed_flag = |name: &str| {
        let mut doc = cameras((0, 0, 100, 2.0, false, false), (5, 0, 100, 3.0, false, false));
        let f = fps(&doc);
        let mut track = KeyframeTrack::new();
        for (n, on) in [(0i64, true), (20, false)] { track.insert(Keyframe { t: RationalTime::try_from_frame(n, f).unwrap(), value: Value::Bool(on), interp: Interp::Hold, spatial: None }); }
        let property = if name == "hidden" { PropertyId::hidden() } else { PropertyId::solo() };
        doc.apply(Intent::SetTrack { layer: LayerId(2), property, track }).unwrap();
        doc
    };
    all.push(("upper camera hidden by a keyed track until frame 20", keyed_flag("hidden")));
    // ---- the composition-level camera track ----
    let comp_track = |layer: bool| {
        let mut doc = blank_project();
        for (name, value) in [(property::CAMERA_CENTER, Value::Vec2([210.0, -130.0])), (property::CAMERA_ZOOM, Value::F64(1.7)), (property::CAMERA_ROLL, Value::F64(47.0))] {
            doc.apply(Intent::SetCameraConstant { property: PropertyId::camera(name).unwrap(), value }).unwrap();
        }
        if layer { add(&mut doc, 1, LayerSource::Camera, 0, 0, 100); }
        doc
    };
    all.push(("composition camera track alone", comp_track(false)));
    all.push(("composition camera track under a camera layer", comp_track(true)));
    // ---- Framing Size ----
    all.push(("framing size on a target shape", target_doc(true, &|d, c, t| { put(d, t, property::POSITION, Value::Vec2([300.0, 200.0])); put(d, c, property::CAMERA_FRAMING, Value::F64(0.5)); })));
    all.push(("framing size without a target", { let mut doc = blank_project(); let c = add(&mut doc, 1, LayerSource::Camera, 0, 0, 100); put(&mut doc, c, property::CAMERA_FRAMING, Value::F64(0.5)); doc }));
    all
}

/// The times each scenario is sampled at: exact frames and a time between frames.
fn times(doc: &Document) -> Vec<(&'static str, RationalTime)> {
    vec![("frame 0", frame(doc, 0)), ("frame 7", frame(doc, 7)), ("between 7 and 8", sub_frame(doc, 7)), ("frame 30", frame(doc, 30))]
}

/// The engine's bounds variant: the resolver's camera, with a Target layer aimed at the bounds centre.
#[test]
fn survey_bounds_variant_against_graph() {
    let mut engine = motolii_render::engine::Engine::new().unwrap();
    let mut differ = Vec::new();
    for (name, doc) in scenarios() {
        let c = comp(&doc);
        let view = doc.view();
        let t = frame(&doc, 7);
        let Some(id) = motolii_render::picture::resolve::camera::active_camera_layer(&view, t).unwrap() else { continue };
        let spec = view.composition().unwrap().unwrap().spec();
        let texture = engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
            label: Some("camera parity"), size: wgpu::Extent3d { width: spec.width, height: spec.height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: motolii_render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
        });
        engine.render_frame_into(&view, t, &texture).unwrap();
        let resolved = motolii_render::picture::resolve::resolved_layers(&view, t).unwrap();
        let bounds = engine.camera_of_layer_in(&view, &resolved, id, t).unwrap();
        let g = graph(&doc, t);
        let d = differences(&pose(c, bounds), &pose(c, g));
        println!("BOUNDS {name}: {}", if d.is_empty() { "same".to_string() } else { d.join("; ") });
        if bounds != g { println!("    bounds {bounds:?}\n    graph  {g:?}"); differ.push(name); }
    }
    // The bounds variant aims a shape target at its bounds centre and reads the animated hidden flag; the production graph
    // aims at the layer origin and reads the static flag. Every other ordinary scenario is identical.
    assert_eq!(differ, KNOWN_DIVERGENCE_BOUNDS_VS_GRAPH, "current divergence set changed (pinned as observed, not as desired semantics)");
}

const KNOWN_DIVERGENCE_BOUNDS_VS_GRAPH: [&str; 7] = [
    "target shape, default anchor", "target shape, anchor at its centre", "target shape, moved anchor", "target shape, scaled and rotated",
    "target is the camera itself", "upper camera hidden by a keyed track until frame 20", "framing size on a target shape",
];
const KNOWN_DIVERGENCE_RESOLVER_VS_GRAPH: [&str; 4] = ["target is the camera itself", "upper camera hidden by a keyed track until frame 20", "composition camera track alone", "framing size on a target shape"];

/// Characterisation: print how the resolver and the graph differ, per scenario. Run with --nocapture.
#[test]
fn survey_resolver_against_graph() {
    let mut equal = 0;
    let mut differ: Vec<&str> = Vec::new();
    for (name, doc) in scenarios() {
        let c = comp(&doc);
        for (label, t) in times(&doc) {
            let (r, g) = (resolver(&doc, t), graph(&doc, t));
            let d = differences(&pose(c, r), &pose(c, g));
            println!("{name} @ {label}: {}", if d.is_empty() { "same".to_string() } else { d.join("; ") });
            if r != g { println!("    fields differ:\n      resolver {r:?}\n      graph    {g:?}"); if !differ.contains(&name) { differ.push(name); } } else { equal += 1; }
        }
    }
    println!("{equal} samples equal");
    assert_eq!(differ, KNOWN_DIVERGENCE_RESOLVER_VS_GRAPH, "current divergence set changed (pinned as observed, not as desired semantics)");
}
