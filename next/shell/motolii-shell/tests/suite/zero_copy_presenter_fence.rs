//! 裁定171 v2(M4、`docs/reviews/2026-08-22-zero-copy-seam-decision.md`)—
//! 「再生中の Stage 経路で readback 呼び出し 0」の oracle(ORACLE (a))。
//!
//! ## 施工経緯(2026-08-22)
//!
//! 初回施工時点では `motolii-compositor` に GPU 出力を CPU 読み戻しせずに
//! 返す公開 API が無く、この試験は `#[ignore]` 付きの red として保存した
//! (施工報告参照)。supervisor 裁定で ALLOWLIST が
//! `next/engine/motolii-compositor/src/lib.rs`+`tests/**`・
//! `next/engine/motolii-engine/src/lib.rs`(いずれも additive のみ)へ拡張され、
//! 次の3つが揃ったことで `#[ignore]` を外せる状態になった:
//!
//! 1. `Compositor::render_to_texture`(additive、既存4メソッド無改造)——
//!    readback せず `(wgpu::Texture, wgpu::TextureView)` を返す
//! 2. `Engine::with_device`/`render_resolved_to_texture`/`render_frame_to_texture`
//!    (additive、headless 経路(`Engine::new`/`render_frame`)は無改造)
//! 3. `motolii-shell` の presenter Pipeline(`StagePresenterPipeline`)が
//!    `Engine::with_device` を自前で所有し、`prepare` で世代ゲート越しに
//!    `render_resolved_to_texture` を呼ぶ——`Shell::refresh_frame` は
//!    revision 不変・playhead のみ変化(市松 OFF・観測カメラ無効・
//!    resolution cap = Auto)の間、`Engine::render_frame`(CPU readback)を
//!    一切呼ばない(`RenderedFrame::rgba_stale`+`Shell::ensure_rgba_fresh`で
//!    `frame_rgba()`/screenshot が実際に要求した時だけ追いつかせる、
//!    EXACT TARGET 4)
//!
//! ## 主張(ORACLE (a) そのもの)
//!
//! 再生 tick を10回回しても、`metrics::render_frame_calls()`
//! (`Engine::render_frame` の呼び出し回数 = CPU 読み戻しの回数そのもの、
//! `Shell::refresh_frame` が `record_render_frame` で計測)は起動直後の初回
//! 描画1回から**増えない**。

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_shell::{metrics, Message, Shell};

/// `metrics_main.rs` の crate root で定義された共有 lock(`metrics` はプロセス
/// 共有 static なので、同バイナリ内で並列実行される `#[test]` 同士 — かつ
/// `render_pipeline_fence` モジュールとの間も — の排他が要る)。
use crate::METRICS_LOCK;

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

/// フェイクの再生セッション(実 cpal/producer は開かない、`playback_drive.rs`・
/// `render_pipeline_fence.rs` と同じ手)。
fn fake_session_at(frame: i64) -> (PlaybackSession, Arc<PlaybackCounters>) {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(DeviceWaitLatency::default());
    let mut clock =
        PlaybackClock::new(Arc::clone(&counters), wait, 48_000).expect("48kHzは有効なsample_rate");
    let at = RationalTime::try_from_frame(frame, fixture_fps()).expect("fixtureのfpsは有効");
    clock.start(at);
    (PlaybackSession::for_simulation(clock), counters)
}

/// **裁定171 v2 ORACLE (a)**。
#[test]
fn playback_ticks_do_not_trigger_cpu_readback_after_warmup() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new_fixture().0;
    // fixture 起動時の初回描画で1回だけ readback が起きるのは想定内
    // (最初の1枚を出すのに絵が要る——EXACT TARGET はこの1回を禁じていない、
    // M17「起動直後に何も見えないのは違反」)。
    let warmup_calls = metrics::render_frame_calls();
    assert!(warmup_calls >= 1, "起動直後に何も描いていない(M17 違反)");

    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    let start_playhead = shell.session().playhead;
    for _ in 0..10 {
        // 1,600サンプル@48kHz = 1/30秒 = comp 30fpsでちょうど1フレーム。
        counters.advance_supplied_for_simulation(1_600);
        let _ = shell.update(Message::PlaybackTick);
    }

    assert!(
        shell.session().playhead > start_playhead,
        "そもそもtickでplayheadが動いていない — 試験の前提が崩れている"
    );
    assert_eq!(
        metrics::render_frame_calls(),
        warmup_calls,
        "裁定171 v2 EXACT TARGET 3/4: 再生中の Stage 経路は readback ゼロのはず\
         (warmup後は{warmup_calls}回で横ばいのはずが、10 tick 回して増えている)"
    );
}

