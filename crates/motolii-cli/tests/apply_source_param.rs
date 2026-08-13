use std::collections::BTreeMap;
use std::path::PathBuf;

use motolii_cli::{apply_document, dump_document};
use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, Command as DocCommand, DocParam, Document, ItemEnvelope, LayerId,
    ProjectSession, ResourceLimits, SaveOptions, ScalarPropertyId, Track, TrackItem,
};
use motolii_testkit::tmp_dir;

const COUNT_KEY: &str = "count";
const COLOR_KEY: &str = "color";
const OLD_COUNT: f64 = 12.0;
const NEW_COUNT: f64 = 8.0;
const OLD_COLOR: [f64; 4] = [1.0, 1.0, 1.0, 1.0];
const NEW_COLOR: [f64; 4] = [0.2, 0.4, 0.6, 1.0];

#[test]
fn apply_source_param_writes_f64_and_color_into_dump() {
    let (path, layer) = saved_plugin_clip("cli-apply-source-param");

    apply_set(
        path.as_path(),
        layer,
        COUNT_KEY,
        DocParam::const_f64(OLD_COUNT),
        DocParam::const_f64(NEW_COUNT),
    );
    let after_f64 = dump_params(&path);
    assert_eq!(
        after_f64.get(COUNT_KEY),
        Some(&DocParam::const_f64(NEW_COUNT))
    );
    assert_eq!(
        after_f64.get(COLOR_KEY),
        Some(&DocParam::const_color(OLD_COLOR))
    );

    apply_set(
        path.as_path(),
        layer,
        COLOR_KEY,
        DocParam::const_color(OLD_COLOR),
        DocParam::const_color(NEW_COLOR),
    );
    let dumped = dump_document(&path).unwrap();
    assert!(
        dumped.contains("\"Color\""),
        "Color DocParam must remain in dump, not coerced to f64: {dumped}"
    );
    let after_color = plugin_params(&dumped);
    assert_eq!(
        after_color.get(COLOR_KEY),
        Some(&DocParam::const_color(NEW_COLOR))
    );
    assert_eq!(
        after_color.get(COUNT_KEY),
        Some(&DocParam::const_f64(NEW_COUNT))
    );
}

#[test]
fn apply_source_param_unknown_key_fails() {
    let (path, layer) = saved_plugin_clip("cli-apply-source-param-missing");
    let before = dump_document(&path).unwrap();
    let err = apply_document(
        &path,
        &set_source_param(
            layer,
            "missing",
            DocParam::const_f64(0.0),
            DocParam::const_f64(1.0),
        ),
        None,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("source param `missing` not found"),
        "{err}"
    );
    assert_eq!(dump_document(&path).unwrap(), before);
}

#[test]
fn apply_source_param_non_plugin_fails() {
    let (path, layer) = saved_asset_clip("cli-apply-source-param-asset");
    let before = dump_document(&path).unwrap();
    let err = apply_document(
        &path,
        &set_source_param(
            layer,
            COUNT_KEY,
            DocParam::const_f64(OLD_COUNT),
            DocParam::const_f64(NEW_COUNT),
        ),
        None,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("source is not ClipSource::Plugin"),
        "{err}"
    );
    assert_eq!(dump_document(&path).unwrap(), before);
}

fn apply_set(path: &std::path::Path, layer: LayerId, key: &str, old: DocParam, new: DocParam) {
    apply_document(path, &set_source_param(layer, key, old, new), None).unwrap();
}

fn set_source_param(layer: LayerId, key: &str, old: DocParam, new: DocParam) -> String {
    serde_json::to_string(&DocCommand::SetProperty {
        target: layer,
        property: ScalarPropertyId::SourceParam(key.into()),
        old_value: old,
        new_value: new,
    })
    .unwrap()
}

fn dump_params(path: &std::path::Path) -> BTreeMap<String, DocParam> {
    plugin_params(&dump_document(path).unwrap())
}

fn plugin_params(dumped: &str) -> BTreeMap<String, DocParam> {
    let doc: Document = serde_json::from_str(dumped).expect("dump is Document JSON");
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("expected clip in dump");
    };
    let ClipSource::Plugin { params, .. } = &clip.source else {
        panic!("expected Plugin source in dump");
    };
    params.clone()
}

fn saved_plugin_clip(tag: &str) -> (PathBuf, LayerId) {
    saved_clip(
        tag,
        ClipSource::Plugin {
            plugin_id: "core.layer_source.radial_repeater".into(),
            effect_version: 1,
            params: BTreeMap::from([
                (COUNT_KEY.into(), DocParam::const_f64(OLD_COUNT)),
                (COLOR_KEY.into(), DocParam::const_color(OLD_COLOR)),
            ]),
            extra: Default::default(),
        },
    )
}

fn saved_asset_clip(tag: &str) -> (PathBuf, LayerId) {
    let mut doc = Document::new_current();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    saved_clip_in(tag, doc, ClipSource::asset_video_only(asset))
}

fn saved_clip(tag: &str, source: ClipSource) -> (PathBuf, LayerId) {
    saved_clip_in(tag, Document::new_current(), source)
}

fn saved_clip_in(tag: &str, mut doc: Document, source: ClipSource) -> (PathBuf, LayerId) {
    let layer = doc.layers.allocate("a").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source,
        })],
    });
    let path = tmp_dir(tag).join("document.json");
    let mut session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
    session
        .save_document(&doc, &SaveOptions::default())
        .unwrap();
    (path, layer)
}
