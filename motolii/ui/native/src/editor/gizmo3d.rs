//! 3D レイヤーの 3 軸ギズモ。`transform-gizmo` に当たり判定と描画頂点を任せ、
//! こちらは Motolii のカメラ規約との差だけを埋めて、結果を Document の値へ戻す。
//! 2D の平面ケージ(`stage.rs` の `CageDrag`)とは経路を共有しない。
use crate::doc::store::*;
use transform_gizmo::math::Transform as GizmoTransform;
use transform_gizmo::{
    mint, Color32, Gizmo, GizmoConfig, GizmoInteraction, GizmoMode, GizmoOrientation, GizmoVisuals, Rect,
};

type PropertyEdit = (LayerId, PropertyId, Value);

/// 上流は近/遠面を [-1,1] に写す規約で ray を作る。Motolii の投影は
/// `perspective_infinite_reverse_rh` なので、**画面上の x/y は 1 対 1 のまま**
/// 深度規約だけを差し替えた行列を渡す。
/// (`perspective_rh_gl` は x/y/w の行が同じで z の行だけ違う。)
fn gizmo_projection(projection: &crate::doc::core::CameraProjection) -> glam::Mat4 {
    let near = projection.near_plane_distance.max(1e-4);
    glam::Mat4::perspective_rh_gl(
        projection.vertical_fov_radians,
        projection.aspect_ratio,
        near,
        near * 1.0e6,
    )
}

fn row_matrix(m: glam::Mat4) -> mint::RowMatrix4<f64> {
    let t = m.transpose();
    let row = |v: glam::Vec4| mint::Vector4 {
        x: f64::from(v.x),
        y: f64::from(v.y),
        z: f64::from(v.z),
        w: f64::from(v.w),
    };
    mint::RowMatrix4 { x: row(t.x_axis), y: row(t.y_axis), z: row(t.z_axis), w: row(t.w_axis) }
}

/// 掴む所と描く所は同じ枠で。Stage が送ってくる点は comp 座標なので、
/// viewport も comp の矩形にする。大きさと線の太さは画面の画素で決め、
/// `view_scale`(画面 px / comp px)で comp 座標へ戻す —— 寄っても引いても同じ手触り。
/// 色は意味だけを運ぶ: 白 = 休んでいる、黒 = hover。実際の色は Stage の theme が決める。
/// `modes` は AE の P / R / S を押している間の絞り込み。押していなければ移動+回転。
pub(crate) fn modes(held: Option<&str>) -> transform_gizmo::EnumSet<GizmoMode> {
    match held {
        Some("position") => GizmoMode::all_translate(),
        Some("rotation") => GizmoMode::all_rotate(),
        Some("scale") => GizmoMode::all_scale(),
        _ => GizmoMode::all_translate() | GizmoMode::all_rotate(),
    }
}

pub(crate) fn config(
    comp: crate::doc::core::CompSpec,
    camera: crate::doc::core::ResolvedCamera,
    view_scale: f64,
    held: Option<&str>,
) -> GizmoConfig {
    let projection = crate::doc::core::camera_projection(comp, camera);
    let px = 1.0 / view_scale.max(1e-3) as f32;
    GizmoConfig {
        view_matrix: row_matrix(projection.view_matrix()),
        projection_matrix: row_matrix(gizmo_projection(&projection)),
        viewport: Rect::from_min_max(
            transform_gizmo::math::Pos2::new(0.0, 0.0),
            transform_gizmo::math::Pos2::new(comp.width as f32, comp.height as f32),
        ),
        modes: modes(held),
        orientation: GizmoOrientation::Local,
        snap_distance: 10.0 * px,
        snap_scale: 0.1,
        visuals: GizmoVisuals {
            gizmo_size: 90.0 * px,
            stroke_width: 3.0 * px,
            x_color: Color32::WHITE,
            y_color: Color32::WHITE,
            z_color: Color32::WHITE,
            s_color: Color32::WHITE,
            highlight_color: Some(Color32::BLACK),
            inactive_alpha: 0.7,
            highlight_alpha: 1.0,
        },
        ..Default::default()
    }
}

/// 層の world から見た 3 軸の姿。旋回の中心は anchor —— 掴んだ所と回る所を合わせる。
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpatialTarget {
    pub(crate) layer: LayerId,
    /// anchor を原点に読み替えた親の world。`local` はこの中にいる。
    parent: glam::Affine3A,
    anchor: glam::Vec3,
    start: GizmoTransform,
}

/// world の姿 → Document の値。skew は 2D 専用なので 0 の層だけが通る。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LocalSpatial {
    pub(crate) position: [f64; 3],
    /// x, y, z の順(度)。`spatial_from_transform` の組み方に合わせる。
    pub(crate) rotation: [f64; 3],
    pub(crate) scale: [f64; 3],
}

/// `LayerPlacement::spatial_from_transform` が組む local を、値へ戻す。
/// 逆写像なので、組み直せば同じ行列になる(`round_trip` で確かめる)。
pub(crate) fn decompose(local: glam::Affine3A, anchor: [f64; 2]) -> Option<LocalSpatial> {
    let m = local.matrix3;
    let mut columns = [
        glam::Vec3::from(m.x_axis),
        glam::Vec3::from(m.y_axis),
        glam::Vec3::from(m.z_axis),
    ];
    let mut scale = columns.map(glam::Vec3::length);
    if scale.iter().any(|s| *s < 1e-6) {
        return None;
    }
    if glam::Mat3::from_cols(columns[0], columns[1], columns[2]).determinant() < 0.0 {
        scale[0] = -scale[0];
    }
    for axis in 0..3 {
        columns[axis] /= scale[axis];
    }
    let r = glam::Mat3::from_cols(columns[0], columns[1], columns[2]);
    // Rx(a)·Ry(b)·Rz(c) を成分から解く。列優先なので m.col(j)[i] が第 i 行 j 列。
    let at = |row: usize, col: usize| f64::from(r.col(col)[row]);
    let sin_b = at(0, 2).clamp(-1.0, 1.0);
    let b = sin_b.asin();
    let cos_b = b.cos();
    let (a, c) = if cos_b.abs() < 1e-6 {
        // 真横を向いた所では x と z の回りが同じ回転になる。z を 0 に決めて x へ寄せる。
        ((sin_b * at(1, 0)).atan2(at(1, 1)), 0.0)
    } else {
        ((-at(1, 2)).atan2(at(2, 2)), (-at(0, 1)).atan2(at(0, 0)))
    };
    let anchor3 = glam::vec3(anchor[0] as f32, anchor[1] as f32, 0.0);
    let position = local.transform_point3(anchor3);
    Some(LocalSpatial {
        position: [f64::from(position.x), f64::from(position.y), f64::from(position.z)],
        rotation: [a.to_degrees(), b.to_degrees(), c.to_degrees()],
        scale: scale.map(f64::from),
    })
}

/// `decompose` の相方。試験のためだけでなく、掴む前に往復を確かめるのにも使う。
pub(crate) fn compose(v: &LocalSpatial, anchor: [f64; 2]) -> glam::Affine3A {
    let anchor = [anchor[0] as f32, anchor[1] as f32];
    let position = [v.position[0] as f32, v.position[1] as f32];
    let planar = crate::doc::core::LayerPlacement::from_transform(
        anchor,
        position,
        [v.scale[0] as f32, v.scale[1] as f32],
        v.rotation[2] as f32,
        0.0,
        0.0,
    );
    crate::doc::core::LayerPlacement::spatial_from_transform(
        planar,
        position,
        v.position[2] as f32,
        v.rotation[0] as f32,
        v.rotation[1] as f32,
        v.scale[2] as f32,
    )
}

fn f64_at(view: &StoreView<'_>, layer: LayerId, name: &str, rt: RationalTime, default: f64) -> f64 {
    let Ok(prop) = PropertyId::new(name) else { return default };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::F64(v)) => v,
        _ => default,
    }
}

fn vec2_at(view: &StoreView<'_>, layer: LayerId, name: &str, rt: RationalTime, default: [f64; 2]) -> [f64; 2] {
    let Ok(prop) = PropertyId::new(name) else { return default };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::Vec2(v)) => v,
        _ => default,
    }
}

/// 親が相似変換(回転・一様拡縮・移動)でないと、親の逆写像は剪断を生む。
/// 剪断は Document の値に持てない —— 掴んでから飛ぶ前に、掴ませない。
fn is_similarity(m: glam::Affine3A) -> bool {
    let columns = [
        glam::Vec3::from(m.matrix3.x_axis),
        glam::Vec3::from(m.matrix3.y_axis),
        glam::Vec3::from(m.matrix3.z_axis),
    ];
    let lengths = columns.map(glam::Vec3::length);
    if lengths.iter().any(|l| !l.is_finite() || *l < 1e-6) {
        return false;
    }
    let tolerance = 1e-3 * lengths[0].max(1.0);
    (lengths[1] - lengths[0]).abs() <= tolerance
        && (lengths[2] - lengths[0]).abs() <= tolerance
        && columns[0].dot(columns[1]).abs() <= tolerance * lengths[0]
        && columns[0].dot(columns[2]).abs() <= tolerance * lengths[0]
        && columns[1].dot(columns[2]).abs() <= tolerance * lengths[0]
}

/// 選んだ層のうち、3 軸ギズモが立つ物だけ。2D・2.5D の層と、
/// 変換を持たない案内の層(Camera・Stage)はここに来ない。
pub(crate) fn spatial_targets(
    view: &StoreView<'_>,
    ids: &[LayerId],
    at: RationalTime,
) -> Result<Vec<SpatialTarget>, String> {
    let e = |x: StoreError| x.to_string();
    let mut out = Vec::new();
    for &layer in ids {
        let attrs = view.attrs(layer).map_err(e)?.unwrap_or_default();
        if attrs.projection != LayerProjection::ThreeD || attrs.locked {
            continue;
        }
        let Some(meta) = view.meta(layer).map_err(e)? else { continue };
        // Camera は Center/Orbit/Distance を、Stage は余白を author する。position を持たないので
        // 3 軸を立てると、掴んでも Document に何も届かない札が世界の別の場所に出る。
        if matches!(meta.source, LayerSource::Camera | LayerSource::Stage) {
            continue;
        }
        let Some(comp) = view.composition().map_err(e)? else { continue };
        let Ok(frame) = at.try_to_frame_floor(comp.fps) else { continue };
        if !meta.timing.covers(frame) {
            continue;
        }
        let parent = match attrs.parent {
            Some(id) => view.world_transform3d(id, at).map_err(e)?,
            None => glam::Affine3A::IDENTITY,
        };
        if !is_similarity(parent) {
            return Err("This layer's parent shears its children; the 3D gizmo cannot write it back".into());
        }
        if f64_at(view, layer, property::SKEW, at, 0.0).abs() > 1e-9 {
            return Err("Clear Skew before using the 3D gizmo".into());
        }
        if has_split_position(view, layer) {
            return Err("Rejoin the separate Position axes before using the 3D gizmo".into());
        }
        let anchor = vec2_at(view, layer, property::ANCHOR, at, [0.0, 0.0]);
        let local = view.local_transform3d(layer, at).map_err(e)?;
        // 値 → 行列 → 値 が閉じない層は、掴んだ瞬間に飛ぶ。先に断る。
        let Some(back) = decompose(local, anchor) else {
            return Err("This layer is flattened along an axis; the 3D gizmo cannot write it back".into());
        };
        if !close(&compose(&back, anchor), &local) {
            return Err("This layer's transform cannot be carried by the 3D gizmo".into());
        }
        let world = parent * local;
        let anchor3 = glam::vec3(anchor[0] as f32, anchor[1] as f32, 0.0);
        let (scale, rotation, _) = world.to_scale_rotation_translation();
        let translation = world.transform_point3(anchor3);
        out.push(SpatialTarget {
            layer,
            parent,
            anchor: anchor3,
            start: GizmoTransform {
                scale: mint::Vector3 { x: f64::from(scale.x), y: f64::from(scale.y), z: f64::from(scale.z) },
                rotation: mint::Quaternion {
                    v: mint::Vector3 { x: f64::from(rotation.x), y: f64::from(rotation.y), z: f64::from(rotation.z) },
                    s: f64::from(rotation.w),
                },
                translation: mint::Vector3 {
                    x: f64::from(translation.x),
                    y: f64::from(translation.y),
                    z: f64::from(translation.z),
                },
            },
        });
    }
    Ok(out)
}

fn has_split_position(view: &StoreView<'_>, layer: LayerId) -> bool {
    [property::POSITION_X, property::POSITION_Y].iter().any(|name| {
        PropertyId::new(name)
            .ok()
            .and_then(|p| view.property_source(layer, &p).ok().flatten())
            .is_some()
    })
}

fn close(a: &glam::Affine3A, b: &glam::Affine3A) -> bool {
    let tolerance = 1e-3 * b.translation.length().max(1.0);
    (0..3).all(|axis| {
        (glam::Vec3::from(a.matrix3.col(axis)) - glam::Vec3::from(b.matrix3.col(axis))).length() <= 1e-3
    }) && (glam::Vec3::from(a.translation) - glam::Vec3::from(b.translation)).length() <= tolerance
}

fn world_from(transform: &GizmoTransform, anchor: glam::Vec3) -> glam::Affine3A {
    let scale = glam::vec3(transform.scale.x as f32, transform.scale.y as f32, transform.scale.z as f32);
    let rotation = glam::Quat::from_xyzw(
        transform.rotation.v.x as f32,
        transform.rotation.v.y as f32,
        transform.rotation.v.z as f32,
        transform.rotation.s as f32,
    )
    .normalize();
    let linear = glam::Mat3::from_quat(rotation) * glam::Mat3::from_diagonal(scale);
    let origin = glam::vec3(
        transform.translation.x as f32,
        transform.translation.y as f32,
        transform.translation.z as f32,
    );
    glam::Affine3A::from_mat3_translation(linear, origin - linear * anchor)
}

/// 描くだけ(掴んでいない時)。頂点は comp 座標なので、Stage はそのまま画面へ写せる。
/// `pointer` が乗っている部品(hover)は黒で返る。
pub(crate) fn draw_data(
    comp: crate::doc::core::CompSpec,
    camera: crate::doc::core::ResolvedCamera,
    targets: &[SpatialTarget],
    pointer: Option<[f64; 2]>,
    view_scale: f64,
    held: Option<&str>,
) -> Option<transform_gizmo::GizmoDrawData> {
    if targets.is_empty() {
        return None;
    }
    let mut gizmo = Gizmo::new(config(comp, camera, view_scale, held));
    let starts: Vec<_> = targets.iter().map(|t| t.start).collect();
    let cursor_pos = pointer.map_or((f32::NAN, f32::NAN), |p| (p[0] as f32, p[1] as f32));
    let _ = gizmo.update(
        GizmoInteraction { cursor_pos, hovered: pointer.is_some(), drag_started: false, dragging: false },
        &starts,
    );
    Some(gizmo.draw())
}

/// 3 軸ギズモを掴んでいる間。Undo は離した時の 1 回だけ —— ここは下書きしか作らない。
pub(crate) struct SpatialDrag {
    inner: std::cell::RefCell<Live>,
    targets: Vec<SpatialTarget>,
    original_values: Vec<PropertyEdit>,
    revision: Revision,
    at: RationalTime,
}

struct Live {
    gizmo: Gizmo,
    /// 上流は 1 フレームごとの差分を返す。前の結果を戻してやらないと動きが積まれない。
    current: Vec<GizmoTransform>,
}

impl SpatialDrag {
    pub(crate) fn begin(
        doc: &Document,
        ids: &[LayerId],
        start: [f64; 2],
        at: RationalTime,
        observer: crate::doc::core::ResolvedCamera,
        view_scale: f64,
        held: Option<&str>,
    ) -> Result<Self, String> {
        let e = |x: StoreError| x.to_string();
        let view = doc.view().without_transients();
        let comp = view.composition().map_err(e)?.ok_or("No composition")?.spec();
        let targets = spatial_targets(&view, ids, at)?;
        if targets.is_empty() {
            return Err("Select a 3D layer".into());
        }
        for target in &targets {
            if let Some(reason) = crate::editor::functions::lens::edit_rejection(&view, target.layer).map_err(e)? {
                return Err(reason.into());
            }
        }
        let mut original_values = Vec::new();
        for target in &targets {
            for (name, value) in transform_values(&view, target.layer, at) {
                original_values.push((target.layer, PropertyId::new(name).map_err(e)?, value));
            }
        }
        let mut gizmo = Gizmo::new(config(comp, observer, view_scale, held));
        let current: Vec<_> = targets.iter().map(|t| t.start).collect();
        let interaction = GizmoInteraction {
            cursor_pos: (start[0] as f32, start[1] as f32),
            hovered: true,
            drag_started: true,
            dragging: true,
        };
        // 上流は掴んだ frame で「掴んだ点(矢の線分の上)」と「軸の直線の上の点」の差を
        // そのまま動きとして返す。矢の端を掴むとそこで層が跳ぶので、掴んだ frame の
        // 結果は捨てる —— 以後の差分はこの点から測られるので、動きは繋がったまま。
        if gizmo.update(interaction, &current).is_none() {
            return Err("The 3D gizmo was not grabbed".into());
        }
        Ok(Self {
            inner: std::cell::RefCell::new(Live { gizmo, current }),
            targets,
            original_values,
            revision: doc.revision(),
            at,
        })
    }

    pub(crate) fn edits(
        &self,
        doc: &Document,
        point: [f64; 2],
        shift: bool,
        animate: Animate,
    ) -> Result<Vec<Intent>, String> {
        if doc.revision() != self.revision {
            return Err("Gesture canceled because document changed".into());
        }
        let mut live = self.inner.borrow_mut();
        let mut config = *live.gizmo.config();
        config.snapping = shift;
        live.gizmo.update_config(config);
        let current = live.current.clone();
        let interaction = GizmoInteraction {
            cursor_pos: (point[0] as f32, point[1] as f32),
            hovered: true,
            drag_started: false,
            dragging: true,
        };
        if let Some((_, updated)) = live.gizmo.update(interaction, &current) {
            live.current = updated;
        }
        let mut out = Vec::new();
        for (target, moved) in self.targets.iter().zip(live.current.iter()) {
            let world = world_from(moved, target.anchor);
            let local = target.parent.inverse() * world;
            if !local.is_finite() {
                return Err("The 3D gizmo cannot reach that transform".into());
            }
            let anchor = [f64::from(target.anchor.x), f64::from(target.anchor.y)];
            let Some(values) = decompose(local, anchor) else {
                return Err("The 3D gizmo would flatten this layer".into());
            };
            for (name, value) in [
                (property::POSITION, Value::Vec2([values.position[0], values.position[1]])),
                (property::POSITION_Z, Value::F64(values.position[2])),
                (property::ROTATION_X, Value::F64(values.rotation[0])),
                (property::ROTATION_Y, Value::F64(values.rotation[1])),
                (property::ROTATION, Value::F64(values.rotation[2])),
                (property::SCALE, Value::Vec2([values.scale[0], values.scale[1]])),
                (property::SCALE_Z, Value::F64(values.scale[2])),
            ] {
                let property = PropertyId::new(name).map_err(|x| x.to_string())?;
                // 触っていない値は書かない。Animate の最中に、動かしていない軸へキーを置かない。
                if self.original_values.iter().any(|(layer, original_property, original)| {
                    *layer == target.layer && *original_property == property && same(original, &value)
                }) {
                    continue;
                }
                if let Some(edit) = doc
                    .place_checked(target.layer, &property, value, self.at, animate)
                    .map_err(|x| x.to_string())?
                {
                    out.push(edit);
                }
            }
        }
        Ok(out)
    }
}

/// world は f32 の行列を通って戻ってくる。f32 の粒より細かい違いは「違い」ではない ——
/// そこで丸めないと、掴んで離しただけで値が書かれ、Animate ではキーまで置かれる。
fn same(a: &Value, b: &Value) -> bool {
    let unchanged = |x: f64, y: f64| (x - y).abs() <= 1e-5 * x.abs().max(1.0);
    match (a, b) {
        (Value::F64(x), Value::F64(y)) => unchanged(*x, *y),
        (Value::Vec2(x), Value::Vec2(y)) => (0..2).all(|i| unchanged(x[i], y[i])),
        _ => a == b,
    }
}

fn transform_values(view: &StoreView<'_>, layer: LayerId, at: RationalTime) -> Vec<(&'static str, Value)> {
    let position = vec2_at(view, layer, property::POSITION, at, [0.0, 0.0]);
    let scale = vec2_at(view, layer, property::SCALE, at, [1.0, 1.0]);
    vec![
        (property::POSITION, Value::Vec2(position)),
        (property::POSITION_Z, Value::F64(f64_at(view, layer, property::POSITION_Z, at, 0.0))),
        (property::ROTATION_X, Value::F64(f64_at(view, layer, property::ROTATION_X, at, 0.0))),
        (property::ROTATION_Y, Value::F64(f64_at(view, layer, property::ROTATION_Y, at, 0.0))),
        (property::ROTATION, Value::F64(f64_at(view, layer, property::ROTATION, at, 0.0))),
        (property::SCALE, Value::Vec2(scale)),
        (property::SCALE_Z, Value::F64(f64_at(view, layer, property::SCALE_Z, at, 1.0))),
    ]
}

#[cfg(test)]
mod spatial_gizmo_tests {
    use super::*;

    fn comp() -> crate::doc::core::CompSpec {
        crate::doc::core::CompSpec { width: 1920, height: 1080 }
    }

    /// 案内の層(Camera・Stage)には 3 軸が立たない。position を持たないので、
    /// 立てると掴んでも何も届かない札が世界の別の場所に出る。
    #[test]
    fn guide_layers_get_no_three_axis_handle() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = RationalTime::ZERO;
        let mut make = |id: u64, source: LayerSource| {
            let layer = LayerId(id);
            doc.apply_all(crate::editor::create::new_layer_intents(
                layer, id as i16, 0, 60, fps, (1920.0, 1080.0),
                match source {
                    LayerSource::Camera => crate::editor::create::NewKind::Camera,
                    LayerSource::Stage => crate::editor::create::NewKind::Stage,
                    _ => crate::editor::create::NewKind::Rectangle,
                },
                None,
            )).unwrap();
            doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::ThreeD), ..Default::default() } }).unwrap();
            layer
        };
        let camera = make(1, LayerSource::Camera);
        let stage = make(2, LayerSource::Stage);
        let shape = make(3, LayerSource::Shape);
        let view = doc.view();
        assert!(spatial_targets(&view, &[camera, stage], at).unwrap().is_empty(), "a camera or a stage carries no transform");
        let targets = spatial_targets(&view, &[camera, stage, shape], at).unwrap();
        assert_eq!(targets.iter().map(|t| t.layer).collect::<Vec<_>>(), vec![shape], "the drawn layer still gets its handle");
    }

    /// Motolii の投影と、ギズモへ渡す深度規約だけ差し替えた投影が、
    /// 同じ world 点を同じ画面座標へ写す。ここがずれると掴んだ所と絵がずれる。
    #[test]
    fn substituted_projection_keeps_the_screen_position() {
        for camera in [
            crate::doc::core::ResolvedCamera::default(),
            crate::doc::core::ResolvedCamera { orbit_degrees: [-22.0, 41.0], distance_scale: 2.5, zoom: 1.7, ..Default::default() },
        ] {
            let projection = crate::doc::core::camera_projection(comp(), camera);
            let motolii = projection.projection_matrix() * projection.view_matrix();
            let gizmo = gizmo_projection(&projection) * projection.view_matrix();
            for point in [
                glam::vec3(0.0, 0.0, 0.0),
                glam::vec3(1920.0, 1080.0, 0.0),
                glam::vec3(640.0, 300.0, -420.0),
                glam::vec3(-200.0, 900.0, 380.0),
            ] {
                let screen = |m: glam::Mat4| {
                    let c = m * point.extend(1.0);
                    glam::vec2((c.x / c.w + 1.0) * 0.5 * 1920.0, (1.0 - c.y / c.w) * 0.5 * 1080.0)
                };
                let (a, b) = (screen(motolii), screen(gizmo));
                assert!((a - b).length() < 1e-2, "{point:?}: {a:?} != {b:?}");
            }
        }
    }

    /// 値 → 行列 → 値 が閉じる。閉じないと、掴んだ瞬間に層が飛ぶ。
    #[test]
    fn values_survive_a_round_trip_through_the_matrix() {
        for anchor in [[0.0, 0.0], [64.0, -30.0]] {
            for values in [
                LocalSpatial { position: [500.0, 400.0, 50.0], rotation: [0.0, 0.0, 0.0], scale: [1.0, 1.0, 1.0] },
                LocalSpatial { position: [-120.0, 900.0, -260.0], rotation: [31.0, -47.0, 118.0], scale: [1.4, 0.6, 2.2] },
                LocalSpatial { position: [10.0, 20.0, 30.0], rotation: [-80.0, 12.0, -170.0], scale: [0.2, 3.0, 0.9] },
            ] {
                let local = compose(&values, anchor);
                let back = decompose(local, anchor).expect("decomposable");
                assert!(close(&compose(&back, anchor), &local), "{values:?} -> {back:?}");
                for axis in 0..3 {
                    assert!((back.position[axis] - values.position[axis]).abs() < 1e-2, "position {axis}: {back:?}");
                    assert!((back.scale[axis] - values.scale[axis]).abs() < 1e-3, "scale {axis}: {back:?}");
                }
            }
        }
    }

    fn document() -> (Document, LayerId) {
        let mut doc = crate::doc::store::blank_project();
        let layer = LayerId(41);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 60) },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { projection: Some(LayerProjection::ThreeD), ..Default::default() },
            },
        ])
        .unwrap();
        for (name, value) in [
            (property::POSITION, Value::Vec2([960.0, 540.0])),
            (property::POSITION_Z, Value::F64(0.0)),
            (property::ROTATION_X, Value::F64(12.0)),
            (property::ROTATION_Y, Value::F64(-24.0)),
            (property::ROTATION, Value::F64(8.0)),
            (property::SCALE, Value::Vec2([1.0, 1.0])),
            (property::SCALE_Z, Value::F64(1.0)),
        ] {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        }
        (doc, layer)
    }

    /// 掴んだだけで動かさなければ、1 画素も動かない。
    /// (掴んだ瞬間に飛ぶ = 見えているカメラと当たり判定が食い違っている印。)
    #[test]
    fn grabbing_without_moving_writes_nothing() {
        let (doc, _) = document();
        let mut grabbed = 0;
        for camera in [
            crate::doc::core::ResolvedCamera::default(),
            crate::doc::core::ResolvedCamera { orbit_degrees: [-18.0, 35.0], distance_scale: 1.6, ..Default::default() },
        ] {
            for x in (700..1250).step_by(25) {
                for y in (300..800).step_by(25) {
                    let start = [f64::from(x), f64::from(y)];
                    let Ok(drag) = SpatialDrag::begin(&doc, &[LayerId(41)], start, RationalTime::ZERO, camera, 0.5, None) else {
                        continue;
                    };
                    grabbed += 1;
                    let held = drag.edits(&doc, start, false, Animate::Off).expect("holding still is legal");
                    assert!(held.is_empty(), "掴んだだけで {held:?} が出た({start:?})");
                }
            }
        }
        assert!(grabbed > 20, "3 軸ギズモを掴める所が {grabbed} しかない");
    }

    /// 掴んで動かすと値が変わり、飛ばない。書くのは動かした軸だけ。
    #[test]
    fn dragging_moves_the_layer_without_flinging_it() {
        let (doc, layer) = document();
        let camera = crate::doc::core::ResolvedCamera { orbit_degrees: [-18.0, 35.0], distance_scale: 1.6, ..Default::default() };
        let mut moved = 0;
        for x in (700..1250).step_by(25) {
            for y in (300..800).step_by(25) {
                let start = [f64::from(x), f64::from(y)];
                let Ok(drag) = SpatialDrag::begin(&doc, &[layer], start, RationalTime::ZERO, camera, 0.5, None) else {
                    continue;
                };
                let Ok(out) = drag.edits(&doc, [start[0] + 30.0, start[1]], false, Animate::Off) else { continue };
                if out.is_empty() {
                    continue;
                }
                moved += 1;
                for intent in &out {
                    let Intent::SetConstant { layer: id, value, .. } = intent else { continue };
                    assert_eq!(*id, layer);
                    let numbers: Vec<f64> = match value {
                        Value::F64(v) => vec![*v],
                        Value::Vec2(v) => v.to_vec(),
                        other => panic!("3 軸ギズモが {other:?} を書いた"),
                    };
                    for n in numbers {
                        assert!(n.is_finite() && n.abs() < 1.0e5, "{start:?} で {n} まで飛んだ: {intent:?}");
                    }
                }
            }
        }
        assert!(moved > 20, "動かせた所が {moved} しかない");
    }

    /// `scale.z` は Document の値。1.0 以外を入れると local の z 列が伸びる。
    #[test]
    fn scale_z_reaches_the_local_transform() {
        let flat = compose(
            &LocalSpatial { position: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0], scale: [1.0, 1.0, 1.0] },
            [0.0, 0.0],
        );
        let deep = compose(
            &LocalSpatial { position: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0], scale: [1.0, 1.0, 3.0] },
            [0.0, 0.0],
        );
        assert!((glam::Vec3::from(flat.matrix3.z_axis).length() - 1.0).abs() < 1e-4);
        assert!((glam::Vec3::from(deep.matrix3.z_axis).length() - 3.0).abs() < 1e-4);
    }
}
