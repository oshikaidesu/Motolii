//! 層の種類ごとの絵 — どの素材がどの内容になるかの振り分けと、種類ごとの焼き方。

use super::*;

impl Engine {

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

    /// The layer's cached vector picture, when `shapes` is the value it was drawn from and the
    /// cut is at least as fine as `tolerance` (`None`: any cut). No outline work on a hit.
    pub(in crate::engine) fn cached_shape(&self, layer: LayerId, on_comp: bool, shapes: &std::sync::Arc<Vec<ShapeNode>>, vector: bool, step: Option<f32>, tolerance: Option<f32>) -> Option<&crate::render::engine::texture::ShapeTexture> {
        self.shape_textures.get(&ShapeCacheKey { layer, on_comp })
            .filter(|c| std::sync::Arc::ptr_eq(&c.shapes, shapes) && c.vector == vector && c.step == step && tolerance.is_none_or(|t| c.tolerance <= t))
    }

    pub(in crate::engine) fn shape_texture_from_shapes(
        &mut self,
        shapes: &std::sync::Arc<Vec<ShapeNode>>,
        layer_id: LayerId,
        vector: bool,
        tolerance: f32,
        comp: CompSpec,
        step: Option<f32>,
        remember: bool,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if let Some(hit) = self.cached_shape(layer_id, false, shapes, vector, step, Some(tolerance)) {
            return Ok((Some(hit.texture.clone()), hit.natural));
        }
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }
        let Some(canvas) = content_canvas(shapes)? else {
            return Ok((None, [0.0, 0.0]));
        };
        self.shape_texture_on_canvas(shapes, layer_id, false, vector, tolerance, comp, step, remember, canvas)
    }

    pub(in crate::engine) fn text_texture_from_shapes(
        &mut self,
        shapes: &std::sync::Arc<Vec<ShapeNode>>,
        layer_id: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if let Some(hit) = self.cached_shape(layer_id, true, shapes, true, None, None) {
            return Ok((Some(hit.texture.clone()), hit.natural));
        }
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }
        let canvas = crate::picture::shapes_ops::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };
        self.shape_texture_on_canvas(shapes, layer_id, true, true, 0.05, comp, None, true, canvas)
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_texture_on_canvas(
        &mut self,
        shapes: &std::sync::Arc<Vec<ShapeNode>>,
        layer_id: LayerId,
        on_comp: bool,
        vector: bool,
        tolerance: f32,
        comp: CompSpec,
        step: Option<f32>,
        remember: bool,
        canvas: crate::picture::shapes_ops::Canvas,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let content = if vector {
            self.compositor.path_model(shapes, &canvas, tolerance, step)?.map(|m| LayerContent::Model(std::sync::Arc::new(m)))
        } else { self.compositor.render_paths("shape", shapes, &canvas, 0.05 / tolerance, raster_pixel_budget(comp))?.map(LayerContent::Texture) };
        let Some(texture) = content else {
            return Ok((None, [0.0, 0.0]));
        };
        let natural = [canvas.width as f32, canvas.height as f32];
        if remember {
            self.shape_textures.insert(
                ShapeCacheKey { layer: layer_id, on_comp },
                crate::render::engine::texture::ShapeTexture { shapes: shapes.clone(), texture: texture.clone(), natural, vector, tolerance, step },
            );
        }
        Ok((Some(texture), natural))
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
    pub(in crate::engine) fn file_content_for(
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
    pub(in crate::engine) fn environment_content_for(
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
    pub(in crate::engine) fn still_texture_for(
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
