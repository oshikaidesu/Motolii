use crate::rerun_stage::StageGizmoAction;
use crate::timeline_skia::TimelinePointerPhase;

use super::RendererCore;
use super::scrub::{ScrubPointerPhase, ScrubTimePump};
use super::types::{PointerPhase, StagePointerButton};

impl RendererCore {
    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        let Some(session) = &self.timeline_session else {
            return None;
        };
        crate::timeline_skia::hit_test(&session.scene, self.config.width, self.config.height, x, y)
    }

    /// hover位置のhit種→cursor。clip drag中はclosedHand。
    pub(crate) fn timeline_hover_cursor(&self, x: f64, y: f64) -> i32 {
        let Some(session) = &self.timeline_session else {
            return crate::timeline_skia::CursorKind::Arrow.as_i32();
        };
        let hit = crate::timeline_skia::timeline_hover_hit(
            &session.scene,
            self.playhead,
            self.config.width,
            self.config.height,
            x,
            y,
        );
        let clip_dragging = matches!(
            session.gesture_kind_for_cursor(),
            Some(crate::timeline_skia::CursorDragKind::Clip)
        );
        crate::timeline_skia::cursor_for_timeline_hover(hit, clip_dragging).as_i32()
    }

    /// Stage上のlayer hover → cursor。x/yはtimeline同様の物理座標。
    pub(crate) fn stage_hover_cursor(&self, x: f64, y: f64) -> i32 {
        let dragging = self.stage_gizmo_pointer_active;
        let over = self
            .stage
            .as_ref()
            .is_some_and(|stage| stage.rerun.gizmo_wants_pointer(x, y));
        crate::timeline_skia::cursor_for_stage_hover(over, dragging).as_i32()
    }

    /// Timeline pointer。戻り値trueはselection/playhead変化(feedback対象)。
    pub(crate) fn timeline_pointer(
        &mut self,
        phase: PointerPhase,
        x: f64,
        y: f64,
        modifiers: u32,
    ) -> Option<(i32, f64)> {
        // 並行gesture防御: Downで進行中scrub pumpを先に解除してから新gestureへ。
        if matches!(phase, PointerPhase::Down) {
            self.scrub_time_pump = ScrubTimePump::new();
            self.scrubbing = false;
        }
        let tl_phase = match phase {
            PointerPhase::Down => {
                self.stats.pointer_downs += 1;
                TimelinePointerPhase::Down
            }
            PointerPhase::Move => {
                self.stats.pointer_moves += 1;
                TimelinePointerPhase::Move
            }
            PointerPhase::Up => {
                self.stats.pointer_ups += 1;
                TimelinePointerPhase::Up
            }
            PointerPhase::Cancel => TimelinePointerPhase::Cancel,
        };
        let width = self.config.width;
        let height = self.config.height;
        let (is_real, outcome) = {
            let Some(session) = &mut self.timeline_session else {
                return None;
            };
            let is_real = session.scene.real;
            let outcome = session.pointer(
                &mut self.selected_object_index,
                &mut self.playhead,
                width,
                height,
                tl_phase,
                x,
                y,
                modifiers,
            );
            (is_real, outcome)
        };
        if outcome.dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        if is_real {
            let maybe_scrub_playhead =
                outcome
                    .scrub_playhead
                    .or(if matches!(phase, PointerPhase::Cancel) {
                        if self.scrub_time_pump.is_active() {
                            Some(self.playhead)
                        } else {
                            None
                        }
                    } else {
                        None
                    });
            if let Some(scrub_playhead) = maybe_scrub_playhead {
                self.dispatch_set_time_for_scrub(
                    match phase {
                        PointerPhase::Down => ScrubPointerPhase::Down,
                        PointerPhase::Move => ScrubPointerPhase::Move,
                        PointerPhase::Up => ScrubPointerPhase::Up,
                        PointerPhase::Cancel => ScrubPointerPhase::Cancel,
                    },
                    scrub_playhead,
                );
            }
            if matches!(phase, PointerPhase::Up) && outcome.edit_commit.is_some() {
                self.force_next_host_snapshot = true;
            }
            if let Some(commit) = outcome.edit_commit {
                if let Some(result) = crate::host_bridge::try_dispatch_timeline_edit(&commit) {
                    let has_projection = result.projection.is_some();
                    self.apply_terminal_timeline_result(result);
                    if !has_projection {
                        // snapshot欠落時だけ次frameのfull読みに委ねる。
                        self.force_next_host_snapshot = true;
                    }
                } else {
                    // 拒否時は次snapshotで幾何を戻す。dirty再描画だけ先に立てる。
                    if let Some(timeline) = &mut self.timeline {
                        timeline.dirty = true;
                    }
                    self.force_next_host_snapshot = true;
                }
            }
            if let Some(commit) = outcome.selection_commit {
                let _ = self.dispatch_timeline_selection(&commit);
            }
            self.scrubbing = self.scrub_time_pump.is_active();
        }
        if matches!(phase, PointerPhase::Down) {
            crate::host_bridge::set_timeline_interacting(true);
        } else if matches!(phase, PointerPhase::Up | PointerPhase::Cancel) {
            crate::host_bridge::set_timeline_interacting(false);
            self.scrubbing = false;
            self.scrub_time_pump = ScrubTimePump::new();
        }
        outcome
            .feedback
            .then_some((self.selected_object_index, self.playhead))
    }

    fn now_ms(&self) -> u64 {
        self.scrub_clock_start.elapsed().as_millis() as u64
    }

    fn dispatch_set_time_for_scrub(&mut self, phase: ScrubPointerPhase, playhead: f64) {
        let Some((fps_num, fps_den)) = self.host_fps else {
            return;
        };
        if fps_num <= 0 || fps_den <= 0 {
            return;
        }
        let song_bars = self
            .timeline_session
            .as_ref()
            .map(|session| session.scene.song_bars)
            .unwrap_or(crate::timeline_skia::SONG_BARS);
        let bar = playhead.clamp(0.0, 1.0) * f64::from(song_bars);
        let Some(frame) =
            self.scrub_time_pump
                .next_frame(phase, bar, self.now_ms(), fps_num, fps_den)
        else {
            return;
        };
        if let Some(result) = crate::host_bridge::try_dispatch_set_time(frame) {
            let has_projection = result.projection.is_some();
            self.apply_terminal_timeline_result(result);
            if !has_projection {
                self.force_next_host_snapshot = true;
            }
        }
    }

    /// Timeline scroll/pinch。戻り値trueは視覚変化(dirty)。feedbackなし。
    pub(crate) fn timeline_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        let Some(session) = &mut self.timeline_session else {
            return false;
        };
        let dirty = session.scroll(
            self.config.width,
            self.config.height,
            delta_x,
            delta_y,
            magnification,
            modifiers,
            x,
            y,
        );
        if dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        dirty
    }

    /// Timeline Delete/Backspace: 選択keyがあれば remove、なければ layer削除。
    pub(crate) fn timeline_keymap_delete(&self) -> bool {
        let Some(session) = &self.timeline_session else {
            return crate::host_bridge::try_dispatch_keymap("delete_layer")
                .is_some_and(|result| result.accepted);
        };
        crate::host_bridge::try_timeline_keymap_delete(&session.scene)
            .is_some_and(|result| result.accepted)
    }

    pub(crate) fn stats(&self) -> RenderStats {
        self.stats
    }

    pub(crate) fn stage_pointer(
        &mut self,
        phase: PointerPhase,
        button: StagePointerButton,
        modifiers: u32,
        x: f64,
        y: f64,
    ) {
        let gizmo_hit = button == StagePointerButton::Primary
            && matches!(phase, PointerPhase::Down)
            && self
                .stage
                .as_ref()
                .is_some_and(|stage| stage.rerun.gizmo_wants_pointer(x, y));
        if (self.stage_gizmo_pointer_active && button == StagePointerButton::Primary) || gizmo_hit {
            self.stage_gizmo_pointer_active =
                !matches!(phase, PointerPhase::Up | PointerPhase::Cancel);
            if let Some(stage) = &mut self.stage {
                // gizmo capture中のprimaryはRerun camera/pickingへ二重配送しない。
                stage.rerun.gizmo_pointer(phase, x, y);
            }
        } else if let Some(stage) = &mut self.stage {
            stage.rerun.pointer(phase, button, modifiers, x, y);
        } else {
            return;
        }
        match phase {
            PointerPhase::Down => self.stats.pointer_downs += 1,
            PointerPhase::Move => self.stats.pointer_moves += 1,
            PointerPhase::Up => self.stats.pointer_ups += 1,
            PointerPhase::Cancel => {}
        }
    }

    pub(crate) fn stage_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        self.stage.as_mut().is_some_and(|stage| {
            stage
                .rerun
                .scroll(delta_x, delta_y, magnification, modifiers, x, y)
        })
    }

    fn process_stage_gizmo_action(&mut self) {
        let action = self
            .stage
            .as_mut()
            .and_then(|stage| stage.rerun.take_gizmo_action());
        let Some(action) = action else {
            return;
        };
        let Some(expected_revision) = self
            .host_revision
            .as_deref()
            .and_then(|revision| revision.parse::<u64>().ok())
        else {
            self.restore_stage_preview("The live Document revision is unavailable");
            return;
        };

        match action {
            StageGizmoAction::Preview { layer_id, edit } => {
                let _ = self.preview_stage_transform_from_app(expected_revision, &layer_id, edit);
            }
            StageGizmoAction::Commit { layer_id, edit } => {
                let _ = self.commit_stage_transform_from_app(expected_revision, &layer_id, edit);
            }
            StageGizmoAction::Cancel => {
                let _ = self.cancel_stage_transform_from_app();
            }
        }
    }

    fn restore_stage_preview(&mut self, message: &str) {
        let Some(stage) = self.stage.as_mut() else {
            return;
        };
        stage.preview_active = false;
        if let Some(geometry) = self.host_stage_geometry.as_ref() {
            let _ = stage.rerun.apply_host_stage_geometry(
                geometry,
                self.config.width,
                self.config.height,
            );
        }
        stage.rerun.set_feedback(message, true);
    }
}
