//! VSM-A7: 現行BPMを既存DataTrack→DocParam::Dataだけへ結線する意味fixture。
//!
//! BeatEvents、consumer plugin、公開port、Document schemaは追加しない。

use motolii_core::{Fps, Quality, RationalTime};
use motolii_doc::param_eval::eval_f64;
use motolii_doc::{Bpm, DocParam, DocValue, Document, ResolvedLayerParams};
use motolii_eval::{DataTrack, DataTrackId, DataTracks, Value};

fn beat_position(t: RationalTime, bpm: Bpm) -> f64 {
    let beat = bpm.try_beat_duration().unwrap();
    let num = (t.num() as i128) * (beat.den() as i128);
    let den = (t.den() as i128) * (beat.num() as i128);
    num as f64 / den as f64
}

fn beat_position_track(
    doc: &Document,
    start: RationalTime,
    sample_rate: Fps,
    sample_count: usize,
) -> DataTrack {
    let values = (0..sample_count)
        .map(|i| {
            let offset = RationalTime::try_from_frame(i as i64, sample_rate).unwrap();
            let t = start.try_add(offset).unwrap();
            Value::F64(beat_position(t, doc.bpm))
        })
        .collect();
    DataTrack {
        start,
        sample_rate,
        values,
    }
}

fn approx(got: f64, want: f64) {
    let delta = (got - want).abs();
    assert!(delta < 1e-11, "got {got}, want {want}, delta {delta}");
}

#[test]
fn fractional_bpm_and_ntsc_rate_produce_deterministic_positions() {
    let mut doc = Document::new_current();
    // 120.35 BPM。小数をf64へ焼かず、既存Bpmの有理数で保持する。
    doc.bpm = Bpm::try_new(12_035, 100).unwrap();
    let rate = Fps::try_new(30_000, 1_001).unwrap();
    let start = RationalTime::try_new(-7, 10).unwrap();

    let a = beat_position_track(&doc, start, rate, 640);
    let b = beat_position_track(&doc, start, rate, 640);
    assert_eq!(a, b);

    for i in [0usize, 1, 137, 511, 639] {
        let t = start
            .try_add(RationalTime::try_from_frame(i as i64, rate).unwrap())
            .unwrap();
        approx(a.values[i].as_f64().unwrap(), beat_position(t, doc.bpm));
    }
}

#[test]
fn doc_param_reads_the_same_beat_position_in_any_seek_order() {
    let mut doc = Document::new_current();
    doc.bpm = Bpm::try_new(12_035, 100).unwrap();
    let rate = Fps::try_new(30_000, 1_001).unwrap();
    let start = RationalTime::from_seconds(-1);
    let track_id = DataTrackId("fixture.beat_position".into());
    let mut tracks = DataTracks::new();
    tracks.insert(
        track_id.clone(),
        beat_position_track(&doc, start, rate, 900),
    );
    let param = DocParam::Data {
        track: track_id,
        fallback: DocValue::F64(-1.0),
    };
    let resolved = ResolvedLayerParams::default();
    let times = [
        RationalTime::ZERO,
        RationalTime::try_new(1, 3).unwrap(),
        RationalTime::try_new(7, 10).unwrap(),
        RationalTime::try_new(10, 1).unwrap(),
        RationalTime::try_new(1234, 125).unwrap(),
    ];

    let forward: Vec<_> = times
        .iter()
        .map(|&t| eval_f64(&param, t, &tracks, &resolved).unwrap())
        .collect();
    let reverse: Vec<_> = times
        .iter()
        .rev()
        .map(|&t| eval_f64(&param, t, &tracks, &resolved).unwrap())
        .collect();

    for (i, &t) in times.iter().enumerate() {
        approx(forward[i], beat_position(t, doc.bpm));
        approx(reverse[times.len() - 1 - i], forward[i]);
    }
}

#[test]
fn quality_does_not_change_data_track_meaning_or_document_bytes() {
    let mut doc = Document::new_current();
    doc.bpm = Bpm::try_new(90, 1).unwrap();
    let before = serde_json::to_vec(&doc).unwrap();
    let track_id = DataTrackId("fixture.beat_position".into());
    let mut tracks = DataTracks::new();
    tracks.insert(
        track_id.clone(),
        beat_position_track(&doc, RationalTime::ZERO, Fps::try_new(24, 1).unwrap(), 240),
    );
    let param = DocParam::Data {
        track: track_id,
        fallback: DocValue::F64(-1.0),
    };
    let t = RationalTime::try_new(13, 6).unwrap();

    let evaluate =
        |_quality: Quality| eval_f64(&param, t, &tracks, &ResolvedLayerParams::default()).unwrap();
    let preview = evaluate(Quality::DRAFT);
    let export = evaluate(Quality::FINAL);

    approx(preview, beat_position(t, doc.bpm));
    assert_eq!(preview, export);
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);
}
