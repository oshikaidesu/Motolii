use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Stdio};
use std::thread::JoinHandle;

use crate::doc::core::{Fps, FrameDesc, PixelFormat};

use crate::render::media::{read_child_stderr, MediaError, Result};

pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    frame_size: usize,
    /// ffmpeg's stderr, read from the moment it starts: a pipe nobody drains can fill and stall ffmpeg, and with
    /// it our writes. Joined where the text is needed (a broken pipe, `finish`).
    stderr: Option<JoinHandle<String>>,
}

impl Encoder {
    pub fn open(out_path: impl AsRef<Path>, desc: &FrameDesc, fps: Fps, qp0: bool) -> Result<Self> {
        Self::open_with_command_and_audio(crate::render::media::ffmpeg_bin(), out_path, desc, fps, qp0, None)
    }

    pub fn open_with_audio(
        out_path: impl AsRef<Path>,
        desc: &FrameDesc,
        fps: Fps,
        qp0: bool,
        audio_path: &Path,
    ) -> Result<Self> {
        Self::open_with_command_and_audio(crate::render::media::ffmpeg_bin(), out_path, desc, fps, qp0, Some(audio_path))
    }

    #[doc(hidden)]
    pub fn open_with_command(
        program: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        desc: &FrameDesc,
        fps: Fps,
        qp0: bool,
    ) -> Result<Self> {
        Self::open_with_command_and_audio(program, out_path, desc, fps, qp0, None)
    }

    fn open_with_command_and_audio(
        program: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        desc: &FrameDesc,
        fps: Fps,
        qp0: bool,
        audio_path: Option<&Path>,
    ) -> Result<Self> {
        if desc.format != PixelFormat::Rgba8Unorm {
            return Err(MediaError::UnsupportedEncoderFormat(desc.format));
        }
        let mut cmd = crate::render::media::tool_command(program.as_ref());
        cmd.args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{}x{}", desc.width, desc.height)])
            .args(["-r", &format!("{}/{}", fps.num(), fps.den())])
            .args(["-i", "-"]);
        if let Some(audio_path) = audio_path {
            cmd.args(["-f", "f32le", "-ar", "48000", "-ac", "2", "-i"])
                .arg(audio_path);
        }
        cmd.args(["-map", "0:v:0"]);
        if audio_path.is_some() {
            cmd.args(["-map", "1:a:0", "-c:a", "aac", "-b:a", "192k", "-shortest"]);
        }
        cmd.args(["-c:v", "libx264"]);
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
        let stderr = child
            .stderr
            .take()
            .map(|mut pipe| std::thread::spawn(move || read_child_stderr(&mut pipe).unwrap_or_default()));
        Ok(Self {
            child,
            stdin,
            frame_size: desc.data_size(),
            stderr,
        })
    }

    pub fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        if data.len() != self.frame_size {
            return Err(MediaError::FrameSizeMismatch {
                expected: self.frame_size,
                got: data.len(),
            });
        }
        let written = self
            .stdin
            .as_mut()
            .expect("encoder already finished")
            .write_all(data);
        match written {
            Ok(()) => Ok(()),
            // ffmpeg が落ちると stdin が EPIPE で返る。本当の理由(No space left 等)は stderr に在る。
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {
                let _ = self.child.wait();
                let err = self.stderr.take().and_then(|reader| reader.join().ok()).unwrap_or_default();
                if err.trim().is_empty() {
                    Err(MediaError::Io(error))
                } else {
                    Err(MediaError::Ffmpeg(err))
                }
            }
            Err(error) => Err(MediaError::Io(error)),
        }
    }

    pub fn finish(self) -> Result<()> {
        self.finish_unless(|| false)
    }

    /// [finish], but [cancelled] is asked while ffmpeg flushes its tail: a cancel then kills it at once (no timeout is
    /// guessed for how long a flush may take). A cancelled finish returns `MediaError::Cancelled`.
    pub fn finish_unless(mut self, cancelled: impl Fn() -> bool) -> Result<()> {
        drop(self.stdin.take());
        let status = loop {
            if let Some(status) = self.child.try_wait()? {
                break status;
            }
            if cancelled() {
                let _ = self.child.kill();
                let _ = self.child.wait();
                return Err(MediaError::Cancelled);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let err = self.stderr.take().and_then(|reader| reader.join().ok()).unwrap_or_default();
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

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::ColorSpace;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::mpsc;
    use std::time::Duration;

    fn fake_tool(dir: &Path, name: &str, script: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn desc() -> FrameDesc {
        FrameDesc::try_packed(256, 256, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false).unwrap()
    }

    /// An encoder that talks a lot on stderr before it reads a frame: if nobody drains stderr, its pipe fills,
    /// the encoder blocks on it and never reads stdin, and our frame write never returns.
    #[test]
    fn a_chatty_encoder_does_not_stall_the_frames() {
        let dir = tempfile::tempdir().unwrap();
        let tool = fake_tool(dir.path(), "chatty", "head -c 1048576 /dev/zero | tr '\\0' e >&2\ncat > /dev/null\nexit 0");
        let out = dir.path().join("out.mp4");
        let (done, result) = mpsc::channel();
        std::thread::spawn(move || {
            let run = || -> Result<()> {
                let mut encoder = Encoder::open_with_command(&tool, &out, &desc(), Fps::try_new(30, 1).unwrap(), false)?;
                for _ in 0..8 {
                    encoder.write_frame(&vec![0u8; desc().data_size()])?;
                }
                encoder.finish()
            };
            let _ = done.send(run());
        });
        let outcome = result.recv_timeout(Duration::from_secs(20)).expect("the frame writes stalled on an undrained stderr");
        outcome.unwrap();
    }

    /// An encoder that never finishes its tail (stdin closed, it keeps going) is stopped by a cancel, promptly.
    #[test]
    fn a_cancel_stops_an_encoder_that_will_not_finish() {
        let dir = tempfile::tempdir().unwrap();
        let tool = fake_tool(dir.path(), "stuck", "cat > /dev/null
sleep 600");
        let out = dir.path().join("out.mp4");
        let encoder = Encoder::open_with_command(&tool, &out, &desc(), Fps::try_new(30, 1).unwrap(), false).unwrap();
        let asked = std::time::Instant::now();
        let result = encoder.finish_unless(|| asked.elapsed() > Duration::from_millis(150));
        assert!(matches!(result, Err(MediaError::Cancelled)));
        assert!(asked.elapsed() < Duration::from_secs(5), "the cancel waited for the stuck encoder");
    }

    /// When the encoder fails, its own words (from stderr) are what comes back, not a bare broken pipe.
    #[test]
    fn a_failing_encoder_reports_what_it_said() {
        let dir = tempfile::tempdir().unwrap();
        let tool = fake_tool(dir.path(), "full", "echo 'No space left on device' >&2\nexit 1");
        let out = dir.path().join("out.mp4");
        let mut encoder = Encoder::open_with_command(&tool, &out, &desc(), Fps::try_new(30, 1).unwrap(), false).unwrap();
        let frame = vec![0u8; desc().data_size()];
        let mut failure = None;
        for _ in 0..64 {
            if let Err(error) = encoder.write_frame(&frame) {
                failure = Some(error);
                break;
            }
        }
        let error = match failure {
            Some(error) => error,
            None => encoder.finish().unwrap_err(),
        };
        assert!(format!("{error}").contains("No space left"), "got {error}");
    }
}
