//! OS から落ちてきた media 1本を Document へ入れる列。
//!
//! **意味はここで1つも決めない。** 呼ぶ関数も順も CLI の import / place
//! (`crates/motolii-cli/src/document_edit.rs:59-107`)と同じで、
//! probe は `motolii-media`、Asset の形は `motolii-doc` が持つ:
//!
//! ```text
//! probe_admission_source        … 拡張子→asset_type、container、content hash
//! AssetDraft::from_probed_source … 名前・path(project 相対も)・size(尺は呼び手が足す)
//! prepare_admit_asset / apply_*  … 素材台帳へ入れる(table-local ID の照合つき)
//! prepare_place_asset_clip       … 最初のトラックの末尾へ clip を置く
//! ```
//!
//! CLI と違うのは**取り込みと配置を1つの `GestureId` に入れる**ことだけ。
//! 1回のドロップは人にとって1操作なので、Undo も1回で戻る。
//!
//! もう1つ、CLI に無い分岐がここにある。**まだ曲が無い project へ音声を落としたら、
//! clip ではなく soundtrack にする**(CapCut / Ableton と同じ既定 — MV を作る人が
//! 最初に置くのは曲で、置いた瞬間に波形帯が出るのが期待される絵である)。
//! 曲が既にあれば従来どおり clip。曲の差し替え・削除の口はまだ無い。

use std::path::Path;

use motolii_core::RationalTime;
use motolii_doc::{AssetDraft, Command, DocumentWriter, LayerId, Soundtrack, TrackItem};
use motolii_media::probe_admission_source;

/// 落ちた media 1本がどこへ着地したか。
pub(crate) enum Admitted {
    /// トラックへ clip として置いた。
    Clip(LayerId),
    /// project 直下の曲として貼った。
    Soundtrack,
}

/// 曲に貼るときの既定。offset / gain の UI はまだ無いので、頭からそのままの音量。
const SOUNDTRACK_GAIN: f64 = 1.0;

/// media を1本取り込んで `at` へ置き、どこへ着地したかを返す。
/// **落ちたら Document は動かない。**
///
/// 素材だけ入って clip が置けない中途半端を作らないため、置けるかどうかは
/// 台帳を触る前に確かめる(`prepare_place_asset_clip` が拒む条件と同じ)。
/// 曲として貼る側はトラックも playhead も要らないので、この確認を通らない。
pub(crate) fn import_and_place(
    writer: &mut DocumentWriter,
    media: &Path,
    project_root: Option<&Path>,
    at: RationalTime,
) -> Result<Admitted, String> {
    let source = probe_admission_source(media).map_err(|error| error.to_string())?;
    let absolute = media
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", media.display()))?;
    let root = project_root.and_then(|root| root.canonicalize().ok());

    // **音声で、まだ曲が無いなら曲にする。** 判定は probe の `asset_type` だけで、
    // 拡張子表は `motolii-media` が持つ(ここで二重に持たない)
    let as_soundtrack =
        source.asset_type.starts_with("audio/") && writer.snapshot().soundtrack.is_none();

    // ---- 置ける所か先に見る。素材だけ入って clip が無い状態を作らない ----
    if !as_soundtrack {
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

    let mut draft = AssetDraft::from_probed_source(
        source.asset_type.clone(),
        &source.fingerprint,
        &absolute,
        root.as_deref(),
    );
    // CLI の import と同じく、probe が測った総尺を Asset まで運ぶ
    // (place の尺がこれを見て素材の終わりで切る)
    draft.duration = source.duration;
    let prepared = writer
        .prepare_admit_asset(draft)
        .map_err(|error| error.to_string())?;
    let asset = prepared.asset().id;

    // **1ドロップ = 1 `GestureId` = 1 Undo 単位。** 取り込みと配置を分けない
    // (曲として貼る側も同じ1 gesture — Undo 1回で素材ごと戻る)
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, prepared)
        .map_err(|error| error.to_string())?;

    if as_soundtrack {
        // 曲を書き換える唯一の経路(2026-08-17 決定)。CLI の `set_soundtrack` と同じ形で、
        // `old` は上の分岐が `None` を保証している
        let soundtrack = Soundtrack::try_new(asset, RationalTime::ZERO, SOUNDTRACK_GAIN)
            .map_err(|error| error.to_string())?;
        writer
            .apply_command(
                gesture,
                Command::SetSoundtrack {
                    old: None,
                    new: Some(soundtrack),
                },
            )
            .map_err(|error| error.to_string())?;
        return Ok(Admitted::Soundtrack);
    }

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
    Ok(Admitted::Clip(layer))
}
