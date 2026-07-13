//! D4完了条件: producer/consumer所有権分離・callback非blocking構造の証明、
//! 十分な供給下でのアンダーラン0シミュレーション、アンダーラン時の無音充填+
//! 不足フレーム数+実sample進行数非加算のテスト。
//!
//! ハードウェア(cpal device)には依存しない — 「hardware smokeだけを完了証跡に
//! しない」(#123完了条件)。ハードウェア専用テストは`tests/device_smoke.rs`に分離し、
//! 既定では`#[ignore]`する。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_audio::{
    channel, fill_or_silence, AudioProducer, PcmCache, PcmFormat, PlaybackCounters,
};

fn sine_cache(frames: usize, rate: u32) -> Arc<PcmCache> {
    let mut samples = Vec::with_capacity(frames);
    for i in 0..frames {
        let t = i as f32 / rate as f32;
        samples.push((t * 440.0 * std::f32::consts::TAU).sin());
    }
    Arc::new(
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 1,
                sample_rate: rate,
            },
        )
        .expect("valid cache"),
    )
}

/// 決定的(実時間非依存)ティックシミュレーション: 各tickで「プロデューサが
/// 空き分だけ詰め直し→コンシューマが固定量読む」を繰り返す。ring容量が
/// 1tick分の消費量以上なら、供給が追いつけない状況は原理的に起きない
/// (=「十分なproducer供給下でunderrun=0」を実時間sleepに頼らず証明する)。
fn simulate_ticks(
    cache: &PcmCache,
    ring_capacity_frames: usize,
    chunk_frames: usize,
    producer_push_cap: usize,
) -> (usize, PlaybackCounters) {
    let format = cache.format();
    let (ring_prod, ring_cons) = channel(format.channels, ring_capacity_frames).unwrap();
    let counters = PlaybackCounters::default();
    let total = cache.frame_count();
    let mut playhead = 0u64;
    // 要求した総フレーム数(=消費側が呼んだ`fill_or_silence`のフレーム数の合計)。
    // 最終tickは`total`に届くよう量を切るため、tick数×chunk_framesとは一致しない。
    let mut requested = 0usize;

    while requested < total as usize {
        // プロデューサ側: 1tickにpush出来る量を`producer_push_cap`で制限する
        // (十分供給シナリオでは capacity 全体、供給不足シナリオでは小さい値)。
        let free = ring_prod.free_frames().min(producer_push_cap);
        let remaining_source = (total - playhead) as usize;
        let push_frames = free.min(remaining_source);
        if push_frames > 0 {
            let chunk = cache.read_frames(playhead, push_frames).unwrap();
            let pushed = ring_prod.push_frames(chunk);
            playhead += pushed as u64;
        }

        // コンシューマ側: 固定量を要求する(実際のコールバックのバッファサイズに相当)。
        let want = chunk_frames.min(total as usize - requested);
        let mut buf = vec![0.0f32; want * format.channels as usize];
        fill_or_silence(&ring_cons, &mut buf, &counters);
        requested += want;
    }
    (requested, counters)
}

#[test]
fn sufficient_supply_yields_zero_underruns_deterministic() {
    let cache = sine_cache(48_000, 48_000);
    // ring容量(4096) >= 1tick消費量(256) なので供給不足は原理的に起きない。
    let (_total_requested, counters) = simulate_ticks(&cache, 4_096, 256, 4_096);
    assert_eq!(counters.underrun_events(), 0);
    assert_eq!(counters.silence_frames(), 0);
    assert_eq!(counters.frames_supplied(), cache.frame_count());
}

#[test]
fn insufficient_supply_reports_underrun_silence_without_advancing_logical_position() {
    let cache = sine_cache(48_000, 48_000);
    let chunk = 256usize;
    // producer_push_capをconsumerの要求量より小さくし、供給が追いつけない状況を
    // 決定的に作る(実時間sleepに依存しない)。
    let (total_requested, counters) = simulate_ticks(&cache, 4_096, chunk, 64);

    assert!(counters.underrun_events() > 0, "must observe underruns");
    assert!(counters.silence_frames() > 0, "must observe silence fill");
    // 要求した総フレーム数 = 実供給 + 無音補填(不足)。この等式が破れていたら
    // カウンタの二重計上/取り漏らしがある。
    let total_requested = total_requested as u64;
    assert_eq!(
        counters.frames_supplied() + counters.silence_frames(),
        total_requested
    );
    // 核心: 論理sample位置(frames_supplied)は無音補填分だけ遅れる —
    // 無音で埋めた分を「進んだ」ことにしない(D4完了条件)。
    assert!(counters.frames_supplied() < total_requested);
    assert!(counters.frames_supplied() < cache.frame_count());
}

/// producer/consumer所有権分離: 実スレッド分離(`AudioProducer`)+メインスレッドの
/// コンシューマで、供給が十分(ring容量・producerポーリングが実時間消費より高速)なら
/// アンダーランが起きないことを確認する。余裕を大きく取り、CI環境差でのフレーク耐性を持たせる。
#[test]
fn producer_thread_keeps_realtime_consumer_fed_without_underrun() {
    let rate = 48_000u32;
    let cache = sine_cache(rate as usize, rate); // 1秒分
    let chunk_frames = 480usize; // 10ms相当
    let (ring_prod, ring_cons) = channel(1, (rate as usize / 4).max(4_096)).unwrap(); // 250ms容量
    let counters = PlaybackCounters::default();

    let producer = AudioProducer::spawn(Arc::clone(&cache), ring_prod, 0).expect("spawn producer");

    // 実コールバックが飢えないよう、消費開始前に短く先読みを待つ(D4契約と同じ手順)。
    let prefill_target = chunk_frames * 4;
    let prefill_deadline = Instant::now() + Duration::from_secs(5);
    while ring_cons.buffered_frames() < prefill_target {
        assert!(Instant::now() < prefill_deadline, "prefill timed out");
        std::thread::sleep(Duration::from_millis(1));
    }

    let total = cache.frame_count();
    let tick = Duration::from_secs_f64(chunk_frames as f64 / rate as f64);
    let mut requested = 0u64;
    while requested < total {
        let want = chunk_frames.min((total - requested) as usize);
        let mut buf = vec![0.0f32; want];
        fill_or_silence(&ring_cons, &mut buf, &counters);
        requested += want as u64;
        std::thread::sleep(tick);
    }

    producer.stop();
    assert_eq!(
        counters.underrun_events(),
        0,
        "unexpected underrun under generous margins"
    );
    assert_eq!(counters.frames_supplied(), total);
}

/// producer/consumer所有権分離の構造証明(実行時): `RingProducer`は`AudioProducer`
/// スレッドへ`move`済みで、呼び出し側はもう触れない(型が返さない)。`RingConsumer`は
/// 呼び出し側スレッドだけが所有し続ける。両者は`channel()`の返り値以降、一切の
/// 共有参照なしにリングだけで通信する。
#[test]
fn producer_and_consumer_never_share_a_reference_after_split() {
    let cache = sine_cache(1_000, 48_000);
    // ring容量はcache全体を保持できるだけ確保する(このテストの主眼はスレッド分離の
    // 構造であって、部分供給の挙動ではない — それは他のテストが担う)。
    let (ring_prod, ring_cons) = channel(1, 2_048).unwrap();
    // ring_prodはここでAudioProducerへ所有権が移り、以後この関数から触れない
    // (`ring_prod`という変数はmoveされ、コンパイラがuse-after-moveを拒否する)。
    let producer = AudioProducer::spawn(Arc::clone(&cache), ring_prod, 0).unwrap();

    let counters = PlaybackCounters::default();
    let deadline = Instant::now() + Duration::from_secs(5);
    while ring_cons.buffered_frames() < cache.frame_count() as usize {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut buf = vec![0.0f32; cache.frame_count() as usize];
    fill_or_silence(&ring_cons, &mut buf, &counters);
    assert_eq!(counters.frames_supplied(), cache.frame_count());
    producer.stop();
}
