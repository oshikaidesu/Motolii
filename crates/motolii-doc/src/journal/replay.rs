//! ジャーナルリプレイと失敗時スナップショットフォールバック(ガード4)。
//!
//! Editレコードの耐久payloadは versioned `Command` envelope のみ。
//! 故障注入は適用不能 Command の commit 等で行い、durable 形式へテスト専用 variant を焼かない。

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{Command, CommandError, Document, DocumentError};

use super::format::{JournalFrame, JournalRecordKind, JournalScanOutcome};
use super::fs::JournalFs;
use super::v1_edit::{apply_v1_journal_command, LegacyJournalCommand};

/// v1 journal Edit の format_version(読取専用互換)。
pub const V1_EDIT_FORMAT_VERSION: u32 = 1;
/// 新規書込みの journal Edit format_version(D1l §3.1)。
pub const V2_EDIT_FORMAT_VERSION: u32 = 2;

/// Editレコードのオンディスクpayload(恒久面)。`Command`を版付きで包む。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEdit {
    pub format_version: u32,
    pub command: Command,
}

impl JournalEdit {
    pub const FORMAT_VERSION: u32 = V2_EDIT_FORMAT_VERSION;

    pub fn new(command: Command) -> Self {
        Self {
            format_version: Self::FORMAT_VERSION,
            command,
        }
    }
}

impl From<Command> for JournalEdit {
    fn from(command: Command) -> Self {
        Self::new(command)
    }
}

#[derive(Debug, Clone)]
enum DecodedJournalEdit {
    V2(Box<Command>),
    V1(LegacyJournalCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplayFailure {
    InvalidEditPayload {
        record_id: Uuid,
        reason: String,
    },
    ApplyEdit {
        record_id: Uuid,
        source: ReplayApplyError,
    },
    MissingSnapshot {
        generation_id: Uuid,
    },
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReplayApplyError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error("legacy journal adapter: {0}")]
    LegacyJournal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayOutcome {
    pub document: Document,
    pub applied_records: usize,
    pub fallback_generation: Option<Uuid>,
    pub replay_failures: Vec<ReplayFailure>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub generation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointPayload {
    pub new_salt: u64,
    pub generation_id: Uuid,
}

pub fn edit_payload(edit: &JournalEdit) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(edit)
}

pub fn snapshot_payload(generation_id: Uuid) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&SnapshotPayload { generation_id })
}

pub fn checkpoint_payload(
    new_salt: u64,
    generation_id: Uuid,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = new_salt.to_le_bytes().to_vec();
    bytes.extend(serde_json::to_vec(&CheckpointPayload {
        new_salt,
        generation_id,
    })?);
    Ok(bytes)
}

pub fn document_fingerprint(doc: &Document) -> u64 {
    // 安定したJSON指紋 — tip照合用(暗号強度は不要)。
    let bytes = serde_json::to_vec(doc).unwrap_or_default();
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn apply_decoded_edit(
    doc: &mut Document,
    edit: &DecodedJournalEdit,
) -> Result<(), ReplayApplyError> {
    match edit {
        DecodedJournalEdit::V2(command) => {
            command.apply(doc)?;
            doc.validate()?;
        }
        DecodedJournalEdit::V1(legacy) => {
            apply_v1_journal_command(doc, legacy)
                .map_err(|e| ReplayApplyError::LegacyJournal(e.to_string()))?;
        }
    }
    Ok(())
}

fn decode_edit(payload: &[u8]) -> Result<DecodedJournalEdit, String> {
    let value: serde_json::Value =
        serde_json::from_slice(payload).map_err(|e| format!("invalid edit json: {e}"))?;
    let format_version = value
        .get("format_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing journal edit format_version".to_string())?;
    match format_version {
        v if v == u64::from(V1_EDIT_FORMAT_VERSION) => {
            let command_value = value
                .get("command")
                .ok_or_else(|| "missing journal edit command".to_string())?;
            let legacy: LegacyJournalCommand = serde_json::from_value(command_value.clone())
                .map_err(|e| format!("invalid v1 journal command: {e}"))?;
            Ok(DecodedJournalEdit::V1(legacy))
        }
        v if v == u64::from(V2_EDIT_FORMAT_VERSION) => {
            let edit: JournalEdit = serde_json::from_value(value)
                .map_err(|e| format!("invalid v2 journal edit: {e}"))?;
            Ok(DecodedJournalEdit::V2(Box::new(edit.command)))
        }
        other => Err(format!(
            "unsupported journal edit format_version {other} (expected {} or {})",
            V1_EDIT_FORMAT_VERSION, V2_EDIT_FORMAT_VERSION
        )),
    }
}

/// `base`からscan内のEditを順に適用。失敗時は直近Snapshot世代へフォールバック可能。
pub fn replay_from_base(
    mut base: Document,
    scan: &JournalScanOutcome,
    load_snapshot: &mut dyn FnMut(Uuid) -> Result<Document, ReplayFailure>,
    fallback_on_failure: bool,
) -> ReplayOutcome {
    let mut applied = 0usize;
    let mut failures = Vec::new();
    let mut last_snapshot: Option<Uuid> = None;
    let mut doc_at_last_snapshot = base.clone();

    for frame in &scan.frames {
        match frame.kind {
            JournalRecordKind::Snapshot => {
                let generation_id = frame.snapshot_ref.or_else(|| {
                    serde_json::from_slice::<SnapshotPayload>(&frame.payload)
                        .ok()
                        .map(|p| p.generation_id)
                });
                if let Some(gid) = generation_id {
                    match load_snapshot(gid) {
                        Ok(doc) => {
                            base = doc.clone();
                            doc_at_last_snapshot = doc;
                            last_snapshot = Some(gid);
                        }
                        Err(e) => failures.push(e),
                    }
                }
            }
            JournalRecordKind::Edit => match decode_edit(&frame.payload) {
                Ok(edit) => match apply_decoded_edit(&mut base, &edit) {
                    Ok(()) => applied += 1,
                    Err(source) => {
                        failures.push(ReplayFailure::ApplyEdit {
                            record_id: frame.record_id,
                            source,
                        });
                        if fallback_on_failure {
                            if let Some(gid) = last_snapshot {
                                return ReplayOutcome {
                                    document: doc_at_last_snapshot,
                                    applied_records: applied,
                                    fallback_generation: Some(gid),
                                    replay_failures: failures,
                                };
                            }
                        }
                    }
                },
                Err(reason) => {
                    failures.push(ReplayFailure::InvalidEditPayload {
                        record_id: frame.record_id,
                        reason,
                    });
                    if fallback_on_failure {
                        if let Some(gid) = last_snapshot {
                            return ReplayOutcome {
                                document: doc_at_last_snapshot,
                                applied_records: applied,
                                fallback_generation: Some(gid),
                                replay_failures: failures,
                            };
                        }
                    }
                }
            },
            JournalRecordKind::Commit | JournalRecordKind::Checkpoint => {}
        }
    }

    ReplayOutcome {
        document: base,
        applied_records: applied,
        fallback_generation: None,
        replay_failures: failures,
    }
}

pub fn load_generation_via_fs(
    fs: &mut dyn JournalFs,
    path: &std::path::Path,
    limits: &crate::limits::ResourceLimits,
) -> Result<Document, crate::PersistError> {
    let bytes = fs.read(path).map_err(|e| match e {
        super::fs::FsError::Io(io) => crate::PersistError::Io(io),
        other => crate::PersistError::Io(std::io::Error::other(other.to_string())),
    })?;
    match crate::load_document_bytes_with_limits(&bytes, limits) {
        Ok(opened) => Ok(opened.document),
        Err(load_err) => {
            if let Ok((doc, _)) = crate::migrate_bytes_with_limits(&bytes, limits) {
                return Ok(doc);
            }
            Err(load_err)
        }
    }
}

/// フレーム列から「最後にCommitされた」範囲だけを残す(未commitテールは無視)。
pub fn frames_through_last_commit(frames: &[JournalFrame]) -> &[JournalFrame] {
    let last_commit = frames
        .iter()
        .rposition(|f| f.kind == JournalRecordKind::Commit);
    match last_commit {
        Some(i) => &frames[..=i],
        None => &[],
    }
}

#[cfg(test)]
mod replay_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use motolii_core::RationalTime;
    use serde_json::json;

    use crate::journal::{
        generation_path_for_document, load_catalog, open_project, replay_from_base, JournalEdit,
        JournalRecordKind, JournalScanOutcome, ReplayFailure, V1_EDIT_FORMAT_VERSION,
    };
    use crate::{
        migrate_bytes, save_project_with_journal, Clip, ClipSource, Command, DocParam, Document,
        EffectUse, ItemEnvelope, LayerId, ResourceLimits, SaveProjectOptions, ScalarPropertyId,
        Track, TrackItem,
    };

    fn unique_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-d1l-replay-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn v1_edit_bytes(command: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "format_version": V1_EDIT_FORMAT_VERSION,
            "command": command,
        }))
        .unwrap()
    }

    fn commit_edit_payload(
        document_path: &std::path::Path,
        payload: Vec<u8>,
        limits: &ResourceLimits,
    ) -> Result<(), super::super::project::ProjectError> {
        use super::super::catalog::load_catalog_fs;
        use super::super::format::{encode_frame, JournalFrame, JournalRecordKind};
        use super::super::fs::{DurabilityStage, JournalFs, StdFs};
        use super::super::wal::{WalError, WalSession};

        let mut fs = StdFs;
        let catalog = load_catalog_fs(&mut fs, document_path)?.ok_or_else(|| {
            super::super::project::ProjectError::Persist(crate::PersistError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "journal catalog missing; save project first",
                ),
            ))
        })?;
        let mut session = WalSession::open_or_create(
            &mut fs,
            document_path,
            catalog.project_id,
            catalog.generation_salt,
            catalog.max_unpinned,
        )?;
        if payload.len() as u32 > limits.max_command_payload_bytes {
            return Err(WalError::RecordPayloadLimit {
                observed: payload.len() as u32,
                limit: limits.max_command_payload_bytes,
            }
            .into());
        }
        let record_id = uuid::Uuid::new_v4();
        let edit_frame = JournalFrame {
            record_id,
            prev_id: session.last_record,
            snapshot_ref: None,
            record_salt: session.header.generation_salt,
            kind: JournalRecordKind::Edit,
            payload,
        };
        fs.append(&session.journal_path, &encode_frame(&edit_frame))?;
        fs.note_stage(DurabilityStage::JournalAppend)?;
        fs.sync_file(&session.journal_path)?;
        fs.note_stage(DurabilityStage::JournalFsync)?;
        let commit_id = uuid::Uuid::new_v4();
        let commit_frame = JournalFrame {
            record_id: commit_id,
            prev_id: Some(record_id),
            snapshot_ref: None,
            record_salt: session.header.generation_salt,
            kind: JournalRecordKind::Commit,
            payload: Vec::new(),
        };
        fs.append(&session.journal_path, &encode_frame(&commit_frame))?;
        fs.note_stage(DurabilityStage::JournalAppend)?;
        fs.sync_file(&session.journal_path)?;
        fs.note_stage(DurabilityStage::JournalFsync)?;
        session.last_record = Some(commit_id);
        session.catalog.edits_since_snapshot =
            session.catalog.edits_since_snapshot.saturating_add(1);
        super::super::catalog::save_catalog_fs(&mut fs, document_path, &session.catalog)?;
        Ok(())
    }

    fn empty_clip_base() -> (Document, LayerId) {
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
        doc.validate().unwrap();
        (doc, layer)
    }

    fn inline_effect_json() -> serde_json::Value {
        json!({
            "id": 5,
            "plugin_id": "vendor.unknown.glow",
            "effect_version": 2,
            "enabled": false,
            "params": {"amount": {"const": {"F64": 0.75}}},
            "vendor_custom": "keep-me"
        })
    }

    fn clip_effects(doc: &Document) -> Vec<EffectUse> {
        let TrackItem::Clip(c) = &doc.tracks[0].items[0] else {
            panic!("expected clip");
        };
        c.envelope.effects.clone()
    }

    fn clip_opacity(doc: &Document) -> DocParam {
        let TrackItem::Clip(c) = &doc.tracks[0].items[0] else {
            panic!("expected clip");
        };
        c.envelope.opacity.clone()
    }

    #[test]
    fn mixed_v1_v2_wal_replays_without_failures() {
        let dir = unique_dir("mixed");
        let path = dir.join("proj.json");
        let (doc, layer) = empty_clip_base();
        save_project_with_journal(&path, &doc, &SaveProjectOptions::default()).unwrap();

        let v1_add = v1_edit_bytes(json!({
            "AddEffect": {
                "target": layer.get(),
                "index": 0,
                "effect": inline_effect_json()
            }
        }));
        commit_edit_payload(&path, v1_add, &ResourceLimits::production()).unwrap();

        let v2_opacity = JournalEdit::new(Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.25),
        });
        save_project_with_journal(
            &path,
            &doc,
            &SaveProjectOptions {
                journal_edit: Some(v2_opacity),
                checkpoint: false,
                ..Default::default()
            },
        )
        .unwrap();

        let opened = open_project(&path).unwrap();
        let replay = opened.replay.expect("journal replay expected");
        assert!(
            replay.replay_failures.is_empty(),
            "{:?}",
            replay.replay_failures
        );
        assert_eq!(clip_opacity(&replay.document), DocParam::const_f64(0.25));
        assert_eq!(clip_effects(&replay.document).len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recovery_replays_v1_edits_after_memory_d1e_on_inline_generation() {
        let dir = unique_dir("recovery-inline");
        let path = dir.join("proj.json");
        let legacy_empty = json!({
            "version": 1,
            "min_reader_version": 1,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1}
            },
            "bpm": {"num": 120, "den": 1},
            "layers": {"next": 1, "entries": [{"id": 0, "name": "a"}]},
            "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
            "tracks": [{
                "id": 0,
                "items": [{
                    "kind": "clip",
                    "envelope": {
                        "layer_id": 0,
                        "effects": [],
                        "transform": {
                            "position": {"const": {"Vec2": [0.0, 0.0]}},
                            "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                            "scale": {"const": {"Vec2": [1.0, 1.0]}},
                            "rotation": {"const": {"F64": 0.0}}
                        },
                        "opacity": {"const": {"F64": 1.0}}
                    },
                    "start": {"num": 0, "den": 1},
                    "duration": {"num": 5, "den": 1},
                    "time_map": {
                        "source_start": {"num": 0, "den": 1},
                        "speed_num": 1,
                        "speed_den": 1,
                        "overrun_mode": "freeze"
                    },
                    "source": {
                        "source": "asset",
                        "asset": 0,
                        "video": {"stream": {"kind": "video", "ordinal": 0}},
                        "audio": []
                    }
                }]
            }],
            "assets": {"next": 1, "entries": [{
                "id": 0, "name": "media", "asset_type": "video/mp4", "content_hash": "hash"
            }]},
            "next_stable_id": 6
        });
        let (migrated, _) = migrate_bytes(&serde_json::to_vec(&legacy_empty).unwrap()).unwrap();
        save_project_with_journal(&path, &migrated, &SaveProjectOptions::default()).unwrap();

        let catalog = load_catalog(&path).unwrap().expect("catalog");
        let gen_id = catalog.latest_generation().expect("generation").id;
        let gen_path = generation_path_for_document(&path, gen_id);
        let legacy_bytes = fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/corpus/timeline_start/speed_clip.json"
        ))
        .unwrap();
        fs::write(&gen_path, &legacy_bytes).unwrap();
        fs::write(&path, b"{broken-main").unwrap();

        let v1_add = v1_edit_bytes(json!({
            "AddEffect": {
                "target": 0,
                "index": 0,
                "effect": inline_effect_json()
            }
        }));
        commit_edit_payload(&path, v1_add, &ResourceLimits::production()).unwrap();

        let opened = open_project(&path).unwrap();
        let replay = opened.replay.expect("replay");
        assert!(
            replay.replay_failures.is_empty(),
            "{:?}",
            replay.replay_failures
        );
        assert_eq!(clip_effects(&opened.document).len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn known_v1_apply_failure_is_typed_not_snapshot_fallback() {
        let (doc, layer) = shared_remove_fixture();
        let before = doc.clone();
        let payload = v1_edit_bytes(json!({
            "RemoveEffect": {
                "target": layer.get(),
                "index": 0,
                "effect": inline_effect_json()
            }
        }));
        let scan = JournalScanOutcome {
            header: super::super::format::JournalHeader {
                version: 1,
                generation_salt: 1,
                project_id: uuid::Uuid::new_v4(),
            },
            frames: vec![super::super::format::JournalFrame {
                record_id: uuid::Uuid::new_v4(),
                prev_id: None,
                snapshot_ref: Some(uuid::Uuid::new_v4()),
                record_salt: 1,
                kind: JournalRecordKind::Edit,
                payload,
            }],
            valid_bytes: 0,
            file_len: 0,
            stopped: None,
        };
        let snapshot_doc = before.clone();
        let outcome = replay_from_base(
            before.clone(),
            &scan,
            &mut |_gid| Ok(snapshot_doc.clone()),
            true,
        );
        assert_eq!(outcome.document, before);
        assert!(outcome.fallback_generation.is_none());
        assert_eq!(outcome.replay_failures.len(), 1);
        assert!(matches!(
            outcome.replay_failures[0],
            ReplayFailure::ApplyEdit { .. }
        ));
    }

    #[test]
    fn corrupt_payload_uses_snapshot_fallback() {
        let before = Document::new_current();
        let snapshot = before.clone();
        let gid = uuid::Uuid::new_v4();
        let scan = JournalScanOutcome {
            header: super::super::format::JournalHeader {
                version: 1,
                generation_salt: 1,
                project_id: uuid::Uuid::new_v4(),
            },
            frames: vec![
                super::super::format::JournalFrame {
                    record_id: uuid::Uuid::new_v4(),
                    prev_id: None,
                    snapshot_ref: Some(gid),
                    record_salt: 1,
                    kind: JournalRecordKind::Snapshot,
                    payload: Vec::new(),
                },
                super::super::format::JournalFrame {
                    record_id: uuid::Uuid::new_v4(),
                    prev_id: None,
                    snapshot_ref: None,
                    record_salt: 1,
                    kind: JournalRecordKind::Edit,
                    payload: b"not-json".to_vec(),
                },
            ],
            valid_bytes: 0,
            file_len: 0,
            stopped: None,
        };
        let outcome = replay_from_base(
            before,
            &scan,
            &mut |id| {
                assert_eq!(id, gid);
                Ok(snapshot.clone())
            },
            true,
        );
        assert_eq!(outcome.document, snapshot);
        assert_eq!(outcome.fallback_generation, Some(gid));
        assert!(matches!(
            outcome.replay_failures[0],
            ReplayFailure::InvalidEditPayload { .. }
        ));
    }

    fn shared_remove_fixture() -> (Document, LayerId) {
        use crate::schema::EffectDefinition;
        use crate::stable_id::{EffectDefinitionId, EffectId};
        use std::collections::BTreeMap;

        let mut doc = Document::new_current();
        let layer_a = doc.layers.allocate("a").unwrap();
        let layer_b = doc.layers.allocate("b").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        let u1 = EffectId::from_raw(5);
        let u2 = EffectId::from_raw(6);
        let d1 = EffectDefinitionId::from_raw(10);
        doc.effect_definitions.push(EffectDefinition::new(
            d1,
            "vendor.unknown.glow",
            2,
            false,
            BTreeMap::from([("amount".into(), DocParam::const_f64(0.75))]),
            serde_json::Map::from_iter([("vendor_custom".into(), json!("keep-me"))]),
        ));
        let mut env_a = ItemEnvelope::new(layer_a);
        env_a.effects.push(EffectUse {
            id: u1,
            definition_id: d1,
        });
        let mut env_b = ItemEnvelope::new(layer_b);
        env_b.effects.push(EffectUse {
            id: u2,
            definition_id: d1,
        });
        doc.tracks.push(Track {
            id: track,
            items: vec![
                TrackItem::Clip(Clip {
                    envelope: env_a,
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(5, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                }),
                TrackItem::Clip(Clip {
                    envelope: env_b,
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(5, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                }),
            ],
        });
        while doc.next_stable_id.peek_next() < 11 {
            let _ = doc.next_stable_id.allocate();
        }
        doc.validate().unwrap();
        (doc, layer_a)
    }
}
