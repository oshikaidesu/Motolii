use crate::doc::store::*;
use crate::render::engine::Engine;
type PropertyEdit=(LayerId,PropertyId,Value);
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum GizmoMode {
    Move,
    ScaleCorner {
        sx: bool,
        sy: bool,
    },
    ScaleEdge {
        axis_x: bool,
        positive: bool,
    },
    Rotate,
    /// 3D: 横で rotation.y、縦で rotation.x を回す。
    Orbit {
        axis_x: bool,
    },
    /// 3D: 縦で position.z を動かす。
    Depth,
}
struct GizmoDrag {
    owner: u64,
    layer: LayerId,
    mode: GizmoMode,
    grab: (f64, f64),
    orig_position: (f64, f64),
    orig_rotation: f64,
    orig_rotation_xy: (f64, f64),
    orig_z: f64,
    anchor: (f64, f64),
    natural: (f64, f64),
    local_bounds: crate::render::media::SpatialBounds,
    orig_box: (f64, f64, f64, f64),
    /// 掴んだ時の奥行き。掴んでいる間、写像はこの面に固定する
    /// (奥行きを動かしている最中に写像まで動くと、指と絵が食い違う)。
    fit_z: f64,
    projection: LayerProjection,
    orig_placement: crate::doc::core::LayerPlacement,
    /// 最後に**動かした**先。動かしていなければ None。
    /// 離した時の座標から差分を取り直すと、掴んでいる間に要素の座標系がずれた分だけ
    /// 勝手に動く。動かしていないなら1画素も動かさない。
    last: Option<(f64, f64)>,
    /// 最後に見せた値。**離した瞬間の見た目がそのまま確定値** —— 離す直前に
    /// Shift を放しても、見えていた物と違う値は書かない。
    preview: Vec<PropertyEdit>,
    original_values: Vec<PropertyEdit>,
    at: RationalTime,
    /// 一緒に選んでいる他の層と、掴んだ時の位置。動かす時は同じ差分で運ぶ。
    others: Vec<(LayerId, SelGeom)>,
}
#[derive(Clone,Debug)]
struct SelGeom {
    projection: LayerProjection,
    placement: crate::doc::core::LayerPlacement,
    z: f64,
    rotation_x: f64,
    rotation_y: f64,
    position: (f64, f64),
    anchor: (f64, f64),
    rotation: f64,
    natural: (f64, f64),
    local_bounds: crate::render::media::SpatialBounds,
    box_: (f64, f64, f64, f64),
}
struct Fit {comp:crate::doc::core::CompSpec,camera:crate::doc::core::ResolvedCamera,projection_camera:crate::doc::core::ResolvedCamera,fx:f64,fy:f64,s:f64}
struct PlaneMap {
    screen_from_uv: glam::DMat3,
    uv_from_screen: glam::DMat3,
}
impl PlaneMap {
    fn to_screen(&self, u: f64, v: f64) -> (f64, f64) {
        apply_h(&self.screen_from_uv, u, v)
    }

    fn to_uv(&self, sx: f64, sy: f64) -> (f64, f64) {
        apply_h(&self.uv_from_screen, sx, sy)
    }
}
fn vec2_at(
    view: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    rt: RationalTime,
    default: (f64, f64),
) -> (f64, f64) {
    let Ok(prop) = PropertyId::new(name) else {
        return default;
    };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => default,
    }
}
fn f64_at(view: &StoreView<'_>, layer: LayerId, name: &str, rt: RationalTime, default: f64) -> f64 {
    let Ok(prop) = PropertyId::new(name) else {
        return default;
    };
    match view.value_at(layer, &prop, rt).ok().flatten() {
        Some(Value::F64(v)) => v,
        _ => default,
    }
}
fn selection_geom_in(
    engine: &Engine,
    view: &StoreView<'_>,
    layer: LayerId,
    rt: RationalTime,
) -> Option<SelGeom> {
    let resolved = view.resolved_layers(rt).ok()?;
    selection_geom_resolved(engine, view, &resolved, layer, rt)
}
fn selection_geom_resolved(
    engine: &Engine,
    view: &StoreView<'_>,
    resolved: &[crate::doc::store::ResolvedLayer],
    layer: LayerId,
    rt: RationalTime,
) -> Option<SelGeom> {
    let meta = view.meta(layer).ok().flatten()?;
    let fps = view.composition().ok().flatten()?.fps;
    let frame = rt.try_to_frame_floor(fps).ok()?;
    if !meta.timing.covers(frame) {
        return None;
    }
    let z = f64_at(view, layer, property::POSITION_Z, rt, 0.0);
    let rotation_x = f64_at(view, layer, property::ROTATION_X, rt, 0.0);
    let rotation_y = f64_at(view, layer, property::ROTATION_Y, rt, 0.0);
    let position = vec2_at(view, layer, property::POSITION, rt, (0.0, 0.0));
    let anchor = vec2_at(view, layer, property::ANCHOR, rt, (0.0, 0.0));
    let scale = vec2_at(view, layer, property::SCALE, rt, (1.0, 1.0));
    let rotation = f64_at(view, layer, property::ROTATION, rt, 0.0);
    let local_bounds = engine.selected_layer_bounds_in(view, resolved, layer, rt)?;
    let [w0, h0, _] = local_bounds.size();
    let natural = (w0 as f64, h0 as f64);
    let box_ = (
        position.0 + scale.0 * (f64::from(local_bounds.min[0]) - anchor.0),
        position.1 + scale.1 * (f64::from(local_bounds.min[1]) - anchor.1),
        scale.0 * natural.0,
        scale.1 * natural.1,
    );
    let evaluated = resolved.iter().find(|l| l.id == layer && !l.ghost)?;
    Some(SelGeom {
        projection: evaluated.projection,
        placement: evaluated.placement,
        z,
        rotation_x,
        rotation_y,
        position,
        anchor,
        rotation,
        natural,
        local_bounds,
        box_,
    })
}
fn compute_scale(
    orig_box: (f64, f64, f64, f64),
    natural: (f64, f64),
    anchor: (f64, f64),
    mode: GizmoMode,
    cur: (f64, f64),
    shift: bool,
    alt: bool,
) -> ((f64, f64), (f64, f64)) {
    let (bx, by, bw, bh) = orig_box;
    let (x0, y0, x1, y1) = (bx, by, bx + bw, by + bh);
    let (cx, cy) = cur;
    let (mut nx0, mut ny0, mut nx1, mut ny1) = (x0, y0, x1, y1);
    match mode {
        GizmoMode::ScaleCorner { sx, sy } => {
            if sx {
                nx1 = cx
            } else {
                nx0 = cx
            }
            if sy {
                ny1 = cy
            } else {
                ny0 = cy
            }
            if alt {
                let (ccx, ccy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
                let (hx, hy) = ((cx - ccx).abs(), (cy - ccy).abs());
                (nx0, nx1) = (ccx - hx, ccx + hx);
                (ny0, ny1) = (ccy - hy, ccy + hy);
            }
            if shift {
                let (fx, fy) = if alt {
                    ((x0 + x1) * 0.5, (y0 + y1) * 0.5)
                } else {
                    (if sx { x0 } else { x1 }, if sy { y0 } else { y1 })
                };
                let orig_w = (x1 - x0).abs().max(1e-6);
                let orig_h = (y1 - y0).abs().max(1e-6);
                let w = (nx1 - nx0).abs();
                let h = (ny1 - ny0).abs();
                let k = (w / orig_w).max(h / orig_h);
                let (new_w, new_h) = (orig_w * k, orig_h * k);
                if alt {
                    (nx0, nx1) = (fx - new_w * 0.5, fx + new_w * 0.5);
                    (ny0, ny1) = (fy - new_h * 0.5, fy + new_h * 0.5);
                } else {
                    if sx {
                        nx1 = fx + new_w
                    } else {
                        nx0 = fx - new_w
                    }
                    if sy {
                        ny1 = fy + new_h
                    } else {
                        ny0 = fy - new_h
                    }
                }
            }
        }
        GizmoMode::ScaleEdge { axis_x, positive } => {
            if axis_x {
                if positive {
                    nx1 = cx
                } else {
                    nx0 = cx
                }
                if alt {
                    let ccx = (x0 + x1) * 0.5;
                    let hx = (cx - ccx).abs();
                    (nx0, nx1) = (ccx - hx, ccx + hx);
                }
            } else {
                if positive {
                    ny1 = cy
                } else {
                    ny0 = cy
                }
                if alt {
                    let ccy = (y0 + y1) * 0.5;
                    let hy = (cy - ccy).abs();
                    (ny0, ny1) = (ccy - hy, ccy + hy);
                }
            }
        }
        GizmoMode::Move | GizmoMode::Rotate | GizmoMode::Orbit { .. } | GizmoMode::Depth => {}
    }
    let (nbx, nby) = (nx0.min(nx1), ny0.min(ny1));
    let (nbw, nbh) = ((nx1 - nx0).abs().max(0.01), (ny1 - ny0).abs().max(0.01));
    let scale = (nbw / natural.0.max(0.01), nbh / natural.1.max(0.01));
    let local_anchor = crate::editor::functions::atom::scale_about(
        [anchor.0, anchor.1],
        [0.0, 0.0],
        [scale.0, scale.1],
    );
    let shifted = crate::editor::functions::atom::translate2([nbx, nby], local_anchor);
    let position = (shifted[0], shifted[1]);
    (scale, position)
}
fn preview_values(
    drag: &GizmoDrag,
    cur: (f64, f64),
    shift: bool,
    alt: bool,
    scale: f64,
) -> Vec<PropertyEdit> {
    let (cx, cy) = cur;
    let mut out = Vec::new();
    match drag.mode {
        GizmoMode::Move => {
            let (mut dx, mut dy) = (cx - drag.grab.0, cy - drag.grab.1);
            if shift {
                if dx.abs() >= dy.abs() {
                    dy = 0.0;
                } else {
                    dx = 0.0;
                }
            }
            let p = drag.orig_position;
            out.push((
                drag.layer,
                property::POSITION,
                Value::Vec2([p.0 + dx, p.1 + dy]),
            ));
        }
        GizmoMode::ScaleCorner { .. } | GizmoMode::ScaleEdge { .. } => {
            let (new_scale, new_pos) = compute_scale(
                drag.orig_box,
                drag.natural,
                (drag.anchor.0 - f64::from(drag.local_bounds.min[0]),
                 drag.anchor.1 - f64::from(drag.local_bounds.min[1])),
                drag.mode,
                cur,
                shift,
                alt,
            );
            out.push((
                drag.layer,
                property::SCALE,
                Value::Vec2([new_scale.0, new_scale.1]),
            ));
            out.push((
                drag.layer,
                property::POSITION,
                Value::Vec2([new_pos.0, new_pos.1]),
            ));
        }
        GizmoMode::Rotate => {
            let r = compute_rotation(
                drag.orig_position,
                drag.grab,
                cur,
                drag.orig_rotation,
                shift,
            );
            out.push((drag.layer, property::ROTATION, Value::F64(r)));
        }
        GizmoMode::Orbit { axis_x } => {
            let (rx, ry) = orbit_axis(drag.orig_rotation_xy, drag.grab, cur, axis_x, scale);
            if axis_x {
                out.push((drag.layer, property::ROTATION_X, Value::F64(rx)));
            } else {
                out.push((drag.layer, property::ROTATION_Y, Value::F64(ry)));
            }
        }
        GizmoMode::Depth => {
            out.push((
                drag.layer,
                property::POSITION_Z,
                Value::F64(drag.orig_z + (drag.grab.1 - cy)),
            ));
        }
    }
    let primary = out.clone();
    for (layer, geom) in &drag.others {
        for (_, name, value) in &primary {
            let lifted = match (*name, value) {
                (property::POSITION, Value::Vec2(v)) => {
                    let mut dx = v[0] - drag.orig_position.0;
                    let mut dy = v[1] - drag.orig_position.1;
                    if matches!(
                        drag.mode,
                        GizmoMode::ScaleCorner { .. } | GizmoMode::ScaleEdge { .. }
                    ) {
                        if drag.orig_box.2.abs() > 1e-9 {
                            dx *= geom.box_.2 / drag.orig_box.2;
                        }
                        if drag.orig_box.3.abs() > 1e-9 {
                            dy *= geom.box_.3 / drag.orig_box.3;
                        }
                    }
                    Value::Vec2([geom.position.0 + dx, geom.position.1 + dy])
                }
                (property::SCALE, Value::Vec2(v)) => {
                    let original = [
                        drag.orig_box.2 / drag.natural.0,
                        drag.orig_box.3 / drag.natural.1,
                    ];
                    let own = [geom.box_.2 / geom.natural.0, geom.box_.3 / geom.natural.1];
                    Value::Vec2(std::array::from_fn(|axis| {
                        if original[axis].abs() > 1e-9 {
                            own[axis] * v[axis] / original[axis]
                        } else {
                            own[axis] + v[axis] - original[axis]
                        }
                    }))
                }
                (property::ROTATION, Value::F64(v)) => {
                    Value::F64(geom.rotation + v - drag.orig_rotation)
                }
                (property::ROTATION_X, Value::F64(v)) => {
                    Value::F64(geom.rotation_x + v - drag.orig_rotation_xy.0)
                }
                (property::ROTATION_Y, Value::F64(v)) => {
                    Value::F64(geom.rotation_y + v - drag.orig_rotation_xy.1)
                }
                (property::POSITION_Z, Value::F64(v)) => Value::F64(geom.z + v - drag.orig_z),
                _ => continue,
            };
            out.push((*layer, *name, lifted));
        }
    }
    out.into_iter()
        .map(|(layer, name, value)| {
            (
                layer,
                PropertyId::new(name).expect("transform property"),
                value,
            )
        })
        .filter(|(layer, property, value)| {
            !drag
                .original_values
                .iter()
                .any(|(original_layer, original_property, original)| {
                    layer == original_layer && property == original_property && value == original
                })
        })
        .collect()
}
fn rotate_around(center: (f64, f64), angle_deg: f64, p: (f64, f64)) -> (f64, f64) {
    let v = crate::editor::functions::atom::rotate_about(
        [p.0, p.1],
        [center.0, center.1],
        angle_deg.to_radians(),
    );
    (v[0], v[1])
}
fn compute_rotation(
    center: (f64, f64),
    grab: (f64, f64),
    cur: (f64, f64),
    orig_rotation: f64,
    shift: bool,
) -> f64 {
    let ang0 = (grab.1 - center.1).atan2(grab.0 - center.0);
    let ang1 = (cur.1 - center.1).atan2(cur.0 - center.0);
    let mut r = orig_rotation + (ang1 - ang0).to_degrees();
    if shift {
        r = (r / 15.0).round() * 15.0;
    }
    r
}
fn orbit_axis(
    orig: (f64, f64),
    grab: (f64, f64),
    now: (f64, f64),
    axis_x: bool,
    scale: f64,
) -> (f64, f64) {
    let (rx, ry) = orbit_angles(orig, grab, now, scale);
    if axis_x {
        (rx, orig.1)
    } else {
        (orig.0, ry)
    }
}
fn homography_from_unit_square(p: [glam::DVec2; 4]) -> glam::DMat3 {
    let (x0, y0) = (p[0].x, p[0].y);
    let (x1, y1) = (p[1].x, p[1].y);
    let (x2, y2) = (p[2].x, p[2].y);
    let (x3, y3) = (p[3].x, p[3].y);
    let sx = x0 - x1 + x2 - x3;
    let sy = y0 - y1 + y2 - y3;

    let (g, h) = if sx.abs() < 1e-9 && sy.abs() < 1e-9 {
        (0.0, 0.0)
    } else {
        let (dx1, dx2) = (x1 - x2, x3 - x2);
        let (dy1, dy2) = (y1 - y2, y3 - y2);
        let den = dx1 * dy2 - dx2 * dy1;
        if den.abs() < 1e-12 {
            (0.0, 0.0)
        } else {
            ((sx * dy2 - dx2 * sy) / den, (dx1 * sy - sx * dy1) / den)
        }
    };

    glam::DMat3::from_cols(
        glam::dvec3(x1 - x0 + g * x1, y1 - y0 + g * y1, g),
        glam::dvec3(x3 - x0 + h * x3, y3 - y0 + h * y3, h),
        glam::dvec3(x0, y0, 1.0),
    )
}
fn apply_h(m: &glam::DMat3, x: f64, y: f64) -> (f64, f64) {
    let q = *m * glam::dvec3(x, y, 1.0);
    if q.z.abs() < 1e-12 {
        return (f64::NAN, f64::NAN);
    }
    (q.x / q.z, q.y / q.z)
}
fn bounds_world_points(fit: &Fit, geom: &SelGeom) -> [glam::Vec3; 8] {
    let bounds = geom.local_bounds;
    if let Some(world) = geom.placement.world_transform {
        let center = world.transform_point3(glam::Vec3::from(bounds.center()));
        let correction = crate::doc::core::layer_projection_transform(
            fit.comp, fit.projection_camera, geom.projection, center,
        );
        return std::array::from_fn(|index| {
            let local = glam::Vec3::from_array(std::array::from_fn(|axis| {
                if index & (1 << axis) == 0 { bounds.min[axis] } else { bounds.max[axis] }
            }));
            correction.transform_point3(world.transform_point3(local))
        });
    }
    let (corner, u, v) = crate::render::compositor::projected_placement_corners(
        fit.comp, fit.projection_camera, geom.projection, geom.placement,
        glam::vec2(bounds.min[0], bounds.min[1]),
        glam::vec2(bounds.max[0] - bounds.min[0], bounds.max[1] - bounds.min[1]),
    );
    std::array::from_fn(|index| corner + u * (index & 1) as f32 + v * ((index >> 1) & 1) as f32)
}
fn project_world_point(fit: &Fit, point: glam::Vec3) -> glam::DVec2 {
    let projection = crate::doc::core::camera_projection(fit.comp, fit.camera);
    let clip = projection.projection_matrix() * projection.view_matrix() * point.extend(1.0);
    let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
    glam::dvec2(
        fit.fx + f64::from((clip.x / w + 1.0) * 0.5 * fit.comp.width as f32) * fit.s,
        fit.fy + f64::from((1.0 - clip.y / w) * 0.5 * fit.comp.height as f32) * fit.s,
    )
}
fn plane_map(fit: &Fit, geom: &SelGeom) -> PlaneMap {
    let points = bounds_world_points(fit, geom);
    let screen_from_uv = homography_from_unit_square([0, 1, 3, 2].map(|index| {
        project_world_point(fit, (points[index] + points[index + 4]) * 0.5)
    }));
    PlaneMap {
        uv_from_screen: screen_from_uv.inverse(),
        screen_from_uv,
    }
}

const MOVE_MAP_SPAN: f64 = 256.0;

fn translation_map(fit: &Fit, geom: &SelGeom, parent: glam::Affine3A, start: [f64; 2]) -> PlaneMap {
    let surface = plane_map(fit, geom);
    let (u, v) = surface.to_uv(start[0], start[1]);
    let screen_from_uv = homography_from_unit_square([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(x, y)| {
        let delta = parent.transform_vector3(glam::vec3(x, y, 0.0) * MOVE_MAP_SPAN as f32);
        let mut moved = geom.clone();
        if let Some(world)=geom.placement.world_transform { moved.placement.world_transform=Some(glam::Affine3A::from_translation(delta)*world); }
        else { moved.placement.transform.translation += delta.truncate(); }
        let p=plane_map(fit,&moved).to_screen(u,v);
        glam::dvec2(p.0,p.1)
    }));
    PlaneMap { uv_from_screen: screen_from_uv.inverse(), screen_from_uv }
}

/// Stage の掴み。**平面ケージ(2D・2.5D)と 3 軸ギズモ(3D)は別の物**で、
/// 始まりの `mode` で分かれたきり、以後は交わらない。
pub(crate) enum DragSession {
    Cage(CageDrag),
    Spatial(crate::editor::gizmo3d::SpatialDrag),
}
impl DragSession {
    pub(crate) fn begin(doc:&Document,engine:&Engine,ids:&[LayerId],mode:&str,handle:&str,start:[f64;2],at:RationalTime,observer:crate::doc::core::ResolvedCamera,view_scale:f64)->Result<Self,String>{
        if mode=="spatial" {
            return Ok(Self::Spatial(crate::editor::gizmo3d::SpatialDrag::begin(doc,ids,start,at,observer,view_scale)?));
        }
        Ok(Self::Cage(CageDrag::begin(doc,engine,ids,mode,handle,start,at,observer)?))
    }
    pub(crate) fn edits(&self,doc:&Document,point:[f64;2],shift:bool,alt:bool,animate:Animate)->Result<Vec<Intent>,String>{
        match self {
            Self::Cage(drag)=>drag.edits(doc,point,shift,alt,animate),
            Self::Spatial(drag)=>drag.edits(doc,point,shift,animate),
        }
    }
}

pub(crate) struct CageDrag {drag:GizmoDrag,map:PlaneMap,revision:Revision,start:[f64;2],moves:Vec<(LayerId,(f64,f64),PlaneMap)>}
impl CageDrag {
    pub(crate) fn begin(doc:&Document,engine:&Engine,ids:&[LayerId],mode:&str,handle:&str,start:[f64;2],at:RationalTime,observer:crate::doc::core::ResolvedCamera)->Result<Self,String>{
        let layer=*ids.last().ok_or("Select a layer")?;
        let view=doc.view().without_transients();
        if let Some(reason)=crate::editor::functions::lens::edit_rejection(&view,layer).map_err(|e|e.to_string())?{return Err(reason.into())}
        let geom=selection_geom_in(engine,&view,layer,at).ok_or("Selected layer bounds are unavailable")?;
        let mode=match mode {"move"=>GizmoMode::Move,"rotate"=>GizmoMode::Rotate,"scale"=>match handle {
            "nw"=>GizmoMode::ScaleCorner{sx:false,sy:false},"ne"=>GizmoMode::ScaleCorner{sx:true,sy:false},"sw"=>GizmoMode::ScaleCorner{sx:false,sy:true},"se"=>GizmoMode::ScaleCorner{sx:true,sy:true},
            "n"=>GizmoMode::ScaleEdge{axis_x:false,positive:false},"s"=>GizmoMode::ScaleEdge{axis_x:false,positive:true},"w"=>GizmoMode::ScaleEdge{axis_x:true,positive:false},"e"=>GizmoMode::ScaleEdge{axis_x:true,positive:true},_=>return Err("Unknown scale handle".into())},_=>return Err("Unsupported stage mode".into())};
        let fit=Fit {comp:view.composition().map_err(|e|e.to_string())?.ok_or("No composition")?.spec(),camera:observer,projection_camera:engine.resolve_camera(&view,at).map_err(|e|e.to_string())?,fx:0.0,fy:0.0,s:1.0};
        let map=plane_map(&fit,&geom);let(u,v)=map.to_uv(start[0],start[1]);
        if !u.is_finite()||!v.is_finite(){return Err("Selected plane is edge-on".into())}
        let(bx,by,bw,bh)=geom.box_;let grab=rotate_around(geom.position,geom.rotation,(bx+u*bw,by+v*bh));
        let mut others=Vec::new();for &id in ids{if id!=layer&&crate::editor::functions::lens::edit_rejection(&view,id).map_err(|e|e.to_string())?.is_none(){if let Some(g)=selection_geom_in(engine,&view,id,at){others.push((id,g));}}}
        let mut original_values=Vec::new();for(id,g)in std::iter::once((layer,&geom)).chain(others.iter().map(|(id,g)|(*id,g))){for(name,value)in[(property::POSITION,Value::Vec2([g.position.0,g.position.1])),(property::SCALE,Value::Vec2([g.box_.2/g.natural.0,g.box_.3/g.natural.1])),(property::ROTATION,Value::F64(g.rotation)),(property::ROTATION_X,Value::F64(g.rotation_x)),(property::ROTATION_Y,Value::F64(g.rotation_y)),(property::POSITION_Z,Value::F64(g.z))]{original_values.push((id,PropertyId::new(name).map_err(|e|e.to_string())?,value));}}
        let drag=GizmoDrag{owner:0,layer,mode,grab,orig_position:geom.position,orig_rotation:geom.rotation,orig_rotation_xy:(geom.rotation_x,geom.rotation_y),orig_z:geom.z,anchor:geom.anchor,natural:geom.natural,local_bounds:geom.local_bounds,orig_box:geom.box_,fit_z:geom.z,projection:geom.projection,orig_placement:geom.placement,last:None,preview:Vec::new(),original_values,at,others};
        let mut moves = Vec::new();
        if mode == GizmoMode::Move {
            for (id, g) in std::iter::once((layer, &geom)).chain(drag.others.iter().map(|(id, g)| (*id, g))) {
                let parent = match view.attrs(id).map_err(|e| e.to_string())?.and_then(|a| a.parent) {
                    Some(id) => view.world_transform3d(id, at).map_err(|e| e.to_string())?,
                    None => glam::Affine3A::IDENTITY,
                };
                let mapping = translation_map(&fit, g, parent, start);
                if !mapping.uv_from_screen.is_finite() { return Err("Position axes are edge-on".into()); }
                moves.push((id, g.position, mapping));
            }
        }
        Ok(Self{drag,map,revision:doc.revision(),start,moves})
    }
    pub(crate) fn edits(&self,doc:&Document,point:[f64;2],shift:bool,alt:bool,animate:Animate)->Result<Vec<Intent>,String>{
        if doc.revision()!=self.revision{return Err("Gesture canceled because document changed".into())}
        if self.drag.mode == GizmoMode::Move {
            let mut point = point;
            if shift {
                if (point[0] - self.start[0]).abs() >= (point[1] - self.start[1]).abs() { point[1] = self.start[1]; }
                else { point[0] = self.start[0]; }
            }
            let mut out = Vec::new();
            for (layer, position, map) in &self.moves {
                let (x0, y0) = map.to_uv(self.start[0], self.start[1]);
                let (x, y) = map.to_uv(point[0], point[1]);
                if !x.is_finite() || !y.is_finite() { return Err("Position axes cannot reach pointer".into()); }
                let value = Value::Vec2([position.0 + (x-x0)*MOVE_MAP_SPAN, position.1 + (y-y0)*MOVE_MAP_SPAN]);
                if let Some(edit) = doc.place_checked(*layer, &PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, value, self.drag.at, animate).map_err(|e| e.to_string())? { out.push(edit); }
            }
            return Ok(out);
        }
        let(u,v)=self.map.to_uv(point[0],point[1]);if !u.is_finite()||!v.is_finite(){return Err("Point cannot project to selected plane".into())}
        let(bx,by,bw,bh)=self.drag.orig_box;let p=rotate_around(self.drag.orig_position,self.drag.orig_rotation,(bx+u*bw,by+v*bh));
        let values=preview_values(&self.drag,p,shift,alt,1.0);let mut out=Vec::new();for(layer,property,value)in values{if let Some(edit)=doc.place_checked(layer,&property,value,self.drag.at,animate).map_err(|e|e.to_string())?{out.push(edit)}}Ok(out)
    }
}
fn orbit_angles(orig: (f64, f64), grab: (f64, f64), now: (f64, f64), scale: f64) -> (f64, f64) {
    let k = ORBIT_DEGREES_PER_PIXEL * scale;
    (orig.0 - (now.1 - grab.1) * k, orig.1 + (now.0 - grab.0) * k)
}

const ORBIT_DEGREES_PER_PIXEL: f64 = 0.5;

#[cfg(test)]
mod drag_projection_tests {
    use super::*;

    #[test]
    fn translating_tilted_layers_keeps_grabbed_point_under_pointer() {
        for observer in [crate::doc::core::ResolvedCamera::default(), crate::doc::core::ResolvedCamera { orbit_degrees:[-15.0,30.0],distance_scale:2.5,..Default::default() }] {
        let fit = Fit { comp: crate::doc::core::CompSpec { width: 1920, height: 1080 }, camera: observer, projection_camera: Default::default(), fx: 0.0, fy: 0.0, s: 1.0 };
        for projection in [LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::ThreeD] {
            for angle in [40.0_f32, 120.0] {
                for parent in [glam::Affine3A::IDENTITY, glam::Affine3A::from_scale_rotation_translation(glam::vec3(-1.2, 0.8, 1.0), glam::Quat::from_rotation_z(0.3), glam::Vec3::ZERO)] {
                    let world = glam::Affine3A::from_translation(glam::vec3(500.0, 400.0, 50.0)) * glam::Affine3A::from_quat(glam::Quat::from_rotation_y(angle.to_radians()));
                    let geom = SelGeom { projection, placement: crate::doc::core::LayerPlacement { world_transform: Some(world), ..Default::default() }, z:50.0, rotation_x:0.0, rotation_y:angle as f64, position:(500.0,400.0), anchor:(0.0,0.0), rotation:0.0, natural:(272.0,272.0), local_bounds:crate::render::media::SpatialBounds { min:[0.0;3], max:[272.0,272.0,0.0] }, box_:(500.0,400.0,272.0,272.0) };
                    let start = plane_map(&fit, &geom).to_screen(0.3, 0.6);
                    let mapping = translation_map(&fit, &geom, parent, [start.0, start.1]);
                    for delta in [(750.0, 120.0), (-300.0, -150.0), (0.0,0.0)] {
                        let target = (start.0 + delta.0, start.1 + delta.1);
                        let (x,y) = mapping.to_uv(target.0,target.1);
                        let (x0,y0) = mapping.to_uv(start.0,start.1);
                        let shift = parent.transform_vector3(glam::vec3(((x-x0)*MOVE_MAP_SPAN) as f32, ((y-y0)*MOVE_MAP_SPAN) as f32, 0.0));
                        let mut moved = geom.clone();
                        moved.placement.world_transform = Some(glam::Affine3A::from_translation(shift)*world);
                        let actual = plane_map(&fit,&moved).to_screen(0.3,0.6);
                        assert!((actual.0-target.0).abs()<0.02 && (actual.1-target.1).abs()<0.02, "{projection:?} {angle}: {actual:?} != {target:?}");
                    }
                }
            }
        }
        }
    }
}
