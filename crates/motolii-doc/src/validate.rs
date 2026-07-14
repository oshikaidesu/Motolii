//! D1b: 保存前のドキュメント不変条件検証(ガード1)。
//! D1h: DocParam期待型・空トラック拒否・AssetRef結線・NaN/Inf/値域(S3/S4/S9)。
//!
//! 壊れた状態を「正常に」シリアライズしないための判定口。
//! 実際のアトミック書き込み拒否はD1cがこの結果を見る。

use std::collections::{HashMap, HashSet};

use motolii_core::{RationalTime, TimeMapError};
use thiserror::Error;

use crate::asset::AssetId;
use crate::doc_keyframe::validate_interp;
use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::param_expect::{
    self, known_plugin_info, known_plugin_param, path_op_scalar, vec2_axis, DocPluginKind,
    ExpectedValueType, ParamConstraints,
};
use crate::schema::{
    asset_components_require_newer_reader, Clip, ClipSource, Group, ItemEnvelope, PathOp,
    StandardShape, StreamKind, TrackItem, Transform2D, VectorContent,
};
use crate::track_id::TrackId;
use crate::{Document, LayerId};

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
    #[error("effect plugin_id must be non-empty (layer {layer_id})")]
    EmptyEffectPluginId { layer_id: u64 },
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
}

/// A8/D2: `EffectInstance.id`/`DocKeyframe.id`を含む文書が宣言すべき最小`min_reader_version`。
pub(crate) const MIN_READER_VERSION_FOR_STABLE_IDS: u32 = 2;

/// AG-1: Asset Clip component入れ子を含む文書が宣言すべき最小`min_reader_version`。
pub const MIN_READER_VERSION_FOR_ASSET_COMPONENTS: u32 = 3;

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

        if let Some(st) = &self.soundtrack {
            self.require_asset(st.asset)?;
        }

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
        self.validate_asset_component_reader_gate()
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
        for track in &self.tracks {
            for item in &track.items {
                collect_stable_ids_item(item, &mut seen, &mut max_observed)?;
            }
        }
        if !seen.is_empty() && self.min_reader_version < MIN_READER_VERSION_FOR_STABLE_IDS {
            return Err(DocumentError::StableIdsRequireNewerReader {
                min_reader_version: self.min_reader_version,
                required: MIN_READER_VERSION_FOR_STABLE_IDS,
            });
        }
        self.next_stable_id.validate_observed_max(max_observed)?;
        Ok(())
    }

    fn require_track(&self, id: TrackId) -> Result<(), DocumentError> {
        if self.track_ids.contains(id) {
            Ok(())
        } else {
            Err(DocumentError::UnknownTrackId { id: id.get() })
        }
    }

    fn require_layer(&self, id: LayerId) -> Result<(), DocumentError> {
        if self.layers.contains(id) {
            Ok(())
        } else {
            Err(DocumentError::UnknownLayerId { id: id.get() })
        }
    }

    fn require_asset(&self, id: AssetId) -> Result<(), DocumentError> {
        if self.assets.get(id).is_some() {
            Ok(())
        } else {
            Err(DocumentError::UnknownAssetId { id: id.get() })
        }
    }

    fn require_asset_type(
        &self,
        id: AssetId,
        allowed: &[&str],
        path: &str,
    ) -> Result<(), DocumentError> {
        let Some(asset) = self.assets.get(id) else {
            return Err(DocumentError::UnknownAssetId { id: id.get() });
        };
        if allowed.iter().any(|t| *t == asset.asset_type) {
            Ok(())
        } else {
            Err(DocumentError::WrongAssetType {
                path: path.to_string(),
                id: id.get(),
                got: asset.asset_type.clone(),
                expected: allowed.join(", "),
            })
        }
    }
}

fn validate_item(
    doc: &Document,
    item: &TrackItem,
    seen_layers: &mut HashSet<u64>,
    parents: &mut HashMap<u64, u64>,
) -> Result<(), DocumentError> {
    match item {
        TrackItem::Clip(clip) => validate_clip(doc, clip, seen_layers, parents),
        TrackItem::Group(group) => validate_group(doc, group, seen_layers, parents),
    }
}

fn validate_group(
    doc: &Document,
    group: &Group,
    seen_layers: &mut HashSet<u64>,
    parents: &mut HashMap<u64, u64>,
) -> Result<(), DocumentError> {
    validate_envelope(doc, &group.envelope, seen_layers, parents)?;
    for child in &group.children {
        validate_item(doc, child, seen_layers, parents)?;
    }
    Ok(())
}

fn validate_clip(
    doc: &Document,
    clip: &Clip,
    seen_layers: &mut HashSet<u64>,
    parents: &mut HashMap<u64, u64>,
) -> Result<(), DocumentError> {
    let layer_id = clip.envelope.layer_id.get();
    validate_envelope(doc, &clip.envelope, seen_layers, parents)?;

    if clip.duration <= RationalTime::ZERO {
        return Err(DocumentError::NonPositiveClipDuration { layer_id });
    }
    // start の下限は検査しない: 負開始を許容(トリムイン相当。AM/AE互換)。
    // 区間正当性は duration>0 と半開終端 end <= composition.duration のみ。
    let end = clip
        .start
        .try_add(clip.duration)
        .map_err(|_| DocumentError::ClipIntervalOverflow { layer_id })?;
    if end > doc.composition.duration {
        return Err(DocumentError::ClipPastComposition {
            layer_id,
            end,
            comp: doc.composition.duration,
        });
    }

    // TimeMapはフィールドがpubのためedit経路で壊せる — deserialize拒否と同じ不変条件を保存前にも強制(監査T-2)
    clip.time_map
        .validate()
        .map_err(|source| DocumentError::InvalidTimeMap { layer_id, source })?;

    match &clip.source {
        ClipSource::Asset {
            asset,
            video,
            audio,
        } => {
            doc.require_asset(*asset)?;
            if video.is_none() && audio.is_empty() {
                return Err(DocumentError::EmptyAssetComponents { layer_id });
            }
            if let Some(video) = video {
                if video.stream.kind != StreamKind::Video {
                    return Err(DocumentError::VideoComponentKindMismatch { layer_id });
                }
            }
            for (index, comp) in audio.iter().enumerate() {
                if comp.stream.kind != StreamKind::Audio {
                    return Err(DocumentError::AudioComponentKindMismatch { layer_id, index });
                }
                validate_param(
                    doc,
                    &comp.gain,
                    ParamConstraints::min_f64(0.0),
                    &format!("layer{layer_id}.source.audio[{index}].gain"),
                )?;
            }
        }
        ClipSource::Plugin {
            plugin_id,
            effect_version,
            params,
            ..
        } => {
            if plugin_id.is_empty() {
                return Err(DocumentError::EmptySourcePluginId { layer_id });
            }
            let source_path = format!("layer{layer_id}.source");
            let degraded = plugin_slot_degraded(
                plugin_id,
                *effect_version,
                DocPluginKind::LayerSource,
                &source_path,
            )?;
            for (name, param) in params {
                let path = format!("{source_path}.{name}");
                validate_plugin_param(doc, plugin_id, name, param, &path, degraded)?;
            }
        }
        ClipSource::Vector { recipe } => {
            validate_vector_content(doc, &recipe.content, &format!("layer{layer_id}.recipe"))?;
            for (i, op) in recipe.modifiers.iter().enumerate() {
                validate_path_op_params(
                    doc,
                    op,
                    &format!("layer{layer_id}.recipe.modifiers[{i}]"),
                )?;
            }
        }
    }

    Ok(())
}

fn validate_envelope(
    doc: &Document,
    env: &ItemEnvelope,
    seen_layers: &mut HashSet<u64>,
    parents: &mut HashMap<u64, u64>,
) -> Result<(), DocumentError> {
    let id = env.layer_id.get();
    doc.require_layer(env.layer_id)?;
    if !seen_layers.insert(id) {
        return Err(DocumentError::DuplicateLayerId { id });
    }
    if let Some(parent) = env.transform.parent {
        doc.require_layer(parent)?;
        parents.insert(id, parent.get());
    }
    let base = format!("layer{id}");
    validate_transform2d(doc, &env.transform, &base)?;
    validate_param(
        doc,
        &env.opacity,
        param_expect::envelope_opacity(),
        &format!("{base}.opacity"),
    )?;
    for effect in &env.effects {
        if effect.plugin_id.is_empty() {
            return Err(DocumentError::EmptyEffectPluginId { layer_id: id });
        }
        let effect_path = format!("{base}.effect[{}]", effect.plugin_id);
        let degraded = plugin_slot_degraded(
            &effect.plugin_id,
            effect.effect_version,
            DocPluginKind::Filter,
            &effect_path,
        )?;
        for (name, param) in &effect.params {
            let path = format!("{effect_path}.{name}");
            validate_plugin_param(doc, &effect.plugin_id, name, param, &path, degraded)?;
        }
    }
    Ok(())
}

/// D1f/S13: 既知plugin_idの種別違いは型付きエラー、未知idと未来版effect_versionは
/// 同一のdegraded扱い(構造検査のみ・型表チェックはスキップ)にする。
fn plugin_slot_degraded(
    plugin_id: &str,
    effect_version: u32,
    expected_kind: DocPluginKind,
    path: &str,
) -> Result<bool, DocumentError> {
    match known_plugin_info(plugin_id) {
        None => Ok(true),
        Some(info) if info.kind != expected_kind => Err(DocumentError::PluginKindMismatch {
            path: path.to_string(),
            plugin_id: plugin_id.to_string(),
            expected: expected_kind.name().to_string(),
            got: info.kind.name().to_string(),
        }),
        Some(info) => Ok(effect_version > info.current_version),
    }
}

/// Transform2Dの4スロット共通検査。エンベロープ本体とRepeater.transformで共用(D1i-2)。
fn validate_transform2d(doc: &Document, t: &Transform2D, base: &str) -> Result<(), DocumentError> {
    validate_param(
        doc,
        &t.position,
        param_expect::transform_position(),
        &format!("{base}.position"),
    )?;
    validate_param(
        doc,
        &t.anchor,
        param_expect::transform_anchor(),
        &format!("{base}.anchor"),
    )?;
    validate_param(
        doc,
        &t.scale,
        param_expect::transform_scale(),
        &format!("{base}.scale"),
    )?;
    validate_param(
        doc,
        &t.rotation,
        param_expect::transform_rotation(),
        &format!("{base}.rotation"),
    )
}

fn validate_plugin_param(
    doc: &Document,
    plugin_id: &str,
    param_id: &str,
    param: &DocParam,
    path: &str,
    degraded: bool,
) -> Result<(), DocumentError> {
    if !degraded {
        if let Some(c) = known_plugin_param(plugin_id, param_id) {
            return validate_param(doc, param, c, path);
        }
    }
    // 未知plugin・既知プラグインの未来版: 型表は当てず有限性・AssetRefダングリングのみ検査(F-9/D1f)
    validate_param_structure(doc, param, path)
}

fn detect_parent_cycles(parents: &HashMap<u64, u64>) -> Result<(), DocumentError> {
    for &start in parents.keys() {
        let mut path = HashSet::new();
        let mut cur = start;
        loop {
            if !path.insert(cur) {
                return Err(DocumentError::ParentCycle { layer_id: cur });
            }
            match parents.get(&cur) {
                Some(&p) if p == cur => {
                    return Err(DocumentError::ParentCycle { layer_id: cur });
                }
                Some(&p) => cur = p,
                None => break,
            }
        }
    }
    Ok(())
}

fn validate_param(
    doc: &Document,
    param: &DocParam,
    constraints: ParamConstraints,
    path: &str,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(v) => validate_value(doc, v, constraints, path),
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(DocumentError::EmptyKeyframeTrack {
                    path: path.to_string(),
                });
            }
            let mut expected_kind: Option<&'static str> = None;
            for key in track.keys() {
                let kind = key.value.kind_name();
                match expected_kind {
                    None => expected_kind = Some(kind),
                    Some(prev) if prev != kind => {
                        return Err(DocumentError::KeyframeVariantMismatch {
                            path: path.to_string(),
                            expected: prev.to_string(),
                            got: kind.to_string(),
                        });
                    }
                    Some(_) => {}
                }
                validate_interp_at(path, &key.interp)?;
                validate_value(doc, &key.value, constraints, path)?;
            }
            Ok(())
        }
        DocParam::Data { fallback, .. } => validate_value(doc, fallback, constraints, path),
        DocParam::Vec2Axes { x, y } => {
            if constraints.expected != ExpectedValueType::Vec2 {
                return Err(DocumentError::ParamTypeMismatch {
                    path: path.to_string(),
                    expected: constraints.expected.name().to_string(),
                    got: "Vec2Axes".to_string(),
                });
            }
            validate_param(doc, x, vec2_axis(), &format!("{path}.x"))?;
            validate_param(doc, y, vec2_axis(), &format!("{path}.y"))
        }
        DocParam::LookAt { target, .. } => {
            if !constraints.allow_look_at {
                return Err(DocumentError::SpatialLinkNotAllowed {
                    path: path.to_string(),
                });
            }
            doc.require_layer(*target)
        }
        DocParam::Follow { target, offset } => {
            if !constraints.allow_follow {
                return Err(DocumentError::SpatialLinkNotAllowed {
                    path: path.to_string(),
                });
            }
            if !offset[0].is_finite() || !offset[1].is_finite() {
                return Err(DocumentError::NonFiniteValue {
                    path: format!("{path}.offset"),
                });
            }
            doc.require_layer(*target)
        }
    }
}

/// 未知plugin向け: 期待型なし。有限性・AssetRef存在・Bezierのみ。
fn validate_param_structure(
    doc: &Document,
    param: &DocParam,
    path: &str,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(v) => validate_value_structure(doc, v, path),
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(DocumentError::EmptyKeyframeTrack {
                    path: path.to_string(),
                });
            }
            let mut expected_kind: Option<&'static str> = None;
            for key in track.keys() {
                let kind = key.value.kind_name();
                match expected_kind {
                    None => expected_kind = Some(kind),
                    Some(prev) if prev != kind => {
                        return Err(DocumentError::KeyframeVariantMismatch {
                            path: path.to_string(),
                            expected: prev.to_string(),
                            got: kind.to_string(),
                        });
                    }
                    Some(_) => {}
                }
                validate_interp_at(path, &key.interp)?;
                validate_value_structure(doc, &key.value, path)?;
            }
            Ok(())
        }
        DocParam::Data { fallback, .. } => validate_value_structure(doc, fallback, path),
        DocParam::Vec2Axes { x, y } => {
            validate_param_structure(doc, x, &format!("{path}.x"))?;
            validate_param_structure(doc, y, &format!("{path}.y"))
        }
        DocParam::LookAt { target, .. } | DocParam::Follow { target, .. } => {
            doc.require_layer(*target)
        }
    }
}

fn validate_interp_at(path: &str, interp: &motolii_eval::Interp) -> Result<(), DocumentError> {
    validate_interp(interp).map_err(|e| match e {
        crate::doc_keyframe::DocKeyframeError::NonFiniteBezier => DocumentError::NonFiniteBezier {
            path: path.to_string(),
        },
        crate::doc_keyframe::DocKeyframeError::InvalidBezier { x1, x2 } => {
            DocumentError::InvalidBezier {
                path: path.to_string(),
                x1,
                x2,
            }
        }
        other => DocumentError::NonFiniteBezier {
            path: format!("{path} ({other})"),
        },
    })
}

fn validate_value(
    doc: &Document,
    value: &DocValue,
    constraints: ParamConstraints,
    path: &str,
) -> Result<(), DocumentError> {
    if !constraints.expected.matches(value) {
        return Err(DocumentError::ParamTypeMismatch {
            path: path.to_string(),
            expected: constraints.expected.name().to_string(),
            got: value.kind_name().to_string(),
        });
    }
    validate_value_structure(doc, value, path)?;
    if constraints.unit_interval {
        match value {
            DocValue::F64(v) if !(0.0..=1.0).contains(v) => {
                return Err(DocumentError::ValueOutOfRange {
                    path: path.to_string(),
                });
            }
            DocValue::Color(c) if c.iter().any(|x| !(0.0..=1.0).contains(x)) => {
                return Err(DocumentError::ValueOutOfRange {
                    path: path.to_string(),
                });
            }
            _ => {}
        }
    }
    if let DocValue::F64(v) = value {
        if constraints.min.is_some_and(|min| *v < min)
            || constraints.max.is_some_and(|max| *v > max)
        {
            return Err(DocumentError::ValueOutOfRange {
                path: path.to_string(),
            });
        }
        if constraints.integer && v.fract().abs() > f64::EPSILON {
            return Err(DocumentError::ValueOutOfRange {
                path: path.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_value_structure(
    doc: &Document,
    value: &DocValue,
    path: &str,
) -> Result<(), DocumentError> {
    match value {
        DocValue::F64(v) => {
            if !v.is_finite() {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Vec2(v) => {
            if v.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Vec3(v) => {
            if v.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Color(c) => {
            if c.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::AssetRef(id) => {
            doc.require_asset(*id)?;
        }
    }
    Ok(())
}

fn validate_vector_content(
    doc: &Document,
    content: &VectorContent,
    path: &str,
) -> Result<(), DocumentError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                validate_param(doc, width, path_op_scalar(), &format!("{path}.width"))?;
                validate_param(doc, height, path_op_scalar(), &format!("{path}.height"))
            }
        },
        VectorContent::SvgAsset { asset } => {
            // S6: ラスタ動画等を SvgAsset に混ぜて modifiers を付けられないよう型を固定
            doc.require_asset_type(*asset, &[SVG_ASSET_TYPE], &format!("{path}.asset"))
        }
        VectorContent::TextPath { font_asset, .. } => {
            doc.require_asset_type(*font_asset, FONT_ASSET_TYPES, &format!("{path}.font_asset"))
        }
        VectorContent::Group { children } => {
            for (i, child) in children.iter().enumerate() {
                validate_vector_content(doc, child, &format!("{path}.children[{i}]"))?;
            }
            Ok(())
        }
    }
}

fn note_stable_id(
    id: u64,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    if !seen.insert(id) {
        return Err(DocumentError::DuplicateStableId { id });
    }
    *max_observed = Some(max_observed.map_or(id, |m| m.max(id)));
    Ok(())
}

fn item_uses_asset_components(item: &TrackItem) -> bool {
    match item {
        TrackItem::Clip(clip) => match &clip.source {
            ClipSource::Asset { video, audio, .. } => {
                asset_components_require_newer_reader(video, audio)
            }
            _ => false,
        },
        TrackItem::Group(group) => group.children.iter().any(item_uses_asset_components),
    }
}

fn collect_stable_ids_item(
    item: &TrackItem,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match item {
        TrackItem::Clip(clip) => {
            collect_stable_ids_envelope(&clip.envelope, seen, max_observed)?;
            match &clip.source {
                ClipSource::Asset { .. } => Ok(()),
                ClipSource::Plugin { params, .. } => {
                    for param in params.values() {
                        collect_stable_ids_param(param, seen, max_observed)?;
                    }
                    Ok(())
                }
                ClipSource::Vector { recipe } => {
                    collect_stable_ids_vector_content(&recipe.content, seen, max_observed)?;
                    for op in &recipe.modifiers {
                        collect_stable_ids_path_op(op, seen, max_observed)?;
                    }
                    Ok(())
                }
            }
        }
        TrackItem::Group(group) => {
            collect_stable_ids_envelope(&group.envelope, seen, max_observed)?;
            for child in &group.children {
                collect_stable_ids_item(child, seen, max_observed)?;
            }
            Ok(())
        }
    }
}

fn collect_stable_ids_envelope(
    env: &ItemEnvelope,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    collect_stable_ids_param(&env.transform.position, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.anchor, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.scale, seen, max_observed)?;
    collect_stable_ids_param(&env.transform.rotation, seen, max_observed)?;
    collect_stable_ids_param(&env.opacity, seen, max_observed)?;
    for effect in &env.effects {
        note_stable_id(effect.id.get(), seen, max_observed)?;
        for param in effect.params.values() {
            collect_stable_ids_param(param, seen, max_observed)?;
        }
    }
    Ok(())
}

fn collect_stable_ids_param(
    param: &DocParam,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(_)
        | DocParam::Data { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => Ok(()),
        DocParam::Keyframes(track) => {
            for key in track.keys() {
                note_stable_id(key.id.get(), seen, max_observed)?;
            }
            Ok(())
        }
        DocParam::Vec2Axes { x, y } => {
            collect_stable_ids_param(x, seen, max_observed)?;
            collect_stable_ids_param(y, seen, max_observed)
        }
    }
}

fn collect_stable_ids_vector_content(
    content: &VectorContent,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                collect_stable_ids_param(width, seen, max_observed)?;
                collect_stable_ids_param(height, seen, max_observed)
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => Ok(()),
        VectorContent::Group { children } => {
            for child in children {
                collect_stable_ids_vector_content(child, seen, max_observed)?;
            }
            Ok(())
        }
    }
}

fn collect_stable_ids_path_op(
    op: &PathOp,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    match op {
        PathOp::PuckerBloat { amount } => collect_stable_ids_param(amount, seen, max_observed),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            collect_stable_ids_param(amount, seen, max_observed)?;
            collect_stable_ids_param(ridges, seen, max_observed)
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => collect_stable_ids_param(distance, seen, max_observed),
        PathOp::RoundCorners { radius } => collect_stable_ids_param(radius, seen, max_observed),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            collect_stable_ids_param(start, seen, max_observed)?;
            collect_stable_ids_param(end, seen, max_observed)?;
            collect_stable_ids_param(offset, seen, max_observed)
        }
        PathOp::Twist { angle, center } => {
            collect_stable_ids_param(angle, seen, max_observed)?;
            collect_stable_ids_param(center, seen, max_observed)
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            collect_stable_ids_param(amp, seen, max_observed)?;
            collect_stable_ids_param(freq, seen, max_observed)
            // seedはu64固定(非DocParam) — stable id走査対象外。
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            collect_stable_ids_param(copies, seen, max_observed)?;
            collect_stable_ids_param(offset, seen, max_observed)?;
            collect_stable_ids_transform2d(transform, seen, max_observed)?;
            collect_stable_ids_param(start_opacity, seen, max_observed)?;
            collect_stable_ids_param(end_opacity, seen, max_observed)
        }
    }
}

fn collect_stable_ids_transform2d(
    transform: &Transform2D,
    seen: &mut HashSet<u64>,
    max_observed: &mut Option<u64>,
) -> Result<(), DocumentError> {
    collect_stable_ids_param(&transform.position, seen, max_observed)?;
    collect_stable_ids_param(&transform.anchor, seen, max_observed)?;
    collect_stable_ids_param(&transform.scale, seen, max_observed)?;
    collect_stable_ids_param(&transform.rotation, seen, max_observed)
}

/// `VectorContent::SvgAsset` が要求する MIME。
const SVG_ASSET_TYPE: &str = "image/svg+xml";

/// `TextPath.font_asset` の許可型(D1i-1で確定。未決を埋めずここで正本化)。
const FONT_ASSET_TYPES: &[&str] = &["font/ttf", "font/otf", "font/woff", "font/woff2"];

/// PathOp意味論表(D1i-2)の拒否項目をここで型付きエラーに落とす。
/// open-path Offsetの拒否は幾何側(`pathgeom::apply`)の責務 — validateはDocumentの
/// 静的スキーマしか見えず、SvgAsset/TextPath由来パスの開閉はレシピからは判定できない。
fn validate_path_op_params(
    doc: &Document,
    op: &crate::schema::PathOp,
    path: &str,
) -> Result<(), DocumentError> {
    use crate::schema::PathOp;
    let scalar = path_op_scalar();
    match op {
        PathOp::PuckerBloat { amount } => validate_param(
            doc,
            amount,
            param_expect::path_op_pucker_bloat_amount(),
            &format!("{path}.amount"),
        ),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            validate_param(
                doc,
                amount,
                param_expect::path_op_non_negative(),
                &format!("{path}.amount"),
            )?;
            validate_param(
                doc,
                ridges,
                param_expect::path_op_non_negative(),
                &format!("{path}.ridges"),
            )
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit,
        } => {
            validate_param(doc, distance, scalar, &format!("{path}.distance"))?;
            if !miter_limit.is_finite() {
                return Err(DocumentError::NonFiniteValue {
                    path: format!("{path}.miter_limit"),
                });
            }
            if *miter_limit <= 0.0 {
                return Err(DocumentError::ValueOutOfRange {
                    path: format!("{path}.miter_limit"),
                });
            }
            Ok(())
        }
        PathOp::RoundCorners { radius } => validate_param(
            doc,
            radius,
            param_expect::path_op_non_negative(),
            &format!("{path}.radius"),
        ),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            validate_param(
                doc,
                start,
                param_expect::path_op_unit_interval(),
                &format!("{path}.start"),
            )?;
            validate_param(
                doc,
                end,
                param_expect::path_op_unit_interval(),
                &format!("{path}.end"),
            )?;
            validate_param(doc, offset, scalar, &format!("{path}.offset"))
        }
        PathOp::Twist { angle, center } => {
            validate_param(doc, angle, scalar, &format!("{path}.angle"))?;
            validate_param(
                doc,
                center,
                param_expect::path_op_vec2(),
                &format!("{path}.center"),
            )
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            validate_param(doc, amp, scalar, &format!("{path}.amp"))?;
            validate_param(doc, freq, scalar, &format!("{path}.freq"))
            // seedはu64固定(非DocParam) — 型で非有限値・キーフレームを構文上排除済み。
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            validate_param(
                doc,
                copies,
                param_expect::path_op_non_negative_integer(),
                &format!("{path}.copies"),
            )?;
            validate_param(doc, offset, scalar, &format!("{path}.offset"))?;
            validate_transform2d(doc, transform, &format!("{path}.transform"))?;
            validate_param(
                doc,
                start_opacity,
                param_expect::path_op_opacity(),
                &format!("{path}.start_opacity"),
            )?;
            validate_param(
                doc,
                end_opacity,
                param_expect::path_op_opacity(),
                &format!("{path}.end_opacity"),
            )
        }
    }
}
