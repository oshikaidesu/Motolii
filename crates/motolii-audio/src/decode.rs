//! Symphonia decode → `PcmCache`(D4契約: 楽曲1本をインポート時に全展開)。

use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::cache::{PcmCache, PcmFormat};
use crate::error::{AudioError, Result};

/// パスから楽曲1本をデコードし、インターリーブf32 PCM全展開キャッシュを作る。
///
/// コンテナ判定はSymphoniaのprobe(拡張子ヒント+マジックバイト)に委ねる。
/// 複数音声トラックを含むファイルではデフォルトトラック1本のみを読む
/// (D4スコープ: ミキサー・複数trackは実装しない)。
pub fn decode_file(path: impl AsRef<Path>) -> Result<PcmCache> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    decode_stream(mss, &hint)
}

/// `MediaSourceStream`から直接デコードする(テスト用: メモリ上のバイト列を
/// ファイルに書かず直接渡せる)。
pub fn decode_stream(mss: MediaSourceStream<'static>, hint: &Hint) -> Result<PcmCache> {
    let mut format = symphonia::default::get_probe().probe(
        hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or(AudioError::NoAudioTrack)?;
    let track_id = track.id;
    let Some(CodecParameters::Audio(audio_params)) = track.codec_params.clone() else {
        return Err(AudioError::NoAudioTrack);
    };

    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(&audio_params, &AudioDecoderOptions::default())?;

    let mut samples: Vec<f32> = Vec::new();
    let mut packet_samples: Vec<f32> = Vec::new();
    let mut pcm_format: Option<PcmFormat> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(Some(packet)) => packet,
            Ok(None) => break, // ストリーム終端。
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                if pcm_format.is_none() {
                    let spec = decoded.spec();
                    pcm_format = Some(PcmFormat {
                        channels: spec.channels().count() as u16,
                        sample_rate: spec.rate(),
                    });
                }
                decoded.copy_to_vec_interleaved(&mut packet_samples);
                samples.extend_from_slice(&packet_samples);
            }
            // Symphoniaの規約: DecodeErrorは当該パケットだけ捨てて継続してよい
            // 回復可能エラー(壊れた1パケットで楽曲全体を拒否しない)。
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    let pcm_format = pcm_format.ok_or(AudioError::NoAudioTrack)?;
    PcmCache::from_interleaved(samples, pcm_format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_typed_io_error() {
        let result = decode_file("/nonexistent/path/to/song.wav");
        assert!(matches!(result, Err(AudioError::Io(_))));
    }

    #[test]
    fn corrupt_input_is_typed_error_not_panic() {
        let dir = motolii_testkit::tmp_dir("motolii-audio-decode-corrupt");
        let path = dir.join("not-audio.bin");
        std::fs::write(&path, b"this is not an audio file at all, just plain bytes")
            .expect("write garbage file");

        let result = decode_file(&path);
        assert!(result.is_err(), "garbage input must not decode");
    }
}
