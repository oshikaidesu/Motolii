//! 層の種類ごとの絵 — どの素材がどの内容になるかの振り分けと、種類ごとの焼き方。

use super::*;

impl Engine {
    pub(crate) fn texture_for_resolved(
        &mut self,
        layer: &ResolvedLayer,
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        camera: crate::doc::core::ResolvedCamera,
        projection_camera: crate::doc::core::ResolvedCamera,
    ) -> Result<(Option<LayerContent>, [f32; 2], Option<crate::render::compositor::effects::vism::ImageFrame>), EngineError> {
        // Track Overlay(出力の効果)を持つ層は、自分の素材の代わりに下の合成から組んだ箱・印を描く(調整層と同じ)。
        if layer.effects.iter().any(|e| crate::extensions::overlay::is_track_overlay(&e.plugin_id)) {
            return Ok(match self.overlay_content(layer.id, comp)? { Some((content, natural)) => (Some(content), natural, None), None => (None, [0.0, 0.0], None) });
        }
        let needs_material = self.compositor.catalog.descriptors.iter().any(|d| matches!(d.stage, crate::render::compositor::EffectStage::Warp | crate::render::compositor::EffectStage::Field) && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        // 絵を読む効果(pass)は素材座標の絵を要る。comp 大に焼くと comp の外が失われ、
        // Blur が縁で切れる(広がりの法: 評価の入力を view・comp・カメラで切らない)。
        // Motion Blur の写しも足す合成に乗るよう矩形(素材座標の絵)で持つ。comp 大に焼くと画面の外に出た部分が切れる。
        let needs_image = self.material_picture == Some(layer.id) || !crate::render::engine::translate::translate_effect_passes(&layer.effects).is_empty() || layer.averaged > 0;
        // 立体を作る族(Extrude・Bevel の効果)。効果が無ければ Depth 属性(互換)。
        let solid = crate::render::engine::translate::translate_solid(&layer.effects)
            .map(|s| if s.depth > 0.0 { s } else { crate::render::compositor::extrude::Solid { depth: layer.depth, ..s } })
            .unwrap_or(crate::render::compositor::extrude::Solid { depth: layer.depth, bevel: None });
        let flat = solid.extent() <= 0.0;
        // 場(Field)だけなら輪郭を刻んだ mesh のまま描く(線に場が乗る)。warp と絵の効果は素材の絵が要る。
        let needs_warp = self.compositor.catalog.descriptors.iter().any(|d| d.stage == crate::render::compositor::EffectStage::Warp && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        let needs_field = needs_material && !needs_warp;
        let vector = flat && layer.masks.is_empty() && !needs_warp && !needs_image && !self.clip_bases.contains(&layer.id);
        let step = (vector && needs_field).then_some(FIELD_STEP);
        // Blob Track が形の素材を箱へ合わせた写し: 輪郭だけを伸ばす(線は太らない)。大きさは写しごとに違うので cache に残さない。
        // Display の Group の背景も形の書類を持ち、形の層と同じ道で描く。
        let shaped = layer.source == LayerSource::Shape || (layer.source == LayerSource::Group && shape_documents.contains_key(&layer.id));
        let stretched = (shaped && layer.shape_stretch != [1.0, 1.0])
            .then(|| crate::picture::shapes_ops::stretch_outline(shape_documents.get(&layer.id).map(Vec::as_slice).unwrap_or(&[]), layer.shape_stretch));
        let natural = if shaped {
            let canvas = content_canvas(stretched.as_deref().or(shape_documents.get(&layer.id).map(Vec::as_slice)).unwrap_or(&[]))?;
            canvas.map_or([1.0; 2], |c| [c.width as f32, c.height as f32])
        } else { [comp.width as f32, comp.height as f32] };
        let size = layer_size(layer, natural);
        let (origin, u, v) = crate::render::compositor::projected_placement_corners(comp, projection_camera, layer.projection, layer.placement, glam::Vec2::ZERO, size.into());
        let projection = crate::doc::core::camera_projection(comp, camera);
        let matrix = projection.projection_matrix() * projection.view_matrix();
        let project = |p: glam::Vec3| {
            let p = matrix * p.extend(1.0);
            glam::vec2(p.x / p.w, p.y / p.w) * glam::vec2(comp.width as f32, comp.height as f32) * 0.5
        };
        let density = [origin, origin+u, origin+v, origin+u+v, origin+(u+v)*0.5].into_iter().flat_map(|p| {
            [(project(p + u / natural[0].max(1.0)) - project(p)).length(),
             (project(p + v / natural[1].max(1.0)) - project(p)).length()]
        }).filter(|v| v.is_finite()).fold(1.0f32, f32::max);
        // 浮動小数の 20.000002 が 361 画素を生み、置いた時に 1 画素ずれて縁が甘くなる — 1/1024 に丸める。
        let mut exact_density = ((density.max(1.0) * 1024.0).round() / 1024.0).max(1.0);
        if needs_image {
            // 絵 + 効果の reach(余白)が device の texture 上限を越えると wgpu は無効な texture を返し、
            // それが scratch pool に入って以後の全フレームが壊れる。密度の側で先に収める。
            let reach = crate::render::engine::translate::translate_effect_passes(&layer.effects).iter().map(|p| p.padding() as f32).fold(0.0f32, f32::max);
            let limit = self.compositor.ctx.device.limits().max_texture_dimension_2d as f32;
            let extent = natural[0].max(natural[1]).max(1.0) + 2.0 * reach;
            exact_density = exact_density.min((limit / extent).max(1.0));
        }
        let density = (density * (1.0 - 1e-4)).log2().ceil().exp2().max(1.0);
        // 輪郭の細分は 2 の冪の段で cache を使い回す。絵に描く時は投影の密度そのもので描く
        // (段に丸めると置いた時に再標本化され、縁が甘くなる)。
        let tolerance = (0.05 / if vector { density } else { exact_density }).max(1e-6);
        let (content, natural, frame) = if layer.source == LayerSource::Particles {
            // 粒は焼かない: 点群と同じく点のまま view へ渡る(効果の法 2026-09-13「粒子は形」)。
            match self.particle_frames.get(&layer.id) {
                Some(frame) => (Some(LayerContent::Cloud {
                    positions: frame.positions.clone(),
                    colors: frame.colors.clone(),
                    bounds: frame.bounds,
                    point_size: 1.0,
                    sizes: Some(frame.sizes.clone()),
                    sprites: true,
                    links: frame.links.clone(),
                }), [frame.bounds.max[0].max(1.0), frame.bounds.max[1].max(1.0)], None),
                None => (None, [1.0, 1.0], None),
            }
        } else if layer.source == LayerSource::Text {
            self.text_texture_from_document(text_documents.get(&layer.id), text::morph_partner(layer, text_documents), text::Flow::of(layer), layer.id, t, comp, vector, tolerance, flat, step)?
        } else if shaped {
            let shapes = stretched.as_deref().or(shape_documents.get(&layer.id).map(Vec::as_slice)).unwrap_or(&[]);
            let (content,natural)=self.shape_texture_from_shapes(shapes, layer.id, vector, tolerance, comp, step, stretched.is_none())?;
            // 密度 > 1 で描いた絵は、その枠を持ち歩く(効果の reach・radius は論理 px)。
            let frame = content.as_ref().and_then(|c| c.texture()).map(|t| crate::render::compositor::effects::vism::ImageFrame { size: natural, origin: [0.0; 2], pixels: t.width_height() });
            (content,natural,frame)
        } else if let LayerSource::File { path, .. } = &layer.source {
            let path = path.clone();
            if layer.environment && crate::render::media::is_still_image_path(&path) {
                return self.environment_content_for(&path).map(|(c,n)|(c,n,None));
            }
            { let (content,natural)=self.file_content_for(&path, layer.source_time, layer.id, comp)?; (content,natural,None) }
        } else {
            { let (content,natural)=self.texture_for(&layer.source, layer.source_frame)?; (content,natural,None) }
        };
        match content {
            Some(LayerContent::Texture(texture)) if !flat && layer.projection != crate::doc::store::LayerProjection::TwoD && layer.masks.is_empty() => {
                Ok((self.extruded_content(layer, texture, natural, text_documents, shape_documents, t, comp, solid)?, natural, frame))
            }
            content => Ok((content, natural, frame)),
        }
    }

    /// 奥行きのある板は輪郭どおりの網になる(形・文字は fill の輪郭、画は矩形)。
    /// 絵の handle と奥行きが同じ間は作り直さない。輪郭が出ない(空の形)なら板のまま。
    fn extruded_content(
        &mut self,
        layer: &ResolvedLayer,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        solid: crate::render::compositor::extrude::Solid,
    ) -> Result<Option<LayerContent>, EngineError> {
        let key = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            texture.handle.hash(&mut hasher);
            solid.hash_key(&mut hasher);
            hasher.finish()
        };
        if let Some((cached, model)) = self.extrusions.get(&layer.id) {
            if *cached == key {
                return Ok(Some(LayerContent::Model(model.clone())));
            }
        }
        let rectangle = |size: [f32; 2]| {
            let v = |x: f32, y: f32| re_renderer::renderer::PathVertex { point: glam::vec2(x, y), in_tangent: glam::Vec2::ZERO, out_tangent: glam::Vec2::ZERO };
            vec![(vec![re_renderer::renderer::PathContour { closed: true, vertices: vec![v(0.0, 0.0), v(size[0], 0.0), v(size[0], size[1]), v(0.0, size[1])] }], re_renderer::renderer::PathFillRule::NonZero)]
        };
        let outlines = match &layer.source {
            LayerSource::Text => {
                let canvas = crate::picture::shapes_ops::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
                match text_documents.get(&layer.id).and_then(|d| text::text_shapes_moving(d, text::morph_partner(layer, text_documents), text::Flow::of(layer), t, &canvas).ok().flatten()) {
                    Some(shapes) => crate::render::compositor::paths::outlines(&shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            LayerSource::Shape => {
                let shapes = shape_documents.get(&layer.id).map(Vec::as_slice).unwrap_or(&[]);
                // 並べる法の Fill / Blob の箱合わせで輪郭を伸ばした形は、押し出しも伸ばした輪郭から(板の絵と同じ形)。
                let stretched = (layer.shape_stretch != [1.0, 1.0]).then(|| crate::picture::shapes_ops::stretch_outline(shapes, layer.shape_stretch));
                let shapes = stretched.as_deref().unwrap_or(shapes);
                match content_canvas(shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            _ => rectangle(natural),
        };
        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(Some(LayerContent::Texture(texture)));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(layer.id, (key, model.clone()));
        Ok(Some(LayerContent::Model(model)))
    }

    #[allow(clippy::too_many_arguments)]
    fn text_texture_from_document(
        &mut self,
        document: Option<&TextDocument>,
        morph: Option<(&TextDocument, f64)>,
        flow: text::Flow<'_>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
        vector: bool,
        tolerance: f32,
        crop: bool,
        step: Option<f32>,
    ) -> Result<(Option<LayerContent>, [f32; 2], Option<crate::render::compositor::effects::vism::ImageFrame>), EngineError> {
        let Some(document) = document else {
            return Ok((None, [0.0, 0.0], None));
        };

        let canvas = crate::picture::shapes_ops::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };

        let key = TextCacheKey::new(layer_id, document, morph, t, canvas.width, canvas.height).moving(flow);
        if let Some(cached) = self.text_textures.get(&key).filter(|c| matches!(c.texture, LayerContent::Model(_)) == vector && c.tolerance <= tolerance && c.step == step) {
            return Ok((
                Some(cached.texture.clone()),
                [canvas.width as f32, canvas.height as f32],
                cached.frame,
            ));
        }

        let Some(shapes) = text::text_shapes_moving(document, morph, flow, t, &canvas)? else {
            return Ok((None, [0.0, 0.0], None));
        };
        let bounds = crate::picture::shapes_ops::content_bounds(&shapes)?.map(|b| crate::render::media::SpatialBounds {
            min: [b[0] as f32, b[1] as f32, 0.0],
            max: [b[2] as f32, b[3] as f32, 0.0],
        });
        let raster_canvas = if !vector && crop { content_canvas(&shapes)?.unwrap_or_else(|| canvas.clone()) } else { canvas.clone() };
        let content = if vector {
            self.compositor.path_model(&shapes, &canvas, tolerance, step)?.map(|m| LayerContent::Model(std::sync::Arc::new(m)))
        } else { self.compositor.render_paths("text", &shapes, &raster_canvas, 0.05 / tolerance, raster_pixel_budget(comp))?.map(LayerContent::Texture) };
        let Some(texture) = content else {
            return Ok((None, [0.0, 0.0], None));
        };
        let frame = texture.texture().map(|t| crate::render::compositor::effects::vism::ImageFrame {
            size: [raster_canvas.width as f32,raster_canvas.height as f32],
            origin: [-(raster_canvas.origin_x as f32),-(raster_canvas.origin_y as f32)], pixels: t.width_height(),
        });
        let fresh = !self.text_textures.contains_key(&key);
        self.text_textures.insert(key.clone(), TextTexture { texture: texture.clone(), bounds, tolerance, step, frame });
        if fresh { self.text_order.push_back(key); }
        // 級数を擦るだけで鍵が増える。歌詞 200 行 + 擦りの残骸で GPU を食い潰さない。
        while self.text_order.len() > TEXT_CACHE_LIMIT {
            if let Some(old) = self.text_order.pop_front() {
                self.text_textures.remove(&old);
            }
        }
        Ok((
            Some(texture),
            [canvas.width as f32, canvas.height as f32],
            frame,
        ))
    }

    pub(in crate::engine) fn shape_texture_from_shapes(
        &mut self,
        shapes: &[ShapeNode],
        layer_id: LayerId,
        vector: bool,
        tolerance: f32,
        comp: CompSpec,
        step: Option<f32>,
        remember: bool,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }

        let Some(canvas) = content_canvas(shapes)? else {
            return Ok((None, [0.0, 0.0]));
        };

        let key = ShapeCacheKey::new(layer_id, shapes, canvas.width, canvas.height);
        if let Some(cached) = self.shape_textures.get(&key).filter(|c| matches!(c.texture, LayerContent::Model(_)) == vector && c.tolerance <= tolerance && c.step == step) {
            return Ok((
                Some(cached.texture.clone()),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let content = if vector {
            self.compositor.path_model(shapes, &canvas, tolerance, step)?.map(|m| LayerContent::Model(std::sync::Arc::new(m)))
        } else { self.compositor.render_paths("shape", shapes, &canvas, 0.05 / tolerance, raster_pixel_budget(comp))?.map(LayerContent::Texture) };
        let Some(texture) = content else {
            return Ok((None, [0.0, 0.0]));
        };
        if remember {
            self.shape_textures.insert(key, TextTexture { texture: texture.clone(), bounds: None, tolerance, step, frame: None });
        }
        Ok((
            Some(texture),
            [canvas.width as f32, canvas.height as f32],
        ))
    }

    /// 網も焼かない。三角形のまま run の view へ渡り、深度で板と刺さり合う。
    /// 素材ファイルの元の寸法(幅・高さ・奥行き)。網・点群は読み込んで bounds、画・動画は probe(どちらも一度だけ)。
    pub(in crate::engine) fn material_extent(&mut self, path: &str, comp: CompSpec) -> Option<[f32; 3]> {
        if crate::render::media::is_mesh_path(path) {
            self.mesh_content_for(path, comp).ok()?;
            return self.models.get(path).map(|m| m.bounds().size());
        }
        if is_point_cloud_path(path) {
            self.point_cloud_content_for(path, comp).ok()?;
            return self.point_clouds.get(path).map(|c| c.bounds().size());
        }
        if !self.probes.contains_key(path) && !self.failed_probes.contains_key(path) {
            match probe(path) {
                Ok(info) => { self.probes.insert(path.to_owned(), info); }
                Err(err) => { self.failed_probes.insert(path.to_owned(), err.to_string()); }
            }
        }
        self.probes.get(path).map(|info| [info.width as f32, info.height as f32, 0.0])
    }

    pub(in crate::engine) fn mesh_content_for(
        &mut self,
        path: &str,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let fallback = [comp.width as f32, comp.height as f32];
        if let Some(model) = self.models.get(path) {
            return Ok((
                Some(LayerContent::Model(model.clone())),
                model.bounds().size_xy(),
            ));
        }
        if let Some(reason) = self.failed_meshes.get(path) {
            self.layer_failures.push(reason.clone());
            return Ok((None, fallback));
        }
        match self.compositor.import_model(path) {
            Ok(model) => {
                let model = std::sync::Arc::new(model);
                let natural = model.bounds().size_xy();
                self.models.insert(path.to_owned(), model.clone());
                Ok((Some(LayerContent::Model(model)), natural))
            }
            Err(err) => {
                let reason = format!("3D素材を読めない: {path}: {err}");
                self.failed_meshes.insert(path.to_owned(), reason.clone());
                self.layer_failures.push(reason);
                Ok((None, fallback))
            }
        }
    }

    fn point_cloud_content_for(
        &mut self,
        path: &str,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let fallback = [comp.width as f32, comp.height as f32];

        let data = match self.point_clouds.get(path) {
            Some(data) => Some(data.clone()),
            None => match self.failed_point_clouds.get(path) {
                Some(reason) => {
                    self.layer_failures.push(reason.clone());
                    None
                }
                None => match load_point_cloud(std::path::Path::new(path)) {
                    Ok(data) => {
                        self.point_clouds.insert(path.to_owned(), data.clone());
                        Some(data)
                    }
                    Err(err) => {
                        let reason = format!("点群を読めない: {path}: {err}");
                        self.failed_point_clouds
                            .insert(path.to_owned(), reason.clone());
                        self.layer_failures.push(reason);
                        None
                    }
                },
            },
        };
        let Some(data) = data else {
            return Ok((None, fallback));
        };
        if data.positions.is_empty() {
            return Ok((None, fallback));
        }
        let natural = data.bounds().size_xy();

        // 焼かない(裁定 2026-08-30)。点のまま run の view へ渡り、深度で板と刺さり合う。
        Ok((
            Some(LayerContent::Cloud {
                positions: data.positions.clone(),
                colors: data.colors.clone(),
                bounds: data.bounds(),
                point_size: DEFAULT_POINT_SIZE,
                sizes: None,
                sprites: false,
                links: None,
            }),
            natural,
        ))
    }

    /// ファイル素材の道はここ1本。**種別で分かれるのはこの関数の中だけ**で、
    /// 呼ぶ側は素材が何かを知らない。種別を足す時に触るのもここだけ。
    fn file_content_for(
        &mut self,
        path: &str,
        source_time: RationalTime,
        layer: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if crate::render::media::is_mesh_path(path) {
            self.mesh_content_for(path, comp)
        } else if is_point_cloud_path(path) {
            self.point_cloud_content_for(path, comp)
        } else if crate::render::media::is_still_image_path(path) {
            self.still_texture_for(path)
        } else if crate::render::media::is_audio_path(path) {
            // 音だけの素材は絵を持たない。probe に回すと「no video stream」が層の失敗になり再生が止まる。
            Ok((None, [0.0, 0.0]))
        } else {
            self.media_texture_for(path, source_time, layer)
        }
    }

    /// 環境にした画。線形の放射輝度で上げ、照度図は上げる時に畳む。
    /// 失敗は普通の画と同じ棚(`failed_probes`)に置く。
    fn environment_content_for(
        &mut self,
        path: &str,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if let Some(env) = self.environments.get(path) {
            return Ok((Some(LayerContent::Environment(env.clone())), env.size));
        }
        if let Some(reason) = self.failed_probes.get(path) {
            self.layer_failures.push(reason.clone());
            return Ok((None, [0.0, 0.0]));
        }
        let made = decode_still_linear_rgb(path)
            .and_then(|(rgb, w, h)| {
                self.compositor
                    .upload_environment("environment", &rgb, w, h)
                    .map_err(|e| e.to_string())
            });
        match made {
            Ok(env) => {
                self.environments.insert(path.to_owned(), env.clone());
                Ok((Some(LayerContent::Environment(env.clone())), env.size))
            }
            Err(err) => {
                let reason = format!("環境を読めない: {path}: {err}");
                self.failed_probes.insert(path.to_owned(), reason.clone());
                self.layer_failures.push(reason);
                Ok((None, [0.0, 0.0]))
            }
        }
    }

    /// 静止画は1枚を焼いて置くだけ。動画の道へ入れると
    /// `load_from_bytes(.., "video/mp4", ..)` が必ず落ちて**絵が出ない**。
    ///
    /// 覚えるのは texture_manager に任せる。当たれば読み込みも起きない。
    fn still_texture_for(
        &mut self,
        path: &str,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if let Some(reason) = self.failed_probes.get(path) {
            self.layer_failures.push(reason.clone());
            return Ok((None, [0.0, 0.0]));
        }
        // 素材の素性は動画と同じ棚へ置く。選択枠の寸法はそこから引かれるので、
        // 置かないと**選んでも枠が出ず、選べていないように見える**。
        if !self.probes.contains_key(path) {
            if let Ok(info) = probe(path) {
                self.probes.insert(path.to_owned(), info);
            }
        }
        // 画素は長く覚え、テクスチャはフレーム単位。寿命の粒度が違うので層を分ける。
        let pixels = &self.pixels;
        let made = self.compositor.cached_rgba(still_key(path), "still", || {
            let image = pixels.get_or_insert_with(path, || {
                let (premultiplied_rgba, width, height) = decode_still_srgb(path)?;
                Ok::<_, String>(std::sync::Arc::new(crate::render::engine::StillImage {
                    premultiplied_rgba,
                    width,
                    height,
                }))
            })?;
            Ok::<_, String>((image.premultiplied_rgba.clone(), image.width, image.height))
        });
        match made {
            Ok(texture) => {
                let [w, h] = texture.width_height();
                Ok((Some(LayerContent::Texture(texture)), [w as f32, h as f32]))
            }
            Err(err) => {
                let reason = format!("素材を読めない(画像decode失敗): {path}: {err}");
                self.failed_probes.insert(path.to_owned(), reason.clone());
                self.layer_failures.push(reason);
                Ok((None, [0.0, 0.0]))
            }
        }
    }
}
