//! AG-3: audio component commandと分離macroの受け入れテスト。

use std::path::Path;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    build_import_clip_source, load_document, plan_detach_audio, save_document, AudioComponent, Clip,
    ClipSource, Command, CommandError, DocParam, Document, DocumentError, DocumentWriter,
    ImportAvMode, ItemEnvelope, ParentLocator, Track, TrackItem, VideoComponent,
    MIN_READER_VERSION_FOR_ASSET_COMPONENTS,
};

struct Fixture {
    doc: Document,
    layer: motolii_doc::LayerId,
    track: motolii_doc::TrackId,
}

fn fixture() -> Fixture {
    let mut doc = Document::new_v1();
    doc.version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
    doc.min_reader_version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
    let asset = doc
        .assets
        .allocate("clip", "video/mp4", "sha256:a")
        .unwrap();
    let layer = doc.layers.allocate("AV").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::try_new(2, 1).unwrap(),
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Asset {
                asset,
                video: Some(VideoComponent::ordinal(0)),
                audio: vec![AudioComponent::ordinal(0)],
            },
        })],
    });
    Fixture { doc, layer, track }
}

fn audio_components(doc: &Document) -> &[AudioComponent] {
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("fixture clip");
    };
    let ClipSource::Asset { audio, .. } = &clip.source else {
        panic!("fixture asset");
    };
    audio
}

#[test]
fn mute_and_gain_roundtrip_through_document_writer() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentEnabled {
                target: f.layer,
                index: 0,
                old: true,
                new: false,
            },
        )
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentGain {
                target: f.layer,
                index: 0,
                old: DocParam::const_f64(1.0),
                new: DocParam::const_f64(0.25),
            },
        )
        .unwrap();
    let snapshot = writer.snapshot();
    assert!(!audio_components(&snapshot)[0].enabled);
    assert_eq!(
        audio_components(&snapshot)[0].gain,
        DocParam::const_f64(0.25)
    );

    writer.undo().unwrap();
    let snapshot = writer.snapshot();
    assert!(audio_components(&snapshot)[0].enabled);
    assert_eq!(
        audio_components(&snapshot)[0].gain,
        DocParam::const_f64(1.0)
    );
}

#[test]
fn detach_to_other_track_then_undo_restores_single_enabled_av_clip() {
    let mut f = fixture();
    let audio_track = f.doc.track_ids.allocate("A1").unwrap();
    f.doc.tracks.push(Track {
        id: audio_track,
        items: vec![],
    });
    let new_layer = f.doc.layers.reserve().unwrap();
    let commands = plan_detach_audio(
        &f.doc,
        ParentLocator::Track(f.track),
        0,
        ParentLocator::Track(audio_track),
        0,
        new_layer,
        "Audio",
    )
    .unwrap();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    for command in commands {
        writer.apply_command(gesture, command).unwrap();
    }
    let snapshot = writer.snapshot();
    assert_eq!(snapshot.tracks[0].items.len(), 1);
    assert_eq!(snapshot.tracks[1].items.len(), 1);
    let TrackItem::Clip(detached) = &snapshot.tracks[1].items[0] else {
        panic!("detached clip");
    };
    assert!(matches!(
        detached.source,
        ClipSource::Asset { video: None, .. }
    ));
    assert!(!audio_components(&snapshot)[0].enabled);

    writer.undo().unwrap();
    let snapshot = writer.snapshot();
    assert_eq!(snapshot.tracks[0].items.len(), 1);
    assert_eq!(snapshot.tracks[1].items.len(), 0);
    assert!(audio_components(&snapshot)[0].enabled);
}

#[test]
fn detach_same_lane_is_rejected() {
    let mut f = fixture();
    let new_layer = f.doc.layers.reserve().unwrap();
    let err = plan_detach_audio(
        &f.doc,
        ParentLocator::Track(f.track),
        0,
        ParentLocator::Track(f.track),
        1,
        new_layer,
        "Audio",
    )
    .unwrap_err();
    assert!(matches!(err, CommandError::DetachSameLane));
}

#[test]
fn duplicate_audio_ordinal_is_typed_validate_error() {
    let mut f = fixture();
    {
        let TrackItem::Clip(clip) = &mut f.doc.tracks[0].items[0] else {
            panic!("fixture clip");
        };
        let ClipSource::Asset { audio, .. } = &mut clip.source else {
            panic!("fixture asset");
        };
        audio.push(AudioComponent::ordinal(0));
    }
    let err = f.doc.validate().unwrap_err();
    assert!(matches!(
        err,
        DocumentError::DuplicateAudioStreamOrdinal {
            ordinal: 0,
            first_index: 0,
            second_index: 1,
            ..
        }
    ));
}

#[test]
fn mute_gain_by_index_targets_second_component() {
    let mut f = fixture();
    {
        let TrackItem::Clip(clip) = &mut f.doc.tracks[0].items[0] else {
            panic!("fixture clip");
        };
        let ClipSource::Asset { audio, .. } = &mut clip.source else {
            panic!("fixture asset");
        };
        audio.push(AudioComponent::ordinal(1));
    }
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentEnabled {
                target: f.layer,
                index: 1,
                old: true,
                new: false,
            },
        )
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentGain {
                target: f.layer,
                index: 1,
                old: DocParam::const_f64(1.0),
                new: DocParam::const_f64(0.5),
            },
        )
        .unwrap();
    let snapshot = writer.snapshot();
    let audio = audio_components(&snapshot);
    assert!(audio[0].enabled);
    assert_eq!(audio[0].gain, DocParam::const_f64(1.0));
    assert!(!audio[1].enabled);
    assert_eq!(audio[1].gain, DocParam::const_f64(0.5));
}

#[test]
fn mute_and_gain_survive_save_reload() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentEnabled {
                target: f.layer,
                index: 0,
                old: true,
                new: false,
            },
        )
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::SetAudioComponentGain {
                target: f.layer,
                index: 0,
                old: DocParam::const_f64(1.0),
                new: DocParam::const_f64(0.3),
            },
        )
        .unwrap();
    let before = writer.snapshot();

    let dir = std::env::temp_dir().join(format!(
        "motolii-ag3-persist-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("doc.json");
    save_document(Path::new(&path), &before).unwrap();
    let after = load_document(Path::new(&path)).unwrap();
    assert!(!audio_components(&after)[0].enabled);
    assert_eq!(
        audio_components(&after)[0].gain,
        DocParam::const_f64(0.3)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_source_builder_respects_component_selection() {
    let asset = motolii_doc::AssetId::from_raw(9);
    assert!(matches!(
        build_import_clip_source(asset, ImportAvMode::VideoOnly),
        ClipSource::Asset {
            video: Some(VideoComponent { .. }),
            audio,
            ..
        } if audio.is_empty()
    ));
    assert!(matches!(
        build_import_clip_source(
            asset,
            ImportAvMode::VideoAndAudio {
                video_ordinal: 0,
                audio_ordinal: 2,
            }
        ),
        ClipSource::Asset {
            video: Some(VideoComponent { stream }),
            audio,
            ..
        } if stream.ordinal == 0 && audio == vec![AudioComponent::ordinal(2)]
    ));
}
