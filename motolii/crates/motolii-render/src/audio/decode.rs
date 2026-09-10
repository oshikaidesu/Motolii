use std::io::Read;
use std::path::Path;
use std::process::Stdio;

use crate::render::audio::cache::PcmCache;
use crate::render::audio::convert::{canonical_format, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::render::audio::error::{AudioError, Result};
use crate::render::media::{ffmpeg_bin, probe_container, read_child_stderr, select_audio_stream, tool_command, MediaError};

pub const MAX_SAMPLES: u64 = 48_000 * 60 * 60 * 4;

pub fn decode_file(path: impl AsRef<Path>) -> Result<PcmCache> {
    decode_file_audio_ordinal(path, 0)
}

/// 音は ffmpeg に復号させ、48kHz stereo 16bit で受け取る。edit list・encoder delay・
/// start_time・リサンプルは ffmpeg の側で片付く。
pub fn decode_file_audio_ordinal(path: impl AsRef<Path>, ordinal: u32) -> Result<PcmCache> {
    let path = path.as_ref();
    let info = probe_container(path)?;
    if info.audio_streams.is_empty() {
        return Err(AudioError::NoAudioTrack);
    }
    select_audio_stream(&info, ordinal).map_err(|_| AudioError::StreamNotFound { ordinal })?;

    let mut child = tool_command(ffmpeg_bin())
        .args(["-v", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-map", &format!("0:a:{ordinal}"), "-vn", "-sn", "-dn"])
        .args(["-f", "s16le", "-ac", &CANONICAL_CHANNELS.to_string(), "-ar", &CANONICAL_SAMPLE_RATE.to_string(), "-"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => AudioError::Media(MediaError::ToolNotFound("ffmpeg")),
            _ => AudioError::Io(e),
        })?;

    let mut stderr = child.stderr.take().expect("piped");
    let stderr_reader = std::thread::spawn(move || read_child_stderr(&mut stderr).unwrap_or_default());

    let mut stdout = child.stdout.take().expect("piped");
    let limit_bytes = MAX_SAMPLES as usize * std::mem::size_of::<i16>();
    let mut bytes = Vec::new();
    let mut chunk = vec![0u8; 1 << 16];
    loop {
        let n = stdout.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..n]);
        if bytes.len() > limit_bytes {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AudioError::SampleCountLimit {
                observed: (bytes.len() / std::mem::size_of::<i16>()) as u64,
                limit: MAX_SAMPLES,
            });
        }
    }
    let status = child.wait()?;
    let stderr = stderr_reader.join().unwrap_or_default();
    if !status.success() {
        return Err(AudioError::Media(MediaError::Ffmpeg(stderr)));
    }

    let samples: Vec<i16> = bytes
        .chunks_exact(std::mem::size_of::<i16>())
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    if samples.is_empty() {
        return Err(AudioError::NoAudioTrack);
    }
    PcmCache::from_interleaved_i16(samples, canonical_format())
}
