//! Skia製Timelineのraster。
//!
//! 元は隔離probe `spikes/skia-timeline-probe/src/bin/motolii_full.rs`(静止PNG)。
//! ここではRN probeのnative timeline surfaceへ描くため、
//! 固定サイズPNGではなくsurface幅へ合わせたscaleで同じ絵を描く。
//!
//! **probe境界**: 本fileは`spikes/`配下のprobeであり製品コードではない。
//! fixtureは`motolii_full.rs`と同じ初期sceneで、Documentは読まない。
//! 時間操作gesture v1はfixture sceneを可変stateとして持つ。

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
const SONG_BARS: f32 = 96.0;
const VIEW_A: f32 = 0.0;
const VIEW_B: f32 = 48.0;

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

const KEY_HIT_PX: f64 = 6.0;
const TRIM_HIT_PX: f64 = 4.0;
const MOVE_ARM_PX: f64 = 3.0;
const MIN_CLIP_BARS: f32 = 1.0;

#[derive(Clone, Debug, PartialEq)]
struct Clip {
    a: f32,
    b: f32,
    slot: usize,
    name: &'static str,
    fx: (u8, u8),
    mute: bool,
    dev: &'static [&'static str],
    keys: Vec<(f32, f32, bool)>,
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
                            name: "sky_plate",
                            fx: (0, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 14.0,
                            b: 27.0,
                            slot: 0,
                            name: "sky_plate",
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![(14.0, 0.42, false), (20.0, 0.03, true)],
                        },
                        Clip {
                            a: 30.0,
                            b: 44.0,
                            slot: 1,
                            name: "city_a",
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
                            name: "hero",
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![
                                (8.0, 0.71, false),
                                (13.0, 0.02, false),
                                (18.0, 0.15, false),
                            ],
                        },
                        Clip {
                            a: 26.0,
                            b: 40.0,
                            slot: 3,
                            name: "hero",
                            fx: (2, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![(31.0, 0.30, false)],
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
                            name: "grain",
                            fx: (1, 0),
                            mute: false,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 20.0,
                            b: 35.0,
                            slot: 2,
                            name: "grain",
                            fx: (1, 0),
                            mute: true,
                            dev: &[],
                            keys: vec![],
                        },
                        Clip {
                            a: 38.0,
                            b: 48.0,
                            slot: 2,
                            name: "grain",
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
                            name: "title_01",
                            fx: (0, 0),
                            mute: false,
                            dev: &["blend", "opacity"],
                            keys: vec![],
                        },
                        Clip {
                            a: 24.0,
                            b: 32.0,
                            slot: 5,
                            name: "title_02",
                            fx: (0, 0),
                            mute: false,
                            dev: &["opacity"],
                            keys: vec![(24.0, 0.5, false)],
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
                        name: "track_master.wav",
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
        }
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
    SelectOrMove {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        press_lx: f64,
        origin_a: f32,
        origin_b: f32,
        origin_keys: Vec<f32>,
        moving: bool,
    },
    TrimStart {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
    },
    TrimEnd {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
    },
    KeyDrag {
        snapshot: GestureSnapshot,
        band: usize,
        clip_idx: usize,
        key_idx: usize,
    },
    Deselect {
        snapshot: GestureSnapshot,
    },
}

/// scene + 進行中gesture。selection/playheadはcallerが所有する。
#[derive(Clone, Debug, Default)]
pub(crate) struct TimelineSession {
    pub scene: TimelineScene,
    gesture: Option<ActiveGesture>,
}

/// pointer処理の結果。`feedback`はselection/playhead変化時のみtrue。
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TimelinePointerOutcome {
    pub feedback: bool,
    pub dirty: bool,
}

impl TimelineSession {
    pub(crate) fn pointer(
        &mut self,
        selected: &mut i32,
        playhead: &mut f64,
        width: u32,
        height: u32,
        phase: TimelinePointerPhase,
        x: f64,
        y: f64,
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

        match phase {
            TimelinePointerPhase::Down => {
                self.gesture = None;
                if let Some(kind) = hit_gesture(&self.scene, lx, ly) {
                    let snapshot = GestureSnapshot {
                        scene: self.scene.clone(),
                        selected: *selected,
                        playhead: *playhead,
                    };
                    match kind {
                        HitKind::Scrub => {
                            *playhead = time_at_lx(lx);
                            self.gesture = Some(ActiveGesture::Scrub { snapshot });
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
                            self.gesture = Some(ActiveGesture::KeyDrag {
                                snapshot,
                                band,
                                clip_idx,
                                key_idx,
                            });
                            dirty = true;
                        }
                        HitKind::TrimStart {
                            band,
                            clip_idx,
                        } => {
                            self.gesture = Some(ActiveGesture::TrimStart {
                                snapshot,
                                band,
                                clip_idx,
                            });
                        }
                        HitKind::TrimEnd {
                            band,
                            clip_idx,
                        } => {
                            self.gesture = Some(ActiveGesture::TrimEnd {
                                snapshot,
                                band,
                                clip_idx,
                            });
                        }
                        HitKind::Clip {
                            band,
                            clip_idx,
                            flat,
                        } => {
                            *selected = flat;
                            let clip = &self.scene.bands[band].clips[clip_idx];
                            self.gesture = Some(ActiveGesture::SelectOrMove {
                                snapshot,
                                band,
                                clip_idx,
                                press_lx: lx,
                                origin_a: clip.a,
                                origin_b: clip.b,
                                origin_keys: clip.keys.iter().map(|k| k.0).collect(),
                                moving: false,
                            });
                            dirty = true;
                        }
                        HitKind::EmptyBar => {
                            *selected = -1;
                            self.gesture = Some(ActiveGesture::Deselect { snapshot });
                            dirty = true;
                        }
                    }
                }
            }
            TimelinePointerPhase::Move => {
                dirty |= self.apply_move(lx, playhead);
            }
            TimelinePointerPhase::Up => {
                self.gesture = None;
            }
            TimelinePointerPhase::Cancel => {
                if let Some(gesture) = self.gesture.take() {
                    let snapshot = match gesture {
                        ActiveGesture::Scrub { snapshot }
                        | ActiveGesture::SelectOrMove { snapshot, .. }
                        | ActiveGesture::TrimStart { snapshot, .. }
                        | ActiveGesture::TrimEnd { snapshot, .. }
                        | ActiveGesture::KeyDrag { snapshot, .. }
                        | ActiveGesture::Deselect { snapshot } => snapshot,
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
        TimelinePointerOutcome { feedback, dirty }
    }

    fn apply_move(&mut self, lx: f64, playhead: &mut f64) -> bool {
        let Some(mut gesture) = self.gesture.take() else {
            return false;
        };
        let mut dirty = false;
        match &mut gesture {
            ActiveGesture::Scrub { .. } => {
                let next = time_at_lx(lx);
                if (next - *playhead).abs() > f64::EPSILON {
                    *playhead = next;
                    dirty = true;
                }
            }
            ActiveGesture::SelectOrMove {
                band,
                clip_idx,
                press_lx,
                origin_a,
                origin_b,
                origin_keys,
                moving,
                ..
            } => {
                let dx_logical = lx - *press_lx;
                if !*moving && dx_logical.abs() > MOVE_ARM_PX {
                    *moving = true;
                }
                if *moving {
                    let dx_bars = (dx_logical / surface_width()) * f64::from(VIEW_B - VIEW_A);
                    let (prev_b, next_a) = neighbors(&self.scene, *band, *clip_idx);
                    let len = *origin_b - *origin_a;
                    let new_a = ((*origin_a as f64) + dx_bars)
                        .clamp(prev_b as f64, (next_a - len) as f64)
                        as f32;
                    let new_b = new_a + len;
                    let key_times: Vec<f32> = origin_keys
                        .iter()
                        .map(|&ot| ot + (new_a - *origin_a))
                        .collect();
                    let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                    if (clip.a - new_a).abs() > f32::EPSILON
                        || (clip.b - new_b).abs() > f32::EPSILON
                    {
                        clip.a = new_a;
                        clip.b = new_b;
                        for (key, nt) in clip.keys.iter_mut().zip(key_times) {
                            key.0 = nt;
                        }
                        dirty = true;
                    }
                }
            }
            ActiveGesture::TrimStart {
                band, clip_idx, ..
            } => {
                let bar = bar_at_lx(lx) as f32;
                let (prev_b, _) = neighbors(&self.scene, *band, *clip_idx);
                let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                let new_a = bar.clamp(prev_b, clip.b - MIN_CLIP_BARS);
                if (clip.a - new_a).abs() > f32::EPSILON {
                    clip.a = new_a;
                    dirty = true;
                }
            }
            ActiveGesture::TrimEnd {
                band, clip_idx, ..
            } => {
                let bar = bar_at_lx(lx) as f32;
                let (_, next_a) = neighbors(&self.scene, *band, *clip_idx);
                let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                let new_b = bar.clamp(clip.a + MIN_CLIP_BARS, next_a);
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
                let bar = bar_at_lx(lx) as f32;
                let clip = &mut self.scene.bands[*band].clips[*clip_idx];
                let new_t = bar.clamp(clip.a, clip.b);
                if let Some(key) = clip.keys.get_mut(*key_idx) {
                    if (key.0 - new_t).abs() > f32::EPSILON {
                        key.0 = new_t;
                        dirty = true;
                    }
                }
            }
            ActiveGesture::Deselect { .. } => {}
        }
        self.gesture = Some(gesture);
        dirty
    }
}

#[derive(Clone, Copy, Debug)]
enum HitKind {
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
}

fn surface_width() -> f64 {
    f64::from(W - SURF_X - 6.0)
}

fn bx(b: f32) -> f32 {
    SURF_X + (b - VIEW_A) / (VIEW_B - VIEW_A) * (W - SURF_X - 6.0)
}

fn bar_at_lx(lx: f64) -> f64 {
    f64::from(VIEW_A) + (lx - f64::from(SURF_X)) / surface_width() * f64::from(VIEW_B - VIEW_A)
}

fn time_at_lx(lx: f64) -> f64 {
    let bar = bar_at_lx(lx);
    ((bar - f64::from(VIEW_A)) / f64::from(VIEW_B - VIEW_A)).clamp(0.0, 1.0)
}

fn body_top() -> f64 {
    f64::from(OVER_H + 1.0 + RULER_H + LOC_H + 1.0)
}

fn body_bottom(scene: &TimelineScene) -> f64 {
    body_top() + f64::from(scene.bands.len() as f32 * ROW)
}

fn neighbors(scene: &TimelineScene, band: usize, clip_idx: usize) -> (f32, f32) {
    let clips = &scene.bands[band].clips;
    let prev_b = if clip_idx == 0 {
        0.0
    } else {
        clips[clip_idx - 1].b
    };
    let next_a = if clip_idx + 1 >= clips.len() {
        SONG_BARS
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

fn hit_gesture(scene: &TimelineScene, lx: f64, ly: f64) -> Option<HitKind> {
    let ry = f64::from(OVER_H + 1.0);
    let time_y0 = body_bottom(scene);
    // 1. ruler帯(小節 / 分秒)
    if ((ly >= ry && ly < ry + f64::from(RULER_H))
        || (ly >= time_y0 && ly < time_y0 + f64::from(TIME_H)))
        && lx >= f64::from(SURF_X)
    {
        return Some(HitKind::Scrub);
    }

    let top = body_top();
    let bottom = body_bottom(scene);
    if ly < top || ly >= bottom || lx < f64::from(SURF_X) {
        return None;
    }

    let band_index = ((ly - top) / f64::from(ROW)) as usize;
    if band_index >= scene.bands.len() {
        return None;
    }
    let band = &scene.bands[band_index];
    let row_cy = top + (band_index as f64 + 0.5) * f64::from(ROW) - 0.5;
    let bar = bar_at_lx(lx);

    let mut flat_before = 0i32;
    for (bi, b) in scene.bands.iter().enumerate() {
        if bi == band_index {
            break;
        }
        flat_before += b.clips.len() as i32;
    }

    // 2. key中心±6
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        for (key_idx, (kt, _, _)) in clip.keys.iter().enumerate() {
            if *kt < clip.a || *kt > clip.b {
                continue;
            }
            let kx = f64::from(bx(*kt));
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

    // 3. trim端 / 4. clip本体
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        let ax = f64::from(bx(clip.a));
        let bx_ = f64::from(bx(clip.b));
        let d_start = (lx - ax).abs();
        let d_end = (lx - bx_).abs();
        let near_start = d_start <= TRIM_HIT_PX;
        let near_end = d_end <= TRIM_HIT_PX;
        if near_start || near_end {
            if near_start && near_end {
                if d_start < d_end {
                    return Some(HitKind::TrimStart {
                        band: band_index,
                        clip_idx,
                    });
                }
                return Some(HitKind::TrimEnd {
                    band: band_index,
                    clip_idx,
                });
            }
            if near_start {
                return Some(HitKind::TrimStart {
                    band: band_index,
                    clip_idx,
                });
            }
            return Some(HitKind::TrimEnd {
                band: band_index,
                clip_idx,
            });
        }
        if bar >= f64::from(clip.a) && bar < f64::from(clip.b) {
            return Some(HitKind::Clip {
                band: band_index,
                clip_idx,
                flat: flat_before + clip_idx as i32,
            });
        }
    }

    // 5. 空きbar面
    Some(HitKind::EmptyBar)
}

/// 描画に使う論理座標系の高さ。
fn logical_height(scene: &TimelineScene) -> f32 {
    OVER_H + RULER_H + LOC_H + 1.0 + scene.bands.len() as f32 * ROW + TIME_H + 2.0
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

fn tf() -> skia_safe::Typeface {
    FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system typeface")
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
/// `playhead`は0..1で表示範囲(VIEW_A..VIEW_B)を走る。
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
    let bx = |b: f32| SURF_X + (b - VIEW_A) / (VIEW_B - VIEW_A) * sw;
    let ox = |b: f32| SURF_X + b / SONG_BARS * sw;

    let count = clip_count(scene).max(1);
    let selected = if selected < 0 {
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
    let (va, vb) = (ox(VIEW_A), ox(VIEW_B));
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

    // ── 小節 ruler ──
    let ry = OVER_H + 1.0;
    fill(
        cv,
        Rect::from_ltrb(0.0, ry, W, ry + RULER_H),
        rgb(SURFACE_HI),
    );
    for b in (VIEW_A as i32..=VIEW_B as i32).step_by(4) {
        let x = bx(b as f32);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H - 6.0, x + 1.0, ry + RULER_H),
            gray(0x6a),
        );
        text(
            cv,
            &format!("{}", b + 1),
            x + 3.0,
            ry + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
    }
    fill(
        cv,
        Rect::from_ltrb(bx(8.0), ry, bx(24.0), ry + 3.0),
        rgb(RULER_MARK),
    );

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

    // ── bands ──
    let mut y = by0;
    let mut flat = 0usize;
    for band in &scene.bands {
        for b in VIEW_A as i32..=VIEW_B as i32 {
            let x = bx(b as f32);
            fill(
                cv,
                Rect::from_ltrb(x, y, x + 1.0, y + ROW - 1.0),
                argb(if b % 4 == 0 { 0x54 } else { 0x14 }, 0x060606),
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
            let nw = measure(c.name, 8.5);
            let mut nx = r.left + 5.0;
            for (b, _, _) in &c.keys {
                if *b < c.a || *b > c.b {
                    continue;
                }
                let kx = bx(*b);
                if kx > nx - 8.0 && kx < nx + nw + 6.0 {
                    nx = kx + 9.0;
                }
            }
            if x - nx > nw * 0.55 {
                text(cv, c.name, nx, cy + 3.2, 8.5, ink);
            }
            for (b, d, s) in &c.keys {
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
        y += ROW;
    }

    // ── 分秒 ruler ──
    fill(cv, Rect::from_ltrb(0.0, y, W, y + TIME_H), rgb(SURFACE_BG));
    for b in (VIEW_A as i32..=VIEW_B as i32).step_by(8) {
        let x = bx(b as f32);
        let sec = b * 2;
        fill(cv, Rect::from_ltrb(x, y, x + 1.0, y + 5.0), gray(0x6a));
        text(
            cv,
            &format!("{}:{:02}", sec / 60, sec % 60),
            x + 3.0,
            y + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
    }
    text(cv, "3:12 total", W - 64.0, y + 11.0, 8.5, rgb(DIM));

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

    // ── playhead。静止画と違い、ここだけ状態で動く ──
    let bar = VIEW_A + (playhead.clamp(0.0, 1.0) as f32) * (VIEW_B - VIEW_A);
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

    cv.restore();
}

/// physical pointerを(平坦化clip index, 表示範囲内の正規化時間)へ写す。
///
/// bar面の外(Inbox / rail / ruler / 分秒ruler)は`None`。
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
    let bar = bar_at_lx(lx);
    let time = time_at_lx(lx);
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
        (TimelineSession::default(), 1, 0.54)
    }

    fn lx_for_bar(bar: f64) -> f64 {
        f64::from(SURF_X) + bar / f64::from(VIEW_B - VIEW_A) * surface_width()
    }

    fn phys(lx: f64, ly: f64) -> (f64, f64) {
        (lx, ly)
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
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(-20.0),
            y,
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
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(1.0),
            y,
        );
        let clip = &sess.scene.bands[0].clips[1];
        assert!((clip.a - 14.0).abs() < 1e-4);
        assert!((clip.b - 27.0).abs() < 1e-4);
    }

    #[test]
    fn trim_cannot_cross_neighbor_clip_edges() {
        let (mut sess, mut selected, mut playhead) = session();
        let y = body_top() + 5.0; // band0

        let clip0_end_x = f64::from(bx(14.0)) + 1.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            clip0_end_x,
            y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(20.0),
            y,
        );
        let clip0 = &sess.scene.bands[0].clips[0];
        assert!((clip0.a - 0.0).abs() < 1e-4);
        assert!((clip0.b - 14.0).abs() < 1e-4);

        let clip1_start_x = f64::from(bx(14.0)) + 1.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            clip1_start_x,
            y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(1.0),
            y,
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
        );
        assert!(!out.feedback);
        assert_eq!(selected, 1);
        assert!((playhead - 0.54).abs() < 1e-9);
    }

    #[test]
    fn scrub_follows_time_ruler() {
        let (mut sess, mut selected, mut playhead) = session();
        let mut y = body_bottom(&TimelineScene::default()) + 4.0;
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            lx_for_bar(33.0),
            y,
        );
        assert!(out.feedback);
        assert!((playhead - 33.0 / 48.0).abs() < 1e-9);
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

    /// `MOTOLII_WRITE_PREVIEW=1 cargo test -- --nocapture` でpanel実寸のPNGを書く。
    /// 見た目の確認用で、判定はしない(probeなのでgoldenを持たない)。
    #[test]
    fn writes_a_panel_sized_preview_when_asked() {
        if std::env::var_os("MOTOLII_WRITE_PREVIEW").is_none() {
            return;
        }
        let (w, h) = (2480u32, 620u32);
        let mut bytes = vec![0u8; (w * h * 4) as usize];
        draw_timeline(&TimelineScene::default(), &mut bytes, w, h, 0.44, 1);
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
        );
        assert!(out.feedback);
        let pressed = playhead;
        assert!((pressed - 12.0 / 48.0).abs() < 1e-9);

        let (x1, y1) = phys(lx_for_bar(24.0), f64::from(OVER_H + 1.0) + 4.0);
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x1,
            y1,
        );
        assert!((playhead - 24.0 / 48.0).abs() < 1e-9);

        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x1,
            y1,
        );
        assert!((playhead - 24.0 / 48.0).abs() < 1e-9);

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
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            x1,
            y1,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            x1,
            y1,
        );
        assert!((playhead - 0.54).abs() < 1e-9);
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
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y,
        );
        assert_eq!(selected, 3); // band0 has 3 clips
        assert!((playhead - 0.54).abs() < 1e-9);
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
        );
        let clip = &sess.scene.bands[1].clips[0];
        let expected_dx = (10.0 / surface_width() * f64::from(VIEW_B - VIEW_A)) as f32;
        assert!((clip.a - (4.0 + expected_dx)).abs() < 1e-4);
        assert!((clip.b - (22.0 + expected_dx)).abs() < 1e-4);
        assert!((clip.keys[0].0 - (8.0 + expected_dx)).abs() < 1e-4);

        // 大きく右へ飛ばしてclamp
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            press_x + 500.0,
            y,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.a - 8.0).abs() < 1e-4); // 26 - 18
        assert!((clip.b - 26.0).abs() < 1e-4);
        assert!((clip.keys[0].0 - 12.0).abs() < 1e-4);
    }

    #[test]
    fn cancel_restores_full_scene_selection_playhead_and_key_state_after_clip_move() {
        let (mut sess, mut selected, mut playhead) = session();
        let key_x = f64::from(bx(8.0));
        let key_y = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            key_x,
            key_y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(9.0),
            key_y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            lx_for_bar(9.0),
            key_y,
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
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            press_x - 200.0,
            y,
        );

        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Cancel,
            press_x - 200.0,
            y,
        );
        assert_eq!(sess.scene, snapshot_scene);
        assert_eq!(selected, snapshot_selected);
        assert!((playhead - snapshot_playhead).abs() < 1e-9);
    }

    #[test]
    fn trim_changes_edges_only_and_respects_min_length() {
        let (mut sess, mut selected, mut playhead) = session();
        // band1 clip0 hero 4..22。右端22にkeyは無い。
        let y = body_top() + f64::from(ROW) + 5.0;
        let end_x = f64::from(bx(22.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            end_x,
            y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(10.0),
            y,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.a - 4.0).abs() < 1e-4);
        assert!((clip.b - 10.0).abs() < 1e-4);

        // 最小長1.0へ押し込み
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(4.2),
            y,
        );
        let clip = &sess.scene.bands[1].clips[0];
        assert!((clip.b - 5.0).abs() < 1e-4);

        // TrimStart: band1 clip1 26..40。左端にkeyは無い。keysは不変。
        let (mut sess, mut selected, mut playhead) = session();
        let keys_before = sess.scene.bands[1].clips[1].keys.clone();
        let start_x = f64::from(bx(26.0));
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            start_x,
            y,
        );
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            lx_for_bar(28.0),
            y,
        );
        let clip = &sess.scene.bands[1].clips[1];
        assert!((clip.a - 28.0).abs() < 1e-4);
        assert!((clip.b - 40.0).abs() < 1e-4);
        assert_eq!(clip.keys, keys_before);
    }

    #[test]
    fn key_drag_moves_time_only_and_selects_single_key() {
        let (mut sess, mut selected, mut playhead) = session();
        // band1 hero key at 8.0
        let kx = f64::from(bx(8.0));
        let ky = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
        sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            kx,
            ky,
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
        );
        assert!((sess.scene.bands[1].clips[0].keys[0].0 - 22.0).abs() < 1e-4);
    }
}
