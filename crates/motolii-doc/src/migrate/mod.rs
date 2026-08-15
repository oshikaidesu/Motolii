//! D1e: ドキュメント版マイグレーション(ガード8 / 監査S12・S14)。
//!
//! - **load経路は旧形式を拒否したまま**(D1g/D1i-1)。変換は本モジュールの明示APIのみ。
//! - in-place禁止。ファイル書換前に `.motolii-pre-migrate.bak` を作る(既存bakは上書きしない)。
//! - #101 `OpenMode` / `ResourceLimits` を消費し、別ロード経路を作らない。

mod count;
mod file;
mod fingerprint;
mod rewrite;
mod stable_ids;
mod types;

pub use count::count_document;
pub use fingerprint::{legacy_timemap_source, modern_timemap_source, semantic_fingerprint};
pub use types::{
    bump_min_reader_for_nest_schema_change, DocumentCounts, MigrateError, MigrateFileOptions,
    MigrateFileResult, MigrationReport, SemanticFingerprint, BACKUP_SUFFIX,
    LATEST_DOCUMENT_VERSION,
};

pub(crate) use file::migrate_document_file_with_limits;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::limits::{check_document_resource_limits, ResourceLimits};
use crate::persist::{
    check_migration_allowed, classify_open_mode, OpenMode, PersistError, READER_VERSION,
    WRITER_VERSION,
};
use crate::schema::CompCameraDoc;
use crate::validate::MIN_READER_VERSION_FOR_COMP_CAMERA;
use crate::Document;

use count::{assert_counts_preserved, count_json_document};
use rewrite::rewrite_legacy_shapes;
use stable_ids::{doc_has_stable_ids, inject_missing_stable_ids_json};

#[derive(Debug, Deserialize)]
struct VersionHeader {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default = "default_min_reader")]
    min_reader_version: u32,
}

fn default_version() -> u32 {
    1
}

fn default_min_reader() -> u32 {
    1
}

fn guard_open_mode_for_migration(version: u32, min_reader: u32) -> Result<(), PersistError> {
    match classify_open_mode(version, min_reader) {
        OpenMode::ReadWrite => Ok(()),
        OpenMode::ReadOnlyNewer => Err(PersistError::SaveRejectedReadOnlyNewer {
            document_version: version,
            writer_version: WRITER_VERSION,
        }),
        OpenMode::Reject => Err(PersistError::ReaderTooOld {
            min_reader_version: min_reader,
            reader_version: READER_VERSION,
        }),
    }
}

pub fn migrate_bytes(bytes: &[u8]) -> Result<(Document, MigrationReport), MigrateError> {
    migrate_bytes_with_limits(bytes, &ResourceLimits::production())
}

/// #101の同じ`ResourceLimits`を通す。別ロード経路を作らない。
pub fn migrate_bytes_with_limits(
    bytes: &[u8],
    limits: &ResourceLimits,
) -> Result<(Document, MigrationReport), MigrateError> {
    limits.check_file_bytes(bytes.len() as u64)?;
    let header: VersionHeader = serde_json::from_slice(bytes)?;
    if header.version > LATEST_DOCUMENT_VERSION + 64 {
        // 極端な未来版はUnsupported。通常のReadOnlyNewerはOpenModeで拒否。
        return Err(MigrateError::UnsupportedVersion(header.version));
    }
    guard_open_mode_for_migration(header.version, header.min_reader_version)?;

    let mut root: Value = serde_json::from_slice(bytes)?;
    let Value::Object(_) = &root else {
        return Err(MigrateError::NotAnObject);
    };

    let before_counts = count_json_document(&root);
    let mut steps = Vec::new();
    let from_version = header.version;

    rewrite_legacy_shapes(&mut root, &mut steps)?;

    // D1l: 欠落stable IDの採番とカウンタ正規化を先に行い、共有空間を確定してから
    // inline EffectInstanceを EffectUse+EffectDefinition へ分離する。
    if inject_missing_stable_ids_json(&mut root)? {
        steps.push("inject_stable_ids");
    }

    if crate::legacy_effect_migrate::migrate_inline_effects_json(&mut root)? {
        steps.push("inline_effects_to_definition_use");
    }

    migrate_comp_camera_json(&mut root, from_version, &mut steps)?;

    // 変換後JSONを現行Documentへ。ResourceLimitsはdeserialize後に再検査。
    let mut doc: Document = serde_json::from_value(root)?;
    check_document_resource_limits(&doc, limits)?;

    let after_rewrite = count_document(&doc);
    // Twist.center注入でConstキーは増えない。旧Wiggle.seedがKeyframesだった場合だけ差が出る —
    // そのときseedキーは意味上seed:u64へ落ちるので件数減少を許容しない(拒否)。
    assert_counts_preserved(before_counts, after_rewrite)?;

    if steps.contains(&"inject_stable_ids")
        || steps.contains(&"inline_effects_to_definition_use")
        || steps.contains(&"insert_default_comp_camera")
    {
        bump_min_reader_for_nest_schema_change(&mut doc, LATEST_DOCUMENT_VERSION);
    } else if doc_has_stable_ids(&doc) && doc.min_reader_version < LATEST_DOCUMENT_VERSION {
        // 既にidを持つ旧JSONでもvalidateのmin_reader下限を満たす。すでに下限を満たす
        // 文書(=再migrateのidempotent経路)ではstepを積まない — 差分ゼロをdid_migrate()に
        // 正しく反映するため。
        bump_min_reader_for_nest_schema_change(&mut doc, LATEST_DOCUMENT_VERSION);
        if !steps.contains(&"bump_min_reader_for_stable_ids") {
            steps.push("bump_min_reader_for_stable_ids");
        }
    }

    let to_version = doc.version;
    doc.validate()?;
    // 書戻し可能であることをOpenModeでも再確認(stable id昇格後)。
    check_migration_allowed(&doc)?;

    let report = if steps.is_empty() && from_version == to_version {
        MigrationReport::identity(from_version)
    } else {
        MigrationReport {
            from_version,
            to_version,
            steps,
            warnings: Vec::new(),
        }
    };
    Ok((doc, report))
}

fn default_comp_camera_json() -> Value {
    serde_json::to_value(CompCameraDoc::default_planar_orthographic())
        .expect("default planar camera serializes")
}

/// v1–v4で`composition.camera`欠落時のみ既定cameraをJSON挿入し、版を5へ上げる。
fn migrate_comp_camera_json(
    root: &mut Value,
    from_version: u32,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    if from_version > MIN_READER_VERSION_FOR_COMP_CAMERA - 1 {
        return Ok(());
    }
    let Value::Object(map) = root else {
        return Err(MigrateError::NotAnObject);
    };
    let composition = map
        .get_mut("composition")
        .ok_or_else(|| MigrateError::CompCameraMigration("composition missing".into()))?;
    let Value::Object(comp_map) = composition else {
        return Err(MigrateError::CompCameraMigration(
            "composition must be object".into(),
        ));
    };
    if comp_map.contains_key("camera") {
        return Err(MigrateError::DisguisedCompCamera {
            version: from_version,
        });
    }
    comp_map.insert("camera".into(), default_comp_camera_json());
    map.insert("version".into(), json!(MIN_READER_VERSION_FOR_COMP_CAMERA));
    let min = map
        .get("min_reader_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;
    map.insert(
        "min_reader_version".into(),
        json!(min.max(MIN_READER_VERSION_FOR_COMP_CAMERA)),
    );
    if !steps.contains(&"insert_default_comp_camera") {
        steps.push("insert_default_comp_camera");
    }
    Ok(())
}
