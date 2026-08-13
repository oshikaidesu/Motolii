//! Document JSONの読み出しとD2 Command適用(GUIなし)。
use crate::CliError;
use motolii_doc::{
    open_project_resolved, Command, Document, DocumentWriter, ProjectSession, ResourceLimits,
    SaveOptions, Track,
};
use motolii_plugins_firstparty::first_party_runtime;
use std::path::Path;
use std::sync::Arc;

/// 空コンポ + 1本の空ビデオトラック。`place` / `AddTrackItem` の受け皿。
pub fn new_document(path: &Path) -> Result<(), CliError> {
    if path.exists() {
        return Err(CliError::Usage(format!(
            "document already exists: {}",
            path.display()
        )));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| CliError::Usage(e.to_string()))?;
    }
    let mut document = Document::new_current();
    let track = document
        .track_ids
        .allocate("V1")
        .map_err(|e| CliError::Usage(e.to_string()))?;
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    let mut session = ProjectSession::acquire(path, &ResourceLimits::production())
        .map_err(|e| CliError::Usage(e.to_string()))?;
    session
        .save_document(&document, &SaveOptions::default())
        .map_err(|e| CliError::Usage(e.to_string()))
}

pub fn dump_document(path: &Path) -> Result<String, CliError> {
    let runtime = first_party_runtime()?;
    let opened = open_project_resolved(path, &ResourceLimits::production(), runtime.catalog())
        .map_err(|e| CliError::Usage(e.to_string()))?;
    serde_json::to_string_pretty(&opened.recovered.document)
        .map_err(|e| CliError::Usage(e.to_string()))
}

pub fn apply_document(
    document: &Path,
    command_json: &str,
    out: Option<&Path>,
) -> Result<(), CliError> {
    let commands = parse_command_json(command_json)?;
    let runtime = first_party_runtime()?;
    let mut opened =
        open_project_resolved(document, &ResourceLimits::production(), runtime.catalog())
            .map_err(|e| CliError::Usage(e.to_string()))?;
    let mut writer = DocumentWriter::new(
        opened.recovered.document,
        Arc::new(runtime.catalog().clone()),
    )
    .map_err(|e| CliError::Usage(e.to_string()))?;
    writer
        .apply_macro(commands)
        .map_err(|e| CliError::Usage(e.to_string()))?;
    save_applied(&mut opened.session, writer.snapshot().as_ref(), out)
}

fn parse_command_json(command_json: &str) -> Result<Vec<Command>, CliError> {
    let value: serde_json::Value =
        serde_json::from_str(command_json).map_err(|e| CliError::Usage(e.to_string()))?;
    match value {
        serde_json::Value::Array(_) => {
            serde_json::from_value(value).map_err(|e| CliError::Usage(e.to_string()))
        }
        serde_json::Value::Object(_) => {
            let command =
                serde_json::from_value(value).map_err(|e| CliError::Usage(e.to_string()))?;
            Ok(vec![command])
        }
        _ => Err(CliError::Usage(
            "command JSON must be an object or array".into(),
        )),
    }
}

fn save_applied(
    session: &mut ProjectSession,
    doc: &Document,
    out: Option<&Path>,
) -> Result<(), CliError> {
    let options = SaveOptions::default();
    match out {
        None => session
            .save_document(doc, &options)
            .map_err(|e| CliError::Usage(e.to_string())),
        Some(out) if same_path(session.document_path(), out) => session
            .save_document(doc, &options)
            .map_err(|e| CliError::Usage(e.to_string())),
        Some(out) => {
            let mut dest = ProjectSession::acquire(out, session.limits())
                .map_err(|e| CliError::Usage(e.to_string()))?;
            dest.save_document(doc, &options)
                .map_err(|e| CliError::Usage(e.to_string()))
        }
    }
}

fn same_path(opened: &Path, out: &Path) -> bool {
    match out.canonicalize() {
        Ok(canon) => canon == opened,
        Err(_) => out == opened,
    }
}
