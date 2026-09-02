use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use crate::doc::core::RationalTime;
use crate::doc::store::LayerId;

use super::cache::PcmCache;
use super::time_map::TimeMap;

pub const BASE_SCALE: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeakColumn {
    pub at_sec: f64,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum WaveformError {
    #[error("waveform PCM has no channels")]
    NoChannels,
    #[error("waveform PCM is too long: {frames} frames")]
    TooLong { frames: u64 },
    #[error("waveform peak generation failed: {0}")]
    Upstream(String),
    #[error("waveform worker could not start: {0}")]
    Worker(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaveformStatus {
    Pending,
    Ready,
    Failed(WaveformError),
}

struct PeakLevels {
    sample_rate: u32,
    frame_count: u64,
    levels: BTreeMap<u32, waveform_data::WaveformData>,
    resident_bytes: usize,
}

enum PeakState {
    Pending,
    Ready(PeakLevels),
    Failed(WaveformError),
}

/// PCMからTimeline用のpeak列を作る唯一のowner。
///
/// `waveform-data`のplanar入力、値幅、scale、binary表現はここから外へ出さない。
pub struct WaveformPeaks {
    state: Mutex<PeakState>,
}

impl std::fmt::Debug for WaveformPeaks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveformPeaks")
            .field("status", &self.status())
            .finish()
    }
}

impl WaveformPeaks {
    pub fn resident_upper_bound(frame_count: u64) -> usize {
        let mut total = 0usize;
        let mut scale = BASE_SCALE;
        loop {
            let points = frame_count.div_ceil(u64::from(scale));
            total = total.saturating_add(24usize.saturating_add(points as usize * 4));
            if u64::from(scale) >= frame_count.max(u64::from(BASE_SCALE)) {
                break;
            }
            let next = scale.saturating_mul(2);
            if next == scale || next > i32::MAX as u32 {
                break;
            }
            scale = next;
        }
        total
    }

    /// 生成を別threadへ送り、すぐPendingを返す。paint/scrubは待たない。
    pub fn spawn(pcm: Arc<PcmCache>) -> Arc<Self> {
        let peaks = Arc::new(Self {
            state: Mutex::new(PeakState::Pending),
        });
        let weak: Weak<Self> = Arc::downgrade(&peaks);
        match std::thread::Builder::new()
            .name("motolii-waveform-peaks".into())
            .spawn(move || {
                let built = build_levels(&pcm);
                if let Some(peaks) = weak.upgrade() {
                    println!(
                        "PROBE room=waveform verdict={}",
                        if built.is_ok() { "ready" } else { "failed" }
                    );
                    *peaks.state.lock().unwrap() = match built {
                        Ok(levels) => PeakState::Ready(levels),
                        Err(err) => PeakState::Failed(err),
                    };
                }
            })
        {
            Ok(_) => {}
            Err(err) => {
                *peaks.state.lock().unwrap() =
                    PeakState::Failed(WaveformError::Worker(err.to_string()));
            }
        }
        peaks
    }

    /// Component oracle用の同期口。製品経路は[`Self::spawn`]だけを使う。
    #[doc(hidden)]
    pub fn from_pcm(pcm: &PcmCache) -> Result<Self, WaveformError> {
        Ok(Self {
            state: Mutex::new(PeakState::Ready(build_levels(pcm)?)),
        })
    }

    pub fn status(&self) -> WaveformStatus {
        match &*self.state.lock().unwrap() {
            PeakState::Pending => WaveformStatus::Pending,
            PeakState::Ready(_) => WaveformStatus::Ready,
            PeakState::Failed(err) => WaveformStatus::Failed(err.clone()),
        }
    }

    pub fn resident_bytes(&self) -> Option<usize> {
        match &*self.state.lock().unwrap() {
            PeakState::Ready(levels) => Some(levels.resident_bytes),
            PeakState::Pending | PeakState::Failed(_) => None,
        }
    }

    pub fn columns(&self, start_sec: f64, end_sec: f64, pixels_per_sec: f64) -> Option<Vec<PeakColumn>> {
        let state = self.state.lock().unwrap();
        let PeakState::Ready(levels) = &*state else {
            return None;
        };
        if !start_sec.is_finite()
            || !end_sec.is_finite()
            || !pixels_per_sec.is_finite()
            || pixels_per_sec <= 0.0
            || end_sec <= start_sec
        {
            return Some(Vec::new());
        }

        let desired = (f64::from(levels.sample_rate) / pixels_per_sec)
            .floor()
            .max(f64::from(BASE_SCALE)) as u32;
        let (_, level) = levels
            .levels
            .range(..=desired)
            .next_back()
            .or_else(|| levels.levels.first_key_value())?;
        let scale = level.scale().max(1) as u64;
        let start_frame = seconds_to_frame_floor(start_sec, levels.sample_rate);
        let end_frame = seconds_to_frame_ceil(end_sec, levels.sample_rate).min(levels.frame_count);
        let start = (start_frame / scale).min(u64::from(level.length())) as i32;
        let end = end_frame
            .div_ceil(scale)
            .min(u64::from(level.length())) as i32;
        let channel = level.channel(0).ok()?;
        let mut out = Vec::with_capacity(end.saturating_sub(start) as usize);
        for index in start..end {
            let min = normalize_peak(channel.min_sample(index).ok()?);
            let max = normalize_peak(channel.max_sample(index).ok()?);
            out.push(PeakColumn {
                at_sec: index as f64 * level.scale() as f64 / level.sample_rate() as f64,
                min,
                max,
            });
        }
        Some(out)
    }

    #[doc(hidden)]
    pub fn deterministic_bytes(&self) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();
        let PeakState::Ready(levels) = &*state else {
            return None;
        };
        let mut out = Vec::with_capacity(levels.resident_bytes);
        for (scale, level) in &levels.levels {
            out.extend_from_slice(&scale.to_le_bytes());
            out.extend_from_slice(level.as_bytes());
        }
        Some(out)
    }
}

#[derive(Clone, Debug)]
pub struct WaveformTrack {
    pub layer: LayerId,
    pub peaks: Arc<WaveformPeaks>,
    pub timeline_start: RationalTime,
    pub timeline_duration: RationalTime,
    pub time_map: TimeMap,
}

impl WaveformTrack {
    pub fn columns(
        &self,
        view_start_sec: f64,
        view_end_sec: f64,
        pixels_per_sec: f64,
    ) -> Option<Vec<PeakColumn>> {
        let timeline_start = self.timeline_start.as_seconds_f64();
        let timeline_end = timeline_start + self.timeline_duration.as_seconds_f64();
        let comp_start = view_start_sec.max(timeline_start);
        let comp_end = view_end_sec.min(timeline_end);
        if comp_end <= comp_start {
            return Some(Vec::new());
        }

        let source_zero = self.time_map.try_map(RationalTime::ZERO).ok()?.as_seconds_f64();
        let speed = self.time_map.speed_num() as f64 / self.time_map.speed_den() as f64;
        if !speed.is_finite() || speed <= 0.0 {
            return Some(Vec::new());
        }
        let source_start = source_zero + (comp_start - timeline_start) * speed;
        let source_end = source_zero + (comp_end - timeline_start) * speed;
        let source_pps = pixels_per_sec / speed;
        let mut columns = self.peaks.columns(source_start, source_end, source_pps)?;
        for column in &mut columns {
            column.at_sec = timeline_start + (column.at_sec - source_zero) / speed;
        }
        columns.retain(|column| column.at_sec >= comp_start && column.at_sec < comp_end);
        Some(columns)
    }
}

fn build_levels(pcm: &PcmCache) -> Result<PeakLevels, WaveformError> {
    let format = pcm.format();
    if format.channels == 0 {
        return Err(WaveformError::NoChannels);
    }
    if format.sample_rate > i32::MAX as u32 {
        return Err(WaveformError::Upstream("sample rate exceeds waveform-data range".into()));
    }
    let frame_count = pcm.frame_count();
    if frame_count > i32::MAX as u64 {
        return Err(WaveformError::TooLong { frames: frame_count });
    }
    let interleaved = pcm
        .read_frames(0, frame_count as usize)
        .map_err(|err| WaveformError::Upstream(err.to_string()))?;
    let channels = format.channels as usize;
    let mut mono = Vec::with_capacity(frame_count as usize);
    for frame in interleaved.chunks_exact(channels) {
        let sum = frame
            .iter()
            .map(|sample| {
                if sample.is_finite() {
                    sample.clamp(-1.0, 1.0) as f64
                } else {
                    0.0
                }
            })
            .sum::<f64>();
        mono.push((sum / channels as f64).clamp(-1.0, 1.0) as f32);
    }
    let channel_refs: [&[f32]; 1] = [&mono];
    let buffer = waveform_data::generate_waveform_data(&waveform_data::GenerateOptions {
        scale: BASE_SCALE as i32,
        bits: 16,
        amplitude_scale: 1.0,
        split_channels: false,
        length: frame_count as i32,
        sample_rate: format.sample_rate as i32,
        channels: &channel_refs,
    });
    let base = waveform_data::WaveformData::from_binary(buffer)
        .map_err(|err| WaveformError::Upstream(err.to_string()))?;
    let mut levels = BTreeMap::new();
    levels.insert(BASE_SCALE, base.clone());
    let mut scale = BASE_SCALE;
    while u64::from(scale) < frame_count.max(u64::from(BASE_SCALE)) {
        let next = scale.saturating_mul(2);
        if next == scale || next > i32::MAX as u32 {
            break;
        }
        let level = base
            .resample(waveform_data::Resample::Scale(f64::from(next)))
            .map_err(|err| WaveformError::Upstream(err.to_string()))?;
        levels.insert(next, level);
        scale = next;
    }
    let resident_bytes = levels.values().map(|level| level.as_bytes().len()).sum();
    Ok(PeakLevels {
        sample_rate: format.sample_rate,
        frame_count,
        levels,
        resident_bytes,
    })
}

fn seconds_to_frame_floor(seconds: f64, sample_rate: u32) -> u64 {
    if seconds <= 0.0 {
        0
    } else {
        (seconds * f64::from(sample_rate)).floor() as u64
    }
}

fn seconds_to_frame_ceil(seconds: f64, sample_rate: u32) -> u64 {
    if seconds <= 0.0 {
        0
    } else {
        (seconds * f64::from(sample_rate)).ceil() as u64
    }
}

fn normalize_peak(value: i32) -> f32 {
    if value < 0 {
        (value as f32 / 32768.0).max(-1.0)
    } else {
        (value as f32 / 32767.0).min(1.0)
    }
}
