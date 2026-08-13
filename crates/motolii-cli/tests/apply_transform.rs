use std::path::{Path, PathBuf};

use motolii_cli::{apply_document, dump_document, CliError};
use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, Command, DocParam, Document, ItemEnvelope, LayerId, ProjectSession,
    ResourceLimits, SaveOptions, ScalarPropertyId, Track, TrackItem, Transform2D,
};
use motolii_testkit::tmp_dir;

#[test]
fn apply_transform2d_set_property_roundtrips_in_dump() {
    let (path, layer) = saved_one_clip("cli-apply-transform");
    let (_, before) = dump_doc(&path);
    let env = envelope(&before);
    assert_eq!(env.transform, Transform2D::identity());
    assert_eq!(env.opacity, DocParam::const_f64(1.0));

    let commands = [
        Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Position,
            old_value: DocParam::const_vec2([0.0, 0.0]),
            new_value: DocParam::const_vec2([0.25, -0.5]),
        },
        Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Scale,
            old_value: DocParam::const_vec2([1.0, 1.0]),
            new_value: DocParam::const_vec2([1.25, 0.75]),
        },
        Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Rotation,
            old_value: DocParam::const_f64(0.0),
            new_value: DocParam::const_f64(0.42),
        },
        Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.37),
        },
    ];
    apply_document(&path, &serde_json::to_string(&commands).unwrap(), None).unwrap();

    let (dumped, after) = dump_doc(&path);
    let env = envelope(&after);
    assert_eq!(
        env.transform.position,
        DocParam::const_vec2([0.25, -0.5]),
        "{dumped}"
    );
    assert_eq!(
        env.transform.scale,
        DocParam::const_vec2([1.25, 0.75]),
        "{dumped}"
    );
    assert_eq!(
        env.transform.rotation,
        DocParam::const_f64(0.42),
        "{dumped}"
    );
    assert_eq!(env.opacity, DocParam::const_f64(0.37), "{dumped}");
}

#[test]
fn apply_set_property_rejects_typed_mismatch() {
    let (path, layer) = saved_one_clip("cli-apply-transform-stale");
    // Position は Vec2。F64 を書くと prepare_plugins/validate が型不一致で戻す。
    let command = Command::SetProperty {
        target: layer,
        property: ScalarPropertyId::Position,
        old_value: DocParam::const_vec2([0.0, 0.0]),
        new_value: DocParam::const_f64(0.5),
    };
    let err = apply_document(&path, &serde_json::to_string(&command).unwrap(), None)
        .expect_err("typed mismatch must reject");
    let msg = match err {
        CliError::Usage(msg) => msg,
        other => panic!("typed reject, not silent: {other:?}"),
    };
    assert!(msg.contains("type mismatch"), "typed reject: {msg}");

    let (dumped, after) = dump_doc(&path);
    assert_eq!(
        envelope(&after).transform.position,
        DocParam::const_vec2([0.0, 0.0]),
        "{dumped}"
    );
}

fn dump_doc(path: &Path) -> (String, Document) {
    let dumped = dump_document(path).unwrap();
    let doc = serde_json::from_str(&dumped).unwrap_or_else(|e| panic!("{e}\n{dumped}"));
    (dumped, doc)
}

fn envelope(doc: &Document) -> &ItemEnvelope {
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    &clip.envelope
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
