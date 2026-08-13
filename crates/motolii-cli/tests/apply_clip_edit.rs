use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use motolii_cli::{apply_document, dump_document};
use motolii_core::RationalTime;
use motolii_doc::{
    load_document, AddPositionKeyPreparation, Clip, ClipSource, Command as DocCommand,
    CommandError, Document, DocumentWriter, ItemEnvelope, LayerId, ProjectSession, ResourceLimits,
    SaveOptions, Track, TrackItem,
};
use motolii_plugin::reference::reference_catalog;
use motolii_testkit::tmp_dir;

#[test]
fn apply_trim_clip_in() {
    let (path, layer) = saved_one_clip("cli-apply-trim-in");
    let writer = writer(load_document(&path).unwrap());
    let command = writer
        .prepare_trim_clip_in(layer, RationalTime::from_seconds(1))
        .unwrap()
        .expect("trim in must change");
    apply_cmd(&path, &command);
    let after = dumped_doc(&path);
    let clip = first_clip(&after);
    assert_eq!(clip.start, RationalTime::from_seconds(1));
    assert_eq!(clip.duration, RationalTime::from_seconds(4));
}

#[test]
fn apply_trim_clip_out() {
    let (path, layer) = saved_one_clip("cli-apply-trim-out");
    let writer = writer(load_document(&path).unwrap());
    let command = writer
        .prepare_trim_clip_out(layer, RationalTime::from_seconds(3))
        .unwrap()
        .expect("trim out must change");
    apply_cmd(&path, &command);
    let after = dumped_doc(&path);
    let clip = first_clip(&after);
    assert_eq!(clip.start, RationalTime::ZERO);
    assert_eq!(clip.duration, RationalTime::from_seconds(3));
}

#[test]
fn apply_set_clip_start() {
    let (path, layer) = saved_one_clip("cli-apply-move-start");
    let writer = writer(load_document(&path).unwrap());
    let command = writer
        .prepare_set_clip_start(layer, RationalTime::from_seconds(1))
        .unwrap()
        .expect("move start must change");
    apply_cmd(&path, &command);
    let after = dumped_doc(&path);
    let clip = first_clip(&after);
    assert_eq!(clip.start, RationalTime::from_seconds(1));
    assert_eq!(clip.duration, RationalTime::from_seconds(5));
}

#[test]
fn apply_split_clip_interior() {
    let (path, layer) = saved_one_clip("cli-apply-split");
    let mut writer = writer(load_document(&path).unwrap());
    let command = writer
        .prepare_split_clip(layer, RationalTime::from_seconds(2))
        .unwrap()
        .expect("interior split must change");
    apply_cmd(&path, &command);
    let after = dumped_doc(&path);
    assert_eq!(after.tracks[0].items.len(), 2);
    let left = first_clip(&after);
    assert_eq!(left.start, RationalTime::ZERO);
    assert_eq!(left.duration, RationalTime::from_seconds(2));
    let TrackItem::Clip(right) = &after.tracks[0].items[1] else {
        panic!("expected right clip");
    };
    assert_eq!(right.start, RationalTime::from_seconds(2));
    assert_eq!(right.duration, RationalTime::from_seconds(3));
}

#[test]
fn apply_add_position_key_dump_contains_keyframes() {
    let (path, layer) = saved_one_clip("cli-apply-position-key");
    let writer = writer(load_document(&path).unwrap());
    let command = match writer
        .prepare_add_position_key(layer, RationalTime::from_seconds(2))
        .unwrap()
    {
        AddPositionKeyPreparation::Prepared { command, .. } => command,
        AddPositionKeyPreparation::AlreadyPresent { .. } => {
            panic!("fresh clip has no position key")
        }
    };
    apply_cmd(&path, &command);
    let dumped = dump_document(&path).unwrap();
    assert!(dumped.contains("keyframes"), "{dumped}");
}

#[test]
fn split_not_interior_fails() {
    let (path, layer) = saved_one_clip("cli-apply-split-edge");
    let mut writer = writer(load_document(&path).unwrap());
    let start = writer
        .prepare_split_clip(layer, RationalTime::ZERO)
        .unwrap_err();
    assert!(matches!(start, CommandError::SplitNotInterior { .. }));
    let end = writer
        .prepare_split_clip(layer, RationalTime::from_seconds(5))
        .unwrap_err();
    assert!(matches!(end, CommandError::SplitNotInterior { .. }));
}

fn writer(doc: Document) -> DocumentWriter {
    DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap()
}

fn apply_cmd(path: &Path, command: &DocCommand) {
    apply_document(path, &serde_json::to_string(command).unwrap(), None).unwrap();
}

fn dumped_doc(path: &Path) -> Document {
    serde_json::from_str(&dump_document(path).unwrap()).unwrap()
}

fn first_clip(doc: &Document) -> &Clip {
    match &doc.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip,
        _ => panic!("expected clip"),
    }
}

fn saved_one_clip(tag: &str) -> (PathBuf, LayerId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("a").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    let path = tmp_dir(tag).join("document.json");
    {
        let mut session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
        session
            .save_document(&doc, &SaveOptions::default())
            .unwrap();
    }
    (path, layer)
}
