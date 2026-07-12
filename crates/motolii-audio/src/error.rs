use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("no default audio track in {path}")]
    NoAudioTrack { path: PathBuf },
    #[error("unsupported channel layout: {channels}")]
    UnsupportedChannels { channels: usize },
    #[error("no audio output device: {detail}")]
    NoOutputDevice { detail: String },
    #[error("no output stream config matching {sample_rate} Hz / {channels} ch: {detail}")]
    UnsupportedOutputConfig {
        sample_rate: u32,
        channels: u16,
        detail: String,
    },
    #[error("cpal error: {0}")]
    Cpal(String),
    #[error("read past end of PCM cache: frame {frame} >= {total_frames}")]
    ReadPastEnd { frame: u64, total_frames: u64 },
}
