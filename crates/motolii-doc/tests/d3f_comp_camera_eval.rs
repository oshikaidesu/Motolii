//! M2-D3f: Document camera 評価と graph/render 接続。

use std::collections::BTreeMap;

use motolii_core::{
    CanonicalPoint, ColorSpace, CompCamera, CompCameraError, FrameDesc, PixelFormat, Quality,
    RationalTime, TimeMap,
};
use motolii_doc::{
    build_document_frame_graph, migrate_bytes,
    param_eval::{eval_f64, eval_vec2, ResolvedLayerParams},
    CameraEvalError, Clip, ClipSource, CompCameraDoc, Composition, DocKeyframe, DocKeyframeTrack,
    DocParam, DocValue, Document, EvaluationTime, GraphError, ItemEnvelope, LayerId,
    ParamEvalError, Track, TrackItem, CLEAR_LAYER_SOURCE, RECT_LAYER_SOURCE,
};
use motolii_eval::{DataTracks, Interp, Value};
use motolii_gpu::download_rgba;
use motolii_plugin::PluginRuntime;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderStep};
use motolii_testkit::{assert_rgba_close, compare_rgba, gpu_or_skip, tol, RgbaImageDesc};
use serde_json::json;

const W: u32 = 16;
const H: u32 = 9;
const CAM_G0_W: u32 = 16;
const CAM_G0_H: u32 = 8;
const COEFF_TOL: f32 = 1e-5;
const CAM_G0_ORACLE: &str =
    include_str!("../../motolii-render/tests/oracles/cam_g0_planar_identity.tsv");

fn frame_desc() -> FrameDesc {
    FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true)
}

fn reference_runtime() -> PluginRuntime {
    first_party_runtime().unwrap()
}

fn identity_camera(doc: &Document) -> CompCamera {
    CompCamera::try_new(
        CanonicalPoint::CENTER,
        0.0,
        1.0,
        doc.composition.aspect_num(),
        doc.composition.aspect_den(),
    )
    .unwrap()
}

fn clear_clip(layer: u64, color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: CLEAR_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([("color".into(), DocParam::const_color(color))]),
            extra: Default::default(),
        },
    }
}

fn rect_clip(layer: u64, center: [f64; 2], size: [f64; 2], color: [f64; 4]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(LayerId::from_raw(layer)),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2(center)),
                ("size".into(), DocParam::const_vec2(size)),
                ("color".into(), DocParam::const_color(color)),
            ]),
            extra: Default::default(),
        },
    }
}

fn cam_g0_frame_desc() -> FrameDesc {
    FrameDesc::packed(
        CAM_G0_W,
        CAM_G0_H,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )
}

fn cam_g0_image_desc() -> RgbaImageDesc {
    RgbaImageDesc {
        width: CAM_G0_W,
        height: CAM_G0_H,
    }
}

fn rgba8(value: &str) -> [u8; 4] {
    let values = value
        .split(',')
        .map(|component| component.parse::<u8>().expect("oracle rgba8 component"))
        .collect::<Vec<_>>();
    values
        .try_into()
        .expect("oracle rgba8 must have 4 components")
}

fn load_cam_g0_oracle_bytes() -> Vec<u8> {
    let mut rgba_rows = Vec::new();
    for line in CAM_G0_ORACLE.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if let ["rgba", y, x, rgba] = fields.as_slice() {
            rgba_rows.push((
                y.parse::<u32>().expect("oracle rgba y"),
                x.parse::<u32>().expect("oracle rgba x"),
                rgba8(rgba),
            ));
        }
    }
    rgba_rows.sort_by_key(|(y, x, _)| (*y, *x));
    let pixel_count = (CAM_G0_W * CAM_G0_H) as usize;
    assert_eq!(
        rgba_rows.len(),
        pixel_count,
        "CAM-G0 oracle rgba row count must match width*height"
    );
    let mut expected = vec![0u8; pixel_count * 4];
    for (idx, (y, x, rgba)) in rgba_rows.into_iter().enumerate() {
        assert_eq!((y, x), (idx as u32 / CAM_G0_W, idx as u32 % CAM_G0_W));
        let i = idx * 4;
        expected[i..i + 4].copy_from_slice(&rgba);
    }
    expected
}

fn build_cam_g0_document(camera: CompCameraDoc) -> Document {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        i64::from(CAM_G0_W),
        i64::from(CAM_G0_H),
        RationalTime::try_new(10, 1).unwrap(),
        doc.composition.fps,
    )
    .unwrap();
    doc.composition.camera = camera;
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();

    let clear_layer = doc.layers.allocate("clear").unwrap();
    let rect_layer = doc.layers.allocate("rect").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();

    let mut clear = clear_clip(clear_layer.get(), [0.0, 0.0, 0.0, 1.0]);
    clear.envelope.layer_id = clear_layer;
    let mut rect = rect_clip(
        rect_layer.get(),
        [0.0, 0.0],
        [0.5, 0.5],
        [1.0, 0.0, 0.0, 1.0],
    );
    rect.envelope.layer_id = rect_layer;

    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clear), TrackItem::Clip(rect)],
    });
    doc.validate().unwrap();
    doc
}

fn populate_cam_g0_scene(doc: &mut Document) {
    doc.composition = Composition::try_new(
        i64::from(CAM_G0_W),
        i64::from(CAM_G0_H),
        RationalTime::try_new(10, 1).unwrap(),
        doc.composition.fps,
    )
    .unwrap();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();

    let clear_layer = doc.layers.allocate("clear").unwrap();
    let rect_layer = doc.layers.allocate("rect").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();

    let mut clear = clear_clip(clear_layer.get(), [0.0, 0.0, 0.0, 1.0]);
    clear.envelope.layer_id = clear_layer;
    let mut rect = rect_clip(
        rect_layer.get(),
        [0.0, 0.0],
        [0.5, 0.5],
        [1.0, 0.0, 0.0, 1.0],
    );
    rect.envelope.layer_id = rect_layer;

    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clear), TrackItem::Clip(rect)],
    });
    doc.validate().unwrap();
}

fn render_document_gpu(doc: &Document) -> Option<Vec<u8>> {
    let gpu = gpu_or_skip()?;
    let runtime = reference_runtime();
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        cam_g0_frame_desc(),
        &DataTracks::new(),
        &runtime,
        None,
    )
    .unwrap();
    let mut session = RenderSession::new(&gpu);
    let rendered = render_graph_cached(
        &gpu,
        &mut session,
        RationalTime::ZERO,
        &built.graph,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::FINAL,
    )
    .unwrap();
    Some(download_rgba(&gpu, &rendered.texture).unwrap())
}

fn build_world_identity_rect_doc(camera: CompCameraDoc) -> Document {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        i64::from(W),
        i64::from(H),
        doc.composition.duration,
        doc.composition.fps,
    )
    .unwrap();
    doc.composition.camera = camera;
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let layer = doc.layers.allocate("rect").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut clip = rect_clip(layer.get(), [0.0, 0.0], [0.5, 0.5], [1.0, 0.0, 0.0, 1.0]);
    clip.envelope.layer_id = layer;
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(clip)],
    });
    doc.validate().unwrap();
    doc
}

fn build_graph(doc: &Document) -> motolii_doc::DocumentFrameGraph {
    build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc(),
        &DataTracks::new(),
        &reference_runtime(),
        None,
    )
    .unwrap()
}

fn affine_place_steps(built: &motolii_doc::DocumentFrameGraph) -> Vec<[f32; 6]> {
    built
        .graph
        .steps
        .iter()
        .filter_map(|step| match step {
            RenderStep::AffinePlace { inverse_uv, .. } => Some(*inverse_uv),
            _ => None,
        })
        .collect()
}

/// UV→正準: x = a*(u-0.5), y = 0.5-v（`Affine2D::to_inverse_uv_matrix` の C と同じ）。
fn uv_to_canonical(u: f64, v: f64, aspect: f64) -> (f64, f64) {
    (aspect * (u - 0.5), 0.5 - v)
}

/// 正準→UV: u = x/a+0.5, v = 0.5-y（C_inv と同じ）。
fn canonical_to_uv(x: f64, y: f64, aspect: f64) -> (f64, f64) {
    (x / aspect + 0.5, 0.5 - y)
}

/// view(camera)⁻¹: p_src = c + R(r) * (h * p_dst)。実装 helper 非依存。
fn camera_inverse_canonical(
    dst_x: f64,
    dst_y: f64,
    center: [f64; 2],
    roll: f64,
    height: f64,
) -> (f64, f64) {
    let cos_r = roll.cos();
    let sin_r = roll.sin();
    let hx = height * dst_x;
    let hy = height * dst_y;
    (
        center[0] + cos_r * hx - sin_r * hy,
        center[1] + sin_r * hx + cos_r * hy,
    )
}

fn dest_uv_to_src_uv_manual(
    u_dst: f64,
    v_dst: f64,
    center: [f64; 2],
    roll: f64,
    height: f64,
    aspect: f64,
) -> (f64, f64) {
    let (dx, dy) = uv_to_canonical(u_dst, v_dst, aspect);
    let (sx, sy) = camera_inverse_canonical(dx, dy, center, roll, height);
    canonical_to_uv(sx, sy, aspect)
}

fn solve_uv_affine_from_three_points(dst: &[(f64, f64); 3], src: &[(f64, f64); 3]) -> [f64; 6] {
    let mut a = [[0.0f64; 6]; 6];
    let mut b = [0.0f64; 6];
    for i in 0..3 {
        let (du, dv) = dst[i];
        let (su, sv) = src[i];
        a[i] = [du, dv, 1.0, 0.0, 0.0, 0.0];
        b[i] = su;
        a[i + 3] = [0.0, 0.0, 0.0, du, dv, 1.0];
        b[i + 3] = sv;
    }
    for col in 0..6 {
        let pivot = (col..6)
            .max_by(|&i, &j| a[i][col].abs().partial_cmp(&a[j][col].abs()).unwrap())
            .unwrap();
        a.swap(col, pivot);
        b.swap(col, pivot);
        let pivot_val = a[col][col];
        assert!(pivot_val.abs() > 1e-12, "degenerate affine fit");
        for j in col..6 {
            a[col][j] /= pivot_val;
        }
        b[col] /= pivot_val;
        for row in 0..6 {
            if row == col {
                continue;
            }
            let factor = a[row][col];
            if factor == 0.0 {
                continue;
            }
            for j in col..6 {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }
    b
}

fn manual_inverse_uv_oracle(center: [f64; 2], roll: f64, height: f64, aspect: f64) -> [f32; 6] {
    let dst = [(0.12, 0.18), (0.73, 0.31), (0.41, 0.87)];
    let mut src = [(0.0, 0.0); 3];
    for (i, &(u, v)) in dst.iter().enumerate() {
        src[i] = dest_uv_to_src_uv_manual(u, v, center, roll, height, aspect);
    }
    let coeffs = solve_uv_affine_from_three_points(&dst, &src);
    [
        coeffs[0] as f32,
        coeffs[1] as f32,
        coeffs[2] as f32,
        coeffs[3] as f32,
        coeffs[4] as f32,
        coeffs[5] as f32,
    ]
}

fn assert_coefficients_near(got: [f32; 6], want: [f32; 6]) {
    for i in 0..6 {
        assert!(
            (got[i] - want[i]).abs() < COEFF_TOL,
            "coeff[{i}]: got {} want {}",
            got[i],
            want[i]
        );
    }
}

fn assert_coefficients_differ(got: [f32; 6], wrong: [f32; 6]) {
    let max_diff = got
        .iter()
        .zip(wrong.iter())
        .map(|(a, b)| (*a - *b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_diff > COEFF_TOL,
        "expected detectable mismatch, max_diff={max_diff} got={got:?} wrong={wrong:?}"
    );
}

#[test]
fn default_camera_evaluates_to_identity_runtime() {
    let doc = Document::new_current();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc(),
        &DataTracks::new(),
        &reference_runtime(),
        None,
    )
    .unwrap();
    assert_eq!(built.camera, identity_camera(&doc));
}

#[test]
fn animated_camera_matches_world_to_ndc_at_timeline_time() {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        i64::from(W),
        i64::from(H),
        doc.composition.duration,
        doc.composition.fps,
    )
    .unwrap();

    let mut center_track = DocKeyframeTrack::new();
    center_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
    });
    center_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::try_new(2, 1).unwrap(),
        value: DocValue::Vec2([0.2, -0.1]),
        interp: Interp::Linear,
    });

    let mut roll_track = DocKeyframeTrack::new();
    roll_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    roll_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::try_new(2, 1).unwrap(),
        value: DocValue::F64(0.5),
        interp: Interp::Linear,
    });

    let mut height_track = DocKeyframeTrack::new();
    height_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    height_track.insert(DocKeyframe {
        id: motolii_doc::KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::try_new(2, 1).unwrap(),
        value: DocValue::F64(2.0),
        interp: Interp::Linear,
    });

    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::Keyframes(center_track),
        roll_radians: DocParam::Keyframes(roll_track),
        height: DocParam::Keyframes(height_track),
    };
    doc.validate().unwrap();

    let t = RationalTime::try_new(1, 1).unwrap();
    let tracks = DataTracks::new();
    let resolved = ResolvedLayerParams::default();
    let built = build_document_frame_graph(
        &doc,
        EvaluationTime::new(t),
        frame_desc(),
        &tracks,
        &reference_runtime(),
        None,
    )
    .unwrap();
    let camera = built.camera;

    let CompCameraDoc::PlanarOrthographic {
        center,
        roll_radians,
        height,
    } = &doc.composition.camera;
    let center_v = eval_vec2(center, t, &tracks, &resolved).unwrap();
    let roll = eval_f64(roll_radians, t, &tracks, &resolved).unwrap();
    let h = eval_f64(height, t, &tracks, &resolved).unwrap();
    let expected = CompCamera::try_new(
        CanonicalPoint {
            x: center_v[0],
            y: center_v[1],
        },
        roll,
        h,
        doc.composition.aspect_num(),
        doc.composition.aspect_den(),
    )
    .unwrap();
    assert_eq!(camera, expected);

    let probe = CanonicalPoint { x: 0.3, y: -0.05 };
    let aspect_num = doc.composition.aspect_num();
    let aspect_den = doc.composition.aspect_den();
    let a = aspect_num as f64 / aspect_den as f64;
    let dx = probe.x - center_v[0];
    let dy = probe.y - center_v[1];
    let cos_r = roll.cos();
    let sin_r = roll.sin();
    // 決定§2: q = R(-r)(p - c)
    let qx = cos_r * dx + sin_r * dy;
    let qy = -sin_r * dx + cos_r * dy;
    let exp_ndc_x = 2.0 * qx / (h * a);
    let exp_ndc_y = 2.0 * qy / h;
    let (ndc_x, ndc_y) = camera.world_to_ndc(probe).unwrap();
    assert!(
        (ndc_x - exp_ndc_x).abs() < 1e-9 && (ndc_y - exp_ndc_y).abs() < 1e-9,
        "NDC: got ({ndc_x}, {ndc_y}), expected ({exp_ndc_x}, {exp_ndc_y})"
    );
    let exp_pixel_x = (exp_ndc_x + 1.0) * f64::from(W) / 2.0;
    let exp_pixel_y = (1.0 - exp_ndc_y) * f64::from(H) / 2.0;
    let pixel = camera.world_to_pixel(probe, &frame_desc()).unwrap();
    assert!(
        (pixel.x - exp_pixel_x).abs() < 1e-9 && (pixel.y - exp_pixel_y).abs() < 1e-9,
        "pixel: got ({}, {}), expected ({exp_pixel_x}, {exp_pixel_y})",
        pixel.x,
        pixel.y
    );
}

#[test]
fn document_frame_graph_carries_evaluated_camera() {
    let doc = Document::new_current();
    let eval = EvaluationTime::new(RationalTime::ZERO);
    let tracks = DataTracks::new();
    let expected = identity_camera(&doc);
    let built = build_document_frame_graph(
        &doc,
        eval,
        frame_desc(),
        &tracks,
        &reference_runtime(),
        None,
    )
    .unwrap();
    assert_eq!(built.camera, expected);
}

#[test]
fn tiny_non_default_center_never_skips_affine_place() {
    let doc = build_world_identity_rect_doc(CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([1e-13, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::const_f64(1.0),
    });
    let places = affine_place_steps(&build_graph(&doc));
    assert!(
        !places.is_empty(),
        "tiny non-default center must not be skipped by is_approx_identity"
    );
}

#[test]
fn non_default_camera_inverse_uv_matches_independent_oracle() {
    let center = [0.1, -0.05];
    let roll = 0.25;
    let height = 1.5;
    let doc = build_world_identity_rect_doc(CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2(center),
        roll_radians: DocParam::const_f64(roll),
        height: DocParam::const_f64(height),
    });
    let built = build_graph(&doc);
    let places = affine_place_steps(&built);
    assert_eq!(
        places.len(),
        1,
        "world identity + non-default camera must emit exactly one AffinePlace"
    );
    let got = places[0];
    let aspect = W as f64 / H as f64;
    let want = manual_inverse_uv_oracle(center, roll, height, aspect);
    assert_coefficients_near(got, want);

    // camera 無視（恒等 uv）を検出できること。
    assert_coefficients_differ(got, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

    // roll 符号反転 oracle は一致しないこと。
    let wrong_sign = manual_inverse_uv_oracle(center, -roll, height, aspect);
    assert_coefficients_differ(got, wrong_sign);

    // 二重 camera 逆写像 oracle は一致しないこと。
    let dst = [(0.12, 0.18), (0.73, 0.31), (0.41, 0.87)];
    let mut double_src = [(0.0, 0.0); 3];
    for (i, &(u, v)) in dst.iter().enumerate() {
        let (x1, y1) = uv_to_canonical(u, v, aspect);
        let (x2, y2) = camera_inverse_canonical(x1, y1, center, roll, height);
        let (x3, y3) = camera_inverse_canonical(x2, y2, center, roll, height);
        double_src[i] = canonical_to_uv(x3, y3, aspect);
    }
    let double_oracle = solve_uv_affine_from_three_points(&dst, &double_src);
    let double_oracle = [
        double_oracle[0] as f32,
        double_oracle[1] as f32,
        double_oracle[2] as f32,
        double_oracle[3] as f32,
        double_oracle[4] as f32,
        double_oracle[5] as f32,
    ];
    assert_coefficients_differ(got, double_oracle);
}

#[test]
fn invalid_height_rejects_without_graph() {
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::const_f64(0.0),
    };
    assert!(doc.validate().is_err());

    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc(),
        &DataTracks::new(),
        &reference_runtime(),
        None,
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            GraphError::CameraEval(CameraEvalError::Camera(
                CompCameraError::NonPositiveHeight { .. }
            ))
        ),
        "camera/build must fail before returning a graph: {err:?}"
    );
}

#[test]
fn non_finite_center_rejects_typed_camera_error() {
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([f64::NAN, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::const_f64(1.0),
    };
    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc(),
        &DataTracks::new(),
        &reference_runtime(),
        None,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        GraphError::CameraEval(CameraEvalError::Camera(
            CompCameraError::NonFiniteCenter { .. }
        ))
    ));
}

#[test]
fn camera_param_type_mismatch_is_typed() {
    use motolii_core::Fps;
    use motolii_eval::DataTrack;

    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Data {
            track: motolii_eval::DataTrackId("bad".into()),
            fallback: DocValue::F64(1.0),
        },
    };
    let mut tracks = DataTracks::new();
    tracks.insert(
        motolii_eval::DataTrackId("bad".into()),
        DataTrack {
            start: RationalTime::ZERO,
            sample_rate: Fps::try_new(1, 1).unwrap(),
            values: vec![Value::Vec2([1.0, 1.0])],
        },
    );
    let err = build_document_frame_graph(
        &doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc(),
        &tracks,
        &reference_runtime(),
        None,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        GraphError::CameraEval(CameraEvalError::Param(
            ParamEvalError::DataTrackTypeMismatch { .. }
        ))
    ));
}

#[test]
fn current_default_camera_document_gpu_matches_cam_g0_oracle() {
    let doc = build_cam_g0_document(CompCameraDoc::default_planar_orthographic());
    let actual = render_document_gpu(&doc).expect("gpu");
    let expected = load_cam_g0_oracle_bytes();
    assert_rgba_close(
        "d3f-current-default-cam-g0",
        cam_g0_image_desc(),
        &actual,
        &expected,
        tol::EXACT,
    );
}

#[test]
fn migrated_default_camera_document_gpu_matches_cam_g0_oracle() {
    let bytes = serde_json::to_vec(&json!({
        "version": 4,
        "min_reader_version": 1,
        "composition": {
            "aspect_num": CAM_G0_W,
            "aspect_den": CAM_G0_H,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1}
    }))
    .unwrap();
    let (mut doc, report) = migrate_bytes(&bytes).unwrap();
    assert!(
        report.steps.contains(&"insert_default_comp_camera"),
        "steps={:?}",
        report.steps
    );
    assert_eq!(
        doc.composition.camera,
        CompCameraDoc::default_planar_orthographic()
    );
    populate_cam_g0_scene(&mut doc);

    let actual = render_document_gpu(&doc).expect("gpu");
    let expected = load_cam_g0_oracle_bytes();
    assert_rgba_close(
        "d3f-migrated-default-cam-g0",
        cam_g0_image_desc(),
        &actual,
        &expected,
        tol::EXACT,
    );
}

#[test]
fn non_default_camera_document_gpu_does_not_match_cam_g0_oracle() {
    let doc = build_cam_g0_document(CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.1, -0.05]),
        roll_radians: DocParam::const_f64(0.25),
        height: DocParam::const_f64(1.5),
    });
    let actual = render_document_gpu(&doc).expect("gpu");
    let expected = load_cam_g0_oracle_bytes();
    let diff = compare_rgba(cam_g0_image_desc(), &actual, &expected).unwrap();
    assert!(
        diff.stats.max_abs_diff > tol::EXACT,
        "non-default camera must not match CAM-G0 oracle: max={}",
        diff.stats.max_abs_diff
    );
}
