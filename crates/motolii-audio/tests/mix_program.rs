//! AG-2完了条件: Document由来AudioProgram、MixProducer、seek非block、overlap不変。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use motolii_audio::{
    channel, fill_or_silence, mix_audio, program_from_sources, AudioMeter, AudioProgram,
    MixProducer, MixSource, PcmCache, PcmFormat, PlaybackCounters, CANONICAL_CHANNELS,
    CANONICAL_SAMPLE_RATE,
};
use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, AudioComponent, AudioOutOfRange, Clip, ClipSource, Document, ItemEnvelope,
    Soundtrack, Track, TrackItem,
};

use support::write_pcm16_wav;

fn stereo_const(frames: usize, l: f32, r: f32) -> Arc<PcmCache> {
    let mut samples = Vec::with_capacity(frames * 2);
    for _ in 0..frames {
        samples.push(l);
        samples.push(r);
    }
    Arc::new(
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 2,
                sample_rate: CANONICAL_SAMPLE_RATE,
            },
        )
        .unwrap(),
    )
}

fn identity_source(pcm: Arc<PcmCache>, frames: u64, gain: f64) -> MixSource {
    MixSource {
        pcm,
        timeline_start: RationalTime::ZERO,
        timeline_duration: RationalTime::try_new(frames as i64, CANONICAL_SAMPLE_RATE as i64)
            .unwrap(),
        time_map: TimeMap::IDENTITY,
        gain: motolii_doc::DocParam::const_f64(gain),
        out_of_range: AudioOutOfRange::Silence,
        enabled: true,
    }
}

#[test]
fn soundtrack_and_clip_audio_mix_together() {
    let dir = tempfile_dir("ag2_mix");
    let bed = dir.join("bed.wav");
    let clip_wav = dir.join("clip.wav");
    // 0.25 / 0.0 の定数ステレオ(PCM16 ≈ 0.25)
    write_pcm16_wav(&bed, 48_000, 2, &vec![i16::MAX / 4, 0].repeat(480));
    write_pcm16_wav(&clip_wav, 48_000, 2, &vec![0, i16::MAX / 4].repeat(480));

    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(1, 1).unwrap();
    let bed_id = AssetId::from_raw(0);
    let clip_id = AssetId::from_raw(1);
    doc.assets
        .insert(Asset {
            id: bed_id,
            name: "bed".into(),
            asset_type: "audio/wav".into(),
            content_hash: "sha256:bed".into(),
            path_absolute: Some(bed.to_string_lossy().into()),
            path_project_relative: None,
            file_name: Some("bed.wav".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    doc.assets
        .insert(Asset {
            id: clip_id,
            name: "clip".into(),
            asset_type: "audio/wav".into(),
            content_hash: "sha256:clip".into(),
            path_absolute: Some(clip_wav.to_string_lossy().into()),
            path_project_relative: None,
            file_name: Some("clip.wav".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    doc.soundtrack = Some(Soundtrack::try_new(bed_id, RationalTime::ZERO, 1.0).unwrap());

    let layer = doc.layers.allocate("A").unwrap();
    let track_id = doc.track_ids.allocate("A1").unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 100).unwrap(),
            time_map: TimeMap::IDENTITY,
            source: ClipSource::Asset {
                asset: clip_id,
                video: None,
                audio: vec![AudioComponent::ordinal(0)],
            },
        })],
    });
    // video:None + audio は min_reader_version が必要
    doc.min_reader_version = 3;

    let mut caches = HashMap::new();
    let program = AudioProgram::from_document(&doc, None, &mut caches).unwrap();
    assert_eq!(program.sources().len(), 2);

    let (out, _) = program.mix_audio(0, 10, None).unwrap();
    // bed ≈ (0.25,0) + clip ≈ (0,0.25)
    assert!(out[0] > 0.20 && out[0] < 0.30);
    assert!(out[1] > 0.20 && out[1] < 0.30);
}

#[test]
fn same_track_overlap_remains_document_validate_concern() {
    // A4はDocument validate側。mixerはsource列を加算するだけで重なり許可を発明しない。
    // 別Track上の同時発音は許可(完了条件: Soundtrack+別Track Clip)。
    let a = stereo_const(8, 0.1, 0.1);
    let b = stereo_const(8, 0.2, 0.2);
    let mut s0 = identity_source(a, 8, 1.0);
    let mut s1 = identity_source(b, 8, 1.0);
    // 同一timeline区間に載る2source(=別Track想定)
    s0.timeline_start = RationalTime::ZERO;
    s1.timeline_start = RationalTime::ZERO;
    let (out, _) = mix_audio(&[s0, s1], 1.0, 0, 1, None).unwrap();
    assert_eq!(out, vec![0.3, 0.3]);
}

#[test]
fn mix_producer_feeds_ring_while_callback_only_reads() {
    let program = Arc::new(program_from_sources(
        vec![identity_source(stereo_const(4_096, 0.5, -0.5), 4_096, 1.0)],
        1.0,
    ));
    let (ring_prod, ring_cons) = channel(CANONICAL_CHANNELS, 2_048).unwrap();
    let meter = Arc::new(AudioMeter::new());
    let producer =
        MixProducer::spawn(Arc::clone(&program), ring_prod, 0, Some(Arc::clone(&meter))).unwrap();

    let counters = PlaybackCounters::default();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut heard = false;
    while Instant::now() < deadline {
        let mut buf = vec![0.0f32; 256 * 2];
        fill_or_silence(&ring_cons, &mut buf, &counters);
        if buf.iter().any(|s| *s != 0.0) {
            heard = true;
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(heard, "mix producer never delivered audible samples");
    assert!(meter.snapshot().peak_l > 0.0 || meter.snapshot().peak_r > 0.0);
    producer.stop();
}

#[test]
fn hundred_seeks_do_not_block_callback_path() {
    let program = Arc::new(program_from_sources(
        vec![identity_source(
            stereo_const(48_000, 0.25, 0.25),
            48_000,
            1.0,
        )],
        1.0,
    ));

    let callback_max_ns = AtomicU64::new(0);
    let counters = PlaybackCounters::default();
    for seek in 0..100u64 {
        // seek = MixProducer再起動。callbackはringを読むだけでdecode/mixを待たない。
        let (ring_prod, ring_cons) = channel(CANONICAL_CHANNELS, 4_096).unwrap();
        let producer =
            MixProducer::spawn(Arc::clone(&program), ring_prod, seek * 100, None).unwrap();

        let t0 = Instant::now();
        let mut buf = vec![0.0f32; 128 * 2];
        fill_or_silence(&ring_cons, &mut buf, &counters);
        let elapsed = t0.elapsed().as_nanos() as u64;
        callback_max_ns.fetch_max(elapsed, Ordering::Relaxed);
        producer.stop();
    }
    assert!(
        callback_max_ns.load(Ordering::Relaxed) < 10_000_000,
        "callback path blocked: {} ns",
        callback_max_ns.load(Ordering::Relaxed)
    );
}

#[test]
fn metering_snapshots_are_lock_free_under_updates() {
    let meter = Arc::new(AudioMeter::new());
    let program = Arc::new(program_from_sources(
        vec![identity_source(stereo_const(8_000, 1.5, -1.25), 8_000, 1.0)],
        1.0,
    ));
    let (ring_prod, ring_cons) = channel(CANONICAL_CHANNELS, 2_048).unwrap();
    let producer =
        MixProducer::spawn(Arc::clone(&program), ring_prod, 0, Some(Arc::clone(&meter))).unwrap();

    let mut max_ns = 0u64;
    for _ in 0..100 {
        let t0 = Instant::now();
        let _ = meter.snapshot();
        max_ns = max_ns.max(t0.elapsed().as_nanos() as u64);
        let mut buf = vec![0.0f32; 64 * 2];
        fill_or_silence(&ring_cons, &mut buf, &PlaybackCounters::default());
    }
    producer.stop();
    assert!(meter.snapshot().clipped);
    assert!(max_ns < 5_000_000, "snapshot blocked: {max_ns} ns");
}

#[test]
fn gap_silence_is_not_ring_underrun() {
    let mut source = identity_source(stereo_const(1, 1.0, 1.0), 1, 1.0);
    source.timeline_start = RationalTime::try_new(10, CANONICAL_SAMPLE_RATE as i64).unwrap();
    let (out, report) = mix_audio(&[source], 1.0, 0, 10, None).unwrap();
    assert!(out[..20].iter().all(|s| *s == 0.0));
    assert_eq!(report.silence_frames, 10);
    // underflowカウンタはring経路のPlaybackCounters。mixの正規silenceとは別。
    let counters = PlaybackCounters::default();
    assert_eq!(counters.underrun_events(), 0);
}

fn tempfile_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
