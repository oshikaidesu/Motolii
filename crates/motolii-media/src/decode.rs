use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use motolii_core::{CpuFrame, FrameDesc, PixelFormat, RationalTime};

use crate::{read_child_stderr, MediaError, MediaInfo, Result};

/// 進行中の読み出し/シーク要求を論理キャンセルするトークン。
///
/// cancel と kill は別系統: 本トークンはフラグのみ立てる。ブロッキング read を
/// 解放するには [`FrameReaderKillHandle::kill`] を別途呼ぶ(U5「最新要求のみ」の協調)。
#[derive(Clone, Debug)]
pub struct FrameReaderCancel {
    cancelled: Arc<AtomicBool>,
}

impl FrameReaderCancel {
    /// この読み出し要求をキャンセル済みにする。
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// キャンセル済みか。
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// ffmpeg 子プロセスを外から終了させるハンドル(cancel とは分離)。
#[derive(Clone, Debug)]
pub struct FrameReaderKillHandle {
    child: Arc<Mutex<Child>>,
}

impl FrameReaderKillHandle {
    /// 子プロセスを kill し wait する。ブロッキング read 中のスレッド解放用。
    pub fn kill(&self) -> Result<()> {
        let mut child = self
            .child
            .lock()
            .map_err(|_| MediaError::Ffmpeg("frame reader child lock poisoned".into()))?;
        if let Err(e) = child.kill() {
            return Err(MediaError::Io(e));
        }
        let _ = child.wait()?;
        Ok(())
    }
}

/// ffmpegサイドカーから**生YUV420p**フレームを順に読むリーダー。
///
/// 色変換はffmpegにやらせない(レビュー指摘#2): YUV→RGBは必ずmotolii-gpuの
/// 変換シェーダ(FrameDesc.color_space準拠)を通す。ffmpegの暗黙rgba変換は
/// CPUで走る上にmatrix/rangeタグの解釈が暗黙で、B-3(色事故)の温床になるため。
///
/// シークはffmpegの入力`-ss`(直前キーフレームへシーク→目的時刻までデコード読み捨て)
/// を使うためフレーム正確。シーク先は「目的フレームの半フレーム手前」を指定することで、
/// 秒数の10進文字列化による丸めがフレーム境界をまたぐことを防ぐ。
pub struct FrameReader {
    child: Arc<Mutex<Child>>,
    cancel: FrameReaderCancel,
    desc: FrameDesc,
    frame_size: usize,
    fps: motolii_core::Fps,
    next_frame_index: i64,
}

impl FrameReader {
    /// `start_frame`から順方向に読むリーダーを開く。
    pub fn open(path: impl AsRef<Path>, info: &MediaInfo, start_frame: i64) -> Result<Self> {
        if start_frame < 0 {
            return Err(MediaError::InvalidStartFrame(start_frame));
        }
        let desc = FrameDesc::try_yuv(
            info.width,
            info.height,
            PixelFormat::Yuv420p,
            info.color_space,
        )
        .map_err(|e| MediaError::Probe(e.to_string()))?;

        let mut cmd = Command::new("ffmpeg");
        // autorotateはffmpegのバージョン/ビルドで既定挙動が揺れた歴史があるため
        // 使わない(レビュー指摘#5)。回転はprobeのrotationから自前で明示指定し、
        // どのffmpegでも決定的な出力にする。
        cmd.args(["-v", "error", "-nostdin", "-noautorotate"]);
        if start_frame > 0 {
            // (start_frame - 0.5) / fps 秒へシーク
            let target = (start_frame as f64 - 0.5) * info.fps.den() as f64 / info.fps.num() as f64;
            cmd.args(["-ss", &format!("{target:.6}")]);
        }
        cmd.arg("-i").arg(path.as_ref());
        if let Some(vf) = rotation_filter(info.rotation) {
            cmd.args(["-vf", vf]);
        }
        cmd.args(["-f", "rawvideo", "-pix_fmt", "yuv420p", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MediaError::ToolNotFound("ffmpeg"),
            _ => MediaError::Io(e),
        })?;

        let child = Arc::new(Mutex::new(child));
        let cancel = FrameReaderCancel {
            cancelled: Arc::new(AtomicBool::new(false)),
        };

        Ok(Self {
            child,
            cancel,
            frame_size: desc.data_size(),
            desc,
            fps: info.fps,
            next_frame_index: start_frame,
        })
    }

    pub fn desc(&self) -> &FrameDesc {
        &self.desc
    }

    /// 次に読むフレーム番号(開いた直後は `start_frame`)。
    pub fn next_frame_index(&self) -> i64 {
        self.next_frame_index
    }

    /// 論理キャンセル用トークン(複製可。別スレッドから古い要求を打ち切る)。
    pub fn cancel_token(&self) -> FrameReaderCancel {
        self.cancel.clone()
    }

    /// 子プロセス kill 用ハンドル(cancel とは別系統)。
    pub fn kill_handle(&self) -> FrameReaderKillHandle {
        FrameReaderKillHandle {
            child: Arc::clone(&self.child),
        }
    }

    /// 次のフレームを読む。ストリーム終端でNone。キャンセル済みなら [`MediaError::Cancelled`]。
    pub fn next_frame(&mut self) -> Result<Option<CpuFrame>> {
        if self.cancel.is_cancelled() {
            return Err(MediaError::Cancelled);
        }

        let mut child = self
            .child
            .lock()
            .map_err(|_| MediaError::Ffmpeg("frame reader child lock poisoned".into()))?;
        let stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| MediaError::Ffmpeg("frame reader stdout not piped".into()))?;

        let mut data = vec![0u8; self.frame_size];
        let mut filled = 0;
        while filled < self.frame_size {
            if self.cancel.is_cancelled() {
                return Err(MediaError::Cancelled);
            }
            match stdout.read(&mut data[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    if self.cancel.is_cancelled() {
                        return Err(MediaError::Cancelled);
                    }
                    return Err(MediaError::Io(e));
                }
            }
        }
        if self.cancel.is_cancelled() {
            return Err(MediaError::Cancelled);
        }
        if filled == 0 {
            let cancelled = self.cancel.is_cancelled();
            return check_child_exit(&mut child, cancelled).map(|_| None);
        }
        if filled < self.frame_size {
            return Err(MediaError::Ffmpeg(format!(
                "truncated frame: got {filled} of {} bytes",
                self.frame_size
            )));
        }
        let pts = RationalTime::try_from_frame(self.next_frame_index, self.fps)?;
        self.next_frame_index += 1;
        Ok(Some(CpuFrame::new(self.desc, pts, data)))
    }
}

fn check_child_exit(child: &mut Child, cancelled: bool) -> Result<()> {
    let mut err = String::new();
    if let Some(stderr) = child.stderr.as_mut() {
        err = read_child_stderr(stderr)?;
    }
    let status = child.wait()?;
    if !status.success() {
        if cancelled {
            return Err(MediaError::Cancelled);
        }
        return Err(MediaError::Ffmpeg(err));
    }
    Ok(())
}

impl Drop for FrameReader {
    fn drop(&mut self) {
        // 途中で読むのをやめた場合にゾンビ化させない
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// probeのrotation(度)に対応する明示回転フィルタ。
/// 方向の正しさ(時計/反時計)は実スマホ素材での目視確認(M1-T11)で最終検証する。
fn rotation_filter(rotation: i64) -> Option<&'static str> {
    match rotation.rem_euclid(360) {
        90 => Some("transpose=2"), // 反時計回り90
        180 => Some("hflip,vflip"),
        270 => Some("transpose=1"), // 時計回り90
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::ColorSpace;

    fn sample_info() -> MediaInfo {
        MediaInfo {
            width: 64,
            height: 48,
            fps: motolii_core::Fps::try_new(30, 1).unwrap(),
            duration: None,
            nb_frames: None,
            color_space: ColorSpace::Rec709Limited,
            rotation: 0,
        }
    }

    #[test]
    fn frame_reader_rejects_negative_start_frame() {
        let result = FrameReader::open("missing.mp4", &sample_info(), -1);
        assert!(matches!(result, Err(MediaError::InvalidStartFrame(-1))));
    }

    #[test]
    fn cancel_token_and_kill_handle_are_independent() {
        let cancel = FrameReaderCancel {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        assert!(!cancel.is_cancelled());
        cancel.cancel();
        assert!(cancel.is_cancelled());
        // kill ハンドルは cancel とは別型(コンパイル時に分離を保証)
        let _ = FrameReaderKillHandle {
            child: Arc::new(Mutex::new(
                Command::new("true").spawn().expect("spawn true"),
            )),
        };
    }
}
