
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::doc::core::RationalTime;
use crate::doc::eval::{KeyframeTrack, Value};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, Timeline};

use crate::doc::store::components::{
    descriptor_assets, descriptor_attrs, descriptor_composition, descriptor_effects,
    descriptor_notebook, descriptor_markers, descriptor_masks, descriptor_meta, descriptor_present, descriptor_shapes,
    descriptor_slots, descriptor_text, descriptor_track, LayerPresent, TrackJson,
};
use super::read::{composition_path, ReadOverlay, RecordCache, TrackCache, TransientKey};
use crate::doc::store::slot::{PropertyBase, PropertySource};
use crate::doc::store::{
    Asset, AssetId, AssetTable, Composition, EffectInstance, LayerAttrs, LayerId,
    LayerMeta, LayerSource, Marker, Mask, PropertyId, Revision, ShapeNode, Slot, SlotId,
    StoreError, TextDocument, EDIT_TIMELINE,
};

/// The renderer-owned solver for one view's flow layout.
///
/// A document owns the values and the resulting [`Frame`] cache, but it must
/// not own a renderer implementation (or its `TaffyTree`).  The renderer
/// supplies this narrow read-only hook for a frame; ordinary document reads
/// deliberately leave it empty.
pub trait LayoutSolver {
    fn compute(&self, view: &StoreView<'_>, time: RationalTime) -> Result<super::scratch::Frame, StoreError>;
}

#[derive(Clone)]
pub struct StoreView<'a> {
    db: &'a EntityDb,
    at: i64,
    transient: &'a HashMap<TransientKey, Value>,
    preview_edits: &'a ReadOverlay,
    ignore_transients: bool,
    revision: Revision,
    track_cache: &'a RefCell<TrackCache>,
    record_cache: &'a RefCell<RecordCache>,
    /// host が描いた絵から解いた値(Blob の塊など)。無ければ解析を読む配置は空。
    analysis: Option<&'a super::analysis::AnalysisInputs>,
    layout_memo: super::scratch::Memo,
    layout_cache: &'a RefCell<super::scratch::LayoutCache>,
    layout_solver: Option<Rc<dyn LayoutSolver>>,
    programs: super::kind::Programs,
    placement_programs: Option<&'a [super::kind::PlacementProgram]>,
}

const MAX_LINK_DEPTH: u32 = 64;

/// 書類ぜんたいを訊く口(居る層・書類の寸法)と、その 1 コマぶんの手控え。
/// 親の private も見えるので、`db` / `at` / `query` はそのまま使える。
#[path = "view_records.rs"]
mod records;
use records::layer_id_of;

impl<'a> StoreView<'a> {
    pub fn new(
        db: &'a EntityDb,
        at: i64,
        transient: &'a HashMap<TransientKey, Value>,
        preview_edits: &'a ReadOverlay,
        revision: Revision,
        track_cache: &'a RefCell<TrackCache>,
        record_cache: &'a RefCell<RecordCache>,
        layout_cache: &'a RefCell<super::scratch::LayoutCache>,
        programs: super::kind::Programs,
    ) -> Self {
        Self {
            db,
            at,
            transient,
            preview_edits,
            ignore_transients: false,
            revision,
            track_cache,
            record_cache,
            analysis: None,
            layout_memo: Default::default(),
            layout_cache,
            layout_solver: None,
            programs,
            placement_programs: None,
        }
    }

    pub fn layout_memo(&self) -> &super::scratch::Memo {
        &self.layout_memo
    }

    /// Give this ephemeral read a renderer cache.  This is deliberately not
    /// part of [`Document`]: the cache is a rendering resource, not authored
    /// state or undoable layout meaning.
    pub fn with_layout_solver(mut self, solver: Rc<dyn LayoutSolver>) -> Self {
        self.layout_solver = Some(solver);
        self
    }

    pub fn layout_solver(&self) -> Option<&dyn LayoutSolver> {
        self.layout_solver.as_deref()
    }

    /// コマをまたぐ配置の覚えを使ってよい view か(解析・仮の編集・一時の値のどれも読まない)。
    pub fn shared_layout_cache(&self) -> Option<(&RefCell<super::scratch::LayoutCache>, &Revision)> {
        (self.placement_programs.is_none() && self.analysis.is_none() && (self.ignore_transients || (self.preview_edits.is_empty() && self.transient.is_empty()))).then_some((self.layout_cache, &self.revision))
    }

    /// Supply the complete set of stateless placement programs for this read view.
    pub fn with_placement_programs(mut self, programs: &'a [super::kind::PlacementProgram]) -> Self {
        self.placement_programs = Some(programs);
        self.layout_memo = Default::default();
        self
    }

    /// 1 コマの中で層を取り直す効果の取っ手。無ければその効果はぼかさない。
    pub fn shutter_of(&self, plugin_id: &str, params: &[(String, crate::doc::eval::Value)]) -> Option<super::kind::Shutter> {
        ((self.programs.sampling)(plugin_id)?.shutter)(params)
    }

    /// 見つけた格子へ寄せる効果の取っ手。寄せない効果なら None。
    pub fn snapping_of(&self, plugin_id: &str, params: &[(String, crate::doc::eval::Value)]) -> Option<super::kind::Snapping> {
        let program = (self.programs.snap)(plugin_id)?;
        (program.snapping)(program.plugin_id, params)
    }

    /// 見つけた格子へ寄せる気のある効果か(寄せるかどうかは取っ手次第)。
    pub fn is_snap_effect(&self, plugin_id: &str) -> bool {
        (self.programs.snap)(plugin_id).is_some()
    }

    pub fn is_sampling_effect(&self, plugin_id: &str) -> bool {
        (self.programs.sampling)(plugin_id).is_some()
    }

    pub fn sampling_program(&self, plugin_id: &str) -> Option<super::kind::SamplingProgram> {
        (self.programs.sampling)(plugin_id)
    }

    pub fn snap_program(&self, plugin_id: &str) -> Option<super::kind::SnapProgram> {
        (self.programs.snap)(plugin_id)
    }

    pub fn placement_program(&self, plugin_id: &str) -> Option<super::kind::PlacementProgram> {
        match self.placement_programs {
            Some(programs) => programs.iter().find(|program| program.plugin_id == plugin_id).copied(),
            None => (self.programs.placement)(plugin_id),
        }
    }

    /// 解析の入力を読む view(resolve が Blob Track の塊を配置にする)。
    pub fn with_analysis(mut self, inputs: &'a super::analysis::AnalysisInputs) -> Self {
        self.analysis = Some(inputs);
        // 並べた結果は解析(塊・素材の寸法)を読むので、読まない view と覚えを分ける。
        self.layout_memo = Default::default();
        self
    }

    pub fn analysis(&self) -> Option<&'a super::analysis::AnalysisInputs> {
        self.analysis
    }

    pub fn without_transients(mut self) -> Self {
        if !self.ignore_transients {
            self.layout_memo = Default::default();
        }
        self.ignore_transients = true;
        self
    }

    fn cache_key(path: &EntityPath, property: &PropertyId) -> Option<TransientKey> {
        if *path == composition_path() {
            Some(TransientKey::Camera(property.clone()))
        } else {
            Some(TransientKey::Layer(layer_id_of(path)?, property.clone()))
        }
    }

    fn query(&self) -> LatestAtQuery {
        LatestAtQuery::new(*Timeline::new_sequence(EDIT_TIMELINE).name(), self.at)
    }

    /// 書類のこの姿の指紋(保存された版 + 編集の頭 + 仮の値)。同じ指紋なら同じ時刻は同じ絵。
    /// 効果の feedback の状態(compositor)は、指紋が変われば入点からやり直す。
    pub fn revision_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        format!("{:?}", self.db.generation()).hash(&mut hasher);
        self.at.hash(&mut hasher);
        self.ignore_transients.hash(&mut hasher);
        if !self.ignore_transients {
            format!("{:?}", self.transient).hash(&mut hasher);
            self.preview_edits.generation.hash(&mut hasher);
        }
        if let Some(programs) = self.placement_programs {
            programs.len().hash(&mut hasher);
            for program in programs {
                program.plugin_id.hash(&mut hasher);
                program.needs_position.hash(&mut hasher);
                (program.evaluate as usize).hash(&mut hasher);
            }
        }
        hasher.finish()
    }


    pub fn has_layer(&self, layer: LayerId) -> bool {
        self.layers().contains(&layer)
    }

    pub fn next_layer_id(&self) -> u64 {
        self.db
            .sorted_entity_paths()
            .filter_map(layer_id_of)
            .map(|id| id.0)
            .max()
            .map(|max| max + 1)
            .unwrap_or(1)
    }

    pub fn properties(&self, layer: LayerId) -> Vec<PropertyId> {
        let path = layer.entity_path();
        let engine = self.db.storage_engine();
        let Some(components) = engine.store().schema().all_components_for_entity(&path) else {
            return Vec::new();
        };
        let mut out: Vec<PropertyId> = components
            .iter()
            .filter_map(|component| {
                let name = component.as_str().strip_prefix("Layer:")?;
                if crate::doc::store::property::RESERVED.contains(&name) {
                    return None;
                }
                PropertyId::new(name).ok()
            })
            .collect();
        out.sort();
        out
    }

    pub fn track_json_components(
        &self,
        path: &EntityPath,
    ) -> Result<Vec<(re_types_core::ComponentIdentifier, String)>, StoreError> {
        let engine = self.db.storage_engine();
        let Some(components) = engine.store().schema().all_components_for_entity(path) else {
            return Ok(Vec::new());
        };
        let query = self.query();
        let present = descriptor_present().component;
        let mut out: Vec<(re_types_core::ComponentIdentifier, String)> = Vec::new();
        for component in components
            .iter()
            .copied()
            .filter(|component| *component != present)
        {
            let results = self.db.latest_at(&query, path, [component]);
            if results.component_batch_raw(component).is_none() {
                continue;
            }
            let Some(json) = results
                .component_batch::<TrackJson>(component)
                .and_then(|batch| batch.into_iter().next())
            else {
                return Err(StoreError::Property(format!(
                    "component `{}` は値を持っているが `TrackJson` として読めない — \
                     `Layer:present` 以外の component は flattened()/save() が\
                     機械的に全部運ぶ前提なので、型が違う component が増えたらここで\
                     気付く必要がある(黙って保存から消してはいけない)",
                    component.as_str()
                )));
            };
            out.push((component, json.0));
        }
        out.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        Ok(out)
    }

    fn source_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        let key = Self::cache_key(path, property);
        if !self.ignore_transients {
            if let Some(source) = key.as_ref().and_then(|key| self.preview_edits.sources.get(key)) {
                return Ok(Some(source.clone()));
            }
        }
        let Some(key) = key else {
            return self.parse_source_at_path(path, property);
        };
        self.track_cache
            .borrow_mut()
            .get_or_try_insert_with(&self.revision, key, || {
                self.parse_source_at_path(path, property)
            })
    }

    fn parse_source_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        let descriptor = descriptor_track(property);
        let results = self
            .db
            .latest_at(&self.query(), path, [descriptor.component]);
        // 型のまま置かれていればそれが答え(新しい書き込み)。無ければ従来の文字列(古い書類)。
        if let Some(value) = super::value_components::constant_from(&results, descriptor.component) {
            return Ok(Some(PropertySource::constant(value)));
        }
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    fn track_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        Ok(match self.source_at_path(path, property)? {
            Some(PropertySource {
                base: Some(PropertyBase::Track(track)),
                ..
            }) => Some(track),
            _ => None,
        })
    }

    pub fn track(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&layer.entity_path(), property)
    }

    pub fn camera_track(&self, property: &PropertyId) -> Result<Option<KeyframeTrack>, StoreError> {
        self.track_at_path(&composition_path(), property)
    }

    pub fn property_source(
        &self,
        layer: LayerId,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&layer.entity_path(), property)
    }

    pub fn camera_property_source(
        &self,
        property: &PropertyId,
    ) -> Result<Option<PropertySource>, StoreError> {
        self.source_at_path(&composition_path(), property)
    }

    pub fn slots(&self) -> Result<Vec<Slot>, StoreError> {
        let descriptor = descriptor_slots();
        let results = self.db.latest_at(
            &self.query(),
            &composition_path(),
            [descriptor.component],
        );
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    fn slot_track(&self, id: &SlotId) -> Result<Option<KeyframeTrack>, StoreError> {
        Ok(self
            .slots()?
            .into_iter()
            .find(|slot| &slot.id == id)
            .map(|slot| slot.track))
    }

    fn value_at_path(
        &self,
        path: &EntityPath,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path_resolving_links(path, property, t, 0)
    }

    fn value_at_path_resolving_links(
        &self,
        path: &EntityPath,
        property: &PropertyId,
        t: RationalTime,
        link_depth: u32,
    ) -> Result<Option<Value>, StoreError> {
        if let Some(value) = (!self.ignore_transients)
            .then(|| self.transient_value_at(path, property))
            .flatten()
        {
            return Ok(Some(value));
        }
        let Some(source) = self.source_at_path(path, property)? else {
            return Ok(None);
        };
        // 順番の札: 層の時刻は親の箱の Stagger でずれる(鍵も効果もこの時刻で読む)。その後、繰り返し(Loop)で畳む。
        let t = match layer_id_of(path) {
            Some(layer) if !super::layout::is_schedule_row(property.name()) => {
                let shifted = self.layer_time(layer, t)?;
                if super::layout::is_loop_row(property.name()) { shifted } else { self.looped_time(layer, shifted)? }
            }
            _ => t,
        };

        let mut acc: Option<Value> = match source.base {
            Some(PropertyBase::Track(track)) => Some(track.eval(t)),
            Some(PropertyBase::Constant(value)) => Some(value.clone()),
            Some(PropertyBase::Slot(slot_id)) => {
                self.slot_track(&slot_id)?.map(|track| track.eval(t))
            }
            None => None,
        };

        for modulator in &source.modulators {
            if link_depth >= MAX_LINK_DEPTH {
                return Err(StoreError::Property(format!(
                    "link/modulator の参照鎖が深すぎる({MAX_LINK_DEPTH}段以上) — \
                     書き込み時の循環拒否をすり抜けた壊れた Document の可能性がある"
                )));
            }
            let source_t = t.try_add(modulator.time_offset).map_err(|e| {
                StoreError::Property(format!("modulator の time_offset を適用できない: {e}"))
            })?;
            let source_value = self.value_at_path_resolving_links(
                &modulator.source_layer.entity_path(),
                &modulator.source_property,
                source_t,
                link_depth + 1,
            )?;
            let Some(source_value) = source_value else {
                continue;
            };
            let Some(contribution) = crate::doc::store::slot::translate_link(
                &modulator.plugin_id,
                &modulator.params,
                source_value,
            ) else {
                continue; // 型不一致・未知の plugin_id は近似せず寄与ゼロ。
            };
            acc = Some(match acc {
                Some(current) => current.add(&contribution).unwrap_or(current),
                None => contribution,
            });
        }

        Ok(acc)
    }

    fn transient_value_at(&self, path: &EntityPath, property: &PropertyId) -> Option<Value> {
        let key = if *path == composition_path() {
            TransientKey::Camera(property.clone())
        } else {
            TransientKey::Layer(layer_id_of(path)?, property.clone())
        };
        self.transient.get(&key).cloned()
    }

    pub fn value_at(
        &self,
        layer: LayerId,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&layer.entity_path(), property, t)
    }

    pub fn camera_value_at(
        &self,
        property: &PropertyId,
        t: RationalTime,
    ) -> Result<Option<Value>, StoreError> {
        self.value_at_path(&composition_path(), property, t)
    }


    pub fn notebook(&self) -> Result<crate::doc::store::Notebook, StoreError> {
        let descriptor = descriptor_notebook();
        let results = self.db.latest_at(&self.query(), &composition_path(), [descriptor.component]);
        let Some(json) = results.component_batch::<TrackJson>(descriptor.component).and_then(|batch|batch.into_iter().next()) else { return Ok(Default::default()); };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn markers(&self) -> Result<Vec<Marker>, StoreError> {
        let descriptor = descriptor_markers();
        let path = composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn assets_table(&self) -> Result<AssetTable, StoreError> {
        let descriptor = descriptor_assets();
        let path = composition_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(AssetTable::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn assets(&self) -> Result<Vec<Asset>, StoreError> {
        Ok(self.assets_table()?.iter().cloned().collect())
    }

    pub fn asset(&self, id: AssetId) -> Result<Option<Asset>, StoreError> {
        Ok(self.assets_table()?.get(id).cloned())
    }

    pub fn meta(&self, layer: LayerId) -> Result<Option<LayerMeta>, StoreError> {
        let mut value = self.persistent_meta(layer)?;
        if !self.ignore_transients {
            if let (Some(meta), Some(timing)) = (value.as_mut(), self.preview_edits.timings.get(&layer)) {
                meta.timing = *timing;
            }
        }
        Ok(value)
    }

    fn persistent_meta(&self, layer: LayerId) -> Result<Option<LayerMeta>, StoreError> {
        {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some(hit) = cache.meta.get(&layer) {
                return Ok(hit.clone());
            }
        }
        let value = self.meta_uncached(layer)?;
        self.record_cache
            .borrow_mut()
            .meta
            .insert(layer, value.clone());
        Ok(value)
    }

    fn meta_uncached(&self, layer: LayerId) -> Result<Option<LayerMeta>, StoreError> {
        let descriptor = descriptor_meta();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    pub fn masks(&self, layer: LayerId) -> Result<Vec<Mask>, StoreError> {
        let descriptor = descriptor_masks();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn attrs(&self, layer: LayerId) -> Result<Option<LayerAttrs>, StoreError> {
        if !self.ignore_transients {
            if let Some(attrs) = self.preview_edits.attrs.get(&layer) {
                return Ok(Some(attrs.clone()));
            }
        }
        self.persistent_attrs(layer)
    }

    fn persistent_attrs(&self, layer: LayerId) -> Result<Option<LayerAttrs>, StoreError> {
        {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some(hit) = cache.attrs.get(&layer) {
                return Ok(hit.clone());
            }
        }
        let value = self.attrs_uncached(layer)?;
        self.record_cache
            .borrow_mut()
            .attrs
            .insert(layer, value.clone());
        Ok(value)
    }

    /// 全層の「切り抜きの基」を 1 回で解く。層ごとに問うと層²回になる。
    pub fn clipping_bases(&self) -> Result<HashMap<LayerId, Option<LayerId>>, StoreError> {
        let cacheable = self.ignore_transients || self.preview_edits.is_empty();
        if cacheable {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some(bases) = &cache.clipping { return Ok(bases.clone()); }
        }
        let mut ordered = Vec::new();
        for id in self.layers() {
            if let Some(meta) = self.meta(id)? {
                let attrs = self.attrs(id)?.unwrap_or_default();
                ordered.push((meta.order, id, attrs.parent, attrs.clip_to_below));
            }
        }
        ordered.sort_by_key(|(order, id, _, _)| (*order, *id));
        let mut previous = HashMap::new();
        let mut bases = HashMap::new();
        for group in ordered.chunk_by(|a, b| a.0 == b.0) {
            for (_, id, parent, _) in group { bases.insert(*id, previous.get(parent).copied()); }
            for (_, id, parent, clipped) in group { if !clipped { previous.insert(*parent, *id); } }
        }
        if cacheable { self.record_cache.borrow_mut().clipping = Some(bases.clone()); }
        Ok(bases)
    }

    pub fn clipping_base(&self, layer: LayerId) -> Result<Option<LayerId>, StoreError> {
        if self.ignore_transients || self.preview_edits.is_empty() {
            let mut cache = self.record_cache.borrow_mut();
            cache.sync(&self.revision);
            if let Some(bases) = &cache.clipping { return Ok(bases.get(&layer).copied().flatten()); }
        }
        Ok(self.clipping_bases()?.get(&layer).copied().flatten())
    }

    fn attrs_uncached(&self, layer: LayerId) -> Result<Option<LayerAttrs>, StoreError> {
        let descriptor = descriptor_attrs();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }

    pub fn frozen_ancestor(&self, layer: LayerId) -> Result<Option<LayerId>, StoreError> {
        let mut current = self.attrs(layer)?.and_then(|attrs| attrs.parent);
        let mut seen = std::collections::HashSet::new();
        while let Some(ancestor) = current {
            if !seen.insert(ancestor) {
                break;
            }
            let is_group = self
                .meta(ancestor)?
                .map(|meta| meta.source == LayerSource::Group)
                .unwrap_or(false);
            if is_group && self.attrs(ancestor)?.unwrap_or_default().frozen {
                return Ok(Some(ancestor));
            }
            current = self.attrs(ancestor)?.and_then(|attrs| attrs.parent);
        }
        Ok(None)
    }

    pub fn effects(&self, layer: LayerId) -> Result<Vec<EffectInstance>, StoreError> {
        let descriptor = descriptor_effects();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }


    pub fn shapes(&self, layer: LayerId) -> Result<Vec<ShapeNode>, StoreError> {
        if !self.ignore_transients {
            if let Some(shapes) = self.preview_edits.shapes.get(&layer) { return Ok(shapes.clone()); }
        }
        let descriptor = descriptor_shapes();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(Vec::new());
        };
        serde_json::from_str(&json.0).map_err(StoreError::Encode)
    }

    pub fn text_document(&self, layer: LayerId) -> Result<Option<TextDocument>, StoreError> {
        if !self.ignore_transients {
            if let Some(document) = self.preview_edits.texts.get(&layer) { return Ok(Some(document.clone())); }
        }
        let descriptor = descriptor_text();
        let path = layer.entity_path();
        let results = self
            .db
            .latest_at(&self.query(), &path, [descriptor.component]);
        let Some(json) = results
            .component_batch::<TrackJson>(descriptor.component)
            .and_then(|batch| batch.into_iter().next())
        else {
            return Ok(None);
        };
        serde_json::from_str(&json.0)
            .map(Some)
            .map_err(StoreError::Encode)
    }
}

