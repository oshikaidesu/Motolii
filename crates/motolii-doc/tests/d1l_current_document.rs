#![allow(deprecated)]

//! D1l: 製品の新規Document生成境界。

use motolii_doc::{
    load_document_bytes_with_limits, Document, OpenMode, ResourceLimits,
    MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS, READER_VERSION, WRITER_VERSION,
};

#[test]
fn current_versions_are_one_contract() {
    assert_eq!(READER_VERSION, 4);
    assert_eq!(WRITER_VERSION, 4);
    assert_eq!(MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS, 4);
}

#[test]
fn new_current_roundtrips_as_read_write_without_version_changes() {
    let doc = Document::new_current();
    assert_eq!(doc.version, WRITER_VERSION);
    assert_eq!(
        doc.min_reader_version,
        MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS
    );

    let bytes = serde_json::to_vec(&doc).unwrap();
    let opened = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap();
    assert_eq!(opened.open_mode, OpenMode::ReadWrite);
    assert_eq!(opened.document, doc);
}

#[test]
fn new_v1_remains_a_legacy_v1_fixture() {
    let doc = Document::new_v1();
    assert_eq!(doc.version, 1);
    assert_eq!(doc.min_reader_version, 1);
    assert_eq!(doc.composition, Document::new_current().composition);
    assert_eq!(doc.bpm, Document::new_current().bpm);
    assert_eq!(doc.assets, Document::new_current().assets);
    assert_eq!(doc.layers, Document::new_current().layers);
    assert_eq!(doc.track_ids, Document::new_current().track_ids);
    assert_eq!(doc.tracks, Document::new_current().tracks);
    assert_eq!(doc.next_stable_id, Document::new_current().next_stable_id);
    assert_eq!(
        doc.effect_definitions,
        Document::new_current().effect_definitions
    );
    assert_eq!(doc.extra, Document::new_current().extra);
}
