use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::limits::ResourceLimitError;
use crate::persist::{PersistError, WRITER_VERSION};
use crate::{Document, DocumentError};

/// 現行スキーマへ揃えたあとの文書版(=書込能力)。
pub const LATEST_DOCUMENT_VERSION: u32 = WRITER_VERSION;

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
    pub warnings: Vec<&'static str>,
}

impl MigrationReport {
    pub(super) fn identity(version: u32) -> Self {
        Self {
            from_version: version,
            to_version: version,
            steps: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn did_migrate(&self) -> bool {
        !self.steps.is_empty() || self.from_version != self.to_version
    }
}

#[derive(Debug, Error)]
pub enum MigrateError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Validate(#[from] DocumentError),
    #[error(transparent)]
    ResourceLimit(#[from] ResourceLimitError),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error("unsupported document version {0}")]
    UnsupportedVersion(u32),
    #[error(
        "migration invariant violated: tracks {before_tracks}->{after_tracks}, \
         clips {before_clips}->{after_clips}, keys {before_keys}->{after_keys}"
    )]
    InvariantViolation {
        before_tracks: usize,
        before_clips: usize,
        before_keys: usize,
        after_tracks: usize,
        after_clips: usize,
        after_keys: usize,
    },
    /// 既存バックアップは上書きしない(最後の既知良品を守る)。
    #[error("backup already exists at {0}")]
    BackupExists(PathBuf),
    #[error("legacy path_ops on non-vector source at {path}: {detail}")]
    PathOpsOnRaster { path: String, detail: String },
    #[error("legacy path_ops migration failed at {path}: {detail}")]
    PathOpsRewrite { path: String, detail: String },
    #[error("legacy TimeMap migration failed at {path}: {detail}")]
    TimeMapRewrite { path: String, detail: String },
    #[error("stable id injection failed: {0}")]
    StableId(String),
    #[error("hybrid effect entry has both definition_id and inline definition fields at {path}")]
    HybridEffectEntry { path: String },
    #[error(
        "document version {version} must not carry composition.camera; migration inserts default camera (D1j)"
    )]
    DisguisedCompCamera { version: u32 },
    #[error("composition.camera migration failed: {0}")]
    CompCameraMigration(String),
    #[error("document root must be a JSON object")]
    NotAnObject,
}

#[derive(Debug, Clone, Default)]
pub struct MigrateFileOptions {
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateFileResult {
    pub backup_path: PathBuf,
    pub report: MigrationReport,
    pub migrated: bool,
}

/// 意味保存比較用の指紋(監査S12)。件数一致だけでは通さない。
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticFingerprint {
    /// `(layer_id, param_path, Debug(Value))` at sample times.
    pub param_evals: Vec<(u64, &'static str, String)>,
    /// `(from_layer, kind, to_layer)` — parent / LookAt / Follow。
    pub dependency_edges: BTreeSet<(u64, &'static str, u64)>,
    /// `(layer_id, clip_local_debug, source_debug)` TimeMap samples。
    pub timemap_samples: Vec<(u64, String, String)>,
}

pub fn bump_min_reader_for_nest_schema_change(doc: &mut Document, required_reader: u32) {
    doc.min_reader_version = doc.min_reader_version.max(required_reader);
    doc.version = doc.version.max(required_reader);
}
