
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::render::compositor::GpuTexture2D;
use crate::doc::core::CompSpec;
use crate::render::media::{is_point_cloud_path, load_point_cloud, probe};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};

use crate::render::engine::render::layer_size;
use crate::render::engine::{shape, text, Engine, EngineError};

fn layer_stream_id(layer: LayerId, path: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    layer.0.hash(&mut hasher);
    path.hash(&mut hasher);
    hasher.finish()
}

impl Engine {
    pub(crate) fn texture_for_layer(
        &mut self,
        view: &StoreView<'_>,
        layer: &ResolvedLayer,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if layer.source == LayerSource::Text {
            self.text_texture_for(view, layer.id, t, comp)
        } else if layer.source == LayerSource::Shape {
            self.shape_texture_for(view, layer.id, comp)
        } else if let LayerSource::File { path, .. } = &layer.source {
            if is_point_cloud_path(path) {
                self.point_cloud_texture_for(path, comp)
            } else {
                let path = path.clone();
                self.media_texture_for(&path, layer.source_frame, layer.id)
            }
        } else {
            self.texture_for(&layer.source, layer.source_frame)
        }
    }

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
                self.text_textures.get(&key)?.width_height().map(|v| v as f32)
            }
            LayerSource::Shape => {
                let shapes = view.shapes(layer_id).ok()?;
                let canvas = crate::render::vector::Canvas::centered(comp.width, comp.height);
                let key = ShapeCacheKey::new(layer_id, &shapes, canvas.width, canvas.height);
                self.shape_textures.get(&key)?.width_height().map(|v| v as f32)
            }
            LayerSource::Null | LayerSource::Group => {
                [comp.width as f32, comp.height as f32]
            }
            LayerSource::Solid { width, height, .. } => [*width as f32, *height as f32],
            LayerSource::File { path, .. } => {
                if is_point_cloud_path(path) {
                    [comp.width as f32, comp.height as f32]
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
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if layer.source == LayerSource::Text {
            self.text_texture_from_document(text_documents.get(&layer.id), layer.id, t, comp)
        } else if layer.source == LayerSource::Shape {
            let shapes = shape_documents
                .get(&layer.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            self.shape_texture_from_shapes(shapes, layer.id, comp)
        } else if let LayerSource::File { path, .. } = &layer.source {
            if is_point_cloud_path(path) {
                self.point_cloud_texture_for(path, comp)
            } else {
                let path = path.clone();
                self.media_texture_for(&path, layer.source_frame, layer.id)
            }
        } else {
            self.texture_for(&layer.source, layer.source_frame)
        }
    }

    fn text_texture_for(
        &mut self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let document = view
            .resolved_text_document(layer_id, t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        self.text_texture_from_document(document.as_ref(), layer_id, t, comp)
    }

    fn text_texture_from_document(
        &mut self,
        document: Option<&TextDocument>,
        layer_id: LayerId,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
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
                Some(texture.clone()),
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
        Ok((Some(texture), [raster.width as f32, raster.height as f32]))
    }

    fn shape_texture_for(
        &mut self,
        view: &StoreView<'_>,
        layer_id: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let shapes = view
            .shapes(layer_id)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        self.shape_texture_from_shapes(&shapes, layer_id, comp)
    }

    fn shape_texture_from_shapes(
        &mut self,
        shapes: &[ShapeNode],
        layer_id: LayerId,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        if shapes.is_empty() {
            return Ok((None, [0.0, 0.0]));
        }

        let canvas = crate::render::vector::Canvas::centered(comp.width, comp.height);

        let key = ShapeCacheKey::new(layer_id, shapes, canvas.width, canvas.height);
        if let Some(texture) = self.shape_textures.get(&key) {
            return Ok((
                Some(texture.clone()),
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
        Ok((Some(texture), [raster.width as f32, raster.height as f32]))
    }

    fn point_cloud_texture_for(
        &mut self,
        path: &str,
        comp: CompSpec,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        let natural = [comp.width as f32, comp.height as f32];

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
            return Ok((None, natural));
        };
        if data.positions.is_empty() {
            return Ok((None, natural));
        }

        let key = (path.to_owned(), comp.width, comp.height);
        if let Some(texture) = self.point_cloud_textures.get(&key) {
            return Ok((Some(texture.clone()), natural));
        }

        let texture = match self.compositor.render_point_cloud_to_texture(
            &data.positions,
            &data.colors,
            comp.width,
            comp.height,
        ) {
            Ok(texture) => texture,
            Err(err) => {
                self.layer_failures
                    .push(format!("点群を描けない: {path}: {err}"));
                return Ok((None, natural));
            }
        };
        self.point_cloud_textures.insert(key, texture.clone());
        Ok((Some(texture), natural))
    }

    fn media_texture_for(
        &mut self,
        path: &str,
        frame: i64,
        layer: LayerId,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
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
                                let reason =
                                    format!("素材を読めない(probe失敗): {path}: {err}");
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
                    let descr = match re_video::VideoDataDescription::load_from_bytes(
                        &bytes,
                        "video/mp4",
                        path,
                    ) {
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
                let output = video.frame_at(
                    self.compositor.render_context(),
                    stream_id,
                    video_time,
                    &source,
                );
                if let Some(err) = output.error {
                    self.layer_failures
                        .push(format!("フレームを読めない(decode失敗): {path} frame={frame}: {err}"));
                }
                match output.output.and_then(|frame_texture| frame_texture.texture) {
                    Some(texture) => {
                        self.video_last_texture.insert(stream_id.0, texture.clone());
                        Ok((Some(texture), natural))
                    }
                    None => Ok((self.video_last_texture.get(&stream_id.0).cloned(), natural)),
                }
                }

    pub(crate) fn texture_for(
        &mut self,
        source: &LayerSource,
        _source_frame: i64,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        match source {
            LayerSource::Solid {
                rgba,
                width,
                height,
            } => {
                let natural = [*width as f32, *height as f32];
                if let Some(texture) = self.textures.get(source) {
                    return Ok((Some(texture.clone()), natural));
                }
                let pixels: Vec<u8> = rgba
                    .iter()
                    .copied()
                    .cycle()
                    .take((width * height * 4) as usize)
                    .collect();
                let texture = self
                    .compositor
                    .upload_rgba("solid", &pixels, *width, *height)?;
                self.textures.insert(source.clone(), texture.clone());
                Ok((Some(texture), natural))
            }

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
