//! N-SOUNDTRACK-WRITE: `Document.soundtrack` を設定する唯一の製品書き込み経路
//! (`Command::SetSoundtrack`)。set None→Some→undo→redo、journal roundtrip、
//! 不在 AssetId の負例を閉じる。

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::RationalTime;
use motolii_doc::{
    AssetDraft, AssetId, Command, CommandError, Document, DocumentError, DocumentWriter,
    JournalEdit, ProjectSession, ResourceLimits, SaveProjectOptions, Soundtrack,
};

fn new_writer(document: Document) -> DocumentWriter {
    let catalog = Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
    DocumentWriter::new(document, catalog).unwrap()
}

fn audio_draft(name: &str) -> AssetDraft {
    AssetDraft {
        name: name.into(),
        asset_type: "audio/mpeg".into(),
        content_hash:
            "motolii-source-v1:sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                .into(),
        path_absolute: Some(r"C:\media\music.mp3".into()),
        path_project_relative: Some(r"media\music.mp3".into()),
        file_name: Some("music.mp3".into()),
        size_bytes: Some(3),
        head_hash: None,
        tail_hash: None,
        duration: None,
    }
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-soundtrack-command-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// 楽曲1本が載ったwriterと、そのasset idを返す。
fn writer_with_audio_asset() -> (DocumentWriter, AssetId) {
    let mut writer = new_writer(Document::new_current());
    let admit = writer.prepare_admit_asset(audio_draft("music")).unwrap();
    let asset_id = admit.asset().id;
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();
    (writer, asset_id)
}

#[test]
fn set_soundtrack_none_to_some_undo_redo_and_clear_round_trip() {
    let (mut writer, asset_id) = writer_with_audio_asset();
    assert!(writer.snapshot().soundtrack.is_none());

    let soundtrack =
        Soundtrack::try_new(asset_id, RationalTime::try_new(1, 2).unwrap(), 0.8).unwrap();
    let set = Command::SetSoundtrack {
        old: None,
        new: Some(soundtrack),
    };
    let clear = Command::SetSoundtrack {
        old: Some(soundtrack),
        new: None,
    };
    // inverse は old/new の入れ替え(近隣 Set* 慣行)。
    assert_eq!(set.inverse(), clear);
    assert_eq!(clear.inverse(), set);

    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, set).unwrap();
    assert_eq!(writer.snapshot().soundtrack, Some(soundtrack));

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().soundtrack, None);
    writer.redo().unwrap();
    assert_eq!(writer.snapshot().soundtrack, Some(soundtrack));

    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, clear).unwrap();
    assert_eq!(writer.snapshot().soundtrack, None);
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().soundtrack, Some(soundtrack));
    writer.redo().unwrap();
    assert_eq!(writer.snapshot().soundtrack, None);
}

#[test]
fn set_soundtrack_journal_edit_serializes_and_replays_after_reopen() {
    let dir = unique_dir("journal");
    let path = dir.join("project.json");
    let limits = ResourceLimits::production();

    let base = Document::new_current();
    let writer = new_writer(base.clone());
    let prepared = writer.prepare_admit_asset(audio_draft("music")).unwrap();
    let asset_id = prepared.asset().id;
    let admit = prepared.into_command_if_current(&base).unwrap();
    let mut with_asset = base;
    admit.apply(&mut with_asset).unwrap();

    let mut session = ProjectSession::acquire(&path, &limits).unwrap();
    session
        .save_with_journal(&with_asset, &SaveProjectOptions::default())
        .unwrap();

    let soundtrack =
        Soundtrack::try_new(asset_id, RationalTime::try_new(3, 4).unwrap(), 0.5).unwrap();
    let command = Command::SetSoundtrack {
        old: None,
        new: Some(soundtrack),
    };

    // 外部tag形(既存 Command 慣行)で old/new を持ち、値ごと roundtrip する。
    let value = serde_json::to_value(&command).unwrap();
    let object = value.as_object().expect("externally tagged serde shape");
    assert_eq!(object.len(), 1);
    let fields = object
        .get("SetSoundtrack")
        .expect("dedicated command tag")
        .as_object()
        .expect("SetSoundtrack payload must be an object");
    for field in ["old", "new"] {
        assert!(fields.contains_key(field), "missing {field}");
    }
    let back: Command = serde_json::from_value(value).unwrap();
    assert_eq!(back, command);

    let edit = JournalEdit::new(command.clone());
    assert_eq!(
        edit.format_version,
        motolii_doc::journal::V3_EDIT_FORMAT_VERSION
    );

    let mut with_soundtrack = with_asset.clone();
    command.apply(&mut with_soundtrack).unwrap();
    session
        .save_with_journal(
            &with_soundtrack,
            &SaveProjectOptions {
                journal_edit: Some(edit),
                checkpoint: false,
                ..SaveProjectOptions::default()
            },
        )
        .unwrap();
    drop(session);

    let (final_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(reopened.document, with_soundtrack);
    assert_eq!(reopened.document.soundtrack, Some(soundtrack));
    assert_eq!(
        reopened
            .replay
            .as_ref()
            .map(|replay| replay.applied_records),
        Some(1)
    );
    drop(final_session);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn set_soundtrack_with_unknown_asset_is_refused_and_leaves_document_unchanged() {
    let (mut writer, _asset_id) = writer_with_audio_asset();
    let missing = AssetId::from_raw(7);
    let soundtrack = Soundtrack::try_new(missing, RationalTime::ZERO, 1.0).unwrap();
    let command = Command::SetSoundtrack {
        old: None,
        new: Some(soundtrack),
    };

    let before = writer.snapshot();
    let revision = writer.revision;
    let gesture = writer.begin_gesture();
    let error = writer.apply_command(gesture, command).unwrap_err();
    assert_eq!(
        error,
        CommandError::Validate(DocumentError::UnknownAssetId { id: missing.get() })
    );
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());
    assert_eq!(writer.revision, revision);
    assert!(writer.snapshot().soundtrack.is_none());
}
