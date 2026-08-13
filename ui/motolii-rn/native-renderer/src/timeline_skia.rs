//! Skia製Timelineのraster。
//!
//! 製品は host の Document 投影（layer / envelope区間 / position key / playhead）だけを描く。
//! `Default` は既存test用の probe fixture であり、製品初期状態には使わない。

use skia_safe::{
    AlphaType, Color, ColorType, Font, FontMgr, FontStyle, ImageInfo, Paint, PaintStyle,
    PathBuilder, Rect, surfaces,
};

const W: f32 = 1240.0;
const INBOX_W: f32 = 118.0;
const RAIL_W: f32 = 84.0;
const SURF_X: f32 = INBOX_W + RAIL_W;
const OVER_H: f32 = 22.0;
const RULER_H: f32 = 18.0;
const LOC_H: f32 = 15.0;
const ROW: f32 = 20.0;
const TIME_H: f32 = 16.0;
pub(crate) const SONG_BARS: f32 = 96.0;
/// 内部単位は秒。旧 1 bar = 2秒は廃止。
pub(crate) const SECONDS_PER_BAR: f64 = 1.0;
/// real投影の曲長[秒]。Composition::new_v1()=10秒。
const MIN_VIEW_SPAN: f32 = 4.0;
const DEFAULT_VIEW_A: f32 = 0.0;
const DEFAULT_VIEW_B: f32 = 48.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const SURFACE_HI: u32 = 0x464646;
const SURFACE_LO: u32 = 0x242424;
const CONTRAST: u32 = 0x111111;
const DIM: u32 = 0x757575;
const RULER_MARK: u32 = 0x919191;
const FILL_HANDLE: u32 = 0x5d5d5d;
const ON_BAR: u32 = 0x141414;
const ACCENT: u32 = 0xffad56;
const P: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

/// 菱形外stroke(path 5.6)と一致させる。視覚は変えない。
const KEY_HIT_PX: f64 = 5.6;
const TRIM_EDGE_MAX_PX: f64 = 15.0;
const TRIM_EDGE_MIN_CLIP_W: f64 = 25.0;
const TRIM_EDGE_MIN_CLIP_H: f64 = 16.0;
/// playhead頭+線の掴み半幅(論理px)。
const PLAYHEAD_HIT_PX: f64 = 4.0;
const MOVE_ARM_PX: f64 = 3.0;
const SNAP_THRESHOLD_LOGICAL_PX: f64 = 6.0;
const MIN_CLIP_BARS: f32 = 1.0 / 30.0;
const SNAPSHOT_KEY_DIAMOND: f32 = 0.42;
const DEFAULT_FPS_NUM: i64 = 30;
const DEFAULT_FPS_DEN: i64 = 1;

/// Host snapshot由来の1 layer行。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SnapshotLayerInput {
    pub layer_id: String,
    pub display_name: String,
    /// `(start_secs, duration_secs)`。Noneは旧host互換の full-width。
    pub interval_secs: Option<(f64, f64)>,
    pub keys: Vec<SnapshotKeyInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SnapshotKeyInput {
    pub key_id: u64,
    pub time_secs: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct Clip {
    a: f32,
    b: f32,
    slot: usize,
    name: String,
    /// real投影のhost layer id。fixtureは空。
    layer_id: String,
    fx: (u8, u8),
    mute: bool,
    dev: &'static [&'static str],
    /// (秒, diamond size, selected, key_id)。fixtureのkey_idは0。
    keys: Vec<(f32, f32, bool, u64)>,
}

#[derive(Clone, Debug, PartialEq)]
struct Band {
    mute: bool,
    solo: bool,
    mixed: bool,
    clips: Vec<Clip>,
}

/// fixture scene。初期値は旧`BANDS`/`LOCATORS`と同一。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TimelineScene {
    bands: Vec<Band>,
    locators: Vec<(f32, &'static str)>,
    pub view_a: f32,
    pub view_b: f32,
    /// 曲全体の長さ[秒]。fixture=96、real=composition秒。
    pub song_bars: f32,
    pub fps_num: i64,
    pub fps_den: i64,
    /// Host snapshot投影。trueの時はreleaseでDocumentへdispatchする。
    pub real: bool,
    /// revision反映時にlocal選択へ載せるflat index。fixtureは未使用。
    pub selected_flat: i32,
    /// gesture中だけ描く吸着位置。Document投影へは載せない。
    snap_guide: Option<f32>,
    /// clip lane drag中だけ描く移動先行。Document投影へは載せない。
    lane_preview_band: Option<usize>,
}

impl Default for TimelineScene {
    fn default() -> Self {
        Self {
            bands: vec![
                Band {
                    mute: false,
                    solo: false,
                    mixed: false,
                    clips: vec![
                        Clip {
                            a: 0.0,
                            b: 14.0,
                            slot: 0,
                            name: "sky_plate".into(),
                            layer_id: String::new(),
                            fx: (0, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 14.0,
                            b: 27.0,
                            slot: 0,
                            name: "sky_plate".into(),
                            layer_id: String::new(),
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![(14.0, 0.42, false, 0), (20.0, 0.03, true, 0)],
                        },
                        Clip {
                            a: 30.0,
                            b: 44.0,
                            slot: 1,
                            name: "city_a".into(),
                            layer_id: String::new(),
                            fx: (3, 1),
                            mute: false,
                            dev: &["retime"],
                            keys: vec![],
                        },
                    ],
                },
                Band {
                    mute: false,
                    solo: true,
                    mixed: false,
                    clips: vec![
                        Clip {
                            a: 4.0,
                            b: 22.0,
                            slot: 3,
                            name: "hero".into(),
                            layer_id: String::new(),
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![
                                (8.0, 0.71, false, 0),
                                (13.0, 0.02, false, 0),
                                (18.0, 0.15, false, 0),
                            ],
                        },
                        Clip {
                            a: 26.0,
                            b: 40.0,
                            slot: 3,
                            name: "hero".into(),
                            layer_id: String::new(),
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![(31.0, 0.30, false, 0)],
                        },
                    ],
                },
                Band {
                    mute: false,
                    solo: false,
                    mixed: true,
                    clips: vec![
                        Clip {
                            a: 0.0,
                            b: 18.0,
                            slot: 2,
                            name: "grain".into(),
                            layer_id: String::new(),
                            fx: (1, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 20.0,
                            b: 35.0,
                            slot: 2,
                            name: "grain".into(),
                            layer_id: String::new(),
                            fx: (1, 0),
                            mute: true,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 38.0,
                            b: 48.0,
                            slot: 2,
                            name: "grain".into(),
                            layer_id: String::new(),
                            fx: (1, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![],
                        },
                    ],
                },
                Band {
                    mute: false,
                    solo: false,
                    mixed: false,
                    clips: vec![
                        Clip {
                            a: 6.0,
                            b: 13.0,
                            slot: 5,
                            name: "title_01".into(),
                            layer_id: String::new(),
                            fx: (0, 0),
                            mute: false,
                            dev: &["blend", "opacity"],
                            keys: vec![],
                        },
                        Clip {
                            a: 24.0,
                            b: 32.0,
                            slot: 5,
                            name: "title_02".into(),
                            layer_id: String::new(),
                            fx: (0, 0),
                            mute: false,
                            dev: &["opacity"],
                            keys: vec![(24.0, 0.5, false, 0)],
                        },
                    ],
                },
                // 空の帯。利用者が作った器なので残る
                Band {
                    mute: false,
                    solo: false,
                    mixed: false,
                    clips: vec![],
                },
                Band {
                    mute: false,
                    solo: false,
                    mixed: false,
                    clips: vec![Clip {
                        a: 0.0,
                        b: 48.0,
                        slot: 1,
                        name: "track_master.wav".into(),
                        layer_id: String::new(),
                        fx: (0, 0),
                        mute: false,
                        dev: &[],
                        keys: vec![],
                    }],
                },
            ],
            locators: vec![
                (0.0, "intro"),
                (8.0, "verse 1"),
                (24.0, "chorus"),
                (40.0, "verse 2"),
            ],
            view_a: DEFAULT_VIEW_A,
            view_b: DEFAULT_VIEW_B,
            song_bars: SONG_BARS,
            fps_num: DEFAULT_FPS_NUM,
            fps_den: DEFAULT_FPS_DEN,
            real: false,
            selected_flat: -1,
            snap_guide: None,
            lane_preview_band: None,
        }
    }
}

impl TimelineScene {
    pub(crate) fn band_count(&self) -> usize {
        self.bands.len()
    }

    /// 製品初期状態。host が空なら composition だけ（bandsなし）。
    pub(crate) fn empty_host() -> Self {
        Self::from_snapshot_with_song_bars(&[], None, 10.0)
    }

    /// Host snapshotの実layer投影。1 layer = 1 band。
    /// `interval_secs`がSomeなら start/duration 秒を内部単位(秒)へ載せる。
    /// Noneは旧host互換の full-width(0..song_bars)。
    pub(crate) fn from_snapshot(
        layers: &[SnapshotLayerInput],
        primary_layer_id: Option<&str>,
    ) -> Self {
        Self::from_snapshot_with_song_bars(layers, primary_layer_id, SONG_BARS)
    }

    /// Host snapshot由来の1 layer行。`song_bars`を上書きして生成。
    pub(crate) fn from_snapshot_with_song_bars(
        layers: &[SnapshotLayerInput],
        primary_layer_id: Option<&str>,
        song_bars: f32,
    ) -> Self {
        let mut song_bars = song_bars;
        if !song_bars.is_finite() || song_bars < 0.0 {
            song_bars = 0.0;
        }
        let mut selected_flat = -1i32;
        let bands = layers
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                let flat = index as i32;
                if primary_layer_id == Some(layer.layer_id.as_str()) {
                    selected_flat = flat;
                }
                let (a, b) = match layer.interval_secs {
                    Some((start_secs, duration_secs)) => {
                        let mut a = (start_secs / SECONDS_PER_BAR) as f32;
                        let mut b = ((start_secs + duration_secs) / SECONDS_PER_BAR) as f32;
                        a = a.clamp(0.0, song_bars);
                        b = b.clamp(0.0, song_bars);
                        if b < a {
                            b = a;
                        }
                        (a, b)
                    }
                    None => (0.0, song_bars),
                };
                let keys = layer
                    .keys
                    .iter()
                    .map(|key| {
                        let bar = (key.time_secs / SECONDS_PER_BAR) as f32;
                        (bar, SNAPSHOT_KEY_DIAMOND, false, key.key_id)
                    })
                    .collect();
                Band {
                    mute: false,
                    solo: false,
                    mixed: false,
                    clips: vec![Clip {
                        a,
                        b,
                        slot: index % 6,
                        name: layer.display_name.clone(),
                        layer_id: layer.layer_id.clone(),
                        fx: (0, 0),
                        mute: false,
                        dev: &[],
                        keys,
                    }],
                }
            })
            .collect();
        Self {
            bands,
            locators: vec![],
            // real初期viewはcomposition全体をfit。
            view_a: 0.0,
            view_b: song_bars,
            song_bars,
            fps_num: DEFAULT_FPS_NUM,
            fps_den: DEFAULT_FPS_DEN,
            real: true,
            selected_flat,
            snap_guide: None,
            lane_preview_band: None,
        }
    }

    pub(crate) fn with_fps(mut self, num: i64, den: i64) -> Self {
        if num > 0 && den > 0 {
            self.fps_num = num;
            self.fps_den = den;
        }
        self
    }

    /// host `visible`/`solo`/effect数を band の M/S と既存 fx バッジへ載せる。1 layer = 1 band。
    pub(crate) fn apply_layer_mute_solo(
        &mut self,
        flags: impl IntoIterator<Item = (bool, bool, usize)>,
    ) {
        for (band, (visible, solo, effect_count)) in self.bands.iter_mut().zip(flags) {
            band.mute = !visible;
            band.solo = solo;
            if let Some(clip) = band.clips.first_mut() {
                clip.mute = !visible;
                clip.fx = (effect_count.min(255) as u8, 0);
            }
        }
    }

    /// 実投影clipの span と key(秒, key_id)。test / bridge検証用。
    pub(crate) fn clip0_span_and_keys(&self, band: usize) -> Option<(f32, f32, Vec<(f32, u64)>)> {
        let clip = self.bands.get(band)?.clips.first()?;
        Some((
            clip.a,
            clip.b,
            clip.keys.iter().map(|key| (key.0, key.3)).collect(),
        ))
    }

    /// 実投影clipのhost layer id。test / bridge検証用。
    pub(crate) fn clip0_layer_id(&self, band: usize) -> Option<&str> {
        self.bands
            .get(band)?
            .clips
            .first()
            .map(|clip| clip.layer_id.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelinePointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Debug)]
struct GestureSnapshot {
    scene: TimelineScene,
    selected: i32,
    playhead: f64,
}

#[derive(Clone, Debug)]
enum ActiveGesture {
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
pub(crate) enum CursorDragKind {
    Clip,
}

/// scene + 進行中gesture。selection/playheadはcallerが所有する。
#[derive(Clone, Debug, Default)]
pub(crate) struct TimelineSession {
    pub scene: TimelineScene,
    gesture: Option<ActiveGesture>,
}

/// pointer処理の結果。`feedback`はselection/playhead変化時のみtrue。
#[derive(Clone, Debug, Default)]
pub(crate) struct TimelinePointerOutcome {
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
pub(crate) enum TimelineSelectionCommit {
    SelectLayer { layer_id: String },
    ClearSelection,
}

/// real Timelineのrelease時にhostへ送る編集intent。
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TimelineEditCommit {
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
fn clip_location(
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

fn key_location(clip: &Clip, key_id: u64, key_idx: usize) -> Option<usize> {
    if key_id != 0 {
        return clip.keys.iter().position(|key| key.3 == key_id);
    }
    (key_idx < clip.keys.len()).then_some(key_idx)
}

fn clip_ref<'a>(
    scene: &'a TimelineScene,
    layer_id: &str,
    band: usize,
    clip_idx: usize,
) -> Option<&'a Clip> {
    let (band, clip_idx) = clip_location(scene, layer_id, band, clip_idx)?;
    scene.bands.get(band)?.clips.get(clip_idx)
}

fn key_ref(clip: &Clip, key_id: u64, key_idx: usize) -> Option<&(f32, f32, bool, u64)> {
    let idx = key_location(clip, key_id, key_idx)?;
    clip.keys.get(idx)
}

fn gesture_target_present(scene: &TimelineScene, gesture: &ActiveGesture) -> bool {
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

fn refresh_gesture_indices(scene: &TimelineScene, gesture: &mut ActiveGesture) {
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

fn edit_commit_from_gesture(
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

#[derive(Clone, Copy, Debug)]
enum HitKind {
    Overview,
    /// playhead頭/線。Scrubと同じ文法へ落とす。
    Playhead,
    Scrub,
    Key {
        band: usize,
        clip_idx: usize,
        key_idx: usize,
        flat: i32,
    },
    TrimStart {
        band: usize,
        clip_idx: usize,
    },
    TrimEnd {
        band: usize,
        clip_idx: usize,
    },
    Clip {
        band: usize,
        clip_idx: usize,
        flat: i32,
    },
    EmptyBar,
    Mute {
        band: usize,
    },
    Solo {
        band: usize,
    },
}

/// hover/cursor用の粗いhit種。gesture内部indexは持たない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineHoverHit {
    None,
    PlayheadOrRuler,
    Key,
    Trim,
    Clip,
}

/// OSカーソル写像の閉集合。ObjC側は薄い分岐だけ。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CursorKind {
    Arrow = 0,
    ResizeLeftRight = 1,
    OpenHand = 2,
    ClosedHand = 3,
    PointingHand = 4,
}

impl CursorKind {
    pub(crate) fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Timeline hover hit → cursor。clip drag中はhitを外れてもclosedHand維持(gesture active優先)。
pub(crate) fn cursor_for_timeline_hover(hit: TimelineHoverHit, clip_dragging: bool) -> CursorKind {
    if clip_dragging {
        return CursorKind::ClosedHand;
    }
    match hit {
        TimelineHoverHit::None => CursorKind::Arrow,
        TimelineHoverHit::PlayheadOrRuler | TimelineHoverHit::Trim => CursorKind::ResizeLeftRight,
        TimelineHoverHit::Key => CursorKind::PointingHand,
        TimelineHoverHit::Clip => CursorKind::OpenHand,
    }
}

/// Stage上の選択可能物hover → cursor。drag中は物体を外れてもclosedHand維持。
pub(crate) fn cursor_for_stage_hover(over_layer: bool, dragging: bool) -> CursorKind {
    if dragging {
        return CursorKind::ClosedHand;
    }
    if over_layer {
        CursorKind::OpenHand
    } else {
        CursorKind::Arrow
    }
}

/// 範囲が逆転していてもpanicしないclamp(逆転時はminへ寄せる)。
/// gesture中のscene差し替え等でclipが隙間より長い縮退があり得るため、
/// `f64::clamp`のmin>max panicを構造的に排除する。
fn clamp_ordered(v: f64, lo: f64, hi: f64) -> f64 {
    if !v.is_finite() || !lo.is_finite() || !hi.is_finite() {
        return if lo.is_finite() { lo } else { 0.0 };
    }
    if hi < lo { lo } else { v.clamp(lo, hi) }
}

fn clamp_ordered_f32(v: f32, lo: f32, hi: f32) -> f32 {
    clamp_ordered(f64::from(v), f64::from(lo), f64::from(hi)) as f32
}

fn surface_width() -> f64 {
    f64::from(W - SURF_X - 6.0)
}

fn bx(scene: &TimelineScene, b: f32) -> f32 {
    SURF_X + (b - scene.view_a) / (scene.view_b - scene.view_a) * (W - SURF_X - 6.0)
}

fn bar_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    f64::from(scene.view_a)
        + (lx - f64::from(SURF_X)) / surface_width() * f64::from(scene.view_b - scene.view_a)
}

fn overview_bar_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    ((lx - f64::from(SURF_X)) / surface_width() * f64::from(scene.song_bars))
        .clamp(0.0, f64::from(scene.song_bars))
}

fn time_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    (bar_at_lx(scene, lx) / f64::from(scene.song_bars)).clamp(0.0, 1.0)
}

fn clamp_view_translate(scene: &mut TimelineScene) {
    let span = (scene.view_b - scene.view_a)
        .max(0.0)
        .min(scene.song_bars.max(0.0));
    if scene.view_a < 0.0 {
        scene.view_a = 0.0;
        scene.view_b = span;
    }
    if scene.view_b > scene.song_bars {
        scene.view_b = scene.song_bars;
        scene.view_a = (scene.song_bars - span).max(0.0);
    }
    if scene.view_a < 0.0 || scene.view_b < scene.view_a {
        scene.view_a = 0.0;
        scene.view_b = span;
    }
}

fn center_view_on(scene: &mut TimelineScene, center: f32) -> bool {
    let span = (scene.view_b - scene.view_a)
        .max(0.0)
        .min(scene.song_bars.max(0.0));
    let mut a = center - span * 0.5;
    let mut b = a + span;
    if a < 0.0 {
        a = 0.0;
        b = span;
    }
    if b > scene.song_bars {
        b = scene.song_bars;
        a = scene.song_bars - span;
    }
    if (scene.view_a - a).abs() > f32::EPSILON || (scene.view_b - b).abs() > f32::EPSILON {
        scene.view_a = a;
        scene.view_b = b;
        true
    } else {
        false
    }
}

fn zoom_at(scene: &mut TimelineScene, lx: f64, span_factor: f64) {
    let va = f64::from(scene.view_a);
    let vb = f64::from(scene.view_b);
    let span = vb - va;
    if span <= f64::EPSILON {
        return;
    }
    let anchor = bar_at_lx(scene, lx);
    let min_span = f64::from(scene.song_bars.min(MIN_VIEW_SPAN));
    let new_span =
        (span * span_factor).clamp(min_span, f64::from(scene.song_bars.max(MIN_VIEW_SPAN)));
    let t = ((anchor - va) / span).clamp(0.0, 1.0);
    let mut new_a = anchor - t * new_span;
    let mut new_b = new_a + new_span;
    if new_a < 0.0 {
        new_a = 0.0;
        new_b = new_span;
    }
    if new_b > f64::from(scene.song_bars) {
        new_b = f64::from(scene.song_bars);
        new_a = new_b - new_span;
    }
    scene.view_a = new_a as f32;
    scene.view_b = new_b as f32;
}

fn body_top() -> f64 {
    f64::from(OVER_H + 1.0 + RULER_H + LOC_H + 1.0)
}

fn body_bottom(scene: &TimelineScene) -> f64 {
    body_top() + f64::from(scene.bands.len() as f32 * ROW + empty_real_guide_rows(scene))
}

fn band_index_at_ly(scene: &TimelineScene, ly: f64) -> Option<usize> {
    let top = body_top();
    if ly < top {
        return None;
    }
    let index = ((ly - top) / f64::from(ROW)) as usize;
    if index < scene.bands.len() {
        Some(index)
    } else {
        None
    }
}

fn reparent_destination_band(
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

fn neighbors(scene: &TimelineScene, band: usize, clip_idx: usize) -> (f32, f32) {
    let clips = &scene.bands[band].clips;
    let prev_b = if clip_idx == 0 {
        0.0
    } else {
        clips[clip_idx - 1].b
    };
    let next_a = if clip_idx + 1 >= clips.len() {
        scene.song_bars
    } else {
        clips[clip_idx + 1].a
    };
    (prev_b, next_a)
}

fn clear_all_key_selection(scene: &mut TimelineScene) {
    for band in &mut scene.bands {
        for clip in &mut band.clips {
            for key in &mut clip.keys {
                key.2 = false;
            }
        }
    }
}

/// revision再投影後に旧key_idの選択を復元する。一致なしなら全解除のまま。
pub(crate) fn restore_key_selection(
    scene: &mut TimelineScene,
    primary_layer_id: &str,
    key_id: u64,
) -> bool {
    if primary_layer_id.is_empty() {
        return false;
    }
    clear_all_key_selection(scene);
    if key_id == 0 {
        return false;
    }
    for band in &mut scene.bands {
        for clip in &mut band.clips {
            if clip.layer_id != primary_layer_id {
                continue;
            }
            for key in &mut clip.keys {
                if key.3 == key_id {
                    key.2 = true;
                    return true;
                }
            }
        }
    }
    false
}

/// real行で選択中のposition key。(layer_id, key_id)
pub(crate) fn selected_real_key(scene: &TimelineScene) -> Option<(String, u64)> {
    if !scene.real {
        return None;
    }
    for band in &scene.bands {
        for clip in &band.clips {
            if clip.layer_id.is_empty() {
                continue;
            }
            for key in &clip.keys {
                if key.2 && key.3 != 0 {
                    return Some((clip.layer_id.clone(), key.3));
                }
            }
        }
    }
    None
}

/// Delete/Backspace: 選択中real keyがあれば remove_position_key。HitKindは増やさない。
pub(crate) fn remove_position_key_commit(scene: &TimelineScene) -> Option<TimelineEditCommit> {
    let (layer_id, key_id) = selected_real_key(scene)?;
    Some(TimelineEditCommit::RemovePositionKey { layer_id, key_id })
}

fn snap_threshold_bars(scene: &TimelineScene) -> f64 {
    let span = f64::from(scene.view_b - scene.view_a);
    SNAP_THRESHOLD_LOGICAL_PX / surface_width() * span
}

/// 短いbarは左右を推測分割せず、既存body moveへ縮退させる。
fn trim_edge_width(clip_width: f64, clip_height: f64) -> Option<f64> {
    if !clip_width.is_finite()
        || !clip_height.is_finite()
        || clip_width < TRIM_EDGE_MIN_CLIP_W
        || clip_height < TRIM_EDGE_MIN_CLIP_H
    {
        return None;
    }
    Some(TRIM_EDGE_MAX_PX.min(clip_width / 4.0))
}

fn min_clip_units(scene: &TimelineScene) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return MIN_CLIP_BARS;
    }
    scene.fps_den as f32 / scene.fps_num as f32
}

fn snap_to_frame(scene: &TimelineScene, raw: f32) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return raw.round();
    }
    let frame = (f64::from(raw) * scene.fps_num as f64 / scene.fps_den as f64).round();
    (frame * scene.fps_den as f64 / scene.fps_num as f64) as f32
}

/// フレーム格子・同band他clip端・playheadへ、画面6論理px閾値で吸着。Cmd中は無効。
fn snap_bar(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> f32 {
    snap_bar_with_guide(scene, playhead, band, exclude_clip, raw, modifiers).0
}

fn snap_bar_with_guide(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> (f32, Option<f32>) {
    if modifiers & 1 != 0 {
        return (raw, None);
    }
    let threshold = snap_threshold_bars(scene) as f32;
    let mut best_dist = threshold;
    let mut best = raw;
    let mut consider = |candidate: f32| {
        let dist = (candidate - raw).abs();
        if dist <= best_dist {
            best_dist = dist;
            best = candidate;
        }
    };
    consider(snap_to_frame(scene, raw));
    if let Some(band_ref) = scene.bands.get(band) {
        for (idx, clip) in band_ref.clips.iter().enumerate() {
            if Some(idx) == exclude_clip {
                continue;
            }
            consider(clip.a);
            consider(clip.b);
        }
    }
    let playhead_bar = (playhead.clamp(0.0, 1.0) as f32) * scene.song_bars;
    consider(playhead_bar);
    if (best - raw).abs() <= threshold {
        (best, Some(best))
    } else {
        (raw, None)
    }
}

/// test用: snap候補の閾値判定を直接検証する。
#[cfg(test)]
pub(crate) fn test_snap_bar(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> f32 {
    snap_bar(scene, playhead, band, exclude_clip, raw, modifiers)
}

/// test用: 先頭real keyのsel flagを立てる。
#[cfg(test)]
pub(crate) fn test_select_first_real_key(scene: &mut TimelineScene) {
    if let Some(key) = scene
        .bands
        .get_mut(0)
        .and_then(|b| b.clips.get_mut(0))
        .and_then(|c| c.keys.get_mut(0))
    {
        key.2 = true;
    }
}

fn hit_gesture(scene: &TimelineScene, playhead: f64, lx: f64, ly: f64) -> Option<HitKind> {
    // 0. overview帯
    if ly < f64::from(OVER_H) && lx >= f64::from(SURF_X) {
        return Some(HitKind::Overview);
    }

    // 1. playhead(頭+線) — ruler/key/trim/clipより先
    if playhead_hit(scene, playhead, lx, ly) {
        return Some(HitKind::Playhead);
    }

    let ry = f64::from(OVER_H + 1.0);
    let time_y0 = body_bottom(scene);
    // 2. ruler帯
    if ((ly >= ry && ly < ry + f64::from(RULER_H))
        || (ly >= time_y0 && ly < time_y0 + f64::from(TIME_H)))
        && lx >= f64::from(SURF_X)
    {
        return Some(HitKind::Scrub);
    }

    let top = body_top();
    let bottom = body_bottom(scene);
    if ly >= top && ly < bottom {
        let band_index = ((ly - top) / f64::from(ROW)) as usize;
        if band_index < scene.bands.len() {
            let cy = top + (band_index as f64 + 0.5) * f64::from(ROW) - 0.5;
            if tog_hit(lx, ly, f64::from(INBOX_W) + 5.0, cy) {
                return Some(HitKind::Mute { band: band_index });
            }
            if tog_hit(lx, ly, f64::from(INBOX_W) + 21.0, cy) {
                return Some(HitKind::Solo { band: band_index });
            }
        }
    }
    if ly < top || ly >= bottom || lx < f64::from(SURF_X) {
        return None;
    }

    let band_index = ((ly - top) / f64::from(ROW)) as usize;
    if band_index >= scene.bands.len() {
        // 空realのガイド帯など、band無しのbodyはEmptyBar扱いにしない(選択解除のみ)。
        return if scene.real && scene.bands.is_empty() {
            Some(HitKind::EmptyBar)
        } else {
            None
        };
    }
    let band = &scene.bands[band_index];
    let row_cy = top + (band_index as f64 + 0.5) * f64::from(ROW) - 0.5;
    let bar = bar_at_lx(scene, lx);

    let mut flat_before = 0i32;
    for (bi, b) in scene.bands.iter().enumerate() {
        if bi == band_index {
            break;
        }
        flat_before += b.clips.len() as i32;
    }

    // 3. key中心±KEY_HIT_PX
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        for (key_idx, (kt, _, _, _)) in clip.keys.iter().enumerate() {
            if *kt < clip.a || *kt > clip.b {
                continue;
            }
            let kx = f64::from(bx(scene, *kt));
            if (lx - kx).abs() <= KEY_HIT_PX && (ly - row_cy).abs() <= KEY_HIT_PX {
                return Some(HitKind::Key {
                    band: band_index,
                    clip_idx,
                    key_idx,
                    flat: flat_before + clip_idx as i32,
                });
            }
        }
    }

    // 4. trim端 / 5. clip本体
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        let ax = f64::from(bx(scene, clip.a));
        let bx_ = f64::from(bx(scene, clip.b));
        if lx >= ax && lx <= bx_ {
            let clip_width = bx_ - ax;
            if let Some(edge_width) = trim_edge_width(clip_width, f64::from(ROW - 1.0)) {
                if lx - ax <= edge_width {
                    return Some(HitKind::TrimStart {
                        band: band_index,
                        clip_idx,
                    });
                }
                if bx_ - lx <= edge_width {
                    return Some(HitKind::TrimEnd {
                        band: band_index,
                        clip_idx,
                    });
                }
            }
            if bar >= f64::from(clip.a) && bar < f64::from(clip.b) {
                return Some(HitKind::Clip {
                    band: band_index,
                    clip_idx,
                    flat: flat_before + clip_idx as i32,
                });
            }
        }
    }

    // 6. 空き面
    Some(HitKind::EmptyBar)
}

fn playhead_hit(scene: &TimelineScene, playhead: f64, lx: f64, ly: f64) -> bool {
    if lx < f64::from(SURF_X) {
        return false;
    }
    let bar = (playhead.clamp(0.0, 1.0) as f32) * scene.song_bars;
    if bar < scene.view_a || bar > scene.view_b {
        return false;
    }
    let px = f64::from(bx(scene, bar));
    if (lx - px).abs() > PLAYHEAD_HIT_PX {
        return false;
    }
    let ry = f64::from(OVER_H + 1.0);
    let tri_top = ry + f64::from(RULER_H) - 6.0;
    let time_y0 = body_bottom(scene);
    let line_bottom = time_y0; // 描画線は分秒ruler上端まで
    ly >= tri_top && ly < line_bottom.max(tri_top + 1.0)
}

/// hover位置のhit種。gesture判定と同じ優先順位をpureに返す。
pub(crate) fn timeline_hover_hit(
    scene: &TimelineScene,
    playhead: f64,
    width: u32,
    height: u32,
    x: f64,
    y: f64,
) -> TimelineHoverHit {
    if width == 0 || height == 0 {
        return TimelineHoverHit::None;
    }
    let scale = f64::from(scale_for(width));
    let lx = x / scale;
    let ly = y / scale;
    match hit_gesture(scene, playhead, lx, ly) {
        Some(HitKind::Playhead) | Some(HitKind::Scrub) => TimelineHoverHit::PlayheadOrRuler,
        Some(HitKind::Key { .. }) => TimelineHoverHit::Key,
        Some(HitKind::TrimStart { .. }) | Some(HitKind::TrimEnd { .. }) => TimelineHoverHit::Trim,
        Some(HitKind::Clip { .. }) => TimelineHoverHit::Clip,
        Some(HitKind::Mute { .. }) | Some(HitKind::Solo { .. }) => TimelineHoverHit::Clip,
        Some(HitKind::Overview) | Some(HitKind::EmptyBar) | None => TimelineHoverHit::None,
    }
}

fn frame_duration_secs(scene: &TimelineScene) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return 1.0;
    }
    scene.fps_den as f32 / scene.fps_num as f32
}

/// ラベル間隔。frame の整数倍。zoom で 1 frame〜数秒。
fn ruler_label_step_secs(scene: &TimelineScene, surface_w: f32) -> f32 {
    let span = (scene.view_b - scene.view_a).max(1e-3);
    let secs_per_px = span / surface_w.max(1.0);
    let min_secs = secs_per_px * 48.0;
    let frame = frame_duration_secs(scene);
    let min_frames = (min_secs / frame).ceil().max(1.0);
    const NICE: [f32; 11] = [
        1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0,
    ];
    let frames = NICE
        .iter()
        .copied()
        .find(|n| *n >= min_frames)
        .unwrap_or(min_frames);
    frames * frame
}

fn first_tick_secs(view_a: f32, step: f32) -> f32 {
    let step = step.max(1e-6);
    (view_a / step).ceil() * step
}

/// 旧整数秒目盛。test互換。
fn first_absolute_tick(view_a: f32, step: i32) -> i32 {
    first_tick_secs(view_a, step.max(1) as f32).round() as i32
}

fn format_ruler_time(secs: f32, scene: &TimelineScene, with_frames: bool) -> String {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        let s = secs.max(0.0).round() as i32;
        return format!("{}:{:02}", s / 60, s % 60);
    }
    let frame =
        (f64::from(secs.max(0.0)) * scene.fps_num as f64 / scene.fps_den as f64).round() as i64;
    let fps = (scene.fps_num as f64 / scene.fps_den as f64)
        .round()
        .max(1.0) as i64;
    let ff = frame.rem_euclid(fps);
    let total_s = frame.div_euclid(fps);
    let m = total_s / 60;
    let s = total_s % 60;
    if with_frames {
        format!("{m}:{s:02}:{ff:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn empty_real_guide_rows(scene: &TimelineScene) -> f32 {
    if scene.real && scene.bands.is_empty() {
        ROW
    } else {
        0.0
    }
}

/// 描画に使う論理座標系の高さ。
fn logical_height(scene: &TimelineScene) -> f32 {
    OVER_H
        + RULER_H
        + LOC_H
        + 1.0
        + scene.bands.len() as f32 * ROW
        + empty_real_guide_rows(scene)
        + TIME_H
        + 2.0
}

/// surface幅へ合わせたscale。probeの静止画は幅1240固定なので、幅で合わせる。
fn scale_for(width: u32) -> f32 {
    (width as f32 / W).max(0.05)
}

/// 帯とclipを平坦化した時の総clip数。
fn clip_count(scene: &TimelineScene) -> usize {
    scene.bands.iter().map(|band| band.clips.len()).sum()
}

fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn argb(a: u8, v: u32) -> Color {
    Color::from_argb(
        a,
        (v >> 16) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}

fn gray(v: u8) -> Color {
    Color::from_rgb(v, v, v)
}

fn dim_bg(c: u32, t: f32) -> u32 {
    let m = |sh: u32| {
        let a = ((c >> sh) & 0xff) as f32;
        let b = ((DESKTOP >> sh) & 0xff) as f32;
        (a + (b - a) * t) as u32
    };
    (m(16) << 16) | (m(8) << 8) | m(0)
}

fn fill(cv: &skia_safe::Canvas, r: Rect, c: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_color(c);
    cv.draw_rect(r, &p);
}

thread_local! {
    static TIMELINE_TYPEFACE: skia_safe::Typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system typeface");
}

fn tf() -> skia_safe::Typeface {
    TIMELINE_TYPEFACE.with(Clone::clone)
}

fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, sz: f32, c: Color) {
    let f = Font::new(tf(), sz);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &f, &p);
}

fn measure(s: &str, sz: f32) -> f32 {
    Font::new(tf(), sz).measure_str(s, None).0
}

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, d: f32, sel: bool) {
    let f = gray(if d.abs() < 0.01 {
        0x2a
    } else if d.abs() < 0.2 {
        0x8c
    } else {
        0xf2
    });
    let path = |s: f32| {
        let mut b = PathBuilder::new();
        b.move_to((cx, cy - s));
        b.line_to((cx + s, cy));
        b.line_to((cx, cy + s));
        b.line_to((cx - s, cy));
        b.close();
        b.detach()
    };
    let mut p = Paint::default();
    p.set_anti_alias(true);
    if sel {
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(1.4);
        p.set_color(Color::WHITE);
        cv.draw_path(&path(7.6), &p);
    }
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(argb(0x4a, 0xffffff));
    cv.draw_path(&path(5.6), &p);
    p.set_style(PaintStyle::Fill);
    p.set_color(f);
    cv.draw_path(&path(4.2), &p);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.6);
    p.set_color(rgb(0x16181c));
    cv.draw_path(&path(4.9), &p);
}

fn glyph(cv: &skia_safe::Canvas, cx: f32, cy: f32, kind: &str, on_dark: bool, quiet: bool) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    let warn = kind == "missing";
    p.set_color(if warn {
        argb(0xcc, 0xc4552e)
    } else {
        argb(if on_dark { 0x40 } else { 0x2e }, 0x000000)
    });
    cv.draw_circle((cx, cy), 5.5, &p);
    let c = if warn {
        gray(0xf4)
    } else if on_dark {
        gray(0xc8)
    } else {
        argb(if quiet { 0x70 } else { 0xd8 }, ON_BAR)
    };
    p.set_color(c);
    match kind {
        "opacity" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.0, cy - 3.0, 6.0, 6.0), &p);
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 3.0, cy - 3.0, 3.0, 6.0), &p);
        }
        "blend" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.4, cy - 3.4, 4.6, 4.6), &p);
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 1.2, cy - 1.2, 4.6, 4.6), &p);
        }
        "retime" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.2);
            for o in [-2.6f32, 0.6] {
                let mut b = PathBuilder::new();
                b.move_to((cx + o, cy - 3.0));
                b.line_to((cx + o + 2.4, cy));
                b.line_to((cx + o, cy + 3.0));
                cv.draw_path(&b.detach(), &p);
            }
        }
        "bypass" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.2);
            cv.draw_circle((cx, cy), 3.2, &p);
            cv.draw_line((cx - 2.6, cy + 2.6), (cx + 2.6, cy - 2.6), &p);
        }
        "muted" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.2, cy - 3.2, 6.4, 6.4), &p);
            for i in 0..3 {
                let o = -2.4 + i as f32 * 2.4;
                cv.draw_line((cx + o, cy + 3.2), (cx + o + 3.2, cy - 3.2), &p);
            }
        }
        _ => {
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 0.8, cy - 3.4, 1.6, 4.4), &p);
            cv.draw_rect(Rect::from_xywh(cx - 0.8, cy + 2.0, 1.6, 1.6), &p);
        }
    }
}

fn tog_hit(lx: f64, ly: f64, x: f64, cy: f64) -> bool {
    lx >= x && lx < x + 14.0 && ly >= cy - 6.5 && ly < cy + 6.5
}

fn tog(cv: &skia_safe::Canvas, x: f32, cy: f32, l: &str, on: bool, mixed: bool, acc: u32) {
    let r = Rect::from_xywh(x, cy - 6.5, 14.0, 13.0);
    fill(cv, r, if on { rgb(acc) } else { rgb(FILL_HANDLE) });
    if mixed {
        fill(
            cv,
            Rect::from_ltrb(r.left, r.top, r.left + 7.0, r.bottom),
            rgb(acc),
        );
    }
    fill(
        cv,
        Rect::from_ltrb(r.left, r.top, r.right, r.top + 1.0),
        gray(0x6e),
    );
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(rgb(0x3a3a3a));
    cv.draw_rect(r, &p);
    let w = measure(l, 8.5);
    text(
        cv,
        l,
        r.left + (14.0 - w) / 2.0,
        cy + 3.2,
        8.5,
        if on || mixed {
            rgb(0x0d0d0d)
        } else {
            gray(0xc4)
        },
    );
}

/// timeline 1枚をRGBA8888 premulのbytesへ描く。
///
/// `playhead`は0..1で曲全体(0..song_bars)を走る。
/// `selected < 0`なら選択ringなし。非負は平坦化clip列のindex。
pub(crate) fn draw_timeline(
    scene: &TimelineScene,
    bytes: &mut [u8],
    width: u32,
    height: u32,
    playhead: f64,
    selected: i32,
) {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let Some(mut surface) = surfaces::wrap_pixels(&info, bytes, Some(width as usize * 4), None)
    else {
        return;
    };
    let cv = surface.canvas();
    cv.clear(rgb(DESKTOP));

    let scale = scale_for(width);
    cv.save();
    cv.scale((scale, scale));

    // 論理座標系。以下は motolii_full.rs と同一。
    let h = logical_height(scene).max(height as f32 / scale);
    let sw = W - SURF_X - 6.0;
    let view_a = scene.view_a;
    let view_b = scene.view_b;
    let song_bars = scene.song_bars;
    let bx = |b: f32| SURF_X + (b - view_a) / (view_b - view_a) * sw;
    let ox = |b: f32| SURF_X + b / song_bars * sw;

    // 0 clip時に rem_euclid(·,1) で幽霊番号を作らない。
    let count = clip_count(scene);
    let selected = if selected < 0 || count == 0 {
        None
    } else {
        Some(selected.rem_euclid(count as i32) as usize)
    };

    fill(cv, Rect::from_ltrb(0.0, 0.0, SURF_X, h), rgb(SURFACE_BG));

    // ── Overview ──
    fill(cv, Rect::from_ltrb(SURF_X, 0.0, W, OVER_H), rgb(SURFACE_LO));
    text(cv, "overview", 8.0, 14.0, 9.0, rgb(DIM));
    for (i, band) in scene.bands.iter().enumerate() {
        let yy = 3.0 + i as f32 * 2.8;
        for c in &band.clips {
            fill(
                cv,
                Rect::from_ltrb(ox(c.a), yy, ox(c.b), yy + 2.2),
                argb(0xcc, P[c.slot]),
            );
        }
    }
    let (va, vb) = (ox(view_a), ox(view_b));
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(gray(0xd8));
    cv.draw_rect(Rect::from_ltrb(va, 1.0, vb, OVER_H - 1.0), &p);
    fill(
        cv,
        Rect::from_ltrb(0.0, OVER_H, W, OVER_H + 1.0),
        rgb(CONTRAST),
    );

    // ── frame 格子 / timecode ruler ──
    let ry = OVER_H + 1.0;
    fill(
        cv,
        Rect::from_ltrb(0.0, ry, W, ry + RULER_H),
        rgb(SURFACE_HI),
    );
    let label_step = ruler_label_step_secs(scene, sw);
    let with_frames = label_step + 1e-6 < 1.0;
    let mut tick = first_tick_secs(view_a, label_step);
    for _ in 0..512 {
        if tick > view_b + 1e-3 {
            break;
        }
        let x = bx(tick);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H - 6.0, x + 1.0, ry + RULER_H),
            gray(0x6a),
        );
        text(
            cv,
            &format_ruler_time(tick, scene, with_frames),
            x + 3.0,
            ry + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
        tick += label_step;
        if !tick.is_finite() {
            break;
        }
    }
    // probe fixture の verse1–chorus 帯。Document 投影へは載せない。
    if !scene.real {
        fill(
            cv,
            Rect::from_ltrb(bx(8.0), ry, bx(24.0), ry + 3.0),
            rgb(RULER_MARK),
        );
    }

    // ── locator lane ──
    let ly = ry + RULER_H;
    fill(cv, Rect::from_ltrb(0.0, ly, W, ly + LOC_H), rgb(SURFACE_BG));
    for &(b, name) in &scene.locators {
        let x = bx(b);
        fill(cv, Rect::from_ltrb(x, ly, x + 1.0, ly + LOC_H), gray(0x8a));
        let mut t = PathBuilder::new();
        t.move_to((x + 1.0, ly + 2.0));
        t.line_to((x + 6.0, ly + 5.0));
        t.line_to((x + 1.0, ly + 8.0));
        t.close();
        let mut pp = Paint::default();
        pp.set_anti_alias(true);
        pp.set_color(gray(0x8a));
        cv.draw_path(&t.detach(), &pp);
        text(cv, name, x + 9.0, ly + 11.0, 8.5, gray(0xa8));
    }
    fill(
        cv,
        Rect::from_ltrb(0.0, ly + LOC_H, W, ly + LOC_H + 1.0),
        rgb(CONTRAST),
    );

    // ── Inbox ──
    text(cv, "Inbox", 9.0, ry + 12.0, 9.5, gray(0xc0));
    // probe dummyはfixture専用。realはhost layer以外をInboxへ描かない。
    if !scene.real {
        text(cv, "3", INBOX_W - 15.0, ry + 12.0, 9.0, rgb(DIM));
        let by0 = ly + LOC_H + 1.0;
        for (i, s) in ["street_loop.mp4", "check cut 1:04", "proxy 2 files"]
            .iter()
            .enumerate()
        {
            let y = by0 + 6.0 + i as f32 * ROW;
            let mut pp = Paint::default();
            pp.set_anti_alias(true);
            pp.set_color(rgb(FILL_HANDLE));
            match i {
                0 => {
                    cv.draw_rect(Rect::from_xywh(9.0, y, 7.0, 7.0), &pp);
                }
                1 => {
                    let mut d = PathBuilder::new();
                    d.move_to((12.5, y - 1.0));
                    d.line_to((16.5, y + 3.5));
                    d.line_to((12.5, y + 8.0));
                    d.line_to((8.5, y + 3.5));
                    d.close();
                    cv.draw_path(&d.detach(), &pp);
                }
                _ => {
                    cv.draw_circle((12.5, y + 3.5), 3.6, &pp);
                }
            }
            text(cv, s, 22.0, y + 7.0, 9.0, gray(0xb0));
            fill(
                cv,
                Rect::from_ltrb(0.0, y + ROW - 6.0, INBOX_W, y + ROW - 5.0),
                argb(0x40, 0x000000),
            );
        }
    }
    let by0 = ly + LOC_H + 1.0;

    // ── bands ──
    let mut y = by0;
    let mut flat = 0usize;
    for (band_index, band) in scene.bands.iter().enumerate() {
        if scene.lane_preview_band == Some(band_index) {
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y, W, y + ROW - 1.0),
                argb(0x24, ACCENT),
            );
        }
        for b in view_a as i32..=view_b as i32 {
            let x = bx(b as f32);
            // realは秒格子。fixtureだけ旧4拍強調を残す。
            let major = scene.real || b % 4 == 0;
            fill(
                cv,
                Rect::from_ltrb(x, y, x + 1.0, y + ROW - 1.0),
                argb(if major { 0x54 } else { 0x14 }, 0x060606),
            );
        }
        fill(
            cv,
            Rect::from_ltrb(0.0, y + ROW - 1.0, W, y + ROW),
            rgb(CONTRAST),
        );
        let cy = y + (ROW - 1.0) / 2.0;

        tog(cv, INBOX_W + 5.0, cy, "M", band.mute, band.mixed, 0xd8d8d8);
        tog(cv, INBOX_W + 21.0, cy, "S", band.solo, false, ACCENT);
        if !band.clips.is_empty() {
            text(
                cv,
                &format!("{}", band.clips.len()),
                INBOX_W + 41.0,
                cy + 3.2,
                8.5,
                gray(0x8e),
            );
        }
        let mut roll: Vec<&str> = vec![];
        for c in &band.clips {
            for d in c.dev {
                if !roll.contains(d) {
                    roll.push(d);
                }
            }
            if c.mute && !roll.contains(&"muted") {
                roll.push("muted");
            }
            if c.fx.1 > 0 && !roll.contains(&"bypass") {
                roll.push("bypass");
            }
        }
        let mut rx = INBOX_W + RAIL_W - 5.0;
        for g in roll.iter().rev() {
            rx -= 13.0;
            glyph(cv, rx + 5.5, cy, g, true, false);
        }

        cv.save();
        cv.clip_rect(Rect::from_ltrb(SURF_X, y, W, y + ROW), None, false);
        for c in &band.clips {
            let is_selected = selected == Some(flat);
            flat += 1;
            let quiet = c.mute || band.mute;
            let r = Rect::from_ltrb(bx(c.a), y, bx(c.b), y + ROW - 1.0);
            fill(
                cv,
                r,
                rgb(if quiet {
                    dim_bg(P[c.slot], 0.74)
                } else {
                    P[c.slot]
                }),
            );
            if quiet {
                let mut pp = Paint::default();
                pp.set_anti_alias(true);
                pp.set_stroke_width(1.0);
                pp.set_color(argb(0x46, 0x000000));
                let mut i = r.left - ROW;
                while i < r.right {
                    cv.draw_line((i, r.bottom), (i + ROW, r.top), &pp);
                    i += 7.0;
                }
            }
            let ink = if quiet {
                argb(0x82, ON_BAR)
            } else {
                argb(0xff, ON_BAR)
            };
            let mut x = r.right - 4.0;
            let place = |wd: f32, x: &mut f32| -> Option<f32> {
                if *x - wd < r.left + 26.0 {
                    None
                } else {
                    *x -= wd;
                    Some(*x)
                }
            };
            if c.fx.0 > 0 {
                let lbl = if c.fx.1 > 0 {
                    format!("fx {}/{}", c.fx.0 - c.fx.1, c.fx.0)
                } else {
                    format!("fx {}", c.fx.0)
                };
                let wd = measure(&lbl, 8.5) + 8.0;
                if let Some(px) = place(wd, &mut x) {
                    text(
                        cv,
                        &lbl,
                        px + 4.0,
                        cy + 3.2,
                        8.5,
                        argb(
                            if quiet {
                                0x66
                            } else if c.fx.1 > 0 {
                                0xff
                            } else {
                                0xb0
                            },
                            ON_BAR,
                        ),
                    );
                }
            }
            for g in c.dev.iter().rev() {
                if let Some(px) = place(13.0, &mut x) {
                    glyph(cv, px + 5.5, cy, g, false, quiet);
                }
            }
            let nw = measure(&c.name, 8.5);
            let mut nx = r.left + 5.0;
            for (b, _, _, _) in &c.keys {
                if *b < c.a || *b > c.b {
                    continue;
                }
                let kx = bx(*b);
                if kx > nx - 8.0 && kx < nx + nw + 6.0 {
                    nx = kx + 9.0;
                }
            }
            if x - nx > nw * 0.55 {
                text(cv, &c.name, nx, cy + 3.2, 8.5, ink);
            }
            for (b, d, s, _) in &c.keys {
                if *b < c.a || *b > c.b {
                    continue;
                }
                diamond(cv, bx(*b), cy, *d, *s);
            }
            if is_selected {
                let mut pp = Paint::default();
                pp.set_anti_alias(false);
                pp.set_style(PaintStyle::Stroke);
                pp.set_stroke_width(1.0);
                pp.set_color(Color::WHITE);
                cv.draw_rect(
                    Rect::from_ltrb(r.left + 0.5, r.top + 0.5, r.right - 0.5, r.bottom - 0.5),
                    &pp,
                );
            }
        }
        cv.restore();
        if scene.lane_preview_band == Some(band_index) {
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y, W, y + 1.0),
                argb(0xd0, ACCENT),
            );
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y + ROW - 2.0, W, y + ROW - 1.0),
                argb(0xd0, ACCENT),
            );
        }
        y += ROW;
    }

    // 空real: 一行ガイド(fixtureはbands非空のため描画不変)。
    if scene.real && scene.bands.is_empty() {
        let guide = "Create の □ Rectangle をダブルクリックで配置";
        let text_w = measure(guide, 9.0);
        let tx = SURF_X + ((sw - text_w) * 0.5).max(0.0);
        text(cv, guide, tx, y + ROW * 0.5 + 3.0, 9.0, rgb(DIM));
        y += ROW;
    }

    // ── 下段 timecode（上段の2倍粗さ、同じ frame 格子）──
    fill(cv, Rect::from_ltrb(0.0, y, W, y + TIME_H), rgb(SURFACE_BG));
    let coarse_step = label_step * 2.0;
    let mut tick = first_tick_secs(view_a, coarse_step);
    for _ in 0..512 {
        if tick > view_b + 1e-3 {
            break;
        }
        let x = bx(tick);
        fill(cv, Rect::from_ltrb(x, y, x + 1.0, y + 5.0), gray(0x6a));
        text(
            cv,
            &format_ruler_time(tick, scene, false),
            x + 3.0,
            y + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
        tick += coarse_step;
        if !tick.is_finite() {
            break;
        }
    }
    if scene.real {
        text(
            cv,
            &format_ruler_time(song_bars, scene, false),
            W - 64.0,
            y + 11.0,
            8.5,
            rgb(DIM),
        );
    } else {
        text(cv, "3:12 total", W - 64.0, y + 11.0, 8.5, rgb(DIM));
    }

    fill(
        cv,
        Rect::from_ltrb(SURF_X - 1.0, 0.0, SURF_X, h),
        rgb(CONTRAST),
    );
    fill(
        cv,
        Rect::from_ltrb(INBOX_W, OVER_H, INBOX_W + 1.0, h),
        rgb(CONTRAST),
    );

    // gesture中のsnap位置だけを表示し、release後のDocument投影には残さない。
    if let Some(guide) = scene
        .snap_guide
        .filter(|guide| *guide >= view_a && *guide <= view_b)
    {
        let x = bx(guide);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H, x + 1.0, y),
            argb(0xd8, ACCENT),
        );
    }

    // ── playhead。曲基準0..1。表示範囲外は描かない ──
    let bar = (playhead.clamp(0.0, 1.0) as f32) * song_bars;
    if bar >= view_a && bar <= view_b {
        let px = bx(bar);
        fill(
            cv,
            Rect::from_ltrb(px, ry + RULER_H, px + 1.0, y),
            gray(0xe7),
        );
        let mut tri = PathBuilder::new();
        tri.move_to((px - 4.0, ry + RULER_H - 6.0));
        tri.line_to((px + 5.0, ry + RULER_H - 6.0));
        tri.line_to((px + 0.5, ry + RULER_H));
        tri.close();
        let mut pp = Paint::default();
        pp.set_anti_alias(true);
        pp.set_color(gray(0xe7));
        cv.draw_path(&tri.detach(), &pp);
    }

    cv.restore();
}

/// physical pointerを(平坦化clip index, 表示範囲内の正規化時間)へ写す。
///
/// 時間面の外(Inbox / rail / ruler)は`None`。
/// clipに当たらなければ`(-1, time)`を返し、呼び出し側でselection解除に使える。
pub(crate) fn hit_test(
    scene: &TimelineScene,
    width: u32,
    height: u32,
    x: f64,
    y: f64,
) -> Option<(i32, f64)> {
    if width == 0 || height == 0 {
        return None;
    }
    let scale = f64::from(scale_for(width));
    let lx = x / scale;
    let ly = y / scale;
    if lx < f64::from(SURF_X) {
        return None;
    }
    let top = body_top();
    let bottom = body_bottom(scene);
    if ly < top || ly >= bottom {
        return None;
    }
    let bar = bar_at_lx(scene, lx);
    let time = time_at_lx(scene, lx);
    let band_index = ((ly - top) / f64::from(ROW)) as usize;

    let mut flat = 0i32;
    for (index, band) in scene.bands.iter().enumerate() {
        for c in &band.clips {
            if index == band_index && bar >= f64::from(c.a) && bar < f64::from(c.b) {
                return Some((flat, time));
            }
            flat += 1;
        }
    }
    Some((-1, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> (TimelineSession, i32, f64) {
        (TimelineSession::default(), 1, 0.27)
    }

    fn lx_for_bar(bar: f64) -> f64 {
        lx_for_bar_in(&TimelineScene::default(), bar)
    }

    fn lx_for_bar_in(scene: &TimelineScene, bar: f64) -> f64 {
        f64::from(SURF_X)
            + (bar - f64::from(scene.view_a)) / f64::from(scene.view_b - scene.view_a)
                * surface_width()
    }

    fn phys(lx: f64, ly: f64) -> (f64, f64) {
        (lx, ly)
    }

    fn bx_default(b: f32) -> f32 {
        bx(&TimelineScene::default(), b)
    }

    #[test]
    fn hit_test_returns_the_clip_under_the_pointer() {
        // 幅1240 → scale 1.0。band 0 の2本目 sky_plate は bar 14..27。
        let scene = TimelineScene::default();
        let scale = scale_for(1240);
        assert!((scale - 1.0).abs() < f32::EPSILON);
        let x = lx_for_bar(20.0);
        let y = body_top() + 5.0;
        assert_eq!(hit_test(&scene, 1240, 400, x, y).map(|hit| hit.0), Some(1));
    }

    #[test]
    fn hit_test_rejects_the_header_columns_and_the_rulers() {
        let scene = TimelineScene::default();
        assert_eq!(hit_test(&scene, 1240, 400, 10.0, 100.0), None);
        assert_eq!(hit_test(&scene, 1240, 400, 600.0, 5.0), None);
    }

    #[test]
    fn drag_move_left_of_first_clip_is_clamped_to_zero_left_bound() {
        let (mut sess, mut selected, mut playhead) = session();
        let press_x = lx_for_bar(10.0);
        let y = body_top() + 5.0; // band0 の clip0
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            press_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(-20.0),
            y,
            0,
        );
        let clip = &sess.scene.bands[0].clips[0];
        assert!((clip.a - 0.0).abs() < 1e-4);
        assert!((clip.b - 14.0).abs() < 1e-4);
    }

    #[test]
    fn drag_move_of_band0_clip1_is_clamped_to_prev_neighbor_start() {
        let (mut sess, mut selected, mut playhead) = session();
        let press_x = lx_for_bar(20.0);
        let y = body_top() + 5.0; // band0 の clip1
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            press_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(1.0),
            y,
            0,
        );
        let clip = &sess.scene.bands[0].clips[1];
        assert!((clip.a - 14.0).abs() < 1e-4);
        assert!((clip.b - 27.0).abs() < 1e-4);
    }

    #[test]
    fn trim_cannot_cross_neighbor_clip_edges() {
        let (mut sess, mut selected, mut playhead) = session();
        let y = body_top() + 5.0; // band0

        let clip0_end_x = f64::from(bx_default(14.0)) + 1.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            clip0_end_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(20.0),
            y,
            0,
        );
        let clip0 = &sess.scene.bands[0].clips[0];
        assert!((clip0.a - 0.0).abs() < 1e-4);
        assert!((clip0.b - 14.0).abs() < 1e-4);

        let clip1_start_x = f64::from(bx_default(14.0)) + 1.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            clip1_start_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(1.0),
            y,
            0,
        );
        let clip1 = &sess.scene.bands[0].clips[1];
        assert!((clip1.a - 14.0).abs() < 1e-4);
        assert!((clip1.b - 27.0).abs() < 1e-4);
    }

    #[test]
    fn scrub_does_not_fire_on_inbox_column_even_with_ruler_y() {
        let (mut sess, mut selected, mut playhead) = session();
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            50.0,
            f64::from(OVER_H + 1.0) + 4.0,
            0,
        );
        assert!(!out.feedback);
        assert_eq!(selected, 1);
        assert!((playhead - 0.27).abs() < 1e-9);
    }

    #[test]
    fn scrub_follows_time_ruler() {
        let (mut sess, mut selected, mut playhead) = session();
        let y = body_bottom(&TimelineScene::default()) + 4.0;
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar(33.0),
            y,
            0,
        );
        assert!(out.feedback);
        assert!(out.scrub_playhead.is_some());
        assert!(!out.scrub_release);
        assert!((playhead - 33.0 / 96.0).abs() < 1e-9);
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar(33.0),
            y,
            0,
        );
        assert!(up.scrub_release);
        assert!((up.scrub_playhead.unwrap() - 33.0 / 96.0).abs() < 1e-9);
    }

    #[test]
    fn hit_test_reports_no_clip_inside_the_empty_band() {
        // 5番目の帯は空。当たらないので -1。
        let scene = TimelineScene::default();
        let y = body_top() + f64::from(ROW) * 4.5;
        assert_eq!(
            hit_test(&scene, 1240, 400, 600.0, y).map(|hit| hit.0),
            Some(-1)
        );
    }

    #[test]
    fn fixture_trim_down_keeps_selection_playhead_and_feedback() {
        let (mut sess, mut selected, mut playhead) = session();
        sess.scene.bands[0].clips[0].a = 8.0;
        sess.scene.bands[0].clips[0].b = 16.0;
        let orig_selected = selected;
        let orig_playhead = playhead;
        let y = body_top() + 5.0;
        let down_cases = [
            f64::from(bx(&sess.scene, 8.0)) + 1.0,
            f64::from(bx(&sess.scene, 16.0)) - 1.0,
        ];
        for x in down_cases {
            let out = sess.pointer(
                &mut selected,
                &mut playhead,
                1240,
                400,
                TimelinePointerPhase::Down,
                x,
                y,
                0,
            );
            assert!(!out.feedback);
            assert!(!out.dirty);
            assert_eq!(selected, orig_selected);
            assert!((playhead - orig_playhead).abs() < 1e-9);
            assert_eq!(sess.scene.bands[0].clips[0].a, 8.0);
            assert_eq!(sess.scene.bands[0].clips[0].b, 16.0);
        }
    }

    /// 決定的疑似乱数のgesture嵐。panicと不変条件破壊を狩る(seed固定で再現可能)。
    #[test]
    fn deterministic_gesture_storm_holds_invariants() {
        // LCG(固定seed)。Date/外部乱数へ依存しない。
        let mut state: u64 = 0x00C0FFEE_5EED_1234;
        let mut next = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let mut rf = {
            let mut n = next;
            move |lo: f64, hi: f64| lo + (n() as f64 / u32::MAX as f64) * (hi - lo)
        };

        let scenes: Vec<TimelineScene> = vec![
            TimelineScene::default(),
            TimelineScene::from_snapshot_with_song_bars(
                &[
                    SnapshotLayerInput {
                        layer_id: "1".into(),
                        display_name: "a".into(),
                        interval_secs: Some((0.0, 10.0)),
                        keys: vec![
                            SnapshotKeyInput {
                                key_id: 10,
                                time_secs: 2.0,
                            },
                            SnapshotKeyInput {
                                key_id: 11,
                                time_secs: 7.0,
                            },
                        ],
                    },
                    SnapshotLayerInput {
                        layer_id: "2".into(),
                        display_name: "b".into(),
                        interval_secs: Some((1.0, 4.0)),
                        keys: vec![],
                    },
                ],
                Some("1"),
                10.0,
            ),
        ];

        for (w, h) in [(1240u32, 400u32), (2480, 620), (620, 200)] {
            let mut session = TimelineSession::default();
            let mut selected = 1i32;
            let mut playhead = 0.5f64;
            for step in 0..120_000u32 {
                let roll = next() % 100;
                if roll < 80 {
                    let phase = match next() % 4 {
                        0 => TimelinePointerPhase::Down,
                        1 => TimelinePointerPhase::Move,
                        2 => TimelinePointerPhase::Up,
                        _ => TimelinePointerPhase::Cancel,
                    };
                    let x = rf(-200.0, w as f64 + 200.0);
                    let y = rf(-200.0, h as f64 + 200.0);
                    let m = next() % 2;
                    let _ = session.pointer(&mut selected, &mut playhead, w, h, phase, x, y, m);
                } else if roll < 95 {
                    let _ = session.scroll(
                        w,
                        h,
                        rf(-500.0, 500.0),
                        rf(-500.0, 500.0),
                        if next() % 4 == 0 { rf(-0.5, 0.5) } else { 0.0 },
                        next() % 2,
                        rf(0.0, w as f64),
                        rf(0.0, h as f64),
                    );
                } else {
                    // gesture中も含む任意タイミングのscene差し替え(P0-2経路)。
                    session.scene = scenes[(next() % scenes.len() as u32) as usize].clone();
                }

                // 不変条件: viewとplayheadとclip幾何が常に健全。
                let sb = session.scene.song_bars;
                assert!(
                    playhead.is_finite() && (0.0..=1.0).contains(&playhead),
                    "step {step}"
                );
                assert!(session.scene.view_a.is_finite() && session.scene.view_b.is_finite());
                assert!(session.scene.view_a >= -0.001 && session.scene.view_b <= sb + 0.001);
                assert!(session.scene.view_b > session.scene.view_a);
                for band in &session.scene.bands {
                    for clip in &band.clips {
                        assert!(clip.a.is_finite() && clip.b.is_finite());
                        assert!(clip.b >= clip.a, "clip inverted at step {step}");
                        for key in &clip.keys {
                            assert!(key.0.is_finite());
                        }
                    }
                }
            }
        }
    }

    /// `MOTOLII_WRITE_PREVIEW=1 cargo test -- --nocapture` でpanel実寸のPNGを書く。
    /// 見た目の確認用で、判定はしない(probeなのでgoldenを持たない)。
    #[test]
    fn writes_a_panel_sized_preview_when_asked() {
        if std::env::var_os("MOTOLII_WRITE_PREVIEW").is_none() {
            return;
        }
        let (w, h) = (2480u32, 620u32);
        let mut bytes = vec![0u8; (w * h * 4) as usize];
        draw_timeline(&TimelineScene::default(), &mut bytes, w, h, 0.22, 1);
        let info = ImageInfo::new(
            (w as i32, h as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let mut surface =
            surfaces::wrap_pixels(&info, &mut bytes, Some(w as usize * 4), None).unwrap();
        let png = surface
            .image_snapshot()
            .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
            .unwrap();
        std::fs::write("timeline-rn-probe-preview.png", png.as_bytes()).unwrap();
        println!("timeline-rn-probe-preview.png {w}x{h}");
    }

    #[test]
    fn draw_timeline_fills_the_requested_buffer() {
        let (w, h) = (620u32, 200u32);
        let mut bytes = vec![0u8; (w * h * 4) as usize];
        draw_timeline(&TimelineScene::default(), &mut bytes, w, h, 0.5, 1);
        assert!(bytes.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn scrub_follows_ruler_and_cancel_restores_press_playhead() {
        let (mut sess, mut selected, mut playhead) = session();
        let (x0, y0) = phys(lx_for_bar(12.0), f64::from(OVER_H + 1.0) + 4.0);
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x0,
            y0,
            0,
        );
        assert!(out.feedback);
        let pressed = playhead;
        assert!((pressed - 12.0 / 96.0).abs() < 1e-9);

        let (x1, y1) = phys(lx_for_bar(24.0), f64::from(OVER_H + 1.0) + 4.0);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x1,
            y1,
            0,
        );
        assert!((playhead - 24.0 / 96.0).abs() < 1e-9);

        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x1,
            y1,
            0,
        );
        assert!((playhead - 24.0 / 96.0).abs() < 1e-9);

        // 新しいscrubをcancelで戻す
        let (mut sess, mut selected, mut playhead) = session();
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x0,
            y0,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x1,
            y1,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            x1,
            y1,
            0,
        );
        assert!((playhead - 0.27).abs() < 1e-9);
        assert_eq!(selected, 1);
    }

    #[test]
    fn clip_body_down_selects_without_moving_playhead() {
        let (mut sess, mut selected, mut playhead) = session();
        let x = lx_for_bar(8.0);
        let y = body_top() + f64::from(ROW) + 5.0; // band1 hero
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y,
            0,
        );
        assert_eq!(selected, 3); // band0 has 3 clips
        assert!((playhead - 0.27).abs() < 1e-9);
    }

    #[test]
    fn empty_bar_down_clears_selection() {
        let (mut sess, mut selected, mut playhead) = session();
        let y = body_top() + f64::from(ROW) * 4.5;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar(10.0),
            y,
            0,
        );
        assert_eq!(selected, -1);
    }

    #[test]
    fn clip_move_shifts_keys_and_clamps_to_neighbors() {
        let (mut sess, mut selected, mut playhead) = session();
        // band1 clip0 hero 4..22, next starts 26 → max a=8
        let press_x = lx_for_bar(10.0);
        let y = body_top() + f64::from(ROW) + 5.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            press_x,
            y,
            0,
        );
        let move_x = press_x + 10.0; // >3px, ~0.465 bars
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            move_x,
            y,
            0,
        );
        let clip = &sess.scene.bands[1].clips[0];
        let expected_dx =
            (10.0 / surface_width() * f64::from(sess.scene.view_b - sess.scene.view_a)) as f32;
        let snapped = test_snap_bar(&sess.scene, playhead, 1, Some(0), 4.0 + expected_dx, 0);
        assert!((clip.a - snapped).abs() < 1e-4);
        assert!((clip.b - (22.0 + (snapped - 4.0))).abs() < 1e-4);
        assert!((clip.keys[0].0 - (8.0 + (snapped - 4.0))).abs() < 1e-4);

        // 大きく右へ飛ばしてclamp
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            press_x + 500.0,
            y,
            0,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.a - 8.0).abs() < 1e-4); // 26 - 18
        assert!((clip.b - 26.0).abs() < 1e-4);
        assert!((clip.keys[0].0 - 12.0).abs() < 1e-4);
    }

    #[test]
    fn cancel_restores_full_scene_selection_playhead_and_key_state_after_clip_move() {
        let (mut sess, mut selected, mut playhead) = session();
        let key_x = f64::from(bx_default(8.0));
        let key_y = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            key_x,
            key_y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(9.0),
            key_y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar(9.0),
            key_y,
            0,
        );

        let snapshot_scene = sess.scene.clone();
        let snapshot_selected = selected;
        let snapshot_playhead = playhead;

        let press_x = lx_for_bar(10.0);
        let y = body_top() + f64::from(ROW) + 5.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            press_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            press_x - 200.0,
            y,
            0,
        );
        assert!(sess.scene.snap_guide.is_some());

        let cancel = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            press_x - 200.0,
            y,
            0,
        );
        assert!(cancel.edit_commit.is_none());
        assert_eq!(sess.scene, snapshot_scene);
        assert_eq!(selected, snapshot_selected);
        assert!((playhead - snapshot_playhead).abs() < 1e-9);
    }

    #[test]
    fn trim_changes_edges_only_and_respects_min_length() {
        let (mut sess, mut selected, mut playhead) = session();
        // band1 clip0 hero 4..22。右端22にkeyは無い。
        let y = body_top() + f64::from(ROW) + 5.0;
        let end_x = f64::from(bx_default(22.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            end_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(10.0),
            y,
            0,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.a - 4.0).abs() < 1e-4);
        assert!((clip.b - 10.0).abs() < 1e-4);

        // 最小長 1 frame へ押し込み
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(3.0),
            y,
            0,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.b - (4.0 + 1.0 / 30.0)).abs() < 1e-4);

        // TrimStart: band1 clip1 26..40。左端にkeyは無い。keysは不変。
        // playhead既定0.27≈bar25.92がTRIM端に重なるため遠ざける(F1優先)。
        let (mut sess, mut selected, mut playhead) = session();
        playhead = 12.0 / 96.0;
        let keys_before = sess.scene.bands[1].clips[1].keys.clone();
        let start_x = f64::from(bx_default(26.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            start_x,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(28.0),
            y,
            0,
        );
        let clip = &sess.scene.bands[1].clips[1];
        assert!((clip.a - 28.0).abs() < 1e-4);
        assert!((clip.b - 40.0).abs() < 1e-4);
        assert_eq!(clip.keys, keys_before);
    }

    #[test]
    fn trim_hit_uses_inside_handle_width_and_keeps_key_priority() {
        let mut scene = TimelineScene::default();
        let band = 1usize;
        let clip = 0usize;
        let y = body_top() + f64::from(ROW) * 1.5 - 0.5;
        let ax = f64::from(bx(&scene, scene.bands[band].clips[clip].a));

        assert_eq!(
            timeline_hover_hit(&scene, 0.0, 1240, 400, ax + 14.9, y),
            TimelineHoverHit::Trim
        );
        assert_eq!(
            timeline_hover_hit(&scene, 0.0, 1240, 400, ax + 15.1, y),
            TimelineHoverHit::Clip
        );
        assert_eq!(
            timeline_hover_hit(&scene, 0.0, 1240, 400, ax - 0.1, y),
            TimelineHoverHit::None
        );

        let key_time = scene.bands[band].clips[clip].a;
        scene.bands[band].clips[clip]
            .keys
            .push((key_time, 0.42, false, 99));
        assert_eq!(
            timeline_hover_hit(&scene, 0.0, 1240, 400, ax, y),
            TimelineHoverHit::Key
        );
    }

    #[test]
    fn trim_edge_width_obeys_clip_width_and_height_cutoffs() {
        assert_eq!(trim_edge_width(24.9, 19.0), None);
        assert_eq!(trim_edge_width(25.0, 15.9), None);
        assert_eq!(trim_edge_width(25.0, 16.0), Some(6.25));
        assert_eq!(trim_edge_width(59.0, 19.0), Some(14.75));
        assert_eq!(trim_edge_width(60.0, 19.0), Some(15.0));
        assert_eq!(trim_edge_width(120.0, 19.0), Some(15.0));
    }

    #[test]
    fn real_trim_hit_inside_fifteen_px_releases_existing_commit_once() {
        for (left, move_bar) in [(true, 3.0), (false, 7.0)] {
            let mut sess = TimelineSession::default();
            sess.scene = TimelineScene::from_snapshot(
                &[SnapshotLayerInput {
                    layer_id: "trim".into(),
                    display_name: "clip".into(),
                    interval_secs: Some((2.0, 6.0)),
                    keys: vec![],
                }],
                Some("trim"),
            );
            let mut selected = 0;
            let mut playhead = 0.0;
            let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
            let (a, b, _) = sess.scene.clip0_span_and_keys(0).unwrap();
            let edge_x = if left {
                f64::from(bx(&sess.scene, a)) + 10.0
            } else {
                f64::from(bx(&sess.scene, b)) - 10.0
            };
            let move_x = lx_for_bar_in(&sess.scene, move_bar);

            let down = sess.pointer(
                &mut selected,
                &mut playhead,
                1240,
                400,
                TimelinePointerPhase::Down,
                edge_x,
                y,
                0,
            );
            assert!(down.edit_commit.is_none());
            let moved = sess.pointer(
                &mut selected,
                &mut playhead,
                1240,
                400,
                TimelinePointerPhase::Move,
                move_x,
                y,
                0,
            );
            assert!(moved.edit_commit.is_none());
            let up = sess.pointer(
                &mut selected,
                &mut playhead,
                1240,
                400,
                TimelinePointerPhase::Up,
                move_x,
                y,
                0,
            );
            let expected = if left {
                TimelineEditCommit::TrimClipIn {
                    layer_id: "trim".into(),
                    bar: move_bar as f32,
                }
            } else {
                TimelineEditCommit::TrimClipOut {
                    layer_id: "trim".into(),
                    bar: move_bar as f32,
                }
            };
            assert_eq!(up.edit_commit, Some(expected));

            let duplicate = sess.pointer(
                &mut selected,
                &mut playhead,
                1240,
                400,
                TimelinePointerPhase::Up,
                move_x,
                y,
                0,
            );
            assert!(duplicate.edit_commit.is_none());
        }
    }

    #[test]
    fn narrow_real_clip_edge_stays_body_move() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "narrow".into(),
                display_name: "clip".into(),
                interval_secs: Some((4.0, 1.0)),
                keys: vec![],
            }],
            Some("narrow"),
        );
        let mut selected = 0;
        let mut playhead = 0.0;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let (a, b, _) = sess.scene.clip0_span_and_keys(0).unwrap();
        let ax = f64::from(bx(&sess.scene, a));
        let bx_ = f64::from(bx(&sess.scene, b));
        assert!(bx_ - ax < TRIM_EDGE_MIN_CLIP_W);

        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            ax + 1.0,
            y,
            1,
        );
        let move_x = ax + 11.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            move_x,
            y,
            1,
        );
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            move_x,
            y,
            1,
        );
        assert!(matches!(
            up.edit_commit,
            Some(TimelineEditCommit::SetClipStart { ref layer_id, .. }) if layer_id == "narrow"
        ));
    }

    #[test]
    fn key_drag_moves_time_only_and_selects_single_key() {
        let (mut sess, mut selected, mut playhead) = session();
        // band1 hero key at 8.0
        let kx = f64::from(bx_default(8.0));
        let ky = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            kx,
            ky,
            0,
        );
        assert_eq!(selected, 3);
        assert!(sess.scene.bands[1].clips[0].keys[0].2);
        assert!(!sess.scene.bands[0].clips[1].keys[1].2); // previous true cleared

        // 水平移動 + 縦ノイズ
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(11.0),
            ky + 40.0,
            0,
        );
        let key = &sess.scene.bands[1].clips[0].keys[0];
        assert!((key.0 - 11.0).abs() < 1e-4);
        assert!((key.1 - 0.71).abs() < 1e-4);

        // clamp to clip.b
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(40.0),
            ky,
            0,
        );
        assert!((sess.scene.bands[1].clips[0].keys[0].0 - 22.0).abs() < 1e-4);
    }

    #[test]
    fn real_projection_key_drag_clamps_to_clip_start() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "12".into(),
                display_name: "keyed".into(),
                interval_secs: Some((4.0, 6.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 11,
                    time_secs: 8.0,
                }],
            }],
            Some("12"),
        );
        assert!(sess.scene.real);
        let mut selected = 0;
        let mut playhead = 0.27;
        let clip = sess.scene.bands[0].clips[0].clone();
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let x = f64::from(bx(&sess.scene, clip.keys[0].0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        let left_x = lx_for_bar_in(&sess.scene, f64::from(clip.a) - 10.0);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            left_x,
            y,
            0,
        );
        assert!((sess.scene.bands[0].clips[0].keys[0].0 - 4.0).abs() < 1e-4);
    }

    #[test]
    fn real_projection_key_drag_commits_once_on_release_with_clamped_time() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            Some("11"),
        );
        assert!(sess.scene.real);
        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let x = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].keys[0].0));
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(down.edit_commit.is_none());
        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, 40.0),
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        // clip span 0..10s。clamp to clip.b=10
        assert!((sess.scene.bands[0].clips[0].keys[0].0 - 10.0).abs() < 1e-3);

        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 40.0),
            y,
            0,
        );
        assert_eq!(
            up.edit_commit,
            Some(TimelineEditCommit::SetPositionKeyTime {
                layer_id: "11".into(),
                key_id: 7,
                bar: sess.scene.bands[0].clips[0].keys[0].0,
            })
        );
        // 二度目のUpではgesture無し → commitなし(二重dispatch防止)
        let up2 = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 40.0),
            y,
            0,
        );
        assert!(up2.edit_commit.is_none());
    }

    #[test]
    fn real_projection_move_cancel_restores_full_state_without_commit() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "3".into(),
                display_name: "a".into(),
                interval_secs: Some((2.0, 4.0)), // bars 1..3、移動余地あり
                keys: vec![],
            }],
            Some("3"),
        );
        let mut selected = 0;
        let mut playhead = 0.1;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let before = sess.scene.clone();
        let before_selected = selected;
        let before_playhead = playhead;
        let origin_a = before.bands[0].clips[0].a;
        let origin_b = before.bands[0].clips[0].b;

        let mid = f64::from(bx(&sess.scene, (origin_a + origin_b) * 0.5));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            mid,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            mid + 24.0,
            y,
            0,
        );
        assert_ne!(sess.scene.bands[0].clips[0].a, before.bands[0].clips[0].a);

        let cancel = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            mid + 24.0,
            y,
            0,
        );
        assert!(cancel.edit_commit.is_none());
        assert_eq!(sess.scene, before);
        assert_eq!(selected, before_selected);
        assert!((playhead - before_playhead).abs() < 1e-9);
    }

    #[test]
    fn real_projection_move_trim_commit_on_release_cancel_restores_without_commit() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[
                SnapshotLayerInput {
                    layer_id: "3".into(),
                    display_name: "a".into(),
                    interval_secs: Some((2.0, 4.0)), // bars 1..3
                    keys: vec![],
                },
                SnapshotLayerInput {
                    layer_id: "4".into(),
                    display_name: "b".into(),
                    interval_secs: Some((0.0, 4.0)), // bars 0..2
                    keys: vec![],
                },
            ],
            Some("3"),
        );
        let mut selected = 0;
        let mut playhead = 40.0 / f64::from(SONG_BARS);
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let origin_a = sess.scene.bands[0].clips[0].a;
        let origin_b = sess.scene.bands[0].clips[0].b;

        // move: mid-clip drag
        let mid = f64::from(bx(&sess.scene, (origin_a + origin_b) * 0.5));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            mid,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            mid + 40.0,
            y,
            0,
        );
        assert!((sess.scene.bands[0].clips[0].a - origin_a).abs() > f32::EPSILON);
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            mid + 40.0,
            y,
            0,
        );
        assert!(matches!(
            up.edit_commit,
            Some(TimelineEditCommit::SetClipStart { .. })
        ));

        // trim left then cancel restores, no commit
        let left = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].a));
        let before = sess.scene.clone();
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            left,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            left + 24.0,
            y,
            0,
        );
        let cancel = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            left + 24.0,
            y,
            0,
        );
        assert!(cancel.edit_commit.is_none());
        assert_eq!(sess.scene.bands[0].clips[0].a, before.bands[0].clips[0].a);
        assert_eq!(sess.scene.bands[0].clips[0].b, before.bands[0].clips[0].b);

        // trim right release commits
        let right = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].b));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            right,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            right - 20.0,
            y,
            0,
        );
        let trim_up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            right - 20.0,
            y,
            0,
        );
        assert!(
            matches!(
                trim_up.edit_commit,
                Some(TimelineEditCommit::TrimClipOut { .. })
            ),
            "got {:?}",
            trim_up.edit_commit
        );
    }

    #[test]
    fn wheel_pan_moves_view_and_clamps_to_song() {
        let (mut sess, _, _) = session();
        // 右へpan(負delta → view増加)。論理48px → span*48/sw bars
        let dirty = sess.scroll(1240, 400, -48.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
        assert!(dirty);
        let expected = 48.0 / surface_width() as f32 * 48.0;
        assert!((sess.scene.view_a - expected).abs() < 1e-3);
        assert!((sess.scene.view_b - (48.0 + expected)).abs() < 1e-3);

        // 大きく右へ飛ばして右端clamp
        sess.scroll(1240, 400, -10_000.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
        assert!((sess.scene.view_a - 48.0).abs() < 1e-3);
        assert!((sess.scene.view_b - 96.0).abs() < 1e-3);

        // 大きく左へ飛ばして左端clamp
        sess.scroll(1240, 400, 10_000.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
        assert!((sess.scene.view_a - 0.0).abs() < 1e-3);
        assert!((sess.scene.view_b - 48.0).abs() < 1e-3);
    }

    #[test]
    fn cmd_wheel_zoom_keeps_anchor_bar_fixed() {
        let (mut sess, _, _) = session();
        let anchor_bar = 24.0;
        let lx = lx_for_bar(anchor_bar);
        let before_bar = bar_at_lx(&sess.scene, lx);
        assert!((before_bar - anchor_bar).abs() < 1e-6);

        assert!(sess.scroll(1240, 400, 0.0, 12.0, 0.0, 1, lx, 100.0));
        assert!((bar_at_lx(&sess.scene, lx) - anchor_bar).abs() < 1e-3);
    }

    #[test]
    fn wheel_vertical_delta_has_priority_for_horizontal_pan() {
        let (mut sess, _, _) = session();
        sess.scene.view_a = 24.0;
        sess.scene.view_b = 72.0;
        let before_a = sess.scene.view_a;
        let before_b = sess.scene.view_b;

        let dirty = sess.scroll(1240, 400, 1.0, -48.0, 0.0, 0, lx_for_bar(24.0), 100.0);
        assert!(dirty);
        let expected =
            before_a + (-(-48.0) / surface_width() * f64::from(before_b - before_a)) as f32;
        assert!((sess.scene.view_a - expected).abs() < 1e-3);
        assert!((sess.scene.view_b - (before_b - before_a + expected)).abs() < 1e-3);
    }

    #[test]
    fn draw_timeline_inbox_pixels_stable_after_pan() {
        let w = 1240u32;
        let h = 620u32;
        let mut before = vec![0u8; (w * h * 4) as usize];
        draw_timeline(&TimelineScene::default(), &mut before, w, h, 0.22, 1);

        let mut sess = TimelineSession::default();
        let delta = -surface_width() / f64::from(SONG_BARS) * 20.0;
        sess.scroll(w, h, delta, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);

        let mut after = vec![0u8; (w * h * 4) as usize];
        draw_timeline(&sess.scene, &mut after, w, h, 0.22, 1);

        let x = 60usize;
        let y = body_top() as usize + 5;
        let idx = (y * w as usize + x) * 4;
        assert_eq!(&before[idx..idx + 4], &after[idx..idx + 4]);
    }

    #[test]
    fn pinch_zoom_keeps_anchor_bar_and_clamps_span() {
        let (mut sess, _, _) = session();
        let anchor_bar = 24.0;
        let lx = lx_for_bar(anchor_bar);
        // magnification=0.5 → span *= 0.5 → 24
        assert!(sess.scroll(1240, 400, 0.0, 0.0, 0.5, 0, lx, 100.0));
        assert!((sess.scene.view_b - sess.scene.view_a - 24.0).abs() < 1e-3);
        assert!((bar_at_lx(&sess.scene, lx) - anchor_bar).abs() < 1e-3);

        // 大きく拡大してmin span 4
        assert!(sess.scroll(1240, 400, 0.0, 0.0, 0.9, 0, lx, 100.0));
        assert!((sess.scene.view_b - sess.scene.view_a - 4.0).abs() < 1e-3);

        // 大きく縮小してmax span 96
        sess.scene.view_a = 20.0;
        sess.scene.view_b = 40.0;
        let lx2 = lx_for_bar_in(&sess.scene, 30.0);
        assert!(sess.scroll(1240, 400, 0.0, 0.0, -10.0, 0, lx2, 100.0));
        assert!((sess.scene.view_a - 0.0).abs() < 1e-3);
        assert!((sess.scene.view_b - 96.0).abs() < 1e-3);
    }

    #[test]
    fn overview_drag_centers_view_and_clamps() {
        let (mut sess, mut selected, mut playhead) = session();
        // overview上で曲中央(bar 48)へ
        let ox_48 = f64::from(SURF_X) + 48.0 / f64::from(SONG_BARS) * surface_width();
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            ox_48,
            5.0,
            0,
        );
        assert!(out.dirty);
        assert!(!out.feedback);
        assert!((sess.scene.view_a - 24.0).abs() < 1e-3);
        assert!((sess.scene.view_b - 72.0).abs() < 1e-3);

        // 右端へ → view 48..96
        let ox_96 = f64::from(SURF_X) + surface_width();
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            ox_96,
            5.0,
            0,
        );
        assert!((sess.scene.view_a - 48.0).abs() < 1e-3);
        assert!((sess.scene.view_b - 96.0).abs() < 1e-3);
    }

    #[test]
    fn playhead_is_song_normalized_and_survives_view_change() {
        let (mut sess, mut selected, mut playhead) = session();
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar(24.0),
            f64::from(OVER_H + 1.0) + 4.0,
            0,
        );
        assert!((playhead - 24.0 / 96.0).abs() < 1e-9);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar(24.0),
            f64::from(OVER_H + 1.0) + 4.0,
            0,
        );

        // viewを後半へ移しても同じpointer barでscrubが正しい
        sess.scene.view_a = 48.0;
        sess.scene.view_b = 96.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar_in(&sess.scene, 72.0),
            f64::from(OVER_H + 1.0) + 4.0,
            0,
        );
        assert!((playhead - 72.0 / 96.0).abs() < 1e-9);
    }

    #[test]
    fn clip_hit_works_after_view_moves_to_second_half() {
        let (mut sess, mut selected, mut playhead) = session();
        sess.scene.view_a = 48.0;
        sess.scene.view_b = 96.0;
        sess.scene.bands[5].clips[0].a = 60.0;
        sess.scene.bands[5].clips[0].b = 80.0;
        let x = lx_for_bar_in(&sess.scene, 70.0);
        let y = body_top() + f64::from(ROW) * 5.5;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        // band0..4 flat: 3+2+3+2+0 = 10
        assert_eq!(selected, 10);
    }

    #[test]
    fn overview_cancel_restores_view() {
        let (mut sess, mut selected, mut playhead) = session();
        let before = (sess.scene.view_a, sess.scene.view_b);
        let ox_48 = f64::from(SURF_X) + 48.0 / f64::from(SONG_BARS) * surface_width();
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            ox_48,
            5.0,
            0,
        );
        assert!((sess.scene.view_a - 24.0).abs() < 1e-3);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            ox_48,
            5.0,
            0,
        );
        assert!((sess.scene.view_a - before.0).abs() < 1e-6);
        assert!((sess.scene.view_b - before.1).abs() < 1e-6);
    }

    #[test]
    fn real_clip_down_emits_select_layer_once_and_empty_bar_clears() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "42".into(),
                display_name: "clip".into(),
                interval_secs: Some((2.0, 8.0)),
                keys: vec![],
            }],
            None,
        );
        assert!(sess.scene.real);
        let mut selected = -1;
        let mut playhead = 0.1;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let x = f64::from(bx(&sess.scene, 3.0));
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert_eq!(
            down.selection_commit,
            Some(TimelineSelectionCommit::SelectLayer {
                layer_id: "42".into(),
            })
        );
        assert_eq!(selected, 0);

        let empty_x = f64::from(bx(&sess.scene, 20.0));
        let clear = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            empty_x,
            y,
            0,
        );
        assert_eq!(
            clear.selection_commit,
            Some(TimelineSelectionCommit::ClearSelection)
        );
        assert_eq!(selected, -1);
    }

    #[test]
    fn fixture_clip_down_does_not_emit_selection_commit() {
        let (mut sess, mut selected, mut playhead) = session();
        assert!(!sess.scene.real);
        let y = body_top() + 5.0;
        let x = lx_for_bar(2.0);
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(down.selection_commit.is_none());
        assert!(selected >= 0);
    }

    #[test]
    fn lane_drag_commits_reparent_to_destination_band() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[
                SnapshotLayerInput {
                    layer_id: "src".into(),
                    display_name: "a".into(),
                    interval_secs: Some((0.0, 4.0)),
                    keys: vec![],
                },
                SnapshotLayerInput {
                    layer_id: "dst".into(),
                    display_name: "b".into(),
                    interval_secs: Some((0.0, 4.0)),
                    keys: vec![],
                },
            ],
            Some("src"),
        );
        let mut selected = 0;
        let mut playhead = 40.0 / f64::from(SONG_BARS);
        let x = f64::from(bx(&sess.scene, 2.0));
        let y0 = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let y1 = body_top() + f64::from(ROW) * 1.5 - 0.5;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y0,
            0,
        );
        assert!(down.edit_commit.is_none());
        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x,
            y1,
            0,
        );
        assert!(moved.edit_commit.is_none());
        assert_eq!(sess.scene.lane_preview_band, Some(1));
        assert!(sess.scene.snap_guide.is_none());
        let mut without_preview = sess.scene.clone();
        without_preview.lane_preview_band = None;
        let mut baseline = vec![0u8; 1240 * 400 * 4];
        let mut preview = vec![0u8; 1240 * 400 * 4];
        draw_timeline(
            &without_preview,
            &mut baseline,
            1240,
            400,
            playhead,
            selected,
        );
        draw_timeline(&sess.scene, &mut preview, 1240, 400, playhead, selected);
        assert_ne!(
            preview, baseline,
            "destination lane feedback must be visible"
        );
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y1,
            0,
        );
        match up.edit_commit {
            Some(TimelineEditCommit::ReparentClip {
                layer_id,
                dest_layer_id,
                bar,
            }) => {
                assert_eq!(layer_id, "src");
                assert_eq!(dest_layer_id, "dst");
                assert!((bar - 0.0).abs() < 1e-3);
            }
            other => panic!("expected ReparentClip, got {other:?}"),
        }
        assert!(sess.scene.lane_preview_band.is_none());
        assert!(sess.scene.snap_guide.is_none());
        let second_up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y1,
            0,
        );
        assert!(second_up.edit_commit.is_none());
    }

    #[test]
    fn real_key_down_emits_select_layer_for_owner_clip() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            None,
        );
        let mut selected = -1;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let x = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].keys[0].0));
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert_eq!(
            down.selection_commit,
            Some(TimelineSelectionCommit::SelectLayer {
                layer_id: "11".into(),
            })
        );
    }

    #[test]
    fn real_projection_move_snaps_to_other_clip_edge_and_commits_snapped_start() {
        let mut sess = TimelineSession::default();
        let mut scene = TimelineScene::default();
        scene.real = true;
        scene.bands[0].clips[0].a = 0.0;
        scene.bands[0].clips[0].b = 7.3;
        scene.bands[0].clips[0].layer_id = "move-a".into();
        scene.bands[0].clips[1].a = 9.0;
        scene.bands[0].clips[1].b = 15.0;
        scene.bands[0].clips[1].layer_id = "move-b".into();
        scene.bands[0].clips[2].a = 18.0;
        scene.bands[0].clips[2].b = 25.0;
        scene.bands[0].clips[2].layer_id = "move-c".into();
        sess.scene = scene;

        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar_in(&sess.scene, 10.0),
            y,
            0,
        );
        assert!(down.edit_commit.is_none());

        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, 7.5),
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        assert!(
            (sess.scene.bands[0].clips[1].a - 7.3).abs() < 1e-3,
            "got {}",
            sess.scene.bands[0].clips[1].a
        );
        assert_eq!(sess.scene.snap_guide, Some(7.3));
        let mut without_guide = sess.scene.clone();
        without_guide.snap_guide = None;
        let mut baseline = vec![0u8; 1240 * 400 * 4];
        let mut preview = vec![0u8; 1240 * 400 * 4];
        draw_timeline(&without_guide, &mut baseline, 1240, 400, playhead, selected);
        draw_timeline(&sess.scene, &mut preview, 1240, 400, playhead, selected);
        assert_ne!(preview, baseline, "snap guide must be visible during drag");

        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 7.5),
            y,
            0,
        );
        assert_eq!(
            up.edit_commit,
            Some(TimelineEditCommit::SetClipStart {
                layer_id: "move-b".into(),
                bar: 7.3,
            })
        );
        assert!(sess.scene.snap_guide.is_none());
    }

    #[test]
    fn real_projection_move_snap_ignores_cmd_key_modifier() {
        let mut sess = TimelineSession::default();
        let mut scene = TimelineScene::default();
        scene.real = true;
        scene.bands[0].clips[0].a = 0.0;
        scene.bands[0].clips[0].b = 7.3;
        scene.bands[0].clips[0].layer_id = "move-a".into();
        scene.bands[0].clips[1].a = 9.0;
        scene.bands[0].clips[1].b = 15.0;
        scene.bands[0].clips[1].layer_id = "move-b".into();
        sess.scene = scene;

        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let down_bar = 10.0;
        let move_bar = 10.6;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar_in(&sess.scene, down_bar),
            y,
            0,
        );
        assert!(down.edit_commit.is_none());
        assert_eq!(selected, 1);
        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, move_bar),
            y,
            1,
        );
        assert!(moved.edit_commit.is_none());
        assert!(sess.scene.snap_guide.is_none());
        let expected_bar = 9.6_f32;
        assert!(
            (sess.scene.bands[0].clips[1].a - expected_bar).abs() < 1e-3,
            "got {}",
            sess.scene.bands[0].clips[1].a
        );

        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, move_bar),
            y,
            1,
        );
        assert_eq!(
            up.edit_commit,
            Some(TimelineEditCommit::SetClipStart {
                layer_id: "move-b".into(),
                bar: expected_bar,
            })
        );
    }

    #[test]
    fn real_projection_trim_snaps_to_frame_and_commits() {
        let mut sess = TimelineSession::default();
        let mut scene = TimelineScene::default();
        scene.real = true;
        scene.bands[0].clips[0].a = 0.0;
        scene.bands[0].clips[0].b = 7.3;
        scene.bands[0].clips[0].layer_id = "trim-a".into();
        scene.bands[0].clips[1].a = 9.0;
        scene.bands[0].clips[1].b = 16.0;
        scene.bands[0].clips[1].layer_id = "trim-b".into();
        scene.bands[0].clips[2].a = 18.0;
        scene.bands[0].clips[2].b = 22.0;
        scene.bands[0].clips[2].layer_id = "trim-c".into();
        sess.scene = scene;

        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            f64::from(bx(&sess.scene, 16.0)),
            y,
            0,
        );
        assert!(down.edit_commit.is_none());
        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, 17.9),
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        assert!(
            (sess.scene.bands[0].clips[1].b - 17.9).abs() < 1e-3,
            "got {}",
            sess.scene.bands[0].clips[1].b
        );

        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 17.9),
            y,
            0,
        );
        match up.edit_commit {
            Some(TimelineEditCommit::TrimClipOut { layer_id, bar }) => {
                assert_eq!(layer_id, "trim-b");
                assert!((bar - 17.9).abs() < 1e-3);
            }
            other => panic!("expected TrimClipOut at 17.9s, got {other:?}"),
        }
    }

    #[test]
    fn real_projection_key_drag_snaps_to_playhead_and_commits() {
        let mut sess = TimelineSession::default();
        let mut scene = TimelineScene::default();
        scene.real = true;
        scene.bands[1].clips[0].layer_id = "11".into();
        sess.scene = scene;

        let mut selected = 0;
        let mut playhead = 4.7 / f64::from(SONG_BARS);
        let y = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
        let key_id = sess.scene.bands[1].clips[0].keys[0].3;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            f64::from(bx(&sess.scene, sess.scene.bands[1].clips[0].keys[0].0)),
            y,
            0,
        );
        assert!(down.edit_commit.is_none());

        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, 4.6),
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        assert!((sess.scene.bands[1].clips[0].keys[0].0 - 4.6).abs() < 1e-3);

        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 4.6),
            y,
            0,
        );
        let edit_bar = match up.edit_commit {
            Some(TimelineEditCommit::SetPositionKeyTime {
                layer_id,
                key_id: up_key_id,
                bar,
            }) if layer_id.as_str() == "11" && up_key_id == key_id => bar,
            _ => {
                panic!(
                    "expected key drag commit for layer 11 key {key_id}, got {:?}",
                    up.edit_commit
                )
            }
        };
        assert!((edit_bar - 4.6).abs() < 1e-3);
    }

    #[test]
    fn snap_threshold_tracks_6px_distance_under_zoom_changes_bar_delta() {
        let base = TimelineScene::default().with_fps(1, 1);
        let band = 4usize;
        let integer_bar = 10.0f32;
        let near6_base = 6.0 / surface_width() as f32 * (base.view_b - base.view_a);
        let far7_base = 7.0 / surface_width() as f32 * (base.view_b - base.view_a);
        assert!(
            (test_snap_bar(&base, 0.0, band, None, integer_bar + near6_base * 0.9, 0,)
                - integer_bar)
                .abs()
                < 2e-3
        );
        assert!(
            (test_snap_bar(&base, 0.0, band, None, integer_bar + far7_base, 0)
                - (integer_bar + far7_base))
                .abs()
                < 2e-3
        );

        let mut sess = TimelineSession::default();
        sess.scene = sess.scene.clone().with_fps(1, 1);
        sess.scroll(1240, 400, 0.0, 0.0, 0.5, 0, lx_for_bar(24.0), 100.0);
        let half = &sess.scene;
        let near6_half = 6.0 / surface_width() as f32 * (half.view_b - half.view_a);
        let far7_half = 7.0 / surface_width() as f32 * (half.view_b - half.view_a);
        assert!(near6_base > near6_half);
        assert!(
            (test_snap_bar(half, 0.0, band, None, integer_bar + near6_half * 0.9, 0) - integer_bar)
                .abs()
                < 2e-3
        );
        assert!(
            (test_snap_bar(half, 0.0, band, None, integer_bar + far7_half, 0)
                - (integer_bar + far7_half))
                .abs()
                < 2e-3
        );
    }

    #[test]
    fn selected_real_key_tracks_sel_flag() {
        let mut scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            None,
        );
        assert!(selected_real_key(&scene).is_none());
        assert!(remove_position_key_commit(&scene).is_none());
        scene.bands[0].clips[0].keys[0].2 = true;
        assert_eq!(selected_real_key(&scene), Some(("11".into(), 7)));
        assert_eq!(
            remove_position_key_commit(&scene),
            Some(TimelineEditCommit::RemovePositionKey {
                layer_id: "11".into(),
                key_id: 7,
            })
        );
    }

    #[test]
    fn key_click_without_drag_does_not_commit_remove() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            Some("11"),
        );
        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let x = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].keys[0].0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y,
            0,
        );
        assert!(up.edit_commit.is_none());
        assert_eq!(selected_real_key(&sess.scene), Some(("11".into(), 7)));
        assert_eq!(
            remove_position_key_commit(&sess.scene),
            Some(TimelineEditCommit::RemovePositionKey {
                layer_id: "11".into(),
                key_id: 7,
            })
        );
    }

    #[test]
    fn real_clip_body_click_clears_key_selection() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            Some("11"),
        );
        test_select_first_real_key(&mut sess.scene);
        assert!(selected_real_key(&sess.scene).is_some());
        let mut selected = 0;
        let mut playhead = 0.1;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        // keyでないclip本体中央。
        let x = f64::from(bx(&sess.scene, 3.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(selected_real_key(&sess.scene).is_none());
    }

    #[test]
    fn clip_move_commit_follows_layer_id_after_band_index_shift() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[
                SnapshotLayerInput {
                    layer_id: "1".into(),
                    display_name: "a".into(),
                    interval_secs: Some((0.0, 4.0)),
                    keys: vec![],
                },
                SnapshotLayerInput {
                    layer_id: "2".into(),
                    display_name: "b".into(),
                    interval_secs: Some((0.0, 4.0)),
                    keys: vec![],
                },
            ],
            Some("2"),
        );
        let mut selected = 1;
        let mut playhead = 0.1;
        // 0..4s clipの中央。adopt済みedge幅との境界をbody pressに使わない。
        let x = lx_for_bar_in(&sess.scene, 2.0);
        let y = body_top() + f64::from(ROW) + 5.0;
        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert_eq!(
            down.selection_commit,
            Some(TimelineSelectionCommit::SelectLayer {
                layer_id: "2".into()
            })
        );
        assert_eq!(sess.scene.bands[1].clips[0].layer_id, "2");

        let dummy = sess.scene.bands[0].clone();
        sess.scene.bands.insert(0, dummy);
        assert_eq!(sess.scene.bands[1].clips[0].layer_id, "1");
        assert_eq!(sess.scene.bands[2].clips[0].layer_id, "2");
        let origin_a = sess.scene.bands[2].clips[0].a;

        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar_in(&sess.scene, 3.0),
            y,
            0,
        );
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar_in(&sess.scene, 3.0),
            y,
            0,
        );
        assert_eq!(sess.scene.bands[2].clips[0].layer_id, "2");
        assert!((sess.scene.bands[1].clips[0].a - 0.0).abs() < 1e-3);
        match up.edit_commit {
            Some(TimelineEditCommit::SetClipStart { layer_id, bar }) => {
                assert_eq!(layer_id, "2");
                assert!((bar - sess.scene.bands[2].clips[0].a).abs() < 1e-3);
                assert!((sess.scene.bands[2].clips[0].a - origin_a).abs() > 1e-3);
            }
            other => panic!("expected SetClipStart for layer 2, got {other:?}"),
        }
    }

    #[test]
    fn restore_key_selection_matches_by_key_id() {
        let mut scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![
                    SnapshotKeyInput {
                        key_id: 7,
                        time_secs: 4.0,
                    },
                    SnapshotKeyInput {
                        key_id: 9,
                        time_secs: 6.0,
                    },
                ],
            }],
            Some("11"),
        );
        assert!(restore_key_selection(&mut scene, "11", 9));
        assert_eq!(selected_real_key(&scene), Some(("11".into(), 9)));
        assert!(!scene.bands[0].clips[0].keys[0].2);
        assert!(scene.bands[0].clips[0].keys[1].2);
    }

    #[test]
    fn restore_key_selection_requires_layer_match() {
        let mut scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![
                    SnapshotKeyInput {
                        key_id: 7,
                        time_secs: 4.0,
                    },
                    SnapshotKeyInput {
                        key_id: 9,
                        time_secs: 6.0,
                    },
                ],
            }],
            Some("11"),
        );
        assert!(!restore_key_selection(&mut scene, "12", 9));
        assert!(selected_real_key(&scene).is_none());
    }

    #[test]
    fn real_projection_key_drag_discarded_when_scene_layer_id_changes() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            Some("11"),
        );
        sess.scene.real = true;
        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let start_key = sess.scene.bands[0].clips[0].keys[0];
        let x = f64::from(bx(&sess.scene, start_key.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        let replaced = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "12".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            Some("12"),
        );
        sess.scene = replaced;
        sess.scene.real = true;

        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x + 12.0,
            y,
            0,
        );
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x + 12.0,
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        assert!(up.edit_commit.is_none());
        assert!(!sess.has_active_gesture());
        assert_eq!(sess.scene.bands[0].clips[0].keys[0].0, start_key.0);
    }

    #[test]
    fn real_clip_move_preview_does_not_shift_keys() {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "k-move".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 8.0)), // bars 0..4
                keys: vec![SnapshotKeyInput {
                    key_id: 3,
                    time_secs: 2.0, // bar 1.0
                }],
            }],
            Some("k-move"),
        );
        let key_before = sess.scene.bands[0].clips[0].keys[0].0;
        let origin_a = sess.scene.bands[0].clips[0].a;
        let mut selected = 0;
        let mut playhead = 0.1;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let mid = f64::from(bx(
            &sess.scene,
            (origin_a + sess.scene.bands[0].clips[0].b) * 0.5,
        ));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            mid,
            y,
            0,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            mid + 40.0,
            y,
            0,
        );
        assert!((sess.scene.bands[0].clips[0].a - origin_a).abs() > f32::EPSILON);
        assert!((sess.scene.bands[0].clips[0].keys[0].0 - key_before).abs() < 1e-4);
    }

    #[test]
    fn discard_active_gesture_drops_without_restore() {
        let mut sess = TimelineSession::default();
        let mut selected = 0;
        let mut playhead = 0.27;
        let y = body_top() + 5.0;
        let x = lx_for_bar(2.0);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        let before = sess.scene.clone();
        assert!(sess.discard_active_gesture());
        assert_eq!(sess.scene, before);
        assert!(!sess.discard_active_gesture());
    }

    #[test]
    fn playhead_near_down_starts_scrub_distant_clip_does_not() {
        let (mut sess, mut selected, mut playhead) = session();
        playhead = 12.0 / 96.0;
        let px = f64::from(bx(&sess.scene, 12.0));
        let body_y = body_top() + f64::from(ROW) + 5.0;
        let near = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            px + 2.0,
            body_y,
            0,
        );
        assert!(near.scrub_playhead.is_some());
        assert!(matches!(sess.gesture, Some(ActiveGesture::Scrub { .. })));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            px + 2.0,
            body_y,
            0,
        );

        let (mut sess, mut selected, mut playhead) = session();
        playhead = 12.0 / 96.0;
        // band1 hero本体。key(8/13/18)とplayhead(12)から離す。
        let clip_x = lx_for_bar(10.0);
        let clip_down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            clip_x,
            body_y,
            0,
        );
        assert!(clip_down.scrub_playhead.is_none());
        assert!(matches!(
            sess.gesture,
            Some(ActiveGesture::SelectOrMove { .. })
        ));
        assert!((playhead - 12.0 / 96.0).abs() < 1e-9);
    }

    #[test]
    fn hover_hit_maps_to_cursor_kinds() {
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::Trim, false),
            CursorKind::ResizeLeftRight
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::PlayheadOrRuler, false),
            CursorKind::ResizeLeftRight
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::Key, false),
            CursorKind::PointingHand
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::Clip, false),
            CursorKind::OpenHand
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::Clip, true),
            CursorKind::ClosedHand
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::None, false),
            CursorKind::Arrow
        );
        // drag中はhitを外れてもclosedHand維持(gesture active優先)。
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::None, true),
            CursorKind::ClosedHand
        );
        assert_eq!(
            cursor_for_timeline_hover(TimelineHoverHit::Trim, true),
            CursorKind::ClosedHand
        );
        assert_eq!(cursor_for_stage_hover(true, false), CursorKind::OpenHand);
        assert_eq!(cursor_for_stage_hover(true, true), CursorKind::ClosedHand);
        assert_eq!(cursor_for_stage_hover(false, true), CursorKind::ClosedHand);
        assert_eq!(cursor_for_stage_hover(false, false), CursorKind::Arrow);

        let scene = TimelineScene::default();
        let playhead = 12.0 / 96.0;
        let px = f64::from(bx(&scene, 12.0));
        let body_y = body_top() + f64::from(ROW) + 5.0;
        assert_eq!(
            timeline_hover_hit(&scene, playhead, 1240, 400, px, body_y),
            TimelineHoverHit::PlayheadOrRuler
        );
        assert_eq!(
            timeline_hover_hit(&scene, playhead, 1240, 400, lx_for_bar(10.0), body_y),
            TimelineHoverHit::Clip
        );
        let ruler_y = f64::from(OVER_H + 1.0) + 4.0;
        assert_eq!(
            timeline_hover_hit(&scene, 0.0, 1240, 400, lx_for_bar(20.0), ruler_y),
            TimelineHoverHit::PlayheadOrRuler
        );
    }

    #[test]
    fn first_absolute_tick_snaps_fractional_view_to_bar_multiples() {
        assert_eq!(first_absolute_tick(0.0, 4), 0);
        assert_eq!(first_absolute_tick(0.1, 4), 4);
        assert_eq!(first_absolute_tick(2.3, 4), 4);
        assert_eq!(first_absolute_tick(4.0, 4), 4);
        assert_eq!(first_absolute_tick(4.01, 4), 8);
        assert_eq!(first_absolute_tick(1.0, 8), 8);
    }

    #[test]
    fn ruler_label_step_is_integer_frames() {
        let scene = TimelineScene::default();
        let step = ruler_label_step_secs(&scene, 1032.0);
        let frame = frame_duration_secs(&scene);
        let frames = step / frame;
        assert!(
            (frames - frames.round()).abs() < 1e-4,
            "step={step} frame={frame}"
        );
        assert_eq!(format_ruler_time(0.0, &scene, false), "0:00");
        assert_eq!(format_ruler_time(1.0, &scene, true), "0:01:00");
    }

    #[test]
    fn snap_to_frame_at_30fps_lands_on_frame_grid() {
        let scene = TimelineScene::default();
        assert!((snap_to_frame(&scene, 1.0) - 1.0).abs() < 1e-5);
        assert!((snap_to_frame(&scene, 1.0 + 1.0 / 60.0) - 1.0).abs() < 1e-5);
        assert!((snap_to_frame(&scene, 1.0 + 1.0 / 30.0) - (1.0 + 1.0 / 30.0)).abs() < 1e-5);
    }

    #[test]
    fn product_snap_bar_uses_fps_frames() {
        let scene = TimelineScene::empty_host().with_fps(30, 1);
        assert!(scene.real);
        let half = test_snap_bar(&scene, 0.0, 0, None, 1.0 + 1.0 / 60.0, 0);
        assert!((half - 1.0).abs() < 1e-5, "snap={half}");
        let on_frame = 1.0 + 1.0 / 30.0;
        let landed = test_snap_bar(&scene, 0.0, 0, None, on_frame, 0);
        assert!((landed - on_frame).abs() < 1e-5, "snap={landed}");
    }

    #[test]
    fn product_ruler_format_is_timecode_not_bar() {
        let scene = TimelineScene::empty_host().with_fps(30, 1);
        assert!(scene.real);
        let twelve = format_ruler_time(12.0, &scene, false);
        assert_eq!(twelve, "0:12");
        assert!(!twelve.to_ascii_lowercase().contains("bar"));
        assert_eq!(format_ruler_time(12.0, &scene, true), "0:12:00");
        assert_eq!(format_ruler_time(72.0, &scene, false), "1:12");
    }

    #[test]
    fn key_hit_radius_matches_outer_stroke_5_6() {
        assert!((KEY_HIT_PX - 5.6).abs() < f64::EPSILON);
    }

    #[test]
    fn key_hit_boundary_is_5_6px() {
        let scene = TimelineScene::default();
        // band 0 clip 1のkey(bar 20.0)。playheadは遠くへ置き優先順位の干渉を避ける。
        let playhead = 0.9;
        let kx = f64::from(bx(&scene, 20.0));
        let row_cy = body_top() + 0.5 * f64::from(ROW) - 0.5;
        assert_eq!(
            timeline_hover_hit(&scene, playhead, 1240, 400, kx + 5.5, row_cy),
            TimelineHoverHit::Key
        );
        assert_eq!(
            timeline_hover_hit(&scene, playhead, 1240, 400, kx, row_cy + 5.5),
            TimelineHoverHit::Key
        );
        // 5.6pxの外はkeyではない(clip本体へ落ちる)。
        assert_ne!(
            timeline_hover_hit(&scene, playhead, 1240, 400, kx + 5.7, row_cy),
            TimelineHoverHit::Key
        );
        assert_ne!(
            timeline_hover_hit(&scene, playhead, 1240, 400, kx, row_cy + 5.7),
            TimelineHoverHit::Key
        );
    }

    #[test]
    fn empty_real_scene_reserves_guide_row_and_filled_scene_does_not() {
        let mut scene = TimelineScene::default();
        scene.real = true;
        scene.bands.clear();
        assert!((empty_real_guide_rows(&scene) - ROW).abs() < f32::EPSILON);

        // layerが1件でも入れば消える。
        let filled = TimelineScene::default();
        if !filled.bands.is_empty() {
            let mut real_filled = filled;
            real_filled.real = true;
            assert_eq!(empty_real_guide_rows(&real_filled), 0.0);
        }

        // fixture(real=false)では空でも出さない(PNG sha不変の根拠)。
        let mut fixture = TimelineScene::default();
        fixture.real = false;
        fixture.bands.clear();
        assert_eq!(empty_real_guide_rows(&fixture), 0.0);
    }

    #[test]
    fn host_empty_scene_has_no_fixture_clips() {
        let scene = TimelineScene::empty_host();
        assert!(scene.real);
        assert!(scene.bands.is_empty());
        assert!(scene.locators.is_empty());
        assert!(!scene.bands.iter().any(|band| {
            band.clips.iter().any(|clip| {
                clip.name == "sky_plate"
                    || clip.name == "hero"
                    || clip.name == "track_master.wav"
                    || clip.name == "street_loop.mp4"
                    || clip.layer_id.is_empty() && !clip.name.is_empty()
            })
        }));
        let product = TimelineSession::host_product();
        assert!(product.scene.real);
        assert!(product.scene.bands.is_empty());
        assert!(product.scene.locators.is_empty());
        assert_ne!(product.scene.real, TimelineScene::default().real);
    }

    #[test]
    fn host_snapshot_layer_id_is_the_visible_clip_identity() {
        let scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "42".into(),
                display_name: "Rectangle".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![],
            }],
            Some("42"),
        );
        assert!(scene.real);
        assert_eq!(scene.bands.len(), 1);
        assert_eq!(scene.bands[0].clips[0].layer_id, "42");
        assert_eq!(scene.bands[0].clips[0].name, "Rectangle");
        assert_eq!(scene.selected_flat, 0);
    }
}
