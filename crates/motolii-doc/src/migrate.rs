//! D1e: ドキュメント版マイグレーション(ガード8)。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use motolii_core::TimeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::param::DocParam;
use crate::schema::{ClipSource, Group, PathOp, TrackItem};
use crate::{Document, DocumentError};

pub const LATEST_DOCUMENT_VERSION: u32 = 2;
pub const BACKUP_SUFFIX: &str = ".motolii-pre-migrate.bak";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentCounts {
    pub track_count: usize,
    pub clip_count: usize,
    pub keyframe_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub from_version: u32,
    pub to_version: u32,
    pub steps: Vec<&'static str>,
}

impl MigrationReport {
    fn identity(version: u32) -> Self {
        Self { from_version: version, to_version: version, steps: Vec::new() }
    }
}

#[derive(Debug, Error)]
pub enum MigrateError {
    #[error(transparent)] Io(#[from] io::Error),
    #[error(transparent)] Json(#[from] serde_json::Error),
    #[error(transparent)] Validate(#[from] DocumentError),
    #[error("unsupported document version {0}")] UnsupportedVersion(u32),
    #[error("migration invariant violated: tracks {before_tracks}->{after_tracks}, clips {before_clips}->{after_clips}, keys {before_keys}->{after_keys}")]
    InvariantViolation { before_tracks: usize, before_clips: usize, before_keys: usize, after_tracks: usize, after_clips: usize, after_keys: usize },
    #[error("backup already exists at {0}")] BackupExists(PathBuf),
}

#[derive(Debug, Clone, Default)]
pub struct MigrateFileOptions { pub dry_run: bool }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateFileResult { pub backup_path: PathBuf, pub report: MigrationReport, pub migrated: bool }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorInterpretation { #[default] StraightSrgb }

#[derive(Debug, Deserialize)]
struct PreludeDocument {
    version: u32,
    #[serde(default = "default_min_reader")] min_reader_version: u32,
    time_map: TimeMap,
    #[serde(default)] extra: serde_json::Map<String, Value>,
}
fn default_min_reader() -> u32 { 1 }

pub fn count_document(doc: &Document) -> DocumentCounts {
    let mut clip_count = 0usize;
    let mut keyframe_count = 0usize;
    for track in &doc.tracks {
        for item in &track.items { count_item(item, &mut clip_count, &mut keyframe_count); }
    }
    DocumentCounts { track_count: doc.tracks.len(), clip_count, keyframe_count }
}

fn count_item(item: &TrackItem, clips: &mut usize, keys: &mut usize) {
    match item {
        TrackItem::Clip(clip) => {
            *clips += 1;
            count_envelope(&clip.envelope, keys);
            for op in &clip.path_ops { count_path_op(op, keys); }
            if let ClipSource::Plugin { params, .. } = &clip.source {
                for param in params.values() { count_param(param, keys); }
            }
        }
        TrackItem::Group(group) => count_group(group, clips, keys),
    }
}
fn count_group(group: &Group, clips: &mut usize, keys: &mut usize) {
    count_envelope(&group.envelope, keys);
    for child in &group.children { count_item(child, clips, keys); }
}
fn count_envelope(env: &crate::schema::ItemEnvelope, keys: &mut usize) {
    count_param(&env.transform.position, keys);
    count_param(&env.transform.anchor, keys);
    count_param(&env.transform.scale, keys);
    count_param(&env.transform.rotation, keys);
    count_param(&env.opacity, keys);
    for effect in &env.effects { for param in effect.params.values() { count_param(param, keys); } }
}
fn count_path_op(op: &PathOp, keys: &mut usize) {
    use PathOp::*;
    match op {
        PuckerBloat { amount } => count_param(amount, keys),
        ZigZag { amount, ridges } => { count_param(amount, keys); count_param(ridges, keys); }
        Offset { distance } => count_param(distance, keys),
        RoundCorners { radius } => count_param(radius, keys),
        Trim { start, end, offset } => { count_param(start, keys); count_param(end, keys); count_param(offset, keys); }
        Twist { angle } => count_param(angle, keys),
        Wiggle { amp, freq, seed } => { count_param(amp, keys); count_param(freq, keys); count_param(seed, keys); }
        Repeater { copies, offset } => { count_param(copies, keys); count_param(offset, keys); }
    }
}
fn count_param(param: &DocParam, keys: &mut usize) {
    match param {
        DocParam::Keyframes(track) => *keys += track.keys().len(),
        DocParam::Vec2Axes { x, y } => { count_param(x, keys); count_param(y, keys); }
        _ => {}
    }
}
fn assert_counts_preserved(before: DocumentCounts, after: DocumentCounts) -> Result<(), MigrateError> {
    if before == after { Ok(()) } else {
        Err(MigrateError::InvariantViolation {
            before_tracks: before.track_count, before_clips: before.clip_count, before_keys: before.keyframe_count,
            after_tracks: after.track_count, after_clips: after.clip_count, after_keys: after.keyframe_count,
        })
    }
}
fn is_prelude_format(value: &Value) -> bool { value.get("composition").is_none() && value.get("time_map").is_some() }
fn document_version(value: &Value) -> u32 { value.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32 }

pub fn migrate_bytes(bytes: &[u8]) -> Result<(Document, MigrationReport), MigrateError> {
    let value: Value = serde_json::from_slice(bytes)?;
    if is_prelude_format(&value) { return migrate_prelude_bytes(bytes); }
    let from_version = document_version(&value);
    if from_version > LATEST_DOCUMENT_VERSION { return Err(MigrateError::UnsupportedVersion(from_version)); }
    if from_version == LATEST_DOCUMENT_VERSION {
        let doc: Document = serde_json::from_slice(bytes)?;
        doc.validate()?;
        return Ok((doc, MigrationReport::identity(from_version)));
    }
    let mut doc: Document = serde_json::from_slice(bytes)?;
    let mut steps = Vec::new();
    let before = count_document(&doc);
    let mut version = from_version;
    while version < LATEST_DOCUMENT_VERSION {
        let step_before = count_document(&doc);
        doc = match version {
            1 => { steps.push("v1_to_v2_color_interpretation"); migrate_v1_to_v2(doc)? }
            other => return Err(MigrateError::UnsupportedVersion(other)),
        };
        assert_counts_preserved(step_before, count_document(&doc))?;
        version += 1;
    }
    assert_counts_preserved(before, count_document(&doc))?;
    doc.validate()?;
    Ok((doc, MigrationReport { from_version, to_version: LATEST_DOCUMENT_VERSION, steps }))
}

fn migrate_prelude_bytes(bytes: &[u8]) -> Result<(Document, MigrationReport), MigrateError> {
    let prelude: PreludeDocument = serde_json::from_slice(bytes)?;
    let before = DocumentCounts { track_count: 0, clip_count: 0, keyframe_count: 0 };
    let mut doc = Document::new_v1();
    doc.version = prelude.version.max(1);
    doc.min_reader_version = prelude.min_reader_version;
    doc.extra = prelude.extra;
    if prelude.time_map != TimeMap::identity() {
        doc.extra.insert("_migrated_prelude_time_map".into(), serde_json::to_value(prelude.time_map)?);
    }
    let mut steps = vec!["prelude_to_d1a"];
    assert_counts_preserved(before, count_document(&doc))?;
    if doc.version < LATEST_DOCUMENT_VERSION {
        let step_before = count_document(&doc);
        doc = migrate_v1_to_v2(doc)?;
        steps.push("v1_to_v2_color_interpretation");
        assert_counts_preserved(step_before, count_document(&doc))?;
    }
    doc.validate()?;
    Ok((doc, MigrationReport { from_version: 0, to_version: LATEST_DOCUMENT_VERSION, steps }))
}

fn migrate_v1_to_v2(mut doc: Document) -> Result<Document, MigrateError> {
    doc.version = 2;
    doc.color_interpretation = ColorInterpretation::StraightSrgb;
    Ok(doc)
}

pub fn bump_min_reader_for_nest_schema_change(doc: &mut Document, required_reader: u32) {
    doc.min_reader_version = doc.min_reader_version.max(required_reader);
}

pub fn migrate_document_file(path: &Path, options: &MigrateFileOptions) -> Result<MigrateFileResult, MigrateError> {
    let bytes = fs::read(path)?;
    let (doc, report) = migrate_bytes(&bytes)?;
    let migrated = report.from_version != report.to_version || report.steps.iter().any(|s| s.contains("prelude"));
    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("document.json");
    let backup_path = path.with_file_name(format!("{file_name}{BACKUP_SUFFIX}"));
    if backup_path.exists() { return Err(MigrateError::BackupExists(backup_path)); }
    fs::copy(path, &backup_path)?;
    if !options.dry_run && migrated {
        crate::save_document(path, &doc).map_err(|e| match e {
            crate::PersistError::Validate(v) => MigrateError::Validate(v),
            crate::PersistError::Io(i) => MigrateError::Io(i),
            crate::PersistError::Json(j) => MigrateError::Json(j),
            crate::PersistError::Migrate(m) => *m,
            crate::PersistError::ReaderTooOld { .. } | crate::PersistError::Aborted { .. } => {
                MigrateError::Io(io::Error::other(e.to_string()))
            }
        })?;
    }
    Ok(MigrateFileResult { backup_path, report, migrated })
}
