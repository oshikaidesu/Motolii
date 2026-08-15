//! 論理layout・色・hit閾値。session/hit/paintが同じ数値契約を共有するため。

pub(super) const W: f32 = 1240.0;
pub(super) const INBOX_W: f32 = 118.0;
pub(super) const RAIL_W: f32 = 84.0;
pub(super) const SURF_X: f32 = INBOX_W + RAIL_W;
pub(super) const OVER_H: f32 = 22.0;
pub(super) const RULER_H: f32 = 18.0;
pub(super) const LOC_H: f32 = 15.0;
pub(super) const ROW: f32 = 20.0;
pub(super) const TIME_H: f32 = 16.0;
pub(super) const SONG_BARS: f32 = 96.0;
/// 内部単位は秒。旧 1 bar = 2秒は廃止。
pub(super) const SECONDS_PER_BAR: f64 = 1.0;
/// real投影の曲長[秒]。Composition::new_v1()=10秒。
pub(super) const MIN_VIEW_SPAN: f32 = 4.0;
pub(super) const DEFAULT_VIEW_A: f32 = 0.0;
pub(super) const DEFAULT_VIEW_B: f32 = 48.0;

pub(super) const DESKTOP: u32 = 0x2a2a2a;
pub(super) const SURFACE_BG: u32 = 0x363636;
pub(super) const SURFACE_HI: u32 = 0x464646;
pub(super) const SURFACE_LO: u32 = 0x242424;
pub(super) const CONTRAST: u32 = 0x111111;
pub(super) const DIM: u32 = 0x757575;
pub(super) const RULER_MARK: u32 = 0x919191;
pub(super) const FILL_HANDLE: u32 = 0x5d5d5d;
pub(super) const ON_BAR: u32 = 0x141414;
pub(super) const ACCENT: u32 = 0xffad56;
pub(super) const P: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

/// 菱形外stroke(path 5.6)と一致させる。視覚は変えない。
pub(super) const KEY_HIT_PX: f64 = 5.6;
pub(super) const TRIM_EDGE_MAX_PX: f64 = 15.0;
pub(super) const TRIM_EDGE_MIN_CLIP_W: f64 = 25.0;
pub(super) const TRIM_EDGE_MIN_CLIP_H: f64 = 16.0;
/// playhead頭+線の掴み半幅(論理px)。
pub(super) const PLAYHEAD_HIT_PX: f64 = 4.0;
pub(super) const MOVE_ARM_PX: f64 = 3.0;
pub(super) const SNAP_THRESHOLD_LOGICAL_PX: f64 = 6.0;
pub(super) const MIN_CLIP_BARS: f32 = 1.0 / 30.0;
pub(super) const SNAPSHOT_KEY_DIAMOND: f32 = 0.42;
pub(super) const DEFAULT_FPS_NUM: i64 = 30;
pub(super) const DEFAULT_FPS_DEN: i64 = 1;

/// surface幅へ合わせたscale。probeの静止画は幅1240固定なので、幅で合わせる。
pub(super) fn scale_for(width: u32) -> f32 {
    (width as f32 / W).max(0.05)
}
