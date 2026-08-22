//! JKL シャトルの状態機械(B21 第1切片、map 採用予定 1097/1098 Play Forward・
//! 1100/1101 Play Reverse・1135/1136 Stop・1125/1126/1127 Shuttle Left/Right/
//! Stop・1056/1058 Fast Forward/Reverse)。**意味の純関数** — 実時間 clock も
//! `Session` も知らない(`nav.rs` と同じ型)。実際の再生(1 tick ごとの
//! playhead 前進と `work_area::advanced_playhead` によるループ折り返し)は
//! shell の PlaybackClock(A2)が担う — supervisor 結線。
//!
//! ## 意味(Premiere/Resolve/AE 共通の JKL 慣習を借用 — 正典 拘束7)
//! - L(順)/ J(逆): 停止中・逆向き中は等速 1x で走り出し、**同方向への連打で
//!   倍速**(1→2→4→8、上限 [`MAX_SHUTTLE_RATE`])。逆向きへの打鍵は方向転換
//!   (等速 1x へ戻る — 加速の慣性を持ち越さない)
//! - K: 停止(map 1127/1135/1136)
//! - Shift+L / Shift+J(map 1056/1058 Fast Forward/Reverse)は keymap 層が
//!   同じ [`ShuttleCommand::Forward`]/[`Reverse`] を連打相当で解決する想定
//!   (このモジュールはキーを知らない — 正典 拘束6)
//!
//! ## 正典との keymap 衝突(逸脱報告)
//! 正典 §8.1 は J/K を JumpPrev/NextMeaningPoint、§5 は L をループ on/off の
//! **既定割当**としている。JKL シャトルの既定割当はこれと衝突するが、正典
//! 拘束6「既定割当は仮置き・意味アクションだけが本決め」により、ここでは
//! **意味だけ**を定義し割当の裁定は keymap 層(supervisor)へ委ねる。

/// シャトルの上限倍率(1→2→4→8)。仮の既定(裁定146 と同じ「実装時に仕様化、
/// 後から差し替え可」の身分) — Premiere は 32x、Resolve は 64x まで伸びるが、
/// フレーム単位編集で目が追える範囲として 8x を初期値に採る。
pub const MAX_SHUTTLE_RATE: i32 = 8;

/// JKL の3動詞(pane-local [`crate::Message::Shuttle`] が運ぶ)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShuttleCommand {
    /// J: 逆再生(連打で倍速)。
    Reverse,
    /// K: 停止。
    Stop,
    /// L: 順再生(連打で倍速)。
    Forward,
}

/// シャトルの現在状態。`rate` は符号つき倍率: `0` = 停止、`+n` = 順再生 n 倍、
/// `-n` = 逆再生 n 倍(n ∈ {1,2,4,8})。1 tick に進むフレーム数 = `rate`
/// (等速 1x が 1 frame/tick — 実時間との対応は shell clock の領分)。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShuttleState {
    pub rate: i32,
}

impl ShuttleState {
    pub fn stopped() -> Self {
        Self::default()
    }

    pub fn is_stopped(&self) -> bool {
        self.rate == 0
    }

    /// 状態遷移の全て(この1関数が状態機械の正本 — テストはここを検分する)。
    #[must_use]
    pub fn apply(self, command: ShuttleCommand) -> Self {
        let rate = match command {
            ShuttleCommand::Stop => 0,
            ShuttleCommand::Forward => {
                if self.rate <= 0 {
                    1 // 停止中・逆向き中は等速で走り出す(方向転換は 1x へ戻る)。
                } else {
                    (self.rate * 2).min(MAX_SHUTTLE_RATE)
                }
            }
            ShuttleCommand::Reverse => {
                if self.rate >= 0 {
                    -1
                } else {
                    (self.rate * 2).max(-MAX_SHUTTLE_RATE)
                }
            }
        };
        Self { rate }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_from_stop_plays_forward_at_1x_and_doubles_up_to_the_cap() {
        let mut s = ShuttleState::stopped();
        let mut seen = Vec::new();
        for _ in 0..5 {
            s = s.apply(ShuttleCommand::Forward);
            seen.push(s.rate);
        }
        assert_eq!(seen, vec![1, 2, 4, 8, 8], "1→2→4→8 で頭打ち");
    }

    #[test]
    fn j_mirrors_l_in_reverse() {
        let mut s = ShuttleState::stopped();
        let mut seen = Vec::new();
        for _ in 0..5 {
            s = s.apply(ShuttleCommand::Reverse);
            seen.push(s.rate);
        }
        assert_eq!(seen, vec![-1, -2, -4, -8, -8]);
    }

    #[test]
    fn k_stops_from_any_state() {
        let fast = ShuttleState { rate: 8 };
        assert!(fast.apply(ShuttleCommand::Stop).is_stopped());
        let back = ShuttleState { rate: -4 };
        assert!(back.apply(ShuttleCommand::Stop).is_stopped());
        assert!(ShuttleState::stopped().apply(ShuttleCommand::Stop).is_stopped());
    }

    /// 方向転換は等速 1x へ戻る — 倍速の慣性を逆向きへ持ち越さない。
    #[test]
    fn changing_direction_resets_to_1x() {
        let fast_forward = ShuttleState { rate: 8 };
        assert_eq!(fast_forward.apply(ShuttleCommand::Reverse).rate, -1);
        let fast_reverse = ShuttleState { rate: -8 };
        assert_eq!(fast_reverse.apply(ShuttleCommand::Forward).rate, 1);
    }
}
