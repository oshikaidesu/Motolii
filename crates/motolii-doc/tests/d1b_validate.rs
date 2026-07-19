#![allow(deprecated)]

//! D1b: Document::validate の正常系・破壊系。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    AssetId, Clip, ClipSource, DocParam, Document, DocumentError, DocumentWriter, EffectDefinition,
    EffectDefinitionDraft, EffectDefinitionId, EffectId, EffectUse, ItemEnvelope, LayerId,
    LookAtAxis, Soundtrack, Track, TrackId, TrackItem, MIN_READER_VERSION_FOR_COMP_CAMERA,
};
use motolii_plugin::reference::reference_catalog;
use serde_json::Map;
use std::collections::BTreeMap;
use std::sync::Arc;

fn reference_writer(doc: Document) -> DocumentWriter {
    DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap()
}

fn valid_minimal() -> Document {
    let mut doc = Document::new_current();
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
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc
}

#[test]
fn valid_document_passes() {
    let doc = valid_minimal();
    assert!(doc.validate().is_ok());
    let writer = reference_writer(doc);
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
        clip.source = ClipSource::asset_video_only(AssetId::from_raw(99));
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
        clip.envelope.transform.rotation = DocParam::LookAt {
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
            source: ClipSource::Asset { asset, .. },
            ..
        }) => (envelope.layer_id, *asset),
        _ => panic!("expected clip"),
    };
    doc.tracks[0].items.push(TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(layer_id),
        start: RationalTime::try_new(5, 1).unwrap(),
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(asset),
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
fn empty_effect_definition_plugin_id_fails() {
    let mut doc = valid_minimal();
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        String::new(),
        1,
        true,
        BTreeMap::new(),
        Map::new(),
    ));
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
        clip.envelope.effects.push(EffectUse {
            id: use_id,
            definition_id: def_id,
        });
    }
    doc.version = 5;
    doc.min_reader_version = 5;
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::EmptyEffectDefinitionPluginId { .. })
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
fn prepare_effect_keeps_current_version_and_writer_unchanged() {
    let mut doc = Document::new_current();
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
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().unwrap();

    let writer = reference_writer(doc);
    let snap_before = writer.snapshot();
    let revision_before = writer.revision;
    let undo_before = writer.undo_len();
    let redo_before = writer.redo_len();

    let draft = EffectDefinitionDraft {
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Map::new(),
    };
    let cmd = writer
        .prepare_create_effect(layer, 0, draft)
        .expect("prepare create on new_current must succeed");

    assert_eq!(writer.snapshot().version, snap_before.version);
    assert_eq!(
        writer.snapshot().min_reader_version,
        snap_before.min_reader_version
    );
    assert_eq!(
        writer.snapshot().next_stable_id.peek_next(),
        snap_before.next_stable_id.peek_next()
    );
    assert_eq!(writer.revision, revision_before);
    assert_eq!(writer.undo_len(), undo_before);
    assert_eq!(writer.redo_len(), redo_before);
    assert_eq!(snap_before.version, 5);
    assert_eq!(
        snap_before.min_reader_version,
        MIN_READER_VERSION_FOR_COMP_CAMERA
    );
    assert!(
        cmd.stable_id_reservation().is_some(),
        "prepare must return identity lifecycle command"
    );
    writer
        .validate()
        .expect("new_current document must validate");
}

#[test]
fn stable_id_document_is_open_mode_read_write() {
    use motolii_doc::{classify_open_mode, OpenMode, READER_VERSION, WRITER_VERSION};

    assert_eq!(READER_VERSION, 5);
    assert_eq!(WRITER_VERSION, 5);
    assert_eq!(
        classify_open_mode(2, 2),
        OpenMode::ReadWrite,
        "version=2 / min_reader_version=2 must be ReadWrite under D2 writer capability"
    );
}

#[test]
fn validate_does_not_mutate_writer() {
    let mut writer = reference_writer(valid_minimal());
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
        Err(motolii_doc::DocumentPluginError::Structural(
            DocumentError::UnknownTrackId { id: 99 }
        ))
    ));
}

#[test]
fn time_map_speed_invariant_is_constructor_gated() {
    // speed は非公開。不正・非正準値は try_new 以外で注入できない。
    assert!(TimeMap::try_new(RationalTime::ZERO, 1, 0, Default::default()).is_err());
    assert!(TimeMap::try_new(RationalTime::ZERO, 0, 1, Default::default()).is_err());
    let reduced = TimeMap::constant_speed(RationalTime::ZERO, 2, 2).unwrap();
    assert_eq!((reduced.speed_num(), reduced.speed_den()), (1, 1));
    assert_eq!(reduced, TimeMap::identity());

    let mut doc = valid_minimal();
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        clip.time_map = reduced;
    }
    assert!(doc.validate().is_ok());
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
    let mut doc = Document::new_current();
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
                source: ClipSource::asset_video_only(asset),
            }),
            TrackItem::Clip(Clip {
                envelope: env_b,
                start: RationalTime::try_new(2, 1).unwrap(),
                duration: RationalTime::try_new(2, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
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
