//! oc-media: ffmpeg/ffprobeをサイドカープロセスとして使うメディアI/O。
//!
//! 方針(落とし穴B-2対策): FFmpegはリンクせずサイドカーで叩く。
//! - リンク・ライセンス問題を回避(LGPL/GPL・コーデック特許)
//! - デコーダのクラッシュがプロセス境界で隔離される
//! - rawvideoパイプなので入出力が決定的
//!
//! デコードは常に「RGBA・タイトパッキング」に正規化してから返す。
//! YUV→RGB変換・色空間の解釈はffmpeg側に寄せ、oc-gpu側の変換シェーダ実装(M1-T3)
//! までの間もパイプライン全体をRGBAで一貫させる。

mod decode;
mod encode;
mod probe;

use std::process::Command;

pub use decode::{read_frame_at, FrameReader};
pub use encode::Encoder;
pub use probe::{probe, MediaInfo};

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("ffmpeg/ffprobe not found on PATH: {0}")]
    ToolNotFound(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("probe failed: {0}")]
    Probe(String),
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),
}

pub type Result<T> = std::result::Result<T, MediaError>;

/// ffmpeg/ffprobeがPATHにあるか。テストはこれがfalseならskipする。
pub fn tools_available() -> bool {
    let ok = |bin: &str| {
        Command::new(bin)
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    ok("ffmpeg") && ok("ffprobe")
}
