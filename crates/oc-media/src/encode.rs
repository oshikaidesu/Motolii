use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use oc_core::{Fps, FrameDesc, PixelFormat};

use crate::{MediaError, Result};

/// RGBAフレーム列をffmpegサイドカーへパイプしてmp4(H.264)に書き出すエンコーダ。
/// 書き出しループ(oc-export)とテストの土台。
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    frame_size: usize,
}

impl Encoder {
    /// 出力先とフレーム仕様を指定してエンコーダを開く。
    /// `qp0`はほぼロスレス(検証・テスト用)。通常書き出しはfalse(crf 18)。
    pub fn open(out_path: impl AsRef<Path>, desc: &FrameDesc, fps: Fps, qp0: bool) -> Result<Self> {
        assert_eq!(
            desc.format,
            PixelFormat::Rgba8Unorm,
            "Encoder expects RGBA input"
        );
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{}x{}", desc.width, desc.height)])
            .args(["-r", &format!("{}/{}", fps.num, fps.den)])
            .args(["-i", "-", "-c:v", "libx264"]);
        if qp0 {
            // 4:4:4 + qp0でクロマ劣化も抑える(ゴールデンテスト用)
            cmd.args(["-qp", "0", "-pix_fmt", "yuv444p"]);
        } else {
            cmd.args(["-crf", "18", "-pix_fmt", "yuv420p"]);
        }
        cmd.arg(out_path.as_ref())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffmpeg"),
            _ => MediaError::Io(e),
        })?;
        let stdin = child.stdin.take();
        Ok(Self {
            child,
            stdin,
            frame_size: desc.data_size(),
        })
    }

    pub fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        assert_eq!(data.len(), self.frame_size, "frame size mismatch");
        self.stdin
            .as_mut()
            .expect("encoder already finished")
            .write_all(data)?;
        Ok(())
    }

    /// stdinを閉じてffmpegの完了を待つ。必ず呼ぶこと(Dropは強制終了する)。
    pub fn finish(mut self) -> Result<()> {
        drop(self.stdin.take());
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

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
