//! N-MEDIA-PLACE(素材取り込み半分・doc側): probe済みplain値からの
//! `AssetDraft`純構築(IOなし)と、既存admission経路
//! (`prepare_admit_asset`→`apply_prepared_asset_admission`)での取り込みを審判する。

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use motolii_doc::{
    AssetDraft, AssetId, Document, DocumentWriter, SourceFingerprintDecode, SourceFingerprintV1,
};

fn fingerprint(bytes: &[u8]) -> SourceFingerprintV1 {
    SourceFingerprintV1::from_reader(Cursor::new(bytes)).unwrap()
}

#[test]
fn draft_from_probed_source_copies_canonical_fingerprint_and_paths() {
    let fp = fingerprint(b"abc");
    let draft = AssetDraft::from_probed_source(
        "video/mp4",
        &fp,
        Path::new("/media/proj/clips/intro.mp4"),
        Some(Path::new("/media/proj")),
    );
    assert_eq!(draft.name, "intro");
    assert_eq!(draft.asset_type, "video/mp4");
    // 正準content_hash(motolii-source-v1)をそのまま写す。
    assert_eq!(
        draft.content_hash,
        "motolii-source-v1:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        draft.path_absolute.as_deref(),
        Some("/media/proj/clips/intro.mp4")
    );
    assert_eq!(
        draft.path_project_relative.as_deref(),
        Some("clips/intro.mp4")
    );
    assert_eq!(draft.file_name.as_deref(), Some("intro.mp4"));
    assert_eq!(draft.size_bytes, Some(3));
    // head_hash/tail_hashはlegacy hint(2026-08-08 serial-core決定)。新規admissionでは発行しない。
    assert_eq!(draft.head_hash, None);
    assert_eq!(draft.tail_hash, None);
}

#[test]
fn draft_outside_project_root_has_no_relative_path() {
    let fp = fingerprint(b"abc");
    let draft = AssetDraft::from_probed_source(
        "audio/mp4",
        &fp,
        Path::new("/elsewhere/song.m4a"),
        Some(Path::new("/media/proj")),
    );
    assert_eq!(draft.name, "song");
    assert_eq!(draft.path_project_relative, None);

    let rootless =
        AssetDraft::from_probed_source("audio/mp4", &fp, Path::new("/elsewhere/song.m4a"), None);
    assert_eq!(rootless.path_project_relative, None);
    assert_eq!(
        rootless.path_absolute.as_deref(),
        Some("/elsewhere/song.m4a")
    );
}

#[test]
fn admitted_document_asset_keeps_strict_fingerprint_via_existing_route() {
    let fp = fingerprint(b"pretend media bytes");
    let draft = AssetDraft::from_probed_source(
        "video/mp4",
        &fp,
        Path::new("/media/proj/clips/intro.mp4"),
        Some(Path::new("/media/proj")),
    );

    let catalog = Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
    let mut writer = DocumentWriter::new(Document::new_current(), catalog).unwrap();
    let admit = writer.prepare_admit_asset(draft).unwrap();
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();

    let snapshot = writer.snapshot();
    let asset = snapshot.assets.get(AssetId::from_raw(0)).unwrap();
    assert_eq!(asset.name, "intro");
    assert_eq!(asset.asset_type, "video/mp4");
    assert_eq!(asset.file_name.as_deref(), Some("intro.mp4"));
    assert_eq!(
        asset.path_project_relative.as_deref(),
        Some("clips/intro.mp4")
    );
    assert_eq!(asset.size_bytes, Some(fp.size_bytes()));
    assert_eq!(asset.head_hash, None);
    assert_eq!(asset.tail_hash, None);
    // Documentに載ったcontent_hashはstrict V1としてdecodeでき、元のfingerprintと一致する。
    match SourceFingerprintV1::decode_persisted(&asset.content_hash, asset.size_bytes) {
        SourceFingerprintDecode::V1(decoded) => assert_eq!(decoded, fp),
        other => panic!("expected strict V1 fingerprint, got {other:?}"),
    }
}
