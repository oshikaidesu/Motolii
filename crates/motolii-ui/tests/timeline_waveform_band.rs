//! soundtrack の波形帯: 縮約・viewport 写像・帯の有無。
//!
//! MV を作る人にとって波形は同期の地図そのものである。地図が信用できる条件は
//! 3つで、ここはその3つだけを判定する。
//!
//! 1. **縮約が嘘をつかない** — 既知 PCM を段へ畳んでも peak は消えない
//! 2. **写像がルーラと同じ** — ズーム・パン・`start_offset` が px を決める
//! 3. **soundtrack が無ければ帯ごと出ない** — 空の帯を置いて場所を取らない
//!
//! decode / 生成の座席側(cache 共有・別 thread)は `waveform_band` の unit 側。

use motolii_audio::{PcmCache, PcmFormat};
use motolii_ui::timeline_editor::waveform_band::{band_height, WaveformPeaks, WaveformWindow};

/// 正準に合わせた 48kHz。秒 ↔ フレームの読み替えを暗算できる値にしておく。
const RATE: u32 = 48_000;

/// mono PCM を1本作る。
fn mono(samples: Vec<f32>) -> PcmCache {
    PcmCache::from_interleaved(
        samples,
        PcmFormat {
            channels: 1,
            sample_rate: RATE,
        },
    )
    .expect("valid PCM")
}

// ---------------------------------------------------------------------------
// 1. 縮約
// ---------------------------------------------------------------------------

/// **既知 PCM → 期待 peak。** 段0は `waveform_peaks` の bucket そのもので、
/// 上の段は隣り合う2つの max。**畳んでも peak は落ちない**(音が消えて見える
/// 波形は地図として使えない)。
#[test]
fn the_pyramid_folds_a_known_pcm_without_losing_its_peaks() {
    // 段0の bucket 幅ちょうど4つ分。bucket ごとの max が読み切れる並びにする。
    let per_bucket = WaveformPeaks::BASE_BUCKET_FRAMES as usize;
    let mut samples = Vec::new();
    for peak in [0.25f32, 0.5, 1.0, 0.75] {
        samples.push(peak);
        samples.push(-peak * 0.5);
        samples.extend(std::iter::repeat_n(0.0, per_bucket - 2));
    }
    let peaks = WaveformPeaks::build(&mono(samples)).expect("pyramid");

    assert_eq!(
        peaks.level_peaks(0),
        [0.25, 0.5, 1.0, 0.75],
        "段0は bucket ごとの絶対値 peak"
    );
    assert_eq!(peaks.level_peaks(1), [0.5, 1.0], "段1は隣り合う2つの max");
    assert_eq!(peaks.level_peaks(2), [1.0], "段2で全体の max へ畳まれる");
    assert_eq!(peaks.level_count(), 3, "1 bucket まで畳んだら止まる");
}

/// **負の側も peak である。** 片側だけ読むと、下向きに振れている所が
/// 無音に見えてしまう。
#[test]
fn the_peak_is_the_absolute_value_not_the_positive_side() {
    let per_bucket = WaveformPeaks::BASE_BUCKET_FRAMES as usize;
    let mut samples = vec![0.0f32; per_bucket];
    samples[3] = -0.8;
    let peaks = WaveformPeaks::build(&mono(samples)).expect("pyramid");
    assert_eq!(peaks.level_peaks(0), [0.8]);
}

// ---------------------------------------------------------------------------
// 2. viewport 写像
// ---------------------------------------------------------------------------

/// 0.5s–0.6s だけ鳴っている1秒の音を作る。
fn burst_one_second() -> PcmCache {
    let mut samples = vec![0.0f32; RATE as usize];
    for sample in samples
        .iter_mut()
        .take((RATE as usize * 6) / 10)
        .skip(RATE as usize / 2)
    {
        *sample = 1.0;
    }
    mono(samples)
}

/// 列は必ず `width_px` 本ぶん出る(足りない所は 0 = 無音)。
#[test]
fn every_pixel_column_gets_a_value() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    let mut columns = Vec::new();
    peaks.columns(
        WaveformWindow {
            view_start: 0.0,
            view_span: 4.0,
            width_px: 200.0,
            start_offset: 0.0,
        },
        &mut columns,
    );
    assert_eq!(columns.len(), 200, "1列=1px");
    assert!(
        columns[80..].iter().all(|v| *v == 0.0),
        "音は1秒で終わる。以降は無音の帯が見える"
    );
}

/// **引いた窓**: 1秒を100pxで見ると、0.5–0.6s の音は 50–60 列に立つ。
#[test]
fn a_zoomed_out_window_puts_the_burst_at_its_share_of_the_width() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    let mut columns = Vec::new();
    peaks.columns(
        WaveformWindow {
            view_start: 0.0,
            view_span: 1.0,
            width_px: 100.0,
            start_offset: 0.0,
        },
        &mut columns,
    );
    assert!(
        columns[50..60].iter().all(|v| (*v - 1.0).abs() < 1e-6),
        "0.5–0.6s は 50–60 列: {:?}",
        &columns[45..65]
    );
    // 縮約の bucket は1列より細かいとは限らないので、境の1列だけは滲む。
    assert!(
        columns[..49].iter().all(|v| *v == 0.0),
        "頭は無音: {:?}",
        &columns[..49]
    );
    assert!(
        columns[61..].iter().all(|v| *v == 0.0),
        "尻は無音: {:?}",
        &columns[61..]
    );
}

/// **寄った窓**: 同じ音を 0.45–0.65s の窓で見ると、音の頭は窓の 1/4 に来る。
/// **時間 ↔ x はルーラと同じ1本の換算**であって、帯だけ別の目盛を持たない。
#[test]
fn zooming_in_keeps_the_burst_at_the_same_time_not_the_same_pixel() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    let mut columns = Vec::new();
    peaks.columns(
        WaveformWindow {
            view_start: 0.45,
            view_span: 0.2,
            width_px: 200.0,
            start_offset: 0.0,
        },
        &mut columns,
    );
    // 0.45 + 0.2*(x/200) = 0.5 → x = 50 / 0.6 → x = 150
    assert!(
        columns[52..148].iter().all(|v| (*v - 1.0).abs() < 1e-6),
        "0.5–0.6s は 50–150 列"
    );
    assert!(columns[..48].iter().all(|v| *v == 0.0), "窓の頭は無音");
    assert!(columns[152..].iter().all(|v| *v == 0.0), "窓の尻は無音");
}

/// **パンしても音は時刻に貼り付く。** 窓を右へ動かした分だけ列が左へ寄る。
#[test]
fn panning_the_window_slides_the_burst_by_the_same_amount() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    let mut columns = Vec::new();
    peaks.columns(
        WaveformWindow {
            view_start: 0.25,
            view_span: 1.0,
            width_px: 100.0,
            start_offset: 0.0,
        },
        &mut columns,
    );
    // 窓が 0.25s 右へ動いた = 音は 25 列ぶん左へ
    assert!(
        columns[25..35].iter().all(|v| (*v - 1.0).abs() < 1e-6),
        "0.5–0.6s は 25–35 列"
    );
    assert!(columns[..24].iter().all(|v| *v == 0.0), "頭は無音");
    assert!(columns[36..].iter().all(|v| *v == 0.0), "尻は無音");
}

/// **`start_offset` はソースin点。** timeline の 0 秒に居るのは音源の
/// `start_offset` 秒であり、波形は soundtrack の実時間位置に置かれる。
#[test]
fn the_start_offset_puts_the_wave_where_the_soundtrack_actually_sounds() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    let mut columns = Vec::new();
    // 音源の 0.4s から始める → 音源 0.5s は timeline 0.1s = 10 列
    peaks.columns(
        WaveformWindow {
            view_start: 0.0,
            view_span: 1.0,
            width_px: 100.0,
            start_offset: 0.4,
        },
        &mut columns,
    );
    assert!(
        columns[10..20].iter().all(|v| (*v - 1.0).abs() < 1e-6),
        "in点を 0.4s へ動かしたら音は 10–20 列: {:?}",
        &columns[..25]
    );
    assert!(columns[..9].iter().all(|v| *v == 0.0), "頭は無音");
    assert!(columns[21..60].iter().all(|v| *v == 0.0), "音の後ろは無音");
    assert!(
        columns[61..].iter().all(|v| *v == 0.0),
        "音源の尻(timeline 0.6s)より後ろに音は無い"
    );
}

/// **音源の外は 0。** 負の時刻も終端の後ろも、無かったことにして描かない。
#[test]
fn outside_the_source_the_band_is_silent() {
    let peaks = WaveformPeaks::build(&burst_one_second()).expect("pyramid");
    assert!((peaks.source_seconds() - 1.0).abs() < 1e-6);
    let mut columns = Vec::new();
    peaks.columns(
        WaveformWindow {
            view_start: 0.0,
            view_span: 1.0,
            width_px: 100.0,
            // in点が音源より後ろ = 何も残らない
            start_offset: 2.0,
        },
        &mut columns,
    );
    assert!(columns.iter().all(|v| *v == 0.0), "音源の外は無音");
}

// ---------------------------------------------------------------------------
// 3. 帯の有無
// ---------------------------------------------------------------------------

/// **soundtrack が無ければ帯ごと出ない。** 空の帯で行の面を削らない。
#[test]
fn a_project_without_a_soundtrack_gets_no_band_at_all() {
    assert_eq!(band_height(false), 0.0, "soundtrack 無し = 帯ごと出さない");
    assert!(band_height(true) > 0.0, "soundtrack 有り = 高さ固定で常設");
}

/// 席に座っているエディタでも同じ。lab fixture は soundtrack を持たない。
#[test]
fn the_fixture_editor_reports_no_band() {
    use motolii_ui::timeline_editor::lab_fixture;
    let (document, _names) = lab_fixture();
    assert!(
        document.soundtrack.is_none(),
        "fixture は soundtrack を持たない"
    );
    assert_eq!(
        band_height(document.soundtrack.is_some()),
        0.0,
        "fixture の面は帯に取られない"
    );
}
