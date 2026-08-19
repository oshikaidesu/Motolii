//! 再生の tick / toggle — **`iced` shell(`motolii-shell-iced`)がここだけを呼ぶ**
//! (2026-08-19 iced 再生機構移植レーン)。
//!
//! ## 二重実装ではない
//!
//! ここに置いた関数の中身は、egui 版 `show()` の再生ブロック(`Key::Space` /
//! `Key::L` の処理、`mod.rs` 内)と**同じ材料**である —
//! [`audio_seat::open_playback`] / [`audio_seat::playhead_moved`] /
//! [`super::wrap_playhead`] / [`super::advance_playhead`] を1つも書き直していない。
//! 分けたのは「呼ばれ方」だけ: egui は毎フレームの `show()` の中で直接この列を
//! 実行し、iced は `motolii_ui::blitz_shell::ShellGateway` の新しい口
//! (`toggle_playing` / `toggle_loop` / `tick_playback`)経由でここを呼ぶ
//! (`crates/motolii-ui/src/blitz_shell/intent.rs`)。egui 側の `show()` は
//! この file を呼ばない — 触っていない(柵: egui shell 本体は変更しない)。
//!
//! ## `TimelineEditor::view` には触らない
//!
//! egui 版は再生中「playhead が窓の中央に居続ける」よう `self.view` を毎フレーム
//! 動かす(`mod.rs` の該当コメント参照)。iced の Timeline view(zoom / pan)は
//! `motolii-shell-iced::timeline::TimelinePane` が別に持つ session 状態で、
//! `TimelineEditor::view` を一度も読まない(M-3 の分担)。ここで `self.view` を
//! 書いても iced 側には届かないので、書かない。
//!
//! ## intent にしていない
//!
//! [`TimelineEditor::toggle_playing`] / [`TimelineEditor::toggle_loop`] /
//! [`TimelineEditor::advance_playback`] はどれも `motolii_ui::blitz_shell::UiIntent`
//! に対応する変種を持たない。理由は `blitz_shell/intent.rs` モジュール doc の
//! 「ここに無いもの」に足した — 時間経過そのものは操作ではない(zoom/pan と同じ
//! scope)、start/stop の bool も「その後どれだけ実時間が経つか」が replay ごとに
//! 揺れる(壁時計 / audio device の実時間)ため、journal に載せると replay
//! oracle が exact に再現できない出力を「意図」として記録することになる。
//! 一方 loop 区間そのもの(`LoopRegion`)は元から egui 版でも
//! 「Project session の状態で、Document には入れない」と明記されている
//! (`mod.rs` の doc)ので、ON/OFF もその同じ scope に留めた。

use motolii_core::Fps;

use super::audio_seat::{self, AudioPlayback, WallClockReason};
use super::{advance_playhead, wrap_playhead, TimelineEditor, MAX_STEP};

impl TimelineEditor {
    /// `Space`: 再生 / 一時停止。掴んでいる最中は入り切りしない — ドラッグ中に
    /// 時間が流れると何が起きたか読めない(egui 版と同じ理由)。
    pub fn toggle_playing(&mut self) {
        if self.hold.is_some() {
            return;
        }
        let comp_seconds = self.document.composition.duration.as_seconds_f64() as f32;
        self.playing = !self.playing;
        if self.playing {
            // 終端で押したら頭から。止まったまま何も起きないのが一番困る
            if self.playhead >= comp_seconds - 1e-3 {
                self.playhead = 0.0;
            }
        } else if self.audio.is_some() {
            // pause は device をその場で手放す。egui は「次フレームの show()」が
            // 同じことをするが、iced は再生中だけ tick を購読するので次が来ない
            // ことがある — ここで明示する。
            self.audio = None;
        }
        self.status = if self.playing { "play" } else { "pause" }.to_owned();
    }

    /// `L`: loop の ON/OFF。帯は引いたまま、効きだけ切る(引き直さずに戻せる)。
    pub fn toggle_loop(&mut self) {
        self.loop_region.on = !self.loop_region.on;
        self.status = format!(
            "loop {} ({:.2}\u{2013}{:.2}s)",
            if self.loop_region.on { "on" } else { "off" },
            self.loop_region.start,
            self.loop_region.end
        );
    }

    // `is_playing()` は既に mod.rs に在る(2026-08-17裁定、Inspector が
    // 「編集は再生を止めない」の確認に使う読み)。ここでは増やさない
    // (`E0592 duplicate definitions` — 同じ名前の2つ目の `impl` メソッドは
    // Rust では単純に定義の重複になる。1つの意味に1つの名前)。

    /// loop が有効か(読み)。
    pub fn loop_on(&self) -> bool {
        self.loop_region.on
    }

    /// 再生を `dt` 秒ぶん進める。再生中でなければ何もしない(audio 座席が
    /// まだ残っていたら手放す — pause の取りこぼしに備えた保険で、通常は
    /// `toggle_playing` が既に手放している)。
    ///
    /// 実体は egui 版 `show()` の再生ブロックと同じ列: soundtrack があれば
    /// `audio_seat` を開いて `Transport`(`motolii-transport`)の聴感時刻に
    /// playhead を同期し、無ければ壁時計(`advance_playhead`)で進む。
    /// ループの折り返しも同じ `wrap_playhead` を通す。
    pub fn advance_playback(&mut self, dt: f32) {
        if !self.playing {
            if self.audio.is_some() {
                self.audio = None;
            }
            return;
        }
        let fps: Fps = self.document.composition.fps;
        let comp_seconds = self.document.composition.duration.as_seconds_f64() as f32;

        // 再生の頭で audio の座席を開く(soundtrack 無しなら壁時計のまま)。
        // 開けない時は黙らない — 理由を status に出して壁時計へ落ちる。
        if self.audio.is_none() {
            let playback = audio_seat::open_playback(
                &self.document,
                self.project_root.as_deref(),
                &mut self.pcm_caches,
                self.playhead,
                fps,
            );
            match &playback {
                AudioPlayback::Synced(_) => self.status = "play (soundtrack)".to_owned(),
                AudioPlayback::WallClock(WallClockReason::NoSoundtrack) => {}
                AudioPlayback::WallClock(WallClockReason::AudioUnavailable(reason)) => {
                    self.status = format!("play (no audio: {reason})");
                }
            }
            self.audio = Some(playback);
        }

        // 手で動かした playhead に音が付いてくる(scrub / locator でジャンプした
        // 印は「audio が最後に書いた値と違う」)。
        let moved = matches!(
            &self.audio,
            Some(AudioPlayback::Synced(seat))
                if audio_seat::playhead_moved(self.playhead, seat.last_synced())
        );
        if moved {
            self.reseek_audio(self.playhead, fps);
        }

        // audio の座席があるときは dt を使わない — clock の正本はデバイスへ
        // 供給済みのサンプル数。無ければ壁時計(MAX_STEP でまとめ足しを防ぐ)。
        let step = match self.audio.as_mut() {
            Some(AudioPlayback::Synced(seat)) => Some(seat.follow(comp_seconds)),
            _ => None,
        };
        let (at, keep) = match step {
            Some(Ok(step)) => step,
            Some(Err(error)) => {
                self.status = format!("play (no audio: {error})");
                self.audio = Some(AudioPlayback::WallClock(WallClockReason::AudioUnavailable(
                    error.to_string(),
                )));
                advance_playhead(self.playhead, dt.min(MAX_STEP), comp_seconds)
            }
            None => advance_playhead(self.playhead, dt.min(MAX_STEP), comp_seconds),
        };

        // 折り返したかどうかで終端判定が変わる(区間より後ろから始めたなら
        // 一度も折り返さない)。
        let wrapped = wrap_playhead(self.playhead, at, self.loop_region);
        let did_wrap = wrapped != at;
        self.playhead = wrapped;
        if did_wrap {
            self.reseek_audio(wrapped, fps);
        }
        if !keep && !did_wrap {
            self.playing = false;
            self.status = "end".to_owned();
            self.audio = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 掴んでいない・soundtrack の無い fixture では壁時計で進む。
    /// **「時計を進める」= `dt` を直接渡す**(実 sleep はしない。決定的)。
    ///
    /// 1呼びぶんは `MAX_STEP`(0.05s)でクランプされる(「窓が隠れていた分を
    /// まとめて進めない」egui 版と同じ仕様)。積み重ねれば大きく進む —
    /// 実際の再生は毎フレーム呼ばれるので、これが本物の使われ方である。
    #[test]
    fn playing_advances_the_playhead_by_dt_on_the_wall_clock() {
        let mut editor = TimelineEditor::with_fixture();
        assert!(
            editor.document().soundtrack.is_none(),
            "fixture は soundtrack を持たない前提(壁時計の経路を確かめたい)"
        );
        assert_eq!(editor.playhead_seconds(), 0.0);

        editor.toggle_playing();
        assert!(editor.is_playing());

        editor.advance_playback(MAX_STEP);
        assert!(
            (editor.playhead_seconds() - MAX_STEP).abs() < 1e-4,
            "1呼びぶん({MAX_STEP}s)進めたはずが {}",
            editor.playhead_seconds()
        );

        // 10呼びぶん(= 0.5s)積み重ねる。
        for _ in 0..9 {
            editor.advance_playback(MAX_STEP);
        }
        assert!(
            (editor.playhead_seconds() - 0.5).abs() < 1e-3,
            "10呼びぶん(0.5s)進めたはずが {}",
            editor.playhead_seconds()
        );
    }

    /// 再生していないあいだは `advance_playback` を呼んでも動かない
    /// (tick の subscription が外れていてもズレない、の保証)。
    #[test]
    fn advance_playback_is_a_no_op_while_paused() {
        let mut editor = TimelineEditor::with_fixture();
        editor.set_playhead_seconds(1.0);
        editor.advance_playback(1.0);
        assert_eq!(editor.playhead_seconds(), 1.0, "止まっているのに動いた");
    }

    /// `Space` の再toggleで pause。pause 中に `advance_playback` を呼んでも
    /// 進まない(= iced の tick 購読が外れる前提と一致する)。
    #[test]
    fn toggle_playing_flips_and_pause_freezes_the_playhead() {
        let mut editor = TimelineEditor::with_fixture();
        editor.toggle_playing();
        editor.advance_playback(MAX_STEP);
        assert!((editor.playhead_seconds() - MAX_STEP).abs() < 1e-4);

        editor.toggle_playing();
        assert!(!editor.is_playing());
        let at_pause = editor.playhead_seconds();
        editor.advance_playback(0.3);
        assert_eq!(editor.playhead_seconds(), at_pause, "pause 後も進んだ");
    }

    /// 終端で `Space` を押すと頭から再生し直す(止まったまま何も起きないのが
    /// 一番困る、egui 版と同じ)。
    #[test]
    fn toggling_play_at_the_end_restarts_from_zero() {
        let mut editor = TimelineEditor::with_fixture();
        let comp = editor.document().composition.duration.as_seconds_f64() as f32;
        editor.set_playhead_seconds(comp);
        editor.toggle_playing();
        assert_eq!(editor.playhead_seconds(), 0.0, "終端から頭へ戻っていない");
    }

    /// 終端に着いたら再生が止まる(壁時計の経路。巻き戻さない)。
    #[test]
    fn playback_stops_at_the_composition_end() {
        let mut editor = TimelineEditor::with_fixture();
        let comp = editor.document().composition.duration.as_seconds_f64() as f32;
        editor.set_playhead_seconds((comp - 0.02).max(0.0));
        editor.toggle_playing();
        for _ in 0..50 {
            editor.advance_playback(MAX_STEP);
            if !editor.is_playing() {
                break;
            }
        }
        assert!(!editor.is_playing(), "終端を過ぎても再生し続けている");
        assert!(
            (editor.playhead_seconds() - comp).abs() < 1e-3,
            "終端で止まっていない: {}",
            editor.playhead_seconds()
        );
    }

    /// `L` で loop の ON/OFF。掴んでいなければ毎回反転する。
    #[test]
    fn toggle_loop_flips_loop_on() {
        let mut editor = TimelineEditor::with_fixture();
        let before = editor.loop_on();
        editor.toggle_loop();
        assert_eq!(editor.loop_on(), !before);
        editor.toggle_loop();
        assert_eq!(editor.loop_on(), before);
    }

    /// loop が ON のあいだ、区間のお尻を越えると頭へ折り返す(composition の
    /// 終端には着かない)。
    #[test]
    fn playback_wraps_inside_an_active_loop_region_instead_of_ending() {
        let mut editor = TimelineEditor::with_fixture();
        // fixture の既定 loop 区間([2.0, 6.0), off)をそのまま使う。
        assert!(!editor.loop_on(), "fixture の既定は loop off の前提");
        editor.toggle_loop();
        assert!(editor.loop_on());

        editor.set_playhead_seconds(5.99);
        editor.toggle_playing();
        for _ in 0..20 {
            editor.advance_playback(0.05);
            assert!(
                editor.is_playing(),
                "loop 区間の中で再生が止まった: {}",
                editor.playhead_seconds()
            );
        }
        assert!(
            editor.playhead_seconds() < 6.0,
            "お尻を越えたのに折り返していない: {}",
            editor.playhead_seconds()
        );
    }
}
