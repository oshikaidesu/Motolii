//! 編集系subcommand(import / place / set-soundtrack)の意味をライブラリ関数で閉じる。
//!
//! binはargvをここへ写すだけ。GUIも後日**同じ関数**(または同じdoc側prepare_*)を呼ぶ。
//! 開く→writer→apply→保存の形は`apply_document`(document_debug.rs)と同一で、
//! 第二の書き込み経路を作らない。時刻の受け口はCLI精度としてミリ秒丸めのf64秒。

use std::path::Path;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::{
    open_project_resolved, AssetDraft, AssetId, Command, DocumentWriter, LayerId, ResourceLimits,
    SaveOptions, Soundtrack, TrackItem,
};
use motolii_media::probe_admission_source;
use motolii_plugins_firstparty::first_party_runtime;

use crate::CliError;

/// 新規プロジェクト。意味は既存`new_document`(空コンポ+V1トラック)そのもの。
pub fn new_project(path: &Path) -> Result<(), CliError> {
    crate::new_document(path)
}

/// CLI精度の時刻: 秒(f64)→ミリ秒丸めの有理数。
fn seconds_to_time(seconds: f64, flag: &str) -> Result<RationalTime, CliError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(CliError::Usage(format!(
            "{flag} must be a finite non-negative number of seconds"
        )));
    }
    let millis = (seconds * 1000.0).round() as i64;
    RationalTime::try_new(millis, 1000).map_err(|e| CliError::Usage(e.to_string()))
}

/// 開く→編集→保存を1回で閉じる共通器。編集の中身だけを受け取る。
fn with_writer<T>(
    project: &Path,
    edit: impl FnOnce(&mut DocumentWriter) -> Result<T, CliError>,
) -> Result<T, CliError> {
    let runtime = first_party_runtime()?;
    let mut opened =
        open_project_resolved(project, &ResourceLimits::production(), runtime.catalog())
            .map_err(|e| CliError::Usage(e.to_string()))?;
    let mut writer = DocumentWriter::new(
        opened.recovered.document,
        Arc::new(runtime.catalog().clone()),
    )
    .map_err(|e| CliError::Usage(e.to_string()))?;
    let value = edit(&mut writer)?;
    opened
        .session
        .save_document(writer.snapshot().as_ref(), &SaveOptions::default())
        .map_err(|e| CliError::Usage(e.to_string()))?;
    Ok(value)
}

/// 実メディアをprobe→`AssetDraft`→既存admission経路でDocumentへ取り込む。
pub fn import_asset(project: &Path, media: &Path) -> Result<AssetId, CliError> {
    let source = probe_admission_source(media).map_err(|e| CliError::Usage(e.to_string()))?;
    let absolute = media
        .canonicalize()
        .map_err(|e| CliError::Usage(format!("cannot resolve {}: {e}", media.display())))?;
    let root = project
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .and_then(|parent| parent.canonicalize().ok());
    with_writer(project, move |writer| {
        let mut draft = AssetDraft::from_probed_source(
            source.asset_type.clone(),
            &source.fingerprint,
            &absolute,
            root.as_deref(),
        );
        // probeが測った総尺を落とさない。placeの尺決めがこれを見る。
        draft.duration = source.duration;
        let prepared = writer
            .prepare_admit_asset(draft)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        let id = prepared.asset().id;
        let gesture = writer.begin_gesture();
        writer
            .apply_prepared_asset_admission(gesture, prepared)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        Ok(id)
    })
}

/// 取り込み済みassetを最初のTrackへ配置する。意味はdoc側`prepare_place_asset_clip`が持つ。
pub fn place_clip(project: &Path, asset: AssetId, at_seconds: f64) -> Result<LayerId, CliError> {
    let start = seconds_to_time(at_seconds, "--at")?;
    with_writer(project, move |writer| {
        let command = writer
            .prepare_place_asset_clip(asset, start)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        let layer = match &command {
            Command::AddTrackItem {
                item: TrackItem::Clip(clip),
                ..
            } => clip.envelope.layer_id,
            _ => unreachable!("prepare_place_asset_clip returns AddTrackItem(Clip)"),
        };
        let gesture = writer.begin_gesture();
        writer
            .apply_command(gesture, command)
            .map_err(|e| CliError::Usage(e.to_string()))?;
        Ok(layer)
    })
}

/// 曲を貼る。`Command::SetSoundtrack`(唯一のsoundtrack書き込み経路)を通す。
pub fn set_soundtrack(
    project: &Path,
    asset: AssetId,
    offset_seconds: f64,
    gain: f64,
) -> Result<(), CliError> {
    let offset = seconds_to_time(offset_seconds, "--offset")?;
    let soundtrack =
        Soundtrack::try_new(asset, offset, gain).map_err(|e| CliError::Usage(e.to_string()))?;
    with_writer(project, move |writer| {
        let old = writer.snapshot().soundtrack;
        let gesture = writer.begin_gesture();
        writer
            .apply_command(
                gesture,
                Command::SetSoundtrack {
                    old,
                    new: Some(soundtrack),
                },
            )
            .map_err(|e| CliError::Usage(e.to_string()))?;
        Ok(())
    })
}
