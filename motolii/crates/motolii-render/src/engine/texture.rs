use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::doc::core::CompSpec;
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};
use crate::render::compositor::LayerContent;
use crate::render::media::{is_point_cloud_path, load_point_cloud, probe};

use crate::render::engine::render::layer_size;
use crate::render::engine::{shape, text, Engine, EngineError};

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
fn unpremultiply(rgba: &mut [u8], width: usize) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 0 || a == 255 {
            continue;
        }
        for c in &mut px[..3] {
            *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
        }
    }
    bleed_edges(rgba, width);
}

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
    texture: crate::render::compositor::GpuTexture2D,
    bounds: Option<crate::render::media::SpatialBounds>,
}

fn raster_alpha_bounds(raster: &crate::doc::vector::Raster) -> Option<crate::render::media::SpatialBounds> {
    let width = raster.width as usize;
    if width == 0 { return None; }
    let mut min = [raster.width, raster.height];
    let mut max = [0u32; 2];
    let mut any = false;
    for (index, pixel) in raster.premultiplied_rgba8.chunks_exact(4).enumerate() {
        if pixel[3] == 0 { continue; }
        let x = (index % width) as u32;
        let y = (index / width) as u32;
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        max[0] = max[0].max(x + 1);
        max[1] = max[1].max(y + 1);
        any = true;
    }
    any.then_some(crate::render::media::SpatialBounds {
        min: [min[0] as f32, min[1] as f32, 0.0],
        max: [max[0] as f32, max[1] as f32, 0.0],
    })
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
                planar(cached.bounds?, cached.texture.width_height().map(|v| v as f32))
            }
            LayerSource::Shape => {
                let shapes = view.shapes(layer_id).ok()?;
                let canvas = content_canvas(&shapes).ok().flatten()?;
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                let natural = self.shape_textures.get(&key)?.width_height().map(|v| v as f32);
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
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if layer.source == LayerSource::Text {
            self.text_texture_from_document(text_documents.get(&layer.id), layer.id, t, comp)
        } else if layer.source == LayerSource::Shape {
            let shapes = shape_documents
                .get(&layer.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            self.shape_texture_from_shapes(shapes, layer.id)
        } else if let LayerSource::File { path, .. } = &layer.source {
            let path = path.clone();
            if layer.environment && crate::render::media::is_still_image_path(&path) {
                return self.environment_content_for(&path);
            }
            self.file_content_for(&path, layer.source_frame, layer.id, comp)
        } else {
            self.texture_for(&layer.source, layer.source_frame)
        }
    }

    fn text_texture_from_document(
        &mut self,
        document: Option<&TextDocument>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        let Some(document) = document else {
            return Ok((None, [0.0, 0.0]));
        };

        let canvas = crate::doc::vector::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };

        let key = TextCacheKey::new(layer_id, document, t, canvas.width, canvas.height);
        if let Some(cached) = self.text_textures.get(&key) {
            return Ok((
                Some(LayerContent::Texture(cached.texture.clone())),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let Some(mut raster) = text::rasterize_text_document(document, t, &canvas)? else {
            return Ok((None, [0.0, 0.0]));
        };
        let bounds = raster_alpha_bounds(&raster);
        unpremultiply(&mut raster.premultiplied_rgba8, raster.width as usize);

        let texture = self.compositor.upload_rgba(
            "text",
            &raster.premultiplied_rgba8,
            raster.width,
            raster.height,
        )?;
        self.text_textures.insert(key.clone(), TextTexture { texture: texture.clone(), bounds });
        self.text_order.push_back(key);
        // 級数を擦るだけで鍵が増える。歌詞 200 行 + 擦りの残骸で GPU を食い潰さない。
        while self.text_order.len() > TEXT_CACHE_LIMIT {
            if let Some(old) = self.text_order.pop_front() {
                self.text_textures.remove(&old);
            }
        }
        Ok((
            Some(LayerContent::Texture(texture)),
            [raster.width as f32, raster.height as f32],
        ))
    }

    fn shape_texture_from_shapes(
        &mut self,
        shapes: &[ShapeNode],
        layer_id: LayerId,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }

        let Some(canvas) = content_canvas(shapes)? else {
            return Ok((None, [0.0, 0.0]));
        };

        let key = ShapeCacheKey::new(layer_id, shapes, canvas.width, canvas.height);
        if let Some(texture) = self.shape_textures.get(&key) {
            return Ok((
                Some(LayerContent::Texture(texture.clone())),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let Some(mut raster) = shape::rasterize_shapes(shapes, &canvas)? else {
            return Ok((None, [0.0, 0.0]));
        };
        unpremultiply(&mut raster.premultiplied_rgba8, raster.width as usize);

        let texture = self.compositor.upload_rgba(
            "shape",
            &raster.premultiplied_rgba8,
            raster.width,
            raster.height,
        )?;
        self.shape_textures.insert(key, texture.clone());
        Ok((
            Some(LayerContent::Texture(texture)),
            [raster.width as f32, raster.height as f32],
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
        source_frame: i64,
        layer: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<LayerContent>, [f32; 2]), EngineError> {
        if crate::render::media::is_mesh_path(path) {
            self.mesh_content_for(path, comp)
        } else if is_point_cloud_path(path) {
            self.point_cloud_content_for(path, comp)
        } else if crate::render::media::is_still_image_path(path) {
            self.still_texture_for(path)
        } else {
            self.media_texture_for(path, source_frame, layer)
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

    fn media_texture_for(
        &mut self,
        path: &str,
        frame: i64,
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

        let last_frame = info.nb_frames.map(|n| n - 1);
        if frame < 0 || last_frame.is_some_and(|last| frame > last) {
            return Ok((None, natural));
        }

        if !self.videos.contains_key(path) {
            let bytes = match std::fs::read(path) {
                Ok(bytes) => bytes,
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
            let video = re_renderer::video::Video::load(
                path.to_owned(),
                descr,
                re_video::DecodeSettings::default(),
            );
            self.videos.insert(path.to_owned(), (bytes, video));
        }
        let (bytes, video) = self.videos.get(path).expect("直前に insert した");

        let Some(timescale) = video.data_descr().timescale else {
            self.layer_failures
                .push(format!("動画にタイムスケールが無い: {path}"));
            return Ok((None, natural));
        };
        // frame → 時刻は正準口を通す(浮動小数の割り算で写さない)。
        // `re_video::Time::from_secs` が秒の f64 を要求するので、
        // 有理数で写してから最後に一度だけ f64 にする。
        let secs = crate::doc::core::RationalTime::try_from_frame(frame, info.fps)
            .map_err(|e| crate::render::engine::EngineError::Time(e.to_string()))?
            .as_seconds_f64();
        let video_time = re_video::Time::from_secs(secs, timescale);
        let stream_id = re_video::player::VideoPlayerStreamId(layer_stream_id(layer, path));
        let source = re_video::player::VideoSliceSource(bytes);
        // デコーダは非同期で、頼んだ直後は返さない。待たずに前のコマを
        // 返すと、**同じ時刻でも辿り着き方で絵が変わり**、窓と書き出しが
        // 一致しなくなる。待つのは素材ごとに初回だけ(実測 764ms、以降 65µs)。
        let deadline = std::time::Instant::now() + DECODE_PATIENCE;
        loop {
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
            if ready {
                let texture = output.output.and_then(|frame| frame.texture);
                return Ok((texture.map(LayerContent::Texture), natural));
            }
            if let Some(err) = output.error {
                self.layer_failures.push(format!(
                    "フレームを読めない(decode失敗): {path} frame={frame}: {err}"
                ));
                return Ok((None, natural));
            }
            if std::time::Instant::now() >= deadline {
                self.layer_failures
                    .push(format!("コマが間に合わなかった: {path} frame={frame}"));
                return Ok((None, natural));
            }
            std::thread::sleep(DECODE_POLL);
        }
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
fn content_canvas(
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
    fn cached_text_bounds_follow_nontransparent_content_not_canvas_or_rgb_bleed() {
        let mut raster = crate::doc::vector::Raster {
            width: 12, height: 8, premultiplied_rgba8: vec![0; 12 * 8 * 4],
        };
        assert!(raster_alpha_bounds(&raster).is_none());
        for (x, y, alpha) in [(4, 2, 255), (8, 5, 1)] {
            raster.premultiplied_rgba8[(y * 12 + x) * 4 + 3] = alpha;
        }
        raster.premultiplied_rgba8[0] = 255;
        assert_eq!(raster_alpha_bounds(&raster), Some(crate::render::media::SpatialBounds {
            min: [4.0, 2.0, 0.0], max: [9.0, 6.0, 0.0],
        }));
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


    #[test]
    fn unpremultiply_restores_the_straight_color_and_leaves_the_edges() {
        // 乗算済み (64,32,0,128) ≒ 非乗算 (128,64,0)。a=0 と a=255 は触らない。
        let mut px = [64, 32, 0, 128, 10, 20, 30, 0, 200, 100, 50, 255];
        unpremultiply(&mut px, 3);
        assert_eq!(&px[..4], &[128, 64, 0, 128]);
        // 透明な texel は隣の色の平均で埋まる(α は 0 のまま)。
        assert_eq!(&px[4..8], &[164, 82, 25, 0]);
        assert_eq!(&px[8..], &[200, 100, 50, 255]);
        // c > a(壊れた入力)は 255 で止まる。
        let mut bad = [200, 0, 0, 100];
        unpremultiply(&mut bad, 1);
        assert_eq!(bad[0], 255);
        // 2px まで広がる。3px 先はまだ黒。
        let mut row = [255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        unpremultiply(&mut row, 4);
        assert_eq!(&row[4..7], &[255, 255, 255]);
        assert_eq!(&row[8..11], &[255, 255, 255]);
        assert_eq!(&row[12..15], &[0, 0, 0]);
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
