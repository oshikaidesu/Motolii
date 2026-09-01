
use crate::render::audio::clock::PlaybackCounters;

pub fn fill_or_silence(
    consumer: &mut rtrb::Consumer<f32>,
    dst: &mut [f32],
    channels: usize,
    counters: &PlaybackCounters,
) {
    if dst.is_empty() || channels == 0 || !dst.len().is_multiple_of(channels) {
        return;
    }
    let frames_req = dst.len() / channels;
    let available_frames = consumer.slots() / channels;
    let frames_to_pop = frames_req.min(available_frames);
    let samples_to_pop = frames_to_pop * channels;

    for slot in dst[..samples_to_pop].iter_mut() {
        *slot = consumer
            .pop()
            .expect("slots() が数えた範囲内なので pop は必ず成功する");
    }
    if samples_to_pop < dst.len() {
        dst[samples_to_pop..].fill(0.0);
    }

    let missing_frames = (frames_req - frames_to_pop) as u64;
    counters.record_block(frames_to_pop as u64, missing_frames);
}
