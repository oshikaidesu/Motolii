#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::doc::core::CompSpec;
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ShapeNode, StoreView, TextDocument,
};
use crate::render::compositor::LayerContent;
use crate::render::media::{is_point_cloud_path, load_point_cloud, probe};

use crate::render::engine::render::layer_size;
use crate::render::engine::{text, Engine, EngineError};

/// 選んだ層の箱と輪郭。
mod bounds;
/// 層の種類ごとの絵。
mod sources;

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

/// 場が乗る形・文字の輪郭を刻む幅(px)。頂点段の場はこの間隔で曲がる。
pub(super) const FIELD_STEP: f32 = 4.0;

pub(super) struct TextTexture {
    texture: LayerContent,
    tolerance: f32,
    /// 場のために輪郭を刻んだ幅(None = 刻んでいない)。
    step: Option<f32>,
    frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
    bounds: Option<crate::render::media::SpatialBounds>,
}

impl Engine {
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
        let (_bytes, video) = self.videos.get(path).expect("直前に insert した");

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
        // 復号の流れ(texture)は層 × 素材で 1 本。別の時刻の合成を同じ frame で描く時は、engine が
        // 流れの名前空間を切り替える — 同じ流れで t′ と t を続けて復号すると、cache へ写す前に
        // texture が t で上書きされ、t′ の写しが t の絵になる。
        let stream_id = re_video::player::VideoPlayerStreamId(layer_stream_id(layer, path) ^ self.video_stream_namespace);
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
            // 揃ったコマはその場で写し、写しの方を渡す。復号器の texture は 1 本を毎コマ書き換えるので、
            // それを渡すと効果の焼き(入力 texture の handle が鍵)が「同じ入力」と見て前のコマの絵を返す
            // (Chroma Key を掛けた動画が 0 コマ目で止まった、2026-09-17)。
            let key = (path.to_owned(), pts);
            if !self.frame_cache.contains_key(&key) {
                self.remember_video_frame(path, pts, texture);
            }
            self.pending_frame_copies.retain(|(p, t, _)| (p, *t) != (&key.0, key.1));
            let tick = self.frame_cache_tick;
            if let Some(hit) = self.frame_cache.get_mut(&key) {
                hit.last_use = tick;
                return Ok((Some(LayerContent::Texture(hit.texture.clone())), natural));
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
            LayerSource::Camera | LayerSource::Stage | LayerSource::Null | LayerSource::Group | LayerSource::Particles => Ok((None, [0.0, 0.0])),
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
        morph: Option<(&TextDocument, f64)>,
        t: RationalTime,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self {
        let snapshot = |document: &TextDocument| serde_json::to_string(&(
            document.content.eval(t),
            document.justify,
            document.alignment,
            document.wrap_size,
            &document.styles,
            &document.runs,
        ))
        .unwrap_or_default();
        // 相手の文字と量も鍵: 相手が変われば組み直す。
        let content_snapshot = format!("{}|{}", snapshot(document), morph.map(|(d, amount)| format!("{amount}|{}", snapshot(d))).unwrap_or_default());
        Self {
            layer,
            canvas_width,
            canvas_height,
            content_snapshot,
        }
    }

    /// 字ごとのずれと避ける物も鍵: 折り返しの移り方の途中や物が動く間は、コマごとに組み直す。
    fn moving(mut self, flow: text::Flow<'_>) -> Self {
        if let Some(offsets) = flow.offsets {
            self.content_snapshot.push_str(&format!("|{offsets:?}"));
        }
        // 避ける物が動けば組み直す。
        if !flow.around.is_empty() {
            self.content_snapshot.push_str(&format!("|{:?}", flow.around));
        }
        self
    }
}

/// 輪郭の点だけを原点から伸ばす(1 枚だけの Repeater の変換は、線を引く前の点に掛かる)。
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
) -> Result<Option<crate::picture::shapes_ops::Canvas>, EngineError> {
    Ok(crate::picture::shapes_ops::content_canvas(shapes)?)
}

/// 点の直径(comp のピクセル)。層の属性になるまでの既定値。
const DEFAULT_POINT_SIZE: f32 = 2.0;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod rich_text_cache_tests {
    use super::*;
    use crate::doc::store::*;
    #[test]
    fn moving_a_style_boundary_invalidates_the_texture() {
        let style=|id,size|TextDocumentStyle{id:TextStyleId(id),font:FontRef::default(),size,fill:[1.0;4],line_height:None,tracking:0.0,axes:vec![],features:vec![]};
        let mut content=ContentTrack::new();content.insert(ContentKeyframe{t:RationalTime::ZERO,content:"AB".into()});
        let mut text=TextDocument{content,justify:TextJustify::Left,wrap_size:None,styles:vec![style(0,20.0),style(1,40.0)],slot_id:None,ranges:vec![],alignment:Default::default(),runs:vec![TextRun{len:1,style:TextStyleId(0)},TextRun{len:1,style:TextStyleId(1)}]};
        let before=TextCacheKey::new(LayerId(1),&text,None,RationalTime::ZERO,400,200);
        text.runs.reverse();
        assert!(before!=TextCacheKey::new(LayerId(1),&text,None,RationalTime::ZERO,400,200));
        // morph の相手と量も鍵。
        let partner=text.clone();
        assert!(TextCacheKey::new(LayerId(1),&text,Some((&partner,0.5)),RationalTime::ZERO,400,200)!=TextCacheKey::new(LayerId(1),&text,Some((&partner,0.6)),RationalTime::ZERO,400,200));
    }
}

/// 動画の時刻はコンポの時計で決まる。素材の fps がコンポと違っても、絵の速さは変わらない。
#[cfg(test)]
mod media_time_contract;

#[cfg(test)]
mod vector_projection_contract;
