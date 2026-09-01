#[cfg(test)]
#[path = "../../../tests/testkit/mod.rs"]
mod testkit;

use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::doc::core::{Fps, FrameDesc, PixelFormat};

use crate::render::media::{read_child_stderr, MediaError, Result};

pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    frame_size: usize,
}

impl Encoder {
    pub fn open(out_path: impl AsRef<Path>, desc: &FrameDesc, fps: Fps, qp0: bool) -> Result<Self> {
        Self::open_with_command("ffmpeg", out_path, desc, fps, qp0)
    }

    #[doc(hidden)]
    pub fn open_with_command(
        program: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        desc: &FrameDesc,
        fps: Fps,
        qp0: bool,
    ) -> Result<Self> {
        if desc.format != PixelFormat::Rgba8Unorm {
            return Err(MediaError::UnsupportedEncoderFormat(desc.format));
        }
        let mut cmd = Command::new(program.as_ref());
        cmd.args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{}x{}", desc.width, desc.height)])
            .args(["-r", &format!("{}/{}", fps.num(), fps.den())])
            .args(["-i", "-", "-c:v", "libx264"]);
        if qp0 {
            cmd.args(["-qp", "0", "-pix_fmt", "yuv444p"]);
        } else {
            cmd.args(["-crf", "18", "-pix_fmt", "yuv420p"]);
        }
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
        if data.len() != self.frame_size {
            return Err(MediaError::FrameSizeMismatch {
                expected: self.frame_size,
                got: data.len(),
            });
        }
        self.stdin
            .as_mut()
            .expect("encoder already finished")
            .write_all(data)?;
        Ok(())
    }

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
