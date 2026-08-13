use std::path::{Path, PathBuf};

use motolii_cli::{apply_document, dump_document};
use motolii_core::RationalTime;
use motolii_doc::param_eval::{eval_vec2, ResolvedLayerParams};
use motolii_doc::{
    load_document, prepare_add_transform_param_key, AddTransformParamKeyPreparation,
    AddTransformParamKeyPrepareError, Clip, ClipSource, Command as DocCommand, DocParam, Document,
    ItemEnvelope, LayerId, ProjectSession, ResourceLimits, SaveOptions, ScalarPropertyId, Track,
    TrackItem,
};
use motolii_eval::DataTracks;
use motolii_testkit::tmp_dir;

#[test]
fn apply_scale_key_dump_contains_keyframes_eval_equals_const() {
    let (path, layer) = saved_one_clip("cli-apply-scale-key");
    let before = dumped_doc(&path);
    assert_eq!(
        first_clip(&before).envelope.transform.scale,
        DocParam::const_vec2([1.0, 1.0])
    );

    let t = RationalTime::from_seconds(1);
    let command = prepared_scale_key(&path, layer, t);
    apply_cmd(&path, &command);

    let dumped = dump_document(&path).unwrap();
    assert!(dumped.contains("keyframes"), "{dumped}");
    let after = dumped_doc(&path);
    let scale = &first_clip(&after).envelope.transform.scale;
    let got = eval_vec2(
        scale,
        t,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap();
    assert_eq!(got, [1.0, 1.0], "{dumped}");
}

#[test]
fn prepare_position_via_transform_param_key_is_unsupported() {
    let (path, layer) = saved_one_clip("cli-apply-scale-key-position");
    let doc = load_document(&path).unwrap();
    let err = prepare_add_transform_param_key(
        &doc,
        layer,
        ScalarPropertyId::Position,
        RationalTime::from_seconds(1),
    )
    .expect_err("Position must be PropertyUnsupported");
    assert!(matches!(
        err,
        AddTransformParamKeyPrepareError::PropertyUnsupported { .. }
    ));

    let dumped = dump_document(&path).unwrap();
    assert!(!dumped.contains("keyframes"), "{dumped}");
    assert_eq!(
        first_clip(&dumped_doc(&path)).envelope.transform.scale,
        DocParam::const_vec2([1.0, 1.0])
    );
}

fn prepared_scale_key(path: &Path, layer: LayerId, t: RationalTime) -> DocCommand {
    let doc = load_document(path).unwrap();
    match prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, t).unwrap() {
        AddTransformParamKeyPreparation::Prepared { command, .. } => command,
        AddTransformParamKeyPreparation::AlreadyPresent { .. } => {
            panic!("const scale must prepare a key")
        }
    }
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
