//! Document→レンダグラフ(D3 / F-3凍結順序)。
//!
//! 評価順(発明しない): source(TimeMap) → effect stack → transform → clipping mask → group composite。
//! グループのエフェクトは子合成後の1枚へ。変形は子へ継承(グループ1枚の事後リサンプルなし)。

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use motolii_core::{FrameDesc, RationalTime, TimeMapError};
use motolii_eval::{DataTracks, Value};
use motolii_nodes::{CanonicalPoint, CanonicalSize, ClippingMaskMode, CompositeMode, RectOverlay};
use motolii_plugin::{NodeDesc, PluginId, PluginRegistry, ResolvedParams};
use motolii_render::{LinearRenderGraph, RenderStep, SolidSource, TextureId};

use crate::affine::{compose_transform, resolve_transform, Affine2D};
use crate::eval_time::EvaluationTime;
use crate::param_eval::{
    eval_color, eval_doc_param, eval_f64, eval_vec2, ParamEvalError, ResolvedLayerParams,
};
use crate::schema::{
    BlendMode, Clip, ClipSource, Group, ItemEnvelope, MaskMode, TrackItem, Transform2D,
};
use crate::{AssetId, Document, LayerId};

/// M1互換の矩形 LayerSource(プラグインID文字列。レジストリ未登録でも D3 が OverlayRect へ落とす)。
pub const RECT_LAYER_SOURCE: &str = "doc.layer_source.rect";
pub const CLEAR_LAYER_SOURCE: &str = "core.layer_source.clear";

#[derive(Debug, Clone, PartialEq)]
pub struct VideoSlot {
    pub texture_id: TextureId,
    pub asset: AssetId,
    pub source_time: RationalTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentFrameGraph {
    pub graph: LinearRenderGraph,
    pub video_slots: Vec<VideoSlot>,
    /// 代表 source_time(先頭の video slot。無ければ timeline)。
    pub source_time: RationalTime,
}

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("no video source clip in document")]
    NoVideoSource,
    #[error("asset {0} has no resolvable path")]
    UnresolvedAsset(u64),
    #[error("clip layer {layer}: {source}")]
    InvalidClip {
        layer: u64,
        #[source]
        source: TimeMapError,
    },
    #[error("param eval layer {layer}: {source}")]
    ParamEval {
        layer: u64,
        #[source]
        source: ParamEvalError,
    },
    #[error("singular transform (non-invertible) on layer {0}")]
    SingularTransform(u64),
    #[error("plugin {plugin_id}: {source}")]
    Plugin {
        plugin_id: String,
        #[source]
        source: motolii_plugin::PluginError,
    },
    #[error("unsupported clip source plugin: {0}")]
    UnsupportedSourcePlugin(String),
    #[error("VectorRecipe clip is not rasterized in D3 v1 (layer {0}); use Plugin rect or Asset")]
    UnsupportedVectorSource(u64),
    #[error("rect layer source missing param `{param}` (layer {layer})")]
    MissingRectParam { layer: u64, param: &'static str },
}

/// ガード10: relative → absolute → same-name → hash。実在ファイルのみ返す。
pub fn resolve_asset_path(asset: &crate::Asset, project_root: Option<&Path>) -> Option<PathBuf> {
    if let (Some(root), Some(rel)) = (project_root, asset.path_project_relative.as_deref()) {
        let p = root.join(rel);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(abs) = asset.path_absolute.as_deref() {
        let p = PathBuf::from(abs);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(root) = project_root {
        if let Some(name) = asset.file_name.as_deref() {
            let p = root.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
        if !asset.content_hash.is_empty() {
            let hashed = root.join(".motolii/media").join(&asset.content_hash);
            if hashed.is_file() {
                return Some(hashed);
            }
            if let Some(name) = asset.file_name.as_deref() {
                let ext = Path::new(name).extension();
                let mut with_ext = hashed.clone();
                if let Some(ext) = ext {
                    with_ext.set_extension(ext);
                    if with_ext.is_file() {
                        return Some(with_ext);
                    }
                }
            }
        }
    }
    None
}

pub fn build_document_frame_graph(
    doc: &Document,
    eval: EvaluationTime,
    desc: FrameDesc,
    data_tracks: &DataTracks,
    registry: &PluginRegistry,
    project_root: Option<&Path>,
) -> Result<DocumentFrameGraph, GraphError> {
    let any_solo = document_has_solo(doc);
    let mut b = GraphBuilder::new(
        doc,
        desc,
        eval.timeline_time,
        data_tracks,
        registry,
        any_solo,
    );
    let output = b.build_document(project_root)?;
    let source_time = b
        .video_slots
        .first()
        .map(|s| s.source_time)
        .unwrap_or(eval.timeline_time);
    Ok(DocumentFrameGraph {
        graph: LinearRenderGraph {
            desc,
            steps: b.steps,
            output,
        },
        video_slots: b.video_slots,
        source_time,
    })
}

struct GraphBuilder<'a> {
    doc: &'a Document,
    timeline_time: RationalTime,
    tracks: &'a DataTracks,
    registry: &'a PluginRegistry,
    any_solo: bool,
    frame_desc: FrameDesc,
    steps: Vec<RenderStep>,
    next_id: usize,
    transparent_id: Option<TextureId>,
    video_slots: Vec<VideoSlot>,
    resolved_layers: ResolvedLayerParams,
}

impl<'a> GraphBuilder<'a> {
    fn new(
        doc: &'a Document,
        desc: FrameDesc,
        timeline_time: RationalTime,
        tracks: &'a DataTracks,
        registry: &'a PluginRegistry,
        any_solo: bool,
    ) -> Self {
        Self {
            doc,
            timeline_time,
            tracks,
            registry,
            any_solo,
            frame_desc: desc,
            steps: Vec::new(),
            next_id: 0,
            transparent_id: None,
            video_slots: Vec::new(),
            resolved_layers: ResolvedLayerParams::default(),
        }
    }

    fn alloc_id(&mut self) -> TextureId {
        let id = TextureId(self.next_id);
        self.next_id += 1;
        id
    }

    fn transparent(&mut self) -> TextureId {
        if let Some(id) = self.transparent_id {
            return id;
        }
        let id = self.alloc_id();
        self.steps.push(RenderStep::SolidSource {
            output: id,
            source: SolidSource {
                color: [0.0, 0.0, 0.0, 0.0],
                time_map: motolii_core::TimeMap::identity(),
                reports_source_time: false,
            },
        });
        self.transparent_id = Some(id);
        id
    }

    fn should_draw(&self, env: &ItemEnvelope) -> bool {
        // lock は描画無影響(B④)。
        if !env.visible {
            return false;
        }
        if self.any_solo && !env.solo {
            return false;
        }
        true
    }

    fn build_document(&mut self, _: Option<&Path>) -> Result<TextureId, GraphError> {
        let mut acc = self.transparent();
        let mut prev_mask = None;
        let items: Vec<&TrackItem> = self
            .doc
            .tracks
            .iter()
            .flat_map(|t| t.items.iter())
            .collect();
        for (i, item) in items.iter().enumerate() {
            let layer = item_layer_id(item);
            let env = item_envelope(item);
            let next_needs_mask = items
                .get(i + 1)
                .map(|n| item_envelope(n).clipping_mask.enabled)
                .unwrap_or(false);
            let draw = self.should_draw(env);
            // B④: visible=false でもマスク/LookAt 用に評価。solo 除外かつマスク不要なら画素は作らない。
            if !draw && !next_needs_mask {
                self.record_layer_position(layer, &env.transform)?;
                continue;
            }
            let tex = self.build_item(item, Affine2D::IDENTITY, prev_mask, layer)?;
            let fg = self.apply_envelope_opacity(tex, env, layer)?;
            if draw {
                if acc == self.transparent_id.unwrap() {
                    acc = fg;
                } else {
                    acc = self.composite(acc, fg, env.blend);
                }
            }
            prev_mask = Some(fg);
        }
        Ok(acc)
    }

    fn ensure_video_slot(
        &mut self,
        asset: AssetId,
        source_time: RationalTime,
    ) -> Result<TextureId, GraphError> {
        // 同一 Asset でも TimeMap が異なれば別スロット(slot 単位の source_time)。
        let id = self.alloc_id();
        self.steps.push(RenderStep::VideoSource { output: id });
        self.video_slots.push(VideoSlot {
            texture_id: id,
            asset,
            source_time,
        });
        Ok(id)
    }

    fn build_item(
        &mut self,
        item: &TrackItem,
        inherited: Affine2D,
        mask_below: Option<TextureId>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        match item {
            TrackItem::Clip(c) => self.build_clip(c, inherited, mask_below, layer),
            TrackItem::Group(g) => self.build_group(g, inherited, mask_below, layer),
        }
    }

    fn build_group(
        &mut self,
        group: &Group,
        inherited: Affine2D,
        mask_below: Option<TextureId>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        let local = self.resolve_xform(&group.envelope.transform, layer)?;
        let child_xform = compose_transform(inherited, local);
        let mut acc = self.transparent();
        let mut prev_child = None;
        for (i, child) in group.children.iter().enumerate() {
            let child_layer = item_layer_id(child);
            let env = item_envelope(child);
            let next_needs_mask = group
                .children
                .get(i + 1)
                .map(|n| item_envelope(n).clipping_mask.enabled)
                .unwrap_or(false);
            let draw = self.should_draw(env);
            if !draw && !next_needs_mask {
                self.record_layer_position(child_layer, &env.transform)?;
                continue;
            }
            let tex = self.build_item(child, child_xform, prev_child, child_layer)?;
            let fg = self.apply_envelope_opacity(tex, env, child_layer)?;
            if draw {
                if acc == self.transparent_id.unwrap() {
                    acc = fg;
                } else {
                    acc = self.composite(acc, fg, env.blend);
                }
            }
            prev_child = Some(fg);
        }
        // F-3: 子合成 → グループ effect stack → clipping mask。変形は継承済み。
        let mut tex = acc;
        for effect in &group.envelope.effects {
            if effect.enabled {
                tex = self.apply_effect(tex, effect, layer)?;
            }
        }
        if group.envelope.clipping_mask.enabled {
            if let Some(mask) = mask_below {
                tex = self.apply_mask(tex, mask, group.envelope.clipping_mask.mode);
            }
        }
        self.record_layer_position(layer, &group.envelope.transform)?;
        Ok(tex)
    }

    fn build_clip(
        &mut self,
        clip: &Clip,
        inherited: Affine2D,
        mask_below: Option<TextureId>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        // OverrunMode: v1 は Freeze のみ。active 窓の外でも Black/Loop を黙って通さない。
        clip.time_map
            .require_freeze_overrun()
            .map_err(|e| GraphError::InvalidClip {
                layer: layer.get(),
                source: e,
            })?;
        if !clip_active(clip, self.timeline_time) {
            self.record_layer_position(layer, &clip.envelope.transform)?;
            return Ok(self.transparent());
        }
        let local_t =
            self.timeline_time
                .try_sub(clip.start)
                .map_err(|e| GraphError::InvalidClip {
                    layer: layer.get(),
                    source: e.into(),
                })?;
        let st = clip
            .time_map
            .try_map(local_t)
            .map_err(|e| GraphError::InvalidClip {
                layer: layer.get(),
                source: e,
            })?;
        let local = self.resolve_xform(&clip.envelope.transform, layer)?;
        let world = compose_transform(inherited, local);
        // F-3: source → effect stack → transform → clipping mask
        let mut tex = self.build_source(clip, st, layer)?;
        for effect in &clip.envelope.effects {
            if effect.enabled {
                tex = self.apply_effect(tex, effect, layer)?;
            }
        }
        tex = self.apply_world_transform(tex, world, layer)?;
        if clip.envelope.clipping_mask.enabled {
            if let Some(mask) = mask_below {
                tex = self.apply_mask(tex, mask, clip.envelope.clipping_mask.mode);
            }
        }
        self.record_layer_position(layer, &clip.envelope.transform)?;
        Ok(tex)
    }

    fn resolve_xform(&self, xform: &Transform2D, layer: LayerId) -> Result<Affine2D, GraphError> {
        let doc = self.doc;
        resolve_transform(
            xform,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
            &|id| crate::command::find_envelope(doc, id).map(|e| &e.transform),
        )
        .map_err(|e| GraphError::ParamEval {
            layer: layer.get(),
            source: e,
        })
    }

    fn record_layer_position(
        &mut self,
        layer: LayerId,
        xform: &Transform2D,
    ) -> Result<(), GraphError> {
        // visible=false でも LookAt/Follow 依存先として位置を記録する。
        if let Ok(pos) = eval_vec2(
            &xform.position,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        ) {
            self.resolved_layers.insert_position(layer, pos);
        }
        Ok(())
    }

    fn build_source(
        &mut self,
        clip: &Clip,
        source_time: RationalTime,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        match &clip.source {
            ClipSource::Asset { asset } => self.ensure_video_slot(*asset, source_time),
            ClipSource::Vector { .. } => Err(GraphError::UnsupportedVectorSource(layer.get())),
            ClipSource::Plugin {
                plugin_id, params, ..
            } if plugin_id == RECT_LAYER_SOURCE => self.build_rect_overlay(params, layer),
            ClipSource::Plugin {
                plugin_id, params, ..
            } if plugin_id == CLEAR_LAYER_SOURCE => {
                let resolved = self.resolve_plugin_params(plugin_id, params, layer)?;
                let out = self.alloc_id();
                self.steps.push(RenderStep::Plugin {
                    id: self.resolve_plugin_id(CLEAR_LAYER_SOURCE)?,
                    params: resolved,
                    inputs: vec![],
                    output: out,
                });
                Ok(out)
            }
            ClipSource::Plugin { plugin_id, .. } => {
                Err(GraphError::UnsupportedSourcePlugin(plugin_id.clone()))
            }
        }
    }

    fn build_rect_overlay(
        &mut self,
        params: &BTreeMap<String, crate::DocParam>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        let pe = |e| GraphError::ParamEval {
            layer: layer.get(),
            source: e,
        };
        let center_p = params.get("center").ok_or(GraphError::MissingRectParam {
            layer: layer.get(),
            param: "center",
        })?;
        let size_p = params.get("size").ok_or(GraphError::MissingRectParam {
            layer: layer.get(),
            param: "size",
        })?;
        let color_p = params.get("color").ok_or(GraphError::MissingRectParam {
            layer: layer.get(),
            param: "color",
        })?;
        // F-3: ソースはローカル空間。変形は effect 後の AffinePlace で適用する。
        let center = eval_vec2(
            center_p,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        let size = eval_vec2(
            size_p,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        let color = eval_color(
            color_p,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        let pre = self.transparent();
        let out = self.alloc_id();
        self.steps.push(RenderStep::OverlayRect {
            input: pre,
            output: out,
            overlay: RectOverlay {
                center: CanonicalPoint {
                    x: center[0],
                    y: center[1],
                },
                size: CanonicalSize {
                    width: size[0],
                    height: size[1],
                },
                color: [
                    color[0] as f32,
                    color[1] as f32,
                    color[2] as f32,
                    color[3] as f32,
                ],
            },
        });
        Ok(out)
    }

    /// F-3 変形段。恒等ならスキップ。
    fn apply_world_transform(
        &mut self,
        input: TextureId,
        world: Affine2D,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        if world.is_approx_identity() {
            return Ok(input);
        }
        let aspect = self.frame_desc.width as f64 / self.frame_desc.height as f64;
        let inverse_uv = world
            .to_inverse_uv_matrix(aspect)
            .ok_or(GraphError::SingularTransform(layer.get()))?;
        let out = self.alloc_id();
        self.steps.push(RenderStep::AffinePlace {
            input,
            output: out,
            inverse_uv,
        });
        Ok(out)
    }

    fn apply_effect(
        &mut self,
        input: TextureId,
        effect: &crate::schema::EffectInstance,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        let resolved = self.resolve_plugin_params(&effect.plugin_id, &effect.params, layer)?;
        let out = self.alloc_id();
        self.steps.push(RenderStep::Plugin {
            id: self.resolve_plugin_id(&effect.plugin_id)?,
            params: resolved,
            inputs: vec![input],
            output: out,
        });
        Ok(out)
    }

    fn apply_envelope_opacity(
        &mut self,
        input: TextureId,
        env: &ItemEnvelope,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        let opacity = self.eval_opacity(env, layer)?;
        if opacity < 1.0 - f64::EPSILON {
            self.apply_opacity(input, opacity)
        } else {
            Ok(input)
        }
    }

    fn apply_opacity(&mut self, input: TextureId, amount: f64) -> Result<TextureId, GraphError> {
        let mut params = ResolvedParams::new();
        params.insert("amount", Value::F64(amount));
        let out = self.alloc_id();
        self.steps.push(RenderStep::Plugin {
            id: self.resolve_plugin_id("core.filter.opacity")?,
            params,
            inputs: vec![input],
            output: out,
        });
        Ok(out)
    }

    fn apply_mask(&mut self, content: TextureId, mask: TextureId, mode: MaskMode) -> TextureId {
        let out = self.alloc_id();
        self.steps.push(RenderStep::ApplyMask {
            content,
            mask,
            output: out,
            mode: mask_to_clipping(mode),
        });
        out
    }

    fn composite(&mut self, bg: TextureId, fg: TextureId, blend: BlendMode) -> TextureId {
        let out = self.alloc_id();
        self.steps.push(RenderStep::Composite {
            background: bg,
            foreground: fg,
            output: out,
            mode: blend_to_composite(blend),
        });
        out
    }

    fn resolve_plugin_params(
        &self,
        plugin_id: &str,
        params: &BTreeMap<String, crate::DocParam>,
        layer: LayerId,
    ) -> Result<ResolvedParams, GraphError> {
        let desc = self
            .lookup_desc(plugin_id)
            .ok_or_else(|| GraphError::UnsupportedSourcePlugin(plugin_id.to_string()))?;
        let mut raw = HashMap::new();
        for (k, p) in params {
            let v = eval_doc_param(p, self.timeline_time, self.tracks, &self.resolved_layers)
                .map_err(|e| GraphError::ParamEval {
                    layer: layer.get(),
                    source: e,
                })?;
            raw.insert(k.clone(), v);
        }
        desc.resolve_params(&raw).map_err(|e| GraphError::Plugin {
            plugin_id: plugin_id.to_string(),
            source: e,
        })
    }

    fn lookup_desc(&self, plugin_id: &str) -> Option<&'static NodeDesc> {
        self.registry
            .filter_by_name(plugin_id)
            .map(|p| p.desc())
            .or_else(|| {
                self.registry
                    .layer_source_by_name(plugin_id)
                    .map(|p| p.desc())
            })
            .or_else(|| self.registry.composite_by_name(plugin_id).map(|p| p.desc()))
    }

    fn resolve_plugin_id(&self, plugin_id: &str) -> Result<PluginId, GraphError> {
        self.lookup_desc(plugin_id)
            .map(|d| d.id.clone())
            .ok_or_else(|| GraphError::UnsupportedSourcePlugin(plugin_id.to_string()))
    }

    fn eval_opacity(&self, env: &ItemEnvelope, layer: LayerId) -> Result<f64, GraphError> {
        eval_f64(
            &env.opacity,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(|e| GraphError::ParamEval {
            layer: layer.get(),
            source: e,
        })
    }
}

fn document_has_solo(doc: &Document) -> bool {
    doc.tracks
        .iter()
        .flat_map(|t| t.items.iter())
        .any(item_tree_has_solo)
}

fn item_tree_has_solo(item: &TrackItem) -> bool {
    let env = item_envelope(item);
    if env.solo {
        return true;
    }
    match item {
        TrackItem::Clip(_) => false,
        TrackItem::Group(g) => g.children.iter().any(item_tree_has_solo),
    }
}

fn clip_active(clip: &Clip, t: RationalTime) -> bool {
    clip.start <= t
        && clip
            .start
            .try_add(clip.duration)
            .map(|e| t < e)
            .unwrap_or(false)
}

fn item_layer_id(item: &TrackItem) -> LayerId {
    match item {
        TrackItem::Clip(c) => c.envelope.layer_id,
        TrackItem::Group(g) => g.envelope.layer_id,
    }
}

fn item_envelope(item: &TrackItem) -> &ItemEnvelope {
    match item {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    }
}

fn blend_to_composite(mode: BlendMode) -> CompositeMode {
    match mode {
        BlendMode::Normal => CompositeMode::Normal,
        BlendMode::Add => CompositeMode::Add,
        BlendMode::Multiply => CompositeMode::Multiply,
    }
}

fn mask_to_clipping(mode: MaskMode) -> ClippingMaskMode {
    match mode {
        MaskMode::Alpha => ClippingMaskMode::Alpha,
        MaskMode::Luminance => ClippingMaskMode::Luminance,
        MaskMode::InvertAlpha => ClippingMaskMode::InvertAlpha,
        MaskMode::InvertLuminance => ClippingMaskMode::InvertLuminance,
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use crate::Asset;
    use std::fs;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, b"x").unwrap();
    }

    fn base_asset() -> Asset {
        Asset {
            id: AssetId::from_raw(0),
            name: "a".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:abc".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        }
    }

    #[test]
    fn resolve_prefers_relative_over_absolute() {
        let root = std::env::temp_dir().join(format!("motolii-resolve-rel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("media")).unwrap();
        let rel = root.join("media/rel.mp4");
        let abs = root.join("abs.mp4");
        touch(&rel);
        touch(&abs);
        let mut asset = base_asset();
        asset.path_project_relative = Some("media/rel.mp4".into());
        asset.path_absolute = Some(abs.to_string_lossy().into());
        let got = resolve_asset_path(&asset, Some(&root)).unwrap();
        assert_eq!(got, rel);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_does_not_return_bare_missing_name() {
        let mut asset = base_asset();
        asset.file_name = Some("missing-nowhere.mp4".into());
        assert!(resolve_asset_path(&asset, None).is_none());
        assert!(resolve_asset_path(&asset, Some(Path::new("/tmp"))).is_none());
    }

    #[test]
    fn resolve_falls_back_to_hash_cache_path() {
        let root =
            std::env::temp_dir().join(format!("motolii-resolve-hash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let hash_path = root.join(".motolii/media/sha256:abc");
        touch(&hash_path);
        let asset = base_asset();
        let got = resolve_asset_path(&asset, Some(&root)).unwrap();
        assert_eq!(got, hash_path);
        let _ = fs::remove_dir_all(&root);
    }
}
