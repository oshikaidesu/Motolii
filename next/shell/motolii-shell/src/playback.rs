

use motolii_store::RationalTime;

use crate::transport::open_real_playback;
use crate::{
    timeline,
    timeline_pane, Shell,
};

impl Shell {
    /// Step Forward/Back(正典 §5・U2)。`delta` の符号・歩幅はキー解決側
    /// (`resolve_navigation_key`)が既に決めている — ここは
    /// `timeline::nav::step_playhead` の clamp をそのまま適用するだけ。
    pub(crate) fn step_playhead(&mut self, delta: i64) {
        let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
        self.session.playhead = timeline::nav::step_playhead(self.session.playhead, delta, duration);
    }

    /// JumpPrev/NextMeaningPoint(正典 §8.1・U2)。**見えている意味点**を集めて
    /// `timeline::nav::nearest_meaning_point` へ渡すだけ:
    /// - 常に: 選択 layer の表示中 property 行のキー菱形時刻(`timeline_property_rows`
    ///   — 選択 layer 1本ぶんしか描かれない、`projection::property_rows` の
    ///   EXACT TARGET 1 どおり)
    /// - `layer_only` が false の時だけ追加: comp locator(`markers()`)。
    ///   locator は layer に紐付かない(comp 単位)ので「選択レイヤー限定」
    ///   (Shift 付き)では対象から外れる — これが `layer_only` の意味そのもの
    ///
    /// 渡る先が無ければ何もしない(`nearest_meaning_point` が `None` を返す =
    /// no-op、既存の「拒否理由の無い no-op」と同じ形 — 意味点が無いのは
    /// エラーではない)。
    pub(crate) fn jump_meaning_point(&mut self, direction: timeline::nav::JumpDirection, layer_only: bool) {
        let mut points: Vec<i64> = self
            .timeline_property_rows()
            .iter()
            .flat_map(|row| row.keys.iter().map(|key| key.frame))
            .collect();
        if !layer_only {
            if let Some(fps) = self.composition().map(|c| c.fps) {
                points.extend(
                    self.markers()
                        .iter()
                        .filter_map(|marker| marker.time.try_to_frame_floor(fps).ok()),
                );
            }
        }
        if let Some(frame) = timeline::nav::nearest_meaning_point(&points, self.session.playhead, direction) {
            self.session.playhead = frame;
        }
    }

    /// JumpToClipIn/Out(正典 §8.1・U2)。対象は `Session::selection`(単一
    /// focus)の clip — 選択が無ければ何もしない(`nudge_keyframe` と同じ
    /// 「選択が無ければ no-op」の形。跳ぶ先を持たない操作を理由つき拒否に
    /// するほどの重さではない)。
    pub(crate) fn jump_clip_edge(&mut self, edge: timeline::nav::ClipEdge) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(row) = self.timeline_rows().into_iter().find(|row| row.id == layer) else {
            return;
        };
        self.session.playhead = timeline::nav::clip_edge_frame(row.start, row.duration, edge);
    }

    // ---- 実時間再生(A2、正典 §2 拘束5) ----

    /// `Message::ScrubTo`/`timeline_pane::Message::ScrubTo` の唯一の書き口。
    /// **再生中の scrub = seek**(発注書 ORACLE (c))— `Transport::seek` は
    /// `PlaybackClock::seek`(純粋、counters非依存)へ委譲するので実デバイス
    /// 無しで検証できる(`transport.rs` doc参照)。
    pub(crate) fn scrub_to(&mut self, frame: i64) {
        let frame = frame.max(0);
        self.session.playhead = frame;
        if self.transport.is_running() {
            if let Some(fps) = self.composition().map(|c| c.fps) {
                if let Ok(at) = RationalTime::try_from_frame(frame, fps) {
                    self.transport.seek(at);
                }
            }
        }
    }

    /// Space(発注書 ORACLE (d))。**ドラッグ中は無効**(正典 §2 拘束5「再生と
    /// 掴みは相互排他」)— Timeline の clip/key ドラッグと Inspector の
    /// 値セルドラッグのどちらでも封じる(掴み全般が対象、Timeline に限らない)。
    pub(crate) fn toggle_playback(&mut self) {
        if self.is_dragging() {
            return;
        }
        // シャトル走行中の Space = 停止(B21 結線)。シャトルも「再生中」の
        // 一種なので、Play‖Pause の「再生中→停止」の読みをそのまま延長する —
        // 停止せず実時間 transport を重ねて起動しない(2つの clock を併走
        // させない、`apply_shuttle` と対称の排他)。
        if !self.shuttle.is_stopped() {
            self.shuttle = timeline_pane::ShuttleState::stopped();
            return;
        }
        if self.transport.is_running() {
            self.freeze_playhead_from_transport();
            self.transport.stop();
        } else if let Err(error) = self.transport.start(open_real_playback, &self.doc, &self.session) {
            self.status = Some(error);
        }
    }

    /// JKL シャトル(B21、第5波結線)。意味(1→2→4→8 の倍率状態機械)は
    /// [`timeline_pane::ShuttleState::apply`] が正本 — ここが持つ判断は2つだけ:
    /// 拘束5(再生と掴みは相互排他 — `toggle_playback` と同じ柵)と、実時間
    /// transport との排他(シャトルへ乗る時は clock 側を位置 freeze してから
    /// 畳む — 2つの再生源を併走させない)。
    pub(crate) fn apply_shuttle(&mut self, command: timeline_pane::ShuttleCommand) {
        if self.is_dragging() {
            return;
        }
        if self.transport.is_running() {
            self.freeze_playhead_from_transport();
            self.transport.stop();
        }
        self.shuttle = self.shuttle.apply(command);
    }

    /// 進行中の掴みがあるか(Timeline clip/key/ループ帯ドラッグ + Inspector
    /// 値セルドラッグ + Stage ギズモドラッグ)。`toggle_playback`(拘束5)専用の
    /// 判定 — 個々の drag 状態はそれぞれの pane/フィールドの持ち物のまま
    /// (このメソッドは束ねて読むだけ)。
    pub(crate) fn is_dragging(&self) -> bool {
        self.timeline.is_dragging() || self.inspector_drag.is_some() || self.gizmo_drag.is_some()
    }

    /// Pause の直前に呼ぶ: 今の再生位置を`Session::playhead`へ確定させる
    /// (`transport.stop()`は位置を保存しないので、呼ぶ前にこれが要る —
    /// `transport.rs::Transport::stop`のdoc参照)。
    pub(crate) fn freeze_playhead_from_transport(&mut self) {
        let Some(fps) = self.composition().map(|c| c.fps) else {
            return;
        };
        if let Some(frame) = self.transport.position_frame(fps) {
            self.session.playhead = frame.max(0);
        }
    }

    /// 再生中tick(発注書 ORACLE (a)/(e))。`PlaybackClock::position()` を
    /// `Session::playhead` へ写す。comp 終端に達したら位置を終端へ揃えて
    /// 自動 Pause する(`JumpPlayheadToEnd`と同じ`comp_end_frame`を使うので
    /// 「終端」の定義が二重にならない)。
    ///
    /// **第5波結線(B21+B18)**: playhead の1歩は
    /// [`timeline_pane::work_area::advanced_playhead`] を通る — ループ on・
    /// 作業範囲の中を再生している時だけ範囲内で折り返す(範囲外・ループ off は
    /// 従来どおり clamp/自動 Pause)。JKL シャトル走行中は実時間 clock ではなく
    /// tick 駆動(1 tick = `rate` フレーム — `shuttle.rs` doc「1 tick に進む
    /// フレーム数 = rate」)で同じ関数を通す。
    pub(crate) fn advance_playback_tick(&mut self) {
        let duration = self.comp_duration();
        let area = self.timeline.work_area();
        let loop_enabled = self.timeline.loop_enabled();

        // ---- JKL シャトル(B21) — 実時間 clock を持たない tick 駆動 ----
        if !self.shuttle.is_stopped() {
            let current = self.session.playhead;
            let next = timeline_pane::work_area::advanced_playhead(
                current,
                i64::from(self.shuttle.rate),
                area,
                loop_enabled,
                duration,
            );
            if next == current {
                // 端で clamp されて進めない(ループ捕捉外)— transport の
                // 「終端で自動 Pause」と同型の自動停止。
                self.shuttle = timeline_pane::ShuttleState::stopped();
            } else {
                self.session.playhead = next;
            }
            return;
        }

        // ---- 実時間 transport(A2) ----
        let Some(fps) = self.composition().map(|c| c.fps) else {
            self.transport.stop();
            return;
        };
        let Some(frame) = self.transport.position_frame(fps) else {
            return;
        };
        let current = self.session.playhead;
        // ループ捕捉(B18): 範囲の中を再生している時だけ折り返す(外は普通に
        // 通過 — `advanced_playhead` doc「罠にしない」)。clock は線形に進み
        // 続けるので、「clock と playhead の差」を1歩として渡す — 折り返し後も
        // `start + (clock - start) % len` に畳まれ、tick ごとに安定する
        // (rem_euclid が範囲長で畳むため前回の折り返し位置に依存しない)。
        if loop_enabled && area.is_some_and(|a| a.contains(current)) {
            self.session.playhead =
                timeline_pane::work_area::advanced_playhead(current, frame - current, area, true, duration);
            return;
        }
        let end = timeline::nav::comp_end_frame(duration);
        if frame >= end {
            self.session.playhead = end;
            self.transport.stop();
        } else {
            self.session.playhead = frame.max(0);
        }
    }

    /// **ORACLE の試験専用の縫い目**(「デバイス抽象はフェイクで — A1と同じ手」)。
    /// `motolii_audio::PlaybackSession::for_simulation` で組んだフェイク
    /// セッション(実cpal無し、`PlaybackCounters`を`advance_supplied_for_
    /// simulation`で手動で進める)を、実デバイスを一切開かずに再生中状態へ
    /// 直接採用する。本番経路(`toggle_playback`)はこれを経由しない
    /// (`open_real_playback`を直接呼ぶ)。
    pub fn debug_start_playback_with_session(&mut self, session: motolii_audio::PlaybackSession) {
        self.transport.start_with_session(session);
    }

    /// 運転席が見るための口(`can_undo`/`can_redo`と同じ形)。
    pub fn is_playing(&self) -> bool {
        self.transport.is_running()
    }

}

