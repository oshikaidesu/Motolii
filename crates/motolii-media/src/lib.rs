//! motolii-media: ffmpeg/ffprobeをサイドカープロセスとして使うメディアI/O。
//!
//! 方針(落とし穴B-2対策): FFmpegはリンクせずサイドカーで叩く。
//! - リンク・ライセンス問題を回避(LGPL/GPL・コーデック特許)
//! - デコーダのクラッシュがプロセス境界で隔離される
//! - rawvideoパイプなので入出力が決定的
//!
//! デコードは常に「RGBA・タイトパッキング」に正規化してから返す。
//! YUV→RGB変換・色空間の解釈はffmpeg側に寄せ、motolii-gpu側の変換シェーダ実装(M1-T3)
//! までの間もパイプライン全体をRGBAで一貫させる。

mod decode;
mod encode;
mod probe;

use std::io::Read;
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
    #[error(transparent)]
    RationalTime(#[from] motolii_core::RationalTimeError),
    #[error("invalid start frame: {0}")]
    InvalidStartFrame(i64),
    #[error("encoder expects RGBA input, got {0:?}")]
    UnsupportedEncoderFormat(motolii_core::PixelFormat),
    #[error("frame size mismatch: expected {expected} bytes, got {got}")]
    FrameSizeMismatch { expected: usize, got: usize },
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),
}

pub type Result<T> = std::result::Result<T, MediaError>;

const MAX_STDERR_BYTES: usize = 64 * 1024;

/// 子プロセスのstderrをEOFまで読む。`wait()`前に呼びパイプ詰まりデッドロックを防ぐ。
pub(crate) fn read_child_stderr(stderr: &mut impl Read) -> std::io::Result<String> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match stderr.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if out.len() < MAX_STDERR_BYTES {
                    let take = (MAX_STDERR_BYTES - out.len()).min(n);
                    out.extend_from_slice(&chunk[..take]);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// ffmpeg/ffprobeがPATHにあるか。アプリ起動時チェック(CLI)用。
/// **テストでは使わないこと** — テストのスキップ判定は
/// `motolii_testkit::ffmpeg_or_skip`を通す(M2E-1: 手書きスキップは
/// REQUIRE環境のスキップ禁止を迂回するため走査denyの対象)。
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

/// 依存するffmpeg/ffprobeの最低メジャーバージョン。
/// 根拠(レビュー指摘#5): side_data回転のJSON出力、scale=out_color_matrix、
/// -display_rotation 等の挙動をこの版以降で確認している。
pub const MIN_FFMPEG_MAJOR: u32 = 6;

/// アプリ起動時に呼ぶ: ffmpeg/ffprobeの存在とバージョンを検証する。
/// バージョン差による挙動ズレ(回転・色タグ)はサポート地獄になるため、
/// 満たさない場合は起動段階で明確に失敗させる。
pub fn verify_tool_versions() -> Result<(u32, u32)> {
    let major = |bin: &'static str| -> Result<u32> {
        let out = Command::new(bin).arg("-version").output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MediaError::ToolNotFound(bin)
            } else {
                MediaError::Io(e)
            }
        })?;
        let text = String::from_utf8_lossy(&out.stdout);
        // 例: "ffmpeg version 6.1.1-3ubuntu5" / "version n7.0" / "version N-113445-g..."
        // (gitマスター) / まれに日付版。判定は false-negative に弱くしない(第3回
        // レビュー#5): 明確に古いと分かる場合だけ弾き、判定不能・特殊ビルドは
        // 警告して通す(起動を止めない)。
        let tok = text
            .split_whitespace()
            .nth(2)
            .unwrap_or("")
            .trim_start_matches(['n', 'N']);
        let digits: String = tok.chars().take_while(|c| c.is_ascii_digit()).collect();
        match digits.parse::<u32>() {
            Ok(major) if major >= 1000 => {
                // 日付/ビルド番号系(gitマスター "N-123456" 等) → 判定不能として通す
                eprintln!(
                    "warning: {bin} version '{tok}' looks like a snapshot build; \
                     assuming >= {MIN_FFMPEG_MAJOR}"
                );
                Ok(0)
            }
            Ok(major) if major < MIN_FFMPEG_MAJOR => Err(MediaError::Probe(format!(
                "{bin} major version {major} < required {MIN_FFMPEG_MAJOR}"
            ))),
            Ok(major) => Ok(major),
            Err(_) => {
                eprintln!(
                    "warning: {bin} version '{tok}' is unparsable; \
                     assuming >= {MIN_FFMPEG_MAJOR}"
                );
                Ok(0)
            }
        }
    };
    Ok((major("ffmpeg")?, major("ffprobe")?))
}
