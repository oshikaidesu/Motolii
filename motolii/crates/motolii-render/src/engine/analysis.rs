//! 解析の入力を解く host の側(2026-09-14 利用者裁定「解析は書類に入れないキャッシュ、resolve は入力として読むだけ」)。
//! 層を本物で組んで読み戻し(Freeze と同じ道)、Blob Track の塊をコマごとに解いて `AnalysisInputs` に置く。
//! ID を持続する・動きで拾う時は入点から 1 コマずつ解き、解いたコマは書類の版が変わるまで持つ(飛んでも辿っても同じ塊)。

use std::collections::BTreeMap;

use crate::doc::core::CompSpec;
use crate::doc::store::analysis::{AnalysisInputs, BlobMark};
use crate::doc::store::{blob, overlay, EffectId, LayerId, RationalTime, ResolvedLayer, StoreView};
use crate::render::compositor::LayerContent;
use crate::render::engine::render::{collect_shape_documents, collect_text_documents};
use crate::render::engine::{Engine, EngineError};
use crate::render::media::blob::{detect, mask, BlobSettings, BlobSource, BlobTracker};

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

#[derive(Default)]
pub(crate) struct BlobTrackState {
    key: u64,
    /// 次に解くコマ(入点から順に解く時)。
    next_frame: i64,
    tracker: BlobTracker,
    previous: Option<(Vec<u8>, u32, u32)>,
    marks: BTreeMap<i64, Vec<BlobMark>>,
    /// Show Mask の時だけ、コマごとの二値。
    masks: BTreeMap<i64, (Vec<u8>, u32, u32)>,
}

impl Engine {
    /// 1 枚の層を本物で組み、効果の列の出口を乗算済み線形で読み戻す。絵にならない層(網・点群)は None。
    pub(super) fn layer_linear_picture(&mut self, view: &StoreView<'_>, resolved: &[ResolvedLayer], target: &ResolvedLayer, t: RationalTime, comp: CompSpec) -> Result<Option<LinearPicture>, EngineError> {
        let texts = collect_text_documents(view, std::slice::from_ref(target), t)?;
        let shapes = collect_shape_documents(view, std::slice::from_ref(target), t)?;
        let camera = self.resolve_camera_in(view, resolved, t)?;
        let Some(lwp) = self.layers_from_resolved(view, comp, camera, camera, t, std::slice::from_ref(target), &texts, &shapes)?.into_iter().next() else { return Ok(None) };
        let (textures, paddings, _spills, checked_out) = self.compositor.effective_layer_textures(std::slice::from_ref(&lwp))?;
        let Some(texture) = textures.first().and_then(|c| c.texture()).cloned() else { return Ok(None) };
        let raw = self.compositor.ctx.gpu_resources.textures.get_from_handle(texture.handle()).map_err(|e| EngineError::Store(e.to_string()))?.texture.clone();
        // 効果の列の出口は乗算済み線形の Rgba16Float。列が空の層は素材のまま(非乗算 sRGB 等)なので同じ空間へ写す。
        let linear = matches!(&textures[0], LayerContent::LinearTexture(_)) || raw.format().is_srgb();
        let (half, owned) = if raw.format() == wgpu::TextureFormat::Rgba16Float {
            (raw, None)
        } else {
            let mut encoder = self.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-linear-picture-encode") });
            let converted = self.compositor.convert_image_encoding(&mut encoder, &raw, true, !linear, linear);
            self.compositor.pending.push(encoder.finish());
            (converted.clone(), Some(converted))
        };
        let bytes = self.compositor.read_texture_bytes(&half)?;
        for (w, h, f, tx) in checked_out { self.compositor.effect_scratch.release(w, h, f, tx); }
        let (width, height) = (half.width(), half.height());
        if let Some(owned) = owned { self.compositor.effect_scratch.release(owned.width(), owned.height(), owned.format(), owned); }
        Ok(Some(LinearPicture { bytes, width, height, natural: lwp.layer.size, padding: paddings[0], frame: lwp.layer.frame }))
    }

    /// 解析の入力を解いてから、それを読む view で resolve する。解析の要る層が無ければ素の resolve と同じ。
    pub(super) fn resolved_with_analysis(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<Vec<ResolvedLayer>, EngineError> {
        let inputs = self.analysis_inputs(view, t)?;
        let resolve = |view: StoreView<'_>| view.resolved_layers(t).map_err(|e| EngineError::Store(e.to_string()));
        if inputs.is_empty() { resolve(view.clone()) } else { resolve(view.clone().with_analysis(&inputs)) }
    }

    /// 連続性の物差しの標本: 解析を読んだ view で解き、層の箱の角と文字の字の位置を画面の平面(px)で返す。
    /// 鍵は `L<id> 名前.min` / `.max` / `.g<n>`(n は組んだ順の字)。関係の動きがコマごとに跳ばないかを測る。
    pub fn continuity_samples(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<Vec<(String, [f32; 2])>, EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let inputs = self.analysis_inputs(view, t)?;
        let view = if inputs.is_empty() { view.clone() } else { view.clone().with_analysis(&inputs) };
        let Some(comp) = view.composition().map_err(store)? else { return Ok(Vec::new()) };
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        let mut out = Vec::new();
        for layer in view.resolved_layers(t).map_err(store)? {
            // 見えない層(Opacity 0 の解析係など)の箱は動きとして読まない。
            if layer.ghost || layer.copy != 0 || layer.placement.opacity <= 0.0 || matches!(layer.source, crate::doc::store::LayerSource::Camera | crate::doc::store::LayerSource::Stage | crate::doc::store::LayerSource::Null) {
                continue;
            }
            let to_screen = |p: glam::Vec2| match layer.placement.world_transform {
                Some(world) => world.transform_point3(p.extend(0.0)).truncate().to_array(),
                None => layer.placement.transform.transform_point2(p).to_array(),
            };
            let name = view.attrs(layer.id).map_err(store)?.unwrap_or_default().name;
            if let Some(b) = view.layer_box(layer.id, t).map_err(store)? {
                out.push((format!("L{} {name}.min", layer.id.0), to_screen(glam::vec2(b[0], b[1]))));
                out.push((format!("L{} {name}.max", layer.id.0), to_screen(glam::vec2(b[2], b[3]))));
            }
            if layer.source == crate::doc::store::LayerSource::Text {
                if let Some(document) = view.resolved_text_document(layer.id, t).map_err(store)? {
                    if let Ok(Some(shaped)) = crate::doc::store::text_frame::shape_document(&document, t, &canvas) {
                        let mut n = 0;
                        for line in &shaped.lines {
                            for x in &line.glyph_xs {
                                let d = layer.glyph_offsets.as_ref().and_then(|o| o.get(n).copied()).unwrap_or([0.0, 0.0]);
                                out.push((format!("L{} {name}.g{n}", layer.id.0), to_screen(glam::vec2(*x + d[0], line.baseline_y + d[1]))));
                                n += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    fn analysis_inputs(&mut self, view: &StoreView<'_>, t: RationalTime) -> Result<AnalysisInputs, EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let mut inputs = AnalysisInputs::default();
        let Some(composition) = view.composition().map_err(store)? else { return Ok(inputs) };
        let Ok(frame) = t.try_to_frame_round(composition.fps) else { return Ok(inputs) };
        let mut seen = Vec::new();
        self.overlay_frames.clear();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer).map_err(store)? else { continue };
            if !meta.timing.covers(frame) { continue; }
            if let crate::doc::store::LayerSource::File { path, .. } = &meta.source {
                if let Some(extent) = self.material_extent(path, composition.spec()) {
                    inputs.set_extent(path, extent);
                }
            }
            let effects = view.resolved_effects(layer, t).map_err(store)?;
            // Blob Track は指した層を、Track Overlay は下の合成を読む。
            let (params, source, settings, detail, show_mask, overlay) = if let Some(effect) = effects.iter().find(|e| blob::is_blob_track(&e.plugin_id)) {
                let source = LayerId(blob::number_of(&effect.params, "source").round().max(0.0) as u64);
                if source.0 == 0 || source == layer { continue; }
                (effect.params.clone(), Source::Layer(source), settings_of(&effect.params), blob::number_of(&effect.params, "detail"), false, false)
            } else if let Some(effect) = effects.iter().find(|e| overlay::is_track_overlay(&e.plugin_id)) {
                (effect.params.clone(), Source::Below(layer), overlay_settings_of(&effect.params), overlay::number_of(&effect.params, "detail"), overlay::switch_of(&effect.params, "show_mask"), true)
            } else {
                continue;
            };
            seen.push(layer);
            let detail = detail.round().clamp(120.0, 3840.0) as u32;
            let key = {
                use std::hash::{Hash, Hasher};
                let mut h = std::hash::DefaultHasher::new();
                format!("{params:?}").hash(&mut h);
                view.revision_key().hash(&mut h);
                h.finish()
            };
            let mut state = self.blob_tracks.remove(&layer).unwrap_or_default();
            if state.key != key {
                state = BlobTrackState { key, next_frame: meta.timing.start, ..Default::default() };
            }
            let sequential = settings.persist || matches!(settings.source, BlobSource::Motion { .. });
            // 移り方は少し前の時刻でも並べ直すので、その時刻の塊も置く(無いと前の時刻は避けない並びになる)。
            let reach = view.transition_reach(t).map_err(store)?;
            if !overlay && reach > 0 && !sequential {
                for f in (frame - reach).max(meta.timing.start)..frame {
                    if !state.marks.contains_key(&f) {
                        let marks = self.blob_frame(view, source, f, &settings, detail, show_mask, composition.spec(), composition.fps, &mut state)?;
                        state.marks.insert(f, marks);
                    }
                }
            }
            if !state.marks.contains_key(&frame) {
                if sequential {
                    if state.next_frame > frame || state.next_frame < meta.timing.start {
                        state = BlobTrackState { key, next_frame: meta.timing.start, ..Default::default() };
                    }
                    for f in state.next_frame..=frame {
                        let marks = self.blob_frame(view, source, f, &settings, detail, show_mask, composition.spec(), composition.fps, &mut state)?;
                        state.marks.insert(f, marks);
                    }
                    state.next_frame = frame + 1;
                } else {
                    let marks = self.blob_frame(view, source, frame, &settings, detail, show_mask, composition.spec(), composition.fps, &mut state)?;
                    state.marks.insert(frame, marks);
                }
            }
            let marks = state.marks.get(&frame).cloned().unwrap_or_default();
            if overlay {
                self.overlay_frames.insert(layer, OverlayFrame { marks, mask: state.masks.get(&frame).cloned(), params });
            } else {
                for f in (frame - reach).max(meta.timing.start)..frame {
                    if let (Some(past), Ok(at)) = (state.marks.get(&f), RationalTime::try_from_frame(f, composition.fps)) {
                        inputs.set_blobs(layer, EffectId(0), at, past.clone());
                    }
                }
                inputs.set_blobs(layer, EffectId(0), t, marks);
            }
            self.blob_tracks.insert(layer, state);
        }
        self.blob_tracks.retain(|layer, _| seen.contains(layer));
        Ok(inputs)
    }

    /// 下の合成(自分より下の層たち、背景込み)を、効果の出口と同じ乗算済み線形で読み戻す。
    fn below_picture(&mut self, view: &StoreView<'_>, layer: LayerId, at: RationalTime, comp: CompSpec) -> Result<Option<LinearPicture>, EngineError> {
        let resolved = view.resolved_layers(at).map_err(|e| EngineError::Store(e.to_string()))?;
        let texts = collect_text_documents(view, &resolved, at)?;
        let shapes = collect_shape_documents(view, &resolved, at)?;
        let Some(composite) = self.composite_at(view, at, &resolved, &texts, &shapes, comp, layer, crate::render::compositor::TimeSource::Below) else { return Ok(None) };
        let raw = self.compositor.ctx.gpu_resources.textures.get_from_handle(composite.handle()).map_err(|e| EngineError::Store(e.to_string()))?.texture.clone();
        let mut encoder = self.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-below-picture-encode") });
        let converted = self.compositor.convert_image_encoding(&mut encoder, &raw, true, !raw.format().is_srgb(), true);
        self.compositor.pending.push(encoder.finish());
        let bytes = self.compositor.read_texture_bytes(&converted)?;
        let (width, height) = (converted.width(), converted.height());
        self.compositor.effect_scratch.release(width, height, converted.format(), converted);
        Ok(Some(LinearPicture { bytes, width, height, natural: [comp.width as f32, comp.height as f32], padding: 0, frame: None }))
    }

    /// 1 コマ: 元の絵を読み戻し、縮めて塊を拾い、ID を振って comp の座標へ戻す。
    #[allow(clippy::too_many_arguments)]
    fn blob_frame(&mut self, view: &StoreView<'_>, source: Source, frame: i64, settings: &BlobSettings, detail: u32, keep_mask: bool, comp: CompSpec, fps: crate::doc::store::Fps, state: &mut BlobTrackState) -> Result<Vec<BlobMark>, EngineError> {
        let at = RationalTime::try_from_frame(frame, fps).map_err(|e| EngineError::Time(e.to_string()))?;
        let (picture, transform) = match source {
            Source::Layer(id) => {
                let resolved = view.resolved_layers(at).map_err(|e| EngineError::Store(e.to_string()))?;
                let Some(target) = resolved.iter().find(|l| l.id == id && l.copy == 0 && !l.ghost).cloned() else {
                    // 元の層が居ないコマ: 塊は無いまま 1 歩進める(見失いの数え)。
                    state.previous = None;
                    state.tracker.step(Vec::new(), settings);
                    return Ok(Vec::new());
                };
                // 解析は元の層そのものの絵を読む。マットやクリップは合成の属性で、その相手がこの塊に依ることもある(動画を箱で切る)。
                let target = ResolvedLayer { matte: None, clip_to_below: false, ..target };
                (self.layer_linear_picture(view, &resolved, &target, at, comp)?, target.placement.transform)
            }
            Source::Below(layer) => (self.below_picture(view, layer, at, comp)?, glam::Affine2::IDENTITY),
        };
        let Some(picture) = picture else { return Ok(Vec::new()) };
        let (pixels, width, height, shrink) = shrink_to_srgb(&picture, detail);
        // 論理 px ↔ 縮めた絵の px。
        let per_logical = picture.width as f32 / (picture.natural[0] + 2.0 * picture.padding as f32).max(1.0) / shrink as f32;
        let to_local = |p: [f32; 2]| glam::vec2(p[0] / per_logical - picture.padding as f32, p[1] / per_logical - picture.padding as f32);
        let comp_per_local = transform.matrix2.x_axis.length().max(1e-6);
        let small = |px: f32| px / comp_per_local * per_logical;
        let area = |a: u32| (a as f64 * f64::from(small(1.0)).powi(2)).round().min(u32::MAX as f64) as u32;
        let scaled = BlobSettings {
            min_area: area(settings.min_area),
            max_area: area(settings.max_area),
            max_move: small(settings.max_move),
            separation: small(settings.separation as f32).round() as u32,
            blur: small(settings.blur as f32).round() as u32,
            ..*settings
        };
        let previous = state.previous.as_ref().filter(|(_, w, h)| *w == width && *h == height).map(|(p, _, _)| p.as_slice());
        if keep_mask {
            let bits = mask(&pixels, width, height, previous, &scaled);
            state.masks.insert(frame, (bits.iter().map(|b| u8::from(*b) * 255).collect(), width, height));
        }
        let regions = detect(&pixels, width, height, previous, &scaled);
        let blobs = state.tracker.step(regions, &scaled);
        state.previous = Some((pixels, width, height));
        Ok(blobs.into_iter().map(|b| {
            let corners = [[b.region.min[0] as f32, b.region.min[1] as f32], [b.region.max[0] as f32 + 1.0, b.region.min[1] as f32], [b.region.min[0] as f32, b.region.max[1] as f32 + 1.0], [b.region.max[0] as f32 + 1.0, b.region.max[1] as f32 + 1.0]]
                .map(|c| transform.transform_point2(to_local(c)));
            let lo = corners.iter().fold(glam::Vec2::splat(f32::MAX), |a, c| a.min(*c));
            let hi = corners.iter().fold(glam::Vec2::splat(f32::MIN), |a, c| a.max(*c));
            let center = transform.transform_point2(to_local(b.region.center));
            BlobMark { id: b.id, center: center.into(), size: (hi - lo).into(), age: b.age }
        }).collect())
    }
}

/// 解析する絵の出所。
#[derive(Clone, Copy)]
enum Source {
    /// その層そのもの(Blob Track)。
    Layer(LayerId),
    /// その層より下の合成(Track Overlay、調整層と同じ)。
    Below(LayerId),
}

/// Track Overlay のこのコマの塊と取っ手(描く側が箱・印を組む)。
pub(crate) struct OverlayFrame {
    pub(crate) marks: Vec<BlobMark>,
    /// Show Mask の二値(縮めた解析の絵の寸法)。
    pub(crate) mask: Option<(Vec<u8>, u32, u32)>,
    pub(crate) params: Vec<(String, crate::doc::store::Value)>,
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

fn settings_of(params: &[(String, crate::doc::store::Value)]) -> BlobSettings {
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
fn shrink_to_srgb(picture: &LinearPicture, long_side: u32) -> (Vec<u8>, u32, u32, u32) {
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
