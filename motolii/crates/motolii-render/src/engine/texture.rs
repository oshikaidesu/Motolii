use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::doc::core::CompSpec;
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};
use crate::render::compositor::LayerContent;
use crate::render::media::{is_point_cloud_path, load_point_cloud, probe};

use crate::render::engine::render::layer_size;
use crate::render::engine::{text, Engine, EngineError};

/// 復号した動画のコマの cache の上限。1 GiB。RAM で持つ(Apple Silicon では GPU メモリ = RAM)。
pub(super) const FRAME_CACHE_BUDGET: u64 = 1 << 30;

pub(super) struct CachedVideoFrame {
    pub(super) texture: crate::render::compositor::GpuTexture2D,
    pub(super) bytes: u64,
    pub(super) last_use: u64,
}

/// コマ1つを待つ上限と、見に行く間隔。越えたら諦め、諦めた事を記録する。
const DECODE_PATIENCE: std::time::Duration = std::time::Duration::from_secs(2);
const DECODE_POLL: std::time::Duration = std::time::Duration::from_millis(2);

fn layer_stream_id(layer: LayerId, path: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layer.0.hash(&mut hasher);
    path.hash(&mut hasher);
    hasher.finish()
}

/// 静止画は道で1枚。層をまたいで同じ絵を二度焼かない。
fn still_key(path: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "still".hash(&mut hasher);
    path.hash(&mut hasher);
    hasher.finish()
}

/// 文字の texture を憶える上限(枚)。
const TEXT_CACHE_LIMIT: usize = 256;

/// tiny-skia の乗算済み RGBA を非乗算へ(上げる直前に 1 回)。透明な texel は隣の色で埋める
/// (edge bleed)— 非乗算は sampler の線形補間と効果の畳み込みが透明部の RGB を混ぜるので、
/// 黒のままだと縁が灰・暈が黒になる(QA 再点検 Q2-1)。

/// 透明(a=0)の texel に、色を持つ 4 近傍の平均を書く。2 回回して 2px ぶん広げる。
fn bleed_edges(rgba: &mut [u8], width: usize) {
    if width == 0 || rgba.len() < 4 {
        return;
    }
    let height = rgba.len() / 4 / width;
    let mut colored: Vec<bool> = rgba.chunks_exact(4).map(|p| p[3] != 0).collect();
    for _ in 0..2 {
        let snapshot = rgba.to_vec();
        let mut next = colored.clone();
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                if colored[i] {
                    continue;
                }
                let mut sum = [0u32; 3];
                let mut n = 0u32;
                let mut take = |j: usize| {
                    if colored[j] {
                        for c in 0..3 {
                            sum[c] += snapshot[j * 4 + c] as u32;
                        }
                        n += 1;
                    }
                };
                if x > 0 { take(i - 1); }
                if x + 1 < width { take(i + 1); }
                if y > 0 { take(i - width); }
                if y + 1 < height { take(i + width); }
                if n > 0 {
                    for c in 0..3 {
                        rgba[i * 4 + c] = (sum[c] / n) as u8;
                    }
                    next[i] = true;
                }
            }
        }
        colored = next;
    }
}

pub(super) struct TextTexture {
    texture: LayerContent,
    tolerance: f32,
    frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
    bounds: Option<crate::render::media::SpatialBounds>,
}


fn group_local_bounds(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    group: LayerId,
    mut leaf_bounds: impl FnMut(&ResolvedLayer) -> Option<crate::render::media::SpatialBounds>,
) -> Option<crate::render::media::SpatialBounds> {
    let group_world = resolved.iter().find(|layer| layer.id == group && !layer.ghost)?.placement.world_transform?;
    if !group_world.is_finite() || group_world.matrix3.determinant() == 0.0 { return None; }
    let local_from_world = group_world.inverse();
    if !local_from_world.is_finite() { return None; }
    let parents: HashMap<_, _> = view.layers().into_iter().filter_map(|layer| {
        Some((layer, view.attrs(layer).ok().flatten()?.parent?))
    }).collect();
    let matte_sources: std::collections::HashSet<_> = resolved.iter().filter(|layer| !layer.clip_to_below).filter_map(|layer| layer.matte.map(|matte| matte.layer)).collect();
    let mut points = Vec::new();
    for leaf in resolved {
        if matches!(leaf.source, LayerSource::Camera | LayerSource::Stage | LayerSource::Group | LayerSource::Null) || leaf.placement.opacity <= 0.0 || matte_sources.contains(&leaf.id) { continue; }
        let mut parent = parents.get(&leaf.id).copied();
        let mut visited = std::collections::HashSet::new();
        let mut descendant = false;
        while let Some(id) = parent {
            if id == group { descendant = true; break; }
            if !visited.insert(id) { break; }
            parent = parents.get(&id).copied();
        }
        if !descendant { continue; }
        let Some(bounds) = leaf_bounds(leaf) else { continue; };
        let world = leaf.placement.world_transform?;
        let local = local_from_world * world;
        for x in [bounds.min[0], bounds.max[0]] {
            for y in [bounds.min[1], bounds.max[1]] {
                for z in [bounds.min[2], bounds.max[2]] {
                    points.push(local.transform_point3(glam::vec3(x, y, z)).to_array());
                }
            }
        }
    }
    crate::render::media::SpatialBounds::from_points(points).ok()
}

impl Engine {
    pub fn selected_layer_size(
        &self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        let resolved = view.resolved_layers(t).ok()?;
        self.selected_layer_size_in(view, &resolved, layer_id, t)
    }

    /// 解いた層の一覧を持っている側(Stage の paint)は、層ごとに解き直さない。
    pub fn selected_layer_size_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[crate::doc::store::ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        self.selected_layer_bounds_in(view, resolved, layer_id, t).map(|bounds| bounds.size_xy())
    }

    /// Camera 層 `id` の姿勢。層ターゲットは Stage の枠・Depth の点と同じ bounds の中心を見る(Document だけなら anchor)。
    pub fn camera_of_layer_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        id: LayerId,
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        let store = |e: crate::doc::store::StoreError| crate::render::engine::EngineError::Store(e.to_string());
        let mut camera = view.camera_of_layer(id, t).map_err(store)?;
        let Some(target) = view.camera_target_layer(id, t).map_err(store)? else { return Ok(camera) };
        let (Some(bounds), Some(comp)) = (self.selected_layer_bounds_in(view, resolved, target, t), view.composition().map_err(store)?) else { return Ok(camera) };
        let comp = comp.spec();
        let point = view.world_transform3d(target, t).map_err(store)?.transform_point3(glam::Vec3::from(bounds.center()));
        camera.center = [point.x - comp.width as f32 * 0.5, point.y - comp.height as f32 * 0.5];
        camera.target_z = point.z;
        Ok(camera)
    }

    /// 作中カメラ。描画・Stage・Depth はすべてこれを通す。
    pub fn resolve_camera_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        let store = |e: crate::doc::store::StoreError| crate::render::engine::EngineError::Store(e.to_string());
        match view.active_camera_layer(t).map_err(store)? {
            Some(id) => self.camera_of_layer_in(view, resolved, id, t),
            None => view.resolve_camera(t).map_err(store),
        }
    }

    /// 解決済みの層の並びが手元に無い時。層ターゲットがある時だけ層を解く。
    pub fn resolve_camera(
        &self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<crate::doc::core::ResolvedCamera, crate::render::engine::EngineError> {
        let store = |e: crate::doc::store::StoreError| crate::render::engine::EngineError::Store(e.to_string());
        let Some(id) = view.active_camera_layer(t).map_err(store)? else { return view.resolve_camera(t).map_err(store) };
        if view.camera_target_layer(id, t).map_err(store)?.is_none() { return view.camera_of_layer(id, t).map_err(store) }
        let resolved = view.resolved_layers(t).map_err(store)?;
        self.camera_of_layer_in(view, &resolved, id, t)
    }

    pub fn selected_layer_bounds_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<crate::render::media::SpatialBounds> {
        let layer = resolved.iter().find(|layer| layer.id == layer_id && !layer.ghost)?;
        if layer.source == LayerSource::Group {
            return group_local_bounds(view, resolved, layer_id, |leaf| self.leaf_local_bounds(view, leaf, t));
        }
        self.leaf_local_bounds(view, layer, t)
    }

    /// The points the layer actually draws, in the same local space as `selected_layer_bounds_in`:
    /// every vertex of a mesh or point cloud, the four corners of a flat picture. `None` for groups
    /// and for anything else the box already describes.
    pub fn selected_layer_outline_in(
        &self,
        view: &StoreView<'_>,
        resolved: &[ResolvedLayer],
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<Vec<glam::Vec3>> {
        let layer = resolved.iter().find(|layer| layer.id == layer_id && !layer.ghost)?;
        let bounds = self.leaf_local_bounds(view, layer, t)?;
        let spatial = match &layer.source {
            LayerSource::File { path, .. } if crate::render::media::is_mesh_path(path) => {
                let model = self.models.get(path)?;
                Some((model.bounds(), model.vertices.iter().copied().collect::<Vec<_>>()))
            }
            LayerSource::File { path, .. } if is_point_cloud_path(path) => {
                let cloud = self.point_clouds.get(path)?;
                Some((cloud.bounds(), cloud.silhouette.iter().copied().collect()))
            }
            _ => None,
        };
        Some(match spatial {
            // Drawn as `depth_scaled(world) * (v - origin)`: the same origin the box uses.
            Some((model, vertices)) => {
                let origin = glam::vec3(model.min[0], model.min[1], (model.min[2] + model.max[2]) * 0.5);
                vertices.into_iter().map(|v| v - origin).collect()
            }
            None => [[0, 0], [1, 0], [1, 1], [0, 1]].into_iter().map(|[x, y]| glam::vec3(
                if x == 0 { bounds.min[0] } else { bounds.max[0] },
                if y == 0 { bounds.min[1] } else { bounds.max[1] },
                0.0,
            )).collect(),
        })
    }

    fn leaf_local_bounds(
        &self,
        view: &StoreView<'_>,
        layer: &ResolvedLayer,
        t: RationalTime,
    ) -> Option<crate::render::media::SpatialBounds> {
        use crate::render::media::SpatialBounds;
        let layer_id = layer.id;
        let comp = view.composition().ok().flatten()?.spec();
        let planar = |bounds: SpatialBounds, natural: [f32; 2]| {
            let displayed = layer_size(layer, natural);
            let scale = [displayed[0] / natural[0], displayed[1] / natural[1]];
            SpatialBounds::from_points([
                [bounds.min[0] * scale[0], bounds.min[1] * scale[1], 0.0],
                [bounds.max[0] * scale[0], bounds.max[1] * scale[1], 0.0],
            ]).ok()
        };
        match &layer.source {
            LayerSource::Text => {
                let document = view.resolved_text_document(layer_id, t).ok().flatten()?;
                let key = TextCacheKey::new(layer_id, &document, t, comp.width, comp.height);
                let cached = self.text_textures.get(&key)?;
                planar(cached.bounds?, [comp.width as f32, comp.height as f32])
            }
            LayerSource::Shape => {
                let shapes = super::render::shown_shapes(&view.shapes_at(layer_id, t).ok()?, layer);
                let canvas = content_canvas(&shapes).ok().flatten()?;
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                let _cached = self.shape_textures.get(&key)?;
                let natural = [canvas.width as f32, canvas.height as f32];
                let bounds = crate::doc::vector::content_bounds(&shapes).ok().flatten()?;
                planar(SpatialBounds {
                    min: [bounds[0] as f32 + canvas.origin_x as f32, bounds[1] as f32 + canvas.origin_y as f32, 0.0],
                    max: [bounds[2] as f32 + canvas.origin_x as f32, bounds[3] as f32 + canvas.origin_y as f32, 0.0],
                }, natural)
            }
            LayerSource::Camera | LayerSource::Stage | LayerSource::Null | LayerSource::Group => None,
            LayerSource::File { path, .. } => {
                let spatial = if crate::render::media::is_mesh_path(path) {
                    Some(self.models.get(path)?.bounds())
                } else if is_point_cloud_path(path) {
                    Some(self.point_clouds.get(path)?.bounds())
                } else { None };
                if let Some(bounds) = spatial {
                    let [x, y, z] = bounds.size();
                    Some(SpatialBounds { min: [0.0, 0.0, -z * 0.5], max: [x, y, z * 0.5] })
                } else {
                    let info = self.probes.get(path)?;
                    let natural = [info.width as f32, info.height as f32];
                    planar(SpatialBounds { min: [0.0; 3], max: [natural[0], natural[1], 0.0] }, natural)
                }
            }
        }
    }

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
        let needs_material = self.compositor.catalog.descriptors.iter().any(|d| matches!(d.stage, crate::render::compositor::EffectStage::Warp | crate::render::compositor::EffectStage::Field) && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        // 絵を読む効果(pass)は素材座標の絵を要る。comp 大に焼くと comp の外が失われ、
        // Blur が縁で切れる(広がりの法: 評価の入力を view・comp・カメラで切らない)。
        let needs_image = !super::translate::translate_effect_passes(&layer.effects).is_empty();
        let vector = layer.depth == 0.0 && layer.masks.is_empty() && !needs_material && !needs_image;
        let natural = if layer.source == LayerSource::Shape {
            let canvas = content_canvas(shape_documents.get(&layer.id).map(Vec::as_slice).unwrap_or(&[]))?;
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
            let reach = super::translate::translate_effect_passes(&layer.effects).iter().map(|p| p.padding() as f32).fold(0.0f32, f32::max);
            let limit = self.compositor.ctx.device.limits().max_texture_dimension_2d as f32;
            let extent = natural[0].max(natural[1]).max(1.0) + 2.0 * reach;
            exact_density = exact_density.min((limit / extent).max(1.0));
        }
        let density = (density * (1.0 - 1e-4)).log2().ceil().exp2().max(1.0);
        // 輪郭の細分は 2 の冪の段で cache を使い回す。絵に描く時は投影の密度そのもので描く
        // (段に丸めると置いた時に再標本化され、縁が甘くなる)。
        let tolerance = (0.05 / if vector { density } else { exact_density }).max(1e-6);
        let (content, natural, frame) = if layer.source == LayerSource::Text {
            self.text_texture_from_document(text_documents.get(&layer.id), layer.id, t, comp, vector, tolerance, layer.depth == 0.0)?
        } else if layer.source == LayerSource::Shape {
            let shapes = shape_documents
                .get(&layer.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let (content,natural)=self.shape_texture_from_shapes(shapes, layer.id, vector, tolerance, comp)?;
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
            Some(LayerContent::Texture(texture)) if layer.depth > 0.0 && layer.projection != crate::doc::store::LayerProjection::TwoD && layer.masks.is_empty() => {
                Ok((self.extruded_content(layer, texture, natural, text_documents, shape_documents, t, comp)?, natural, frame))
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
    ) -> Result<Option<LayerContent>, EngineError> {
        let key = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            texture.handle.hash(&mut hasher);
            layer.depth.to_bits().hash(&mut hasher);
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
                let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
                match text_documents.get(&layer.id).and_then(|d| text::text_shapes(d, t, &canvas).ok().flatten()) {
                    Some(shapes) => crate::render::compositor::paths::outlines(&shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            LayerSource::Shape => {
                let shapes = shape_documents.get(&layer.id).map(Vec::as_slice).unwrap_or(&[]);
                match content_canvas(shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            _ => rectangle(natural),
        };
        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, layer.depth)? else {
            return Ok(Some(LayerContent::Texture(texture)));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(layer.id, (key, model.clone()));
        Ok(Some(LayerContent::Model(model)))
    }

    fn text_texture_from_document(
        &mut self,
        document: Option<&TextDocument>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
        vector: bool,
        tolerance: f32,
        crop: bool,
    ) -> Result<(Option<LayerContent>, [f32; 2], Option<crate::render::compositor::effects::vism::ImageFrame>), EngineError> {
        let Some(document) = document else {
            return Ok((None, [0.0, 0.0], None));
        };

        let canvas = crate::doc::vector::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };

        let key = TextCacheKey::new(layer_id, document, t, canvas.width, canvas.height);
        if let Some(cached) = self.text_textures.get(&key).filter(|c| matches!(c.texture, LayerContent::Model(_)) == vector && c.tolerance <= tolerance) {
            return Ok((
                Some(cached.texture.clone()),
                [canvas.width as f32, canvas.height as f32],
                cached.frame,
            ));
        }

        let Some(shapes) = text::text_shapes(document, t, &canvas)? else {
            return Ok((None, [0.0, 0.0], None));
        };
        let bounds = crate::doc::vector::content_bounds(&shapes)?.map(|b| crate::render::media::SpatialBounds {
            min: [b[0] as f32, b[1] as f32, 0.0],
            max: [b[2] as f32, b[3] as f32, 0.0],
        });
        let raster_canvas = if !vector && crop { content_canvas(&shapes)?.unwrap_or_else(|| canvas.clone()) } else { canvas.clone() };
        let content = if vector {
            self.compositor.path_model(&shapes, &canvas, tolerance)?.map(|m| LayerContent::Model(std::sync::Arc::new(m)))
        } else { self.compositor.render_paths("text", &shapes, &raster_canvas, 0.05 / tolerance, raster_pixel_budget(comp))?.map(LayerContent::Texture) };
        let Some(texture) = content else {
            return Ok((None, [0.0, 0.0], None));
        };
        let frame = texture.texture().map(|t| crate::render::compositor::effects::vism::ImageFrame {
            size: [raster_canvas.width as f32,raster_canvas.height as f32],
            origin: [-(raster_canvas.origin_x as f32),-(raster_canvas.origin_y as f32)], pixels: t.width_height(),
        });
        let fresh = !self.text_textures.contains_key(&key);
        self.text_textures.insert(key.clone(), TextTexture { texture: texture.clone(), bounds, tolerance, frame });
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

    fn shape_texture_from_shapes(
        &mut self,
        shapes: &[ShapeNode],
        layer_id: LayerId,
        vector: bool,
        tolerance: f32,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }

        let Some(canvas) = content_canvas(shapes)? else {
            return Ok((None, [0.0, 0.0]));
        };

        let key = ShapeCacheKey::new(layer_id, shapes, canvas.width, canvas.height);
        if let Some(cached) = self.shape_textures.get(&key).filter(|c| matches!(c.texture, LayerContent::Model(_)) == vector && c.tolerance <= tolerance) {
            return Ok((
                Some(cached.texture.clone()),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let content = if vector {
            self.compositor.path_model(shapes, &canvas, tolerance)?.map(|m| LayerContent::Model(std::sync::Arc::new(m)))
        } else { self.compositor.render_paths("shape", shapes, &canvas, 0.05 / tolerance, raster_pixel_budget(comp))?.map(LayerContent::Texture) };
        let Some(texture) = content else {
            return Ok((None, [0.0, 0.0]));
        };
        self.shape_textures.insert(key, TextTexture { texture: texture.clone(), bounds: None, tolerance, frame: None });
        Ok((
            Some(texture),
            [canvas.width as f32, canvas.height as f32],
        ))
    }

    /// 網も焼かない。三角形のまま run の view へ渡り、深度で板と刺さり合う。
    fn mesh_content_for(
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

    pub(super) fn media_texture_for(
        &mut self,
        path: &str,
        source_time: RationalTime,
        layer: LayerId,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let info = match self.probes.get(path) {
            Some(info) => Some(info.clone()),
            None => match self.failed_probes.get(path) {
                Some(reason) => {
                    self.layer_failures.push(reason.clone());
                    None
                }
                None => match probe(path) {
                    Ok(info) => {
                        self.probes.insert(path.to_owned(), info.clone());
                        Some(info)
                    }
                    Err(err) => {
                        let reason = format!("素材を読めない(probe失敗): {path}: {err}");
                        self.failed_probes.insert(path.to_owned(), reason.clone());
                        self.layer_failures.push(reason);
                        None
                    }
                },
            },
        };
        let Some(info) = info else {
            return Ok((None, [0.0, 0.0]));
        };
        let natural = [info.width as f32, info.height as f32];

        let end = info.duration.or_else(|| {
            info.nb_frames
                .and_then(|n| RationalTime::try_from_frame(n, info.fps).ok())
        });
        if source_time < RationalTime::ZERO || end.is_some_and(|end| source_time >= end) {
            return Ok((None, natural));
        }

        if !self.videos.contains_key(path) {
            let bytes = match std::fs::File::open(path)
                .and_then(|file| unsafe { memmap2::Mmap::map(&file) })
            {
                Ok(map) => map,
                Err(err) => {
                    self.layer_failures
                        .push(format!("素材を読めない(read失敗): {path}: {err}"));
                    return Ok((None, natural));
                }
            };
            let descr =
                match re_video::VideoDataDescription::load_from_bytes(&bytes, "video/mp4", path) {
                    Ok(descr) => descr,
                    Err(err) => {
                        self.layer_failures
                            .push(format!("動画を読めない(decode失敗): {path}: {err}"));
                        return Ok((None, natural));
                    }
                };
            // 素材の色をそのまま受け、GPU で RGB にする。ffmpeg に変換させると CPU の色変換が
            // 挟まり、4K で復号の 6 倍遅い(実測 M4)。probe が bt709/bt601 以外を門で断るので、
            // ここに来る素材は必ずどちらか。
            let source_yuv = match info.color_space {
                crate::doc::core::ColorSpace::Rec709Full => Some((re_video::YuvRange::Full, re_video::YuvMatrixCoefficients::Bt709)),
                crate::doc::core::ColorSpace::Rec601Limited => Some((re_video::YuvRange::Limited, re_video::YuvMatrixCoefficients::Bt601)),
                crate::doc::core::ColorSpace::Rec709Limited => Some((re_video::YuvRange::Limited, re_video::YuvMatrixCoefficients::Bt709)),
                crate::doc::core::ColorSpace::Srgb | crate::doc::core::ColorSpace::LinearRgb => None,
            };
            let video = re_renderer::video::Video::load(
                path.to_owned(),
                descr,
                re_video::DecodeSettings { source_yuv, in_process: true, ..Default::default() },
            );
            self.videos.insert(path.to_owned(), (bytes, video));
        }
        let (bytes, video) = self.videos.get(path).expect("直前に insert した");

        let Some(timescale) = video.data_descr().timescale else {
            self.layer_failures
                .push(format!("動画にタイムスケールが無い: {path}"));
            return Ok((None, natural));
        };
        // `re_video::Time::from_secs` が秒の f64 を要求するので、最後に一度だけ f64 にする。
        let video_time = re_video::Time::from_secs(source_time.as_seconds_f64(), timescale);
        // 頼むコマの pts。同じコマを二度復号しない鍵。
        let sample_pts = video
            .data_descr()
            .latest_sample_index_at_presentation_timestamp(video_time)
            .ok()
            .and_then(|idx| video.data_descr().samples.get(idx).and_then(|s| s.sample()))
            .map(|sample| sample.presentation_timestamp.0);
        if let Some(pts) = sample_pts {
            let tick = self.frame_cache_tick;
            if let Some(hit) = self.frame_cache.get_mut(&(path.to_owned(), pts)) {
                hit.last_use = tick;
                self.frame_cache_hits += 1;
                return Ok((Some(LayerContent::Texture(hit.texture.clone())), natural));
            }
        }
        let (bytes, video) = self.videos.get(path).expect("直前に insert した");
        let stream_id = re_video::player::VideoPlayerStreamId(layer_stream_id(layer, path));
        let source = re_video::player::VideoSliceSource(&bytes[..]);
        // デコーダは非同期で、頼んだ直後は返さない。待たずに前のコマを
        // 返すと、**同じ時刻でも辿り着き方で絵が変わり**、窓と書き出しが
        // 一致しなくなる。待つのは素材ごとに初回だけ(実測 764ms、以降 65µs)。
        let deadline = std::time::Instant::now() + DECODE_PATIENCE;
        // (出す texture, cache に入れる pts, 失敗)。頼んだコマが揃った時だけ pts が付く。
        let (texture, fresh_pts, failure) = loop {
            let output = video.frame_at(
                self.compositor.render_context(),
                stream_id,
                video_time,
                &source,
            );
            let ready = output.output.as_ref().is_some_and(|frame| {
                frame.texture.is_some()
                    && matches!(
                        frame.decoder_delay_state,
                        re_video::player::DecoderDelayState::UpToDate
                    )
            });
            if ready || self.realtime {
                let fresh = ready
                    .then(|| output.output.as_ref().and_then(|frame| frame.frame_info.as_ref()).map(|info| info.presentation_timestamp.0))
                    .flatten();
                break (output.output.and_then(|frame| frame.texture), fresh, None);
            }
            if let Some(err) = output.error {
                break (None, None, Some(format!("フレームを読めない(decode失敗): {path} at={source_time:?}: {err}")));
            }
            if std::time::Instant::now() >= deadline {
                break (None, None, Some(format!("コマが間に合わなかった: {path} at={source_time:?}")));
            }
            std::thread::sleep(DECODE_POLL);
        };
        if let Some(failure) = failure {
            self.layer_failures.push(failure);
            return Ok((None, natural));
        }
        if let (Some(texture), Some(pts)) = (&texture, fresh_pts) {
            let key = (path.to_owned(), pts);
            if !self.frame_cache.contains_key(&key) && !self.pending_frame_copies.iter().any(|(p, t, _)| (p, *t) == (&key.0, key.1)) {
                self.pending_frame_copies.push((key.0, key.1, texture.clone()));
            }
        }
        Ok((texture.map(LayerContent::Texture), natural))
    }

    /// 前の frame で揃ったコマを cache に写す。frame の頭、復号器が texture を書き換える前に呼ぶ。
    pub(super) fn flush_pending_frame_copies(&mut self) {
        for (path, pts, texture) in std::mem::take(&mut self.pending_frame_copies) {
            self.remember_video_frame(&path, pts, &texture);
        }
    }

    /// 復号したコマを GPU texture のまま写して取っておく。上限を超えたら、使われてから一番古い物を捨てる。
    fn remember_video_frame(&mut self, path: &str, pts: i64, source: &crate::render::compositor::GpuTexture2D) {
        let [width, height] = source.width_height();
        let bytes = u64::from(width) * u64::from(height) * 4;
        if bytes == 0 || bytes > self.frame_cache_budget {
            return;
        }
        while self.frame_cache_bytes + bytes > self.frame_cache_budget {
            let Some(oldest) = self.frame_cache.iter().min_by_key(|(_, f)| f.last_use).map(|(k, _)| k.clone()) else { break };
            if let Some(gone) = self.frame_cache.remove(&oldest) {
                self.frame_cache_bytes -= gone.bytes;
            }
        }
        // 動画の texture は不透明。pool から同じ形を確保して写し、`Opaque` のまま包む
        // (α ありとして輸入すると α 0 で消える)。
        let texture = {
            let ctx = self.compositor.render_context();
            let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
            let copy = ctx.gpu_resources.textures.alloc(
                &ctx.device,
                &re_renderer::TextureDesc {
                    label: "Motolii video frame cache".into(),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: source.format(),
                    usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
                },
            );
            let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Motolii video frame cache copy") });
            encoder.copy_texture_to_texture(source.texture.as_image_copy(), copy.texture.as_image_copy(), size);
            ctx.queue.submit([encoder.finish()]);
            re_renderer::resource_managers::GpuTexture2D::new(copy, re_renderer::resource_managers::AlphaChannelUsage::Opaque)
        };
        let Some(texture) = texture else { return };
        let tick = self.frame_cache_tick;
        self.frame_cache.insert((path.to_owned(), pts), CachedVideoFrame { texture, bytes, last_use: tick });
        self.frame_cache_bytes += bytes;
    }

    pub(crate) fn texture_for(
        &mut self,
        source: &LayerSource,
        _source_frame: i64,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        match source {
            LayerSource::Text | LayerSource::Shape | LayerSource::File { .. } => {
                Ok((None, [0.0, 0.0]))
            }
            LayerSource::Camera | LayerSource::Stage | LayerSource::Null | LayerSource::Group => Ok((None, [0.0, 0.0])),
        }
    }
}

/// 静止画を**非乗算 sRGB** の RGBA8 に揃える。乗算は shader(decode の後)。
///
/// ファイルに ICC(iPhone の Display P3、カメラの Adobe RGB)が埋まっていれば
/// sRGB へ写す。Finder / Preview と同じ見え方になる。profile が壊れていて
/// 読めない時は Preview と同じく無視して sRGB 扱い。
pub fn decode_still_srgb(path: &str) -> Result<(Vec<u8>, u32, u32), String> {
    use image::ImageDecoder as _;
    let mut decoder = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .into_decoder()
        .map_err(|e| e.to_string())?;
    let icc = decoder.icc_profile().ok().flatten();
    let decoded = image::DynamicImage::from_decoder(decoder)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    let (width, height) = decoded.dimensions();
    let mut rgba = decoded.into_raw();
    if let Some(transform) = icc.as_deref().and_then(icc_to_srgb_rgba8) {
        let mut out = vec![0u8; rgba.len()];
        transform
            .transform(&rgba, &mut out)
            .map_err(|e| format!("ICC transform failed: {e:?}"))?;
        rgba = out;
    }
    Ok((rgba, width, height))
}

/// 環境用: 線形の RGB f32(`width * height * 3`)。float の画(hdr)はそのまま、
/// 8bit の画は sRGB を線形へ戻す。ICC は写さない(環境の画に付く事が稀)。
pub fn decode_still_linear_rgb(path: &str) -> Result<(Vec<f32>, u32, u32), String> {
    let decoded = image::ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let (width, height) = (decoded.width(), decoded.height());
    let is_float = matches!(
        decoded,
        image::DynamicImage::ImageRgb32F(_) | image::DynamicImage::ImageRgba32F(_)
    );
    let rgb = decoded.into_rgb32f().into_raw();
    let rgb = if is_float {
        rgb
    } else {
        rgb.into_iter().map(srgb_to_linear).collect()
    };
    Ok((rgb, width, height))
}

/// IEC 61966-2-1 の逆変換。
fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// RGB 系の ICC だけ写す。Gray / CMYK の profile は `to_rgba8` 後の並びと
/// 合わないので None(= sRGB 扱い)。
fn icc_to_srgb_rgba8(icc: &[u8]) -> Option<std::sync::Arc<moxcms::Transform8BitExecutor>> {
    let source = moxcms::ColorProfile::new_from_slice(icc).ok()?;
    source
        .create_transform_8bit(
            moxcms::Layout::Rgba,
            &moxcms::ColorProfile::new_srgb(),
            moxcms::Layout::Rgba,
            moxcms::TransformOptions::default(),
        )
        .ok()
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextCacheKey {
    layer: LayerId,
    canvas_width: u32,
    canvas_height: u32,
    content_snapshot: String,
}

impl TextCacheKey {
    fn new(
        layer: LayerId,
        document: &TextDocument,
        t: RationalTime,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        let content = document.content.eval(t);
        let content_snapshot = serde_json::to_string(&(
            &content,
            document.justify,
            document.wrap_size,
            &document.styles,
            &document.runs,
        ))
        .unwrap_or_default();
        Self {
            layer,
            canvas_width,
            canvas_height,
            content_snapshot,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ShapeCacheKey {
    layer: LayerId,
    canvas_width: u32,
    canvas_height: u32,
    content_snapshot: String,
}

impl ShapeCacheKey {
    fn new(layer: LayerId, shapes: &[ShapeNode], canvas_width: u32, canvas_height: u32) -> Self {
        let content_snapshot = serde_json::to_string(shapes).unwrap_or_default();
        Self {
            layer,
            canvas_width,
            canvas_height,
            content_snapshot,
        }
    }
}

/// 形が占める範囲だけの canvas。層の箱が中身に吸い付く。
/// 反アリアスのはみ出しを1画素見込む。
/// 絵に描く画素の予算 = comp の 4 倍。層が comp より大きい・極端な拡大でも、効果 1 段の費用を
/// comp の定数倍で止める(古い「comp 大に焼く」道の費用の上限に近い)。
pub(crate) fn raster_pixel_budget(comp: CompSpec) -> u64 {
    4 * u64::from(comp.width.max(1)) * u64::from(comp.height.max(1))
}

pub fn content_canvas(
    shapes: &[ShapeNode],
) -> Result<Option<crate::doc::vector::Canvas>, EngineError> {
    const AA: f64 = 1.0;
    let Some(b) = crate::doc::vector::content_bounds(shapes)? else {
        return Ok(None);
    };
    let min_x = (b[0] - AA).floor();
    let min_y = (b[1] - AA).floor();
    let max_x = (b[2] + AA).ceil();
    let max_y = (b[3] + AA).ceil();
    Ok(Some(crate::doc::vector::Canvas {
        width: ((max_x - min_x) as i64).max(1) as u32,
        height: ((max_y - min_y) as i64).max(1) as u32,
        origin_x: -min_x as i32,
        origin_y: -min_y as i32,
    }))
}

/// 点の直径(comp のピクセル)。層の属性になるまでの既定値。
const DEFAULT_POINT_SIZE: f32 = 2.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// Display P3 の ICC が埋まった PNG は、Finder / Preview と同じく
    /// profile を適用して読む(適用しないと (224,64,32) が (206,76,46) 程に沈む)。
    #[test]
    fn still_with_embedded_icc_is_mapped_to_srgb() {
        use image::ImageEncoder;
        let srgb = [224u8, 64, 32, 255];
        let p3 = moxcms::ColorProfile::new_display_p3();
        let to_p3 = moxcms::ColorProfile::new_srgb()
            .create_transform_8bit(
                moxcms::Layout::Rgba,
                &p3,
                moxcms::Layout::Rgba,
                moxcms::TransformOptions::default(),
            )
            .unwrap();
        let mut in_p3 = [0u8; 4];
        to_p3.transform(&srgb, &mut in_p3).unwrap();
        assert_ne!(in_p3[..3], srgb[..3], "P3 の数字は sRGB と違うはず");

        let dir = std::env::temp_dir().join(format!("motolii-icc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("p3.png");
        let mut encoder =
            image::codecs::png::PngEncoder::new(std::fs::File::create(&path).unwrap());
        encoder.set_icc_profile(p3.encode().unwrap()).unwrap();
        encoder
            .write_image(&in_p3.repeat(4), 2, 2, image::ExtendedColorType::Rgba8)
            .unwrap();

        let (rgba, w, h) = decode_still_srgb(path.to_str().unwrap()).unwrap();
        assert_eq!((w, h), (2, 2));
        for c in 0..3 {
            assert!(
                (rgba[c] as i32 - srgb[c] as i32).abs() <= 2,
                "channel {c}: got {} want {}",
                rgba[c],
                srgb[c]
            );
        }
        assert_eq!(rgba[3], 255);
        std::fs::remove_dir_all(&dir).ok();
    }


    #[test]
    fn group_bounds_use_offset_descendants_in_group_space_and_empty_groups_have_no_box() {
        use crate::doc::store::{Intent, LayerMeta, LayerTiming, LayerAttrsPatch, PropertyId, Value};
        let mut doc = crate::doc::store::blank_project();
        for (id, source, parent, position) in [
            (1, LayerSource::Group, None, [100.0, 200.0]),
            (2, LayerSource::Shape, Some(1), [10.0, 20.0]),
            (3, LayerSource::Group, Some(1), [30.0, 40.0]),
            (4, LayerSource::Shape, Some(3), [5.0, -10.0]),
            (5, LayerSource::Shape, None, [-100.0, -200.0]),
            (6, LayerSource::Group, None, [0.0, 0.0]),
        ] {
            let layer = LayerId(id);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta {
                    source, order: id as i16, timing: LayerTiming::place(0, None, 30),
                } },
                Intent::SetAttrs { layer, patch: LayerAttrsPatch {
                    parent: Some(parent.map(LayerId)), ..Default::default()
                } },
                Intent::SetConstant { layer, property: PropertyId::new("position").unwrap(), value: Value::Vec2(position) },
            ]).unwrap();
        }
        let view = doc.view();
        let mut resolved = view.resolved_layers(RationalTime::ZERO).unwrap();
        let leaf = |layer: &ResolvedLayer| Some(if layer.id == LayerId(4) {
            crate::render::media::SpatialBounds { min: [1.0, 2.0, -1.0], max: [3.0, 4.0, 1.0] }
        } else {
            crate::render::media::SpatialBounds { min: [0.0; 3], max: [4.0, 6.0, 0.0] }
        });
        assert_eq!(group_local_bounds(&view, &resolved, LayerId(1), leaf), Some(crate::render::media::SpatialBounds {
            min: [10.0, 20.0, -1.0], max: [38.0, 34.0, 1.0],
        }));
        assert_eq!(group_local_bounds(&view, &resolved, LayerId(3), leaf), Some(crate::render::media::SpatialBounds {
            min: [6.0, -8.0, -1.0], max: [8.0, -6.0, 1.0],
        }));
        assert!(group_local_bounds(&view, &resolved, LayerId(6), leaf).is_none());
        resolved.iter_mut().find(|layer| layer.id == LayerId(1)).unwrap().placement.world_transform = Some(glam::Affine3A::from_scale(glam::vec3(0.0, 1.0, 1.0)));
        assert!(group_local_bounds(&view, &resolved, LayerId(1), leaf).is_none());
    }


}

#[cfg(test)]
mod rich_text_cache_tests {
    use super::*;
    use crate::doc::store::*;
    #[test]
    fn moving_a_style_boundary_invalidates_the_texture() {
        let style=|id,size|TextDocumentStyle{id:TextStyleId(id),font:FontRef::default(),size,fill:[1.0;4],line_height:None,tracking:0.0,stroke_color:None,stroke_width:0.0,stroke_over_fill:false,axes:vec![],features:vec![]};
        let mut content=ContentTrack::new();content.insert(ContentKeyframe{t:RationalTime::ZERO,content:"AB".into()});
        let mut text=TextDocument{content,justify:TextJustify::Left,wrap_size:None,styles:vec![style(0,20.0),style(1,40.0)],slot_id:None,ranges:vec![],alignment:Default::default(),runs:vec![TextRun{len:1,style:TextStyleId(0)},TextRun{len:1,style:TextStyleId(1)}]};
        let before=TextCacheKey::new(LayerId(1),&text,RationalTime::ZERO,400,200);
        text.runs.reverse();
        assert!(before!=TextCacheKey::new(LayerId(1),&text,RationalTime::ZERO,400,200));
    }
}

/// 動画の時刻はコンポの時計で決まる。素材の fps がコンポと違っても、絵の速さは変わらない。
#[cfg(test)]
mod media_time_contract {
    use crate::doc::store::{
        Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
        RationalTime,
    };
    use crate::render::engine::Engine;

    fn ffmpeg_available() -> bool {
        crate::render::media::test_encoders_available(&["libx264"])
    }

    /// 24fps で「1 秒黒、1 秒白」の動画を作る。
    fn black_then_white_24fps(dir: &std::path::Path) -> std::path::PathBuf {
        let clip = dir.join("black_then_white_24.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args([
                "-v", "error", "-y",
                "-f", "lavfi", "-i", "color=c=black:s=64x64:r=24:d=1",
                "-f", "lavfi", "-i", "color=c=white:s=64x64:r=24:d=1",
                "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
                "-pix_fmt", "yuv420p", "-c:v", "libx264",
            ])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success(), "video fixture failed");
        clip
    }

    fn center_pixel(engine: &mut Engine, doc: &Document, frame: i64) -> [u8; 3] {
        let at = RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
        let rgba = engine.render_frame(&doc.view(), at).unwrap();
        assert!(
            engine.layer_failures().is_empty(),
            "frame {frame}: {:?}",
            engine.layer_failures()
        );
        let i = ((32 * 64) + 32) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    }

    #[test]
    fn a_24fps_clip_in_a_30fps_comp_keeps_its_own_speed() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = black_then_white_24fps(dir.path());

        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            // 層が出なかった時に黒と見分けるため、背景は赤。
            background: [1.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
                order: 0,
                timing: LayerTiming::place(0, Some(60), 60),
            },
        })
        .unwrap();

        let mut engine = Engine::new().unwrap();
        // 0.9 秒: 素材でも 0.9 秒なので黒。fps を取り違えると 27/24 = 1.125 秒で白になる。
        let at_0_9 = center_pixel(&mut engine, &doc, 27);
        // 1.1 秒: 白。
        let at_1_1 = center_pixel(&mut engine, &doc, 33);
        assert!(at_0_9[0] < 40 && at_0_9[1] < 40 && at_0_9[2] < 40, "0.9s should be black, got {at_0_9:?}");
        assert!(at_1_1[0] > 200 && at_1_1[1] > 200 && at_1_1[2] > 200, "1.1s should be white, got {at_1_1:?}");
    }

    /// 可変フレームレートの素材も第一線。24fps の黒 1 秒 + 60fps の白 1 秒を 1 本にし、0.9 秒が黒、1.1 秒が白。
    #[test]
    fn a_variable_frame_rate_clip_is_placed_by_time() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("vfr.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args([
                "-v", "error", "-y",
                "-f", "lavfi", "-i", "color=c=black:s=64x64:r=24:d=1",
                "-f", "lavfi", "-i", "color=c=white:s=64x64:r=60:d=1",
                "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
                "-fps_mode", "vfr", "-pix_fmt", "yuv420p", "-c:v", "libx264",
            ])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success(), "video fixture failed");
        let info = crate::render::media::probe(&clip).expect("VFR must be admitted");
        assert!(info.nb_frames.is_some_and(|n| n == 84), "24 + 60 frames, got {:?}", info.nb_frames);

        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [1.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
                order: 0,
                timing: LayerTiming::place(0, Some(60), 60),
            },
        })
        .unwrap();
        let mut engine = Engine::new().unwrap();
        let at_0_9 = center_pixel(&mut engine, &doc, 27);
        let at_1_1 = center_pixel(&mut engine, &doc, 33);
        assert!(at_0_9.iter().all(|c| *c < 40), "0.9s should be black, got {at_0_9:?}");
        assert!(at_1_1.iter().all(|c| *c > 200), "1.1s should be white, got {at_1_1:?}");
    }

    /// 上の不透明な動画に丸ごと覆われた動画は復号も描画もしない。半透明なら両方描く。
    #[test]
    fn a_video_hidden_behind_an_opaque_video_is_not_drawn() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = |name: &str, color: &str| {
            let path = dir.path().join(name);
            let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
                .args(["-v", "error", "-y", "-f", "lavfi", "-i", &format!("color=c={color}:s=64x64:r=30:d=1"), "-pix_fmt", "yuv420p", "-c:v", "libx264"])
                .arg(&path)
                .status()
                .expect("spawn ffmpeg");
            assert!(status.success());
            path
        };
        let below = clip("below.mp4", "black");
        let above = clip("above.mp4", "white");

        let build = |top_opacity: Option<f64>| {
            let mut doc = Document::new();
            doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
            for (id, path, order) in [(1, &below, 0), (2, &above, 1)] {
                let layer = LayerId(id);
                doc.apply(Intent::AddLayer(layer)).unwrap();
                doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_str().unwrap().to_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
            }
            if let Some(opacity) = top_opacity {
                doc.apply(Intent::SetConstant { layer: LayerId(2), property: crate::doc::store::PropertyId::new(crate::doc::store::property::OPACITY).unwrap(), value: crate::doc::eval::Value::F64(opacity) }).unwrap();
            }
            doc
        };

        let mut engine = Engine::new().unwrap();
        let opaque = build(None);
        // 1 コマ目は寸を知るために両方開く。2 コマ目から隠れが効く。
        let _ = center_pixel(&mut engine, &opaque, 1);
        let px = center_pixel(&mut engine, &opaque, 2);
        assert!(px.iter().all(|c| *c > 200), "the white layer on top should show, got {px:?}");
        assert_eq!(engine.drawn_layers(), 1, "the covered layer must not be drawn");

        let translucent = build(Some(0.5));
        let _ = center_pixel(&mut engine, &translucent, 1);
        let _ = center_pixel(&mut engine, &translucent, 2);
        assert_eq!(engine.drawn_layers(), 2, "a translucent layer hides nothing");
    }

    /// 30 コマ目から始まる層は、再生中にその瞬間から描かれる。先読みが復号器を開いておくから。
    #[test]
    fn a_layer_that_starts_later_is_drawn_from_its_first_frame_while_playing() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("late.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success());
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(30, Some(30), 90) } }).unwrap();

        let fps = Fps::try_new(30, 1).unwrap();
        let mut engine = Engine::new().unwrap();
        engine.set_realtime(true);
        // 再生中の型: 描いたら先を温める。層が始まる 15 コマ前から温まり始める。
        for frame in 0..30 {
            let t = RationalTime::try_from_frame(frame, fps).unwrap();
            let _ = engine.render_frame(&doc.view(), t).unwrap();
            engine.warm_upcoming(&doc.view(), t).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        let at_start = center_pixel(&mut engine, &doc, 30);
        assert_eq!(engine.drawn_layers(), 1, "the layer must be drawn on its first frame");
        assert!(at_start.iter().all(|c| *c > 200), "first frame should already be white, got {at_start:?}");
    }

    /// bt2020 HLG(iPhone の HDR)の素材も断らない。白は白、赤は赤のまま出る。
    #[test]
    fn hdr_bt2020_clips_are_admitted_and_shown() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("hlg.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args([
                "-v", "error", "-y",
                "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=0.5",
                "-f", "lavfi", "-i", "color=c=red:s=64x64:r=30:d=0.5",
                "-filter_complex", "[0][1]concat=n=2:v=1:a=0",
                "-pix_fmt", "yuv420p", "-c:v", "libx264",
                "-colorspace", "bt2020nc", "-color_primaries", "bt2020", "-color_trc", "arib-std-b67", "-color_range", "tv",
            ])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success());
        crate::render::media::probe(&clip).expect("HDR must be admitted");
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [0.0, 1.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
        let mut engine = Engine::new().unwrap();
        let white = center_pixel(&mut engine, &doc, 7);
        let red = center_pixel(&mut engine, &doc, 22);
        assert!(white.iter().all(|c| *c >= 240), "white should stay white, got {white:?}");
        assert!(red[0] >= 200 && red[1] <= 60 && red[2] <= 60, "red should stay red, got {red:?}");
    }

    /// 音だけの層(mp3)は絵を持たず、失敗にもならない。動画の層と並べても再生は止まらない。
    #[test]
    fn an_audio_only_layer_is_silent_on_stage_and_not_a_failure() {
        if !crate::render::media::test_encoders_available(&["libx264", "libmp3lame"]) {
            eprintln!("skip: ffmpeg encoders missing");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let song = dir.path().join("song.mp3");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000:duration=1", "-c:a", "libmp3lame"])
            .arg(&song).status().unwrap();
        assert!(status.success());
        let clip = dir.path().join("clip.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
            .arg(&clip).status().unwrap();
        assert!(status.success());
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
        for (id, path) in [(1, &clip), (2, &song)] {
            let layer = LayerId(id);
            doc.apply(Intent::AddLayer(layer)).unwrap();
            doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_str().unwrap().to_owned(), fingerprint: None }, order: id as i16, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();
        }
        let mut engine = Engine::new().unwrap();
        let px = center_pixel(&mut engine, &doc, 5);
        assert!(px.iter().all(|c| *c > 200), "the video still shows, got {px:?}");
        assert_eq!(engine.drawn_layers(), 1);
    }

    /// 一度描いたコマは cache に入り、2 周目は復号器に聞かない。上限を超えると使われてから古い物が消える。
    #[test]
    fn frames_seen_once_come_from_the_cache_within_the_budget() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("loop.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=1", "-pix_fmt", "yuv420p", "-c:v", "libx264"])
            .arg(&clip).status().unwrap();
        assert!(status.success());
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 64, height: 64, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 30, background: [1.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(30), 30) } }).unwrap();

        let mut engine = Engine::new().unwrap();
        for frame in 0..10 {
            let _ = center_pixel(&mut engine, &doc, frame);
        }
        // 写しは次の frame の頭で入るので、直後は最後の 1 枚がまだ待ち。
        let (hits, entries, bytes) = engine.video_frame_cache_stats();
        assert_eq!((hits, entries, bytes), (0, 9, 9 * 64 * 64 * 4), "first pass fills the cache");
        for frame in 0..10 {
            let px = center_pixel(&mut engine, &doc, frame);
            assert!(px.iter().all(|c| *c > 200), "cached frame must look the same, got {px:?}");
        }
        let (hits, entries, _) = engine.video_frame_cache_stats();
        assert_eq!((hits, entries), (10, 10), "second pass hits every frame");

        let mut small = Engine::new().unwrap();
        small.set_video_frame_cache_budget(3 * 64 * 64 * 4);
        for frame in 0..10 {
            let _ = center_pixel(&mut small, &doc, frame);
        }
        let (_, entries, bytes) = small.video_frame_cache_stats();
        assert!(entries <= 3 && bytes <= 3 * 64 * 64 * 4, "budget holds: {entries} entries, {bytes} bytes");
    }

    /// tv range の bt709 素材。白は 255、黒は 0、赤は赤。range を取り違えると白が 235・黒が 16 になる。
    #[test]
    fn limited_range_bt709_colors_come_out_right() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("wbr_709_tv.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args([
                "-v", "error", "-y",
                "-f", "lavfi", "-i", "color=c=white:s=64x64:r=30:d=0.5",
                "-f", "lavfi", "-i", "color=c=black:s=64x64:r=30:d=0.5",
                "-f", "lavfi", "-i", "color=c=red:s=64x64:r=30:d=0.5",
                "-filter_complex", "[0][1][2]concat=n=3:v=1:a=0",
                "-pix_fmt", "yuv420p", "-c:v", "libx264", "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv",
            ])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success(), "video fixture failed");

        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 45,
            background: [0.0, 1.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None },
                order: 0,
                timing: LayerTiming::place(0, Some(45), 45),
            },
        })
        .unwrap();

        let mut engine = Engine::new().unwrap();
        let white = center_pixel(&mut engine, &doc, 7);
        let black = center_pixel(&mut engine, &doc, 22);
        let red = center_pixel(&mut engine, &doc, 37);
        assert!(white.iter().all(|c| *c >= 250), "white should be 255, got {white:?}");
        assert!(black.iter().all(|c| *c <= 5), "black should be 0, got {black:?}");
        assert!(red[0] >= 240 && red[1] <= 20 && red[2] <= 20, "red should stay red, got {red:?}");
    }

    /// 1080p H.264、同じ測り方。
    #[test]
    #[ignore]
    fn full_hd_playback_pace() {
        if !ffmpeg_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("k1.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=1920x1080:rate=30:duration=3", "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p"])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success());
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 1920, height: 1080, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(90), 90) } }).unwrap();
        let mut engine = Engine::new().unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(0, fps).unwrap()).unwrap();
        let started = std::time::Instant::now();
        for frame in 1..61 {
            let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
        }
        println!("PROBE room=video verdict=1080p-pace ms-per-frame={:.1}", started.elapsed().as_secs_f64() * 1000.0 / 60.0);
    }

    /// 4K H.264 を順に 60 コマ描いた時の 1 コマの時間。数字を見る物(`cargo test -- --ignored four_k`)。
    #[test]
    #[ignore]
    fn four_k_playback_pace() {
        if !ffmpeg_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let clip = dir.path().join("k4.mp4");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=3840x2160:rate=30:duration=3", "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p"])
            .arg(&clip)
            .status()
            .expect("spawn ffmpeg");
        assert!(status.success());
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 3840, height: 2160, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.to_str().unwrap().to_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, Some(90), 90) } }).unwrap();
        let mut engine = Engine::new().unwrap();
        let fps = Fps::try_new(30, 1).unwrap();
        let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(0, fps).unwrap()).unwrap();
        let started = std::time::Instant::now();
        for frame in 1..61 {
            let _ = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
        }
        let per_frame_ms = started.elapsed().as_secs_f64() * 1000.0 / 60.0;
        println!("PROBE room=video verdict=4k-pace ms-per-frame={per_frame_ms:.1} failures={:?}", engine.layer_failures());
    }
}

#[cfg(test)]
mod vector_projection_contract {
    use super::*;
    use crate::doc::store::*;
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};

    fn circle(size: f64, scale: f64, pass: bool) -> Document {
        let shapes = vec![ShapeNode::Leaf(Shape {
            source: PathSource::Ellipse { size: Point { x: size, y: size } },
            ops: Vec::new(), stroke: None,
            fill: Some(Fill { brush: Brush::Solid(Rgb { r: 0.0, g: 0.0, b: 0.0 }), ..Default::default() }),
        })];
        let canvas = content_canvas(&shapes).unwrap().unwrap();
        let mut doc = blank_project();
        let mut comp = doc.view().composition().unwrap().unwrap();
        comp.width = 512; comp.height = 512; comp.background = [1.0; 4];
        let id = LayerId(1);
        doc.apply_all([
            Intent::SetComposition(comp), Intent::AddLayer(id),
            Intent::SetMeta { layer: id, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 60) } },
            Intent::SetShapes { layer: id, shapes },
            Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
            Intent::SetConstant { layer: id, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([256.0,256.0]) },
            Intent::SetConstant { layer: id, property: PropertyId::new(property::ANCHOR).unwrap(), value: Value::Vec2([canvas.origin_x as f64,canvas.origin_y as f64]) },
            Intent::SetConstant { layer: id, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([scale,scale]) },
        ]).unwrap();
        if pass {
            doc.apply_all([
                Intent::SetEffects { layer: id, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.gain".into() }] },
                Intent::SetConstant { layer: id, property: PropertyId::new("effect.0.param.gain").unwrap(), value: Value::F64(1.0) },
            ]).unwrap();
        }
        doc
    }

    /// 広がりの法: 効果は素材全体に素材座標で評価する。comp からはみ出した円の Blur が、
    /// comp の縁で切れない(comp を広げても、重なる範囲の絵は同じ)。
    #[test]
    fn blur_reaches_past_the_composition_edge() {
        let scene = |width: u32| {
            let mut doc = circle(48.0, 1.0, false);
            let mut comp = doc.view().composition().unwrap().unwrap();
            comp.width = width; comp.height = 64;
            doc.apply_all([
                Intent::SetComposition(comp),
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([64.0, 32.0]) },
                Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.radius").unwrap(), value: Value::F64(8.0) },
            ]).unwrap();
            doc
        };
        let mut engine = Engine::new().unwrap();
        let narrow = engine.render_frame(&scene(64).view(), RationalTime::ZERO).unwrap();
        let wide = engine.render_frame(&scene(128).view(), RationalTime::ZERO).unwrap();
        let mut differing = 0;
        let mut blurred_edge = 0;
        for y in 0..64usize {
            for x in 0..64usize {
                let a = &narrow[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
                let b = &wide[(y * 128 + x) * 4..(y * 128 + x) * 4 + 4];
                if a.iter().zip(b).any(|(a, b)| a.abs_diff(*b) > 3) { differing += 1; }
                // 縁の 1 列: 円の中(白地に黒)がぼけて灰になっている画素
                if x == 63 && a[0] > 8 && a[0] < 247 { blurred_edge += 1; }
            }
        }
        assert!(blurred_edge > 4, "the circle must straddle the right edge and be blurred there");
        assert!(differing < 20, "the composition edge cut the blur: {differing} pixels differ from the wider composition");
    }

    /// 広がりの法(密度): 効果の radius・reach は論理 px。20 倍に置いた小さな円の Blur 4 は、
    /// 大きな円の Blur 80 と同じ絵になる(余白の置き方も密度で割れている)。
    #[test]
    fn blur_radius_is_measured_in_logical_pixels_at_any_raster_density() {
        let blur = |mut doc: Document, radius: f64| {
            doc.apply_all([
                Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.radius").unwrap(), value: Value::F64(radius) },
            ]).unwrap();
            doc
        };
        let mut engine = Engine::new().unwrap();
        let a = engine.render_frame(&blur(circle(16.0, 20.0, false), 4.0).view(), RationalTime::ZERO).unwrap();
        let b = engine.render_frame(&blur(circle(320.0, 1.0, false), 80.0).view(), RationalTime::ZERO).unwrap();
        if let Some(out) = std::env::var_os("MOTOLII_VECTOR_EVIDENCE") {
            let dir = std::path::PathBuf::from(out); std::fs::create_dir_all(&dir).unwrap();
            image::save_buffer(dir.join("blur-density20.png"), &a, 512,512,image::ColorType::Rgba8).unwrap();
            image::save_buffer(dir.join("blur-density1.png"), &b, 512,512,image::ColorType::Rgba8).unwrap();
        }
        let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a, b)| a[0].abs_diff(b[0]) > 12).count();
        let soft = a.chunks_exact(4).filter(|p| p[0] > 16 && p[0] < 240).count();
        assert!(soft > 20000, "the magnified circle must be visibly blurred: {soft} soft pixels");
        assert!(bad < 1500, "blur width or padding changed with raster density: {bad} differing pixels");
    }

    #[test]
    fn a_slow_field_does_not_tear_the_fill_and_stroke_of_one_plane() {
        let scene = |opacity: f64, evolution: f64| {
            let mut doc = circle(320.0, 1.0, false);
            let mut shapes = doc.view().shapes(LayerId(1)).unwrap();
            if let ShapeNode::Leaf(shape) = &mut shapes[0] {
                shape.fill.as_mut().unwrap().brush = Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 });
            }
            shapes.insert(0, ShapeNode::Leaf(Shape {
                source: PathSource::Rectangle { size: Point { x: 400.0, y: 400.0 } }, ops: Vec::new(), stroke: None,
                fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), opacity, ..Default::default() }),
            }));
            let mut comp = doc.view().composition().unwrap().unwrap(); comp.background = [0.0,0.0,0.0,1.0];
            doc.apply_all([
                Intent::SetComposition(comp), Intent::SetShapes { layer: LayerId(1), shapes },
                Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.turbulent_displace".into() }] },
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.amount").unwrap(), value: Value::F64(400.0) },
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.size").unwrap(), value: Value::F64(10000.0) },
                Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.evolution").unwrap(), value: Value::F64(evolution) },
            ]).unwrap();
            doc
        };
        let mut engine = Engine::new().unwrap();
        let white = |pixels: &[u8]| pixels.chunks_exact(4).filter(|p| p[0]>230 && p[1]>230 && p[2]>230).count();
        for evolution in [0.0,1.0,2.0] {
            let reference = engine.render_frame(&scene(0.5,evolution).view(), RationalTime::ZERO).unwrap();
            let opaque = engine.render_frame(&scene(1.0,evolution).view(), RationalTime::ZERO).unwrap();
            let (expected,actual) = (white(&reference),white(&opaque));
            assert!(expected>5000 && actual*100 >= expected*98, "underpaint opacity must not occlude opaque foreground paint: {expected} -> {actual}, evolution={evolution}");
        }
    }

    #[test]
    fn a_clipped_vector_selection_uses_only_the_visible_half() {
        let mut doc = circle(320.0, 1.0, false);
        doc.apply_all([
            Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.clip".into() }] },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new("effect.0.param.axis").unwrap(), value: Value::F64(0.0) },
        ]).unwrap();
        let mut engine = Engine::new().unwrap();
        let texture = engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
            label: Some("clipped vector selection"), size: wgpu::Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
        });
        engine.render_frame_into_with_camera(&doc.view(), RationalTime::ZERO, &texture, Default::default(), true, &[LayerId(1)]).unwrap();
        engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let bounds = engine.take_selection_bounds().unwrap();
        let (_, b) = bounds.iter().find(|(id,_)| *id == LayerId(1)).unwrap();
        assert!((95.0..=97.0).contains(&b[0]) && (255.0..=257.0).contains(&b[2]), "the clipped circle's mask must stop at its center: {b:?}");
    }

    #[test]
    fn camera_magnification_keeps_the_contour_at_output_precision() {
        let mut engine = Engine::new().unwrap();
        let mut doc = circle(16.0, 1.0, false);
        doc.apply(Intent::SetAttrs { layer: LayerId(1), patch: LayerAttrsPatch { projection: Some(LayerProjection::ThreeD), ..Default::default() } }).unwrap();
        let camera = crate::doc::core::ResolvedCamera { zoom: 20.0, ..Default::default() };
        let a = engine.render_with_camera_override(&doc.view(), RationalTime::ZERO, true, Some(camera)).unwrap();
        let b = engine.render_frame(&circle(320.0, 1.0, false).view(), RationalTime::ZERO).unwrap();
        let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a,b)| a[0].abs_diff(b[0]) > 16).count();
        assert!(bad < 300, "camera magnification introduced {bad} differing pixels");
    }

    #[test]
    fn translucent_vector_paint_blends_over_the_background() {
        let mut doc = circle(320.0, 1.0, false);
        let mut shapes = doc.view().shapes(LayerId(1)).unwrap();
        if let ShapeNode::Leaf(shape) = &mut shapes[0] { shape.fill.as_mut().unwrap().opacity = 0.5; }
        doc.apply(Intent::SetShapes { layer: LayerId(1), shapes }).unwrap();
        let pixels = Engine::new().unwrap().render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let center = pixels[(256*512+256)*4];
        assert!((180..=195).contains(&center), "half black over linear white, encoded as sRGB: {center}");
    }

    #[test]
    fn enlarging_a_path_matches_drawing_the_large_contour_even_at_an_image_effect_boundary() {
        let mut engine = Engine::new().unwrap();
        for pass in [false,true] {
            let small = circle(16.0, 20.0, pass);
            let a = engine.render_frame(&small.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            assert!(engine.shape_textures.values().any(|c| matches!(c.texture, LayerContent::Model(ref m) if m.planar_size.is_some())), "the contour must remain geometry");
            let b = engine.render_frame(&circle(320.0,1.0,pass).view(), RationalTime::ZERO).unwrap();
            let bad = a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a,b)| a[0].abs_diff(b[0]) > 16).count();
            let ink = a.chunks_exact(4).filter(|p| p[0] < 128).count();
            if let Some(out) = std::env::var_os("MOTOLII_VECTOR_EVIDENCE") {
                let dir = std::path::PathBuf::from(out); std::fs::create_dir_all(&dir).unwrap();
                image::save_buffer(dir.join(format!("circle-scale20-pass{pass}.png")), &a, 512,512,image::ColorType::Rgba8).unwrap();
                image::save_buffer(dir.join(format!("circle-reference-pass{pass}.png")), &b, 512,512,image::ColorType::Rgba8).unwrap();
                std::fs::write(dir.join(format!("circle-comparison-pass{pass}.json")), serde_json::json!({"different_pixels_over_16":bad,"ink_pixels":ink,"pixels":512*512,"scale":20,"image_effect":pass}).to_string()).unwrap();
            }
            assert!(ink > 70000 && ink < 90000, "the unlit circle keeps its size and paint: {ink}");
            assert!(bad < 300, "magnifying a contour introduced {bad} differing pixels (image effect={pass})");
        }
    }
}
