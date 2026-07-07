use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use oc_core::{ColorSpace, CpuFrame, FrameDesc, PixelFormat, RationalTime};

use crate::{MediaError, MediaInfo, Result};

/// ffmpegサイドカーからRGBAフレームを順に読むリーダー。
///
/// シークはffmpegの入力`-ss`(直前キーフレームへシーク→目的時刻までデコード読み捨て)
/// を使うためフレーム正確。シーク先は「目的フレームの半フレーム手前」を指定することで、
/// 秒数の10進文字列化による丸めがフレーム境界をまたぐことを防ぐ。
pub struct FrameReader {
    child: Child,
    desc: FrameDesc,
    frame_size: usize,
    fps: oc_core::Fps,
    next_frame_index: i64,
}

impl FrameReader {
    /// `start_frame`から順方向に読むリーダーを開く。
    pub fn open(path: impl AsRef<Path>, info: &MediaInfo, start_frame: i64) -> Result<Self> {
        assert!(start_frame >= 0, "start_frame must be >= 0");
        let desc = FrameDesc::packed(
            info.width,
            info.height,
            PixelFormat::Rgba8Unorm,
            // ffmpegにsRGB相当のRGBAへ変換させて受ける
            ColorSpace::Srgb,
            false,
        );

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-v", "error", "-nostdin"]);
        if start_frame > 0 {
            // (start_frame - 0.5) / fps 秒へシーク
            let target = (start_frame as f64 - 0.5) * info.fps.den as f64 / info.fps.num as f64;
            cmd.args(["-ss", &format!("{target:.6}")]);
        }
        cmd.arg("-i")
            .arg(path.as_ref())
            .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffmpeg"),
            _ => MediaError::Io(e),
        })?;

        Ok(Self {
            child,
            frame_size: desc.data_size(),
            desc,
            fps: info.fps,
            next_frame_index: start_frame,
        })
    }

    pub fn desc(&self) -> &FrameDesc {
        &self.desc
    }

    /// 次のフレームを読む。ストリーム終端でNone。
    pub fn next_frame(&mut self) -> Result<Option<CpuFrame>> {
        let stdout = self.child.stdout.as_mut().expect("stdout piped");
        let mut data = vec![0u8; self.frame_size];
        let mut filled = 0;
        while filled < self.frame_size {
            match stdout.read(&mut data[filled..])? {
                0 => break,
                n => filled += n,
            }
        }
        if filled == 0 {
            self.check_exit()?;
            return Ok(None);
        }
        if filled < self.frame_size {
            return Err(MediaError::Ffmpeg(format!(
                "truncated frame: got {filled} of {} bytes",
                self.frame_size
            )));
        }
        let pts = RationalTime::from_frame(self.next_frame_index, self.fps);
        self.next_frame_index += 1;
        Ok(Some(CpuFrame::new(self.desc, pts, data)))
    }

    fn check_exit(&mut self) -> Result<()> {
        let status = self.child.wait()?;
        if !status.success() {
            let mut err = String::new();
            if let Some(stderr) = self.child.stderr.as_mut() {
                let _ = stderr.read_to_string(&mut err);
            }
            return Err(MediaError::Ffmpeg(err));
        }
        Ok(())
    }
}

impl Drop for FrameReader {
    fn drop(&mut self) {
        // 途中で読むのをやめた場合にゾンビ化させない
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 指定フレーム1枚だけを読む(スクラブ・テスト用のショートカット)。
pub fn read_frame_at(
    path: impl AsRef<Path>,
    info: &MediaInfo,
    frame_index: i64,
) -> Result<CpuFrame> {
    let mut reader = FrameReader::open(path, info, frame_index)?;
    reader
        .next_frame()?
        .ok_or_else(|| MediaError::Ffmpeg(format!("frame {frame_index} out of range")))
}
