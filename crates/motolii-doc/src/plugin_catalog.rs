//! D1f: ロード時の未知`plugin_id`検出(ガード9の「開く」側)。
//!
//! 書き出し厳格化はD6。ここでは拒否せず警告だけ返し、Effect/Pluginの`extra`と
//! 未知ID自体はそのままDocumentに載せる(F-9パススルー)。
//!
//! スロット不一致(例: filter席にlayer_source ID)は`Unknown*`ではなく`PluginIdWrongKind` —
//! 「未登録」と「席違い」を混同すると、呼び手が欠落インストールと誤診断するため。

use std::cmp::Ordering;
use std::collections::HashSet;

use thiserror::Error;

use crate::schema::{ClipSource, ItemEnvelope, TrackItem};
use crate::Document;

/// ロード時に「既知」とみなすプラグインID集合。呼び出し側(レジストリ等)が供給する。
///
/// motolii-docはGPUレジストリに依存しない — ID集合だけを受け取る。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginCatalog {
    filters: HashSet<String>,
    layer_sources: HashSet<String>,
}

impl PluginCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_filter(&mut self, id: impl Into<String>) {
        self.filters.insert(id.into());
    }

    pub fn insert_layer_source(&mut self, id: impl Into<String>) {
        self.layer_sources.insert(id.into());
    }

    /// 参照実装(motolii-plugin `register_reference_plugins`)のv1 ID集合。
    pub fn reference_v1() -> Self {
        let mut catalog = Self::new();
        for id in [
            "core.filter.clear",
            "core.filter.tint",
            "core.filter.opacity",
        ] {
            catalog.insert_filter(id);
        }
        catalog.insert_layer_source("core.layer_source.clear");
        catalog
    }

    pub fn knows_filter(&self, plugin_id: &str) -> bool {
        self.filters.contains(plugin_id)
    }

    pub fn knows_layer_source(&self, plugin_id: &str) -> bool {
        self.layer_sources.contains(plugin_id)
    }

    /// カタログ内のどれかの席で既知か(WrongKind判定用)。
    fn known_kind(&self, plugin_id: &str) -> Option<PluginSlot> {
        if self.filters.contains(plugin_id) {
            Some(PluginSlot::Filter)
        } else if self.layer_sources.contains(plugin_id) {
            Some(PluginSlot::LayerSource)
        } else {
            None
        }
    }
}

/// Document上のプラグイン席(Effectスタック vs ClipSource::Plugin)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginSlot {
    Filter,
    LayerSource,
}

impl std::fmt::Display for PluginSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Filter => write!(f, "filter"),
            Self::LayerSource => write!(f, "layer_source"),
        }
    }
}

/// オープン経路の警告(エラーではない — ロードは成功する)。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoadWarning {
    #[error("unknown effect plugin `{plugin_id}` on layer {layer_id}")]
    UnknownEffectPlugin { plugin_id: String, layer_id: u64 },
    #[error("unknown layer source plugin `{plugin_id}` on layer {layer_id}")]
    UnknownLayerSourcePlugin { plugin_id: String, layer_id: u64 },
    /// 既知IDだが席違い。未登録と区別する(誤って「欠落プラグイン」扱いしないため)。
    #[error(
        "plugin `{plugin_id}` on layer {layer_id} is known as {actual}, but used as {expected}"
    )]
    PluginIdWrongKind {
        plugin_id: String,
        layer_id: u64,
        expected: PluginSlot,
        actual: PluginSlot,
    },
}

/// `load_document*` の成功ペイロード。警告はサイドチャネルとして返す(黙殺しない)。
///
/// `document`だけ取り出すヘルパーは意図的に置かない — ガード9違反の黙殺経路を作らないため。
#[derive(Debug, Clone, PartialEq)]
pub struct LoadResult {
    pub document: Document,
    pub warnings: Vec<LoadWarning>,
}

/// Document内の未知/席違い`plugin_id`を走査し、決定的順序の警告リストを返す。
pub fn collect_plugin_warnings(doc: &Document, catalog: &PluginCatalog) -> Vec<LoadWarning> {
    let mut warnings = Vec::new();
    for track in &doc.tracks {
        for item in &track.items {
            collect_item_warnings(item, catalog, &mut warnings);
        }
    }
    warnings.sort_by(compare_warnings);
    warnings
}

fn compare_warnings(a: &LoadWarning, b: &LoadWarning) -> Ordering {
    warning_sort_key(a).cmp(&warning_sort_key(b))
}

fn warning_sort_key(w: &LoadWarning) -> (u8, u64, &str) {
    match w {
        LoadWarning::UnknownEffectPlugin {
            plugin_id,
            layer_id,
        } => (0, *layer_id, plugin_id.as_str()),
        LoadWarning::UnknownLayerSourcePlugin {
            plugin_id,
            layer_id,
        } => (1, *layer_id, plugin_id.as_str()),
        LoadWarning::PluginIdWrongKind {
            plugin_id,
            layer_id,
            ..
        } => (2, *layer_id, plugin_id.as_str()),
    }
}

fn collect_item_warnings(
    item: &TrackItem,
    catalog: &PluginCatalog,
    warnings: &mut Vec<LoadWarning>,
) {
    match item {
        TrackItem::Clip(clip) => {
            collect_envelope_warnings(&clip.envelope, catalog, warnings);
            if let ClipSource::Plugin { plugin_id, .. } = &clip.source {
                let layer_id = clip.envelope.layer_id.get();
                if let Some(w) =
                    classify_slot(plugin_id, layer_id, PluginSlot::LayerSource, catalog)
                {
                    warnings.push(w);
                }
            }
        }
        TrackItem::Group(group) => {
            collect_envelope_warnings(&group.envelope, catalog, warnings);
            for child in &group.children {
                collect_item_warnings(child, catalog, warnings);
            }
        }
    }
}

fn collect_envelope_warnings(
    envelope: &ItemEnvelope,
    catalog: &PluginCatalog,
    warnings: &mut Vec<LoadWarning>,
) {
    let layer_id = envelope.layer_id.get();
    for effect in &envelope.effects {
        if let Some(w) = classify_slot(&effect.plugin_id, layer_id, PluginSlot::Filter, catalog) {
            warnings.push(w);
        }
    }
}

fn classify_slot(
    plugin_id: &str,
    layer_id: u64,
    expected: PluginSlot,
    catalog: &PluginCatalog,
) -> Option<LoadWarning> {
    if plugin_id.is_empty() {
        return None;
    }
    let ok = match expected {
        PluginSlot::Filter => catalog.knows_filter(plugin_id),
        PluginSlot::LayerSource => catalog.knows_layer_source(plugin_id),
    };
    if ok {
        return None;
    }
    // 席違いと未登録を分ける — Unknownだと「未インストール」と誤認される
    if let Some(actual) = catalog.known_kind(plugin_id) {
        return Some(LoadWarning::PluginIdWrongKind {
            plugin_id: plugin_id.to_string(),
            layer_id,
            expected,
            actual,
        });
    }
    Some(match expected {
        PluginSlot::Filter => LoadWarning::UnknownEffectPlugin {
            plugin_id: plugin_id.to_string(),
            layer_id,
        },
        PluginSlot::LayerSource => LoadWarning::UnknownLayerSourcePlugin {
            plugin_id: plugin_id.to_string(),
            layer_id,
        },
    })
}
