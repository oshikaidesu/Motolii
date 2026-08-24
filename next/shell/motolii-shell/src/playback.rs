

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

use iced::Task;
use crate::Message;

impl Shell {
    /// 今の playhead を comp の fps で時刻へ写す。comp が無い/fps が壊れているなら
    /// `None`(M16: panic しない)。
    pub(crate) fn time_at_playhead(&self) -> Option<RationalTime> {
        let composition = self.doc.view().composition().ok().flatten()?;
        RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()
    }

    /// 今の Timeline の行。運転席が「層3枚の行が立つ」「選択が行と一致する」を
    /// 確かめる口(pane 自身が使う投影と同じ関数を呼ぶ)。
    pub fn timeline_rows(&self) -> Vec<timeline_pane::RowProjection> {
        timeline_pane::rows(&self.doc.view(), &self.session)
    }

    /// 今の property 行(キー行、第2波 T3)。選択 layer がキーを持つ property を
    /// 持たなければ空。運転席/`screenshot.rs` 器具が pane 自身と同じ投影を読む口
    /// (`timeline_rows` と同じ形)。
    pub fn timeline_property_rows(&self) -> Vec<timeline_pane::PropertyRowProjection> {
        let fps = self.composition().map(|c| c.fps);
        timeline_pane::property_rows(&self.doc.view(), &self.session, fps)
    }

    /// 今のマーカー一覧。**screenshot 器具**が Timeline のマーカー線を描くのに使う
    /// (`timeline_pane::TimelinePane::new` も同じ `markers()` 呼び出しをする)。
    pub fn markers(&self) -> Vec<motolii_store::Marker> {
        self.doc.view().markers().unwrap_or_default()
    }

    /// `TimelinePane` の組み立て。`view()` はこれを呼ぶだけ(第2波T5、正典
    /// §5.5「プレビューは毎フレーム」) — ドラッグ preview(`self.timeline` =
    /// `timeline_pane::PaneState`)を投影へ焼き込む経路を運転席が検査できる
    /// よう関数化した。**`TimelinePane::new` 自体のシグネチャ・既存呼び出し元は
    /// 汚さない** — `with_key_drag_active` と同じ「薄い builder を積み増す
    /// だけ」の形をもう2つ足しただけ。裁定160 切片7で `self.timeline_drag`/
    /// `timeline_key_drag` の2フィールド直読みから `self.timeline`(pane crate
    /// 所有の `PaneState`)経由の読み取り専用アクセサへ差し替えた
    /// (`clip_preview`/`key_preview`/`key_drag_active`、値は無改変)。
    pub fn build_timeline_pane(&self) -> timeline_pane::TimelinePane {
        let store = self.doc.view();
        // `ui_scale` 適用済み(`Shell::dims` — 適用点1箇所)。
        let dims = self.dims();
        let colors = self.tokens.colors;
        timeline_pane::TimelinePane::new(&store, &self.session, dims, colors, self.keyboard_modifiers)
            // 第2波T4: `timeline::key_rows` が継続イベント(move/release/右
            // クリック)を拾うかどうかの唯一の判断材料
            // (`TimelinePane::with_key_drag_active` の doc comment 参照)。
            .with_key_drag_active(self.timeline.key_drag_active())
            .with_clip_preview(self.timeline.clip_preview())
            .with_key_preview(self.timeline.key_preview())
            // B21+B18(第5波結線): 作業範囲/ループの状態は `PaneState` が持ち
            // (`work_area.rs` doc「型の置き場」)、絵と当たりへはこの読み口
            // 経由で毎フレーム運ぶ(`with_playing` と同じ薄い builder)。
            .with_work_area(self.timeline.work_area(), self.timeline.loop_enabled())
            // 第6波(rename 統合手順1): inline rename の下書きを rail の
            // `text_input` へ運ぶ(`rail.rs` の `pane.rename` 読み — 供給は
            // supervisor の仕事、`write.rs` 冒頭 doc 参照)。
            .with_rename(
                self.timeline
                    .rename_draft()
                    .map(|(layer, draft)| (layer, draft.to_owned())),
            )
            .with_marker_rename(
                self.timeline
                    .marker_rename_draft()
                    .map(|(index, draft)| (index, draft.to_owned())),
            )
            .with_frame_draft(self.timeline.frame_draft().map(str::to_owned))
            .with_graph_editor(
                self.timeline.graph_editor_open(),
                self.timeline.graph_editor_drafts(),
            )
            // 波形取得状態(TL7 統合手順3、S2 発注 #17「shell 側の呼び出し
            // 経路が無い」の穴埋め)。`self.timeline.waveforms()` を
            // `with_rename` と同じ「薄い builder で読み取り専用に運ぶだけ」
            // の形でそのまま渡す(実際の要求発火は `poll_waveform_fetches`)。
            .with_waveforms(self.timeline.waveforms().clone())
    }

    /// 音声 layer の波形取得を計画し、必要な分だけ非同期に発火する(TL7
    /// 統合手順1・5、S2 発注 #17「shell 側の呼び出し経路が無い」の穴埋め)。
    /// `Shell::update` の末尾から毎メッセージ後に呼ぶ(`refresh_frame` と
    /// 同じ「都度呼んでも安いので判断を持たせない」形 — `plan_waveforms`
    /// 自体が `Loading`/`Ready` を見て何もしない側へ落ちるので、音声 layer が
    /// 無い/既に取得済みの通常のフレームでは実質 no-op)。
    ///
    /// **画面幅は未知(EXACT TARGET 外)**: 実際の bar 幅は canvas 描画時の
    /// window 幅に依存する(`ruler.rs`/`canvas.rs` の `bounds.width`)が、
    /// `Shell` は window サイズを保持していない(`grep -n window_size` 0件、
    /// 実測)。ここでは固定の目安幅
    /// (`NOMINAL_WAVEFORM_WIDTH_PX`)を渡す — bucket 数が実窓とズレるのは
    /// 承知の上(発注書「波形は呼び出し経路の説明で足りる。描画の正しさは
    /// 窓が要るので【未確認】のまま残してよい」)。呼び出し経路(plan→
    /// Task::perform→WaveformFetched→Ready→canvas 描画)自体は実働する。
    pub(crate) fn poll_waveform_fetches(&mut self) -> Task<Message> {
        const NOMINAL_WAVEFORM_WIDTH_PX: f32 = 960.0;
        let store = self.doc.view();
        let rows = timeline_pane::audio_rows(&store);
        if rows.is_empty() {
            return Task::none();
        }
        let requests = self.timeline.plan_waveforms(&rows, |_layer| NOMINAL_WAVEFORM_WIDTH_PX);
        if requests.is_empty() {
            return Task::none();
        }
        Task::batch(requests.into_iter().map(|(layer, path, buckets)| {
            Task::perform(
                async move { motolii_media::waveform_peaks(path, buckets) },
                move |result| match result {
                    Ok(peaks) => Message::Timeline(timeline_pane::Message::WaveformFetched {
                        layer,
                        buckets,
                        peaks,
                    }),
                    Err(_) => Message::Timeline(timeline_pane::Message::WaveformFetchFailed {
                        layer,
                        buckets,
                    }),
                },
            )
        }))
    }

    /// 作業範囲の現在値(B18、第5波結線)。運転席(`tests/suite/`)が
    /// 「B/N・ループ帯ドラッグ → 範囲が立つ」「Esc → 復元」を検分する読み口
    /// (`timeline_rows`/`markers` と同じ「pane 自身が読むのと同じ状態」の形)。
    pub fn timeline_work_area(&self) -> Option<timeline_pane::WorkArea> {
        self.timeline.work_area()
    }

    /// ループ on/off の現在値(同上 — `advance_playback_tick` が読むのと同じ値)。
    pub fn timeline_loop_enabled(&self) -> bool {
        self.timeline.loop_enabled()
    }



    /// `Shell::update` から委譲される領域別 dispatch(2026-08-23 SP-1 レーン、
    /// `docs/reviews/2026-08-23-shell-split-plan.md` の続き)。**中身は無改変** —
    /// 元の巨大な `update()` match の腕をそのままここへ移しただけ(裁定どおり
    /// 移送と委譲だけ、バグ修正・整形は混ぜない)。渡された `message` がこの
    /// 領域の variant でなければ `Err(message)` で突き返す — `crate::dispatch_message`
    /// の chain-of-responsibility が次の領域dispatchへ渡す。**新しい Message 枝は
    /// ここへ腕を1本足すだけで済み、`lib.rs` は触らない**(MC-1 と同じ効能)。
    pub(crate) fn dispatch_playback(&mut self, message: Message) -> Result<Task<Message>, Message> {
        let mut task = Task::none();
        match message {
            Message::ScrubTo(frame) => self.scrub_to(frame),
            Message::Timeline(msg) => match msg {
                timeline_pane::Message::Select(layer) => self.click_select_layer(layer),
                timeline_pane::Message::ScrubTo(frame) => self.scrub_to(frame),
                timeline_pane::Message::ToggleMute(layer) => self.toggle_layer_hidden(layer),
                timeline_pane::Message::ToggleSolo(layer) => self.toggle_layer_solo(layer),
                timeline_pane::Message::ToggleLock(layer) => self.toggle_layer_lock(layer),
                // transport 帯(裁定180)— 意味は shell の既存腕そのもの(5例外と
                // 同じ先取りの型。pane 側 `PaneState::update` は no-op)。
                timeline_pane::Message::TogglePlayback => self.toggle_playback(),
                timeline_pane::Message::StepPlayhead(delta) => self.step_playhead(delta),
                timeline_pane::Message::JumpPlayheadToStart => self.session.playhead = 0,
                timeline_pane::Message::JumpPlayheadToEnd => {
                    let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
                    self.session.playhead = timeline::nav::comp_end_frame(duration);
                }
                // JKL シャトル(B21、第5波結線)— transport 4腕と同じ「shell
                // 先取りの例外」(`timeline_pane::Message::Shuttle` doc): 実時間
                // 再生の clock は shell(A2)が持つので、状態遷移と tick 駆動を
                // ここで畳む(`PaneState::update` では no-op)。
                timeline_pane::Message::Shuttle(command) => self.apply_shuttle(command),
                timeline_pane::Message::Marker(marker) => {
                    use timeline::markers::MarkerMessage;
                    match marker {
                        MarkerMessage::RenameBegin(index) => {
                            if let Some(name) = self.markers().get(index).map(|marker| marker.name.clone()) {
                                self.timeline.begin_marker_rename(index, name);
                            }
                        }
                        MarkerMessage::RenameEdited(draft) => {
                            self.timeline.edit_marker_rename(draft);
                        }
                        MarkerMessage::RenameCommit => {
                            if let Some((index, name)) = self.timeline.take_marker_rename() {
                                self.update_marker(MarkerMessage::Rename { index, name });
                            }
                        }
                        MarkerMessage::RenameCancel => {
                            self.timeline.cancel_marker_rename();
                        }
                        other => self.update_marker(other),
                    }
                }
                // ルーラ locator lane 右クリック(S2 発注 #22「マーカー追加
                // UI が無い」の穴埋め、2入口目)— キーボード M
                // (`Message::Marker(MarkerMessage::AddAtPlayhead)`)と同じ
                // `update_marker` 経路へ畳む(S6 併存、裁定195)。
                timeline_pane::Message::AddMarkerAt(frame) => {
                    self.update_marker(timeline::markers::MarkerMessage::AddAtFrame(frame))
                }
                other => {
                    if let Some(reason) =
                        self.timeline.update(other, &mut self.doc, &mut self.session, self.keyboard_modifiers)
                    {
                        self.status = Some(reason);
                    }
                }
            },
            Message::StepPlayhead(delta) => self.step_playhead(delta),
            Message::JumpPlayheadToStart => self.session.playhead = 0,
            Message::JumpPlayheadToEnd => {
                let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
                self.session.playhead = timeline::nav::comp_end_frame(duration);
            }
            Message::JumpMeaningPoint { direction, layer_only } => {
                self.jump_meaning_point(direction, layer_only);
            }
            Message::JumpClipEdge(edge) => self.jump_clip_edge(edge),
            Message::JumpToWorkAreaStart => {
                if let Some(area) = self.timeline.work_area() {
                    self.session.playhead = area.first_frame();
                }
            }
            Message::JumpToWorkAreaEnd => {
                if let Some(area) = self.timeline.work_area() {
                    self.session.playhead = area.last_frame();
                }
            }
            Message::TogglePlayback => self.toggle_playback(),
            Message::PlaybackTick => self.advance_playback_tick(),
            Message::DeleteSelectionRequested => {
                // Backspace/Delete の入力翻訳は `input.rs` が担当し、ここだけが
                // front の文脈(`selected_keys`)を読む。キーがあれば既存の
                // Timeline 動詞へ、無ければ既存の layer 削除 Message へ戻す。
                if self.session.selected_keys.is_empty() {
                    task = match self.dispatch_selection(Message::DeleteSelectedLayers) {
                        Ok(task) => task,
                        Err(_) => Task::none(),
                    };
                } else if let Some(reason) = self.timeline.update(
                    timeline_pane::Message::DeleteSelectedKeys,
                    &mut self.doc,
                    &mut self.session,
                    self.keyboard_modifiers,
                ) {
                    self.status = Some(reason);
                }
            }
            other => return Err(other),
        }
        Ok(task)
    }
}
