//! D1e: ドキュメント版マイグレーション(ガード8 / 監査S12・S14)。
//!
//! - **load経路は旧形式を拒否したまま**(D1g/D1i-1)。変換は本モジュールの明示APIのみ。
//! - in-place禁止。ファイル書換前に `.motolii-pre-migrate.bak` を作る(既存bakは上書きしない)。
//! - #101 `OpenMode` / `ResourceLimits` を消費し、別ロード経路を作らない。

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use motolii_core::{RationalTime, TimeMap};
use motolii_eval::DataTracks;
use serde::Deserialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::limits::{check_document_resource_limits, ResourceLimitError, ResourceLimits};
use crate::param::DocParam;
use crate::param_eval::{eval_doc_param, ResolvedLayerParams};
use crate::persist::{
    check_migration_allowed, classify_open_mode, save_document, OpenMode, PersistError,
    READER_VERSION, WRITER_VERSION,
};
use crate::schema::{
    Clip, ClipSource, CompCameraDoc, Group, PathOp, TrackItem, VectorContent, VectorRecipe,
};
use crate::validate::MIN_READER_VERSION_FOR_COMP_CAMERA;
use crate::{Document, DocumentError};

/// 現行スキーマへ揃えたあとの文書版(=書込能力)。
pub const LATEST_DOCUMENT_VERSION: u32 = WRITER_VERSION;

pub const BACKUP_SUFFIX: &str = ".motolii-pre-migrate.bak";

const SVG_ASSET_TYPE: &str = "image/svg+xml";

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
    fn identity(version: u32) -> Self {
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

/// ネストスキーマ追加時に`min_reader_version`を上げる前方互換口。
pub fn bump_min_reader_for_nest_schema_change(doc: &mut Document, required_reader: u32) {
    doc.min_reader_version = doc.min_reader_version.max(required_reader);
    doc.version = doc.version.max(required_reader);
}

pub fn count_document(doc: &Document) -> DocumentCounts {
    let mut clip_count = 0usize;
    let mut keyframe_count = 0usize;
    for track in &doc.tracks {
        for item in &track.items {
            count_item(item, &mut clip_count, &mut keyframe_count);
        }
    }
    // D1l: effect paramsはUseではなくDefinition台帳が持つ(1回だけ数える。共有でも重複しない)。
    for def in &doc.effect_definitions {
        for param in def.params.values() {
            count_param(param, &mut keyframe_count);
        }
    }
    count_comp_camera(&doc.composition.camera, &mut keyframe_count);
    DocumentCounts {
        track_count: doc.tracks.len(),
        clip_count,
        keyframe_count,
    }
}

fn count_item(item: &TrackItem, clips: &mut usize, keys: &mut usize) {
    match item {
        TrackItem::Clip(clip) => {
            *clips += 1;
            count_envelope(&clip.envelope, keys);
            if let ClipSource::Vector { recipe } = &clip.source {
                count_vector_recipe(recipe, keys);
            }
            if let ClipSource::Plugin { params, .. } = &clip.source {
                for param in params.values() {
                    count_param(param, keys);
                }
            }
            if let ClipSource::Asset { audio, .. } = &clip.source {
                for comp in audio {
                    count_param(&comp.gain, keys);
                }
            }
        }
        TrackItem::Group(group) => count_group(group, clips, keys),
    }
}

fn count_group(group: &Group, clips: &mut usize, keys: &mut usize) {
    count_envelope(&group.envelope, keys);
    for child in &group.children {
        count_item(child, clips, keys);
    }
}

fn count_envelope(env: &crate::schema::ItemEnvelope, keys: &mut usize) {
    count_param(&env.transform.position, keys);
    count_param(&env.transform.anchor, keys);
    count_param(&env.transform.scale, keys);
    count_param(&env.transform.rotation, keys);
    count_param(&env.opacity, keys);
    // D1l: EffectUseはid参照のみ。paramsは`count_document`側でDefinition台帳を1回だけ数える。
}

fn count_vector_recipe(recipe: &VectorRecipe, keys: &mut usize) {
    count_vector_content(&recipe.content, keys);
    for op in &recipe.modifiers {
        count_path_op(op, keys);
    }
}

fn count_vector_content(content: &VectorContent, keys: &mut usize) {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            crate::schema::StandardShape::Rect { width, height }
            | crate::schema::StandardShape::Ellipse { width, height } => {
                count_param(width, keys);
                count_param(height, keys);
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => {}
        VectorContent::Group { children } => {
            for child in children {
                count_vector_content(child, keys);
            }
        }
    }
}

fn count_path_op(op: &PathOp, keys: &mut usize) {
    match op {
        PathOp::PuckerBloat { amount } => count_param(amount, keys),
        PathOp::ZigZag {
            amount,
            ridges,
            point_type: _,
        } => {
            count_param(amount, keys);
            count_param(ridges, keys);
        }
        PathOp::Offset {
            distance,
            line_join: _,
            miter_limit: _,
        } => count_param(distance, keys),
        PathOp::RoundCorners { radius } => count_param(radius, keys),
        PathOp::Trim {
            start,
            end,
            offset,
            mode: _,
        } => {
            count_param(start, keys);
            count_param(end, keys);
            count_param(offset, keys);
        }
        PathOp::Twist { angle, center } => {
            count_param(angle, keys);
            count_param(center, keys);
        }
        PathOp::Wiggle { amp, freq, seed: _ } => {
            count_param(amp, keys);
            count_param(freq, keys);
        }
        PathOp::Repeater {
            copies,
            offset,
            transform,
            composite: _,
            start_opacity,
            end_opacity,
        } => {
            count_param(copies, keys);
            count_param(offset, keys);
            count_param(&transform.position, keys);
            count_param(&transform.anchor, keys);
            count_param(&transform.scale, keys);
            count_param(&transform.rotation, keys);
            count_param(start_opacity, keys);
            count_param(end_opacity, keys);
        }
    }
}

fn count_param(param: &DocParam, keys: &mut usize) {
    match param {
        DocParam::Keyframes(track) => *keys += track.keys().len(),
        DocParam::Vec2Axes { x, y } => {
            count_param(x, keys);
            count_param(y, keys);
        }
        _ => {}
    }
}

fn count_comp_camera(camera: &CompCameraDoc, keys: &mut usize) {
    match camera {
        CompCameraDoc::PlanarOrthographic {
            center,
            roll_radians,
            height,
        } => {
            count_param(center, keys);
            count_param(roll_radians, keys);
            count_param(height, keys);
        }
    }
}

fn assert_counts_preserved(
    before: DocumentCounts,
    after: DocumentCounts,
) -> Result<(), MigrateError> {
    if before == after {
        Ok(())
    } else {
        Err(MigrateError::InvariantViolation {
            before_tracks: before.track_count,
            before_clips: before.clip_count,
            before_keys: before.keyframe_count,
            after_tracks: after.track_count,
            after_clips: after.clip_count,
            after_keys: after.keyframe_count,
        })
    }
}

/// JSON走査の量的カウント(変換前の不変条件用。serde拒否前に測る)。
fn count_json_document(root: &Value) -> DocumentCounts {
    let mut clip_count = 0usize;
    let mut keyframe_count = 0usize;
    let tracks = root
        .get("tracks")
        .and_then(|t| t.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    for track in tracks {
        if let Some(items) = track.get("items").and_then(|i| i.as_array()) {
            for item in items {
                count_json_item(item, &mut clip_count, &mut keyframe_count);
            }
        }
    }
    // D1l: 旧inline effect.paramsはenvelope側で数える(count_json_envelope)。
    // 既に新形式(root.effect_definitions)のドキュメントはこちらで1回だけ数える。
    if let Some(defs) = root.get("effect_definitions").and_then(|d| d.as_array()) {
        for def in defs {
            if let Some(params) = def.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), &mut keyframe_count);
                }
            }
        }
    }
    if let Some(camera) = root
        .get("composition")
        .and_then(|c| c.get("camera"))
        .and_then(|c| c.as_object())
    {
        for key in ["center", "roll_radians", "height"] {
            count_json_param(camera.get(key), &mut keyframe_count);
        }
    }
    DocumentCounts {
        track_count: tracks.len(),
        clip_count,
        keyframe_count,
    }
}

fn count_json_item(item: &Value, clips: &mut usize, keys: &mut usize) {
    let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "clip" => {
            *clips += 1;
            count_json_envelope(item.get("envelope"), keys);
            if let Some(ops) = item.get("path_ops").and_then(|v| v.as_array()) {
                for op in ops {
                    count_json_path_op(op, keys);
                }
            }
            if let Some(source) = item.get("source") {
                count_json_source(source, keys);
            }
        }
        "group" => {
            count_json_envelope(item.get("envelope"), keys);
            if let Some(children) = item.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    count_json_item(child, clips, keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_envelope(envelope: Option<&Value>, keys: &mut usize) {
    let Some(env) = envelope else {
        return;
    };
    if let Some(xf) = env.get("transform") {
        count_json_param(xf.get("position"), keys);
        count_json_param(xf.get("anchor"), keys);
        count_json_param(xf.get("scale"), keys);
        count_json_param(xf.get("rotation"), keys);
    }
    count_json_param(env.get("opacity"), keys);
    if let Some(effects) = env.get("effects").and_then(|e| e.as_array()) {
        for effect in effects {
            if let Some(params) = effect.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), keys);
                }
            }
        }
    }
}

fn count_json_source(source: &Value, keys: &mut usize) {
    let tag = source.get("source").and_then(|s| s.as_str()).unwrap_or("");
    match tag {
        "plugin" => {
            if let Some(params) = source.get("params").and_then(|p| p.as_object()) {
                for param in params.values() {
                    count_json_param(Some(param), keys);
                }
            }
        }
        "vector" => {
            if let Some(recipe) = source.get("recipe") {
                count_json_vector_content(recipe.get("content"), keys);
                if let Some(mods) = recipe.get("modifiers").and_then(|m| m.as_array()) {
                    for op in mods {
                        count_json_path_op(op, keys);
                    }
                }
            }
        }
        "asset" => {
            if let Some(audio) = source.get("audio").and_then(|a| a.as_array()) {
                for comp in audio {
                    count_json_param(comp.get("gain"), keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_vector_content(content: Option<&Value>, keys: &mut usize) {
    let Some(c) = content else {
        return;
    };
    match c.get("kind").and_then(|k| k.as_str()).unwrap_or("") {
        "standard_shape" => {
            count_json_param(c.get("width"), keys);
            count_json_param(c.get("height"), keys);
        }
        "group" => {
            if let Some(children) = c.get("children").and_then(|ch| ch.as_array()) {
                for child in children {
                    count_json_vector_content(Some(child), keys);
                }
            }
        }
        _ => {}
    }
}

fn count_json_path_op(op: &Value, keys: &mut usize) {
    // 旧Twistはcenter無し。新形式はcenterあり — 件数はキーフレーム数のみ。
    for key in [
        "amount",
        "ridges",
        "distance",
        "radius",
        "start",
        "end",
        "offset",
        "angle",
        "center",
        "amp",
        "freq",
        "copies",
        "start_opacity",
        "end_opacity",
    ] {
        count_json_param(op.get(key), keys);
    }
    if let Some(xf) = op.get("transform") {
        count_json_param(xf.get("position"), keys);
        count_json_param(xf.get("anchor"), keys);
        count_json_param(xf.get("scale"), keys);
        count_json_param(xf.get("rotation"), keys);
    }
    // 旧Wiggle.seedがDocParam(Keyframes)だった場合のみキー数に入る。
    if op.get("seed").and_then(|s| s.as_object()).is_some() {
        count_json_param(op.get("seed"), keys);
    }
}

fn count_json_param(param: Option<&Value>, keys: &mut usize) {
    let Some(p) = param else {
        return;
    };
    if let Some(kf) = p.get("keyframes") {
        if let Some(arr) = kf.get("keys").and_then(|k| k.as_array()) {
            *keys += arr.len();
        }
    } else if p.get("x").is_some() || p.get("y").is_some() {
        // Vec2Axes
        count_json_param(p.get("x"), keys);
        count_json_param(p.get("y"), keys);
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

fn rewrite_legacy_shapes(
    root: &mut Value,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let asset_types = collect_asset_types(root);
    let Some(tracks) = root.get_mut("tracks").and_then(|t| t.as_array_mut()) else {
        return Ok(());
    };
    for (ti, track) in tracks.iter_mut().enumerate() {
        let Some(items) = track.get_mut("items").and_then(|i| i.as_array_mut()) else {
            continue;
        };
        for (ii, item) in items.iter_mut().enumerate() {
            rewrite_item(
                item,
                &format!("tracks[{ti}].items[{ii}]"),
                &asset_types,
                steps,
            )?;
        }
    }
    Ok(())
}

fn collect_asset_types(root: &Value) -> BTreeMap<u64, String> {
    let mut out = BTreeMap::new();
    let Some(entries) = root
        .get("assets")
        .and_then(|a| a.get("entries"))
        .and_then(|e| e.as_array())
    else {
        return out;
    };
    for entry in entries {
        let Some(id) = entry.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        if let Some(ty) = entry.get("asset_type").and_then(|v| v.as_str()) {
            out.insert(id, ty.to_string());
        }
    }
    out
}

fn rewrite_item(
    item: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let kind = item
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "clip" => rewrite_clip(item, path, asset_types, steps),
        "group" => {
            if let Some(children) = item.get_mut("children").and_then(|c| c.as_array_mut()) {
                for (ci, child) in children.iter_mut().enumerate() {
                    rewrite_item(child, &format!("{path}.children[{ci}]"), asset_types, steps)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn rewrite_clip(
    clip: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let clip_start = clip.get("start").cloned();
    if let Some(tm) = clip.get_mut("time_map") {
        rewrite_timemap(tm, path, &clip_start, steps)?;
    }
    rewrite_path_ops(clip, path, asset_types, steps)?;
    Ok(())
}

fn rewrite_timemap(
    tm: &mut Value,
    path: &str,
    clip_start: &Option<Value>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let Value::Object(map) = tm else {
        return Ok(());
    };
    if !map.contains_key("timeline_start") {
        return Ok(());
    }
    let timeline_start_val =
        map.remove("timeline_start")
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "timeline_start missing after check".into(),
            })?;

    // 現行TimeMapは source_start/speed 必須。欠ければ補正も写像保存もできない。
    if !map.contains_key("source_start")
        || !map.contains_key("speed_num")
        || !map.contains_key("speed_den")
    {
        return Err(MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: "legacy TimeMap missing source_start/speed fields".into(),
        });
    }

    let timeline_start: RationalTime =
        serde_json::from_value(timeline_start_val).map_err(|e| MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: format!("invalid timeline_start: {e}"),
        })?;

    // clip.start が無いと clip_local 契約へ写せない — 警告黙殺せず拒否。
    let Some(clip_start_val) = clip_start.as_ref() else {
        return Err(MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail: "clip.start missing; cannot reconcile timeline_start".into(),
        });
    };
    let clip_start_rt: RationalTime =
        serde_json::from_value(clip_start_val.clone()).map_err(|e| {
            MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("invalid clip.start: {e}"),
            }
        })?;

    // 旧: source = source_start + (t - timeline_start) * speed
    // 新: source = source_start' + (t - clip.start) * speed
    // → source_start' = source_start + (clip.start - timeline_start) * speed
    if clip_start_rt != timeline_start {
        let source_start: RationalTime =
            serde_json::from_value(map.get("source_start").cloned().ok_or_else(|| {
                MigrateError::TimeMapRewrite {
                    path: path.to_string(),
                    detail: "source_start missing after check".into(),
                }
            })?)
            .map_err(|e| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("invalid source_start: {e}"),
            })?;
        let speed_num = map
            .get("speed_num")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "speed_num must be i64".into(),
            })?;
        let speed_den = map
            .get("speed_den")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: "speed_den must be i64".into(),
            })?;

        let corrected = adjust_source_start_for_clip_anchor(
            source_start,
            timeline_start,
            clip_start_rt,
            speed_num,
            speed_den,
        )
        .map_err(|detail| MigrateError::TimeMapRewrite {
            path: path.to_string(),
            detail,
        })?;
        map.insert(
            "source_start".into(),
            serde_json::to_value(corrected).map_err(|e| MigrateError::TimeMapRewrite {
                path: path.to_string(),
                detail: format!("serialize corrected source_start: {e}"),
            })?,
        );
        if !steps.contains(&"adjust_source_start_for_timeline_start") {
            steps.push("adjust_source_start_for_timeline_start");
        }
    }

    if !steps.contains(&"drop_timeline_start") {
        steps.push("drop_timeline_start");
    }
    Ok(())
}

/// 旧 timeline_start 基準写像を clip.start 基準へ移す source_start 補正。
fn adjust_source_start_for_clip_anchor(
    source_start: RationalTime,
    timeline_start: RationalTime,
    clip_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
) -> Result<RationalTime, String> {
    let delta = clip_start
        .try_sub(timeline_start)
        .map_err(|e| format!("clip.start - timeline_start: {e}"))?;
    let scaled = delta
        .try_mul_i64(speed_num)
        .map_err(|e| format!("delta * speed_num: {e}"))?;
    let unit = RationalTime::try_new(1, speed_den).map_err(|e| format!("1/speed_den: {e}"))?;
    let mapped = scaled
        .try_mul(unit)
        .map_err(|e| format!("scaled * unit: {e}"))?;
    source_start
        .try_add(mapped)
        .map_err(|e| format!("source_start + offset: {e}"))
}

fn rewrite_path_ops(
    clip: &mut Value,
    path: &str,
    asset_types: &BTreeMap<u64, String>,
    steps: &mut Vec<&'static str>,
) -> Result<(), MigrateError> {
    let Value::Object(map) = clip else {
        return Ok(());
    };
    let Some(raw_ops) = map.remove("path_ops") else {
        return Ok(());
    };

    // null / 空配列はフィールド削除だけで現行へ。
    let ops_arr = match raw_ops {
        Value::Null => Vec::new(),
        Value::Array(a) => a,
        other => {
            return Err(MigrateError::PathOpsRewrite {
                path: path.to_string(),
                detail: format!("path_ops must be array or null, got {other}"),
            });
        }
    };

    let upgraded: Vec<Value> = ops_arr
        .into_iter()
        .map(|op| upgrade_legacy_path_op(op, path))
        .collect::<Result<_, _>>()?;

    if !steps.contains(&"move_path_ops_to_recipe") {
        steps.push("move_path_ops_to_recipe");
    }

    if upgraded.is_empty() {
        return Ok(());
    }

    let source = map
        .get_mut("source")
        .ok_or_else(|| MigrateError::PathOpsRewrite {
            path: path.to_string(),
            detail: "clip missing source while path_ops present".into(),
        })?;

    let tag = source
        .get("source")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    match tag.as_str() {
        "vector" => {
            let recipe = source
                .get_mut("recipe")
                .and_then(|r| r.as_object_mut())
                .ok_or_else(|| MigrateError::PathOpsRewrite {
                    path: path.to_string(),
                    detail: "vector source missing recipe object".into(),
                })?;
            let existing = recipe
                .remove("modifiers")
                .and_then(|m| match m {
                    Value::Array(a) => Some(a),
                    Value::Null => Some(Vec::new()),
                    _ => None,
                })
                .unwrap_or_default();
            let mut merged = upgraded;
            merged.extend(existing);
            recipe.insert("modifiers".into(), Value::Array(merged));
            Ok(())
        }
        "asset" => {
            let asset_id = source
                .get("asset")
                .and_then(|a| a.as_u64())
                .ok_or_else(|| MigrateError::PathOpsRewrite {
                    path: path.to_string(),
                    detail: "asset source missing asset id".into(),
                })?;
            let ty = asset_types.get(&asset_id).map(String::as_str).unwrap_or("");
            if ty != SVG_ASSET_TYPE {
                return Err(MigrateError::PathOpsOnRaster {
                    path: path.to_string(),
                    detail: format!(
                        "asset {asset_id} type `{ty}` cannot host path_ops; only {SVG_ASSET_TYPE} converts to Vector"
                    ),
                });
            }
            *source = json!({
                "source": "vector",
                "recipe": {
                    "content": {
                        "kind": "svg_asset",
                        "asset": asset_id
                    },
                    "modifiers": upgraded
                }
            });
            Ok(())
        }
        other => Err(MigrateError::PathOpsOnRaster {
            path: path.to_string(),
            detail: format!("source `{other}` cannot host path_ops"),
        }),
    }
}

fn upgrade_legacy_path_op(mut op: Value, path: &str) -> Result<Value, MigrateError> {
    let Value::Object(map) = &mut op else {
        return Err(MigrateError::PathOpsRewrite {
            path: path.to_string(),
            detail: "path_op must be object".into(),
        });
    };
    let op_name = map
        .get("op")
        .and_then(|o| o.as_str())
        .unwrap_or("")
        .to_string();

    match op_name.as_str() {
        "twist" => {
            // D1i-2: center必須。旧JSONは原点を注入(正準空間の形状中心既定)。
            if !map.contains_key("center") {
                map.insert("center".into(), json!({"const": {"Vec2": [0.0, 0.0]}}));
            }
        }
        "wiggle" => {
            if let Some(seed) = map.get("seed") {
                if seed.as_u64().is_none() && seed.as_i64().is_none() {
                    let as_u64 =
                        extract_seed_u64(seed).ok_or_else(|| MigrateError::PathOpsRewrite {
                            path: path.to_string(),
                            detail: format!("cannot coerce Wiggle.seed {seed} to u64"),
                        })?;
                    map.insert("seed".into(), json!(as_u64));
                }
            } else {
                map.insert("seed".into(), json!(0u64));
            }
        }
        _ => {}
    }
    Ok(op)
}

fn extract_seed_u64(seed: &Value) -> Option<u64> {
    if let Some(n) = seed.as_u64() {
        return Some(n);
    }
    if let Some(n) = seed.as_i64() {
        return u64::try_from(n).ok();
    }
    // DocParam::Const F64
    if let Some(v) = seed
        .get("const")
        .and_then(|c| c.get("F64"))
        .and_then(|f| f.as_f64())
    {
        if v.is_finite() && v >= 0.0 && v == v.trunc() {
            return Some(v as u64);
        }
    }
    None
}

/// EffectInstance / DocKeyframe の欠落`id`をJSON段階で採番(D2必須。拒否→D1e変換)。
fn inject_missing_stable_ids_json(root: &mut Value) -> Result<bool, MigrateError> {
    let mut next = root
        .get("next_stable_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mut observed_max: Option<u64> = None;
    let mut injected = false;

    if let Some(tracks) = root.get_mut("tracks").and_then(|t| t.as_array_mut()) {
        for track in tracks.iter_mut() {
            let Some(items) = track.get_mut("items").and_then(|i| i.as_array_mut()) else {
                continue;
            };
            for item in items.iter_mut() {
                if inject_ids_in_item(item, &mut next, &mut observed_max)? {
                    injected = true;
                }
            }
        }
    }

    if inject_ids_in_effect_definitions(root, &mut next, &mut observed_max)? {
        injected = true;
    }

    // カウンタ整合はinjected(新規採番の有無)と無関係に行う: 既存idだけでも
    // 観測した既存idの最大値がカウンタ以上なら追い越して書き戻す。
    let mut counter_updated = false;
    if let Some(max_id) = observed_max {
        if next <= max_id {
            next = max_id
                .checked_add(1)
                .ok_or_else(|| MigrateError::StableId("stable id sequence exhausted".into()))?;
            counter_updated = true;
        }
    }
    if injected || counter_updated {
        if let Value::Object(map) = root {
            map.insert("next_stable_id".into(), json!(next));
        }
    }
    Ok(injected)
}

/// D1l: `root.effect_definitions[].params`内のキーフレームid欠落を採番する。
fn inject_ids_in_effect_definitions(
    root: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(defs) = root
        .get_mut("effect_definitions")
        .and_then(|d| d.as_array_mut())
    {
        for def in defs.iter_mut() {
            let Value::Object(map) = def else {
                continue;
            };
            if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                note_id(observed_max, id);
            }
            if let Some(params) = map.get_mut("params").and_then(|p| p.as_object_mut()) {
                for param in params.values_mut() {
                    if inject_ids_in_param(param, next, observed_max)? {
                        injected = true;
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_item(
    item: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let kind = item
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "clip" => inject_ids_in_clip(item, next, observed_max),
        "group" => {
            let mut injected = false;
            if let Some(env) = item.get_mut("envelope") {
                if inject_ids_in_envelope(env, next, observed_max)? {
                    injected = true;
                }
            }
            if let Some(children) = item.get_mut("children").and_then(|c| c.as_array_mut()) {
                for child in children.iter_mut() {
                    if inject_ids_in_item(child, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_clip(
    clip: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(env) = clip.get_mut("envelope") {
        if inject_ids_in_envelope(env, next, observed_max)? {
            injected = true;
        }
    }
    if let Some(source) = clip.get_mut("source") {
        if inject_ids_in_source(source, next, observed_max)? {
            injected = true;
        }
    }
    Ok(injected)
}

fn inject_ids_in_envelope(
    env: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(xf) = env.get_mut("transform") {
        for key in ["position", "anchor", "scale", "rotation"] {
            if let Some(p) = xf.get_mut(key) {
                if inject_ids_in_param(p, next, observed_max)? {
                    injected = true;
                }
            }
        }
    }
    if let Some(opacity) = env.get_mut("opacity") {
        if inject_ids_in_param(opacity, next, observed_max)? {
            injected = true;
        }
    }
    if let Some(effects) = env.get_mut("effects").and_then(|e| e.as_array_mut()) {
        for effect in effects.iter_mut() {
            if let Value::Object(map) = effect {
                if !map.contains_key("id") {
                    let id = allocate_stable(next)?;
                    note_id(observed_max, id);
                    map.insert("id".into(), json!(id));
                    injected = true;
                } else if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                    note_id(observed_max, id);
                }
                if let Some(params) = map.get_mut("params").and_then(|p| p.as_object_mut()) {
                    for param in params.values_mut() {
                        if inject_ids_in_param(param, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_source(
    source: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let tag = source
        .get("source")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    match tag.as_str() {
        "plugin" => {
            let mut injected = false;
            if let Some(params) = source.get_mut("params").and_then(|p| p.as_object_mut()) {
                for param in params.values_mut() {
                    if inject_ids_in_param(param, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        "vector" => {
            let mut injected = false;
            if let Some(recipe) = source.get_mut("recipe") {
                if let Some(content) = recipe.get_mut("content") {
                    if inject_ids_in_vector_content(content, next, observed_max)? {
                        injected = true;
                    }
                }
                if let Some(mods) = recipe.get_mut("modifiers").and_then(|m| m.as_array_mut()) {
                    for op in mods.iter_mut() {
                        if inject_ids_in_path_op(op, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
            Ok(injected)
        }
        "asset" => {
            let mut injected = false;
            if let Some(audio) = source.get_mut("audio").and_then(|a| a.as_array_mut()) {
                for comp in audio.iter_mut() {
                    if let Some(gain) = comp.get_mut("gain") {
                        if inject_ids_in_param(gain, next, observed_max)? {
                            injected = true;
                        }
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_vector_content(
    content: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let kind = content
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("")
        .to_string();
    match kind.as_str() {
        "standard_shape" => {
            let mut injected = false;
            for key in ["width", "height"] {
                if let Some(p) = content.get_mut(key) {
                    if inject_ids_in_param(p, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        "group" => {
            let mut injected = false;
            if let Some(children) = content.get_mut("children").and_then(|c| c.as_array_mut()) {
                for child in children.iter_mut() {
                    if inject_ids_in_vector_content(child, next, observed_max)? {
                        injected = true;
                    }
                }
            }
            Ok(injected)
        }
        _ => Ok(false),
    }
}

fn inject_ids_in_path_op(
    op: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Value::Object(map) = op {
        for key in [
            "amount",
            "ridges",
            "distance",
            "radius",
            "start",
            "end",
            "offset",
            "angle",
            "center",
            "amp",
            "freq",
            "copies",
            "start_opacity",
            "end_opacity",
        ] {
            if let Some(p) = map.get_mut(key) {
                if inject_ids_in_param(p, next, observed_max)? {
                    injected = true;
                }
            }
        }
        if let Some(xf) = map.get_mut("transform") {
            for key in ["position", "anchor", "scale", "rotation"] {
                if let Some(p) = xf.get_mut(key) {
                    if inject_ids_in_param(p, next, observed_max)? {
                        injected = true;
                    }
                }
            }
        }
    }
    Ok(injected)
}

fn inject_ids_in_param(
    param: &mut Value,
    next: &mut u64,
    observed_max: &mut Option<u64>,
) -> Result<bool, MigrateError> {
    let mut injected = false;
    if let Some(keys) = param
        .get_mut("keyframes")
        .and_then(|kf| kf.get_mut("keys"))
        .and_then(|k| k.as_array_mut())
    {
        for key in keys.iter_mut() {
            if let Value::Object(map) = key {
                if !map.contains_key("id") {
                    let id = allocate_stable(next)?;
                    note_id(observed_max, id);
                    map.insert("id".into(), json!(id));
                    injected = true;
                } else if let Some(id) = map.get("id").and_then(|v| v.as_u64()) {
                    note_id(observed_max, id);
                }
            }
        }
    }
    if param.get("x").is_some() || param.get("y").is_some() {
        if let Some(x) = param.get_mut("x") {
            if inject_ids_in_param(x, next, observed_max)? {
                injected = true;
            }
        }
        if let Some(y) = param.get_mut("y") {
            if inject_ids_in_param(y, next, observed_max)? {
                injected = true;
            }
        }
    }
    Ok(injected)
}

fn allocate_stable(next: &mut u64) -> Result<u64, MigrateError> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or_else(|| MigrateError::StableId("stable id sequence exhausted".into()))?;
    Ok(id)
}

fn note_id(observed_max: &mut Option<u64>, id: u64) {
    *observed_max = Some(observed_max.map_or(id, |m| m.max(id)));
}

fn doc_has_stable_ids(doc: &Document) -> bool {
    fn walk_item(item: &TrackItem) -> bool {
        match item {
            TrackItem::Clip(clip) => {
                if !clip.envelope.effects.is_empty() {
                    return true;
                }
                param_has_keys(&clip.envelope.opacity)
                    || param_has_keys(&clip.envelope.transform.position)
                    || param_has_keys(&clip.envelope.transform.anchor)
                    || param_has_keys(&clip.envelope.transform.scale)
                    || param_has_keys(&clip.envelope.transform.rotation)
                    || match &clip.source {
                        ClipSource::Plugin { params, .. } => params.values().any(param_has_keys),
                        ClipSource::Vector { recipe } => recipe_has_keys(recipe),
                        ClipSource::Asset { audio, .. } => {
                            audio.iter().any(|comp| param_has_keys(&comp.gain))
                        }
                    }
            }
            TrackItem::Group(group) => {
                !group.envelope.effects.is_empty()
                    || param_has_keys(&group.envelope.opacity)
                    || group.children.iter().any(walk_item)
            }
        }
    }
    fn param_has_keys(p: &DocParam) -> bool {
        match p {
            DocParam::Keyframes(k) => !k.keys().is_empty(),
            DocParam::Vec2Axes { x, y } => param_has_keys(x) || param_has_keys(y),
            _ => false,
        }
    }
    fn recipe_has_keys(recipe: &VectorRecipe) -> bool {
        content_has_keys(&recipe.content)
            || recipe.modifiers.iter().any(|op| match op {
                PathOp::PuckerBloat { amount }
                | PathOp::Offset {
                    distance: amount, ..
                }
                | PathOp::RoundCorners { radius: amount } => param_has_keys(amount),
                PathOp::ZigZag { amount, ridges, .. } => {
                    param_has_keys(amount) || param_has_keys(ridges)
                }
                PathOp::Trim {
                    start, end, offset, ..
                } => param_has_keys(start) || param_has_keys(end) || param_has_keys(offset),
                PathOp::Twist { angle, center } => param_has_keys(angle) || param_has_keys(center),
                PathOp::Wiggle { amp, freq, .. } => param_has_keys(amp) || param_has_keys(freq),
                PathOp::Repeater {
                    copies,
                    offset,
                    transform,
                    start_opacity,
                    end_opacity,
                    ..
                } => {
                    param_has_keys(copies)
                        || param_has_keys(offset)
                        || param_has_keys(&transform.position)
                        || param_has_keys(start_opacity)
                        || param_has_keys(end_opacity)
                }
            })
    }
    fn content_has_keys(c: &VectorContent) -> bool {
        match c {
            VectorContent::StandardShape { shape } => match shape {
                crate::schema::StandardShape::Rect { width, height }
                | crate::schema::StandardShape::Ellipse { width, height } => {
                    param_has_keys(width) || param_has_keys(height)
                }
            },
            VectorContent::Group { children } => children.iter().any(content_has_keys),
            _ => false,
        }
    }
    !doc.effect_definitions.is_empty()
        || doc
            .tracks
            .iter()
            .flat_map(|t| t.items.iter())
            .any(walk_item)
}

/// ファイルをbackup後に現行スキーマへ書換える。dry_run/noopでは原本を触らない。
pub(crate) fn migrate_document_file_with_limits(
    path: &Path,
    options: &MigrateFileOptions,
    limits: &ResourceLimits,
) -> Result<MigrateFileResult, MigrateError> {
    // loadと同じbounded readを使う(別経路禁止)。
    let bytes = read_file_bounded(path, limits)?;
    let (doc, report) = migrate_bytes_with_limits(&bytes, limits)?;
    let migrated = report.did_migrate();
    let backup_path = pre_migrate_backup_path(path);

    if options.dry_run || !migrated {
        return Ok(MigrateFileResult {
            backup_path,
            report,
            migrated,
        });
    }

    // exists()+copy の TOCTOU を避け、create_new で排他作成してから内容を書く。
    // backup の fsync(+親dir fsync)が終わるまで save_document しない。
    let mut backup_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return Err(MigrateError::BackupExists(backup_path));
        }
        Err(e) => return Err(MigrateError::Io(e)),
    };
    // 読んだ bytes を書く(再読込 TOCTOU も避ける)。失敗時は不完全 bak を消す。
    if let Err(e) = (|| -> io::Result<()> {
        backup_file.write_all(&bytes)?;
        backup_file.flush()?;
        backup_file.sync_all()?;
        Ok(())
    })() {
        let _ = fs::remove_file(&backup_path);
        return Err(MigrateError::Io(e));
    }
    drop(backup_file);

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if let Err(e) = sync_dir(parent) {
        // bak は残してよい(最後の既知良品)。原本は未書換。
        return Err(MigrateError::Io(e));
    }

    save_document(path, &doc)?;
    Ok(MigrateFileResult {
        backup_path,
        report,
        migrated,
    })
}

/// 原本ファイル名に `BACKUP_SUFFIX` をバイト列のまま付与する。
/// `to_str()` フォールバックで `document.json.bak` に化けるのを防ぐ。
fn pre_migrate_backup_path(path: &Path) -> PathBuf {
    let mut backup_name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| OsString::from("document.json"));
    backup_name.push(BACKUP_SUFFIX);
    path.with_file_name(backup_name)
}

/// persist.rs と同型: Unix は親ディレクトリ fsync、非Unix は省略。
fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let dir_file = File::open(dir)?;
        dir_file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}

fn read_file_bounded(path: &Path, limits: &ResourceLimits) -> Result<Vec<u8>, MigrateError> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    Read::by_ref(&mut file)
        .take(limits.max_file_bytes.saturating_add(1))
        .read_to_end(&mut buf)?;
    limits.check_file_bytes(buf.len() as u64)?;
    Ok(buf)
}

/// 移行後Documentから意味指紋を取る(S12自動比較用)。
pub fn semantic_fingerprint(doc: &Document, sample_times: &[RationalTime]) -> SemanticFingerprint {
    let tracks = DataTracks::new();
    let resolved = ResolvedLayerParams::default();
    let mut param_evals = Vec::new();
    let mut dependency_edges = BTreeSet::new();
    let mut timemap_samples = Vec::new();

    for track in &doc.tracks {
        for item in &track.items {
            collect_semantics(
                item,
                sample_times,
                &tracks,
                &resolved,
                &mut param_evals,
                &mut dependency_edges,
                &mut timemap_samples,
            );
        }
    }

    SemanticFingerprint {
        param_evals,
        dependency_edges,
        timemap_samples,
    }
}

fn collect_semantics(
    item: &TrackItem,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
    timemap_samples: &mut Vec<(u64, String, String)>,
) {
    match item {
        TrackItem::Clip(clip) => {
            collect_clip_semantics(
                clip,
                sample_times,
                tracks,
                resolved,
                param_evals,
                deps,
                timemap_samples,
            );
        }
        TrackItem::Group(group) => {
            collect_envelope_semantics(
                &group.envelope,
                sample_times,
                tracks,
                resolved,
                param_evals,
                deps,
            );
            for child in &group.children {
                collect_semantics(
                    child,
                    sample_times,
                    tracks,
                    resolved,
                    param_evals,
                    deps,
                    timemap_samples,
                );
            }
        }
    }
}

fn collect_clip_semantics(
    clip: &Clip,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
    timemap_samples: &mut Vec<(u64, String, String)>,
) {
    let layer = clip.envelope.layer_id.get();
    collect_envelope_semantics(
        &clip.envelope,
        sample_times,
        tracks,
        resolved,
        param_evals,
        deps,
    );
    for t in sample_times {
        if let Ok(src) = clip.time_map.try_map(*t) {
            timemap_samples.push((layer, format!("{t:?}"), format!("{src:?}")));
        }
    }
    if let ClipSource::Vector { recipe } = &clip.source {
        for (i, op) in recipe.modifiers.iter().enumerate() {
            let _ = i;
            collect_path_op_params(layer, op, sample_times, tracks, resolved, param_evals);
        }
    }
    if let ClipSource::Asset { audio, .. } = &clip.source {
        for (i, comp) in audio.iter().enumerate() {
            if let Some(name) = audio_gain_fingerprint_name(i) {
                for t in sample_times {
                    push_eval(param_evals, layer, name, &comp.gain, *t, tracks, resolved);
                }
            }
        }
    }
}

fn audio_gain_fingerprint_name(index: usize) -> Option<&'static str> {
    match index {
        0 => Some("audio[0].gain"),
        1 => Some("audio[1].gain"),
        2 => Some("audio[2].gain"),
        3 => Some("audio[3].gain"),
        _ => Some("audio[n].gain"),
    }
}

fn collect_envelope_semantics(
    env: &crate::schema::ItemEnvelope,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
    deps: &mut BTreeSet<(u64, &'static str, u64)>,
) {
    let layer = env.layer_id.get();
    if let Some(parent) = env.transform.parent {
        deps.insert((layer, "parent", parent.get()));
    }
    collect_param_deps(layer, &env.transform.position, deps);
    collect_param_deps(layer, &env.transform.anchor, deps);
    collect_param_deps(layer, &env.transform.scale, deps);
    collect_param_deps(layer, &env.transform.rotation, deps);
    collect_param_deps(layer, &env.opacity, deps);

    for t in sample_times {
        push_eval(
            param_evals,
            layer,
            "opacity",
            &env.opacity,
            *t,
            tracks,
            resolved,
        );
        push_eval(
            param_evals,
            layer,
            "position",
            &env.transform.position,
            *t,
            tracks,
            resolved,
        );
        push_eval(
            param_evals,
            layer,
            "rotation",
            &env.transform.rotation,
            *t,
            tracks,
            resolved,
        );
    }
}

fn collect_param_deps(from: u64, param: &DocParam, deps: &mut BTreeSet<(u64, &'static str, u64)>) {
    match param {
        DocParam::LookAt { target, .. } => {
            deps.insert((from, "look_at", target.get()));
        }
        DocParam::Follow { target, .. } => {
            deps.insert((from, "follow", target.get()));
        }
        DocParam::Vec2Axes { x, y } => {
            collect_param_deps(from, x, deps);
            collect_param_deps(from, y, deps);
        }
        _ => {}
    }
}

fn collect_path_op_params(
    layer: u64,
    op: &PathOp,
    sample_times: &[RationalTime],
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
    param_evals: &mut Vec<(u64, &'static str, String)>,
) {
    let params: &[(&str, &DocParam)] = match op {
        PathOp::PuckerBloat { amount } => &[("pucker_bloat.amount", amount)],
        PathOp::Offset { distance, .. } => &[("offset.distance", distance)],
        PathOp::Trim {
            start, end, offset, ..
        } => &[
            ("trim.start", start),
            ("trim.end", end),
            ("trim.offset", offset),
        ],
        PathOp::Twist { angle, center } => &[("twist.angle", angle), ("twist.center", center)],
        PathOp::RoundCorners { radius } => &[("round.radius", radius)],
        PathOp::ZigZag { amount, ridges, .. } => {
            &[("zigzag.amount", amount), ("zigzag.ridges", ridges)]
        }
        PathOp::Wiggle { amp, freq, .. } => &[("wiggle.amp", amp), ("wiggle.freq", freq)],
        PathOp::Repeater { copies, offset, .. } => {
            &[("repeater.copies", copies), ("repeater.offset", offset)]
        }
    };
    for (name, param) in params {
        for t in sample_times {
            push_eval(param_evals, layer, name, param, *t, tracks, resolved);
        }
    }
}

fn push_eval(
    out: &mut Vec<(u64, &'static str, String)>,
    layer: u64,
    name: &'static str,
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) {
    if let Ok(v) = eval_doc_param(param, t, tracks, resolved) {
        out.push((layer, name, format!("{v:?}")));
    }
}

/// 旧`timeline_start`付きTimeMapの写像を、現行契約(clip_local)で再現して比較する。
pub fn legacy_timemap_source(
    source_start: RationalTime,
    timeline_start: RationalTime,
    speed_num: i64,
    speed_den: i64,
    timeline_time: RationalTime,
) -> Result<RationalTime, motolii_core::TimeMapError> {
    let delta = timeline_time.try_sub(timeline_start)?;
    let scaled = delta.try_mul_i64(speed_num)?;
    let unit = RationalTime::try_new(1, speed_den)?;
    let mapped = scaled.try_mul(unit)?;
    Ok(source_start.try_add(mapped)?)
}

/// `clip.start == timeline_start`のとき、現行TimeMapのclip_local写像と一致することの検査口。
pub fn modern_timemap_source(
    time_map: &TimeMap,
    clip_start: RationalTime,
    timeline_time: RationalTime,
) -> Result<RationalTime, motolii_core::TimeMapError> {
    let local = timeline_time.try_sub(clip_start)?;
    time_map.try_map(local)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::schema::ItemEnvelope;
    use motolii_core::RationalTime;

    #[test]
    fn counts_include_vector_modifiers_and_nested_groups() {
        let mut doc = Document::new_v1();
        let layer_a = doc.layers.allocate("a").unwrap();
        let layer_g = doc.layers.allocate("g").unwrap();
        let layer_c = doc.layers.allocate("c").unwrap();
        let tid = doc.track_ids.allocate("V1").unwrap();

        let mut keys = crate::DocKeyframeTrack::new();
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(0),
            t: RationalTime::ZERO,
            value: crate::DocValue::F64(0.0),
            interp: motolii_eval::Interp::Linear,
        });
        keys.insert(crate::DocKeyframe {
            id: crate::KeyframeId::from_raw(1),
            t: RationalTime::try_new(1, 1).unwrap(),
            value: crate::DocValue::F64(1.0),
            interp: motolii_eval::Interp::Hold,
        });
        doc.next_stable_id = {
            let mut seq = crate::StableIdSeq::new();
            let _ = seq.allocate();
            let _ = seq.allocate();
            seq
        };

        let nested = Clip {
            envelope: {
                let mut e = ItemEnvelope::new(layer_c);
                e.opacity = DocParam::Keyframes(keys);
                e
            },
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::StandardShape {
                        shape: crate::schema::StandardShape::Rect {
                            width: DocParam::const_f64(1.0),
                            height: DocParam::const_f64(1.0),
                        },
                    },
                    modifiers: vec![PathOp::Offset {
                        distance: DocParam::const_f64(0.1),
                        line_join: Default::default(),
                        miter_limit: 4.0,
                    }],
                },
            },
        };
        let top = Clip {
            envelope: ItemEnvelope::new(layer_a),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::asset_video_only(crate::AssetId::from_raw(0)),
        };
        doc.tracks.push(crate::Track {
            id: tid,
            items: vec![
                TrackItem::Clip(top),
                TrackItem::Group(Group {
                    envelope: ItemEnvelope::new(layer_g),
                    children: vec![TrackItem::Clip(nested)],
                }),
            ],
        });

        let counts = count_document(&doc);
        assert_eq!(counts.track_count, 1);
        assert_eq!(counts.clip_count, 2);
        assert_eq!(counts.keyframe_count, 2);
    }

    /// 非UTF-8ファイル名でも原本名+suffixのbakになり、document.json.bakへフォールバックしない。
    /// (macOSは非UTF-8パスを作れないので、パス組み立てのみを検証する)
    #[cfg(unix)]
    #[test]
    fn pre_migrate_backup_path_preserves_non_utf8_file_name() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let name = OsStr::from_bytes(b"legacy\xff.json");
        let path = Path::new("/tmp").join(name);
        let bak = pre_migrate_backup_path(&path);
        let mut expected = name.to_os_string();
        expected.push(BACKUP_SUFFIX);
        assert_eq!(bak.file_name(), Some(expected.as_os_str()));
        assert_ne!(
            bak.file_name().and_then(|s| s.to_str()),
            Some(format!("document.json{BACKUP_SUFFIX}").as_str())
        );
    }
}
