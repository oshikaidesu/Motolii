//! D1b: 保存前のドキュメント不変条件検証(ガード1)。
//!
//! 壊れた状態を「正常に」シリアライズしないための判定口。
//! 実際のアトミック書き込み拒否はD1cがこの結果を見る。

use std::collections::{HashMap, HashSet};

use motolii_core::{RationalTime, TimeMapError};
use thiserror::Error;

use crate::asset::AssetId;
use crate::param::DocParam;
use crate::schema::{Clip, ClipSource, Group, ItemEnvelope, TrackItem};
use crate::track_id::TrackId;
use crate::{Document, LayerId};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DocumentError {
    #[error("Document.version ({version}) < min_reader_version ({min_reader_version})")]
    VersionBelowMinReader {
        version: u32,
        min_reader_version: u32,
    },
    /// v2 スキーマ追加以降は min_reader を同時に上げる(ガード7)。
    #[error(
        "Document.version {version} requires min_reader_version >= 2, got {min_reader_version}"
    )]
    MinReaderTooLowForVersion {
        version: u32,
        min_reader_version: u32,
    },
    /// v1 に v2 フィールドを載せた不正状態(シリアライズ汚染防止)。
    #[error("color_interpretation must be absent when version < 2")]
    ColorInterpretationOnV1,
    #[error("color_interpretation is required when version >= 2")]
    ColorInterpretationRequiredForV2,
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
}

impl Document {
    /// 保存前不変条件。失敗しても`self`は変更しない(検証のみ)。
    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.version < self.min_reader_version {
            return Err(DocumentError::VersionBelowMinReader {
                version: self.version,
                min_reader_version: self.min_reader_version,
            });
        }
        if self.version >= 2 && self.min_reader_version < 2 {
            return Err(DocumentError::MinReaderTooLowForVersion {
                version: self.version,
                min_reader_version: self.min_reader_version,
            });
        }
        if self.version < 2 && self.color_interpretation.is_some() {
            return Err(DocumentError::ColorInterpretationOnV1);
        }
        if self.version >= 2 && self.color_interpretation.is_none() {
            return Err(DocumentError::ColorInterpretationRequiredForV2);
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
        detect_parent_cycles(&parents)
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
        ClipSource::Asset { asset } => doc.require_asset(*asset)?,
        ClipSource::Plugin {
            plugin_id, params, ..
        } => {
            if plugin_id.is_empty() {
                return Err(DocumentError::EmptySourcePluginId { layer_id });
            }
            for param in params.values() {
                validate_param(doc, param)?;
            }
        }
    }

    for op in &clip.path_ops {
        validate_path_op_params(doc, op)?;
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
    validate_param(doc, &env.transform.position)?;
    validate_param(doc, &env.transform.anchor)?;
    validate_param(doc, &env.transform.scale)?;
    validate_param(doc, &env.transform.rotation)?;
    validate_param(doc, &env.opacity)?;
    for effect in &env.effects {
        if effect.plugin_id.is_empty() {
            return Err(DocumentError::EmptyEffectPluginId { layer_id: id });
        }
        for param in effect.params.values() {
            validate_param(doc, param)?;
        }
    }
    Ok(())
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

fn validate_param(doc: &Document, param: &DocParam) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(_) | DocParam::Keyframes(_) | DocParam::Data { .. } => Ok(()),
        DocParam::Vec2Axes { x, y } => {
            validate_param(doc, x)?;
            validate_param(doc, y)
        }
        DocParam::LookAt { target, .. } | DocParam::Follow { target, .. } => {
            doc.require_layer(*target)
        }
    }
}

fn validate_path_op_params(
    doc: &Document,
    op: &crate::schema::PathOp,
) -> Result<(), DocumentError> {
    use crate::schema::PathOp;
    match op {
        PathOp::PuckerBloat { amount } => validate_param(doc, amount),
        PathOp::ZigZag { amount, ridges } => {
            validate_param(doc, amount)?;
            validate_param(doc, ridges)
        }
        PathOp::Offset { distance } => validate_param(doc, distance),
        PathOp::RoundCorners { radius } => validate_param(doc, radius),
        PathOp::Trim { start, end, offset } => {
            validate_param(doc, start)?;
            validate_param(doc, end)?;
            validate_param(doc, offset)
        }
        PathOp::Twist { angle } => validate_param(doc, angle),
        PathOp::Wiggle { amp, freq, seed } => {
            validate_param(doc, amp)?;
            validate_param(doc, freq)?;
            validate_param(doc, seed)
        }
        PathOp::Repeater { copies, offset } => {
            validate_param(doc, copies)?;
            validate_param(doc, offset)
        }
    }
}
