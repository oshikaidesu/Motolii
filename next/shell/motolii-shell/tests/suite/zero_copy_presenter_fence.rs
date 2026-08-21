//! 裁定171 v2(M4 レーン、`docs/reviews/2026-08-22-zero-copy-seam-decision.md`)—
//! 「再生中の Stage 経路で readback 呼び出し 0」の red 先行 oracle。
//!
//! ## 現状(2026-08-22 施工時点): red のまま `#[ignore]` 付きで保存する
//!
//! M4 の EXACT TARGET 1〜3(Pipeline が自前の `Compositor::with_device` を持ち、
//! `prepare` で直接 GPU へ描き、`render` で main_target を blit する)は、
//! `motolii-compositor`(`next/engine/motolii-compositor/src/lib.rs`)に
//! **GPU 出力を CPU 読み戻しせずに返す公開 API が1つも無い**ことで構造的に
//! ブロックされている——既存の `render`/`render_with_timing`/
//! `render_with_effects`/`render_sequential` はどれも内部で
//! `ScreenshotProcessor::next_readback_result`(GPU→CPU 読み戻し)を呼んでから
//! `Vec<u8>` を返す(`render_sequential` が `ViewBuilder::main_target()` を
//! 保持するのも「次 layer の背景として使い回す」ためで、最終的には同じ
//! readback で終わる——裁定161 BL1b のモジュール doc 参照)。
//!
//! `motolii-compositor` は M4 レーンの ALLOWLIST に入っていない
//! (`next/shell/motolii-shell/**`・`next/shell/motolii-shell/tests/**`・
//! `next/ui/motolii-stage-pane/src/**`・`next/engine/motolii-engine/src/lib.rs`
//! のみ)ので、この施工ではそこへ手を出さず、代わりにこの oracle を
//! **red のまま `#[ignore]` 付きで**保存する——次のレーンが
//! `motolii-compositor` へ「readback しない GPU 出力口」を1つ足せば、
//! `#[ignore]` を外すだけでこの試験がそのまま受入条件になる
//! (提案シグネチャ・詳細は M4 施工報告 2026-08-22 参照)。
//!
//! ## 主張(ORACLE (a) そのもの)
//!
//! 再生 tick を10回回しても、`metrics::render_frame_calls()`
//! (`Engine::render_frame` の呼び出し回数 = CPU 読み戻しの回数そのもの、
//! `Shell::refresh_frame` が `record_render_frame` で計測)は起動直後の初回
//! 描画1回から**増えない**——現状は毎 tick 増える(red、`refresh_frame` の
//! 「revision/playhead が変わったら engine.render_frame を呼ぶ」経路が
//! 再生中も無条件に生きているため)。

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_shell::{metrics, Message, Shell};

/// `render_pipeline_fence.rs` の `METRICS_LOCK` と同型・別 static(`metrics`は
/// プロセス共有 static なので、同バイナリ内で並列実行される `#[test]` 同士の
/// 排他が要る)。
static METRICS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

/// **裁定171 v2 ORACLE (a)**。次のレーンが `motolii-compositor` へ zero-copy
/// GPU 出力口を足し、Pipeline 側の blit 配線を終えたら `#[ignore]` を外す。
#[test]
#[ignore = "裁定171 M4: motolii-compositor に readback しない GPU 出力 API が無く構造的に red \
            (この crate は M4 ALLOWLIST 外)。施工報告(2026-08-22)参照 — \
            API が足されたら #[ignore] を外して受入条件にする。"]
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
         (現状 red — GPU→CPU→GPU 往復がまだ生きている、warmup後は{warmup_calls}回で\
         横ばいのはずが、10 tick 回して増えている)"
    );
}
