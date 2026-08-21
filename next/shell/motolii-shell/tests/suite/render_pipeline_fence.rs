//! 実機チラつき調査(2026-08-20)の柵 — **裁定166で経路ごと置き換え済み**。
//!
//! ## 経緯: 2026-08-20 に特定した一次原因
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
//! 一次原因と特定した。当時の柵は `stage_handle_rgba` の sqrt 自動縮小
//! (`STAGE_HANDLE_SYNC_BUDGET_BYTES = 1.5MB`)で、fixture を 816×459 まで
//! 縮めて同期アップロード予算に収めていた。
//!
//! ## 裁定166: 「実機報告のイージングのガタつき」で経路そのものを置き換え
//!
//! 2026-08-21深夜、利用者実窓較正で「イージングがガタついてる。image widget は
//! 無理があったんじゃないか」と裁定(`docs/reviews/2026-08-21-stage-presenter-
//! decision.md`)。release ビルドでもガタつきが再現し、構造の問題と判断 —
//! Stage の絵を `image::Handle` から `iced::widget::shader` の自前 Program
//! (永続 `wgpu::Texture` + 世代ゲート付き `queue.write_texture`)へ置き換えた
//! (`crate::StagePresenterPipeline`)。非同期アップロードの「その間何も描かない」
//! 穴が経路自体から消えたので、1.5MB 柵・`stage_auto_scale`(sqrt 自動縮小)は
//! 撤去し、**フル解像度復帰**した(Auto は 1.0 固定。resolution cap ½/¼ だけが
//! 「明示的な縮小」として残る)。
//!
//! **この1点だけ、旧柵と正反対の主張になる** —
//! `fixture_stage_handle_bytes_stay_inside_the_synchronous_upload_budget`
//! (旧: フル解像度は2MB予算を超えてはいけない)は
//! `fixture_stage_presenter_matches_native_composition_dimensions`
//! (新: フル解像度のまま2MB予算を平気で超える)へ**主張ごと逆転**した。
//! 矛盾ではない——旧柵が守っていたのは「image widget 経路で2MB を超えると
//! チラつく」という*当時の経路固有の制約*であって、その経路自体が無くなった
//! ので制約も消えた(EXACT TARGET 2「柵の存在理由が消える」)。
//!
//! ## この柵が新しく縛るもの(裁定166 ORACLE)
//!
//! - (a) 再生 tick を N 回回しても `metrics::handle_creations()` が増えない
//!   (Stage 描画経路はもう `image::Handle` を作らない — 呼び出し元ごと無くなった)
//! - (b) presenter へ渡る寸法 == comp 寸法(fixture 1920×1080、Auto の間)
//! - resolution cap ½/¼ は引き続き「明示的な縮小」として効く
//! - presenter の内容が変わった時だけ `presenter_generation` が動く(旧
//!   `handle_creations` が担っていた「1操作1生成」の主張を引き継ぐ)
//!
//! (c)(`--fixture --screenshot` PNG バイト不変)・(d)(shell suite 全緑)は
//! 自動化された `#[test]` の対象外(前者はビルド成果物のバイト比較、後者は
//! テスト実行そのもの)——施工記録側で確認する。

use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_engine::ObservationCamera;
use motolii_shell::{metrics, stage, Message, Shell};

/// プロセス全体で共有される `metrics` の static を、テスト間で取り合わないための
/// 排他(この1ファイル内の `#[test]` はデフォルトで並列実行されるため)。
static METRICS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// fixture(`Shell::new_fixture()`)の comp fps。`playback_drive.rs` の
/// `fixture_fps` と同じ値(30fps) — フェイク再生セッションを組むのに要る。
fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

/// フェイクの再生セッションを`frame`(comp フレーム)から組む。実cpal/producer
/// は一切開かない(`PlaybackSession::for_simulation`、`playback_drive.rs` と
/// 同じ手 — ORACLE「デバイス抽象はフェイクで」)。`Arc<PlaybackCounters>` を
/// 呼び出し側へ返すので、テストは `advance_supplied_for_simulation` で直接
/// 供給を進められる。
fn fake_session_at(frame: i64) -> (PlaybackSession, Arc<PlaybackCounters>) {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(DeviceWaitLatency::default());
    let mut clock =
        PlaybackClock::new(Arc::clone(&counters), wait, 48_000).expect("48kHzは有効なsample_rate");
    let at = RationalTime::try_from_frame(frame, fixture_fps()).expect("fixtureのfpsは有効");
    clock.start(at);
    (PlaybackSession::for_simulation(clock), counters)
}

// ---------------------------------------------------------------------------
// (a) 再生 tick を回しても、退役した `handle_creations` はもう増えない。
// ---------------------------------------------------------------------------

/// **裁定166 ORACLE (a)**: 再生中の tick を10回回しても
/// `metrics::handle_creations()` は増えない。旧経路(image widget)では
/// 再生 tick のたびに playhead が動く = 毎回新しい `image::Handle` が要る =
/// 毎フレーム `handle_creations` が+1(裁定166 施工前の red)。shader Program
/// へ置き換えたことで Stage 描画経路はもう `image::Handle` を一切作らない
/// ので、この経路からは金輪際増えなくなる。
#[test]
fn playback_ticks_no_longer_increment_the_retired_handle_creations_counter() {
    let _guard = METRICS_LOCK.lock().unwrap();
    metrics::reset();

    let mut shell = Shell::new_fixture().0;
    assert_eq!(metrics::handle_creations(), 0, "reset直後は0のはず");

    let (session, counters) = fake_session_at(900);
    shell.debug_start_playback_with_session(session);

    let start_playhead = shell.session().playhead;
    for _ in 0..10 {
        // 1,600サンプル@48kHz = 1/30秒 = comp 30fpsでちょうど1フレーム。
        // 空振り(同じフレームへのtick)にならないよう、毎回確実に動かす。
        counters.advance_supplied_for_simulation(1_600);
        let _ = shell.update(Message::PlaybackTick);
    }

    assert!(
        shell.session().playhead > start_playhead,
        "そもそもtickでplayheadが動いていない — 試験の前提が崩れている"
    );
    assert_eq!(
        metrics::handle_creations(),
        0,
        "裁定166: Stage は image::Handle をもう作らないはず\
         (build_stage_presenter_rgba は record_handle_creation を呼ばない) —\
         再生 tick で増えている(旧経路の再演)"
    );
}

// ---------------------------------------------------------------------------
// (b) presenter へ渡る寸法 == comp 寸法(フル解像度復帰)
// ---------------------------------------------------------------------------

/// **裁定166 ORACLE (b) の本命**: fixture(1920×1080・15層)を開くと、Stage
/// presenter へ渡る RGBA の寸法は comp 寸法そのまま(Auto = 1.0固定、旧
/// `stage_auto_scale` の sqrt 縮小は撤去済み — 施工前は 816×459 だった)。
#[test]
fn fixture_stage_presenter_matches_native_composition_dimensions() {
    let shell = Shell::new_fixture().0;
    assert_eq!(shell.layer_count(), 15, "fixtureの前提(層15枚)が崩れている");

    let composition = shell.composition().expect("fixtureはcompを持つ");
    assert_eq!((composition.width, composition.height), (1920, 1080));

    assert_eq!(
        shell.stage_presenter_dims(),
        Some((composition.width, composition.height)),
        "presenterへ渡る寸法がcomp寸法と一致しない(フル解像度復帰のはず)"
    );
}

/// scrub で新しい comp フレームを作っても、Auto の間は一貫してフル解像度の
/// まま(fixture の全尺を通して縮小されないことの確認 — 冒頭1フレームだけの
/// 偶然ではない)。
#[test]
fn scrubbing_through_the_fixture_stays_at_native_resolution_while_auto() {
    let mut shell = Shell::new_fixture().0;
    let composition = shell.composition().expect("fixtureはcompを持つ");

    for frame in [0, 300, 510, 900, 1500, 1799] {
        let _ = shell.update(Message::ScrubTo(frame));
        assert_eq!(
            shell.stage_presenter_dims(),
            Some((composition.width, composition.height)),
            "frame {frame} でpresenterがcomp解像度から縮んでいる(Autoのはず)"
        );
    }
}

// ---------------------------------------------------------------------------
// presenter_generation — 旧 `handle_creations` の「1操作1生成」の主張を継ぐ
// ---------------------------------------------------------------------------

/// **アイドル時は presenter を作り直さない**。`Select`/空の `FlushDrops` の
/// ような Document も再生位置も変えない Message を連打しても、Stage の
/// presenter 生成(`presenter_generation`)は増えない(`refresh_frame` の
/// revision/playhead 早期 return が効いている)こと。
#[test]
fn idle_messages_do_not_bump_the_presenter_generation() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let after_first_edit = shell
        .stage_presenter_generation()
        .expect("layerを足せばpresenterが立つはず");

    for _ in 0..10 {
        let _ = shell.update(Message::Select(motolii_store::LayerId(1)));
        let _ = shell.update(Message::FlushDrops); // pending_drops は空 — 何もしないはず
    }

    assert_eq!(
        shell.stage_presenter_generation(),
        Some(after_first_edit),
        "アイドル相当の Message で presenter_generation が動いている"
    );
}

/// **scrub 中も1操作1生成**。playhead を N 回動かしたら、presenter の世代も
/// ちょうど N 回増える。
#[test]
fn scrubbing_bumps_the_presenter_generation_exactly_once_per_distinct_frame() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let before = shell.stage_presenter_generation().expect("layerを足した直後");

    let frames = [1, 5, 12, 40, 41, 100];
    for &frame in &frames {
        let _ = shell.update(Message::ScrubTo(frame));
    }

    assert_eq!(
        shell.stage_presenter_generation(),
        Some(before + frames.len() as u64),
        "scrub {}回に対してpresenter_generationの増分が一致しない(1操作1生成のはず)",
        frames.len()
    );
}

/// **観測カメラ(裁定157)の鍵拡張の柵**: `RenderedFrame` のキャッシュ早期
/// return は `revision`/`playhead`/`checkerboard` に加えて `observation` も
/// 見ている(`lib.rs::refresh_frame`)。同じ観測カメラ値をもう一度送っても
/// presenter は作り直さない(`idle_messages_do_not_bump_the_presenter_
/// generation` の観測カメラ版)。
#[test]
fn idle_observation_does_not_bump_the_presenter_generation() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let camera = ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 1.8,
    };
    let _ = shell.update(Message::Stage(stage::Message::Observe(camera)));
    let after_observe = shell
        .stage_presenter_generation()
        .expect("観測カメラへ入れば描かれているはず");

    // 同じ値をもう一度送る — アイドル相当。
    for _ in 0..5 {
        let _ = shell.update(Message::Stage(stage::Message::Observe(camera)));
    }
    assert_eq!(
        shell.stage_presenter_generation(),
        Some(after_observe),
        "同じ観測カメラの再送でpresenter_generationが動いている"
    );

    // 値を変えれば作り直す(鍵が本当に効いていることの裏付け — 変化を
    // 検知できないだけの「常に return」になっていないか)。
    let _ = shell.update(Message::Stage(stage::Message::Observe(ObservationCamera {
        pan: [40.0, -15.0],
        zoom: 2.5,
    })));
    assert!(
        shell.stage_presenter_generation().unwrap() > after_observe,
        "観測カメラの値が変わったのにpresenter_generationが動いていない"
    );
}

// ---------------------------------------------------------------------------
// resolution cap ½/¼ は「明示的な縮小」として引き続き効く(EXACT TARGET 2)。
// ---------------------------------------------------------------------------

/// **裁定163 Stage 下縁状態帯 ORACLE (b) の裁定166 版**: プレビュー解像度 cap
/// のクリック巡回(`Message::Stage(stage::Message::CycleResolutionCap)`)が
/// 実際に presenter の寸法を変える。裁定166 で Auto は常にフル解像度
/// (旧 auto_scale の sqrt 縮小は撤去済み)なので、旧試験の「auto がすでに
/// ½ より小さいので ½ cap は no-op」という前提はもう成立しない —
/// ½ は native から確実に縮み、¼ はさらに縮む、という単純な単調減少になる。
#[test]
fn cycling_the_resolution_cap_scales_the_presenter_relative_to_native_resolution() {
    let shell = Shell::new_fixture().0;
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Auto);
    let native = shell.composition().expect("fixtureはcompを持つ");
    assert_eq!(
        shell.stage_presenter_dims(),
        Some((native.width, native.height)),
        "Auto はフル解像度のはず"
    );

    let mut shell = shell;
    let _ = shell.update(Message::Stage(stage::Message::CycleResolutionCap));
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Half);
    let (half_w, half_h) = shell.stage_presenter_dims().expect("½ capでも描けているはず");
    assert!(
        half_w < native.width && half_h < native.height,
        "½ cap で縮んでいない: half=({half_w}, {half_h}) native=({}, {})",
        native.width,
        native.height
    );

    let _ = shell.update(Message::Stage(stage::Message::CycleResolutionCap));
    assert_eq!(shell.resolution_cap(), stage::PreviewResolutionCap::Quarter);
    let (quarter_w, quarter_h) = shell.stage_presenter_dims().expect("¼ capでも描けているはず");
    assert!(
        quarter_w < half_w && quarter_h < half_h,
        "¼ cap が ½ よりさらに縮んでいない: half=({half_w}, {half_h}) quarter=({quarter_w}, {quarter_h})"
    );
}
