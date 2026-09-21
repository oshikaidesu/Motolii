//! 解析の入力を解く host の側(2026-09-14 利用者裁定「解析は書類に入れないキャッシュ、resolve は入力として読むだけ」)。
//! 層を本物で組んで読み戻し(Freeze と同じ道)、Blob Track の塊をコマごとに解いて `AnalysisInputs` に置く。
//! ID を持続する・動きで拾う時は入点から 1 コマずつ解き、解いたコマは書類の版が変わるまで持つ(飛んでも辿っても同じ塊)。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::analysis::{AnalysisInputs, BlobMark};
use crate::doc::store::{EffectId, LayerId, RationalTime, StoreView};
use crate::extensions::{blob, overlay};
use crate::render::compositor::LayerContent;
use crate::render::engine::{Engine, EngineError};
use crate::render::media::blob::{detect, mask, BlobSettings, BlobSource};

/// 効果の列の出口を乗算済み線形 Rgba16Float で読み戻した 1 枚(Freeze の cache と Blob の解析が使う)。
pub(crate) struct LinearPicture {
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// 論理の大きさ(余白を除く)。
    pub(crate) natural: [f32; 2],
    /// 効果が宣言した余白(論理 px)。
    pub(crate) padding: u32,
    pub(crate) frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
}

impl Engine {
    pub(super) fn layer_with_passes_linear_picture(
        &mut self,
        lwp: &crate::render::compositor::LayerWithPasses,
    ) -> Result<Option<LinearPicture>, EngineError> {
        let (textures, paddings, _spills, checked_out) = self.compositor.effective_layer_textures(std::slice::from_ref(lwp))?;
        let Some(texture) = textures.first().and_then(|content| content.texture()).cloned() else { return Ok(None) };
        let raw = self.compositor.ctx.gpu_resources.textures.get_from_handle(texture.handle())
            .map_err(|error| EngineError::Store(error.to_string()))?.texture.clone();
        let linear = matches!(&textures[0], LayerContent::LinearTexture(_)) || raw.format().is_srgb();
        let (half, owned) = if raw.format() == wgpu::TextureFormat::Rgba16Float {
            (raw, None)
        } else {
            let mut encoder = self.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-linear-picture-encode"),
            });
            let converted = self.compositor.convert_image_encoding(&mut encoder, &raw, true, !linear, linear);
            self.compositor.pending.push(encoder.finish());
            (converted.clone(), Some(converted))
        };
        let bytes = self.compositor.read_texture_bytes(&half)?;
        for (width, height, format, texture) in checked_out {
            self.compositor.effect_scratch.release(width, height, format, texture);
        }
        let (width, height) = (half.width(), half.height());
        if let Some(owned) = owned {
            self.compositor.effect_scratch.release(owned.width(), owned.height(), owned.format(), owned);
        }
        Ok(Some(LinearPicture {
            bytes,
            width,
            height,
            natural: lwp.layer.size,
            padding: paddings[0],
            frame: lwp.layer.frame,
        }))
    }

    pub(in crate::engine) fn frame_graph_blob_analysis(
        &mut self,
        request: &crate::frame_graph::BlobAnalysisRequestValue,
        previous: Option<&crate::frame_graph::BlobAnalysisValue>,
        t: RationalTime,
        comp: CompSpec,
        fps: crate::doc::store::Fps,
    ) -> Result<crate::frame_graph::BlobAnalysisValue, EngineError> {
        let mut tracker = previous.map(|value| value.tracker.clone()).unwrap_or_default();
        let previous_pixels = previous.and_then(|value| value.previous.as_ref());

        let mut inputs = AnalysisInputs::default();
        let Some(source) = request.source.as_ref() else {
            tracker.step(Vec::new(), &request.settings);
            inputs.set_blobs(request.target, EffectId(0), t, Vec::new());
            return Ok(crate::frame_graph::BlobAnalysisValue {
                inputs,
                tracker,
                previous: None,
                mask: None,
            });
        };

        let previous_clock = self.compositor.clock;
        let frame = t.try_to_frame_round(fps).unwrap_or(0) as f32;
        self.compositor.clock = Some([
            t.as_seconds_f64() as f32,
            fps.den() as f32 / fps.num() as f32,
            frame,
        ]);
        let picture = (|| {
            let scene = crate::frame_graph::SceneValue { layers: vec![source.clone()] };
            let prepared = self.prepare_gpu_scene(&scene, comp, ResolvedCamera::default())?;
            let Some(layer) = prepared.layers.first() else { return Ok(None); };
            self.layer_with_passes_linear_picture(layer)
        })();
        self.compositor.clock = previous_clock;
        let Some(picture) = picture? else {
            tracker.step(Vec::new(), &request.settings);
            inputs.set_blobs(request.target, EffectId(0), t, Vec::new());
            return Ok(crate::frame_graph::BlobAnalysisValue {
                inputs,
                tracker,
                previous: None,
                mask: None,
            });
        };

        let (pixels, width, height, shrink) = shrink_to_srgb(&picture, request.detail);
        let per_logical = picture.width as f32
            / (picture.natural[0] + 2.0 * picture.padding as f32).max(1.0)
            / shrink as f32;
        let to_local = |point: [f32; 2]| {
            glam::vec2(
                point[0] / per_logical - picture.padding as f32,
                point[1] / per_logical - picture.padding as f32,
            )
        };
        let transform = source.transform.affine;
        let comp_per_local = transform.matrix2.x_axis.length().max(1e-6);
        let small = |px: f32| px / comp_per_local * per_logical;
        let area = |value: u32| {
            (value as f64 * f64::from(small(1.0)).powi(2))
                .round()
                .min(u32::MAX as f64) as u32
        };
        let scaled = BlobSettings {
            min_area: area(request.settings.min_area),
            max_area: area(request.settings.max_area),
            max_move: small(request.settings.max_move),
            separation: small(request.settings.separation as f32).round() as u32,
            blur: small(request.settings.blur as f32).round() as u32,
            ..request.settings
        };
        let previous_rgba = previous_pixels
            .filter(|(_, old_width, old_height)| *old_width == width && *old_height == height)
            .map(|(pixels, _, _)| pixels.as_slice());
        let bits = mask(&pixels, width, height, previous_rgba, &scaled);
        let regions = detect(&pixels, width, height, previous_rgba, &scaled);
        let blobs = tracker.step(regions, &scaled);
        let marks: Vec<BlobMark> = blobs.into_iter().map(|blob| {
            let corners = [
                [blob.region.min[0] as f32, blob.region.min[1] as f32],
                [blob.region.max[0] as f32 + 1.0, blob.region.min[1] as f32],
                [blob.region.min[0] as f32, blob.region.max[1] as f32 + 1.0],
                [blob.region.max[0] as f32 + 1.0, blob.region.max[1] as f32 + 1.0],
            ].map(|corner| transform.transform_point2(to_local(corner)));
            let lo = corners.iter().fold(glam::Vec2::splat(f32::MAX), |acc, point| acc.min(*point));
            let hi = corners.iter().fold(glam::Vec2::splat(f32::MIN), |acc, point| acc.max(*point));
            let center = transform.transform_point2(to_local(blob.region.center));
            BlobMark { id: blob.id, center: center.into(), size: (hi - lo).into(), age: blob.age }
        }).collect();
        inputs.set_blobs(request.target, EffectId(0), t, marks);

        Ok(crate::frame_graph::BlobAnalysisValue {
            inputs,
            tracker,
            previous: Some((pixels, width, height)),
            mask: Some((bits.into_iter().map(|bit| u8::from(bit) * 255).collect(), width, height)),
        })
    }

    pub(in crate::engine) fn frame_graph_overlay_analysis(
        &mut self,
        layer: LayerId,
        parent: Option<LayerId>,
        effect: &crate::picture::resolved::ResolvedEffect,
        scene: &crate::frame_graph::SceneValue,
        solver: &crate::frame_graph::SolverPlanValue,
        camera: ResolvedCamera,
        previous: Option<&crate::frame_graph::OverlayAnalysisValue>,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<crate::frame_graph::OverlayAnalysisValue, EngineError> {
        let params = overlay::with_defaults(&effect.plugin_id, &effect.params);
        let method = overlay::number_of(&params, "method").round() as i64;
        let physics = effect.plugin_id == crate::extensions::overlay::PHYSICS_TRACE;
        let own = scene.layers.iter().find(|candidate| candidate.layer == layer && candidate.instance == 0 && !candidate.ghost);
        let own_order = own.map_or(i16::MAX, |candidate| candidate.order);
        let own_z = own.map_or(0.0, |candidate| candidate.transform.spatial.translation.z);

        if method == 2 {
            if physics {
                return Ok(crate::frame_graph::OverlayAnalysisValue {
                    layer,
                    marks: Vec::new(),
                    mask: None,
                    params,
                    depths: None,
                    pushes: Vec::new(),
                    physics: true,
                    tracker: Default::default(),
                    previous: None,
                });
            }
            let mut marks = Vec::new();
            let mut depths = Vec::new();
            let mut pushes = Vec::new();
            for candidate in &scene.layers {
                if candidate.layer == layer || candidate.order >= own_order || candidate.ghost {
                    continue;
                }
                let relation_parent = solver.layers.get(&candidate.layer).and_then(|value| value.relation.parent);
                if relation_parent != parent {
                    continue;
                }
                let size = solver.layers.get(&candidate.layer).and_then(|value| value.size)
                    .or_else(|| semantic_extent(self, candidate, comp));
                let Some(size) = size.filter(|size| size[0] > 0.0 && size[1] > 0.0) else { continue };
                let transform = candidate.transform.affine;
                let corners = [
                    glam::Vec2::ZERO,
                    glam::vec2(size[0], 0.0),
                    glam::Vec2::from(size),
                    glam::vec2(0.0, size[1]),
                ].map(|point| transform.transform_point2(point));
                let lo = corners.iter().fold(glam::Vec2::MAX, |acc, point| acc.min(*point));
                let hi = corners.iter().fold(glam::Vec2::MIN, |acc, point| acc.max(*point));
                marks.push(BlobMark {
                    id: marks.len() as u32,
                    center: ((lo + hi) * 0.5).into(),
                    size: (hi - lo).into(),
                    age: 0,
                });
                depths.push(candidate.transform.spatial.translation.z - own_z);
                pushes.push([0.0, 0.0]);
            }
            return Ok(crate::frame_graph::OverlayAnalysisValue {
                layer,
                marks,
                mask: None,
                params,
                depths: own.filter(|candidate| candidate.projection == crate::doc::store::LayerProjection::ThreeD).map(|_| depths),
                pushes,
                physics: false,
                tracker: Default::default(),
                previous: None,
            });
        }

        let below = crate::frame_graph::SceneValue {
            layers: scene.layers.iter()
                .filter(|candidate| candidate.layer != layer && candidate.order < own_order)
                .cloned()
                .collect(),
        };
        let prepared = self.prepare_gpu_scene(&below, comp, camera)?;
        let picture = if prepared.layers.is_empty() {
            None
        } else {
            let (texture, _view) = self.compositor.render_to_texture(
                comp,
                camera,
                &prepared.layers,
                crate::render::compositor::NO_BACKGROUND,
            )?;
            let mut encoder = self.compositor.ctx.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: Some("motolii-framegraph-overlay-analysis") },
            );
            let converted = self.compositor.convert_image_encoding(
                &mut encoder,
                &texture,
                true,
                !texture.format().is_srgb(),
                true,
            );
            self.compositor.pending.push(encoder.finish());
            let bytes = self.compositor.read_texture_bytes(&converted)?;
            let (width, height) = (converted.width(), converted.height());
            self.compositor.effect_scratch.release(width, height, converted.format(), converted);
            Some(LinearPicture {
                bytes,
                width,
                height,
                natural: [comp.width as f32, comp.height as f32],
                padding: 0,
                frame: None,
            })
        };

        let settings = overlay_settings_of(&params);
        let detail = overlay::number_of(&params, "detail").round().clamp(120.0, 3840.0) as u32;
        let mut tracker = previous.map(|value| value.tracker.clone()).unwrap_or_default();
        let previous_pixels = previous.and_then(|value| value.previous.as_ref());

        let Some(picture) = picture else {
            tracker.step(Vec::new(), &settings);
            return Ok(crate::frame_graph::OverlayAnalysisValue {
                layer,
                marks: Vec::new(),
                mask: None,
                params,
                depths: None,
                pushes: Vec::new(),
                physics: false,
                tracker,
                previous: None,
            });
        };
        let (pixels, width, height, shrink) = shrink_to_srgb(&picture, detail);
        let previous_rgba = previous_pixels
            .filter(|(_, old_width, old_height)| *old_width == width && *old_height == height)
            .map(|(pixels, _, _)| pixels.as_slice());
        let scaled = BlobSettings {
            min_area: ((settings.min_area as f64) / (shrink as f64).powi(2)).round().max(0.0) as u32,
            max_area: ((settings.max_area as f64) / (shrink as f64).powi(2)).round().min(u32::MAX as f64) as u32,
            max_move: settings.max_move / shrink as f32,
            separation: (settings.separation as f32 / shrink as f32).round() as u32,
            blur: (settings.blur as f32 / shrink as f32).round() as u32,
            ..settings
        };
        let bits = mask(&pixels, width, height, previous_rgba, &scaled);
        let regions = detect(&pixels, width, height, previous_rgba, &scaled);
        let blobs = tracker.step(regions, &scaled);
        let marks = blobs.into_iter().map(|blob| BlobMark {
            id: blob.id,
            center: [blob.region.center[0] * shrink as f32, blob.region.center[1] * shrink as f32],
            size: [
                (blob.region.max[0] + 1 - blob.region.min[0]) as f32 * shrink as f32,
                (blob.region.max[1] + 1 - blob.region.min[1]) as f32 * shrink as f32,
            ],
            age: blob.age,
        }).collect();

        Ok(crate::frame_graph::OverlayAnalysisValue {
            layer,
            marks,
            mask: Some((bits.into_iter().map(|bit| u8::from(bit) * 255).collect(), width, height)),
            params,
            depths: None,
            pushes: Vec::new(),
            physics: false,
            tracker,
            previous: Some((pixels, width, height)),
        })
    }

    pub(in crate::engine) fn install_frame_graph_overlays(
        &mut self,
        values: &crate::frame_graph::OverlaySetValue,
    ) {
        self.overlay_frames.clear();
        for (layer, value) in &values.layers {
            self.overlay_frames.insert(*layer, OverlayFrame {
                marks: value.marks.clone(),
                mask: value.mask.clone(),
                params: value.params.clone(),
                depths: value.depths.clone(),
                pushes: value.pushes.clone(),
                links: Vec::new(),
                wells: Vec::new(),
                contacts: Vec::new(),
                velocities: Vec::new(),
                hulls: Vec::new(),
                physics: value.physics,
            });
        }
    }

    /// Compatibility projection for tests/tools that still consume ResolvedLayer.
    /// Product meaning is evaluated by FrameGraph first; this must never re-enter
    /// the legacy StoreView + analysis_inputs + resolved_layers owner.
    pub(super) fn resolved_with_analysis(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<Vec<ResolvedLayer>, EngineError> {
        self.resolve_tally.clear();
        self.resolve_worst.clear();
        let (scene, _camera, _comp, fps) = self.evaluate_frame_graph_semantics(view, t)?;
        Ok(super::frame_graph::resolved_layers_from_scene(&scene, t, fps))
    }

    /// 連続性の物差しの標本。作品意味は FrameGraph で一度だけ評価し、
    /// semantic Scene/TextFlow の最終値から測る。診断のために legacy resolve を再実行しない。
    pub fn continuity_samples(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<Vec<(String, [f32; 2])>, EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let (scene, camera, comp, _fps) = self.evaluate_frame_graph_semantics(view, t)?;
        let canvas = crate::picture::shapes_ops::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        let mut out = Vec::new();
        out.push(("camera.center".to_owned(), camera.center));
        out.push(("camera.distance".to_owned(), [camera.distance_scale.max(1e-3).ln() * 300.0, camera.target_z]));

        for layer in &scene.layers {
            if layer.ghost || layer.instance != 0 || layer.opacity <= 0.0
                || matches!(layer.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Stage | crate::doc::store::LayerSource::Null)
            {
                continue;
            }
            let world = layer.transform.spatial;
            let to_screen = |point: glam::Vec2| world.transform_point3(point.extend(0.0)).truncate().to_array();
            let name = view.attrs(layer.layer).map_err(store)?.unwrap_or_default().name;

            let bounds = match &layer.content {
                crate::frame_graph::SceneContentValue::Text(text) => {
                    crate::picture::text_frame::line_box(&text.document, &text.shaped, &canvas)
                }
                crate::frame_graph::SceneContentValue::Shape(shapes) => {
                    let stretched;
                    let shapes = if layer.shape_stretch != [1.0, 1.0] {
                        stretched = crate::picture::shapes_ops::stretch_outline(shapes, layer.shape_stretch);
                        stretched.as_slice()
                    } else {
                        shapes.as_slice()
                    };
                    crate::picture::shapes_ops::content_bounds(shapes).ok().flatten()
                        .map(|bounds| bounds.map(|value| value as f32))
                }
                crate::frame_graph::SceneContentValue::Media { source, .. }
                | crate::frame_graph::SceneContentValue::Material(crate::frame_graph::MaterialValue { source }) => {
                    self.material_extent(&source.path, comp).map(|extent| [0.0, 0.0, extent[0], extent[1]])
                }
                crate::frame_graph::SceneContentValue::Plate(_) => Some([0.0, 0.0, comp.width as f32, comp.height as f32]),
                _ => None,
            };
            if let Some(bounds) = bounds {
                out.push((format!("L{} {name}.min", layer.layer.0), to_screen(glam::vec2(bounds[0], bounds[1]))));
                out.push((format!("L{} {name}.max", layer.layer.0), to_screen(glam::vec2(bounds[2], bounds[3]))));
                let center = world.transform_point3(glam::vec3(
                    (bounds[0] + bounds[2]) * 0.5,
                    (bounds[1] + bounds[3]) * 0.5,
                    0.0,
                ));
                out.push((format!("L{} {name}.z", layer.layer.0), [center.z, 0.0]));
            }

            let crate::frame_graph::SceneContentValue::Text(text) = &layer.content else { continue };
            let content = text.document.content.eval(t);
            let glyph_bytes: Vec<usize> = text.shaped.lines.iter()
                .flat_map(|line| line.glyph_bytes.iter().copied())
                .collect();
            // TextFlow has already applied Shape Outside and transition offsets to
            // contours. Use those final contours instead of reconstructing offsets
            // through the legacy text resolver.
            let mut glyph_points: std::collections::BTreeMap<usize, glam::Vec2> = std::collections::BTreeMap::new();
            for (contour, glyph) in text.shaped.contours.iter().zip(&text.shaped.contour_glyphs) {
                let lo = contour.vertices.iter().fold(glam::Vec2::splat(f32::MAX), |acc, vertex| {
                    acc.min(glam::vec2(vertex.point.x as f32, vertex.point.y as f32))
                });
                if lo.is_finite() {
                    glyph_points.entry(*glyph).and_modify(|point| *point = point.min(lo)).or_insert(lo);
                }
            }
            for (glyph, byte) in glyph_bytes.into_iter().enumerate() {
                if content.get(byte..).and_then(|rest| rest.chars().next()).is_some_and(char::is_whitespace) {
                    continue;
                }
                if let Some(point) = glyph_points.get(&glyph) {
                    out.push((format!("L{} {name}.g{byte}", layer.layer.0), to_screen(*point)));
                }
            }
        }
        Ok(out)
    }

}


/// Track Overlay のこのコマの塊と取っ手(描く側が箱・印を組む)。
pub(crate) struct OverlayFrame {
    pub(crate) marks: Vec<BlobMark>,
    /// Show Mask の二値(縮めた解析の絵の寸法)。
    pub(crate) mask: Option<(Vec<u8>, u32, u32)>,
    pub(crate) params: Vec<(String, crate::doc::store::Value)>,
    /// 3D の Found Grid(Layers)が読んだ物ごとの奥行き(`marks` と同じ順、層の奥行きからの差)。2D なら None。
    pub(crate) depths: Option<Vec<f32>>,
    /// Layers が読んだ物ごとの押されたずれ(comp の向き、`marks` と同じ順)。塊を読む時は空。
    pub(crate) pushes: Vec<[f32; 2]>,
    /// 物理の可視: 触れ合っている組の線(comp の px)。
    pub(crate) links: Vec<([f32; 2], [f32; 2])>,
    /// 物理の可視: 場の元と届く距離(0 なら箱じゅう)と、一様な向き。
    pub(crate) wells: Vec<([f32; 2], f32, [f32; 2])>,
    /// 物理の可視: 触れ合っている点と法線、今の速さ、当たりに使っている輪郭。
    pub(crate) contacts: Vec<([f32; 2], [f32; 2])>,
    pub(crate) velocities: Vec<([f32; 2], [f32; 2])>,
    pub(crate) hulls: Vec<Vec<[f32; 2]>>,
    /// 物理の可視なら真(中身は描く直前に解き手から取る)。
    pub(crate) physics: bool,
}

fn semantic_extent(engine: &mut Engine, layer: &crate::frame_graph::SceneLayerValue, comp: CompSpec) -> Option<[f32; 2]> {
    match &layer.content {
        crate::frame_graph::SceneContentValue::Text(text) => crate::picture::shapes_ops::content_canvas(&text.shapes()).ok().flatten().map(|canvas| [canvas.width as f32, canvas.height as f32]),
        crate::frame_graph::SceneContentValue::Shape(shapes) => crate::picture::shapes_ops::content_canvas(shapes).ok().flatten().map(|canvas| [canvas.width as f32, canvas.height as f32]),
        crate::frame_graph::SceneContentValue::Media { source, .. } | crate::frame_graph::SceneContentValue::Material(crate::frame_graph::MaterialValue { source }) => engine.material_extent(&source.path, comp).map(|extent| [extent[0], extent[1]]),
        crate::frame_graph::SceneContentValue::Particles(value) => {
            let mut hi = glam::Vec2::ZERO;
            for particle in &value.particles { hi = hi.max(glam::Vec2::new(particle.position[0], particle.position[1])); }
            Some([hi.x.max(1.0), hi.y.max(1.0)])
        }
        crate::frame_graph::SceneContentValue::Plate(_) => Some([comp.width as f32, comp.height as f32]),
        crate::frame_graph::SceneContentValue::None => None,
    }
}

fn overlay_settings_of(params: &[(String, crate::doc::store::Value)]) -> BlobSettings {
    let n = |name| overlay::number_of(params, name);
    let threshold = (n("threshold") / 100.0) as f32;
    let key = overlay::color_of(params, "key_color");
    BlobSettings {
        source: if n("method").round() as i64 == 1 { BlobSource::Color { target: [key[0] as f32, key[1] as f32, key[2] as f32], tolerance: threshold } } else { BlobSource::Motion { threshold } },
        min_area: n("min_region").max(0.0) as u32,
        max_area: n("max_region").clamp(0.0, u32::MAX as f64) as u32,
        max_blobs: 512,
        persist: overlay::switch_of(params, "keep_ids"),
        max_move: 60.0,
        revive_frames: 5,
        separation: n("separation").max(0.0).round() as u32,
        blur: n("blur").max(0.0).round() as u32,
    }
}

pub(crate) fn settings_of(params: &[(String, crate::doc::store::Value)]) -> BlobSettings {
    let n = |name| blob::number_of(params, name);
    let source = match n("mode").round() as i64 {
        1 => BlobSource::Motion { threshold: n("threshold") as f32 },
        2 => BlobSource::Color { target: [n("red") as f32, n("green") as f32, n("blue") as f32], tolerance: n("tolerance") as f32 },
        _ => BlobSource::Luminance { threshold: n("threshold") as f32, invert: n("invert") >= 0.5 },
    };
    BlobSettings {
        source,
        min_area: n("min_area").max(0.0) as u32,
        max_area: n("max_area").clamp(0.0, u32::MAX as f64) as u32,
        max_blobs: n("max_blobs").max(1.0) as usize,
        persist: n("persist") >= 0.5,
        max_move: n("max_move").max(0.0) as f32,
        revive_frames: n("revive").max(0.0) as u32,
        separation: n("separation").max(0.0).round() as u32,
        blur: 0,
    }
}

/// 乗算済み線形 Rgba16Float を、長辺が上限に収まるよう箱で縮めて非乗算 sRGB の RGBA8 に。戻り値の最後は縮めた倍率。
pub(crate) fn shrink_to_srgb(picture: &LinearPicture, long_side: u32) -> (Vec<u8>, u32, u32, u32) {
    let shrink = picture.width.max(picture.height).div_ceil(long_side.max(1)).max(1);
    let (w, h) = (picture.width / shrink, picture.height / shrink);
    let texel = |x: u32, y: u32, c: usize| {
        let i = ((y * picture.width + x) * 4) as usize * 2 + c * 2;
        half::f16::from_le_bytes([picture.bytes[i], picture.bytes[i + 1]]).to_f32()
    };
    let encode = |v: f32| {
        let v = v.clamp(0.0, 1.0);
        let s = if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
        (s * 255.0).round() as u8
    };
    let mut out = vec![0u8; (w * h * 4) as usize];
    let count = (shrink * shrink) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0.0f32; 4];
            for dy in 0..shrink {
                for dx in 0..shrink {
                    for (c, s) in sum.iter_mut().enumerate() { *s += texel(x * shrink + dx, y * shrink + dy, c); }
                }
            }
            let [r, g, b, a] = sum.map(|s| s / count);
            let o = ((y * w + x) * 4) as usize;
            let unpremultiply = |v: f32| if a > 1e-5 { v / a } else { 0.0 };
            out[o..o + 4].copy_from_slice(&[encode(unpremultiply(r)), encode(unpremultiply(g)), encode(unpremultiply(b)), (a.clamp(0.0, 1.0) * 255.0).round() as u8]);
        }
    }
    (out, w, h, shrink)
}
