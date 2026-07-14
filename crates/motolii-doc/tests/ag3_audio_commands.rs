//! AG-3: audio component commandと分離macroの受け入れテスト。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    build_import_clip_source, plan_detach_audio, AudioComponent, Clip, ClipSource, Command,
    DocParam, Document, DocumentWriter, ImportAvMode, ItemEnvelope, ParentLocator, Track,
    TrackItem, VideoComponent, MIN_READER_VERSION_FOR_ASSET_COMPONENTS,
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
                ordinal: 0,
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
                ordinal: 0,
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
fn detach_then_undo_restores_single_enabled_av_clip() {
    let mut f = fixture();
    let new_layer = f.doc.layers.reserve().unwrap();
    let commands =
        plan_detach_audio(&f.doc, ParentLocator::Track(f.track), 0, new_layer, "Audio").unwrap();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    for command in commands {
        writer.apply_command(gesture, command).unwrap();
    }
    let snapshot = writer.snapshot();
    assert_eq!(snapshot.tracks[0].items.len(), 2);
    let TrackItem::Clip(detached) = &snapshot.tracks[0].items[1] else {
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
    assert!(audio_components(&snapshot)[0].enabled);
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
