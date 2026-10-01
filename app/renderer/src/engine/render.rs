#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ShapeNode, StoreView,
    TextDocument,
};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses, Window,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_cast_shadow, translate_clip, translate_effect_passes, translate_matte_mode, translate_point_displace,
};
use crate::render::compositor::effects::isf::TimeBase;
use crate::render::engine::{Engine, EngineError};

/// 1 コマ分の層を建てて焼く。
mod build;
/// 先読みと、見えない層の捨て方。
mod warm;

impl Engine {
    pub fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        self.layer_failures.clear();
        self.purge_idle_video_players();
        self.render_frame_graph_pixels(view, t, include_background, camera_override)
    }

    /// 効果が宣言した時刻のずれごとに、**その時刻の層の絵**を用意する。
    ///
    /// 効果が自分で前フレームを覚えるのは恒久禁止(`docs/plugin-resources.md` §6) — 追跡できなくなり、
    /// 純関数契約・フレーム並列・スクラブが壊れるため。ここは逆で、ホストが時刻を決めて渡すので
    /// `render_frame(t)` は純関数のまま。
    ///
    /// 「時刻 t の層の姿」を作るのは Document の resolve 1 箇所だけ。ここでは引き直した姿を使う
    /// (`source_time` だけを手でずらすと、mask やキーフレームは t のままの継ぎ接ぎになる)。
    /// 別の時刻 `at` の**合成**を 1 枚に描く(下の合成 / 自分の群 / comp 全体)。自分は除く(非再帰)。
    /// 組み立ての入れ子: 今の frame の失敗の記録と数は保ち、時計は呼び手が戻す。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn composite_at(
        &mut self,
        view: &StoreView<'_>,
        at: RationalTime,
        then: &[ResolvedLayer],
        texts: &HashMap<LayerId, TextDocument>,
        shapes: &HashMap<LayerId, Vec<ShapeNode>>,
        comp: CompSpec,
        exclude: LayerId,
        source: crate::render::compositor::TimeSource,
    ) -> Option<crate::render::compositor::GpuTexture2D> {
        use crate::render::compositor::TimeSource;
        let picked: Vec<ResolvedLayer> = match source {
            TimeSource::Own => return None,
            TimeSource::Below => {
                let end = then.iter().position(|l| l.id == exclude).unwrap_or(then.len());
                then[..end].to_vec()
            }
            TimeSource::Comp => then.iter().filter(|l| l.id != exclude).cloned().collect(),
            TimeSource::Group => {
                // 自分が群(板の持ち主)ならその子、そうでなければ自分の親の群の子。
                let owner_of_plate = then.iter().any(|l| l.plate == Some(exclude));
                let group = if owner_of_plate { Some(exclude) } else { view.attrs(exclude).ok().flatten().and_then(|a| a.parent) };
                let Some(group) = group else { return None };
                let descends = |id: LayerId| -> bool {
                    let mut cursor = Some(id);
                    for _ in 0..64 {
                        let Some(here) = cursor else { return false };
                        if here == group { return true; }
                        cursor = view.attrs(here).ok().flatten().and_then(|a| a.parent);
                    }
                    false
                };
                then.iter().filter(|l| l.id != exclude && (l.plate == Some(group) || descends(l.id))).cloned().collect()
            }
        };
        let background = match source {
            TimeSource::Group => crate::render::compositor::NO_BACKGROUND,
            _ => view.composition().ok().flatten().map_or(crate::render::compositor::NO_BACKGROUND, |c| c.background),
        };
        let camera = self.resolve_camera_in(view, then, at).ok()?;
        let failures = std::mem::take(&mut self.layer_failures);
        let drawn = self.drawn_layers;
        let was_nested = self.feedback_replaying;
        self.feedback_replaying = true;
        // 復号の流れを本番と分ける(同じ流れで t′ → t と復号すると t′ の写しが t の絵になる)。
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        (offset_key_of(at, view), source as u8).hash(&mut hasher);
        self.video_stream_namespace = hasher.finish() | 1;
        // feedback の状態も分ける: t′ の列は t′ の列で 1 歩ずつ進む(offset が一定なら順送り)。
        self.feedback_namespace = self.video_stream_namespace;
        let built = self.build_layers(view, comp, camera, camera, at, &picked, texts, shapes);
        self.feedback_namespace = 0;
        self.video_stream_namespace = 0;
        self.feedback_replaying = was_nested;
        self.drawn_layers = drawn;
        let mut nested = std::mem::replace(&mut self.layer_failures, failures);
        self.layer_failures.append(&mut nested);
        let layers = built.ok()?;
        let (texture, _view) = self.compositor.render_to_texture(comp, camera, &layers, background).ok()?;
        self.compositor.import_premultiplied(&texture).ok()
    }

    #[allow(clippy::too_many_arguments)]
    fn sources_at_other_times(
        &mut self,
        composites: &Composites,
        passes: &[EffectPass],
        layer: LayerId,
        others: &OtherTimes,
        now: (&[ResolvedLayer], &HashMap<LayerId, TextDocument>, &HashMap<LayerId, Vec<ShapeNode>>, RationalTime),
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
    ) -> Vec<Vec<crate::render::compositor::GpuTexture2D>> {
        if passes.iter().all(|pass| pass.image_time_offsets().is_empty() && pass.image_layers().is_empty()) {
            return Vec::new();
        }
        passes
            .iter()
            .map(|pass| {
                // 指した層の絵(同じ時刻)。自分自身と無い層は断る — 黙って今の絵で代用しない。
                let (resolved, texts, shapes, t) = now;
                let picked: Option<Vec<_>> = pass.image_layers().iter().map(|&target| {
                    let got = (|| {
                        if target == layer { return None; }
                        let mut then = resolved.iter().find(|l| l.id == target)?.clone();
                        then.id = lookbehind_layer_id(target, 0);
                        let (content, _, _) = self.texture_for_resolved(&then, texts, shapes, t, comp, camera, projection_camera).ok()?;
                        content.as_ref().and_then(|c| c.texture()).and_then(|t| self.compositor.snapshot_texture(t))
                    })();
                    if got.is_none() {
                        self.layer_failures.push(format!("指した層 {} の絵が無い(自分自身か、無い層)", target.0));
                    }
                    got
                }).collect();
                if !pass.image_layers().is_empty() {
                    return picked.unwrap_or_default();
                }
                pass.image_time_offsets()
                    .iter()
                    .enumerate()
                    .map(|(i, offset)| {
                        let source = pass.image_time_sources().get(i).copied().unwrap_or_default();
                        let key = time_key(*offset, pass.image_time_base(i), layer);
                        let got = (|| {
                        let (at, resolved, texts, shapes) = others.get(&key)?;
                        if source != crate::render::compositor::TimeSource::Own {
                            // 合成の t′ は組み立ての前に描いて写してある(復号 texture は層ごとに 1 本なので、
                            // 後から t′ で復号すると本番の t の絵まで巻き添えになる)。
                            let _ = (resolved, texts, shapes);
                            return composites.get(&(key, source, layer)).cloned();
                        }
                        let mut then = resolved.iter().find(|l| l.id == layer)?.clone();
                        // 別の流れとして読む。復号器の texture は層ごとに 1 本なので、同じ層として
                        // 読むと今の時刻の絵まで巻き添えで上書きされる(差が 0 になる)。
                        then.id = lookbehind_layer_id(layer, key);
                        let (content, _, _) = self
                            .texture_for_resolved(&then, texts, shapes, *at, comp, camera, projection_camera)
                            .ok()?;
                        // 素材の texture は時刻ごとに同じ 1 枚へ上書きされるので、使う分を写しておく。
                        content.as_ref().and_then(|c| c.texture()).and_then(|t| self.compositor.snapshot_texture(t))
                        })();
                        if got.is_none() {
                            // 届かなかった物を黙って「今の絵」で代用しない — 同じ時刻が辿り方で
                            // 変わる原因になる(それは時間参照を入れた意味を消す)。
                            self.layer_failures.push(format!("{offset} 秒前の絵が間に合わなかった"));
                        }
                        got
                    })
                    .collect::<Option<Vec<_>>>()
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 時計(TIME 系)は comp の時刻と fps から。壁時計は使わない — 同じ時刻は何度描いても同じ絵。
    fn stamp_clock(&mut self, view: &StoreView<'_>, t: RationalTime) {
        self.compositor.clock = view.composition().ok().flatten().map(|c| {
            let fps = c.fps;
            let frame = t.try_to_frame_round(fps).unwrap_or(0) as f32;
            [t.as_seconds_f64() as f32, fps.den() as f32 / fps.num() as f32, frame]
        });
    }

    /// 凍った層の絵(cache)があれば、その絵で層を組む。素材は復号せず、効果の列も走らない。
    /// 場・面の hook(頂点段)と配置・不透明度・blend・マットは生きたまま(法 §2-2)。
    fn frozen_layer(&mut self, view: &StoreView<'_>, layer: &ResolvedLayer, t: RationalTime, projection_camera: ResolvedCamera, blend_mode: CompositeBlendMode) -> Result<Option<(Layer, u32)>, EngineError> {
        if self.freezing == Some(layer.id) || layer.copy != 0 || layer.ghost { return Ok(None); }
        if !view.attrs(layer.id).ok().flatten().is_some_and(|a| a.frozen) { return Ok(None); }
        let Some(composition) = view.composition().ok().flatten() else { return Ok(None) };
        let Ok(comp_frame) = t.try_to_frame_round(composition.fps) else { return Ok(None) };
        let start = view.meta(layer.id).ok().flatten().map_or(0, |m| m.timing.start);
        let layer_frame = comp_frame - start;
        let Self { frozen, compositor, .. } = self;
        let Some(picture) = frozen.load(layer.id, layer_frame, &mut |bytes, w, h| compositor.upload_rgba16f("motolii-frozen", bytes, w, h).ok()) else { return Ok(None) };
        let outline = self.outline_id(layer.id);
        let shading = self.compositor.surface_shading_for(&layer.effects, false).map_err(EngineError::Store)?;
        Ok(Some((Layer {
            content: LayerContent::LinearTexture(picture.texture),
            size: picture.natural,
            placement: layer.placement,
            projection: layer.projection,
            projection_camera,
            blend_mode,
            shading,
            displace: translate_point_displace(&layer.effects),
            clip: translate_clip(&layer.effects),
            shadow: translate_cast_shadow(&layer.effects),
            outline,
            frame: picture.frame,
        }, picture.padding)))
    }

    /// Freeze の 1 コマを焼く: 層を本物で組み、効果の列の出口(乗算済み線形)を読み戻して cache へ。
    /// 順に呼ぶ(feedback は 1 歩ずつ進む)。絵にならない層(網・点群)は false。
    pub fn freeze_bake_frame(&mut self, view: &StoreView<'_>, layer_id: LayerId, comp_frame: i64) -> Result<bool, EngineError> {
        let composition = view.composition()
            .map_err(|error| EngineError::Store(error.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let t = RationalTime::try_from_frame(comp_frame, composition.fps)
            .map_err(|error| EngineError::Time(error.to_string()))?;
        let (scene, camera, comp, fps) = self.evaluate_frame_graph_semantics(view, t)?;
        let Some(mut target) = scene.layers.into_iter().find(|layer| layer.layer == layer_id && layer.freeze_eligible && !layer.ghost) else {
            return Ok(false);
        };

        // Freeze stores the material result before external coverage/composite
        // semantics. Placement/opacity/blend/matte stay live when the cache is read.
        target.matte = None;
        target.clip_to_below = false;
        target.blend = crate::doc::store::BlendMode::Normal;

        self.freezing = Some(layer_id);
        let previous_clock = self.compositor.clock;
        let frame = t.try_to_frame_round(fps).unwrap_or(comp_frame) as f32;
        self.compositor.clock = Some([
            t.as_seconds_f64() as f32,
            fps.den() as f32 / fps.num() as f32,
            frame,
        ]);
        let prepared = self.prepare_gpu_scene(
            &crate::frame_graph::SceneValue { layers: vec![target] },
            comp,
            camera,
        );
        self.compositor.clock = previous_clock;
        self.freezing = None;

        let Some(layer) = prepared?.layers.into_iter().next() else { return Ok(false) };
        let Some(picture) = self.layer_with_passes_linear_picture(&layer)? else { return Ok(false) };
        let uploaded = self.compositor.upload_rgba16f(
            "motolii-frozen",
            picture.bytes.clone(),
            picture.width,
            picture.height,
        )?;
        let start = view.meta(layer_id)
            .map_err(|error| EngineError::Store(error.to_string()))?
            .map_or(0, |meta| meta.timing.start);
        let frozen = super::frozen::FrozenFrame {
            texture: uploaded,
            natural: picture.natural,
            padding: picture.padding,
            frame: picture.frame,
        };
        self.frozen.remember(
            layer_id,
            comp_frame - start,
            frozen,
            Some(&picture.bytes),
        ).map_err(|error| EngineError::Store(format!("Freeze の cache を書けない: {error}")))?;
        Ok(true)
    }

    /// Freeze の cache の置き場(書類の隣)。None なら GPU の中だけ。
    pub fn set_cache_root(&mut self, root: Option<std::path::PathBuf>) { self.frozen.root = root; }
    /// 書類の path から cache の置き場(`<name>.motolii-cache`)。
    pub fn cache_root_for(path: &std::path::Path) -> Option<std::path::PathBuf> { super::frozen::FrozenStore::root_for_document(Some(path)) }
    /// 焼いている最中は disk に新しいコマが増える: 「無い」と覚えた物を忘れて、また見に行く。
    pub fn refresh_frozen(&mut self) { self.frozen.forget_missing(); }
    /// Unfreeze: 層の cache を捨てる。
    pub fn forget_frozen(&mut self, layer: LayerId) { self.frozen.forget(layer); }
    /// 凍った層の、disk にあるコマの数。
    pub fn frozen_frames_on_disk(&self, layer: LayerId) -> usize { self.frozen.frames_on_disk(layer) }
    pub fn frozen_frames_resident(&self, layer: LayerId) -> usize { self.frozen.resident_count(layer) }

    /// feedback を持つ pass に状態の鍵を刻み、この frame で見た鍵として覚える(辿り直しの要否を後で見る)。
    pub(super) fn stamp_feedback(&mut self, passes: &mut [EffectPass], layer: LayerId, copy: u32, chain: u8, screen: Option<[u32; 2]>) {
        super::translate::stamp_feedback(passes, layer, copy, chain, screen, self.feedback_namespace);
        // 本番の鍵だけ辿り直しの対象(別の時刻の列は自分の列で進む)。
        if self.feedback_namespace == 0 {
            self.feedback_keys_seen.extend(passes.iter().filter_map(|p| p.feedback));
        }
    }

    /// 層の組み立て + feedback の辿り直し。
    ///
    /// feedback は「入点を初期条件とする漸化式」(`docs/plugin-resources.md` §6-3)。組んだ後で、
    /// 初期条件から描かれてしまった状態(飛んで来た)があれば、直近の checkpoint(無ければ入点)から
    /// t の手前まで順に描いてから、t をもう一度組む。順再生と書き出しは 1 歩ずつなので何もしない。
    ///
    /// - 板の道(層の絵に焼く列)は**その層だけ**を辿り直す。
    /// - 画面の道(板に焼けない層・下の合成を読む列)は窓ごとに状態を持ち、**フレームを丸ごと**辿り直す。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layers_from_resolved(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        if self.feedback_replaying {
            self.prepare_blocks(view, comp, t, resolved)?;
            let mut layers = self.build_layers(view, comp, camera, projection_camera, t, resolved, text_documents, shape_documents)?;
            self.run_blocks(t, view.composition().ok().flatten().map_or(30.0, |c| c.fps.as_f64()));
            self.cut_moved_in_their_boxes(comp, camera, &mut layers)?;
            return Ok(layers);
        }
        self.compositor.feedback_set_revision(view.revision_key());
        self.feedback_keys_seen.clear();
        self.prepare_blocks(view, comp, t, resolved)?;
        let mut layers = self.build_layers(view, comp, camera, projection_camera, t, resolved, text_documents, shape_documents)?;
        let seen = std::mem::take(&mut self.feedback_keys_seen);
        if !self.replay_feedback(view, comp, camera, projection_camera, t, &seen)? {
            self.run_blocks(t, view.composition().ok().flatten().map_or(30.0, |c| c.fps.as_f64()));
            self.cut_moved_in_their_boxes(comp, camera, &mut layers)?;
            return Ok(layers);
        }
        // 辿り直しで状態が動いた: 焼いた絵は辿り直す前の物なので捨て、t をもう一度組む。
        let c = &mut self.compositor;
        c.baked_effects.clear(&mut c.effect_scratch);
        self.stamp_clock(view, t);
        self.prepare_blocks(view, comp, t, resolved)?;
        let mut layers = self.build_layers(view, comp, camera, projection_camera, t, resolved, text_documents, shape_documents)?;
        self.run_blocks(t, view.composition().ok().flatten().map_or(30.0, |c| c.fps.as_f64()));
        self.cut_moved_in_their_boxes(comp, camera, &mut layers)?;
        self.feedback_keys_seen.clear();
        Ok(layers)
    }

    /// 辿り直しが要る鍵があれば辿り直し、動かしたら true。
    #[allow(clippy::too_many_arguments)]
    fn replay_feedback(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
        t: RationalTime,
        seen: &[crate::render::compositor::FeedbackKey],
    ) -> Result<bool, EngineError> {
        let Some(composition) = view.composition().ok().flatten() else { return Ok(false) };
        let fps = composition.fps;
        let Ok(now) = t.try_to_frame_round(fps) else { return Ok(false) };
        let mut start: Option<i64> = None;
        let mut ids: HashSet<LayerId> = HashSet::new();
        // 別の時刻の合成がある frame は丸ごと辿り直す: t′ の列(名前空間付きの状態)も同じ歩で進める。
        let mut whole_frames = self.feedback_saw_composites;
        for &key in seen {
            let ok = match self.compositor.feedback_frame(key) {
                // 板の道はこの frame の組み立てで既に描かれている: 初期条件からでなければ正しい。
                Some(have) if have == now => !self.compositor.feedback_is_fresh(key),
                Some(have) if have + 1 == now => true,
                _ => false,
            };
            if ok { continue; }
            let in_point = view.meta(key.layer).ok().flatten().map_or(0, |m| m.timing.start);
            let from = self.compositor.feedback_restore(key, now - 1).map_or(in_point, |c| c + 1);
            if from >= now { continue; }
            ids.insert(key.layer);
            whole_frames |= key.screen.is_some();
            start = Some(start.map_or(from, |s: i64| s.min(from)));
        }
        let Some(start) = start else { return Ok(false) };
        let window = self.feedback_window;
        let background = composition.background;
        self.feedback_replaying = true;
        let result = (|| {
            for frame in start..now {
                // 辿り直しの歩は cache へ写さない: 復号の texture は 1 本で、写す前に次の歩が上書きする
                // (全部の歩の写しが最後の絵になる)。本番の t の写しは組み直しがもう一度登録する。
                self.pending_frame_copies.clear();
                let at = RationalTime::try_from_frame(frame, fps).map_err(|e| EngineError::Store(e.to_string()))?;
                let mut then = crate::picture::resolve::resolved_layers(view, at).map_err(|e| EngineError::Store(e.to_string()))?;
                if !whole_frames {
                    then.retain(|l| ids.contains(&l.id) || l.plate.is_some_and(|g| ids.contains(&g)));
                }
                let texts = collect_text_documents(view, &then, at)?;
                let shapes = collect_shape_documents(view, &then, at)?;
                // 作中カメラの窓(出力)は t′ のカメラ、Stage の窓は利用者のカメラのまま。
                let document_camera = self.resolve_camera_in(view, &then, at)?;
                let (camera, projection_camera) = match window {
                    Some(w) if w.projection_camera.is_some() => (camera, projection_camera),
                    Some(_) => (document_camera, document_camera),
                    None => (document_camera, document_camera),
                };
                let layers = self.build_layers(view, comp, camera, projection_camera, at, &then, &texts, &shapes)?;
                if !whole_frames {
                    self.compositor.effective_layer_textures(&layers)?;
                } else if let Some(window) = window {
                    let target = self.compositor.ctx.device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("motolii-feedback-replay-window"),
                        size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
                        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                        format: crate::render::compositor::PRESENTABLE_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    self.compositor.render_into_window(&target, comp, camera, &layers, background, window)?;
                } else {
                    self.compositor.render_to_texture(comp, camera, &layers, background)?;
                }
            }
            self.pending_frame_copies.clear();
            Ok(())
        })();
        self.feedback_replaying = false;
        result.map(|()| true)
    }

    pub fn render_resolved_to_texture(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_resolved_to_texture_with_shapes(
            view,
            comp,
            background,
            camera,
            t,
            resolved,
            text_documents,
            &HashMap::new(),
        )
    }

    pub fn render_resolved_to_texture_with_shapes(
        &mut self,
        view: &StoreView<'_>,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers =
            self.layers_from_resolved(view, comp, camera, camera, t, resolved, text_documents, shape_documents)?;
        Ok(self
            .compositor
            .render_to_texture(comp, camera, &layers, background)?)
    }

    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_frame_graph_to_texture_output(view, t, true)
    }

    pub fn render_frame_into(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
    ) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?.spec();
        let camera = self.frame_graph_document_camera(view, t)?;
        self.render_frame_graph_into_window(
            view,
            t,
            target,
            camera,
            true,
            &[],
            Window::output(comp),
            crate::frame_graph::ViewProjection::Camera,
        )
    }

    /// Render from an observation camera while retaining authored layer projection.
    pub fn render_frame_into_with_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
    ) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        self.render_frame_into_window(view, t, target, camera, include_background, outline, Window::output(comp))
    }

    /// 同じ世界を、出力寸法以外の窓へ(Stage のタブ: 寸法と関心域は窓が言う)。
    pub fn render_frame_into_window(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
        window: Window,
    ) -> Result<(), EngineError> {
        self.render_frame_graph_into_window(
            view,
            t,
            target,
            camera,
            include_background,
            outline,
            window,
            crate::frame_graph::ViewProjection::Camera,
        )
    }
}

// These adapters support the explicit `ResolvedLayer` oracle APIs below. The
// production render entry points evaluate a FrameGraph scene before lowering.
pub(super) fn collect_text_documents(view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) -> Result<HashMap<LayerId, TextDocument>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Text {
            if let Some(document) = crate::picture::resolve::text::resolved_text_document(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))? {
                documents.insert(layer.id, document);
            }
            let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
            if let Some((target, _)) = crate::extensions::text::morph(&effects) {
                if !documents.contains_key(&target) {
                    if let Some(document) = crate::picture::resolve::text::resolved_text_document(view, target, t).map_err(|error| EngineError::Store(error.to_string()))? {
                        documents.insert(target, document);
                    }
                }
            }
        }
    }
    Ok(documents)
}

pub(super) fn collect_shape_documents(view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = crate::picture::shapes::shapes_at(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))?;
            documents.insert(layer.id, shown_shapes(&shapes, layer));
        } else if layer.source == LayerSource::Group {
            if let Some(background) = crate::picture::boxes::background_shapes(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))? {
                documents.insert(layer.id, background);
            }
        }
    }
    Ok(documents)
}

pub(crate) fn shown_shapes(shapes: &[ShapeNode], layer: &ResolvedLayer) -> Vec<ShapeNode> {
    let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
    crate::extensions::pathop::with_effects(shapes, &effects)
}

pub(crate) fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 { layer.declared_size[0] } else { natural[0] },
        if layer.declared_size[1] > 0.0 { layer.declared_size[1] } else { natural[1] },
    ]
}

fn shifted_by_seconds(t: RationalTime, offset: f32) -> RationalTime {
    const DEN: i64 = 1000;
    let num = (offset as f64 * DEN as f64).round() as i64;
    let shifted = t.num().checked_mul(DEN)
        .and_then(|scaled| num.checked_mul(t.den()).map(|by| scaled + by))
        .zip(t.den().checked_mul(DEN))
        .and_then(|(num, den)| RationalTime::try_new(num, den).ok());
    match shifted {
        Some(at) if at.as_seconds_f64() >= 0.0 => at,
        _ => RationalTime::ZERO,
    }
}

/// 別の時刻ごとに引き直した「層の姿」。鍵はずれ(ミリ秒)。
type OtherTimes = BTreeMap<i64, (RationalTime, Vec<ResolvedLayer>, HashMap<LayerId, TextDocument>, HashMap<LayerId, Vec<ShapeNode>>)>;
/// 合体後の別時刻の写し: (時刻のずれ, 相手, 求めた層) → 絵。
type Composites = HashMap<(i64, crate::render::compositor::TimeSource, LayerId), crate::render::compositor::GpuTexture2D>;

/// 時刻のずれ(秒)を鍵にする — 同じずれは 1 回しか引かない。
/// 絶対時刻の鍵(ms)。復号の流れの名前空間に使う。
fn offset_key_of(at: RationalTime, _view: &StoreView<'_>) -> i64 { (at.as_seconds_f64() * 1000.0).round() as i64 }

fn offset_key(offset: f32) -> i64 {
    (offset as f64 * 1000.0).round() as i64
}

/// 別の時刻の鍵: ずれ(ms)か、層ごとの絶対時刻(入点からの ms に層の番号を混ぜ、上の bit で区別)。
fn time_key(offset: f32, base: TimeBase, layer: LayerId) -> i64 {
    match base {
        TimeBase::Offset => return offset_key(offset),
        // コマ数の鍵は秒の鍵と混ざらないよう上の bit で分ける(-1 コマと -0.001 秒は別の時刻)。
        TimeBase::Frames => return offset.round() as i64 | 1 << 60,
        TimeBase::At => {}
    }
    let mut hasher = std::hash::DefaultHasher::new();
    use std::hash::{Hash as _, Hasher as _};
    layer.0.hash(&mut hasher);
    offset_key(offset).hash(&mut hasher);
    (hasher.finish() >> 2) as i64 | 1 << 61
}

/// 層の入点(comp の時刻)。TIME_AT はここからの秒。
fn layer_in_point(view: &StoreView<'_>, layer: LayerId) -> RationalTime {
    let start = view.meta(layer).ok().flatten().map_or(0, |m| m.timing.start);
    view.composition().ok().flatten().and_then(|c| RationalTime::try_from_frame(start, c.fps).ok()).unwrap_or(RationalTime::ZERO)
}

/// 別の時刻を読むための層の番号。復号器の流れを本体と分けるためだけの物で、Document には無い。
fn lookbehind_layer_id(layer: LayerId, key: i64) -> LayerId {
    let mut hasher = std::hash::DefaultHasher::new();
    use std::hash::{Hash as _, Hasher as _};
    layer.0.hash(&mut hasher);
    key.hash(&mut hasher);
    LayerId(hasher.finish() | 1 << 63)
}

/// 別の時刻を読む効果は、**たどり着き方で絵が変わってはいけない**(実 GPU)。
///
/// 効果が自分で前フレームを覚える道を恒久禁止している理由がここ(`docs/plugin-resources.md` §6)。
/// ホストが時刻を渡す形なら、同じ時刻は何度描いても、どの順で描いても同じ絵になる。
#[cfg(test)]
mod time_reference_is_deterministic;

/// feedback(前のフレームを保つ効果)は**入点を初期条件とする漸化式**(`docs/plugin-resources.md` §6-3)。
/// 効果は覚えない — host が状態を持ち、飛んで来ても入点(か checkpoint)から辿り直すので、
/// 同じ時刻は何度描いても、どの順で描いても同じ絵(実 GPU)。
#[cfg(test)]
mod feedback_is_a_recurrence_from_the_in_point;

/// 合体後の別時刻(`SOURCE: below / comp`)。下の合成を t′ で読む効果は、ホストが t′ の下の層たちを
/// 描いて渡す。だから「下の合成の 0.5 秒前」は、下の層だけの書類を 0.5 秒前に描いた絵と同じで、
/// 飛んでも辿っても同じ(`docs/plugin-resources.md` §6-1 CompLookbehind、非再帰)。
#[cfg(test)]
mod composite_at_another_time;

/// Freeze(docs/freeze-and-flatten.md §2-6): 凍っても絵は変わらない、飛んでも辿っても同じ、Unfreeze で戻る。
#[cfg(test)]
mod freeze_keeps_the_picture;
