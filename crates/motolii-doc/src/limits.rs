//! D1c-FU(#101, 監査S10): 入力資源上限(`ResourceLimits`)。
//!
//! ファイルbytes・Group深度・Track/Layer/Key数・string bytes・extra bytes・
//! command payload bytes・journal bytes・sample数を **1つのlimits policy** へ集約する。
//! ロード入口(`persist::load_document_with_limits`)へ注入可能。production既定値は
//! 運用調整値であり、永続JSON・migration・plugin契約には焼かない(#101完了条件)。
//!
//! command payload / journal / sample はD1d(#105)/D2/D4がまだ存在しないため、
//! ここでは検査プリミティブ(`check_*`)のみを提供し、Document構造の走査には含めない。
//! D1d/D2/D4は上限を別定義せず、ここの`ResourceLimits`を再利用する(実装ガード)。

use serde_json::{Map, Value as JsonValue};
use thiserror::Error;

use crate::param::DocParam;
use crate::schema::{
    Clip, ClipSource, EffectInstance, Group, ItemEnvelope, PathOp, StandardShape, TrackItem,
    Transform2D, VectorContent,
};
use crate::Document;

/// 入力資源上限。呼出側(I/O境界)が注入する単一policy(S10)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    /// ロード対象ファイルの総bytes。
    pub max_file_bytes: u64,
    /// `Group`の入れ子深度(トラック直下=1)。
    pub max_group_depth: u32,
    /// `Document.tracks`の要素数。
    pub max_tracks: u32,
    /// `Document.layers`(LayerIdTable)の登録数。
    pub max_layers: u32,
    /// `DocParam::Keyframes`1本あたりのキー数。
    pub max_keys_per_track: u32,
    /// 個々の文字列フィールドのbytes長(asset名・パス・plugin_id・テキスト等)。
    pub max_string_bytes: u32,
    /// `extra` flatten(Document/EffectInstance/ClipSource::Plugin)のJSONシリアライズbytes。
    pub max_extra_bytes: u32,
    /// コマンド(D2)1件のpayload bytes。D1d/D2はこの値を再利用する。
    pub max_command_payload_bytes: u32,
    /// ジャーナル(D1d)総量bytes。
    pub max_journal_bytes: u64,
    /// 音声サンプル数(D4)。
    pub max_samples: u64,
}

impl ResourceLimits {
    /// production既定(運用調整値)。**仕様・永続JSON・migration・plugin契約には焼かない**。
    pub const fn production() -> Self {
        Self {
            max_file_bytes: 512 * 1024 * 1024,
            max_group_depth: 64,
            max_tracks: 4_096,
            max_layers: 100_000,
            max_keys_per_track: 100_000,
            max_string_bytes: 1024 * 1024,
            max_extra_bytes: 4 * 1024 * 1024,
            max_command_payload_bytes: 4 * 1024 * 1024,
            max_journal_bytes: 8 * 1024 * 1024 * 1024,
            max_samples: 48_000 * 60 * 60 * 4,
        }
    }

    pub fn check_file_bytes(&self, observed: u64) -> Result<(), ResourceLimitError> {
        if observed > self.max_file_bytes {
            Err(ResourceLimitError::FileBytes {
                observed,
                limit: self.max_file_bytes,
            })
        } else {
            Ok(())
        }
    }

    /// D1d/D2向けプリミティブ。ジャーナル/コマンドが未実装のため呼び出し元はまだ無いが、
    /// #101完了条件(全limits項目の境界テスト)としてここで単体検査可能にする。
    pub fn check_command_payload_bytes(&self, observed: u32) -> Result<(), ResourceLimitError> {
        if observed > self.max_command_payload_bytes {
            Err(ResourceLimitError::CommandPayloadBytes {
                observed,
                limit: self.max_command_payload_bytes,
            })
        } else {
            Ok(())
        }
    }

    pub fn check_journal_bytes(&self, observed: u64) -> Result<(), ResourceLimitError> {
        if observed > self.max_journal_bytes {
            Err(ResourceLimitError::JournalBytes {
                observed,
                limit: self.max_journal_bytes,
            })
        } else {
            Ok(())
        }
    }

    pub fn check_sample_count(&self, observed: u64) -> Result<(), ResourceLimitError> {
        if observed > self.max_samples {
            Err(ResourceLimitError::SampleCount {
                observed,
                limit: self.max_samples,
            })
        } else {
            Ok(())
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::production()
    }
}

/// 上限超過。項目ごとに観測値/上限を持つ(呼出側が種別でmatchできる)。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceLimitError {
    #[error("file size {observed} bytes exceeds limit {limit} bytes")]
    FileBytes { observed: u64, limit: u64 },
    #[error("group nesting depth {observed} exceeds limit {limit} at {path}")]
    GroupDepth {
        path: String,
        observed: u32,
        limit: u32,
    },
    #[error("track count {observed} exceeds limit {limit}")]
    TrackCount { observed: u32, limit: u32 },
    #[error("layer count {observed} exceeds limit {limit}")]
    LayerCount { observed: u32, limit: u32 },
    #[error("keyframe count {observed} exceeds limit {limit} at {path}")]
    KeyCount {
        path: String,
        observed: u32,
        limit: u32,
    },
    #[error("string length {observed} bytes exceeds limit {limit} bytes at {path}")]
    StringBytes {
        path: String,
        observed: u32,
        limit: u32,
    },
    #[error("extra field payload {observed} bytes exceeds limit {limit} bytes at {path}")]
    ExtraBytes {
        path: String,
        observed: u32,
        limit: u32,
    },
    #[error("command payload {observed} bytes exceeds limit {limit} bytes")]
    CommandPayloadBytes { observed: u32, limit: u32 },
    #[error("journal size {observed} bytes exceeds limit {limit} bytes")]
    JournalBytes { observed: u64, limit: u64 },
    #[error("sample count {observed} exceeds limit {limit}")]
    SampleCount { observed: u64, limit: u64 },
}

/// Document構造を走査し、file bytes以外の全項目(Group深度/Track・Layer・Key数/
/// string bytes/extra bytes)を検査する。呼出元(persist)は読込直後に呼ぶ。
pub(crate) fn check_document_resource_limits(
    doc: &Document,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    let track_count = clamp_u32(doc.tracks.len());
    if track_count > limits.max_tracks {
        return Err(ResourceLimitError::TrackCount {
            observed: track_count,
            limit: limits.max_tracks,
        });
    }
    let layer_count = clamp_u32(doc.layers.len());
    if layer_count > limits.max_layers {
        return Err(ResourceLimitError::LayerCount {
            observed: layer_count,
            limit: limits.max_layers,
        });
    }

    check_extra(&doc.extra, "document.extra", limits)?;

    for (id, name) in doc.layers.iter() {
        check_string(name, &format!("layers[{}].name", id.get()), limits)?;
    }
    for (id, name) in doc.track_ids.iter() {
        check_string(name, &format!("track_ids[{}].name", id.get()), limits)?;
    }
    for asset in doc.assets.iter() {
        let base = format!("assets[{}]", asset.id.get());
        check_string(&asset.name, &format!("{base}.name"), limits)?;
        check_string(&asset.asset_type, &format!("{base}.asset_type"), limits)?;
        check_string(&asset.content_hash, &format!("{base}.content_hash"), limits)?;
        for (label, value) in [
            ("path_absolute", &asset.path_absolute),
            ("path_project_relative", &asset.path_project_relative),
            ("file_name", &asset.file_name),
            ("head_hash", &asset.head_hash),
            ("tail_hash", &asset.tail_hash),
        ] {
            if let Some(s) = value {
                check_string(s, &format!("{base}.{label}"), limits)?;
            }
        }
    }

    for track in &doc.tracks {
        let base = format!("track{}", track.id.get());
        for (i, item) in track.items.iter().enumerate() {
            check_track_item(item, 0, &format!("{base}.items[{i}]"), limits)?;
        }
    }
    Ok(())
}

fn check_track_item(
    item: &TrackItem,
    current_depth: u32,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    match item {
        TrackItem::Clip(clip) => check_clip(clip, path, limits),
        TrackItem::Group(group) => {
            let depth = current_depth + 1;
            if depth > limits.max_group_depth {
                return Err(ResourceLimitError::GroupDepth {
                    path: path.to_string(),
                    observed: depth,
                    limit: limits.max_group_depth,
                });
            }
            check_group(group, depth, path, limits)
        }
    }
}

fn check_group(
    group: &Group,
    depth: u32,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    check_envelope(&group.envelope, path, limits)?;
    for (i, child) in group.children.iter().enumerate() {
        check_track_item(child, depth, &format!("{path}.children[{i}]"), limits)?;
    }
    Ok(())
}

fn check_clip(clip: &Clip, path: &str, limits: &ResourceLimits) -> Result<(), ResourceLimitError> {
    check_envelope(&clip.envelope, path, limits)?;
    match &clip.source {
        ClipSource::Asset { .. } => Ok(()),
        ClipSource::Plugin {
            plugin_id,
            params,
            extra,
            ..
        } => {
            check_string(plugin_id, &format!("{path}.source.plugin_id"), limits)?;
            check_extra(extra, &format!("{path}.source.extra"), limits)?;
            for (name, param) in params {
                // キー名をpathへ埋め込む前に長さ検査 — 巨大param IDのすり抜けと、
                // 拒否メッセージ構築時の巨大formatを防ぐ。
                check_string(name, &format!("{path}.source.param_id"), limits)?;
                check_param(param, &format!("{path}.source.{name}"), limits)?;
            }
            Ok(())
        }
        ClipSource::Vector { recipe } => {
            check_vector_content(&recipe.content, &format!("{path}.recipe"), limits)?;
            for (i, op) in recipe.modifiers.iter().enumerate() {
                check_path_op(op, &format!("{path}.recipe.modifiers[{i}]"), limits)?;
            }
            Ok(())
        }
    }
}

fn check_envelope(
    env: &ItemEnvelope,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    check_param(&env.transform.position, &format!("{path}.position"), limits)?;
    check_param(&env.transform.anchor, &format!("{path}.anchor"), limits)?;
    check_param(&env.transform.scale, &format!("{path}.scale"), limits)?;
    check_param(&env.transform.rotation, &format!("{path}.rotation"), limits)?;
    check_param(&env.opacity, &format!("{path}.opacity"), limits)?;
    for (i, effect) in env.effects.iter().enumerate() {
        check_effect(effect, &format!("{path}.effects[{i}]"), limits)?;
    }
    Ok(())
}

fn check_effect(
    effect: &EffectInstance,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    check_string(&effect.plugin_id, &format!("{path}.plugin_id"), limits)?;
    check_extra(&effect.extra, &format!("{path}.extra"), limits)?;
    for (name, param) in &effect.params {
        check_string(name, &format!("{path}.param_id"), limits)?;
        check_param(param, &format!("{path}.{name}"), limits)?;
    }
    Ok(())
}

fn check_vector_content(
    content: &VectorContent,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                check_param(width, &format!("{path}.width"), limits)?;
                check_param(height, &format!("{path}.height"), limits)
            }
        },
        VectorContent::SvgAsset { .. } => Ok(()),
        VectorContent::TextPath { text, .. } => check_string(text, &format!("{path}.text"), limits),
        VectorContent::Group { children } => {
            for (i, child) in children.iter().enumerate() {
                check_vector_content(child, &format!("{path}.children[{i}]"), limits)?;
            }
            Ok(())
        }
    }
}

fn check_path_op(
    op: &PathOp,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    match op {
        PathOp::PuckerBloat { amount } => check_param(amount, &format!("{path}.amount"), limits),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            check_param(amount, &format!("{path}.amount"), limits)?;
            check_param(ridges, &format!("{path}.ridges"), limits)
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => check_param(distance, &format!("{path}.distance"), limits),
        PathOp::RoundCorners { radius } => check_param(radius, &format!("{path}.radius"), limits),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            check_param(start, &format!("{path}.start"), limits)?;
            check_param(end, &format!("{path}.end"), limits)?;
            check_param(offset, &format!("{path}.offset"), limits)
        }
        PathOp::Twist { angle, center } => {
            check_param(angle, &format!("{path}.angle"), limits)?;
            check_param(center, &format!("{path}.center"), limits)
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            check_param(amp, &format!("{path}.amp"), limits)?;
            check_param(freq, &format!("{path}.freq"), limits)
            // seedはu64固定(非DocParam) — キーフレーム走査対象外。
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            check_param(copies, &format!("{path}.copies"), limits)?;
            check_param(offset, &format!("{path}.offset"), limits)?;
            check_transform2d(transform, &format!("{path}.transform"), limits)?;
            check_param(start_opacity, &format!("{path}.start_opacity"), limits)?;
            check_param(end_opacity, &format!("{path}.end_opacity"), limits)
        }
    }
}

fn check_transform2d(
    transform: &Transform2D,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    check_param(&transform.position, &format!("{path}.position"), limits)?;
    check_param(&transform.anchor, &format!("{path}.anchor"), limits)?;
    check_param(&transform.scale, &format!("{path}.scale"), limits)?;
    check_param(&transform.rotation, &format!("{path}.rotation"), limits)
}

fn check_param(
    param: &DocParam,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    match param {
        DocParam::Const(_) => Ok(()),
        DocParam::Keyframes(track) => {
            let n = clamp_u32(track.keys().len());
            if n > limits.max_keys_per_track {
                return Err(ResourceLimitError::KeyCount {
                    path: path.to_string(),
                    observed: n,
                    limit: limits.max_keys_per_track,
                });
            }
            Ok(())
        }
        DocParam::Data { track, .. } => check_string(&track.0, &format!("{path}.track"), limits),
        DocParam::Vec2Axes { x, y } => {
            check_param(x, &format!("{path}.x"), limits)?;
            check_param(y, &format!("{path}.y"), limits)
        }
        DocParam::LookAt { .. } | DocParam::Follow { .. } => Ok(()),
    }
}

fn check_string(s: &str, path: &str, limits: &ResourceLimits) -> Result<(), ResourceLimitError> {
    let n = clamp_u32(s.len());
    if n > limits.max_string_bytes {
        return Err(ResourceLimitError::StringBytes {
            path: path.to_string(),
            observed: n,
            limit: limits.max_string_bytes,
        });
    }
    Ok(())
}

fn check_extra(
    extra: &Map<String, JsonValue>,
    path: &str,
    limits: &ResourceLimits,
) -> Result<(), ResourceLimitError> {
    if extra.is_empty() {
        return Ok(());
    }
    // extraは常にDeserialize済みJsonValueで構成される — シリアライズは失敗しない。
    let bytes = serde_json::to_vec(extra).unwrap_or_default();
    let n = clamp_u32(bytes.len());
    if n > limits.max_extra_bytes {
        return Err(ResourceLimitError::ExtraBytes {
            path: path.to_string(),
            observed: n,
            limit: limits.max_extra_bytes,
        });
    }
    Ok(())
}

fn clamp_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClippingMaskSettings, Track, Transform2D};
    use crate::{AssetTable, LayerIdTable, TrackIdTable};

    fn tiny_limits() -> ResourceLimits {
        ResourceLimits {
            max_file_bytes: 1_000,
            max_group_depth: 2,
            max_tracks: 2,
            max_layers: 2,
            max_keys_per_track: 2,
            max_string_bytes: 8,
            max_extra_bytes: 8,
            max_command_payload_bytes: 8,
            max_journal_bytes: 8,
            max_samples: 8,
        }
    }

    /// 他次元を突かないための緩い上限。1項目だけを狙う境界テストの土台にする
    /// (`"video/mp4"`が9bytesのため`tiny_limits`のstring上限8では常に落ちる=交絡)。
    fn generous_limits() -> ResourceLimits {
        ResourceLimits {
            max_file_bytes: 1_000_000,
            max_group_depth: 1_000,
            max_tracks: 1_000,
            max_layers: 1_000,
            max_keys_per_track: 1_000,
            max_string_bytes: 1_000,
            max_extra_bytes: 1_000,
            max_command_payload_bytes: 1_000,
            max_journal_bytes: 1_000,
            max_samples: 1_000,
        }
    }

    fn empty_doc() -> Document {
        Document {
            version: 1,
            min_reader_version: 1,
            composition: crate::schema::Composition::new_v1(),
            bpm: crate::Bpm::DEFAULT,
            soundtrack: None,
            assets: AssetTable::new(),
            layers: LayerIdTable::new(),
            track_ids: TrackIdTable::new(),
            tracks: Vec::new(),
            next_stable_id: Default::default(),
            extra: Map::new(),
        }
    }

    // --- 単発チェッカー(D1d/D2/D4がまだ無いためここで直接境界検査) ---

    #[test]
    fn file_bytes_boundary() {
        let limits = tiny_limits();
        assert!(limits.check_file_bytes(1_000).is_ok());
        assert_eq!(
            limits.check_file_bytes(1_001),
            Err(ResourceLimitError::FileBytes {
                observed: 1_001,
                limit: 1_000
            })
        );
    }

    #[test]
    fn command_payload_bytes_boundary() {
        let limits = tiny_limits();
        assert!(limits.check_command_payload_bytes(8).is_ok());
        assert_eq!(
            limits.check_command_payload_bytes(9),
            Err(ResourceLimitError::CommandPayloadBytes {
                observed: 9,
                limit: 8
            })
        );
    }

    #[test]
    fn journal_bytes_boundary() {
        let limits = tiny_limits();
        assert!(limits.check_journal_bytes(8).is_ok());
        assert_eq!(
            limits.check_journal_bytes(9),
            Err(ResourceLimitError::JournalBytes {
                observed: 9,
                limit: 8
            })
        );
    }

    #[test]
    fn sample_count_boundary() {
        let limits = tiny_limits();
        assert!(limits.check_sample_count(8).is_ok());
        assert_eq!(
            limits.check_sample_count(9),
            Err(ResourceLimitError::SampleCount {
                observed: 9,
                limit: 8
            })
        );
    }

    // --- Document構造ウォーカー ---

    #[test]
    fn track_count_boundary() {
        let limits = tiny_limits();
        let mut doc = empty_doc();
        for i in 0..2 {
            let id = doc.track_ids.allocate(format!("t{i}")).unwrap();
            doc.tracks.push(Track {
                id,
                items: Vec::new(),
            });
        }
        assert!(check_document_resource_limits(&doc, &limits).is_ok());

        let id = doc.track_ids.allocate("t2").unwrap();
        doc.tracks.push(Track {
            id,
            items: Vec::new(),
        });
        assert_eq!(
            check_document_resource_limits(&doc, &limits),
            Err(ResourceLimitError::TrackCount {
                observed: 3,
                limit: 2
            })
        );
    }

    #[test]
    fn layer_count_boundary() {
        let limits = tiny_limits();
        let mut doc = empty_doc();
        doc.layers.allocate("a").unwrap();
        doc.layers.allocate("b").unwrap();
        assert!(check_document_resource_limits(&doc, &limits).is_ok());
        doc.layers.allocate("c").unwrap();
        assert_eq!(
            check_document_resource_limits(&doc, &limits),
            Err(ResourceLimitError::LayerCount {
                observed: 3,
                limit: 2
            })
        );
    }

    fn simple_clip(layer_id: crate::LayerId, asset: crate::AssetId) -> TrackItem {
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope {
                layer_id,
                effects: Vec::new(),
                transform: Transform2D::identity(),
                clipping_mask: ClippingMaskSettings::default(),
                blend: Default::default(),
                opacity: DocParam::const_f64(1.0),
            },
            start: motolii_core::RationalTime::ZERO,
            duration: motolii_core::RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Asset { asset },
        })
    }

    #[test]
    fn group_depth_boundary() {
        let limits = ResourceLimits {
            max_group_depth: 2,
            ..generous_limits()
        };
        let mut doc = empty_doc();
        let asset = doc.assets.allocate("a", "video/mp4", "h").unwrap();
        let track_id = doc.track_ids.allocate("t").unwrap();

        // depth=2(上限どおり): group1 > group2 > clip
        let inner_layer = doc.layers.allocate("inner").unwrap();
        let inner_group_layer = doc.layers.allocate("inner-group").unwrap();
        let outer_group_layer = doc.layers.allocate("outer-group").unwrap();
        let inner_group = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(inner_group_layer),
            children: vec![simple_clip(inner_layer, asset)],
        });
        let outer_group = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(outer_group_layer),
            children: vec![inner_group],
        });
        doc.tracks.push(Track {
            id: track_id,
            items: vec![outer_group],
        });
        assert!(check_document_resource_limits(&doc, &limits).is_ok());

        // depth=3: 上限超過
        let mut doc2 = empty_doc();
        let asset2 = doc2.assets.allocate("a", "video/mp4", "h").unwrap();
        let track_id2 = doc2.track_ids.allocate("t").unwrap();
        let l1 = doc2.layers.allocate("l1").unwrap();
        let g1 = doc2.layers.allocate("g1").unwrap();
        let g2 = doc2.layers.allocate("g2").unwrap();
        let g3 = doc2.layers.allocate("g3").unwrap();
        let group1 = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(g1),
            children: vec![simple_clip(l1, asset2)],
        });
        let group2 = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(g2),
            children: vec![group1],
        });
        let group3 = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(g3),
            children: vec![group2],
        });
        doc2.tracks.push(Track {
            id: track_id2,
            items: vec![group3],
        });
        let err = check_document_resource_limits(&doc2, &limits).unwrap_err();
        assert!(matches!(
            err,
            ResourceLimitError::GroupDepth {
                observed: 3,
                limit: 2,
                ..
            }
        ));
    }

    #[test]
    fn keys_per_track_boundary() {
        let limits = ResourceLimits {
            max_keys_per_track: 2,
            ..generous_limits()
        };
        let mut doc = empty_doc();
        let track_id = doc.track_ids.allocate("t").unwrap();
        let layer_id = doc.layers.allocate("l").unwrap();
        let asset = doc.assets.allocate("a", "video/mp4", "h").unwrap();

        let mut keys = crate::DocKeyframeTrack::new();
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(1),
            t: motolii_core::RationalTime::ZERO,
            value: crate::DocValue::F64(0.0),
            interp: motolii_eval::Interp::Linear,
        });
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(2),
            t: motolii_core::RationalTime::try_new(1, 1).unwrap(),
            value: crate::DocValue::F64(1.0),
            interp: motolii_eval::Interp::Linear,
        });

        let mut clip = simple_clip(layer_id, asset);
        if let TrackItem::Clip(c) = &mut clip {
            c.envelope.opacity = DocParam::Keyframes(keys.clone());
        }
        doc.tracks.push(Track {
            id: track_id,
            items: vec![clip],
        });
        assert!(check_document_resource_limits(&doc, &limits).is_ok());

        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(3),
            t: motolii_core::RationalTime::try_new(2, 1).unwrap(),
            value: crate::DocValue::F64(2.0),
            interp: motolii_eval::Interp::Linear,
        });
        let mut doc2 = empty_doc();
        let track_id2 = doc2.track_ids.allocate("t").unwrap();
        let layer_id2 = doc2.layers.allocate("l").unwrap();
        let asset2 = doc2.assets.allocate("a", "video/mp4", "h").unwrap();
        let mut clip2 = simple_clip(layer_id2, asset2);
        if let TrackItem::Clip(c) = &mut clip2 {
            c.envelope.opacity = DocParam::Keyframes(keys);
        }
        doc2.tracks.push(Track {
            id: track_id2,
            items: vec![clip2],
        });
        let err = check_document_resource_limits(&doc2, &limits).unwrap_err();
        assert!(matches!(
            err,
            ResourceLimitError::KeyCount {
                observed: 3,
                limit: 2,
                ..
            }
        ));
    }

    #[test]
    fn string_bytes_boundary_on_asset_name() {
        let limits = ResourceLimits {
            max_string_bytes: 8,
            ..generous_limits()
        };
        let mut doc = empty_doc();
        // asset_type/content_hashは`max_string_bytes`未満に保ち、nameだけを境界へ寄せる
        doc.assets.allocate("12345678", "video", "h").unwrap();
        assert!(check_document_resource_limits(&doc, &limits).is_ok());

        let mut doc2 = empty_doc();
        doc2.assets.allocate("123456789", "video", "h").unwrap();
        let err = check_document_resource_limits(&doc2, &limits).unwrap_err();
        assert!(matches!(
            err,
            ResourceLimitError::StringBytes {
                observed: 9,
                limit: 8,
                ..
            }
        ));
    }

    #[test]
    fn extra_bytes_boundary_on_document_extra() {
        let limits = tiny_limits();
        let mut doc = empty_doc();
        // {"k":1} は7 bytes
        doc.extra
            .insert("k".to_string(), JsonValue::Number(1.into()));
        assert!(check_document_resource_limits(&doc, &limits).is_ok());

        let mut doc2 = empty_doc();
        doc2.extra
            .insert("key".to_string(), JsonValue::Number(1234.into()));
        let err = check_document_resource_limits(&doc2, &limits).unwrap_err();
        assert!(matches!(
            err,
            ResourceLimitError::ExtraBytes { limit: 8, .. }
        ));
    }

    #[test]
    fn production_defaults_are_generous_for_typical_projects() {
        let limits = ResourceLimits::production();
        assert!(limits.max_tracks >= 40);
        assert!(limits.max_file_bytes >= 10 * 1024 * 1024);
    }
}
