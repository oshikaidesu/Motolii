//! Timeline 音声波形(TL7 統合手順3・5)。**未実行**(裁定189)。
//! `plan_waveforms`/`Message::WaveformFetched`/`WaveformFetchFailed` は
//! Document/Session を触らないので、以下は空 `Document::new()` +
//! `Session::default()` を素通りさせるだけの最小 fixture を使う。SP-2 分割で
//! 元は `waveform_message_tests`(`write.rs` 内の兄弟モジュール)だった物を
//! そのまま移設(中身は無改変)。

use crate::write::*;
use motolii_store::Document;

fn no_mods() -> iced::keyboard::Modifiers {
    iced::keyboard::Modifiers::default()
}

fn audio_row(layer: LayerId, path: &str) -> AudioRowProjection {
    AudioRowProjection { layer, has_audio: true, source_path: Some(path.to_owned()) }
}

/// **オラクル(赤→緑)**: 未着手の音声 layer は `plan_waveforms` の初回呼び出しで
/// 要求列へ積まれ、内部状態は即 `Loading` へ遷移する(次フレームで重複発火
/// しないための下準備 — `waveform_view::plan` のヒステリシスと同じ思想)。
#[test]
fn plan_waveforms_requests_a_fetch_for_a_new_audio_layer_and_marks_it_loading() {
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [audio_row(layer, "clip.mov")];

    let requests = pane.plan_waveforms(&rows, |_| 500.0);

    let expected_buckets = waveform_view::required_buckets(500.0);
    assert_eq!(requests, vec![(layer, "clip.mov".to_owned(), expected_buckets)]);
    assert_eq!(
        pane.waveforms().get(&layer),
        Some(&WaveformState::Loading { buckets: expected_buckets }),
        "要求した layer が Loading へ遷移していない"
    );
}

/// 音声を持たない layer(`has_audio == false`)は要求されず、内部状態にも
/// 現れない。
#[test]
fn plan_waveforms_skips_layers_without_audio() {
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [AudioRowProjection { layer, has_audio: false, source_path: None }];

    let requests = pane.plan_waveforms(&rows, |_| 500.0);

    assert!(requests.is_empty(), "音声の無い layer が要求された");
    assert!(pane.waveforms().get(&layer).is_none());
}

/// 同じ画面幅で2回呼んでも(`Loading` のまま)重複要求しない — 取得中の
/// bucket 数と一致する限り再要求は起きない(`waveform_view::plan` 参照)。
#[test]
fn plan_waveforms_does_not_refetch_while_already_loading_the_same_width() {
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [audio_row(layer, "clip.mov")];

    let first = pane.plan_waveforms(&rows, |_| 500.0);
    assert_eq!(first.len(), 1);
    let second = pane.plan_waveforms(&rows, |_| 500.0);
    assert!(second.is_empty(), "取得中の同じ幅で重複要求している");
}

/// **オラクル(赤→緑)**: `Message::WaveformFetched` は一致する `Loading` を
/// `Ready` へ遷移させる。
#[test]
fn waveform_fetched_message_transitions_loading_to_ready() {
    let mut doc = Document::new();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [audio_row(layer, "clip.mov")];

    let requests = pane.plan_waveforms(&rows, |_| 500.0);
    let (_, _, buckets) = requests[0].clone();
    let peaks = vec![(-0.5, 0.5); buckets];

    let reason = pane.update(
        Message::WaveformFetched { layer, buckets, peaks: peaks.clone() },
        &mut doc,
        &mut session,
        no_mods(),
    );
    assert!(reason.is_none());
    assert_eq!(pane.waveforms().get(&layer), Some(&WaveformState::Ready { buckets, peaks }));
}

/// stale な結果(取得中に別のズームで再要求が発火済み)は捨てる —
/// 現在の `Loading` の bucket 数と食い違う `WaveformFetched` は無視される。
#[test]
fn waveform_fetched_message_ignores_a_stale_result() {
    let mut doc = Document::new();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [audio_row(layer, "clip.mov")];

    let first = pane.plan_waveforms(&rows, |_| 10.0);
    let (_, _, stale_buckets) = first[0].clone();
    // 大きくズームして新しい要求を発火させる(旧要求は now stale)。
    let second = pane.plan_waveforms(&rows, |_| 100_000.0);
    let (_, _, fresh_buckets) = second[0].clone();
    assert_ne!(stale_buckets, fresh_buckets, "テスト前提: ズームで bucket 数が変わる");

    let reason = pane.update(
        Message::WaveformFetched { layer, buckets: stale_buckets, peaks: vec![(0.0, 0.0); stale_buckets] },
        &mut doc,
        &mut session,
        no_mods(),
    );
    assert!(reason.is_none());
    assert_eq!(
        pane.waveforms().get(&layer),
        Some(&WaveformState::Loading { buckets: fresh_buckets }),
        "stale な結果で新しい Loading が上書きされてしまった"
    );
}

/// **オラクル(赤→緑)**: `Message::WaveformFetchFailed` は空 peaks の
/// `Ready` へ落とす — `NotRequested` へ戻すと `plan_waveforms` が次の
/// 呼び出しで即再要求してしまう(ヒステリシス無しの無限リトライ)ため。
#[test]
fn waveform_fetch_failed_message_settles_into_an_empty_ready_state() {
    let mut doc = Document::new();
    let mut session = Session::default();
    let mut pane = PaneState::new();
    let layer = LayerId(1);
    let rows = [audio_row(layer, "clip.mov")];

    let requests = pane.plan_waveforms(&rows, |_| 500.0);
    let (_, _, buckets) = requests[0].clone();

    let reason = pane.update(
        Message::WaveformFetchFailed { layer, buckets },
        &mut doc,
        &mut session,
        no_mods(),
    );
    assert!(reason.is_none());
    assert_eq!(pane.waveforms().get(&layer), Some(&WaveformState::Ready { buckets, peaks: Vec::new() }));

    // 同じ画面幅で再度 plan しても再要求しない(無限リトライにならない)。
    let after = pane.plan_waveforms(&rows, |_| 500.0);
    assert!(after.is_empty(), "failed 後に同じ幅で即再要求してしまっている(無限リトライ)");
}
