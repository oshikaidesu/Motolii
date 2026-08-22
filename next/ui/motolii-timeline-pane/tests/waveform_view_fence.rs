//! Timeline 音声波形(`waveform_view`)の落ちるテスト先行(発注 2026-08-22
//! 「Timeline の音声波形表示」)。細粒度の純関数テストは `src/waveform_view.rs`
//! 内(`required_buckets`/`needs_recompute`/`waveform_segments`/`plan` それぞれの
//! 退化・境界値)— ここは `transport_fence.rs`/`work_area_shuttle_fence.rs` と
//! 同じ型で、**crate の公開 API 境界を通した**統合面のオラクルだけを見る。
//!
//! オラクル:
//! - (a) 3状態(`WaveformState::NotRequested`/`Loading`/`Ready`)の投影が
//!   `waveform_state_segments` を通して一貫する(`Ready` だけが描画データを
//!   持つ)
//! - (b) ズーム(クリップ画面幅の変化)に応じて `plan` が要求する bucket 数が
//!   変わる/変わらないの筋が通る(id 74 `Zoom Audio Waveform` の意味その物)
//! - (c) peaks→画面座標の写像がクリップの矩形(x/y/width/height)に正しく
//!   従う(中央線・振幅の符号)
//! - (d) 退化(0 bucket・無音・音声なしクリップ)でも panic しない
//! - (e) このモジュールの追加が既存 `TimelinePane`(canvas/transport)の
//!   構築・`view()` を壊していない(スモーク — 名前衝突/コンパイル破壊が
//!   無いことの確認。絵と意味の対応自体は `transport_fence.rs`/
//!   `canvas.rs::ratio_tests` が正本)
//!
//! **未結線(次波)**: このテストは `PaneState`/`Message`/`canvas::draw` を
//! 一切通さない — `waveform_view` の関数は今はどこからも呼ばれていない
//! (発注書 EXACT TARGET「新ファイル縛り」の帰結、モジュール doc 参照)。

use iced::Rectangle;
use motolii_shell_state::Session;
use motolii_store::{Composition, Document, Fps, Intent};
use motolii_timeline_pane::tokens::{Colors, Dimensions};
use motolii_timeline_pane::waveform_view::{
    needs_recompute, plan, required_buckets, waveform_ink, waveform_segments,
    waveform_state_segments, WaveformAction, WaveformState,
};
use motolii_timeline_pane::TimelinePane;

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    Rectangle { x, y, width, height }
}

// ---------------------------------------------------------------------------
// (a) 3状態の投影 — Ready だけが描画データを持つ
// ---------------------------------------------------------------------------

#[test]
fn only_the_ready_state_yields_drawable_segments() {
    let r = rect(0.0, 0.0, 40.0, 18.0);

    let not_requested = waveform_state_segments(&WaveformState::NotRequested, r);
    assert!(not_requested.is_empty(), "未着手は何も描かない");

    let loading = waveform_state_segments(&WaveformState::Loading { buckets: 32 }, r);
    assert!(loading.is_empty(), "取得中は(このモジュール単体では)何も描かない");

    let ready = WaveformState::Ready { buckets: 4, peaks: vec![(-0.4, 0.6); 4] };
    let ready_segments = waveform_state_segments(&ready, r);
    assert_eq!(ready_segments.len(), r.width.floor() as usize, "取得済みは矩形の幅ぶん描く");
}

// ---------------------------------------------------------------------------
// (b) ズームで要求 bucket 数が変わる/変わらない筋(id 74 の意味そのもの)
// ---------------------------------------------------------------------------

#[test]
fn zooming_the_clip_wider_can_raise_the_requested_bucket_count() {
    let narrow_state = WaveformState::NotRequested;
    let narrow_action = plan(&narrow_state, 20.0, true);
    let WaveformAction::Fetch(narrow_buckets) = narrow_action else {
        panic!("未着手の音声クリップは要求すべき: {narrow_action:?}");
    };

    // 取得済みにしてから大きくズーム(クリップ画面幅を広げる)する。
    let ready = WaveformState::Ready { buckets: narrow_buckets, peaks: vec![(0.0, 0.0); narrow_buckets] };
    let zoomed_action = plan(&ready, 50_000.0, true);
    let WaveformAction::Fetch(zoomed_buckets) = zoomed_action else {
        panic!("十分ズームしたら再要求すべき: {zoomed_action:?}");
    };
    assert!(zoomed_buckets > narrow_buckets, "ズームしたのに要求 bucket 数が増えていない");

    // 同じ幅のままなら再要求しない(id 74「ズーム」に反応した bucket 数の
    // 変化であって、無条件の連続再取得ではないことの確認)。
    let settled = WaveformState::Ready { buckets: zoomed_buckets, peaks: vec![(0.0, 0.0); zoomed_buckets] };
    assert_eq!(plan(&settled, 50_000.0, true), WaveformAction::None, "同じズーム幅では再要求しない");
}

#[test]
fn a_clip_without_audio_never_requests_regardless_of_zoom() {
    for width in [1.0, 100.0, 10_000.0] {
        assert_eq!(
            plan(&WaveformState::NotRequested, width, false),
            WaveformAction::None,
            "音声を持たないクリップはズームしても要求しない(幅={width})"
        );
    }
}

// ---------------------------------------------------------------------------
// (c) peaks→画面座標の写像がクリップ矩形に従う
// ---------------------------------------------------------------------------

#[test]
fn segments_stay_within_the_clip_rect_bounds() {
    let clip_rect = rect(120.0, 8.0, 64.0, 18.0);
    let peaks = vec![(-1.0, 1.0), (-0.2, 0.9), (0.0, 0.0), (-0.7, 0.1)];
    let segments = waveform_segments(&peaks, clip_rect);

    assert_eq!(segments.len(), clip_rect.width.floor() as usize);
    for segment in &segments {
        assert!(
            segment.x >= clip_rect.x && segment.x < clip_rect.x + clip_rect.width,
            "x がクリップ矩形の外: {segment:?}"
        );
        assert!(
            segment.y_top >= clip_rect.y && segment.y_top <= clip_rect.y + clip_rect.height,
            "y_top がクリップ矩形の外: {segment:?}"
        );
        assert!(
            segment.y_bottom >= clip_rect.y && segment.y_bottom <= clip_rect.y + clip_rect.height,
            "y_bottom がクリップ矩形の外: {segment:?}"
        );
        assert!(segment.y_top <= segment.y_bottom, "上端が下端より下にある: {segment:?}");
    }
}

// ---------------------------------------------------------------------------
// (d) 退化 — 0 bucket・無音・音声なしクリップでも panic しない
// ---------------------------------------------------------------------------

#[test]
fn degenerate_inputs_never_panic() {
    assert_eq!(required_buckets(0.0), 0);
    assert!(!needs_recompute(None, required_buckets(0.0)));
    assert!(waveform_segments(&[], rect(0.0, 0.0, 100.0, 20.0)).is_empty());
    assert!(waveform_segments(&[(0.0, 0.0)], rect(0.0, 0.0, 0.0, 20.0)).is_empty());

    let silent = vec![(0.0, 0.0); 16];
    let segments = waveform_segments(&silent, rect(0.0, 0.0, 30.0, 20.0));
    assert_eq!(segments.len(), 30);
    assert!(segments.iter().all(|s| s.y_top == s.y_bottom), "無音は上下端が同じ(中央線)であるべき");

    // data ロールとクリップ色が同じでも(壊れた token 読み)混色は破綻しない。
    let c = iced::Color::from_rgb(0.5, 0.5, 0.5);
    assert_eq!(waveform_ink(c, c), c);
}

// ---------------------------------------------------------------------------
// (e) 既存 TimelinePane の構築/view() が壊れていないスモーク
// ---------------------------------------------------------------------------

#[test]
fn the_pane_still_builds_and_views_after_the_waveform_module_lands() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    let store = doc.view();
    let session = Session::default();
    let pane = TimelinePane::new(
        &store,
        &session,
        Dimensions::default(),
        Colors::default(),
        iced::keyboard::Modifiers::default(),
    );
    let _view = pane.view();
}
