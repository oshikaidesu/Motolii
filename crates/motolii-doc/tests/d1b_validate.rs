//! D1b: Document::validate の正常系・破壊系。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    AssetId, Clip, ClipSource, DocParam, Document, DocumentError, DocumentWriter, EffectInstance,
    ItemEnvelope, LayerId, LookAtAxis, Soundtrack, Track, TrackId, TrackItem,
};
use serde_json::Map;
use std::collections::BTreeMap;

fn valid_minimal() -> Document {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("a").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Asset { asset },
            path_ops: Vec::new(),
        })],
    });
    doc
}

#[test]
fn valid_document_passes() {
    let doc = valid_minimal();
    assert!(doc.validate().is_ok());
    let writer = DocumentWriter::new(doc);
    assert!(writer.validate().is_ok());
}

#[test]
fn unknown_layer_id_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.layer_id = LayerId::from_raw(99);
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownLayerId { id: 99 })
    ));
}

#[test]
fn unknown_asset_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.source = ClipSource::Asset {
            asset: AssetId::from_raw(99),
        };
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownAssetId { id: 99 })
    ));
}

#[test]
fn unknown_track_id_fails() {
    let mut doc = valid_minimal();
    doc.tracks[0].id = TrackId::from_raw(99);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownTrackId { id: 99 })
    ));
}

#[test]
fn look_at_missing_target_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.transform.position = DocParam::LookAt {
            target: LayerId::from_raw(42),
            axis: LookAtAxis::PlusY,
        };
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownLayerId { id: 42 })
    ));
}

#[test]
fn clip_past_composition_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.duration = RationalTime::try_new(20, 1).unwrap();
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ClipPastComposition { .. })
    ));
}

#[test]
fn zero_clip_duration_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.duration = RationalTime::ZERO;
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::NonPositiveClipDuration { .. })
    ));
}

#[test]
fn duplicate_layer_in_tree_fails() {
    let mut doc = valid_minimal();
    let (layer_id, asset) = match &doc.tracks[0].items[0] {
        TrackItem::Clip(Clip {
            envelope,
            source: ClipSource::Asset { asset },
            ..
        }) => (envelope.layer_id, *asset),
        _ => panic!("expected clip"),
    };
    doc.tracks[0].items.push(TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(layer_id),
        start: RationalTime::try_new(5, 1).unwrap(),
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::Asset { asset },
        path_ops: Vec::new(),
    }));
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::DuplicateLayerId { .. })
    ));
}

#[test]
fn soundtrack_unknown_asset_fails() {
    let mut doc = valid_minimal();
    doc.soundtrack =
        Some(Soundtrack::try_new(AssetId::from_raw(7), RationalTime::ZERO, 1.0).unwrap());
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownAssetId { id: 7 })
    ));
}

#[test]
fn empty_effect_plugin_id_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.effects.push(EffectInstance {
            plugin_id: String::new(),
            effect_version: 1,
            enabled: true,
            params: BTreeMap::new(),
            extra: Map::new(),
        });
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::EmptyEffectPluginId { .. })
    ));
}

#[test]
fn version_below_min_reader_fails() {
    let mut doc = valid_minimal();
    doc.version = 1;
    doc.min_reader_version = 2;
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::VersionBelowMinReader {
            version: 1,
            min_reader_version: 2
        })
    ));
}

#[test]
fn version_2_requires_min_reader_at_least_2() {
    let mut doc = Document::new_v2();
    doc.min_reader_version = 1;
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::MinReaderTooLowForVersion {
            version: 2,
            min_reader_version: 1
        })
    ));
}

#[test]
fn v1_with_color_interpretation_fails_validate() {
    let mut doc = Document::new_v1();
    doc.color_interpretation = Some(motolii_doc::ColorInterpretation::StraightSrgb);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ColorInterpretationOnV1)
    ));
}

#[test]
fn validate_does_not_mutate_writer() {
    let mut writer = DocumentWriter::new(valid_minimal());
    let rev = writer.revision;
    writer.edit(|doc| {
        doc.tracks[0].id = TrackId::from_raw(99);
    });
    assert_eq!(writer.revision, rev + 1);
    let snap = writer.snapshot();
    assert!(writer.validate().is_err());
    // 検証失敗後もスナップショット内容は edit 結果のまま(検証が巻き戻さない)
    assert_eq!(snap.tracks[0].id, TrackId::from_raw(99));
    assert!(matches!(
        writer.validate(),
        Err(DocumentError::UnknownTrackId { id: 99 })
    ));
}

#[test]
fn invalid_time_map_speed_den_zero_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        // pubフィールド直書き — deserializeは拒否するがedit経路では壊せる
        clip.time_map = TimeMap {
            source_start: RationalTime::ZERO,
            timeline_start: RationalTime::ZERO,
            speed_num: 1,
            speed_den: 0,
        };
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::InvalidTimeMap { .. })
    ));
}

#[test]
fn invalid_time_map_non_positive_speed_num_fails() {
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.time_map.speed_num = 0;
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::InvalidTimeMap { .. })
    ));
}

#[test]
fn parent_self_cycle_fails() {
    let mut doc = valid_minimal();
    let layer = match &doc.tracks[0].items[0] {
        TrackItem::Clip(c) => c.envelope.layer_id,
        _ => panic!("expected clip"),
    };
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.envelope.transform.parent = Some(layer);
    }
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParentCycle { .. })
    ));
}

#[test]
fn parent_mutual_cycle_fails() {
    let mut doc = Document::new_v1();
    let a = doc.layers.allocate("a").unwrap();
    let b = doc.layers.allocate("b").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let mut env_a = ItemEnvelope::new(a);
    env_a.transform.parent = Some(b);
    let mut env_b = ItemEnvelope::new(b);
    env_b.transform.parent = Some(a);
    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: env_a,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(2, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::Asset { asset },
                path_ops: Vec::new(),
            }),
            TrackItem::Clip(Clip {
                envelope: env_b,
                start: RationalTime::try_new(2, 1).unwrap(),
                duration: RationalTime::try_new(2, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::Asset { asset },
                path_ops: Vec::new(),
            }),
        ],
    });
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParentCycle { .. })
    ));
}

#[test]
fn negative_clip_start_is_allowed() {
    // 設計判断: start下限は検査しない(トリムイン相当)。endがcomp内なら通る。
    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.start = RationalTime::try_new(-2, 1).unwrap();
        clip.duration = RationalTime::try_new(5, 1).unwrap(); // end = 3 ≤ 10
    }
    assert!(doc.validate().is_ok());
}
