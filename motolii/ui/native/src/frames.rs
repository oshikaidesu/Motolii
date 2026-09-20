//! 出したコマの列。
//!
//! 窓の描画は「出して返る」。GPU が終えたかは**次の拍の頭で 1 度だけ**見て(待たない)、
//! 終わっていた物の合図を鳴らす。UI thread の上に GPU の待ちは無い。
//! 止まっている 1 枚だけは `finish` でその場で待つ —— 絵と窓が同じコマで揃わないといけないので。

use std::collections::VecDeque;
use std::ffi::{c_char, c_void, CString};
use std::time::Duration;

/// `(user, view, surface_id)`: この IOSurface にこの view の絵が入った。
pub type FrameReady = unsafe extern "C" fn(*mut c_void, *const c_char, u32);

/// host が渡した関数と、host にしか意味のない user。
struct Target {
    ready: FrameReady,
    user: *mut c_void,
}

/// 出した 1 コマ。この束が終われば、この view の絵がこの surface に入っている。
struct Flight {
    submission: Option<wgpu::SubmissionIndex>,
    view: CString,
    surface: u32,
}

/// 出した順に並ぶコマの列。
pub struct Frames {
    device: wgpu::Device,
    target: Option<Target>,
    flights: VecDeque<Flight>,
    skipped: u64,
}

impl Frames {
    pub fn new(device: &wgpu::Device) -> Self {
        Self { device: device.clone(), target: None, flights: VecDeque::new(), skipped: 0 }
    }

    /// この surface に描いてよいか。まだ合図の来ていない面なら描かず、飛ばした数を 1 つ数える。
    /// 面ごとに surface は 2 枚あるので、ここが false になるのは 1 コマ以上遅れている時だけ。
    /// `0` は窓を持たない道(試験)の合図で、番人は効かない。
    pub fn claim(&mut self, surface: u32) -> bool {
        if surface != 0 && self.flights.iter().any(|flight| flight.surface == surface) {
            self.skipped += 1;
            return false;
        }
        true
    }

    /// 出した。描き終わりは待たない —— 番号だけ預ける。
    pub fn submitted(&mut self, submission: Option<wgpu::SubmissionIndex>, view: &str, surface: u32) {
        let view = CString::new(view).unwrap_or_default();
        self.flights.push_back(Flight { submission, view, surface });
    }

    /// もう描き終わっている物だけ、合図を鳴らして列から外す。**待たない**(timeout 0 の
    /// 問い合わせ 1 回で fence の値を見るだけ)。再生の次の拍の頭で呼ぶ。
    pub fn collect(&mut self) {
        while self.advance(Some(Duration::ZERO)) {}
    }

    /// 同期の口: 残り全部を待ってから鳴らす。channel 越しの render・止まっている 1 枚・終い。
    pub fn finish(&mut self) {
        while self.advance(None) {}
    }

    /// 番人が飛ばしたコマの数。
    pub fn skipped(&self) -> u64 {
        self.skipped
    }

    /// 合図の相手を差し替える。前の相手宛ての合図はここで消える。
    pub fn set_ready(&mut self, ready: Option<FrameReady>, user: *mut c_void) {
        self.target = ready.map(|ready| Target { ready, user });
    }

    /// 先頭の 1 コマを片付けられたら片付けて `true`。`timeout` が `Some(0)` なら
    /// 問い合わせるだけ(まだなら `false`)、`None` なら終わるまで待つ。
    fn advance(&mut self, timeout: Option<Duration>) -> bool {
        let Some(submission) = self.flights.front().map(|flight| flight.submission.clone()) else {
            return false;
        };
        let drawn = match submission {
            None => true,
            Some(submission) => match self.device.poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout }) {
                Ok(_) => true,
                Err(wgpu::PollError::Timeout) => return false,
                // device が落ちた。絵は無いので鳴らさず、列だけ進める。
                Err(_) => false,
            },
        };
        let Some(flight) = self.flights.pop_front() else { return false };
        if drawn {
            if let Some(target) = &self.target {
                unsafe { (target.ready)(target.user, flight.view.as_ptr(), flight.surface) }
            }
        }
        true
    }
}
