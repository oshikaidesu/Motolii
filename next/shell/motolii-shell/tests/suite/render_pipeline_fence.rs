//! 実機チラつき調査(2026-08-20)の柵。
//!
//! ## 特定した一次原因
//!
//! `next/` が実際に使う crates.io `iced 0.14.0` は、`image::Handle::from_rgba`
//! を呼ぶたび `Id::unique()` で新しい id を割る(`iced_core-0.14.0/src/image.rs`
//! 実測)。さらに `iced_wgpu-0.14.0/src/image/cache.rs::upload_raster` は
//! `MAX_SYNC_SIZE = 2 * 1024 * 1024` byte を境に、それ以上の RGBA をバック
//! グラウンドスレッドへ非同期アップロードへ回す(実測)。非同期の間、
//! `draw_image` はその frame 何も描かない(`iced_core-0.14.0/src/image.rs` の
//! `Allocation` doc comment: "If you are animating images, this can cause
//! undesirable flicker" と明記)。
//!
//! fixture の comp は 1920×1080 = 8,294,400 byte(上限の約4倍)。scrub の
//! たびに新しい Handle → 非同期アップロード → 空白フレーム、が実機チラつきの
//! 一次原因と特定した。`Shell::refresh_frame`(`src/lib.rs`)は Stage 表示用の
//! RGBA を `stage_handle_rgba` で縮めてから Handle を作ることでこれを避ける。
//!
//! `Shell` は revision/playhead が同じなら元々 Handle を作り直さない
//! (`frame_cache_follows_revision_and_playhead`、`tests/drive.rs`)。この柵が
//! 新しく縛るのはその先 — **実際に新しい Handle を作る時に、iced 自身の
//! 同期アップロード予算を超えないこと**。

use motolii_engine::ObservationCamera;
use motolii_shell::{metrics, stage, Message, Shell};

/// プロセス全体で共有される `metrics` の static を、テスト間で取り合わないための
/// 排他(この1ファイル内の `#[test]` はデフォルトで並列実行されるため)。
static METRICS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// iced_wgpu-0.14.0/src/image/cache.rs::upload_raster の `MAX_SYNC_SIZE` と
/// 同じ値。柵の期待値をこの1箇所からだけ引く(数値の二重管理をしない)。
const ICED_SYNC_UPLOAD_BUDGET_BYTES: usize = 2 * 1024 * 1024;

/// **アイドル時は Handle を作り直さない**。`Select`/空の `FlushDrops` のような
/// Document も再生位置も変えない Message を連打しても、Stage の Handle 生成は
/// 増えない(`refresh_frame` の revision/playhead 早期 return が効いている)こと。
#[test]
fn idle_messages_do_not_recreate_the_stage_handle() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let after_first_edit = metrics::handle_creations();
    assert!(
        after_first_edit >= 1,
        "layer を足しても Stage が一度も描かれていない"
    );

    // Document も playhead も動かさない Message を10回連打する
    // (旧発注書が名指しした容疑者: 無関係な Message で Handle が作り直されないか)。
    for _ in 0..10 {
        let _ = shell.update(Message::Select(motolii_store::LayerId(1)));
        let _ = shell.update(Message::FlushDrops); // pending_drops は空 — 何もしないはず
    }

    assert_eq!(
        metrics::handle_creations(),
        after_first_edit,
        "アイドル相当の Message で Stage の Handle が作り直されている(容疑者1)"
    );
}

/// **scrub 中も1操作1生成**。playhead を N 回動かしたら、Handle 生成もちょうど
/// N 回増える(1回の scrub で複数生成したり、逆に取りこぼしたりしない)。
#[test]
fn scrubbing_creates_exactly_one_handle_per_distinct_frame() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let before = metrics::handle_creations();

    let frames = [1, 5, 12, 40, 41, 100];
    for &frame in &frames {
        let _ = shell.update(Message::ScrubTo(frame));
    }

    assert_eq!(
        metrics::handle_creations(),
        before + frames.len() as u64,
        "scrub {}回に対して Handle 生成回数が一致しない(1操作1生成のはず)",
        frames.len()
    );
}

/// **一次原因の直接の柵**: fixture(1920×1080・15層)を開いても、Stage の
/// `image::Handle` を作る RGBA は iced の同期アップロード予算を超えない。
/// 超えていれば非同期アップロード経路に落ち、実機で報告されたチラつきが
/// 再現する。
#[test]
fn fixture_stage_handle_bytes_stay_inside_the_synchronous_upload_budget() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let shell = Shell::new_fixture().0;
    assert_eq!(
        shell.layer_count(),
        15,
        "fixture の前提(層15枚)が崩れている"
    );

    let composition = shell.composition().expect("fixture は comp を持つ");
    let native_bytes = composition.width as usize * composition.height as usize * 4;
    assert!(
        native_bytes > ICED_SYNC_UPLOAD_BUDGET_BYTES,
        "fixture の comp が実は同期予算に収まっている — この柵の前提(1920x1080で\
         越えること)が崩れている: {native_bytes} byte"
    );

    let handle_bytes = metrics::last_handle_bytes();
    assert!(
        handle_bytes > 0,
        "fixture 起動で Stage が一度も描かれていない"
    );
    assert!(
        handle_bytes < ICED_SYNC_UPLOAD_BUDGET_BYTES,
        "Stage の Handle が iced の同期アップロード予算({ICED_SYNC_UPLOAD_BUDGET_BYTES} byte)を\
         超えている({handle_bytes} byte) — 非同期アップロード経路に落ち、\
         実機チラつきが再現する"
    );
}

/// **観測カメラ(裁定157)の鍵拡張の柵**: `RenderedFrame` のキャッシュ早期
/// return は `revision`/`playhead`/`checkerboard` に加えて `observation` も
/// 見ている(`lib.rs::refresh_frame`)。同じ観測カメラ値をもう一度送っても
/// Handle は作り直さない(`idle_messages_do_not_recreate_the_stage_handle` の
/// 観測カメラ版)。
#[test]
fn idle_observation_does_not_recreate_the_stage_handle() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let camera = ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 1.8,
    };
    let _ = shell.update(Message::Stage(stage::Message::Observe(camera)));
    let after_observe = metrics::handle_creations();
    assert!(after_observe >= 1, "観測カメラへ入っても一度も描かれていない");

    // 同じ値をもう一度送る — アイドル相当。
    for _ in 0..5 {
        let _ = shell.update(Message::Stage(stage::Message::Observe(camera)));
    }
    assert_eq!(
        metrics::handle_creations(),
        after_observe,
        "同じ観測カメラの再送で Stage の Handle が作り直されている"
    );

    // 値を変えれば作り直す(鍵が本当に効いていることの裏付け — 変化を
    // 検知できないだけの「常に return」になっていないか)。
    let _ = shell.update(Message::Stage(stage::Message::Observe(ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 2.5,
    })));
    assert!(
        metrics::handle_creations() > after_observe,
        "観測カメラの値が変わったのに Handle が作り直されていない"
    );
}

/// scrub で新しい comp 解像度の frame を作っても、毎回 iced の同期予算に収まる
/// (fixture の全尺を通して縮小が効き続けることの確認 — 冒頭1フレームだけの
/// 偶然ではない)。
#[test]
fn scrubbing_through_the_fixture_never_exceeds_the_upload_budget() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new_fixture().0;
    for frame in [0, 300, 510, 900, 1500, 1799] {
        let _ = shell.update(Message::ScrubTo(frame));
        let handle_bytes = metrics::last_handle_bytes();
        assert!(
            handle_bytes < ICED_SYNC_UPLOAD_BUDGET_BYTES,
            "frame {frame} の Stage Handle が同期予算を超えている({handle_bytes} byte)"
        );
    }
}
