use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use motolii_core::{Fps, FrameDesc, PixelFormat};

use crate::{read_child_stderr, MediaError, Result};

/// RGBAフレーム列をffmpegサイドカーへパイプしてmp4(H.264)に書き出すエンコーダ。
/// 書き出しループ(motolii-export)とテストの土台。
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    frame_size: usize,
}

impl Encoder {
    /// 出力先とフレーム仕様を指定してエンコーダを開く。
    /// `qp0`はほぼロスレス(検証・テスト用)。通常書き出しはfalse(crf 18)。
    pub fn open(out_path: impl AsRef<Path>, desc: &FrameDesc, fps: Fps, qp0: bool) -> Result<Self> {
        Self::open_with_command("ffmpeg", out_path, desc, fps, qp0)
    }

    /// 実行ファイルパスを明示してエンコーダを開く(テスト用)。
    #[doc(hidden)]
    pub fn open_with_command(
        program: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        desc: &FrameDesc,
        fps: Fps,
        qp0: bool,
    ) -> Result<Self> {
        assert_eq!(
            desc.format,
            PixelFormat::Rgba8Unorm,
            "Encoder expects RGBA input"
        );
        let mut cmd = Command::new(program.as_ref());
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
        // 出力の色タグを明示する(レビュー指摘#5): タグ無しRGB由来のmp4は
        // プレイヤーごとに解釈が割れ「書き出したら色が違う」を生む。
        // v1はBT.709 limited固定。RGB→YUVの変換行列自体もbt709を強制する。
        cmd.args([
            "-vf",
            "scale=out_color_matrix=bt709:out_range=tv",
            "-colorspace",
            "bt709",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-color_range",
            "tv",
        ]);
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
        let mut err = String::new();
        if let Some(stderr) = self.child.stderr.as_mut() {
            err = read_child_stderr(stderr)?;
        }
        let status = self.child.wait()?;
        if !status.success() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::ColorSpace;
    use std::time::{Duration, Instant};

    #[cfg(unix)]
    #[test]
    fn finish_drains_stderr_before_wait_without_deadlock() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!(
            "motolii-media-stderr-flood-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let fake_ffmpeg = dir.join("fake-ffmpeg");
        std::fs::write(
            &fake_ffmpeg,
            "#!/bin/sh\n\
             cat >/dev/null\n\
             i=0\n\
             while [ \"$i\" -lt 20000 ]; do\n\
               echo \"err $i\" 1>&2\n\
               i=$((i+1))\n\
             done\n\
             exit 0\n",
        )
        .unwrap();
        std::fs::set_permissions(&fake_ffmpeg, std::fs::Permissions::from_mode(0o755)).unwrap();

        let desc = FrameDesc::packed(4, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        let out = dir.join("out.mp4");
        let mut enc =
            Encoder::open_with_command(&fake_ffmpeg, &out, &desc, Fps::new(1, 1), true).unwrap();
        enc.write_frame(&vec![0u8; desc.data_size()]).unwrap();

        let started = Instant::now();
        enc.finish().expect("finish failed");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "finish took too long ({:?}); stderr drain may have deadlocked",
            started.elapsed()
        );

        let _ = std::fs::remove_dir_all(dir);
    }
}
