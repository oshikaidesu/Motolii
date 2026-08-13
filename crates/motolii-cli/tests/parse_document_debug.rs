use std::path::PathBuf;

use motolii_cli::{apply_document, dump_document, parse_args, Command};
use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, Command as DocCommand, DocParam, Document, ItemEnvelope, LayerId,
    ProjectSession, ResourceLimits, SaveOptions, ScalarPropertyId, Track, TrackItem,
};
use motolii_testkit::tmp_dir;

#[test]
fn parses_dump_arguments() {
    let command = parse_args(["dump", "--document", "document.json"]).unwrap();
    let Command::Dump(args) = command else {
        panic!("expected dump command");
    };
    assert_eq!(args.document, PathBuf::from("document.json"));
}

#[test]
fn rejects_invalid_dump_arguments() {
    assert!(parse_args(["dump"]).is_err());
    assert!(parse_args(["dump", "--output", "out.json"]).is_err());
}

#[test]
fn dump_document_contains_version() {
    let dir = tmp_dir("cli-dump");
    let path = dir.join("document.json");
    let limits = ResourceLimits::production();
    {
        let mut session = ProjectSession::acquire(&path, &limits).unwrap();
        session
            .save_document(&Document::new_current(), &SaveOptions::default())
            .unwrap();
    }
    let json = dump_document(&path).unwrap();
    assert!(json.contains("\"version\""), "{json}");
}

#[test]
fn parses_apply_arguments() {
    let command = parse_args([
        "apply",
        "--document",
        "document.json",
        "--command",
        "[]",
        "--out",
        "out.json",
    ])
    .unwrap();
    let Command::Apply(args) = command else {
        panic!("expected apply command");
    };
    assert_eq!(args.document, PathBuf::from("document.json"));
    assert_eq!(args.command_json, "[]");
    assert_eq!(args.out, Some(PathBuf::from("out.json")));
}

#[test]
fn parses_apply_without_out() {
    let command = parse_args(["apply", "--document", "document.json", "--command", "{}"]).unwrap();
    let Command::Apply(args) = command else {
        panic!("expected apply command");
    };
    assert_eq!(args.out, None);
}

#[test]
fn rejects_invalid_apply_arguments() {
    assert!(parse_args(["apply", "--document", "document.json"]).is_err());
    assert!(parse_args(["apply", "--command", "[]"]).is_err());
}

#[test]
fn apply_empty_macro_fails() {
    let path = saved_current_document("cli-apply-empty");
    assert!(apply_document(&path, "[]", None).is_err());
}

#[test]
fn apply_garbage_json_fails() {
    let path = saved_current_document("cli-apply-garbage");
    assert!(apply_document(&path, "not-json", None).is_err());
}

#[test]
fn apply_set_property_opacity_shows_in_dump() {
    let (path, layer) = saved_one_clip("cli-apply-opacity");
    let command = DocCommand::SetProperty {
        target: layer,
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(1.0),
        new_value: DocParam::const_f64(0.37),
    };
    apply_document(&path, &serde_json::to_string(&command).unwrap(), None).unwrap();
    let dumped = dump_document(&path).unwrap();
    assert!(dumped.contains("0.37"), "{dumped}");
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

fn saved_current_document(tag: &str) -> PathBuf {
    let path = tmp_dir(tag).join("document.json");
    let mut session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
    session
        .save_document(&Document::new_current(), &SaveOptions::default())
        .unwrap();
    path
}
