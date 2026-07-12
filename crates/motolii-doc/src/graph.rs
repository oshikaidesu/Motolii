//! Document→レンダグラフ(D3 / F-3凍結順序)。

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use motolii_core::{FrameDesc, RationalTime, TimeMapError};
use motolii_eval::{DataTracks, Value};
use motolii_nodes::{CanonicalPoint, CanonicalSize, ClippingMaskMode, CompositeMode, RectOverlay};
use motolii_plugin::{NodeDesc, PluginId, PluginRegistry, ResolvedParams};
use motolii_render::{LinearRenderGraph, RenderStep, SolidSource, TextureId};

use crate::eval_time::EvaluationTime;
use crate::param_eval::{
    eval_color, eval_doc_param, eval_f64, eval_vec2, ParamEvalError, ResolvedLayerParams,
};
use crate::schema::{
    BlendMode, Clip, ClipSource, Group, ItemEnvelope, MaskMode, TrackItem, Transform2D,
};
use crate::{AssetId, Document, LayerId};

pub const RECT_LAYER_SOURCE: &str = "doc.layer_source.rect";
pub const CLEAR_LAYER_SOURCE: &str = "core.layer_source.clear";

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentFrameGraph {
    pub graph: LinearRenderGraph,
    pub video_slots: Vec<(TextureId, AssetId)>,
    pub source_time: RationalTime,
}

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("no video source clip in document")]
    NoVideoSource,
    #[error("multiple video asset clips in one frame")]
    MultipleVideoSources,
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
    #[error("plugin {plugin_id}: {source}")]
    Plugin {
        plugin_id: String,
        #[source]
        source: motolii_plugin::PluginError,
    },
    #[error("unsupported clip source plugin: {0}")]
    UnsupportedSourcePlugin(String),
    #[error("rect layer source missing param `{param}` (layer {layer})")]
    MissingRectParam { layer: u64, param: &'static str },
}

pub fn resolve_asset_path(asset: &crate::Asset, project_root: Option<&Path>) -> Option<PathBuf> {
    // ガード10: relative → absolute → same-name → hash。実在ファイルのみ返す。
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
        // content_hash キーのキャッシュ席(未配置ならスキップして Unresolved)。
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
    let mut b = GraphBuilder::new(doc, desc, eval.timeline_time, data_tracks, registry);
    let output = b.build_document(project_root)?;
    Ok(DocumentFrameGraph {
        graph: LinearRenderGraph {
            desc,
            steps: b.steps,
            output,
        },
        video_slots: b.video_slots,
        // 動画クリップが無い文書(矩形のみ等)はtimeline縮退を返す。
        source_time: b.video_source_time.unwrap_or(eval.timeline_time),
    })
}

struct GraphBuilder<'a> {
    doc: &'a Document,
    _desc: FrameDesc,
    timeline_time: RationalTime,
    tracks: &'a DataTracks,
    registry: &'a PluginRegistry,
    steps: Vec<RenderStep>,
    next_id: usize,
    transparent_id: Option<TextureId>,
    video_slots: Vec<(TextureId, AssetId)>,
    /// 動画ソースクリップの TimeMap 結果のみ。非動画クリップで上書きしない。
    video_source_time: Option<RationalTime>,
    resolved_layers: ResolvedLayerParams,
}

impl<'a> GraphBuilder<'a> {
    fn new(
        doc: &'a Document,
        desc: FrameDesc,
        timeline_time: RationalTime,
        tracks: &'a DataTracks,
        registry: &'a PluginRegistry,
    ) -> Self {
        Self {
            doc,
            _desc: desc,
            timeline_time,
            tracks,
            registry,
            steps: Vec::new(),
            next_id: 0,
            transparent_id: None,
            video_slots: Vec::new(),
            video_source_time: None,
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
    fn build_document(&mut self, _: Option<&Path>) -> Result<TextureId, GraphError> {
        let mut acc = self.transparent();
        let mut prev_mask = None;
        for track in &self.doc.tracks {
            for item in &track.items {
                let layer = item_layer_id(item);
                let tex = self.build_item(item, &Transform2D::identity(), prev_mask, layer)?;
                let env = item_envelope(item);
                // 最下層でも envelope opacity を適用する(prefill 直結で飛ばさない)。
                let fg = self.apply_envelope_opacity(tex, env, layer)?;
                if acc == self.transparent_id.unwrap() {
                    acc = fg;
                } else {
                    acc = self.composite(acc, fg, env.blend);
                }
                prev_mask = Some(fg);
            }
        }
        Ok(acc)
    }
    fn ensure_video_slot(&mut self, asset: AssetId) -> Result<TextureId, GraphError> {
        if let Some((id, existing)) = self.video_slots.first() {
            return if *existing == asset {
                Ok(*id)
            } else {
                Err(GraphError::MultipleVideoSources)
            };
        }
        let id = self.alloc_id();
        self.steps.push(RenderStep::VideoSource { output: id });
        self.video_slots.push((id, asset));
        Ok(id)
    }
    fn build_item(
        &mut self,
        item: &TrackItem,
        inherited: &Transform2D,
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
        inherited: &Transform2D,
        mask_below: Option<TextureId>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        let child_xform = compose_transform(inherited, &group.envelope.transform);
        let mut acc = self.transparent();
        let mut prev_child = None;
        for child in &group.children {
            let child_layer = item_layer_id(child);
            let tex = self.build_item(child, &child_xform, prev_child, child_layer)?;
            let env = item_envelope(child);
            let fg = self.apply_envelope_opacity(tex, env, child_layer)?;
            if acc == self.transparent_id.unwrap() {
                acc = fg;
            } else {
                acc = self.composite(acc, fg, env.blend);
            }
            prev_child = Some(fg);
        }
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
        if let Ok(pos) = eval_vec2(
            &group.envelope.transform.position,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        ) {
            self.resolved_layers.insert_position(layer, pos);
        }
        Ok(tex)
    }
    fn build_clip(
        &mut self,
        clip: &Clip,
        inherited: &Transform2D,
        mask_below: Option<TextureId>,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        if !clip_active(clip, self.timeline_time) {
            return Ok(self.transparent());
        }
        let local =
            self.timeline_time
                .try_sub(clip.start)
                .map_err(|e| GraphError::InvalidClip {
                    layer: layer.get(),
                    source: e.into(),
                })?;
        let st = clip
            .time_map
            .try_map(local)
            .map_err(|e| GraphError::InvalidClip {
                layer: layer.get(),
                source: e,
            })?;
        // DocumentFrameGraph.source_time は動画ソースのみが正本(最終クリップ上書き禁止)。
        if matches!(&clip.source, ClipSource::Asset { .. }) {
            self.video_source_time = Some(st);
        }
        let xform = compose_transform(inherited, &clip.envelope.transform);
        let mut tex = self.build_source(clip, &xform, layer)?;
        for effect in &clip.envelope.effects {
            if effect.enabled {
                tex = self.apply_effect(tex, effect, layer)?;
            }
        }
        if clip.envelope.clipping_mask.enabled {
            if let Some(mask) = mask_below {
                tex = self.apply_mask(tex, mask, clip.envelope.clipping_mask.mode);
            }
        }
        if let Ok(pos) = eval_vec2(
            &clip.envelope.transform.position,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        ) {
            self.resolved_layers.insert_position(layer, pos);
        }
        Ok(tex)
    }
    fn build_source(
        &mut self,
        clip: &Clip,
        xform: &Transform2D,
        layer: LayerId,
    ) -> Result<TextureId, GraphError> {
        match &clip.source {
            ClipSource::Asset { asset } => self.ensure_video_slot(*asset),
            ClipSource::Plugin {
                plugin_id, params, ..
            } if plugin_id == RECT_LAYER_SOURCE => {
                // envelope opacity は合成段で一度だけ適用(色αへ焼き込まない)。
                self.build_rect_overlay(params, xform, layer)
            }
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
        xform: &Transform2D,
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
        let mut center = eval_vec2(
            center_p,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        let mut size = eval_vec2(
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
        let pos = eval_vec2(
            &xform.position,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        let scale = eval_vec2(
            &xform.scale,
            self.timeline_time,
            self.tracks,
            &self.resolved_layers,
        )
        .map_err(pe)?;
        center[0] += pos[0];
        center[1] += pos[1];
        size[0] *= scale[0];
        size[1] *= scale[1];
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
    /// レジストリ内の静的 PluginId を返す。毎フレーム Box::leak しない。
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
fn compose_transform(parent: &Transform2D, child: &Transform2D) -> Transform2D {
    let mut out = child.clone();
    if let (crate::DocParam::Const(Value::Vec2(pp)), crate::DocParam::Const(Value::Vec2(cp))) =
        (&parent.position, &child.position)
    {
        out.position = crate::DocParam::const_vec2([pp[0] + cp[0], pp[1] + cp[1]]);
    }
    if let (crate::DocParam::Const(Value::Vec2(ps)), crate::DocParam::Const(Value::Vec2(cs))) =
        (&parent.scale, &child.scale)
    {
        out.scale = crate::DocParam::const_vec2([ps[0] * cs[0], ps[1] * cs[1]]);
    }
    out
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
