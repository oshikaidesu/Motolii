//! N-MEDIA-PLACE(素材取り込み半分): 実mediaファイルをprobe+fingerprintし、
//! `AssetDraft`構築(`motolii_doc::AssetDraft::from_probed_source`)に渡すplain値へ写す。
//!
//! - container/streamは既存`probe_container`をそのまま使う(重複実装しない)
//! - hashは正準`SourceFingerprintV1`(motolii-doc所有。source_bindingと同じ実装)
//! - Documentへの書き込みは既存`DocumentWriter::prepare_admit_asset`経路(呼び出し側)
//! - projectディレクトリへのコピー/proxy/配置は範囲外

use std::fs::File;
use std::path::Path;

use motolii_core::RationalTime;
use motolii_store::SourceFingerprintV1;

use crate::{probe_container, ContainerInfo, MediaError, Result};

/// admission用のsource probe結果。plain値のみでDocument意味は持たない。
#[derive(Debug, Clone)]
pub struct AdmissionSource {
    /// doc Assetのopaque type(例 `video/mp4`)。導出表はUIの
    /// `media_library::asset_type_for_name`のvideo/audio部分と同値。
    pub asset_type: String,
    /// 正準content_hash/size_bytes(`motolii-source-v1:sha256:<hex64>` + exact size)。
    pub fingerprint: SourceFingerprintV1,
    /// container内の全video/audio stream(既存`probe_container`の結果)。
    pub container: ContainerInfo,
    /// format(container)levelの総尺。
    pub duration: Option<RationalTime>,
}

/// 拡張子→admission対象のasset_type(video / audio / 静止画)。
///
/// UI側`crates/motolii-ui/src/media_library.rs`の`asset_type_for_name`と同表
/// (UIコードは動かさない決定のため、library側はここが持つ)。
///
/// 静止画は2026-08-18に開いた扉(利用者の初回タッチ観察(2))。ffmpegが**1枚の絵
/// として**開けるものだけを載せる: png / jpeg / webp。ここに無いのは意図的で、
/// - svg … ラスタライザが要る(ffmpegは開けない)
/// - gif / exr / tiff … 動く絵・HDR・多ページで、静止画1枚の意味に収まらない
/// UI の library 表(絵の一覧に出す側)はもっと広いままで構わない — 出せる物と
/// 取り込める物が違うのは、取り込みで理由つきに落ちるので人には見える。
pub fn admission_asset_type_for_path(path: &Path) -> Option<&'static str> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())?;
    match ext.as_str() {
        "mp4" | "m4v" => Some("video/mp4"),
        "mov" => Some("video/quicktime"),
        "mkv" => Some("video/x-matroska"),
        "webm" => Some("video/webm"),
        "wav" => Some("audio/wav"),
        "mp3" => Some("audio/mpeg"),
        "aac" | "m4a" => Some("audio/mp4"),
        "aiff" | "aif" => Some("audio/aiff"),
        "flac" => Some("audio/flac"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

/// 実mediaファイル(動画/音声/静止画)をadmission用にprobeする。
///
/// 1. 拡張子からasset_typeを導出(未対応拡張子はprobe前にtyped error)
/// 2. `probe_container`で全stream列挙
/// 3. 粗いkind整合(`video/*`と`image/*`はvideo stream必須、`audio/*`はaudio stream必須)
/// 4. exact source bytesを`SourceFingerprintV1`でfull hash
///
/// 静止画は ffmpeg からも video stream として見える(png_pipe等の1枚 container)。
/// 尺は無く`duration`は`None`のまま — placeの「長さ不明」の意味がそれを受ける。
pub fn probe_admission_source(path: impl AsRef<Path>) -> Result<AdmissionSource> {
    let path = path.as_ref();
    let asset_type = admission_asset_type_for_path(path).ok_or_else(|| {
        MediaError::Probe(format!(
            "unsupported media file extension for admission: {}",
            path.display()
        ))
    })?;

    let container = probe_container(path)?;
    if asset_type.starts_with("image/") && container.video_streams.is_empty() {
        return Err(MediaError::Probe(format!(
            "extension says {asset_type} but no image could be read from: {}",
            path.display()
        )));
    }
    if asset_type.starts_with("video/") && container.video_streams.is_empty() {
        return Err(MediaError::Probe(format!(
            "extension says {asset_type} but container has no video stream: {}",
            path.display()
        )));
    }
    if asset_type.starts_with("audio/") && container.audio_streams.is_empty() {
        return Err(MediaError::Probe(format!(
            "extension says {asset_type} but container has no audio stream: {}",
            path.display()
        )));
    }

    let fingerprint = SourceFingerprintV1::from_reader(File::open(path)?)?;
    let duration = container.duration;

    Ok(AdmissionSource {
        asset_type: asset_type.to_owned(),
        fingerprint,
        container,
        duration,
    })
}
