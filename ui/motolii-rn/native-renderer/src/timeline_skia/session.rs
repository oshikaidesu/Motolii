//! pointer session と gesture identity。indexを描画用にその場解決する。

use super::geometry::{
    band_index_at_ly, bar_at_lx, center_view_on, clamp_ordered, clamp_ordered_f32,
    clamp_view_translate, min_clip_units, neighbors, overview_bar_at_lx, snap_bar_with_guide,
    surface_width, time_at_lx, zoom_at,
};
use super::hit::{hit_gesture, HitKind};
use super::layout::{scale_for, MOVE_ARM_PX};
use super::scene::{clear_all_key_selection, selected_real_key, Clip, TimelineScene};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TimelinePointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Debug)]
pub(super) struct GestureSnapshot {
    scene: TimelineScene,
    selected: i32,
    playhead: f64,
}

#[derive(Clone, Debug)]
pub(super) enum ActiveGesture {
    Scrub {
        snapshot: GestureSnapshot,
    },
    Overview {
        snapshot: GestureSnapshot,
    },
    SelectOrMove {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        layer_id: String,
        press_lx: f64,
        press_ly: f64,
        origin_a: f32,
        origin_b: f32,
        origin_keys: Vec<f32>,
        moving: bool,
    },
    TrimStart {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        layer_id: String,
    },
    TrimEnd {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        layer_id: String,
    },
    KeyDrag {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        key_idx: usize,
        key_id: u64,
        layer_id: String,
    },
    Deselect {
        snapshot: GestureSnapshot,
    },
    Mute {
        snapshot: GestureSnapshot,
        layer_id: String,
    },
    Solo {
        snapshot: GestureSnapshot,
        layer_id: String,
    },
}

/// cursor写像で必要なgesture粗分類。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CursorDragKind {
    Clip,
}

/// scene + 進行中gesture。selection/playheadはcallerが所有する。
#[derive(Clone, Debug, Default)]
pub(super) struct TimelineSession {
    pub scene: TimelineScene,
    gesture: Option<ActiveGesture>,
}

/// pointer処理の結果。`feedback`はselection/playhead変化時のみtrue。
#[derive(Clone, Debug, Default)]
pub(super) struct TimelinePointerOutcome {
    pub feedback: bool,
    pub dirty: bool,
    /// scrub中のplayhead(0..1)。Cancelでは出さない。
    pub scrub_playhead: Option<f64>,
    /// scrubのUp確定。set_timeはthrottle無視で必発。
    pub scrub_release: bool,
    /// real行の編集gestureをUpで確定した時の1回dispatch。
    pub edit_commit: Option<TimelineEditCommit>,
    /// real行のDown選択をhostへ1回dispatchする。
    pub selection_commit: Option<TimelineSelectionCommit>,
}

/// real TimelineのDown時にhostへ送る選択intent。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TimelineSelectionCommit {
    SelectLayer { layer_id: String },
    ClearSelection,
}

/// real Timelineのrelease時にhostへ送る編集intent。
#[derive(Clone, Debug, PartialEq)]
pub(super) enum TimelineEditCommit {
    SetClipStart {
        layer_id: String,
        bar: f32,
    },
    TrimClipIn {
        layer_id: String,
        bar: f32,
    },
    TrimClipOut {
        layer_id: String,
        bar: f32,
    },
    SetPositionKeyTime {
        layer_id: String,
        key_id: u64,
        bar: f32,
    },
    ReparentClip {
        layer_id: String,
        dest_layer_id: String,
        bar: f32,
    },
    ToggleMute {
        layer_id: String,
    },
    ToggleSolo {
        layer_id: String,
    },
    /// Delete/Backspaceが選択中keyに当たった時。pointer clickでは出さない。
    RemovePositionKey {
        layer_id: String,
        key_id: u64,
    },
}

impl TimelineSession {
    /// 製品Timeline。fixture bands を初期状態にしない。
    pub(crate) fn host_product() -> Self {
        Self {
            scene: TimelineScene::empty_host(),
            gesture: None,
        }
    }

    /// scene差し替え時: 進行中gestureを復元せず破棄する。trueならdirty。
    pub(crate) fn discard_active_gesture(&mut self) -> bool {
        let gesture = self.gesture.take().is_some();
        let feedback = self.clear_drag_feedback();
        gesture || feedback
    }

    pub(crate) fn has_active_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    /// cursor用: clip本体drag中かどうかだけ公開する。
    pub(crate) fn gesture_kind_for_cursor(&self) -> Option<CursorDragKind> {
        match &self.gesture {
            Some(ActiveGesture::SelectOrMove { moving: true, .. }) => Some(CursorDragKind::Clip),
            _ => None,
        }
    }

    pub(crate) fn pointer(
        &mut self,
        selected: &mut i32,
        playhead: &mut f64,
        width: u32,
        height: u32,
        phase: TimelinePointerPhase,
        x: f64,
        y: f64,
        modifiers: u32,
    ) -> TimelinePointerOutcome {
        if width == 0 || height == 0 {
            return TimelinePointerOutcome::default();
        }
        let scale = f64::from(scale_for(width));
        let lx = x / scale;
        let ly = y / scale;
        let before_sel = *selected;
        let before_ph = *playhead;
        let mut dirty = false;
        let mut scrub_playhead = None;
        let mut scrub_release = false;
        let mut edit_commit = None;
        let mut selection_commit = None;

        match phase {
            TimelinePointerPhase::Down => {
                self.gesture = None;
                dirty |= self.clear_drag_feedback();
                if let Some(kind) = hit_gesture(&self.scene, *playhead, lx, ly) {
                    let snapshot = GestureSnapshot {
                        scene: self.scene.clone(),
                        selected: *selected,
                        playhead: *playhead,
                    };
                    match kind {
                        HitKind::Overview => {
                            let bar = overview_bar_at_lx(&self.scene, lx) as f32;
                            dirty |= center_view_on(&mut self.scene, bar);
                            self.gesture = Some(ActiveGesture::Overview { snapshot });
                        }
                        HitKind::Scrub | HitKind::Playhead => {
                            *playhead = time_at_lx(&self.scene, lx);
                            self.gesture = Some(ActiveGesture::Scrub { snapshot });
                            scrub_playhead = Some(*playhead);
                            dirty = true;
                        }
                        HitKind::Key {
                            band,
                            clip_idx,
                            key_idx,
                            flat,
                        } => {
                            clear_all_key_selection(&mut self.scene);
                            if let Some(key) =
                                self.scene.bands[band].clips[clip_idx].keys.get_mut(key_idx)
                            {
                                key.2 = true;
                            }
                            *selected = flat;
                            let layer_id = self.scene.bands[band].clips[clip_idx].layer_id.clone();
                            if self.scene.real {
                                if !layer_id.is_empty() {
                                    selection_commit = Some(TimelineSelectionCommit::SelectLayer {
                                        layer_id: layer_id.clone(),
                                    });
                                }
                            }
                            self.gesture = Some(ActiveGesture::KeyDrag {
                                snapshot,
                                band,
                                clip_idx,
                                key_idx,
                                key_id: self.scene.bands[band].clips[clip_idx].keys[key_idx].3,
                                layer_id,
                            });
                            dirty = true;
                        }
                        HitKind::TrimStart { band, clip_idx } => {
                            if self.scene.real {
                                let mut flat = 0i32;
                                for (bi, b) in self.scene.bands.iter().enumerate() {
                                    if bi == band {
                                        *selected = flat + clip_idx as i32;
                                        break;
                                    }
                                    flat += b.clips.len() as i32;
                                }
                                dirty = true;
                                let layer_id =
                                    self.scene.bands[band].clips[clip_idx].layer_id.clone();
                                if !layer_id.is_empty() {
                                    selection_commit =
                                        Some(TimelineSelectionCommit::SelectLayer { layer_id });
                                }
                            }
                            self.gesture = Some(ActiveGesture::TrimStart {
                                snapshot,
                                band,
                                clip_idx,
                                layer_id: self.scene.bands[band].clips[clip_idx].layer_id.clone(),
                            });
                        }
                        HitKind::TrimEnd { band, clip_idx } => {
                            if self.scene.real {
                                let mut flat = 0i32;
                                for (bi, b) in self.scene.bands.iter().enumerate() {
                                    if bi == band {
                                        *selected = flat + clip_idx as i32;
                                        break;
                                    }
                                    flat += b.clips.len() as i32;
                                }
                                dirty = true;
                                let layer_id =
                                    self.scene.bands[band].clips[clip_idx].layer_id.clone();
                                if !layer_id.is_empty() {
                                    selection_commit =
                                        Some(TimelineSelectionCommit::SelectLayer { layer_id });
                                }
                            }
                            self.gesture = Some(ActiveGesture::TrimEnd {
                                snapshot,
                                band,
                                clip_idx,
                                layer_id: self.scene.bands[band].clips[clip_idx].layer_id.clone(),
                            });
                        }
                        HitKind::Clip {
                            band,
                            clip_idx,
                            flat,
                        } => {
                            // clip本体クリックでkey選択を外し、Delete意味をlayer削除へ揃える。
                            clear_all_key_selection(&mut self.scene);
                            *selected = flat;
                            let clip = &self.scene.bands[band].clips[clip_idx];
                            if self.scene.real && !clip.layer_id.is_empty() {
                                selection_commit = Some(TimelineSelectionCommit::SelectLayer {
                                    layer_id: clip.layer_id.clone(),
                                });
                            }
                            self.gesture = Some(ActiveGesture::SelectOrMove {
                                snapshot,
                                band,
                                clip_idx,
                                layer_id: self.scene.bands[band].clips[clip_idx].layer_id.clone(),
                                press_lx: lx,
                                press_ly: ly,
                                origin_a: clip.a,
                                origin_b: clip.b,
                                origin_keys: clip.keys.iter().map(|k| k.0).collect(),
                                moving: false,
                            });
                            dirty = true;
                        }
                        HitKind::EmptyBar => {
                            *selected = -1;
                            if self.scene.real {
                                selection_commit = Some(TimelineSelectionCommit::ClearSelection);
                            }
                            self.gesture = Some(ActiveGesture::Deselect { snapshot });
                            dirty = true;
                        }
                        HitKind::Mute { band } => {
                            let layer_id = self.scene.bands[band]
                                .clips
                                .first()
                                .map(|clip| clip.layer_id.clone())
                                .unwrap_or_default();
                            self.gesture = Some(ActiveGesture::Mute { snapshot, layer_id });
                        }
                        HitKind::Solo { band } => {
                            let layer_id = self.scene.bands[band]
                                .clips
                                .first()
                                .map(|clip| clip.layer_id.clone())
                                .unwrap_or_default();
                            self.gesture = Some(ActiveGesture::Solo { snapshot, layer_id });
                        }
                    }
                }
            }
            TimelinePointerPhase::Move => {
                let scrubbing = matches!(self.gesture, Some(ActiveGesture::Scrub { .. }));
                dirty |= self.apply_move(lx, ly, playhead, modifiers);
                if scrubbing {
                    scrub_playhead = Some(*playhead);
                }
            }
            TimelinePointerPhase::Up => {
                if matches!(self.gesture, Some(ActiveGesture::Scrub { .. })) {
                    scrub_playhead = Some(*playhead);
                    scrub_release = true;
                }
                if self.scene.real {
                    if let Some(mut gesture) = self.gesture.take() {
                        if gesture_target_present(&self.scene, &gesture) {
                            refresh_gesture_indices(&self.scene, &mut gesture);
                            edit_commit = edit_commit_from_gesture(&self.scene, &gesture, ly);
                        }
                    }
                }
                self.gesture = None;
                dirty |= self.clear_drag_feedback();
            }
            TimelinePointerPhase::Cancel => {
                if let Some(gesture) = self.gesture.take() {
                    let snapshot = match gesture {
                        ActiveGesture::Scrub { snapshot }
                        | ActiveGesture::Overview { snapshot }
                        | ActiveGesture::SelectOrMove { snapshot, .. }
                        | ActiveGesture::TrimStart { snapshot, .. }
                        | ActiveGesture::TrimEnd { snapshot, .. }
                        | ActiveGesture::KeyDrag { snapshot, .. }
                        | ActiveGesture::Deselect { snapshot }
                        | ActiveGesture::Mute { snapshot, .. }
                        | ActiveGesture::Solo { snapshot, .. } => snapshot,
                    };
                    if self.scene != snapshot.scene {
                        self.scene = snapshot.scene;
                    }
                    *selected = snapshot.selected;
                    *playhead = snapshot.playhead;
                    dirty = true;
                }
            }
        }

        let _ = height; // surface高さはhit範囲に使わず、論理layout定数で判定する
        let feedback = *selected != before_sel || (*playhead - before_ph).abs() > f64::EPSILON;
        TimelinePointerOutcome {
            feedback,
            dirty,
            scrub_playhead,
            scrub_release,
            edit_commit,
            selection_commit,
        }
    }

    fn clear_drag_feedback(&mut self) -> bool {
        let snap = self.scene.snap_guide.take().is_some();
        let lane = self.scene.lane_preview_band.take().is_some();
        snap || lane
    }

    fn apply_move(&mut self, lx: f64, ly: f64, playhead: &mut f64, modifiers: u32) -> bool {
        let Some(mut gesture) = self.gesture.take() else {
            return false;
        };
        // 対象は LayerId/KeyframeId。indexは描画用にその場解決する。
        if !gesture_target_present(&self.scene, &gesture) {
            return self.clear_drag_feedback();
        }
        refresh_gesture_indices(&self.scene, &mut gesture);
        let mut dirty = false;
        match &mut gesture {
            ActiveGesture::Scrub { .. } => {
                let next = time_at_lx(&self.scene, lx);
                if (next - *playhead).abs() > f64::EPSILON {
                    *playhead = next;
                    dirty = true;
                }
            }
            ActiveGesture::Overview { .. } => {
                let bar = overview_bar_at_lx(&self.scene, lx) as f32;
                dirty |= center_view_on(&mut self.scene, bar);
            }
            ActiveGesture::SelectOrMove {
                snapshot,
                band,
                clip_idx,
                layer_id,
                press_lx,
                press_ly,
                origin_a,
                origin_b,
                origin_keys,
                moving,
                ..
            } => {
                let dx_logical = lx - *press_lx;
                let dy_logical = ly - *press_ly;
                if !*moving && (dx_logical.abs() > MOVE_ARM_PX || dy_logical.abs() > MOVE_ARM_PX) {
                    *moving = true;
                }
                if *moving {
                    let lane_preview_band =
                        reparent_destination_band(&self.scene, snapshot, layer_id, *press_ly, ly);
                    if self.scene.lane_preview_band != lane_preview_band {
                        self.scene.lane_preview_band = lane_preview_band;
                        dirty = true;
                    }
                    let span = f64::from(self.scene.view_b - self.scene.view_a);
                    let dx_bars = (dx_logical / surface_width()) * span;
                    let (prev_b, next_a) = neighbors(&self.scene, *band, *clip_idx);
                    let len = *origin_b - *origin_a;
                    let raw_a = clamp_ordered(
                        (*origin_a as f64) + dx_bars,
                        prev_b as f64,
                        (next_a - len) as f64,
                    ) as f32;
                    let (snapped_a, snap_guide) = snap_bar_with_guide(
                        &self.scene,
                        *playhead,
                        *band,
                        Some(*clip_idx),
                        raw_a,
                        modifiers,
                    );
                    let new_a = clamp_ordered_f32(snapped_a, prev_b, next_a - len);
                    let snap_guide = if dx_logical.abs() > MOVE_ARM_PX && snap_guide == Some(new_a)
                    {
                        snap_guide
                    } else {
                        None
                    };
                    if self.scene.snap_guide != snap_guide {
                        self.scene.snap_guide = snap_guide;
                        dirty = true;
                    }
                    let new_b = new_a + len;
                    let move_keys = !self.scene.real;
                    let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                    if (clip.a - new_a).abs() > f32::EPSILON
                        || (clip.b - new_b).abs() > f32::EPSILON
                    {
                        clip.a = new_a;
                        clip.b = new_b;
                        // realのcommitはSetClipStartのみ。key同伴はfixtureだけ。
                        if move_keys {
                            let key_times: Vec<f32> = origin_keys
                                .iter()
                                .map(|&ot| ot + (new_a - *origin_a))
                                .collect();
                            for (key, nt) in clip.keys.iter_mut().zip(key_times) {
                                key.0 = nt;
                            }
                        }
                        dirty = true;
                    }
                }
            }
            ActiveGesture::TrimStart { band, clip_idx, .. } => {
                let bar = bar_at_lx(&self.scene, lx) as f32;
                let (prev_b, _) = neighbors(&self.scene, *band, *clip_idx);
                let clip_b = self.scene.bands[*band].clips[*clip_idx].b;
                let min_clip = min_clip_units(&self.scene);
                let raw_a = clamp_ordered_f32(bar, prev_b, clip_b - min_clip);
                let (snapped_a, snap_guide) = snap_bar_with_guide(
                    &self.scene,
                    *playhead,
                    *band,
                    Some(*clip_idx),
                    raw_a,
                    modifiers,
                );
                let new_a = clamp_ordered_f32(snapped_a, prev_b, clip_b - min_clip);
                let snap_guide = snap_guide.filter(|guide| *guide == new_a);
                if self.scene.snap_guide != snap_guide {
                    self.scene.snap_guide = snap_guide;
                    dirty = true;
                }
                let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                if (clip.a - new_a).abs() > f32::EPSILON {
                    clip.a = new_a;
                    dirty = true;
                }
            }
            ActiveGesture::TrimEnd { band, clip_idx, .. } => {
                let bar = bar_at_lx(&self.scene, lx) as f32;
                let (_, next_a) = neighbors(&self.scene, *band, *clip_idx);
                let clip_a = self.scene.bands[*band].clips[*clip_idx].a;
                let min_clip = min_clip_units(&self.scene);
                let raw_b = clamp_ordered_f32(bar, clip_a + min_clip, next_a);
                let (snapped_b, snap_guide) = snap_bar_with_guide(
                    &self.scene,
                    *playhead,
                    *band,
                    Some(*clip_idx),
                    raw_b,
                    modifiers,
                );
                let new_b = clamp_ordered_f32(snapped_b, clip_a + min_clip, next_a);
                let snap_guide = snap_guide.filter(|guide| *guide == new_b);
                if self.scene.snap_guide != snap_guide {
                    self.scene.snap_guide = snap_guide;
                    dirty = true;
                }
                let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                if (clip.b - new_b).abs() > f32::EPSILON {
                    clip.b = new_b;
                    dirty = true;
                }
            }
            ActiveGesture::KeyDrag {
                band,
                clip_idx,
                key_idx,
                ..
            } => {
                let bar = bar_at_lx(&self.scene, lx) as f32;
                let (clip_a, clip_b) = {
                    let clip = &self.scene.bands[*band].clips[*clip_idx];
                    (clip.a, clip.b)
                };
                let raw_t = clamp_ordered_f32(bar, clip_a, clip_b);
                let (snapped_t, snap_guide) = snap_bar_with_guide(
                    &self.scene,
                    *playhead,
                    *band,
                    Some(*clip_idx),
                    raw_t,
                    modifiers,
                );
                let new_t = clamp_ordered_f32(snapped_t, clip_a, clip_b);
                let snap_guide = snap_guide.filter(|guide| *guide == new_t);
                if self.scene.snap_guide != snap_guide {
                    self.scene.snap_guide = snap_guide;
                    dirty = true;
                }
                if let Some(key) = self.scene.bands[*band].clips[*clip_idx]
                    .keys
                    .get_mut(*key_idx)
                {
                    if (key.0 - new_t).abs() > f32::EPSILON {
                        key.0 = new_t;
                        dirty = true;
                    }
                }
            }
            ActiveGesture::Deselect { .. }
            | ActiveGesture::Mute { .. }
            | ActiveGesture::Solo { .. } => {}
        }
        self.gesture = Some(gesture);
        dirty
    }

    /// wheel / pinch。戻り値trueは視覚変化(dirty)。feedbackは出さない。
    pub(crate) fn scroll(
        &mut self,
        width: u32,
        height: u32,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        let _ = (height, y);
        let scale = f64::from(scale_for(width));
        let lx = x / scale;
        let dx_log = delta_x / scale;
        let dy_log = delta_y / scale;
        let before = (self.scene.view_a, self.scene.view_b);

        if magnification != 0.0 {
            zoom_at(&mut self.scene, lx, 1.0 - magnification);
        } else if modifiers & 1 != 0 {
            zoom_at(&mut self.scene, lx, (-dy_log * 0.01).exp());
        } else {
            let delta = if dx_log.abs() >= dy_log.abs() {
                dx_log
            } else {
                dy_log
            };
            let span = f64::from(self.scene.view_b - self.scene.view_a);
            let dx_bars = (-delta / surface_width() * span) as f32;
            self.scene.view_a += dx_bars;
            self.scene.view_b += dx_bars;
            clamp_view_translate(&mut self.scene);
        }

        self.scene.view_a != before.0 || self.scene.view_b != before.1
    }
}

/// realは LayerId、fixture（空id）は保存index。描画indexをidentityにしない。
pub(super) fn clip_location(
    scene: &TimelineScene,
    layer_id: &str,
    band: usize,
    clip_idx: usize,
) -> Option<(usize, usize)> {
    if !layer_id.is_empty() {
        for (band_idx, band) in scene.bands.iter().enumerate() {
            if let Some(clip_idx) = band.clips.iter().position(|clip| clip.layer_id == layer_id) {
                return Some((band_idx, clip_idx));
            }
        }
        return None;
    }
    let _ = scene.bands.get(band)?.clips.get(clip_idx)?;
    Some((band, clip_idx))
}

pub(super) fn key_location(clip: &Clip, key_id: u64, key_idx: usize) -> Option<usize> {
    if key_id != 0 {
        return clip.keys.iter().position(|key| key.3 == key_id);
    }
    (key_idx < clip.keys.len()).then_some(key_idx)
}

pub(super) fn clip_ref<'a>(
    scene: &'a TimelineScene,
    layer_id: &str,
    band: usize,
    clip_idx: usize,
) -> Option<&'a Clip> {
    let (band, clip_idx) = clip_location(scene, layer_id, band, clip_idx)?;
    scene.bands.get(band)?.clips.get(clip_idx)
}

pub(super) fn key_ref(clip: &Clip, key_id: u64, key_idx: usize) -> Option<&(f32, f32, bool, u64)> {
    let idx = key_location(clip, key_id, key_idx)?;
    clip.keys.get(idx)
}

pub(super) fn gesture_target_present(scene: &TimelineScene, gesture: &ActiveGesture) -> bool {
    match gesture {
        ActiveGesture::Scrub { .. }
        | ActiveGesture::Overview { .. }
        | ActiveGesture::Deselect { .. }
        | ActiveGesture::Mute { .. }
        | ActiveGesture::Solo { .. } => true,
        ActiveGesture::SelectOrMove {
            band,
            clip_idx,
            layer_id,
            ..
        }
        | ActiveGesture::TrimStart {
            band,
            clip_idx,
            layer_id,
            ..
        }
        | ActiveGesture::TrimEnd {
            band,
            clip_idx,
            layer_id,
            ..
        } => clip_location(scene, layer_id, *band, *clip_idx).is_some(),
        ActiveGesture::KeyDrag {
            band,
            clip_idx,
            key_idx,
            key_id,
            layer_id,
            ..
        } => clip_location(scene, layer_id, *band, *clip_idx)
            .and_then(|(band, clip_idx)| {
                scene
                    .bands
                    .get(band)
                    .and_then(|band| band.clips.get(clip_idx))
                    .and_then(|clip| key_location(clip, *key_id, *key_idx))
            })
            .is_some(),
    }
}

pub(super) fn refresh_gesture_indices(scene: &TimelineScene, gesture: &mut ActiveGesture) {
    match gesture {
        ActiveGesture::SelectOrMove {
            band,
            clip_idx,
            layer_id,
            ..
        }
        | ActiveGesture::TrimStart {
            band,
            clip_idx,
            layer_id,
            ..
        }
        | ActiveGesture::TrimEnd {
            band,
            clip_idx,
            layer_id,
            ..
        } => {
            if let Some((next_band, next_clip)) = clip_location(scene, layer_id, *band, *clip_idx) {
                *band = next_band;
                *clip_idx = next_clip;
            }
        }
        ActiveGesture::KeyDrag {
            band,
            clip_idx,
            key_idx,
            key_id,
            layer_id,
            ..
        } => {
            if let Some((next_band, next_clip)) = clip_location(scene, layer_id, *band, *clip_idx) {
                if let Some(next_key) = scene
                    .bands
                    .get(next_band)
                    .and_then(|band| band.clips.get(next_clip))
                    .and_then(|clip| key_location(clip, *key_id, *key_idx))
                {
                    *band = next_band;
                    *clip_idx = next_clip;
                    *key_idx = next_key;
                }
            }
        }
        _ => {}
    }
}

pub(super) fn edit_commit_from_gesture(
    scene: &TimelineScene,
    gesture: &ActiveGesture,
    ly: f64,
) -> Option<TimelineEditCommit> {
    match gesture {
        ActiveGesture::SelectOrMove {
            snapshot,
            band,
            clip_idx,
            layer_id,
            origin_a,
            moving,
            press_ly,
            ..
        } => {
            if !*moving {
                return None;
            }
            let clip = clip_ref(scene, layer_id, *band, *clip_idx)?;
            if clip.layer_id.is_empty() {
                return None;
            }
            let press_row = band_index_at_ly(&snapshot.scene, *press_ly);
            let dest_row = band_index_at_ly(scene, ly);
            if press_row != dest_row {
                if let Some(dest_band) = dest_row {
                    if let Some(dest_id) = scene
                        .bands
                        .get(dest_band)
                        .and_then(|band| band.clips.first())
                        .map(|dest| dest.layer_id.clone())
                        .filter(|dest| !dest.is_empty() && dest != &clip.layer_id)
                    {
                        return Some(TimelineEditCommit::ReparentClip {
                            layer_id: clip.layer_id.clone(),
                            dest_layer_id: dest_id,
                            bar: clip.a,
                        });
                    }
                }
            }
            if (clip.a - *origin_a).abs() <= f32::EPSILON {
                return None;
            }
            Some(TimelineEditCommit::SetClipStart {
                layer_id: clip.layer_id.clone(),
                bar: clip.a,
            })
        }
        ActiveGesture::TrimStart {
            snapshot,
            band,
            clip_idx,
            layer_id,
        } => {
            let clip = clip_ref(scene, layer_id, *band, *clip_idx)?;
            let before = clip_ref(&snapshot.scene, layer_id, *band, *clip_idx)?;
            if (clip.a - before.a).abs() <= f32::EPSILON || clip.layer_id.is_empty() {
                return None;
            }
            Some(TimelineEditCommit::TrimClipIn {
                layer_id: clip.layer_id.clone(),
                bar: clip.a,
            })
        }
        ActiveGesture::TrimEnd {
            snapshot,
            band,
            clip_idx,
            layer_id,
        } => {
            let clip = clip_ref(scene, layer_id, *band, *clip_idx)?;
            let before = clip_ref(&snapshot.scene, layer_id, *band, *clip_idx)?;
            if (clip.b - before.b).abs() <= f32::EPSILON || clip.layer_id.is_empty() {
                return None;
            }
            Some(TimelineEditCommit::TrimClipOut {
                layer_id: clip.layer_id.clone(),
                bar: clip.b,
            })
        }
        ActiveGesture::KeyDrag {
            snapshot,
            band,
            clip_idx,
            key_idx,
            layer_id,
            key_id,
        } => {
            let clip = clip_ref(scene, layer_id, *band, *clip_idx)?;
            let key = key_ref(clip, *key_id, *key_idx)?;
            let before_clip = clip_ref(&snapshot.scene, layer_id, *band, *clip_idx)?;
            let before = key_ref(before_clip, *key_id, *key_idx)?;
            if (key.0 - before.0).abs() <= f32::EPSILON || clip.layer_id.is_empty() {
                return None;
            }
            Some(TimelineEditCommit::SetPositionKeyTime {
                layer_id: clip.layer_id.clone(),
                key_id: key.3,
                bar: key.0,
            })
        }
        ActiveGesture::Scrub { .. }
        | ActiveGesture::Overview { .. }
        | ActiveGesture::Deselect { .. } => None,
        ActiveGesture::Mute { layer_id, .. } => {
            if layer_id.is_empty() {
                None
            } else {
                Some(TimelineEditCommit::ToggleMute {
                    layer_id: layer_id.clone(),
                })
            }
        }
        ActiveGesture::Solo { layer_id, .. } => {
            if layer_id.is_empty() {
                None
            } else {
                Some(TimelineEditCommit::ToggleSolo {
                    layer_id: layer_id.clone(),
                })
            }
        }
    }
}

pub(super) fn reparent_destination_band(
    scene: &TimelineScene,
    snapshot: &GestureSnapshot,
    source_layer_id: &str,
    press_ly: f64,
    current_ly: f64,
) -> Option<usize> {
    if !scene.real || source_layer_id.is_empty() {
        return None;
    }
    let source_band = band_index_at_ly(&snapshot.scene, press_ly)?;
    let destination_band = band_index_at_ly(scene, current_ly)?;
    if source_band == destination_band {
        return None;
    }
    let destination_layer_id = scene
        .bands
        .get(destination_band)?
        .clips
        .first()?
        .layer_id
        .as_str();
    (!destination_layer_id.is_empty() && destination_layer_id != source_layer_id)
        .then_some(destination_band)
}

/// Delete/Backspace: 選択中real keyがあれば remove_position_key。HitKindは増やさない。
pub(super) fn remove_position_key_commit(scene: &TimelineScene) -> Option<TimelineEditCommit> {
    let (layer_id, key_id) = selected_real_key(scene)?;
    Some(TimelineEditCommit::RemovePositionKey { layer_id, key_id })
}
