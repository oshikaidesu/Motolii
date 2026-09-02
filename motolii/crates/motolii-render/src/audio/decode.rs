#[cfg(test)]
#[path = "../../../../tests/testkit/mod.rs"]
mod testkit;

use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::render::audio::cache::{PcmCache, PcmFormat};
use crate::render::audio::error::{AudioError, Result};

pub const MAX_SAMPLES: u64 = 48_000 * 60 * 60 * 4;

pub fn decode_file(path: impl AsRef<Path>) -> Result<PcmCache> {
    decode_file_audio_ordinal(path, 0)
}

pub fn decode_file_audio_ordinal(path: impl AsRef<Path>, ordinal: u32) -> Result<PcmCache> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    decode_stream_audio_ordinal(mss, &hint, ordinal)
}

pub fn decode_stream(mss: MediaSourceStream<'static>, hint: &Hint) -> Result<PcmCache> {
    decode_stream_audio_ordinal(mss, hint, 0)
}

fn decode_stream_audio_ordinal(
    mss: MediaSourceStream<'static>,
    hint: &Hint,
    ordinal: u32,
) -> Result<PcmCache> {
    let mut format = symphonia::default::get_probe().probe(
        hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    )?;

    let audio_tracks: Vec<_> = format
        .tracks()
        .iter()
        .filter(|t| matches!(t.codec_params, Some(CodecParameters::Audio(_))))
        .cloned()
        .collect();
    let track = audio_tracks
        .get(ordinal as usize)
        .ok_or(AudioError::StreamNotFound { ordinal })?;
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
                check_sample_limit(samples.len() as u64)?;
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    let pcm_format = pcm_format.ok_or(AudioError::NoAudioTrack)?;
    PcmCache::from_interleaved(samples, pcm_format)
}

fn check_sample_limit(observed: u64) -> Result<()> {
    if observed > MAX_SAMPLES {
        return Err(AudioError::SampleCountLimit {
            observed,
            limit: MAX_SAMPLES,
        });
    }
    Ok(())
}
