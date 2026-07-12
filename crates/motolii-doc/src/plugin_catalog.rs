//! D1f: ロード時の未知`plugin_id`検出(ガード9の「開く」側)。

use std::collections::HashSet;
use thiserror::Error;
use crate::schema::{ClipSource, ItemEnvelope, TrackItem};
use crate::Document;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginCatalog {
    filters: HashSet<String>,
    layer_sources: HashSet<String>,
}

impl PluginCatalog {
    pub fn new() -> Self { Self::default() }
    pub fn insert_filter(&mut self, id: impl Into<String>) { self.filters.insert(id.into()); }
    pub fn insert_layer_source(&mut self, id: impl Into<String>) { self.layer_sources.insert(id.into()); }
    pub fn reference_v1() -> Self {
        let mut catalog = Self::new();
        for id in ["core.filter.clear", "core.filter.tint", "core.filter.opacity"] {
            catalog.insert_filter(id);
        }
        catalog.insert_layer_source("core.layer_source.clear");
        catalog
    }
    pub fn knows_filter(&self, plugin_id: &str) -> bool { self.filters.contains(plugin_id) }
    pub fn knows_layer_source(&self, plugin_id: &str) -> bool { self.layer_sources.contains(plugin_id) }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoadWarning {
    #[error("unknown effect plugin `{plugin_id}` on layer {layer_id}")]
    UnknownEffectPlugin { plugin_id: String, layer_id: u64 },
    #[error("unknown layer source plugin `{plugin_id}` on layer {layer_id}")]
    UnknownLayerSourcePlugin { plugin_id: String, layer_id: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadResult {
    pub document: Document,
    pub warnings: Vec<LoadWarning>,
}

impl LoadResult {
    pub fn into_document(self) -> Document { self.document }
}

pub fn collect_plugin_warnings(doc: &Document, catalog: &PluginCatalog) -> Vec<LoadWarning> {
    let mut warnings = Vec::new();
    for track in &doc.tracks {
        for item in &track.items {
            collect_item_warnings(item, catalog, &mut warnings);
        }
    }
    warnings.sort_by(|a, b| match (a, b) {
        (LoadWarning::UnknownEffectPlugin { layer_id: la, plugin_id: pa }, LoadWarning::UnknownEffectPlugin { layer_id: lb, plugin_id: pb }) => la.cmp(lb).then_with(|| pa.cmp(pb)),
        (LoadWarning::UnknownLayerSourcePlugin { layer_id: la, plugin_id: pa }, LoadWarning::UnknownLayerSourcePlugin { layer_id: lb, plugin_id: pb }) => la.cmp(lb).then_with(|| pa.cmp(pb)),
        (LoadWarning::UnknownEffectPlugin { .. }, LoadWarning::UnknownLayerSourcePlugin { .. }) => std::cmp::Ordering::Less,
        (LoadWarning::UnknownLayerSourcePlugin { .. }, LoadWarning::UnknownEffectPlugin { .. }) => std::cmp::Ordering::Greater,
    });
    warnings
}

fn collect_item_warnings(item: &TrackItem, catalog: &PluginCatalog, warnings: &mut Vec<LoadWarning>) {
    match item {
        TrackItem::Clip(clip) => {
            collect_envelope_warnings(&clip.envelope, catalog, warnings);
            if let ClipSource::Plugin { plugin_id, .. } = &clip.source {
                let layer_id = clip.envelope.layer_id.get();
                if !plugin_id.is_empty() && !catalog.knows_layer_source(plugin_id) {
                    warnings.push(LoadWarning::UnknownLayerSourcePlugin { plugin_id: plugin_id.clone(), layer_id });
                }
            }
        }
        TrackItem::Group(group) => {
            collect_envelope_warnings(&group.envelope, catalog, warnings);
            for child in &group.children { collect_item_warnings(child, catalog, warnings); }
        }
    }
}

fn collect_envelope_warnings(envelope: &ItemEnvelope, catalog: &PluginCatalog, warnings: &mut Vec<LoadWarning>) {
    let layer_id = envelope.layer_id.get();
    for effect in &envelope.effects {
        if !effect.plugin_id.is_empty() && !catalog.knows_filter(&effect.plugin_id) {
            warnings.push(LoadWarning::UnknownEffectPlugin { plugin_id: effect.plugin_id.clone(), layer_id });
        }
    }
}
