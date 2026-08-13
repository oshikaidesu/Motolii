use std::collections::BTreeMap;
use std::path::PathBuf;

use motolii_cli::{apply_document, dump_document};
use motolii_core::RationalTime;
use motolii_doc::{
    prepare_plugin_recipe, Clip, ClipSource, Command, Document, ItemEnvelope, LayerId,
    ParentLocator, ProjectSession, ResourceLimits, SaveOptions, Track, TrackItem,
};
use motolii_plugin::PluginKind;
use motolii_plugins_firstparty::first_party_catalog;
use motolii_testkit::tmp_dir;

const RADIAL_REPEATER: &str = "core.layer_source.radial_repeater";

#[test]
fn apply_place_vism_dump_shows_plugin_id_and_default_params() {
    let catalog = first_party_catalog().unwrap();
    let contract = catalog
        .get(RADIAL_REPEATER)
        .expect("first-party radial_repeater");
    let recipe = prepare_plugin_recipe(
        RADIAL_REPEATER,
        PluginKind::LayerSource,
        contract.node.version,
        &BTreeMap::new(),
        &catalog,
    )
    .unwrap();

    let mut doc = Document::new_current();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: Vec::new(),
    });
    let layer_id = LayerId::from_raw(doc.layers.peek_next());
    let layer_name = if contract.node.display_name.trim().is_empty() {
        RADIAL_REPEATER.to_owned()
    } else {
        contract.node.display_name.to_owned()
    };
    let command = Command::AddTrackItem {
        parent: ParentLocator::Track(track),
        index: 0,
        item: TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer_id),
            start: RationalTime::ZERO,
            duration: doc.composition.duration,
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: recipe.plugin_id,
                effect_version: recipe.current_version,
                params: recipe.params,
                extra: Default::default(),
            },
        }),
        layer_names: BTreeMap::from([(layer_id, layer_name)]),
    };

    let path = saved_document_with_track("cli-apply-place-vism", doc);
    let before = dump_document(&path).unwrap();
    assert!(
        !before.contains(RADIAL_REPEATER),
        "pre-apply dump must not already contain the vism: {before}"
    );

    apply_document(&path, &serde_json::to_string(&command).unwrap(), None).unwrap();
    let dumped = dump_document(&path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&dumped).unwrap();
    let source = &json["tracks"][0]["items"][0]["source"];
    assert_eq!(
        source["plugin_id"].as_str(),
        Some(RADIAL_REPEATER),
        "{dumped}"
    );
    assert_eq!(source["params"]["count"]["const"]["F64"], 12.0, "{dumped}");
    // Color は天井ではない。radial_repeater 既定に含まれるなら dump に出る。
    assert_eq!(
        source["params"]["color"]["const"]["Color"],
        serde_json::json!([1.0, 1.0, 1.0, 1.0]),
        "{dumped}"
    );
}

fn saved_document_with_track(tag: &str, doc: Document) -> PathBuf {
    let path = tmp_dir(tag).join("document.json");
    let mut session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
    session
        .save_document(&doc, &SaveOptions::default())
        .unwrap();
    path
}
