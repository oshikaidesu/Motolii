//! D1c と並走するプロジェクト open/save。D1c 契約はそのまま呼び出す。
//!
//! ## tip 判定の方針
//!
//! - `last_journaled_fingerprint` がある場合:
//!   - 一致 → InSync(リプレイ可)。フォールバックが tip より古ければ main/base 保持
//!   - 不一致 + main が旧世代一致 + tip 証明可 → MainBehind(前方復旧)
//!   - 不一致 + 世代不一致 → main 先行/曖昧 → KeepMain
//! - 指紋欠落時も journal/世代から tip を再構成し、同様に MainBehind / InSync / KeepMain を決める。
//!   tip を証明できないときだけ KeepMain(安全側)
//! - `open_without_main` も generation `base` に対し同じ anti-rewind を適用する

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::persist::{save_document, save_document_with_options, SaveOptions};
use crate::{Document, PersistError};

use super::catalog::{
    generation_path_for_document, load_catalog_lenient, rebuild_catalog_from_generations,
    rotate_generations, save_catalog, GenerationCatalog, PinGenerationOptions, RotateOptions,
};
use super::format::{
    append_frame, encode_frame, encode_header, journal_path_for_document, read_or_create_header,
    scan_journal, truncate_journal, JournalFrame, JournalHeader, JournalRecordKind,
    JournalScanOutcome, ScanJournalOptions, HEADER_LEN,
};
use super::replay::{
    document_fingerprint, edit_payload, load_generation_snapshot, replay_journal, snapshot_payload,
    JournalEdit, ReplayOptions, ReplayOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    MainFile,
    JournalReplay,
    SnapshotFallback,
    TruncatedJournalThenReplay,
    /// main が読めず世代スナップショット+リプレイで復元した。
    GenerationRecovery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenProjectOutcome {
    pub document: Document,
    pub source: RecoverySource,
    pub truncated_bytes: u64,
    pub replay: Option<ReplayOutcome>,
}

#[derive(Debug, Clone, Default)]
pub struct SaveProjectOptions {
    pub persist: SaveOptions,
    pub journal_edit: Option<JournalEdit>,
    pub snapshot_every_n_edits: Option<u32>,
    /// true ならこの保存ではスナップショット世代を作らない(テール注入用)。
    pub skip_snapshot: bool,
    pub pin_generation: Option<PinGenerationOptions>,
    pub rotate: Option<RotateOptions>,
    pub max_unpinned_generations: Option<u32>,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Journal(#[from] super::format::JournalFormatError),
    #[error(transparent)]
    Catalog(#[from] super::catalog::CatalogError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("main document missing at {0}")]
    MissingMainDocument(PathBuf),
    #[error("main document corrupt and no recoverable journal/generation at {path}")]
    Unrecoverable { path: PathBuf },
}

fn new_journal_salt(document_path: &Path) -> Result<u64, ProjectError> {
    if let Ok((Some(catalog), _)) = load_catalog_lenient(document_path) {
        return Ok(catalog.journal_salt);
    }
    let journal_path = journal_path_for_document(document_path);
    if journal_path.exists() {
        let data = fs::read(&journal_path)?;
        if data.len() >= super::format::HEADER_LEN {
            return Ok(super::format::read_header(&data)?.file_salt);
        }
    }
    Ok(Uuid::new_v4().as_u128() as u64)
}

struct ProjectLayout {
    journal_path: PathBuf,
    catalog: GenerationCatalog,
    header: JournalHeader,
    last_record: Option<Uuid>,
}

fn ensure_layout(document_path: &Path, file_salt: u64) -> Result<ProjectLayout, ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let header = read_or_create_header(&journal_path, file_salt)?;
    let catalog = match load_catalog_lenient(document_path)? {
        (Some(mut c), _) => {
            c.journal_salt = header.file_salt;
            c
        }
        (None, corrupted) => {
            if corrupted {
                rebuild_catalog_from_generations(document_path, header.file_salt, 5)?
            } else {
                GenerationCatalog::new(header.file_salt, 5)
            }
        }
    };
    let last_record = scan_journal(&journal_path, &ScanJournalOptions::default())
        .ok()
        .and_then(|s| s.frames.last().map(|f| f.record_id));
    Ok(ProjectLayout {
        journal_path,
        catalog,
        header,
        last_record,
    })
}

fn write_generation_snapshot(
    document_path: &Path,
    generation_id: Uuid,
    doc: &Document,
) -> Result<(), ProjectError> {
    let path = generation_path_for_document(document_path, generation_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    save_document(&path, doc)?;
    Ok(())
}

fn push_frame(
    layout: &mut ProjectLayout,
    kind: JournalRecordKind,
    snapshot_ref: Option<Uuid>,
    payload: Vec<u8>,
) -> Result<Uuid, ProjectError> {
    let record_id = Uuid::new_v4();
    let frame = JournalFrame {
        record_id,
        prev_id: layout.last_record,
        snapshot_ref,
        record_salt: layout.header.file_salt,
        kind,
        payload,
    };
    append_frame(&layout.journal_path, &frame)?;
    layout.last_record = Some(record_id);
    Ok(record_id)
}

/// D1c の atomic save の後にジャーナル追記・世代管理を行う。
pub fn save_project_with_journal(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    save_document_with_options(document_path, doc, &options.persist)?;

    let mut layout = ensure_layout(document_path, new_journal_salt(document_path)?)?;
    if let Some(max) = options.max_unpinned_generations {
        layout.catalog.max_unpinned = max;
    }

    // catalog に永続化したカウンタを跨ぎ save で使う
    let mut edits_since_snapshot = layout.catalog.edits_since_snapshot;

    if let Some(edit) = &options.journal_edit {
        let payload = edit_payload(edit)?;
        push_frame(&mut layout, JournalRecordKind::Edit, None, payload)?;
        edits_since_snapshot = edits_since_snapshot.saturating_add(1);
    }

    let snapshot_interval = options.snapshot_every_n_edits.unwrap_or(0);
    let need_snapshot = !options.skip_snapshot
        && snapshot_interval > 0
        && edits_since_snapshot >= snapshot_interval;
    let bootstrap_snapshot = !options.skip_snapshot
        && options.journal_edit.is_none()
        && layout.catalog.generations.is_empty();
    if need_snapshot || bootstrap_snapshot {
        let generation_id = Uuid::new_v4();
        write_generation_snapshot(document_path, generation_id, doc)?;
        let payload = snapshot_payload(generation_id);
        let journal_record = push_frame(
            &mut layout,
            JournalRecordKind::Snapshot,
            Some(generation_id),
            payload,
        )?;
        layout
            .catalog
            .register_generation(generation_id, journal_record, false);
        edits_since_snapshot = 0;
    }

    if let Some(pin) = &options.pin_generation {
        layout.catalog.pin_generation(pin.generation_id)?;
        let payload = serde_json::to_vec(&PinPayload {
            generation_id: pin.generation_id,
        })?;
        push_frame(
            &mut layout,
            JournalRecordKind::PinGeneration,
            Some(pin.generation_id),
            payload,
        )?;
    }

    if let Some(rotate) = &options.rotate {
        rotate_generations(document_path, &mut layout.catalog, rotate)?;
    }

    layout.catalog.edits_since_snapshot = edits_since_snapshot;
    layout.catalog.last_journaled_fingerprint = Some(document_fingerprint(doc)?);
    save_catalog(document_path, &layout.catalog)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct PinPayload {
    generation_id: Uuid,
}

enum MainLoad {
    Ok(Box<Document>),
    Missing,
    Corrupt,
}

fn try_load_main(document_path: &Path) -> Result<MainLoad, ProjectError> {
    if !document_path.exists() {
        return Ok(MainLoad::Missing);
    }
    match crate::load_document(document_path) {
        Ok(doc) => Ok(MainLoad::Ok(Box::new(doc))),
        Err(_) => Ok(MainLoad::Corrupt),
    }
}

fn heal_journal(document_path: &Path) -> Result<(JournalScanOutcome, u64), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    if !journal_path.exists() {
        return Err(ProjectError::Journal(
            super::format::JournalFormatError::PartialFrame(0),
        ));
    }
    let data = fs::read(&journal_path)?;
    let before_len = data.len() as u64;
    let scan = super::format::scan_journal_bytes(&data, &ScanJournalOptions::default())?;
    let truncated = if scan.valid_bytes < before_len {
        truncate_journal(&journal_path, scan.valid_bytes)?;
        before_len - scan.valid_bytes
    } else {
        0
    };
    Ok((scan, truncated))
}

fn resolve_catalog(
    document_path: &Path,
    file_salt: u64,
) -> Result<(GenerationCatalog, bool), ProjectError> {
    match load_catalog_lenient(document_path)? {
        (Some(catalog), false) => Ok((catalog, false)),
        (None, corrupted) => {
            let rebuilt = rebuild_catalog_from_generations(document_path, file_salt, 5)?;
            Ok((rebuilt, corrupted))
        }
        (Some(_), true) => unreachable!("lenient returns None when corrupted"),
    }
}

fn base_from_latest_generation(
    document_path: &Path,
    catalog: &GenerationCatalog,
) -> Option<Document> {
    let mut gens: Vec<_> = catalog.generations.iter().collect();
    gens.sort_by_key(|g| std::cmp::Reverse(g.created_seq));
    for entry in gens {
        if let Ok(doc) = load_generation_snapshot(document_path, entry.id) {
            return Some(doc);
        }
    }
    None
}

fn classify_replay_source(
    replay: &ReplayOutcome,
    truncated: u64,
    scan: &JournalScanOutcome,
) -> RecoverySource {
    if !replay.replay_failures.is_empty() {
        RecoverySource::SnapshotFallback
    } else if truncated > 0 {
        RecoverySource::TruncatedJournalThenReplay
    } else if scan
        .frames
        .iter()
        .any(|f| f.kind == JournalRecordKind::Edit)
    {
        RecoverySource::JournalReplay
    } else {
        RecoverySource::MainFile
    }
}

/// main と journal tip の関係。
///
/// - 指紋欠落/曖昧: main を巻き戻さない(安全側)
/// - main が旧世代に一致し tip が証明できる: 前方復旧
enum TipRelation {
    /// catalog 指紋 == main。journal リプレイ可。
    InSync,
    /// tip 不明、または main が先行/無関係。main を保持。
    KeepMain,
    /// main は旧世代、journal tip が新しい。前方復旧。
    MainBehind(Box<Document>),
}

fn tip_relation(
    document_path: &Path,
    catalog: &GenerationCatalog,
    main_fp: u64,
    scan: &JournalScanOutcome,
) -> Result<TipRelation, ProjectError> {
    let reconstructed = reconstruct_journal_tip(document_path, catalog, scan)?;

    // catalog 指紋があればそれを tip 証明に使う。無ければ再構成結果の指紋を tip とする。
    let tip_fp = match catalog.last_journaled_fingerprint {
        Some(fp) => fp,
        None => {
            let Some(tip_doc) = reconstructed.as_ref() else {
                return Ok(TipRelation::KeepMain);
            };
            let tip_fp = document_fingerprint(tip_doc)?;
            if tip_fp == main_fp {
                return Ok(TipRelation::InSync);
            }
            if main_matches_generation(document_path, catalog, main_fp)? {
                // tip 再構成が main より新しい(指紋が違う)こと自体が前方復旧の根拠
                return Ok(TipRelation::MainBehind(Box::new(tip_doc.clone())));
            }
            // 世代に無い main = 先行編集、または曖昧
            return Ok(TipRelation::KeepMain);
        }
    };

    if tip_fp == main_fp {
        return Ok(TipRelation::InSync);
    }

    let Some(tip_doc) = reconstructed else {
        return Ok(TipRelation::KeepMain);
    };
    let reconstructed_fp = document_fingerprint(&tip_doc)?;
    if reconstructed_fp != tip_fp {
        // tip を証明できない — 巻き戻さない
        return Ok(TipRelation::KeepMain);
    }

    if main_matches_generation(document_path, catalog, main_fp)? {
        return Ok(TipRelation::MainBehind(Box::new(tip_doc)));
    }

    Ok(TipRelation::KeepMain)
}

fn main_matches_generation(
    document_path: &Path,
    catalog: &GenerationCatalog,
    main_fp: u64,
) -> Result<bool, ProjectError> {
    for entry in &catalog.generations {
        if let Ok(snap) = load_generation_snapshot(document_path, entry.id) {
            if document_fingerprint(&snap)? == main_fp {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn generation_seq_for_fingerprint(
    document_path: &Path,
    catalog: &GenerationCatalog,
    fp: u64,
) -> Result<Option<u64>, ProjectError> {
    let mut best = None;
    for entry in &catalog.generations {
        if let Ok(snap) = load_generation_snapshot(document_path, entry.id) {
            if document_fingerprint(&snap)? == fp {
                best = Some(entry.created_seq);
            }
        }
    }
    Ok(best)
}

/// journal を初期 Document からリプレイし tip 状態を得る。
fn reconstruct_journal_tip(
    document_path: &Path,
    catalog: &GenerationCatalog,
    scan: &JournalScanOutcome,
) -> Result<Option<Document>, ProjectError> {
    if scan.frames.is_empty() {
        return Ok(base_from_latest_generation(document_path, catalog));
    }
    let replay = replay_journal(
        document_path,
        Document::new_v1(),
        scan,
        catalog,
        &ReplayOptions {
            fallback_on_failure: false,
        },
    );
    if replay.replay_failures.is_empty() {
        return Ok(Some(replay.document));
    }
    if let Some(tip_fp) = catalog.last_journaled_fingerprint {
        for entry in catalog.generations.iter().rev() {
            if let Ok(snap) = load_generation_snapshot(document_path, entry.id) {
                if document_fingerprint(&snap)? == tip_fp {
                    return Ok(Some(snap));
                }
            }
        }
    }
    Ok(base_from_latest_generation(document_path, catalog))
}

/// replay 結果が anchor より古い/フォールバック巻き戻しなら anchor を選ぶ。
fn prefer_anchor_against_rewind(
    document_path: &Path,
    catalog: &GenerationCatalog,
    anchor: Document,
    replay: ReplayOutcome,
    truncated: u64,
) -> Result<(Document, ReplayOutcome, bool), ProjectError> {
    let anchor_fp = document_fingerprint(&anchor)?;
    let replay_fp = document_fingerprint(&replay.document)?;

    let keep_anchor = if (!replay.replay_failures.is_empty() && replay_fp != anchor_fp)
        || (truncated > 0 && replay.document != anchor)
    {
        true
    } else if replay_fp != anchor_fp {
        match (
            generation_seq_for_fingerprint(document_path, catalog, replay_fp)?,
            generation_seq_for_fingerprint(document_path, catalog, anchor_fp)?,
        ) {
            (Some(r), Some(a)) => r < a,
            _ => false,
        }
    } else {
        false
    };

    if keep_anchor {
        Ok((anchor, replay, true))
    } else {
        Ok((replay.document.clone(), replay, false))
    }
}

fn open_with_main(
    document_path: &Path,
    base: Document,
) -> Result<OpenProjectOutcome, ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    if !journal_path.exists() {
        return Ok(OpenProjectOutcome {
            document: base,
            source: RecoverySource::MainFile,
            truncated_bytes: 0,
            replay: None,
        });
    }

    let (scan, truncated) = match heal_journal(document_path) {
        Ok(v) => v,
        Err(ProjectError::Journal(_)) => {
            return Ok(OpenProjectOutcome {
                document: base,
                source: RecoverySource::MainFile,
                truncated_bytes: 0,
                replay: None,
            });
        }
        Err(e) => return Err(e),
    };

    let (catalog, _) = resolve_catalog(document_path, scan.header.file_salt)?;
    let main_fp = document_fingerprint(&base)?;
    let relation = tip_relation(document_path, &catalog, main_fp, &scan)?;

    match relation {
        TipRelation::KeepMain => {
            return Ok(OpenProjectOutcome {
                document: base,
                source: RecoverySource::MainFile,
                truncated_bytes: truncated,
                replay: None,
            });
        }
        TipRelation::MainBehind(tip) => {
            return Ok(OpenProjectOutcome {
                document: *tip,
                source: RecoverySource::JournalReplay,
                truncated_bytes: truncated,
                replay: None,
            });
        }
        TipRelation::InSync => {}
    }

    if scan.frames.is_empty() {
        return Ok(OpenProjectOutcome {
            document: base,
            source: if truncated > 0 {
                RecoverySource::TruncatedJournalThenReplay
            } else {
                RecoverySource::MainFile
            },
            truncated_bytes: truncated,
            replay: None,
        });
    }

    let replay = replay_journal(
        document_path,
        base.clone(),
        &scan,
        &catalog,
        &ReplayOptions {
            fallback_on_failure: true,
        },
    );

    let (document, replay, kept_anchor) =
        prefer_anchor_against_rewind(document_path, &catalog, base, replay, truncated)?;
    if kept_anchor {
        return Ok(OpenProjectOutcome {
            document,
            source: RecoverySource::MainFile,
            truncated_bytes: truncated,
            replay: Some(replay),
        });
    }

    let source = classify_replay_source(&replay, truncated, &scan);
    Ok(OpenProjectOutcome {
        document,
        source,
        truncated_bytes: truncated,
        replay: Some(replay),
    })
}

fn catalog_salt_hint(document_path: &Path) -> u64 {
    load_catalog_lenient(document_path)
        .ok()
        .and_then(|(c, _)| c.map(|c| c.journal_salt))
        .unwrap_or(0)
}

fn open_without_main(document_path: &Path) -> Result<OpenProjectOutcome, ProjectError> {
    let journal_path = journal_path_for_document(document_path);

    // journal が無くても世代スナップショットから復元できる
    let healed = if journal_path.exists() {
        match heal_journal(document_path) {
            Ok(v) => Some(v),
            Err(ProjectError::Journal(_)) => None,
            Err(e) => return Err(e),
        }
    } else {
        None
    };

    let salt = healed
        .as_ref()
        .map(|(scan, _)| scan.header.file_salt)
        .unwrap_or_else(|| catalog_salt_hint(document_path));
    let (catalog, _) = resolve_catalog(document_path, salt)?;
    let Some(base) = base_from_latest_generation(document_path, &catalog) else {
        return Err(ProjectError::Unrecoverable {
            path: document_path.to_path_buf(),
        });
    };

    let Some((scan, truncated)) = healed else {
        return Ok(OpenProjectOutcome {
            document: base,
            source: RecoverySource::GenerationRecovery,
            truncated_bytes: 0,
            replay: None,
        });
    };

    if scan.frames.is_empty() {
        return Ok(OpenProjectOutcome {
            document: base,
            source: RecoverySource::GenerationRecovery,
            truncated_bytes: truncated,
            replay: None,
        });
    }

    let replay = replay_journal(
        document_path,
        base.clone(),
        &scan,
        &catalog,
        &ReplayOptions {
            fallback_on_failure: true,
        },
    );

    let (document, replay, _kept_anchor) =
        prefer_anchor_against_rewind(document_path, &catalog, base, replay, truncated)?;
    Ok(OpenProjectOutcome {
        document,
        source: RecoverySource::GenerationRecovery,
        truncated_bytes: truncated,
        replay: Some(replay),
    })
}

/// 本体 JSON + ジャーナルリプレイ + スナップショットフォールバックで復元する。
pub fn open_project(document_path: &Path) -> Result<OpenProjectOutcome, ProjectError> {
    match try_load_main(document_path)? {
        MainLoad::Ok(doc) => open_with_main(document_path, *doc),
        MainLoad::Missing => {
            // main 無しでも世代があれば復元を試みる
            open_without_main(document_path).or(Err(ProjectError::MissingMainDocument(
                document_path.to_path_buf(),
            )))
        }
        MainLoad::Corrupt => open_without_main(document_path),
    }
}

/// テスト注入: ジャーナル末尾に壊れたバイト列を付与する。
#[doc(hidden)]
pub fn inject_corrupt_journal_tail(
    document_path: &Path,
    garbage: &[u8],
) -> Result<(), ProjectError> {
    use std::io::Write;
    let journal_path = journal_path_for_document(document_path);
    let mut file = fs::OpenOptions::new().append(true).open(journal_path)?;
    file.write_all(garbage)?;
    file.sync_all()?;
    Ok(())
}

/// テスト注入: 特定フレームの checksum を破壊する。
#[doc(hidden)]
pub fn inject_bad_checksum_at_last_frame(document_path: &Path) -> Result<(), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let data = fs::read(&journal_path)?;
    let scan = super::format::scan_journal_bytes(&data, &ScanJournalOptions::default())?;
    if scan.frames.is_empty() {
        return Ok(());
    }
    let mut offset = HEADER_LEN;
    for (index, frame) in scan.frames.iter().enumerate() {
        if index + 1 == scan.frames.len() {
            let mut corrupted = data;
            corrupted[offset] ^= 0xFF;
            fs::write(journal_path, corrupted)?;
            return Ok(());
        }
        offset += encode_frame(frame).len();
    }
    Ok(())
}

/// テスト注入: 旧 salt のフレームを末尾に追記する(WAL 世代不一致)。
#[doc(hidden)]
pub fn inject_salt_mismatch_frame(document_path: &Path) -> Result<(), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let scan = scan_journal(&journal_path, &ScanJournalOptions::default())?;
    let bad_frame = JournalFrame {
        record_id: Uuid::new_v4(),
        prev_id: scan.frames.last().map(|f| f.record_id),
        snapshot_ref: None,
        record_salt: scan.header.file_salt.wrapping_add(1),
        kind: JournalRecordKind::Edit,
        payload: br#"{"op":"set_bpm","num":1,"den":1}"#.to_vec(),
    };
    append_frame(&journal_path, &bad_frame)?;
    Ok(())
}

/// テスト注入: catalog.json を壊す。
#[doc(hidden)]
pub fn inject_corrupt_catalog(document_path: &Path) -> Result<(), ProjectError> {
    use super::catalog::catalog_path_for_document;
    let path = catalog_path_for_document(document_path);
    fs::write(path, b"{not-valid-json")?;
    Ok(())
}

/// テスト注入: main JSON を壊す。
#[doc(hidden)]
pub fn inject_corrupt_main(document_path: &Path) -> Result<(), ProjectError> {
    fs::write(document_path, b"{broken main")?;
    Ok(())
}

/// テスト注入: tip 指紋を消して「不明 tip」状態にする。
#[doc(hidden)]
pub fn inject_clear_fingerprint(document_path: &Path) -> Result<(), ProjectError> {
    let mut catalog =
        load_catalog_lenient(document_path)?
            .0
            .ok_or_else(|| ProjectError::Unrecoverable {
                path: document_path.to_path_buf(),
            })?;
    catalog.last_journaled_fingerprint = None;
    save_catalog(document_path, &catalog)?;
    Ok(())
}

/// テスト注入: 先頭 Snapshot の直後で欠落世代 Snapshot を挟み、後続を落とす。
///
/// corrupt main 経路で「世代 base より古い fallback」を起こすため。
#[doc(hidden)]
pub fn inject_orphan_snapshot_after_first_frame(document_path: &Path) -> Result<(), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let data = fs::read(&journal_path)?;
    let scan = super::format::scan_journal_bytes(&data, &ScanJournalOptions::default())?;
    let Some(first) = scan.frames.first() else {
        return Ok(());
    };
    let missing = Uuid::new_v4();
    let orphan = JournalFrame {
        record_id: Uuid::new_v4(),
        prev_id: Some(first.record_id),
        snapshot_ref: Some(missing),
        record_salt: scan.header.file_salt,
        kind: JournalRecordKind::Snapshot,
        payload: snapshot_payload(missing),
    };
    let mut out = encode_header(&scan.header).to_vec();
    out.extend_from_slice(&encode_frame(first));
    out.extend_from_slice(&encode_frame(&orphan));
    fs::write(journal_path, out)?;
    Ok(())
}

#[cfg(test)]
mod project_tests {
    use super::*;
    use crate::{Bpm, Document, JournalEdit, SaveProjectOptions};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-d1d-unit-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn clean_journal_scans_without_truncation() {
        let dir = dir();
        let path = dir.join("doc.json");
        let mut doc = Document::new_v1();
        save_project_with_journal(
            &path,
            &doc,
            &SaveProjectOptions {
                snapshot_every_n_edits: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        doc.bpm = Bpm::try_new(150, 1).unwrap();
        save_project_with_journal(
            &path,
            &doc,
            &SaveProjectOptions {
                journal_edit: Some(JournalEdit::SetBpm { num: 150, den: 1 }),
                snapshot_every_n_edits: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        let journal_path = journal_path_for_document(&path);
        let len = fs::metadata(&journal_path).unwrap().len();
        let scan = scan_journal(&journal_path, &ScanJournalOptions::default()).unwrap();
        assert_eq!(scan.stopped, None, "unexpected stop: {:?}", scan.stopped);
        assert_eq!(scan.valid_bytes, len);
        let _ = fs::remove_dir_all(dir);
    }
}
