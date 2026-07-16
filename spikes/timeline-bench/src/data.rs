//! 合成タイムラインデータ(製品スキーマ非依存)。

use std::f32::consts::TAU;

/// クリップ1個分の配置(秒・トラック番号)。
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub track: u32,
    pub start_sec: f32,
    pub duration_sec: f32,
    pub hue: f32,
}

/// キーフレーム1個(親クリップ内の相対秒)。
#[derive(Clone, Copy, Debug)]
pub struct Keyframe {
    pub clip_index: u32,
    pub time_in_clip_sec: f32,
}

/// ベンチ用の大規模タイムライン。
#[derive(Clone, Debug)]
pub struct TimelineModel {
    pub track_count: u32,
    pub total_duration_sec: f32,
    pub clips: Vec<Clip>,
    pub keyframes: Vec<Keyframe>,
}

impl TimelineModel {
  /// `clip_count` クリップと `keyframe_count` キーを決定的に生成する。
    pub fn generate(clip_count: usize, keyframe_count: usize) -> Self {
        let track_count = 32_u32;
        let clips_per_track = clip_count.div_ceil(track_count as usize);
        let total_duration_sec = 600.0_f32;

        let mut clips = Vec::with_capacity(clip_count);
        for i in 0..clip_count {
            let track = (i % track_count as usize) as u32;
            let slot = i / track_count as usize;
            let track_span = total_duration_sec * 0.92;
            let slot_width = track_span / clips_per_track as f32;
            let start_sec = 4.0 + slot as f32 * slot_width + (track as f32 * 0.17);
            let duration_sec = (slot_width * 0.72).clamp(0.35, slot_width * 0.9);
            clips.push(Clip {
                track,
                start_sec,
                duration_sec,
                hue: (i as f32 * 0.618_033_988_7) % 1.0,
            });
        }

        let mut keyframes = Vec::with_capacity(keyframe_count);
        if keyframe_count > 0 && !clips.is_empty() {
            let per_clip = keyframe_count / clips.len();
            let remainder = keyframe_count % clips.len();
            for (clip_index, clip) in clips.iter().enumerate() {
                let count = per_clip + usize::from(clip_index < remainder);
                if count == 0 {
                    continue;
                }
                for k in 0..count {
                    let t = if count == 1 {
                        clip.duration_sec * 0.5
                    } else {
                        clip.duration_sec * (k as f32 / (count - 1) as f32)
                    };
                    keyframes.push(Keyframe {
                        clip_index: clip_index as u32,
                        time_in_clip_sec: t,
                    });
                }
            }
        }

        Self {
            track_count,
            total_duration_sec,
            clips,
            keyframes,
        }
    }

    /// キーの絶対秒(クリップ外は None)。
    #[allow(dead_code)]
    pub fn keyframe_abs_sec(&self, key: &Keyframe) -> Option<f32> {
        let clip = self.clips.get(key.clip_index as usize)?;
        Some(clip.start_sec + key.time_in_clip_sec)
    }
}

/// パン/ズームアニメ用のビュー状態。
#[derive(Clone, Copy, Debug)]
pub struct ViewState {
    pub pan_sec: f32,
    pub pan_track: f32,
    pub zoom: f32,
}

impl ViewState {
    pub fn animate(frame: u32, model: &TimelineModel) -> Self {
        let t = frame as f32 * 0.04;
        let zoom = 0.55 + 0.45 * (t * 0.31).sin().powi(2);
        let max_pan_sec = (model.total_duration_sec - 120.0 / zoom).max(0.0);
        let pan_sec = max_pan_sec * (t * 0.23).sin().mul_add(0.5, 0.5);
        let max_pan_track = (model.track_count as f32 - 8.0 / zoom).max(0.0);
        let pan_track = max_pan_track * (t * 0.17 + TAU * 0.25).cos().mul_add(0.5, 0.5);
        Self {
            pan_sec,
            pan_track,
            zoom,
        }
    }
}
