//! クリッピング(↳)の描く側。下地の局所座標で絵だけを合わせ、下地の配置は畳まない。
use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses,
};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    /// クリップ層を下地の**局所座標**(素材の画素、効果の余白込み)へ描き、下地の alpha で
    /// source-atop する。下地の配置(z・回転・world)は触らない — 畳むのは絵だけで、
    /// 点群や他の 3D 層との前後は下地が普通の層として解く。下地に texture が無ければ None。
    pub(super) fn clip_onto_base(
        &mut self,
        base: LayerWithPasses,
        upper: &Layer,
        upper_passes: &[EffectPass],
    ) -> Result<Option<LayerWithPasses>, EngineError> {
        use crate::doc::core::LayerPlacement;
        use crate::doc::store::LayerProjection;
        use glam::{Affine2, Affine3A, Vec2};

        let Some(base_texture) = base.layer.content.texture().cloned() else {
            return Ok(None);
        };
        let [width, height] = base_texture.width_height();
        let stretch = Vec2::new(base.layer.size[0] / width as f32, base.layer.size[1] / height as f32);
        let comp_from_pixels = base.layer.placement.transform * Affine2::from_scale(stretch);
        if comp_from_pixels.matrix2.determinant().abs() < 1e-9 {
            return Ok(Some(base));
        }
        let pad = base.passes.iter().map(EffectPass::padding).max().unwrap_or(0);
        let padf = pad as f32;
        let local = CompSpec { width: width + 2 * pad, height: height + 2 * pad };
        let flat = ResolvedCamera::default();
        let flat_layer = |content: LayerContent, size: [f32; 2], transform: Affine2, opacity: f32| Layer {
            content,
            size,
            placement: LayerPlacement { transform, opacity, ..LayerPlacement::default() },
            projection: LayerProjection::TwoD,
            projection_camera: flat,
            blend_mode: CompositeBlendMode::Normal,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            blocks_light: base.layer.blocks_light,
            outline: base.layer.outline,
        };
        let bake = |engine: &mut Self, layer: Layer, passes: &[EffectPass]| {
            let (texture, _view) = engine.compositor.render_to_texture(
                local,
                flat,
                &[LayerWithPasses { layer, passes: passes.to_vec() }],
                crate::render::compositor::NO_BACKGROUND,
            )?;
            engine.compositor.import_premultiplied(&texture)
        };
        let base_local = bake(
            self,
            flat_layer(
                LayerContent::Texture(base_texture),
                [width as f32, height as f32],
                Affine2::from_translation(Vec2::splat(padf)),
                1.0,
            ),
            &base.passes,
        )?;
        let pixels_from_comp = Affine2::from_translation(Vec2::splat(padf)) * comp_from_pixels.inverse();
        let upper_local = bake(
            self,
            flat_layer(
                upper.content.clone(),
                upper.size,
                pixels_from_comp * upper.placement.transform,
                upper.placement.opacity,
            ),
            upper_passes,
        )?;
        let clipped = self.compositor.source_atop(&base_local, &upper_local, upper.blend_mode)?;
        let unpad = Vec2::splat(-padf);
        Ok(Some(LayerWithPasses {
            layer: Layer {
                content: LayerContent::Texture(clipped),
                size: [local.width as f32, local.height as f32],
                placement: LayerPlacement {
                    transform: comp_from_pixels * Affine2::from_translation(unpad),
                    world_transform: base.layer.placement.world_transform.map(|world| {
                        world
                            * Affine3A::from_scale(stretch.extend(1.0))
                            * Affine3A::from_translation(unpad.extend(0.0))
                    }),
                    ..base.layer.placement
                },
                ..base.layer
            },
            passes: Vec::new(),
        }))
    }
}

#[cfg(test)]
mod clipping_contract {
    //! クリッピング(↳)は下地の局所座標で**絵だけ**を合わせる。下地の配置(z・回転・projection)は
    //! Document でも render でも畳まないので、点群や他の 3D 層との前後は普通の層として解ける。
    use crate::doc::store::{
        property, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
        LayerProjection, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
    };
    use crate::render::engine::Engine;

    fn composition(width: u32, height: u32) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width,
            height,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0; 4],
        }))
        .unwrap();
        doc
    }

    fn add_file_layer(doc: &mut Document, id: u64, path: &std::path::Path, order: i16, at: [f64; 2]) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                    order,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2(at),
            },
        ])
        .unwrap();
        layer
    }

    fn png(path: &std::path::Path, width: u32, height: u32, pixel: impl Fn(u32, u32) -> [u8; 4]) {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&pixel(x, y));
            }
        }
        image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8).unwrap();
    }

    fn at(frame: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let start = ((y * width + x) * 4) as usize;
        frame[start..start + 4].try_into().unwrap()
    }

    fn clip(doc: &mut Document, layer: LayerId) {
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { clip_to_below: Some(true), ..Default::default() } })
            .unwrap();
    }

    #[test]
    fn clipping_keeps_the_visible_base_and_never_draws_without_coverage() {
        let dir = tempfile::tempdir().unwrap();
        let (width, height) = (16u32, 16u32);
        let base_path = dir.path().join("base.png");
        let upper_path = dir.path().join("upper.png");
        png(&base_path, width, height, |x, _| [0, 255, 0, if x < width / 2 { 128 } else { 0 }]);
        png(&upper_path, width, height, |_, y| [255, 0, 0, if y < height / 2 { 255 } else { 0 }]);
        let mut doc = composition(width, height);
        let base = add_file_layer(&mut doc, 1, &base_path, 0, [0.0, 0.0]);
        let upper = add_file_layer(&mut doc, 2, &upper_path, 1, [0.0, 0.0]);
        clip(&mut doc, upper);
        let mut engine = Engine::new().unwrap();
        let clipped = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let p = |x, y| at(&clipped, width, x, y);
        assert!(p(3, 3)[0] > 60 && p(3, 3)[1] < 15, "upper color missing inside base: {:?}", p(3, 3));
        assert!((120..=136).contains(&p(3, 3)[3]), "clipping inflated the base alpha: {:?}", p(3, 3));
        assert!(p(3, 12)[1] > 60 && (120..=136).contains(&p(3, 12)[3]), "base was consumed like a legacy matte");
        assert_eq!(p(12, 3)[3], 0, "upper escaped base coverage");

        let exported = crate::render::export::export_lottie(&doc.view()).unwrap();
        assert!(exported.unsupported.iter().any(|issue| issue.layer == Some(upper) && issue.category == "clipping"));

        let backdrop_path = dir.path().join("backdrop.png");
        png(&backdrop_path, width, height, |_, _| [0, 0, 255, 255]);
        let backdrop = add_file_layer(&mut doc, 3, &backdrop_path, -1, [0.0, 0.0]);
        let with_backdrop = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let outside = at(&with_backdrop, width, 12, 3);
        assert!(outside[0] < 15 && outside[2] > 240, "unrelated backdrop became clipping coverage: {outside:?}");
        let inside = at(&with_backdrop, width, 3, 3);
        assert!(inside[0] > 60 && inside[2] > 60, "clipping group failed to composite translucently over backdrop: {inside:?}");
        doc.apply(Intent::RemoveLayer(backdrop)).unwrap();

        doc.apply(Intent::SetAttrs { layer: base, patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() } })
            .unwrap();
        let hidden = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(hidden.chunks_exact(4).all(|px| px[3] == 0), "hidden base left unbounded clipping");
        doc.apply(Intent::RemoveLayer(base)).unwrap();
        let orphan = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(orphan.chunks_exact(4).all(|px| px[3] == 0), "missing base left unbounded clipping");
    }

    fn ply_plane(path: &std::path::Path, size: u32, rgb: [u8; 3]) {
        let mut text = format!(
            "ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nend_header\n",
            (size + 1) * (size + 1)
        );
        for y in 0..=size {
            for x in 0..=size {
                text.push_str(&format!("{x} {y} 0 {} {} {}\n", rgb[0], rgb[1], rgb[2]));
            }
        }
        std::fs::write(path, text).unwrap();
    }

    /// 点群を下地の前(z<0)と後ろ(z>0)に置く。クリップ後も下地は自分の z に居るので、
    /// 前の点は下地を隠し、後ろの点は下地に隠れる。
    #[test]
    fn clipping_leaves_the_base_in_its_own_depth() {
        let dir = tempfile::tempdir().unwrap();
        let (width, height) = (64u32, 64u32);
        let base_path = dir.path().join("base.png");
        let upper_path = dir.path().join("upper.png");
        let cloud_path = dir.path().join("cloud.ply");
        png(&base_path, 16, 16, |_, _| [0, 255, 0, 255]);
        png(&upper_path, 16, 16, |_, _| [255, 0, 0, 255]);
        ply_plane(&cloud_path, 32, [0, 0, 255]);

        let scene = |cloud_z: f64| {
            let mut doc = composition(width, height);
            let cloud = add_file_layer(&mut doc, 1, &cloud_path, 0, [16.0, 16.0]);
            doc.apply(Intent::SetConstant {
                layer: cloud,
                property: PropertyId::new(property::POSITION_Z).unwrap(),
                value: Value::F64(cloud_z),
            })
            .unwrap();
            let base = add_file_layer(&mut doc, 2, &base_path, 1, [24.0, 24.0]);
            let upper = add_file_layer(&mut doc, 3, &upper_path, 2, [24.0, 24.0]);
            clip(&mut doc, upper);
            (doc, base)
        };
        let mut engine = Engine::new().unwrap();

        let (behind, base) = scene(4.0);
        let frame = engine.render_frame(&behind.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        let center = at(&frame, width, 32, 32);
        assert!(center[0] > 200 && center[2] < 40, "clipped upper should show inside the base over a cloud behind it: {center:?}");
        let beside = at(&frame, width, 20, 20);
        assert!(beside[2] > 120 && beside[0] < 40, "cloud must stay visible outside the base: {beside:?}");

        let attrs = behind.view().attrs(base).unwrap().unwrap_or_default();
        assert_eq!(attrs.projection, LayerProjection::ThreeD, "clipping must not fold the base's projection");
        assert!(!attrs.flatten, "clipping must not flatten the base");

        let (in_front, _) = scene(-4.0);
        let frame = engine.render_frame(&in_front.view(), RationalTime::ZERO).unwrap();
        let center = at(&frame, width, 32, 32);
        assert!(center[2] > 120 && center[0] < 40, "a cloud in front of the base must cover the clipped pair: {center:?}");
    }
}
