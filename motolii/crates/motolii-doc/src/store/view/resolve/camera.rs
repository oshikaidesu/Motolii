//! 観測 — その時刻に効いている Stage と Camera を選び、姿勢を解く。
//! どちらも「区間内・可視・solo が優先・最上位」の同じ規則で 1 枚だけ選ぶ。
//! 層を並べ替えも書き換えもしない。見る側の話で、描く中身には触れない。

use super::*;

/// Camera と同じ規則で active な非描画層を選ぶ: 区間内・可視・solo が優先・最上位。
pub(crate) fn active_guide(view: &StoreView<'_>, source: crate::doc::store::LayerSource, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
    let frame = view.composition()?.map(|c| t.try_to_frame_floor(c.fps)).transpose()
        .map_err(|e| StoreError::Property(e.to_string()))?.unwrap_or(0);
    let mut guides = Vec::new();
    for id in view.layers() {
        if let Some(meta) = view.meta(id)? {
            if meta.source == source && meta.timing.covers(frame) {
                let attrs = view.attrs(id)?.unwrap_or_default();
                if !crate::doc::store::view::resolve::resolved_hidden(view, id, t, attrs.hidden)? { guides.push((crate::doc::store::view::resolve::resolved_solo(view, id, t, attrs.solo)?, meta.order, id)); }
            }
        }
    }
    guides.sort();
    Ok(guides.last().map(|(_, _, id)| *id))
}

pub fn resolve_stage_extent(view: &StoreView<'_>, t: RationalTime) -> Result<crate::doc::store::StageExtent, StoreError> {
    let Some(id) = active_guide(view, crate::doc::store::LayerSource::Stage, t)? else { return Ok(Default::default()) };
    let mut margins = [0.0; 4];
    for (margin, name) in margins.iter_mut().zip(property::STAGE_MARGINS) {
        if let Some(Value::F64(v)) = view.value_at(id, &PropertyId::new(name)?, t)? { *margin = v.max(0.0) as f32; }
    }
    Ok(crate::doc::store::StageExtent { layer: Some(id), margins })
}

/// Camera 層 `id` が時刻 `t` に見ている姿勢。層ターゲットが在れば、その層の局所原点の world 点を注視点にする(描画側は bounds の中心で上書きする)。
pub fn camera_of_layer(view: &StoreView<'_>, id: LayerId, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
    let get = |name| view.value_at(id, &PropertyId::new(name)?, t);
    let vec2 = |v: Option<Value>, d: [f32; 2]| match v { Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32], _ => d };
    let f = |v: Option<Value>, d: f32| match v { Some(Value::F64(v)) if v.is_finite() => v as f32, _ => d };
    let mut camera = crate::doc::core::ResolvedCamera {
        center: vec2(get(property::CAMERA_CENTER)?, [0.0, 0.0]),
        target_z: f(get(property::CAMERA_TARGET_Z)?, 0.0),
        orbit_degrees: vec2(get(property::CAMERA_ORBIT)?, [0.0, 0.0]),
        distance_scale: f(get(property::CAMERA_DISTANCE)?, 1.0).max(0.01),
        zoom: f(get(property::CAMERA_ZOOM)?, 1.0),
        roll_degrees: f(get(property::CAMERA_ROLL)?, 0.0),
        near_fade: f(get(property::CAMERA_NEAR_FADE)?, 0.0).max(0.0),
    };
    if let Some(framed) = framed_camera(view, id, t, camera)? {
        return Ok(framed);
    }
    if let Some(target) = camera_target_layer(view, id, t)? {
        if let Some(comp) = view.composition()? {
            let comp = comp.spec();
            let present = view.layers().into_iter().collect();
            if let Some(world) = crate::doc::store::view::resolve::transform::world_transform3d_chain(view, target, t, &present)?.get(&target) {
                let point = world.transform_point3(glam::Vec3::ZERO);
                camera.center = [point.x - comp.width as f32 * 0.5, point.y - comp.height as f32 * 0.5];
                camera.target_z = point.z;
            }
        }
    }
    Ok(camera)
}

/// Framing Size が 0 より大きく Target があれば、箱を画面に収めたカメラ。Camera 層の Transition があれば、少し前の時刻の
/// 収め方(注視点・奥行き・Distance の対数)を区間の重みで混ぜる(Target を替えると箱から箱へ滑る)。
pub(crate) fn framed_camera(view: &StoreView<'_>, id: LayerId, t: RationalTime, authored: crate::doc::core::ResolvedCamera) -> Result<Option<crate::doc::core::ResolvedCamera>, StoreError> {
    let framing = match view.value_at(id, &PropertyId::new(property::CAMERA_FRAMING)?, t)? {
        Some(Value::F64(v)) if v > 0.0 => v as f32,
        _ => return Ok(None),
    };
    let Some(now) = frame_of(view, id, t, framing, authored)? else { return Ok(None) };
    let samples = view.transition_samples(id, t)?;
    if samples.is_empty() {
        return Ok(Some(now));
    }
    let (mut center, mut z, mut log_distance, mut total) = (glam::Vec2::ZERO, 0.0f32, 0.0f32, 0.0f32);
    for (at, w) in samples {
        let Some(past) = frame_of(view, id, at, framing, authored)? else { continue };
        center += glam::Vec2::from(past.center) * w;
        z += past.target_z * w;
        log_distance += past.distance_scale.ln() * w;
        total += w;
    }
    if total <= 1e-6 {
        return Ok(Some(now));
    }
    Ok(Some(crate::doc::core::ResolvedCamera { center: (center / total).to_array(), target_z: z / total, distance_scale: (log_distance / total).exp(), ..now }))
}

/// その時刻の Target の箱(世界、軸に沿った箱)を画面の `framing` の割合に収めるカメラ。
pub(crate) fn frame_of(view: &StoreView<'_>, id: LayerId, t: RationalTime, framing: f32, authored: crate::doc::core::ResolvedCamera) -> Result<Option<crate::doc::core::ResolvedCamera>, StoreError> {
    let Some(target) = camera_target_layer(view, id, t)? else { return Ok(None) };
    let Some(comp) = view.composition()? else { return Ok(None) };
    let present = view.layers().into_iter().collect();
    let Some(world) = crate::doc::store::view::resolve::transform::world_transform3d_chain(view, target, t, &present)?.get(&target).copied() else { return Ok(None) };
    let Some(b) = crate::doc::store::layout::boxes::layer_box(view, target, t)? else { return Ok(None) };
    let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| world.transform_point3(glam::vec3(c[0], c[1], 0.0)));
    let lo = corners.iter().fold(glam::Vec3::MAX, |a, p| a.min(*p));
    let hi = corners.iter().fold(glam::Vec3::MIN, |a, p| a.max(*p));
    let (w, h) = ((hi.x - lo.x).max(1.0), (hi.y - lo.y).max(1.0));
    let middle = (lo + hi) * 0.5;
    // 注視点の面で、画面の倍率 = Zoom / Distance。箱が割合 framing に収まる倍率へ Distance を解く。
    let magnify = (framing * comp.width as f32 / w).min(framing * comp.height as f32 / h);
    Ok(Some(crate::doc::core::ResolvedCamera {
        center: [middle.x - comp.width as f32 * 0.5, middle.y - comp.height as f32 * 0.5],
        target_z: middle.z,
        distance_scale: (authored.zoom.max(1e-3) / magnify.max(1e-3)).clamp(0.01, 100.0),
        ..authored
    }))
}

/// `camera.target` が指す、いま在る別の層。0・消えた層・自分自身は無し。
pub fn camera_target_layer(view: &StoreView<'_>, id: LayerId, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
    Ok(match view.value_at(id, &PropertyId::new(property::CAMERA_TARGET)?, t)? {
        Some(Value::LayerId(raw)) if raw != 0 && raw != id.0 && view.layers().contains(&LayerId(raw)) => Some(LayerId(raw)),
        _ => None,
    })
}

/// 時刻 `t` に効いている Camera 層。無ければ comp 常在のカメラ track が効いている。
pub fn active_camera_layer(view: &StoreView<'_>, t: RationalTime) -> Result<Option<LayerId>, StoreError> {
    active_guide(view, crate::doc::store::LayerSource::Camera, t)
}

pub fn resolve_camera(view: &StoreView<'_>, t: RationalTime) -> Result<crate::doc::core::ResolvedCamera, StoreError> {
    if let Some(id) = active_camera_layer(view, t)? {
        return camera_of_layer(view, id, t);
    }
    let center_property = PropertyId::camera(property::CAMERA_CENTER)?;
    let center = match view.camera_value_at(&center_property, t)? {
        Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
        Some(other) => {
            return Err(StoreError::Property(format!(
                "{} に2成分でない値が入っている: {other:?}",
                property::CAMERA_CENTER
            )))
        }
        None => [0.0, 0.0],
    };

    let zoom_property = PropertyId::camera(property::CAMERA_ZOOM)?;
    let zoom = match view.camera_value_at(&zoom_property, t)? {
        Some(Value::F64(v)) => v as f32,
        Some(other) => {
            return Err(StoreError::Property(format!(
                "{} に数値でない値が入っている: {other:?}",
                property::CAMERA_ZOOM
            )))
        }
        None => 1.0,
    };

    let roll_property = PropertyId::camera(property::CAMERA_ROLL)?;
    let roll_degrees = match view.camera_value_at(&roll_property, t)? {
        Some(Value::F64(v)) => v as f32,
        Some(other) => {
            return Err(StoreError::Property(format!(
                "{} に数値でない値が入っている: {other:?}",
                property::CAMERA_ROLL
            )))
        }
        None => 0.0,
    };

    Ok(crate::doc::core::ResolvedCamera {
        center,
        zoom,
        roll_degrees, ..Default::default() })
}

#[cfg(test)]
mod stage_extent_contract {
    use crate::doc::store::*;

    /// Boxcam の working comp / AE の guide layer: 区間内の最上位が効き、区間の外では出力枠に戻る。
    #[test]
    fn the_topmost_stage_layer_in_range_widens_the_frame_and_nothing_else_does() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |frame| RationalTime::try_from_frame(frame, fps).unwrap();
        for (id, order, start, margins) in [(1u64, 0i16, 0i64, [100.0, 0.0, 100.0, 0.0]), (2, 5, 10, [0.0, 400.0, 0.0, 400.0])] {
            let layer = LayerId(id);
            let mut intents = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Stage, order, timing: LayerTiming::place(start, None, 10) } },
            ];
            for (name, value) in property::STAGE_MARGINS.iter().zip(margins) {
                intents.push(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value: Value::F64(value) });
            }
            doc.apply_all(intents).unwrap();
        }
        let view = doc.view();
        assert_eq!(crate::doc::store::view::resolve::camera::resolve_stage_extent(&view, at(3)).unwrap().rect(comp), [-100.0, 0.0, comp.width as f32 + 200.0, comp.height as f32]);
        assert_eq!(crate::doc::store::view::resolve::camera::resolve_stage_extent(&view, at(12)).unwrap().rect(comp), [0.0, -400.0, comp.width as f32, comp.height as f32 + 800.0]);
        assert_eq!(crate::doc::store::view::resolve::camera::resolve_stage_extent(&view, at(25)).unwrap(), StageExtent::default());
        assert_eq!(crate::doc::store::view::resolve::camera::resolve_camera(&view, at(3)).unwrap(), crate::doc::core::ResolvedCamera::default(), "a stage layer is not a camera");
    }
}

#[cfg(test)]
mod camera_target_contract {
    use crate::doc::core::{camera_projection, ResolvedCamera};
    use crate::doc::store::*;

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }
    fn add(doc: &mut Document, id: u64, source: LayerSource) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all(vec![
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 100) } },
        ]).unwrap();
        layer
    }
    /// world 点が出力の枠中央に来るか(注視点は常に中央)。
    fn centred(comp: crate::doc::core::CompSpec, camera: ResolvedCamera, point: glam::Vec3) -> bool {
        let projection = camera_projection(comp, camera);
        let clip = projection.projection_matrix() * projection.view_matrix() * point.extend(1.0);
        clip.w > 0.0 && (clip.x / clip.w).abs() < 1e-3 && (clip.y / clip.w).abs() < 1e-3
    }

    /// Framing Size: Target の箱の中心を注視点にし、箱が画面のその割合に収まる距離へ(Cinemachine の Group Framing Size)。
    #[test]
    fn framing_size_fits_the_target_box_on_screen_and_follows_it() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let card = add(&mut doc, 2, LayerSource::Shape);
        doc.apply(Intent::SetShapes { layer: card, shapes: vec![rect_shape([255; 4], [200.0, 100.0])] }).unwrap();
        put(&mut doc, card, property::POSITION, Value::Vec2([300.0, 200.0]));
        let camera = add(&mut doc, 1, LayerSource::Camera);
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(card.0));
        put(&mut doc, camera, property::CAMERA_FRAMING, Value::F64(0.5));
        let on_screen = |doc: &Document| {
            let view = doc.view();
            let resolved = crate::doc::store::view::resolve::camera::resolve_camera(&view, RationalTime::ZERO).unwrap();
            let projection = camera_projection(comp, resolved);
            let matrix = projection.projection_matrix() * projection.view_matrix();
            let world = crate::doc::store::view::resolve::transform::world_transform3d(&view, card, RationalTime::ZERO).unwrap();
            let b = crate::doc::store::layout::boxes::layer_box(&view, card, RationalTime::ZERO).unwrap().unwrap();
            let ndc: Vec<glam::Vec2> = [[b[0], b[1]], [b[2], b[3]]].iter().map(|c| {
                let clip = matrix * world.transform_point3(glam::vec3(c[0], c[1], 0.0)).extend(1.0);
                glam::vec2(clip.x / clip.w, clip.y / clip.w)
            }).collect();
            (((ndc[1].x - ndc[0].x).abs() * 0.5), ((ndc[1].y - ndc[0].y).abs() * 0.5), (ndc[0] + ndc[1]) * 0.5)
        };
        let (w, h, middle) = on_screen(&doc);
        assert!(middle.length() < 1e-3, "the box's centre is the centre of the frame: {middle:?}");
        assert!(((w.max(h)) - 0.5).abs() < 0.01 && w.max(h) >= w.min(h), "the tighter side takes half the frame: {w} {h}");
        put(&mut doc, card, property::POSITION, Value::Vec2([900.0, 700.0]));
        put(&mut doc, card, property::SCALE, Value::Vec2([2.0, 2.0]));
        let (w2, h2, middle2) = on_screen(&doc);
        assert!(middle2.length() < 1e-3 && ((w2.max(h2)) - 0.5).abs() < 0.01, "moving and growing the box keeps it framed: {w2} {h2} {middle2:?}");
    }

    /// AE の Point of Interest を rerun の球面座標で持つ: 注視点・軌道・距離が Camera 層から解決へ流れ、eye は導出。
    #[test]
    fn orbit_distance_and_target_z_flow_from_the_camera_layer() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let camera = add(&mut doc, 1, LayerSource::Camera);
        put(&mut doc, camera, property::CAMERA_CENTER, Value::Vec2([120.0, -40.0]));
        put(&mut doc, camera, property::CAMERA_TARGET_Z, Value::F64(300.0));
        put(&mut doc, camera, property::CAMERA_ORBIT, Value::Vec2([-20.0, 35.0]));
        put(&mut doc, camera, property::CAMERA_DISTANCE, Value::F64(2.0));
        let resolved = crate::doc::store::view::resolve::camera::resolve_camera(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(resolved, ResolvedCamera { center: [120.0, -40.0], target_z: 300.0, orbit_degrees: [-20.0, 35.0], distance_scale: 2.0, ..Default::default() });
        let target = resolved.target(comp);
        assert!(centred(comp, resolved, target), "the point of interest sits under the frame centre");
        let eye = camera_projection(comp, resolved).eye;
        let front = camera_projection(comp, ResolvedCamera { orbit_degrees: [0.0; 2], ..resolved }).eye;
        assert!((eye.distance(target) - front.distance(target)).abs() < 0.01, "orbit keeps the distance");
        assert!(eye.distance(front) > 1.0, "orbit moves the eye");
    }

    /// 層ターゲット: null を動かせば注視点が追う。無い層・0・自分自身は無視して center に戻る。
    #[test]
    fn a_target_layer_moves_the_point_of_interest_with_its_position() {
        let mut doc = blank_project();
        let comp = doc.view().composition().unwrap().unwrap().spec();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |frame| RationalTime::try_from_frame(frame, fps).unwrap();
        let camera = add(&mut doc, 1, LayerSource::Camera);
        let null = add(&mut doc, 2, LayerSource::Null);
        put(&mut doc, camera, property::CAMERA_CENTER, Value::Vec2([500.0, 500.0]));
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(2));
        put(&mut doc, null, property::POSITION_Z, Value::F64(250.0));
        let mut track = KeyframeTrack::new();
        for (frame, xy) in [(0, [100.0, 200.0]), (10, [300.0, 400.0])] {
            track.insert(Keyframe { t: at(frame), value: Value::Vec2(xy), interp: Interp::Linear, spatial: None });
        }
        doc.apply(Intent::SetTrack { layer: null, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        for (frame, expect) in [(0, glam::vec3(100.0, 200.0, 250.0)), (5, glam::vec3(200.0, 300.0, 250.0)), (10, glam::vec3(300.0, 400.0, 250.0))] {
            let resolved = crate::doc::store::view::resolve::camera::resolve_camera(&doc.view(), at(frame)).unwrap();
            assert!(resolved.target(comp).distance(expect) < 1e-3, "frame {frame}: {:?} != {expect:?}", resolved.target(comp));
            assert!(centred(comp, resolved, expect));
        }
        // parent を挟んでも world の位置を見る
        let parent = add(&mut doc, 3, LayerSource::Null);
        put(&mut doc, parent, property::POSITION, Value::Vec2([1000.0, 0.0]));
        doc.apply(Intent::SetAttrs { layer: null, patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() } }).unwrap();
        assert!(crate::doc::store::view::resolve::camera::resolve_camera(&doc.view(), at(0)).unwrap().target(comp).distance(glam::vec3(1100.0, 200.0, 250.0)) < 1e-3);
        for dead in [Value::LayerId(0), Value::LayerId(1), Value::LayerId(99)] {
            put(&mut doc, camera, property::CAMERA_TARGET, dead.clone());
            assert_eq!(crate::doc::store::view::resolve::camera::resolve_camera(&doc.view(), at(0)).unwrap().center, [500.0, 500.0], "{dead:?} falls back to center");
        }
        put(&mut doc, camera, property::CAMERA_TARGET, Value::LayerId(2));
        doc.apply(Intent::RemoveLayer(null)).unwrap();
        assert_eq!(crate::doc::store::view::resolve::camera::resolve_camera(&doc.view(), at(0)).unwrap().center, [500.0, 500.0], "a removed target falls back to center");
    }
}
