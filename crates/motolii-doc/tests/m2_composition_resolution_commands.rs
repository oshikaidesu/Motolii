//! 2026-08-17決定「出力解像度はCompositionが所有し、素材はfitで受ける」の粒1(doc側):
//! `Composition.resolution`のschema roundtrip(None省略/Some保存)、唯一の書き込み経路
//! `Command::SetCompositionResolution`のundo/redo・journal replay・serde roundtrip、
//! validate(正値・上限)の負例を閉じる。SetSoundtrack(`m2_soundtrack_commands.rs`)と同型。

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    Command, CommandError, Document, DocumentError, DocumentWriter, JournalEdit, ProjectSession,
    ResourceLimits, SaveProjectOptions, MAX_COMPOSITION_RESOLUTION_DIMENSION,
};

fn new_writer(document: Document) -> DocumentWriter {
    let catalog = Arc::new(motolii_plugin::reference::reference_catalog().unwrap());
    DocumentWriter::new(document, catalog).unwrap()
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-comp-resolution-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn composition_resolution_none_is_omitted_and_some_round_trips() {
    // None: 旧文書とバイト単位で同じ形(キー自体を書かない)。
    let doc = Document::new_current();
    assert_eq!(doc.composition.resolution(), None);
    let json = serde_json::to_value(&doc).unwrap();
    let composition = json
        .get("composition")
        .and_then(|v| v.as_object())
        .expect("composition object");
    assert!(
        !composition.contains_key("resolution"),
        "None must not serialize a resolution key (locator-style additive field)"
    );

    // Some: 保存され、開き直しても同じ値。
    let mut doc = Document::new_current();
    doc.composition.set_resolution(Some((1920, 1080))).unwrap();
    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"resolution\""));
    let back: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(back.composition.resolution(), Some((1920, 1080)));
    assert_eq!(back, doc);
}

#[test]
fn composition_resolution_rejects_zero_and_over_limit_on_deserialize() {
    let mut json = serde_json::to_value(Document::new_current()).unwrap();
    json["composition"]["resolution"] = serde_json::json!([0, 1080]);
    let err = serde_json::from_value::<Document>(json.clone()).unwrap_err();
    assert!(
        err.to_string().contains("positive"),
        "zero width must be a typed rejection, got: {err}"
    );

    json["composition"]["resolution"] =
        serde_json::json!([MAX_COMPOSITION_RESOLUTION_DIMENSION + 1, 1080]);
    let err = serde_json::from_value::<Document>(json).unwrap_err();
    assert!(
        err.to_string().contains("exceeds max dimension"),
        "over-limit width must be a typed rejection, got: {err}"
    );
}

#[test]
fn set_composition_resolution_undo_redo_and_clear_round_trip() {
    let mut writer = new_writer(Document::new_current());
    assert_eq!(writer.snapshot().composition.resolution(), None);

    let set = Command::SetCompositionResolution {
        old: None,
        new: Some((1920, 1080)),
    };
    let clear = Command::SetCompositionResolution {
        old: Some((1920, 1080)),
        new: None,
    };
    // inverse は old/new の入れ替え(SetSoundtrackと同じ対称)。
    assert_eq!(set.inverse(), clear);
    assert_eq!(clear.inverse(), set);

    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, set).unwrap();
    assert_eq!(
        writer.snapshot().composition.resolution(),
        Some((1920, 1080))
    );

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().composition.resolution(), None);
    writer.redo().unwrap();
    assert_eq!(
        writer.snapshot().composition.resolution(),
        Some((1920, 1080))
    );

    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, clear).unwrap();
    assert_eq!(writer.snapshot().composition.resolution(), None);
    writer.undo().unwrap();
    assert_eq!(
        writer.snapshot().composition.resolution(),
        Some((1920, 1080))
    );
    writer.redo().unwrap();
    assert_eq!(writer.snapshot().composition.resolution(), None);
}

#[test]
fn set_composition_resolution_journal_edit_serializes_and_replays_after_reopen() {
    let dir = unique_dir("journal");
    let path = dir.join("project.json");
    let limits = ResourceLimits::production();

    let base = Document::new_current();
    let mut session = ProjectSession::acquire(&path, &limits).unwrap();
    session
        .save_with_journal(&base, &SaveProjectOptions::default())
        .unwrap();

    let command = Command::SetCompositionResolution {
        old: None,
        new: Some((1920, 1080)),
    };

    // 外部tag形(既存 Command 慣行)で old/new を持ち、値ごと roundtrip する。
    let value = serde_json::to_value(&command).unwrap();
    let object = value.as_object().expect("externally tagged serde shape");
    assert_eq!(object.len(), 1);
    let fields = object
        .get("SetCompositionResolution")
        .expect("dedicated command tag")
        .as_object()
        .expect("SetCompositionResolution payload must be an object");
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

    let mut with_resolution = base.clone();
    command.apply(&mut with_resolution).unwrap();
    session
        .save_with_journal(
            &with_resolution,
            &SaveProjectOptions {
                journal_edit: Some(edit),
                checkpoint: false,
                ..SaveProjectOptions::default()
            },
        )
        .unwrap();
    drop(session);

    let (final_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(reopened.document, with_resolution);
    assert_eq!(
        reopened.document.composition.resolution(),
        Some((1920, 1080))
    );
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
fn set_composition_resolution_rejects_invalid_values_and_leaves_document_unchanged() {
    for invalid in [
        (0, 1080),
        (1920, 0),
        (0, 0),
        (MAX_COMPOSITION_RESOLUTION_DIMENSION + 1, 1080),
        (1920, MAX_COMPOSITION_RESOLUTION_DIMENSION + 1),
    ] {
        let mut writer = new_writer(Document::new_current());
        let before = writer.snapshot();
        let revision = writer.revision;
        let gesture = writer.begin_gesture();
        let error = writer
            .apply_command(
                gesture,
                Command::SetCompositionResolution {
                    old: None,
                    new: Some(invalid),
                },
            )
            .unwrap_err();
        assert!(
            matches!(
                error,
                CommandError::Validate(DocumentError::InvalidCompositionResolution { .. })
            ),
            "{invalid:?} must be a typed validate rejection, got {error:?}"
        );
        assert_eq!(writer.snapshot().as_ref(), before.as_ref());
        assert_eq!(writer.revision, revision);
        assert_eq!(writer.snapshot().composition.resolution(), None);
    }
}
