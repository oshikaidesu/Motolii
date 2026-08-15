//! D1b: 保存前のドキュメント不変条件検証(ガード1)。
//! D1h: DocParam期待型・空トラック拒否・AssetRef結線・NaN/Inf/値域(S3/S4/S9)。
//!
//! 壊れた状態を「正常に」シリアライズしないための判定口。
//! 実際のアトミック書き込み拒否はD1cがこの結果を見る。

use std::collections::{HashMap, HashSet};

use motolii_core::{RationalTime, TimeMapError};
use thiserror::Error;

use crate::asset::AssetId;
use crate::Document;

mod asset_uses;
mod items;
mod params;
mod stable_ids;

pub(crate) use asset_uses::AssetUse;
pub(crate) use params::{
    validate_interp_at, validate_keyframe_draft_values, validate_param, validate_param_structure,
};
pub(crate) use stable_ids::{collect_document_stable_ids, stable_id_in_use};

use asset_uses::{
    asset_use, collect_asset_uses_comp_camera, collect_asset_uses_item, collect_asset_uses_param,
};
use items::{
    detect_parent_cycles, document_has_any_effect_use, validate_comp_camera_doc, validate_item,
};
use stable_ids::{
    collect_stable_ids_comp_camera, collect_stable_ids_item, collect_stable_ids_param,
    item_uses_asset_components, note_stable_id,
};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DocumentError {
    #[error("Document.version ({version}) < min_reader_version ({min_reader_version})")]
    VersionBelowMinReader {
        version: u32,
        min_reader_version: u32,
    },
    #[error("composition.duration must be positive, got {duration:?}")]
    NonPositiveCompositionDuration { duration: RationalTime },
    #[error("track id {id} is not registered in track_ids")]
    UnknownTrackId { id: u64 },
    #[error("duplicate track id {id} in tracks")]
    DuplicateTrackId { id: u64 },
    #[error("layer id {id} is not registered in layers")]
    UnknownLayerId { id: u64 },
    #[error("duplicate layer id {id} in timeline items")]
    DuplicateLayerId { id: u64 },
    #[error("asset id {id} is not registered in assets")]
    UnknownAssetId { id: u64 },
    #[error("clip duration must be positive (layer {layer_id})")]
    NonPositiveClipDuration { layer_id: u64 },
    #[error("clip interval overflows (layer {layer_id})")]
    ClipIntervalOverflow { layer_id: u64 },
    #[error(
        "clip extends past composition duration (layer {layer_id}: end={end:?} > comp={comp:?})"
    )]
    ClipPastComposition {
        layer_id: u64,
        end: RationalTime,
        comp: RationalTime,
    },
    #[error("invalid clip time_map (layer {layer_id}): {source}")]
    InvalidTimeMap {
        layer_id: u64,
        #[source]
        source: TimeMapError,
    },
    #[error("transform.parent cycle involving layer {layer_id}")]
    ParentCycle { layer_id: u64 },
    /// D1l: `EffectUse.definition_id`が`effect_definitions`に存在しない(黙ってdropしない — GAP-14§4-1)。
    #[error("effect use {id} on layer {layer_id} references unknown definition {definition_id}")]
    DanglingEffectDefinition {
        layer_id: u64,
        id: u64,
        definition_id: u64,
    },
    #[error("effect definition {id} plugin_id must be non-empty")]
    EmptyEffectDefinitionPluginId { id: u64 },
    /// D1l: `EffectDefinition`/`EffectUse`を含む文書が宣言すべき`min_reader_version`下限。
    #[error(
        "document contains EffectDefinition/EffectUse but min_reader_version ({min_reader_version}) < {required} required (D1l)"
    )]
    EffectDefinitionsRequireNewerReader {
        min_reader_version: u32,
        required: u32,
    },
    #[error("clip plugin source plugin_id must be non-empty (layer {layer_id})")]
    EmptySourcePluginId { layer_id: u64 },
    /// D1f/実装ガード9: 既知plugin_idを構造上違う種別のスロットに置く「バグ」は
    /// degraded(警告)では救わず、型付きエラーで拒否する。
    #[error("plugin `{plugin_id}` at {path} is registered as {expected} but used as {got}")]
    PluginKindMismatch {
        path: String,
        plugin_id: String,
        expected: String,
        got: String,
    },
    #[error("param type mismatch at {path}: expected {expected}, got {got}")]
    ParamTypeMismatch {
        path: String,
        expected: String,
        got: String,
    },
    #[error("empty keyframe track at {path}")]
    EmptyKeyframeTrack { path: String },
    #[error("keyframe variant mismatch at {path}: expected {expected}, got {got}")]
    KeyframeVariantMismatch {
        path: String,
        expected: String,
        got: String,
    },
    #[error("non-finite value at {path}")]
    NonFiniteValue { path: String },
    #[error("value out of range at {path}")]
    ValueOutOfRange { path: String },
    #[error("spatial link (LookAt/Follow) not allowed at {path}")]
    SpatialLinkNotAllowed { path: String },
    #[error("non-finite Bezier control points at {path}")]
    NonFiniteBezier { path: String },
    #[error("invalid Bezier control points at {path}: x1={x1} x2={x2}")]
    InvalidBezier { path: String, x1: f64, x2: f64 },
    #[error("asset {id} has type `{got}` at {path}; expected one of: {expected}")]
    WrongAssetType {
        path: String,
        id: u64,
        got: String,
        expected: String,
    },
    /// A8: EffectId/KeyframeIdは1つのID空間を共有する(document-local安定u64 ID)。
    #[error("duplicate stable id {id} (EffectId/KeyframeId share one id space — A8)")]
    DuplicateStableId { id: u64 },
    #[error(transparent)]
    StableIdCounterInvalid(#[from] crate::stable_id::StableIdError),
    /// M2E-11①: ネスト(EffectInstance/DocKeyframe)への永続フィールド追加は
    /// `min_reader_version`を上げる規律。実在すれば強制する(旧readerでのresave時の消失を防ぐ)。
    #[error(
        "document contains EffectId/KeyframeId but min_reader_version ({min_reader_version}) < {required} required for stable ids (A8/D2)"
    )]
    StableIdsRequireNewerReader {
        min_reader_version: u32,
        required: u32,
    },
    /// AG-1: Asset Clipのvideo/audio component入れ子は`min_reader_version`を上げる。
    #[error(
        "document contains Asset Clip video/audio components but min_reader_version ({min_reader_version}) < {required} required for asset components (AG-1)"
    )]
    AssetComponentsRequireNewerReader {
        min_reader_version: u32,
        required: u32,
    },
    #[error("asset clip has neither video nor audio component (layer {layer_id})")]
    EmptyAssetComponents { layer_id: u64 },
    #[error("video component stream.kind must be video (layer {layer_id})")]
    VideoComponentKindMismatch { layer_id: u64 },
    #[error("audio component[{index}] stream.kind must be audio (layer {layer_id})")]
    AudioComponentKindMismatch { layer_id: u64, index: usize },
    #[error(
        "duplicate audio stream ordinal {ordinal} on layer {layer_id} (indices {first_index} and {second_index})"
    )]
    DuplicateAudioStreamOrdinal {
        layer_id: u64,
        ordinal: u32,
        first_index: usize,
        second_index: usize,
    },
    /// AG-1: decode/exportがvideo ordinal 0のみ。非0を黙ってv:0へ落とさない。
    #[error(
        "video stream ordinal {ordinal} is not supported yet (layer {layer_id}); only ordinal 0 is drawable in AG-1"
    )]
    UnsupportedVideoStreamOrdinal { layer_id: u64, ordinal: u32 },
    /// D1j: `composition.camera`を含む文書が宣言すべき`min_reader_version`下限。
    #[error(
        "document contains composition.camera but min_reader_version ({min_reader_version}) < {required} required (D1j)"
    )]
    CompCameraRequiresNewerReader {
        min_reader_version: u32,
        required: u32,
    },
    /// D1j: v1–v4 JSONにcamera形payloadを載せた版偽装はtyped reject。
    #[error(
        "document version {version} must not carry composition.camera before v{required}; use migrate (D1j)"
    )]
    CompCameraDisguisedOldVersion { version: u32, required: u32 },
}

/// A8/D2: `EffectInstance.id`/`DocKeyframe.id`を含む文書が宣言すべき最小`min_reader_version`。
pub(crate) const MIN_READER_VERSION_FOR_STABLE_IDS: u32 = 2;

/// AG-1: Asset Clip component入れ子を含む文書が宣言すべき最小`min_reader_version`。
pub const MIN_READER_VERSION_FOR_ASSET_COMPONENTS: u32 = 3;

/// D1l: `EffectDefinition`/`EffectUse`共有schemaを含む文書が宣言すべき最小`min_reader_version`。
pub const MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS: u32 = 4;

/// D1j: `composition.camera`を含む文書が宣言すべき最小`min_reader_version`。
pub const MIN_READER_VERSION_FOR_COMP_CAMERA: u32 = 5;

impl Document {
    /// 保存前不変条件。失敗しても`self`は変更しない(検証のみ)。
    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.version < self.min_reader_version {
            return Err(DocumentError::VersionBelowMinReader {
                version: self.version,
                min_reader_version: self.min_reader_version,
            });
        }
        if self.composition.duration <= RationalTime::ZERO {
            return Err(DocumentError::NonPositiveCompositionDuration {
                duration: self.composition.duration,
            });
        }
        self.validate_comp_camera()?;

        let mut seen_tracks = HashSet::new();
        // LayerIdはドキュメント全体で一意(LookAt/Followがトラック横断参照するため)
        let mut seen_layers = HashSet::new();
        // transform.parent の森性検査用(child → parent)
        let mut parents = HashMap::new();
        for track in &self.tracks {
            self.require_track(track.id)?;
            if !seen_tracks.insert(track.id.get()) {
                return Err(DocumentError::DuplicateTrackId { id: track.id.get() });
            }
            for item in &track.items {
                validate_item(self, item, &mut seen_layers, &mut parents)?;
            }
        }
        detect_parent_cycles(&parents)?;
        self.validate_stable_ids()?;
        self.validate_asset_component_reader_gate()?;
        self.validate_effect_definitions()?;
        self.validate_asset_uses()
    }

    pub(crate) fn asset_uses(&self) -> Vec<AssetUse> {
        let Document {
            version: _,
            min_reader_version: _,
            composition,
            bpm: _,
            soundtrack,
            assets: _,
            layers: _,
            track_ids: _,
            tracks,
            next_stable_id: _,
            effect_definitions,
            extra: _,
        } = self;
        let mut uses = Vec::new();
        collect_asset_uses_comp_camera(&composition.camera, &mut uses);
        if let Some(soundtrack) = soundtrack {
            uses.push(asset_use(soundtrack.asset, "soundtrack.asset", &[]));
        }
        for track in tracks {
            let crate::schema::Track { id: _, items } = track;
            for item in items {
                collect_asset_uses_item(item, &mut uses);
            }
        }
        for definition in effect_definitions {
            let crate::schema::EffectDefinition {
                id,
                plugin_id: _,
                effect_version: _,
                enabled: _,
                params,
                extra: _,
            } = definition;
            let base = format!("effect_definitions[{}]", id.get());
            for (name, param) in params {
                collect_asset_uses_param(param, &format!("{base}.{name}"), &mut uses);
            }
        }
        uses
    }

    pub(crate) fn asset_use_count(&self, id: AssetId) -> usize {
        self.asset_uses()
            .iter()
            .filter(|asset_use| asset_use.id == id)
            .count()
    }

    fn validate_asset_uses(&self) -> Result<(), DocumentError> {
        for asset_use in self.asset_uses() {
            let Some(asset) = self.assets.get(asset_use.id) else {
                return Err(DocumentError::UnknownAssetId {
                    id: asset_use.id.get(),
                });
            };
            if !asset_use.allowed_types.is_empty()
                && !asset_use
                    .allowed_types
                    .iter()
                    .any(|allowed| *allowed == asset.asset_type)
            {
                return Err(DocumentError::WrongAssetType {
                    path: asset_use.path,
                    id: asset_use.id.get(),
                    got: asset.asset_type.clone(),
                    expected: asset_use.allowed_types.join(", "),
                });
            }
        }
        Ok(())
    }

    /// D1l: `effect_definitions`台帳自体(plugin_id/params)とreader gateを検査する。
    /// 参照整合(dangling)は`validate_envelope`側で個々のUseについて検査する。
    fn validate_effect_definitions(&self) -> Result<(), DocumentError> {
        for def in &self.effect_definitions {
            if def.plugin_id.is_empty() {
                return Err(DocumentError::EmptyEffectDefinitionPluginId { id: def.id.get() });
            }
            let path = format!("effect_definitions[{}]", def.id.get());
            for (name, param) in &def.params {
                let p = format!("{path}.{name}");
                validate_param_structure(self, param, &p)?;
            }
        }
        let any_effects = !self.effect_definitions.is_empty() || document_has_any_effect_use(self);
        if any_effects && self.min_reader_version < MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS {
            return Err(DocumentError::EffectDefinitionsRequireNewerReader {
                min_reader_version: self.min_reader_version,
                required: MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS,
            });
        }
        Ok(())
    }

    /// D1j: planar cameraのreader gateとDocParam契約。
    fn validate_comp_camera(&self) -> Result<(), DocumentError> {
        if self.version < MIN_READER_VERSION_FOR_COMP_CAMERA {
            return Err(DocumentError::CompCameraDisguisedOldVersion {
                version: self.version,
                required: MIN_READER_VERSION_FOR_COMP_CAMERA,
            });
        }
        if self.min_reader_version < MIN_READER_VERSION_FOR_COMP_CAMERA {
            return Err(DocumentError::CompCameraRequiresNewerReader {
                min_reader_version: self.min_reader_version,
                required: MIN_READER_VERSION_FOR_COMP_CAMERA,
            });
        }
        validate_comp_camera_doc(self, &self.composition.camera, "composition.camera")
    }

    /// load経路: JSON上で旧版にcameraが載っていた場合の版偽装拒否。
    pub(crate) fn reject_disguised_comp_camera_wire(
        document_version: u32,
        composition_has_camera: bool,
    ) -> Result<(), DocumentError> {
        if document_version < MIN_READER_VERSION_FOR_COMP_CAMERA && composition_has_camera {
            return Err(DocumentError::CompCameraDisguisedOldVersion {
                version: document_version,
                required: MIN_READER_VERSION_FOR_COMP_CAMERA,
            });
        }
        Ok(())
    }

    /// AG-1: 非legacyのAsset componentを含む文書は`min_reader_version>=3`。
    fn validate_asset_component_reader_gate(&self) -> Result<(), DocumentError> {
        let mut needs = false;
        for track in &self.tracks {
            for item in &track.items {
                if item_uses_asset_components(item) {
                    needs = true;
                    break;
                }
            }
            if needs {
                break;
            }
        }
        if needs && self.min_reader_version < MIN_READER_VERSION_FOR_ASSET_COMPONENTS {
            return Err(DocumentError::AssetComponentsRequireNewerReader {
                min_reader_version: self.min_reader_version,
                required: MIN_READER_VERSION_FOR_ASSET_COMPONENTS,
            });
        }
        Ok(())
    }

    /// A8: EffectId/KeyframeIdの一意性・`next_stable_id`カウンタの整合性・
    /// stable id存在時の`min_reader_version`下限(M2E-11①のネスト規律を機械判定)。
    fn validate_stable_ids(&self) -> Result<(), DocumentError> {
        let mut seen = HashSet::new();
        let mut max_observed: Option<u64> = None;
        // D1l: EffectDefinitionIdもEffectId/KeyframeIdと同じ共有counterの空間(stable_id.rs)。
        for def in &self.effect_definitions {
            note_stable_id(def.id.get(), &mut seen, &mut max_observed)?;
            for param in def.params.values() {
                collect_stable_ids_param(param, &mut seen, &mut max_observed)?;
            }
        }
        for track in &self.tracks {
            for item in &track.items {
                collect_stable_ids_item(item, &mut seen, &mut max_observed)?;
            }
        }
        collect_stable_ids_comp_camera(&self.composition.camera, &mut seen, &mut max_observed)?;
        if !seen.is_empty() && self.min_reader_version < MIN_READER_VERSION_FOR_STABLE_IDS {
            return Err(DocumentError::StableIdsRequireNewerReader {
                min_reader_version: self.min_reader_version,
                required: MIN_READER_VERSION_FOR_STABLE_IDS,
            });
        }
        self.next_stable_id.validate_observed_max(max_observed)?;
        Ok(())
    }
}
