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

impl Engine {
    pub fn selected_layer_size(
        &self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
    ) -> Option<[f32; 2]> {
        let composition = view.composition().ok().flatten()?;
        let comp = composition.spec();
        let resolved = view.resolved_layers(t).ok()?;
        let layer = resolved.iter().find(|l| l.id == layer_id)?;

        let natural = match &layer.source {
            LayerSource::Text => {
                let document = view.resolved_text_document(layer_id, t).ok().flatten()?;
                let key = TextCacheKey::new(layer_id, &document, t, comp.width, comp.height);
                self.text_textures
                    .get(&key)?
                    .width_height()
                    .map(|v| v as f32)
            }
            LayerSource::Shape => {
                let shapes = view.shapes(layer_id).ok()?;
                let canvas = content_canvas(&shapes).ok().flatten()?;
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                self.shape_textures
                    .get(&key)?
                    .width_height()
                    .map(|v| v as f32)
            }
            LayerSource::Null | LayerSource::Group => [comp.width as f32, comp.height as f32],
            LayerSource::File { path, .. } => {
                if crate::render::media::is_mesh_path(path) {
                    self.models.get(path)?.bounds().size_xy()
                } else if is_point_cloud_path(path) {
                    self.point_clouds.get(path)?.bounds().size_xy()
                } else {
                    let info = self.probes.get(path)?;
                    [info.width as f32, info.height as f32]
                }
            }
        };
        Some(layer_size(layer, natural))
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

        let canvas = crate::render::vector::Canvas {
            width: comp.width,
            height: comp.height,
            origin_x: 0,
            origin_y: 0,
        };

        let key = TextCacheKey::new(layer_id, document, t, canvas.width, canvas.height);
        if let Some(texture) = self.text_textures.get(&key) {
            return Ok((
                Some(LayerContent::Texture(texture.clone())),
                [canvas.width as f32, canvas.height as f32],
            ));
        }

        let Some(raster) = text::rasterize_text_document(document, t, &canvas)? else {
            return Ok((None, [0.0, 0.0]));
        };

        let texture = self.compositor.upload_rgba(
            "text",
            &raster.premultiplied_rgba8,
            raster.width,
            raster.height,
        )?;
        self.text_textures.insert(key, texture.clone());
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

        let Some(raster) = shape::rasterize_shapes(shapes, &canvas)? else {
            return Ok((None, [0.0, 0.0]));
        };

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
                let decoded = image::ImageReader::open(path)
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.decode().map_err(|e| e.to_string()))?
                    .to_rgba8();
                let (width, height) = decoded.dimensions();
                // 合成は乗算済みを前提にしている。素の RGBA を渡すと縁が光る。
                let mut premultiplied_rgba = decoded.into_raw();
                for px in premultiplied_rgba.chunks_exact_mut(4) {
                    let a = px[3] as u32;
                    for c in &mut px[..3] {
                        *c = ((*c as u32 * a + 127) / 255) as u8;
                    }
                }
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
            LayerSource::Null | LayerSource::Group => Ok((None, [0.0, 0.0])),
        }
    }
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
) -> Result<Option<crate::render::vector::Canvas>, EngineError> {
    const AA: f64 = 1.0;
    let Some(b) = crate::render::vector::content_bounds(shapes)? else {
        return Ok(None);
    };
    let min_x = (b[0] - AA).floor();
    let min_y = (b[1] - AA).floor();
    let max_x = (b[2] + AA).ceil();
    let max_y = (b[3] + AA).ceil();
    Ok(Some(crate::render::vector::Canvas {
        width: ((max_x - min_x) as i64).max(1) as u32,
        height: ((max_y - min_y) as i64).max(1) as u32,
        origin_x: -min_x as i32,
        origin_y: -min_y as i32,
    }))
}

/// 点の直径(comp のピクセル)。層の属性になるまでの既定値。
const DEFAULT_POINT_SIZE: f32 = 2.0;
