//! Host投影とfixture scene。描画・hit・gestureの単一入力。

use super::layout::{
    DEFAULT_FPS_DEN, DEFAULT_FPS_NUM, DEFAULT_VIEW_A, DEFAULT_VIEW_B, SECONDS_PER_BAR,
    SNAPSHOT_KEY_DIAMOND, SONG_BARS,
};

/// Host snapshot由来の1 layer行。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SnapshotLayerInput {
    pub layer_id: String,
    pub display_name: String,
    /// `(start_secs, duration_secs)`。Noneは旧host互換の full-width。
    pub interval_secs: Option<(f64, f64)>,
    pub keys: Vec<SnapshotKeyInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SnapshotKeyInput {
    pub key_id: u64,
    pub time_secs: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Clip {
    pub(super) a: f32,
    pub(super) b: f32,
    pub(super) slot: usize,
    pub(super) name: String,
    /// real投影のhost layer id。fixtureは空。
    pub(super) layer_id: String,
    pub(super) fx: (u8, u8),
    pub(super) mute: bool,
    pub(super) dev: &'static [&'static str],
    /// (秒, diamond size, selected, key_id)。fixtureのkey_idは0。
    pub(super) keys: Vec<(f32, f32, bool, u64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Band {
    pub(super) mute: bool,
    pub(super) solo: bool,
    pub(super) mixed: bool,
    pub(super) clips: Vec<Clip>,
}

/// fixture scene。初期値は旧`BANDS`/`LOCATORS`と同一。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TimelineScene {
    pub(super) bands: Vec<Band>,
    pub(super) locators: Vec<(f32, &'static str)>,
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
    pub(super) snap_guide: Option<f32>,
    /// clip lane drag中だけ描く移動先行。Document投影へは載せない。
    pub(super) lane_preview_band: Option<usize>,
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

pub(super) fn clear_all_key_selection(scene: &mut TimelineScene) {
    for band in &mut scene.bands {
        for clip in &mut band.clips {
            for key in &mut clip.keys {
                key.2 = false;
            }
        }
    }
}

/// revision再投影後に旧key_idの選択を復元する。一致なしなら全解除のまま。
pub(super) fn restore_key_selection(
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
pub(super) fn selected_real_key(scene: &TimelineScene) -> Option<(String, u64)> {
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

/// test用: 先頭real keyのsel flagを立てる。
#[cfg(test)]
pub(super) fn test_select_first_real_key(scene: &mut TimelineScene) {
    if let Some(key) = scene
        .bands
        .get_mut(0)
        .and_then(|b| b.clips.get_mut(0))
        .and_then(|c| c.keys.get_mut(0))
    {
        key.2 = true;
    }
}
