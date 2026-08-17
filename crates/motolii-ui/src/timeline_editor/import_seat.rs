//! OS から落ちてきた media 1本を Document へ入れる列。
//!
//! **意味はここで1つも決めない。** 呼ぶ関数も順も CLI の import / place
//! (`crates/motolii-cli/src/document_edit.rs:59-107`)と同じで、
//! probe は `motolii-media`、Asset の形は `motolii-doc` が持つ:
//!
//! ```text
//! probe_admission_source        … 拡張子→asset_type、container、content hash
//! AssetDraft::from_probed_source … 名前・path(project 相対も)・size
//! prepare_admit_asset / apply_*  … 素材台帳へ入れる(table-local ID の照合つき)
//! prepare_place_asset_clip       … 最初のトラックの末尾へ clip を置く
//! ```
//!
//! CLI と違うのは**取り込みと配置を1つの `GestureId` に入れる**ことだけ。
//! 1回のドロップは人にとって1操作なので、Undo も1回で戻る。

use std::path::Path;

use motolii_core::RationalTime;
use motolii_doc::{AssetDraft, Command, DocumentWriter, LayerId, TrackItem};
use motolii_media::probe_admission_source;

/// media を1本取り込んで `at` へ置き、立った clip の layer を返す。
/// **落ちたら Document は動かない。**
///
/// 素材だけ入って clip が置けない中途半端を作らないため、置けるかどうかは
/// 台帳を触る前に確かめる(`prepare_place_asset_clip` が拒む条件と同じ)。
pub(crate) fn import_and_place(
    writer: &mut DocumentWriter,
    media: &Path,
    project_root: Option<&Path>,
    at: RationalTime,
) -> Result<LayerId, String> {
    let source = probe_admission_source(media).map_err(|error| error.to_string())?;
    let absolute = media
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", media.display()))?;
    let root = project_root.and_then(|root| root.canonicalize().ok());

    // ---- 置ける所か先に見る。素材だけ入って clip が無い状態を作らない ----
    {
        let document = writer.snapshot();
        if document.tracks.is_empty() {
            return Err("this project has no track to place a clip on".to_owned());
        }
        if at.as_seconds_f64() >= document.composition.duration.as_seconds_f64() {
            return Err(format!(
                "the playhead ({:.2}s) is at or past the end of the composition ({:.2}s)",
                at.as_seconds_f64(),
                document.composition.duration.as_seconds_f64()
            ));
        }
    }

    let draft = AssetDraft::from_probed_source(
        source.asset_type.clone(),
        &source.fingerprint,
        &absolute,
        root.as_deref(),
    );
    let prepared = writer
        .prepare_admit_asset(draft)
        .map_err(|error| error.to_string())?;
    let asset = prepared.asset().id;

    // **1ドロップ = 1 `GestureId` = 1 Undo 単位。** 取り込みと配置を分けない
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, prepared)
        .map_err(|error| error.to_string())?;
    let command = writer
        .prepare_place_asset_clip(asset, at)
        .map_err(|error| error.to_string())?;
    let layer = match &command {
        Command::AddTrackItem {
            item: TrackItem::Clip(clip),
            ..
        } => clip.envelope.layer_id,
        _ => unreachable!("prepare_place_asset_clip returns AddTrackItem(Clip)"),
    };
    writer
        .apply_command(gesture, command)
        .map_err(|error| error.to_string())?;
    Ok(layer)
}
