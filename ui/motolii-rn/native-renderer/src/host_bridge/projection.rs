#[cfg(target_os = "macos")]
use motolii_ui::{motolii_rn_host_projection_stamp, motolii_rn_host_read_snapshot_json};

use super::parse_wire::parse_timeline_projection;
use super::slot::{host_slot, slice_from_written, MAX_SNAPSHOT_JSON_BYTES};
use super::types::HostTimelineProjection;

#[cfg(test)]
use std::sync::atomic::Ordering;

#[cfg(test)]
use super::slot::TEST_SNAPSHOT_READ_COUNT;

/// Hostが在る時だけsnapshotを読む。不在・失敗はNone。
pub(crate) fn try_read_timeline_projection() -> Option<HostTimelineProjection> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(test)]
        TEST_SNAPSHOT_READ_COUNT.fetch_add(1, Ordering::SeqCst);
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(slot.handle, out.as_mut_ptr(), out.len()) };
        if written <= 0 {
            return None;
        }
        let Some(json_bytes) = slice_from_written(&out, written) else {
            return None;
        };
        let Ok(json) = std::str::from_utf8(json_bytes) else {
            return None;
        };
        parse_timeline_projection(json)
    }
}

/// 軽量stamp。(revision, generation)。不在・失敗はNone。serializeしない。
pub(crate) fn try_read_projection_stamp() -> Option<(u64, u64)> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let mut revision = 0u64;
        let mut generation = 0u64;
        if !unsafe { motolii_rn_host_projection_stamp(slot.handle, &mut revision, &mut generation) }
        {
            return None;
        }
        Some((revision, generation))
    }
}

/// scrub 秒 → set_time frame。`bar` は Skia 凍結名で中身は秒。
/// `frame = round(secs * fps.num / fps.den)`。
pub(crate) fn frame_from_scrub_bar(bar: f64, fps_num: i64, fps_den: i64) -> i64 {
    if fps_num <= 0 || fps_den <= 0 {
        return 0;
    }
    const SCALE: i128 = 1_000_000;
    // なぜ: 秒を先にµs丸めすると 24fps の 0.5 frame が 0 になる
    let frames_fixed =
        (bar * crate::timeline_skia::SECONDS_PER_BAR * (fps_num as f64) * (SCALE as f64)).round()
            as i128;
    let den = (fps_den as i128) * SCALE;
    let half = den / 2;
    let signed_half = if frames_fixed.is_negative() {
        -half
    } else {
        half
    };
    ((frames_fixed + signed_half) / den) as i64
}

/// 秒 → RationalTime wire `{num,den}`。`bar` は Skia 凍結名で中身は秒。
/// SCALE固定小数でf64連鎖丸めを避ける。
pub(crate) fn rational_time_parts_from_bar(bar: f64) -> (i64, i64) {
    const SCALE: i128 = 1_000_000;
    let s_fixed = (bar * crate::timeline_skia::SECONDS_PER_BAR * (SCALE as f64)).round() as i128;
    (s_fixed as i64, SCALE as i64)
}

/// host `current_time`(秒) → playhead(0..1)。
/// `song_bars` は Skia 凍結名。SECONDS_PER_BAR=1 なので曲長秒。
pub(crate) fn playhead_from_current_time(num: i64, den: i64, song_bars: f32) -> f64 {
    if den == 0 || song_bars <= 0.0 {
        return 0.0;
    }
    let secs = num as f64 / den as f64;
    let scene_secs = secs / crate::timeline_skia::SECONDS_PER_BAR;
    (scene_secs / f64::from(song_bars)).clamp(0.0, 1.0)
}
