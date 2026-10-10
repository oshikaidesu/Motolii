//! Beat times from PCM we already decoded. A rise in energy is a hit; the typical
//! gap between hits is the tempo. A gap of two tempos gets a beat in the middle.
//! Not a port of aubio or librosa: those decode again and pull their own stack.

use super::cache::PcmCache;

const HOP: usize = 1024;

pub fn beat_times(pcm: &PcmCache) -> Vec<f64> {
    let channels = pcm.format().channels as usize;
    let rate = pcm.format().sample_rate as f64;
    if channels == 0 || rate <= 0.0 { return Vec::new(); }
    let frames = pcm.frame_count() as usize;
    if frames < rate as usize / 2 { return Vec::new(); }
    let samples = pcm.samples_i16();
    let mut energy = Vec::new();
    let mut at = 0;
    while at + HOP <= frames {
        let start = at * channels;
        let end = (at + HOP) * channels;
        if end > samples.len() { break; }
        let mut sum = 0.0f64;
        for sample in &samples[start..end] {
            let v = *sample as f64;
            sum += v * v;
        }
        energy.push((sum / (HOP * channels) as f64) as f32);
        at += HOP;
    }
    if energy.len() < 8 { return Vec::new(); }
    let mut onset = vec![0.0f32; energy.len()];
    if energy[0] > energy[1] { onset[0] = energy[0]; }
    for i in 1..energy.len() {
        let rise = energy[i] - energy[i - 1];
        if rise > 0.0 { onset[i] = rise; }
    }
    let mean = onset.iter().copied().sum::<f32>() / onset.len() as f32;
    let var = onset.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / onset.len() as f32;
    let floor = mean + 0.5 * var.sqrt();
    let min_gap = ((0.2 * rate) / HOP as f64).round().max(1.0) as usize;
    let mut peaks = Vec::new();
    for i in 0..onset.len() {
        let left = if i == 0 { 0.0 } else { onset[i - 1] };
        let right = onset.get(i + 1).copied().unwrap_or(0.0);
        if onset[i] >= floor && onset[i] >= left && onset[i] > right && peaks.last().is_none_or(|prev: &usize| i - prev >= min_gap) {
            peaks.push(i);
        }
    }
    peaks = on_the_tempo(&peaks, rate);
    peaks.into_iter().map(|hop| hop as f64 * HOP as f64 / rate).collect()
}

/// Keep hits whose spacing is a tempo (about 0.25–1.5s). A hole about twice
/// that wide is a beat the rise missed, so one is put in the middle.
fn on_the_tempo(peaks: &[usize], rate: f64) -> Vec<usize> {
    if peaks.len() < 2 { return Vec::new(); }
    let mut gaps: Vec<usize> = peaks.windows(2).map(|pair| pair[1] - pair[0]).collect();
    gaps.sort_unstable();
    let median = gaps[gaps.len() / 2];
    let shortest = ((0.25 * rate) / HOP as f64).round() as usize;
    let longest = ((1.5 * rate) / HOP as f64).round() as usize;
    if median < shortest.max(1) || median > longest { return Vec::new(); }
    let mut beats = vec![peaks[0]];
    for pair in peaks.windows(2) {
        let gap = pair[1] - pair[0];
        let steps = (gap as f64 / median as f64).round() as usize;
        if (2..=3).contains(&steps) {
            for step in 1..steps {
                beats.push(pair[0] + gap * step / steps);
            }
        }
        beats.push(pair[1]);
    }
    beats
}

#[cfg(test)]
mod tests {
    use super::super::cache::{PcmCache, PcmFormat};
    use super::beat_times;

    fn clicks(beats: usize, gap: f64) -> PcmCache {
        let rate = 48_000u32;
        let frames = (rate as f64 * gap * beats as f64) as usize + rate as usize;
        let mut samples = vec![0i16; frames * 2];
        for beat in 0..beats {
            let at = (beat as f64 * gap * rate as f64) as usize;
            for k in 0..180 {
                if at + k >= frames { break; }
                let s = (12_000.0 * (1.0 - k as f64 / 180.0)) as i16;
                samples[(at + k) * 2] = s;
                samples[(at + k) * 2 + 1] = s;
            }
        }
        PcmCache::from_interleaved_i16(samples, PcmFormat { channels: 2, sample_rate: rate }).unwrap()
    }

    #[test]
    fn silence_has_no_beats() {
        let pcm = PcmCache::from_interleaved_i16(vec![0i16; 48_000 * 2 * 2], PcmFormat { channels: 2, sample_rate: 48_000 }).unwrap();
        assert!(beat_times(&pcm).is_empty());
    }

    #[test]
    fn a_click_every_half_second_lands_on_those_clicks() {
        let times = beat_times(&clicks(8, 0.5));
        assert!(times.len() >= 6 && times.len() <= 10, "{times:?}");
        for beat in 0..8 {
            let want = beat as f64 * 0.5;
            assert!(times.iter().any(|t| (t - want).abs() < 0.08), "missing {want} in {times:?}");
        }
    }
}
