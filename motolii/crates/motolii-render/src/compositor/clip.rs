//! 層に載せる切断(Clip)。平面は**世界**の物: 層の中心から層の軸の向きに `offset` px。
//! z=0 の板も、点群も、網も、fork の同じ式(`clip_outside`)で切れる — 3D は 2D の上位互換。
use re_renderer::ClipPlane;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipSpec {
    /// 層の枠での向き(±X / ±Y / ±Z の単位ベクトル)。
    pub axis: glam::Vec3,
    /// 層の中心からこの向きへ何 px の所で切るか。
    pub offset: f32,
    pub cap: bool,
}

impl ClipSpec {
    /// 世界の平面へ。`frame` は層の枠(向きと大きさ)、`center` は層の中心(世界)。
    pub fn world(&self, center: glam::Vec3, frame: glam::Affine3A) -> ClipPlane {
        let normal = frame.transform_vector3(self.axis).normalize_or_zero();
        if normal == glam::Vec3::ZERO {
            return ClipPlane::NONE;
        }
        ClipPlane { normal, distance: normal.dot(center) + self.offset, cap: self.cap }
    }

    /// 板の枠: 左上の角と 2 辺から。中心は角 + (u + v) / 2。
    pub fn world_for_rect(&self, corner: glam::Vec3, u: glam::Vec3, v: glam::Vec3) -> ClipPlane {
        let w = u.cross(v).normalize_or_zero();
        let frame = glam::Affine3A::from_cols(u.normalize_or_zero().into(), v.normalize_or_zero().into(), w.into(), corner.into());
        self.world(corner + (u + v) * 0.5, frame)
    }
}

#[cfg(test)]
mod tests {
    use super::ClipSpec;

    #[test]
    fn the_plane_sits_offset_px_from_the_centre_along_the_layer_axis() {
        let spec = ClipSpec { axis: glam::Vec3::X, offset: 10.0, cap: true };
        let plane = spec.world_for_rect(glam::vec3(100.0, 100.0, 0.0), glam::vec3(200.0, 0.0, 0.0), glam::vec3(0.0, 50.0, 0.0));
        assert!((plane.normal - glam::Vec3::X).length() < 1e-6);
        // centre x = 200, +10 px → keep x <= 210
        assert!((plane.distance - 210.0).abs() < 1e-4, "{}", plane.distance);
        let scaled = ClipSpec { axis: glam::Vec3::Y, offset: -5.0, cap: false }
            .world(glam::Vec3::ZERO, glam::Affine3A::from_scale(glam::vec3(3.0, 7.0, 1.0)));
        assert!((scaled.normal - glam::Vec3::Y).length() < 1e-6 && (scaled.distance + 5.0).abs() < 1e-6);
    }
}

/// 板・点群・網が同じ平面で切れる(実 GPU)。z=0 の板は特別扱いしない。
#[cfg(test)]
mod contract {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const SIZE: u32 = 64;

    fn document(path: &std::path::Path, scale: Option<f64>, axis: f64) -> Document {
        let mut doc = Document::new();
        // 背景は不透明の黒: 透明背景だと点群の一部画素が出ない(既存の挙動)ので、色で判定する。
        doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([16.0, 16.0]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.clip".into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "axis").unwrap(), value: Value::F64(axis) },
        ]).unwrap();
        if let Some(scale) = scale {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale, scale]) }).unwrap();
        }
        doc
    }

    /// 黒の上に何か描かれているか(色成分の最大)。
    fn drawn(frame: &[u8], x: u32, y: u32) -> u8 {
        let i = ((y * SIZE + x) * 4) as usize;
        frame[i..i + 3].iter().copied().max().unwrap()
    }

    #[test]
    fn a_flat_image_a_point_cloud_and_a_mesh_are_cut_by_the_same_plane() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("red.png");
        let pixels: Vec<u8> = [255u8, 0, 0, 255].into_iter().cycle().take(32 * 32 * 4).collect();
        image::save_buffer(&png, &pixels, 32, 32, image::ColorType::Rgba8).unwrap();
        let ply = dir.path().join("grid.ply");
        let mut text = String::from("ply\nformat ascii 1.0\nelement vertex 1089\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n");
        for y in 0..=32 { for x in 0..=32 { text += &format!("{x} {y} 0 0 0 255\n"); } }
        std::fs::write(&ply, text).unwrap();
        let obj = dir.path().join("quad.obj");
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 0 -1\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
        let mut engine = Engine::new().unwrap();
        // 層は 32×32 を (16,16) に置くので中心は (32,32)。+X で切ると右半分が消え、-X で左半分が消える。
        for (name, path, scale) in [("image", &png, None), ("cloud", &ply, None), ("mesh", &obj, Some(16.0))] {
            let right_gone = engine.render_frame(&document(path, scale, 0.0).view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{name}: {:?}", engine.layer_failures());
            assert!(drawn(&right_gone, 24, 32) > 0, "{name}: the kept side is drawn");
            assert_eq!(drawn(&right_gone, 40, 32), 0, "{name}: past the plane is gone");
            let left_gone = engine.render_frame(&document(path, scale, 1.0).view(), RationalTime::ZERO).unwrap();
            assert_eq!(drawn(&left_gone, 24, 32), 0, "{name}: -X flips the cut");
            assert!(drawn(&left_gone, 40, 32) > 0, "{name}: -X keeps the other side");
        }
    }
}
