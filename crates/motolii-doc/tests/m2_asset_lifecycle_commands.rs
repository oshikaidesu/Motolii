use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::RationalTime;
use motolii_doc::{
    AssetDraft, AssetId, Clip, ClipSource, CommandError, Document, DocumentWriter, ItemEnvelope,
    JournalEdit, ProjectSession, ResourceLimits, SaveProjectOptions, Track, TrackItem,
};

fn new_writer(document: Document) -> DocumentWriter {
    let catalog = Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
    DocumentWriter::new(document, catalog).unwrap()
}

fn draft(name: &str) -> AssetDraft {
    AssetDraft {
        name: name.into(),
        asset_type: "video/mp4".into(),
        content_hash:
            "motolii-source-v1:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                .into(),
        path_absolute: Some(r"C:\media\clip.mp4".into()),
        path_project_relative: Some(r"media\clip.mp4".into()),
        file_name: Some("clip.mp4".into()),
        size_bytes: Some(3),
        head_hash: None,
        tail_hash: None,
    }
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-asset-command-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn prepare_admit_and_remove_are_atomic_undoable_and_keep_the_same_id() {
    let mut writer = new_writer(Document::new_current());
    let before = writer.snapshot();
    let admit = writer.prepare_admit_asset(draft("clip")).unwrap();
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());

    let asset = admit.asset().clone();
    assert_eq!(asset.id, AssetId::from_raw(0));
    assert_eq!(asset.path_absolute.as_deref(), Some("C:/media/clip.mp4"));

    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();
    assert_eq!(writer.snapshot().assets.get(asset.id), Some(&asset));
    assert_eq!(writer.snapshot().assets.peek_next(), 1);

    writer.undo().unwrap();
    assert!(writer.snapshot().assets.get(asset.id).is_none());
    assert_eq!(writer.snapshot().assets.peek_next(), 1);
    writer.redo().unwrap();
    assert_eq!(writer.snapshot().assets.get(asset.id), Some(&asset));
    assert_eq!(writer.snapshot().assets.peek_next(), 1);

    let remove = writer.prepare_remove_asset(asset.id).unwrap();
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, remove).unwrap();
    assert!(writer.snapshot().assets.get(asset.id).is_none());
    assert_eq!(writer.snapshot().assets.peek_next(), 1);

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().assets.get(asset.id), Some(&asset));
    assert_eq!(writer.snapshot().assets.peek_next(), 1);
    writer.redo().unwrap();
    assert!(writer.snapshot().assets.get(asset.id).is_none());
    assert_eq!(writer.snapshot().assets.peek_next(), 1);
}

#[test]
fn stale_admit_and_stale_remove_payload_leave_document_unchanged() {
    let mut writer = new_writer(Document::new_current());
    let first = writer.prepare_admit_asset(draft("first")).unwrap();
    let stale = writer.prepare_admit_asset(draft("stale")).unwrap();
    let id = first.asset().id;
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, first)
        .unwrap();
    let remove = writer.prepare_remove_asset(id).unwrap();
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, remove).unwrap();
    let before_stale = writer.snapshot();
    let revision = writer.revision;
    let stale_gesture = writer.begin_gesture();
    let error = writer
        .apply_prepared_asset_admission(stale_gesture, stale)
        .unwrap_err();
    assert_eq!(
        error,
        CommandError::StaleAssetAdmission {
            expected_next: 0,
            actual_next: 1,
        }
    );
    assert_eq!(writer.snapshot().as_ref(), before_stale.as_ref());
    assert_eq!(writer.revision, revision);

    writer.undo().unwrap();
    let stale_remove = writer.prepare_remove_asset(id).unwrap();
    let mut changed = writer.snapshot().as_ref().clone();
    let mut changed_asset = changed.assets.remove(id).unwrap();
    changed_asset.name = "relinked-name".into();
    changed.assets.restore(changed_asset).unwrap();
    let before_remove = changed.clone();
    let error = stale_remove.apply(&mut changed).unwrap_err();
    assert_eq!(
        error,
        CommandError::RemoveAssetPayloadMismatch { id: id.get() }
    );
    assert_eq!(changed, before_remove);
}

#[test]
fn referenced_asset_cannot_be_removed() {
    let mut writer = new_writer(Document::new_current());
    let admit = writer.prepare_admit_asset(draft("used")).unwrap();
    let id = admit.asset().id;
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();

    let mut document = writer.snapshot().as_ref().clone();
    let layer = document.layers.allocate("clip").unwrap();
    let track = document.track_ids.allocate("V1").unwrap();
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(id),
        })],
    });
    document.validate().unwrap();
    let writer = new_writer(document);
    let before = writer.snapshot();
    let error = writer.prepare_remove_asset(id).unwrap_err();
    assert_eq!(
        error,
        CommandError::AssetInUse {
            id: id.get(),
            use_count: 1,
        }
    );
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());
}

#[test]
fn v3_journal_reopens_admit_then_remove_without_changing_document_version() {
    let dir = unique_dir("journal");
    let path = dir.join("project.json");
    let limits = ResourceLimits::production();
    let base = Document::new_current();
    let version = base.version;
    let min_reader_version = base.min_reader_version;

    let mut session = ProjectSession::acquire(&path, &limits).unwrap();
    session
        .save_with_journal(&base, &SaveProjectOptions::default())
        .unwrap();

    let writer = new_writer(base.clone());
    let prepared = writer.prepare_admit_asset(draft("journaled")).unwrap();
    let id = prepared.asset().id;
    let admit = prepared.into_command_if_current(&base).unwrap();
    let mut admitted = base;
    admit.apply(&mut admitted).unwrap();
    session
        .save_with_journal(
            &admitted,
            &SaveProjectOptions {
                journal_edit: Some(JournalEdit::new(admit)),
                checkpoint: false,
                ..SaveProjectOptions::default()
            },
        )
        .unwrap();
    drop(session);

    let (mut session, opened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(opened.document, admitted);
    assert_eq!(
        opened.replay.as_ref().map(|replay| replay.applied_records),
        Some(1)
    );

    let writer = new_writer(opened.document.clone());
    let remove = writer.prepare_remove_asset(id).unwrap();
    let mut removed = opened.document;
    remove.apply(&mut removed).unwrap();
    session
        .save_with_journal(
            &removed,
            &SaveProjectOptions {
                journal_edit: Some(JournalEdit::new(remove)),
                checkpoint: false,
                ..SaveProjectOptions::default()
            },
        )
        .unwrap();
    drop(session);

    let (final_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(reopened.document, removed);
    assert_eq!(
        reopened
            .replay
            .as_ref()
            .map(|replay| replay.applied_records),
        Some(2)
    );
    assert_eq!(reopened.document.version, version);
    assert_eq!(reopened.document.min_reader_version, min_reader_version);
    drop(final_session);
    fs::remove_dir_all(dir).unwrap();
}
