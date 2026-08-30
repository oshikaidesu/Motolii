
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

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;

    fn push_all(producer: &mut rtrb::Producer<f32>, samples: &[f32]) {
        for &s in samples {
            producer.push(s).expect("capacity 十分なテスト用データ");
        }
    }

    #[test]
    fn full_pop_advances_supplied_only() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(8);
        push_all(&mut producer, &[1.0, -1.0, 2.0, -2.0]); // 2 frames, stereo
        let counters = PlaybackCounters::default();
        let mut dst = [0.0; 4];

        fill_or_silence(&mut consumer, &mut dst, 2, &counters);

        assert_eq!(dst, [1.0, -1.0, 2.0, -2.0]);
        assert_eq!(counters.frames_supplied(), 2);
        assert_eq!(counters.silence_frames(), 0);
        assert_eq!(counters.underrun_events(), 0);
    }

    #[test]
    fn underrun_fills_silence_and_does_not_advance_supplied_frames() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(8);
        push_all(&mut producer, &[0.5]); // mono, 1 frame available
        let counters = PlaybackCounters::default();
        let mut dst = [1.0, 1.0]; // 事前に非0を入れて上書きを検証

        fill_or_silence(&mut consumer, &mut dst, 1, &counters);

        assert_eq!(dst, [0.5, 0.0]);
        assert_eq!(
            counters.frames_supplied(),
            1,
            "無音補填分は論理sample位置に加算されない"
        );
        assert_eq!(counters.silence_frames(), 1);
        assert_eq!(counters.underrun_events(), 1);
    }

    #[test]
    fn empty_ring_is_pure_silence_and_one_underrun_event() {
        let (_producer, mut consumer) = RingBuffer::<f32>::new(4);
        let counters = PlaybackCounters::default();
        let mut dst = [9.0; 4];

        fill_or_silence(&mut consumer, &mut dst, 2, &counters);

        assert_eq!(dst, [0.0; 4]);
        assert_eq!(counters.frames_supplied(), 0);
        assert_eq!(counters.silence_frames(), 2);
        assert_eq!(counters.underrun_events(), 1);
    }

    #[test]
    fn mismatched_frame_alignment_is_ignored_without_panic() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(4);
        push_all(&mut producer, &[1.0, 2.0]);
        let counters = PlaybackCounters::default();
        let mut dst = [0.0; 3]; // channels=2に対し非倍数長

        fill_or_silence(&mut consumer, &mut dst, 2, &counters);

        assert_eq!(dst, [0.0; 3], "設定ミスは無視するだけでdstを書き換えない");
        assert_eq!(counters.frames_supplied(), 0);
    }
}
