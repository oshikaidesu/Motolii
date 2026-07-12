use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AudioError;

/// デコード済みPCMのフォーマット(インターリーブf32)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

/// 楽曲1本のPCM全展開キャッシュ。ミキサーは存在しない(D4)。
#[derive(Debug, Clone)]
pub struct PcmCache {
    samples: Vec<f32>,
    format: PcmFormat,
}

impl PcmCache {
    pub fn from_interleaved(samples: Vec<f32>, format: PcmFormat) -> Result<Self, AudioError> {
        if format.channels == 0 {
            return Err(AudioError::UnsupportedChannels {
                channels: format.channels as usize,
            });
        }
        if !samples.len().is_multiple_of(format.channels as usize) {
            return Err(AudioError::Decode(format!(
                "sample count {} is not divisible by channel count {}",
                samples.len(),
                format.channels
            )));
        }
        Ok(Self { samples, format })
    }

    pub fn format(&self) -> PcmFormat {
        self.format
    }

    pub fn frame_count(&self) -> u64 {
        (self.samples.len() / self.format.channels as usize) as u64
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// 任意フレーム位置から連続フレームを読む。終端を超える分は切り詰める。
    pub fn read_frames(&self, start_frame: u64, frame_count: usize) -> Result<&[f32], AudioError> {
        let ch = self.format.channels as usize;
        let total = self.frame_count();
        if start_frame >= total {
            return Err(AudioError::ReadPastEnd {
                frame: start_frame,
                total_frames: total,
            });
        }
        let available = (total - start_frame) as usize;
        let take = frame_count.min(available);
        let start = start_frame as usize * ch;
        let end = start + take * ch;
        Ok(&self.samples[start..end])
    }

    /// 1フレーム(全チャンネル)を返す。範囲外は`None`。
    pub fn frame_at(&self, frame: u64) -> Option<&[f32]> {
        let ch = self.format.channels as usize;
        if frame >= self.frame_count() {
            return None;
        }
        let start = frame as usize * ch;
        Some(&self.samples[start..start + ch])
    }
}

/// Symphoniaで1ファイルをデコードし、インターリーブf32の`PcmCache`へ展開する。
pub fn decode_file(path: impl AsRef<Path>) -> Result<PcmCache, AudioError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| AudioError::Decode(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| AudioError::NoAudioTrack {
            path: path.to_path_buf(),
        })?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| AudioError::Decode("missing sample rate".into()))?;
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .ok_or_else(|| AudioError::Decode("missing channel layout".into()))?;
    if channels == 0 || channels > u16::MAX as usize {
        return Err(AudioError::UnsupportedChannels { channels });
    }

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| AudioError::Decode(e.to_string()))?;

    let mut samples = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                return Err(AudioError::Decode("decoder reset required".into()));
            }
            Err(e) => return Err(AudioError::Decode(e.to_string())),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| AudioError::Decode(e.to_string()))?;
        append_decoded(&mut samples, decoded);
    }

    PcmCache::from_interleaved(
        samples,
        PcmFormat {
            channels: channels as u16,
            sample_rate,
        },
    )
}

fn append_decoded(dst: &mut Vec<f32>, decoded: AudioBufferRef<'_>) {
    match decoded {
        AudioBufferRef::F32(buf) => interleave(dst, &buf),
        AudioBufferRef::U8(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    let v = buf.chan(c)[f];
                    dst.push((v as f32 - 128.0) / 128.0);
                }
            }
        }
        AudioBufferRef::U16(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    let v = buf.chan(c)[f];
                    dst.push((v as f32 - 32768.0) / 32768.0);
                }
            }
        }
        AudioBufferRef::U24(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    let v = buf.chan(c)[f];
                    dst.push((v.inner() as f32 - 8388608.0) / 8388608.0);
                }
            }
        }
        AudioBufferRef::U32(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    let v = buf.chan(c)[f];
                    dst.push((v as f32 - 2147483648.0) / 2147483648.0);
                }
            }
        }
        AudioBufferRef::S8(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    dst.push(buf.chan(c)[f] as f32 / 128.0);
                }
            }
        }
        AudioBufferRef::S16(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    dst.push(buf.chan(c)[f] as f32 / 32768.0);
                }
            }
        }
        AudioBufferRef::S24(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    dst.push(buf.chan(c)[f].inner() as f32 / 8388608.0);
                }
            }
        }
        AudioBufferRef::S32(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    dst.push(buf.chan(c)[f] as f32 / 2147483648.0);
                }
            }
        }
        AudioBufferRef::F64(buf) => {
            let spec = *buf.spec();
            let ch = spec.channels.count();
            let frames = buf.frames();
            for f in 0..frames {
                for c in 0..ch {
                    dst.push(buf.chan(c)[f] as f32);
                }
            }
        }
    }
}

fn interleave(dst: &mut Vec<f32>, buf: &symphonia::core::audio::AudioBuffer<f32>) {
    let spec = *buf.spec();
    let ch = spec.channels.count();
    let frames = buf.frames();
    for f in 0..frames {
        for c in 0..ch {
            dst.push(buf.chan(c)[f]);
        }
    }
}
