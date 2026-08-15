use std::time::Instant;

use crate::timeline_skia::TimelineSession;

use super::RendererCore;
use super::host::{
    HostStageGeometryCommand, host_stage_geometry_command, timeline_projection_selected_flat,
    timeline_scene_from_projection,
};
use super::scrub::ScrubTimePump;
use super::types::{NativeHostTerminalEvent, SceneKind};

impl RendererCore {
    fn dispatch_timeline_selection(
        &mut self,
        commit: &crate::timeline_skia::TimelineSelectionCommit,
    ) -> bool {
        let Some(result) = crate::host_bridge::try_dispatch_timeline_selection(commit) else {
            return false;
        };
        let accepted = result.accepted;
        let has_projection = result.projection.is_some();
        match self.scene {
            SceneKind::Stage => self.apply_terminal_stage_result(result),
            SceneKind::Timeline => self.apply_terminal_timeline_result(result),
        }
        if !has_projection {
            self.force_next_host_snapshot = true;
        }
        accepted
    }

    fn apply_terminal_timeline_result(&mut self, result: crate::host_bridge::HostTerminalResult) {
        if !self.host_terminal_latch.record_if_current(
            self.host_handle.as_deref(),
            self.host_projection_generation.as_deref(),
            &result,
        ) {
            return;
        }
        let stamp = result.stamp();
        let Some(projection) = result.projection else {
            if !result.accepted {
                self.force_next_host_snapshot = true;
            }
            return;
        };
        self.host_projection_stamp = stamp;
        self.host_fps = projection.fps;
        let revision_changed = self.host_revision.as_deref() != Some(projection.revision.as_str());
        let generation_changed = self.host_projection_generation.as_deref()
            != Some(projection.projection_generation.as_str());
        let primary_changed =
            self.selected_object_index != timeline_projection_selected_flat(&projection);
        let should_reproject = revision_changed || primary_changed || !result.accepted;
        if should_reproject {
            let Some(session) = &mut self.timeline_session else {
                return;
            };
            let gesture_dirty = session.discard_active_gesture();
            if gesture_dirty {
                self.scrub_time_pump = ScrubTimePump::new();
                self.scrubbing = false;
                crate::host_bridge::set_timeline_interacting(false);
            }
            let scene = timeline_scene_from_projection(&session.scene, &projection);
            self.selected_object_index = scene.selected_flat;
            session.scene = scene;
        }
        self.host_revision = Some(projection.revision.clone());
        if let Some(host_handle) = projection.host_handle.clone() {
            self.host_handle = Some(host_handle);
        }
        self.host_projection_generation = Some(projection.projection_generation.clone());
        if !self.scrubbing && (generation_changed || revision_changed || !result.accepted) {
            let song_bars = self
                .timeline_session
                .as_ref()
                .map(|session| session.scene.song_bars)
                .unwrap_or(crate::timeline_skia::SONG_BARS);
            self.playhead = crate::host_bridge::playhead_from_current_time(
                projection.current_time.0,
                projection.current_time.1,
                song_bars,
            );
        }
        if (should_reproject || generation_changed)
            && let Some(timeline) = &mut self.timeline
        {
            timeline.dirty = true;
        }
        self.force_next_host_snapshot = false;
    }

    fn apply_terminal_stage_result(&mut self, result: crate::host_bridge::HostTerminalResult) {
        if !self.host_terminal_latch.record_if_current(
            self.host_handle.as_deref(),
            self.host_projection_generation.as_deref(),
            &result,
        ) {
            return;
        }
        let stamp = result.stamp();
        let feedback = result.feedback().map(str::to_owned);
        let accepted = result.accepted;
        let Some(projection) = result.projection else {
            if !accepted {
                self.restore_stage_preview(feedback.as_deref().unwrap_or("Host rejected the edit"));
                self.force_next_host_snapshot = true;
            }
            return;
        };
        self.host_projection_stamp = stamp;
        self.host_revision = Some(projection.revision.clone());
        if let Some(host_handle) = projection.host_handle.clone() {
            self.host_handle = Some(host_handle);
        }
        self.host_projection_generation = Some(projection.projection_generation.clone());
        let Some(stage) = self.stage.as_mut() else {
            return;
        };
        stage.preview_active = false;
        stage
            .rerun
            .set_host_primary_layer_id(projection.primary_layer_id.clone());
        match host_stage_geometry_command(self.host_stage_geometry.as_ref(), Some(&projection)) {
            HostStageGeometryCommand::Apply(geometry) => {
                if stage.rerun.apply_host_stage_geometry(
                    &geometry,
                    self.config.width,
                    self.config.height,
                ) {
                    self.host_stage_geometry = Some(geometry);
                    self.host_stage_viewport = Some((self.config.width, self.config.height));
                }
            }
            HostStageGeometryCommand::Clear => {
                if stage.rerun.clear_host_projection() {
                    self.host_stage_geometry = None;
                    self.host_stage_viewport = None;
                }
            }
            HostStageGeometryCommand::Noop => {}
        }
        if !accepted {
            stage.rerun.set_feedback(
                feedback.as_deref().unwrap_or("Host rejected the edit"),
                true,
            );
        }
        self.force_next_host_snapshot = false;
    }

    pub(crate) fn take_host_terminal_event(&mut self) -> Option<NativeHostTerminalEvent> {
        self.host_terminal_latch.take()
    }

    /// F9: stamp不変かつforceでなく初回でもないtickはfull JSON読みを飛ばす。
    /// 引数 `current` はtick冒頭で1回だけ採るstamp。gateと保存で同じ値を使う。
    pub(crate) fn host_snapshot_read_needed(
        last: Option<(u64, u64)>,
        current: Option<(u64, u64)>,
        force: bool,
    ) -> bool {
        if force || last.is_none() {
            return true;
        }
        match current {
            Some(stamp) => last != Some(stamp),
            // stamp取得失敗時は従来どおりfull読みへ落とす(挙動維持)。
            None => true,
        }
    }

    /// mount時1回だけ。resize/scene切替では呼ばない。
    fn run_mount_warmup(&mut self) {
        if self.mount_warmup_done {
            return;
        }
        self.mount_warmup_done = true;
        let started = Instant::now();
        let ok = match self.scene {
            SceneKind::Stage => self.warmup_stage_offscreen(),
            SceneKind::Timeline => self.warmup_timeline_skia(),
        };
        let warmup_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        // telemetry 1行。ObjC ABIは触らない。
        eprintln!(
            "[MotoliiRenderProbe] warmup_us={warmup_us} scene={:?} ok={}",
            self.scene,
            u8::from(ok),
        );
    }

    fn warmup_stage_offscreen(&mut self) -> bool {
        let width = self.config.width.max(1);
        let height = self.config.height.max(1);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Motolii stage mount warmup"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // 既存Stage描画経路をpresent無しで1回(submit+poll)。
        let ok = self.present_stage(&view).is_ok();
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        ok
    }

    fn warmup_timeline_skia(&mut self) -> bool {
        // 初回Skia画素をhost snapshotにする。未syncだとempty/fixtureが製品mountに残る。
        self.sync_host_timeline_projection();
        let width = self.config.width.max(1);
        let height = self.config.height.max(1);
        let scene = self
            .timeline_session
            .as_ref()
            .map(|session| session.scene.clone())
            .unwrap_or_else(TimelineScene::empty_host);
        let playhead = self.playhead;
        let selected = self.selected_object_index;
        let Some(timeline) = self.timeline.as_mut() else {
            return true;
        };
        // font/typeface/surface初期化を実サイズbufferへ先払い。
        crate::timeline_skia::draw_timeline(
            &scene,
            &mut timeline.pixels,
            width,
            height,
            playhead,
            selected,
        );
        // 次の可視frameでも再rasterしてよい(dirty維持)。warm-upは初期化コスト払いが目的。
        timeline.dirty = true;
        true
    }

    fn sync_host_timeline_projection(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let read_stamp = crate::host_bridge::try_read_projection_stamp();
            let force_scene = self.force_next_host_snapshot;
            if !Self::host_snapshot_read_needed(self.host_projection_stamp, read_stamp, force_scene)
            {
                return;
            }
            let Some(projection) = crate::host_bridge::try_read_timeline_projection() else {
                self.host_projection_stamp = None;
                return;
            };
            if let Some(host_handle) = projection.host_handle.clone() {
                self.host_handle = Some(host_handle);
            }
            self.host_projection_stamp = read_stamp;
            self.host_fps = projection.fps;
            let revision_changed =
                self.host_revision.as_deref() != Some(projection.revision.as_str());
            let generation_changed = self.host_projection_generation.as_deref()
                != Some(projection.projection_generation.as_str());
            let primary_changed =
                self.selected_object_index != timeline_projection_selected_flat(&projection);
            let has_active_gesture = self
                .timeline_session
                .as_ref()
                .is_some_and(TimelineSession::has_active_gesture);
            let should_reproject =
                revision_changed || primary_changed || (force_scene && !has_active_gesture);
            if should_reproject {
                let Some(session) = &mut self.timeline_session else {
                    return;
                };
                // 進行中gestureは復元せず破棄。古いband indexでのpanicを防ぐ。
                let gesture_dirty = session.discard_active_gesture();
                if gesture_dirty {
                    self.scrub_time_pump = ScrubTimePump::new();
                    self.scrubbing = false;
                    crate::host_bridge::set_timeline_interacting(false);
                }
                let scene = timeline_scene_from_projection(&session.scene, &projection);
                self.selected_object_index = scene.selected_flat;
                session.scene = scene;
                if revision_changed {
                    self.host_revision = Some(projection.revision);
                }
                if let Some(timeline) = &mut self.timeline {
                    timeline.dirty = true;
                }
                if force_scene {
                    self.force_next_host_snapshot = false;
                }
            }
            if !self.scrubbing && (generation_changed || revision_changed || force_scene) {
                let song_bars = self
                    .timeline_session
                    .as_ref()
                    .map(|session| session.scene.song_bars)
                    .unwrap_or(crate::timeline_skia::SONG_BARS);
                let next = crate::host_bridge::playhead_from_current_time(
                    projection.current_time.0,
                    projection.current_time.1,
                    song_bars,
                );
                if (self.playhead - next).abs() > f64::EPSILON {
                    self.playhead = next;
                    if let Some(timeline) = &mut self.timeline {
                        timeline.dirty = true;
                    }
                }
            }
            self.host_projection_generation = Some(projection.projection_generation);
        }
    }
}
