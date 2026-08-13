use std::collections::BTreeMap;
use std::path::PathBuf;

use motolii_cli::{apply_document, dump_document};
use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, Command as DocCommand, DocParam, Document, ItemEnvelope, LayerId,
    ParentLocator, ProjectSession, ResourceLimits, SaveOptions, StandardShape, Track, TrackId,
    TrackItem, VectorContent, VectorRecipe,
};
use motolii_testkit::tmp_dir;

#[test]
fn apply_add_track_item_vector_ellipse_shows_in_dump() {
    let (path, track, layer) = saved_one_track("cli-apply-place-ellipse");
    let command = DocCommand::AddTrackItem {
        parent: ParentLocator::Track(track),
        index: 0,
        item: TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::StandardShape {
                        shape: StandardShape::Ellipse {
                            width: DocParam::const_f64(0.2),
                            height: DocParam::const_f64(0.2),
                        },
                    },
                    modifiers: Vec::new(),
                },
            },
        }),
        layer_names: BTreeMap::from([(layer, "Ellipse".to_owned())]),
    };
    apply_document(&path, &serde_json::to_string(&command).unwrap(), None).unwrap();
    let dumped = dump_document(&path).unwrap();
    assert!(dumped.contains("Ellipse"), "{dumped}");
    assert!(dumped.contains("\"source\": \"vector\""), "{dumped}");
    assert!(dumped.contains("\"shape\": \"ellipse\""), "{dumped}");
}

fn saved_one_track(tag: &str) -> (PathBuf, TrackId, LayerId) {
    let mut doc = Document::new_current();
    // 空DocumentにはTrackが無くAddTrackItemのparentが成立しない。
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: Vec::new(),
    });
    let layer = LayerId::from_raw(doc.layers.peek_next());
    let path = tmp_dir(tag).join("document.json");
    {
        let mut session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
        session
            .save_document(&doc, &SaveOptions::default())
            .unwrap();
    }
    (path, track, layer)
}
