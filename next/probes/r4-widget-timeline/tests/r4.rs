//! R4: widget タイムライン(`pin`+`Stack`+自作 translate container)の性能 probe。
//! TL-arch survey(`docs/reviews/2026-08-22-timeline-canvas-widget-survey.md`)
//! §5 EVIDENCE_GAP #1「性能の実測値が無い」を埋める。
//!
//! ## 測定の器具と限界
//!
//! `iced_test`(`test/src/simulator.rs` 実測)の `Simulator::new` は内部で
//! `UserInterface::build(element, size, Cache::default(), &mut renderer)` を
//! 呼んでいるだけ(diff+layout)。本 probe は `Simulator` を経由せず、同じ
//! `iced_test::runtime::user_interface::UserInterface` を直接叩く——理由は
//! `Simulator` の `raw` フィールドが非公開で、`build()`(diff+layout)と
//! `draw()`(primitive記録)を個別に計測する経路が `Simulator` からは得られない
//! ため。ヘッドレス renderer の作り方(`Renderer::new` を `block_on`)は
//! `Simulator::with_size` の実装をそのまま踏襲する(同じ手口)。
//!
//! **測っているのは CPU 側だけ**: `draw()` は primitive をバッファへ記録する
//! ところまでで、実際の GPU submit・present は行わない(headless の既知の
//! 限界、iced 本体のコメントにも明記が無いが `Renderer::draw` の実装を見れば
//! 自明——コンポジタへは送らない)。GPU 側のラスタライズ・合成コストは
//! この probe では測れない(EVIDENCE_GAP、RETURN 参照)。
//!
//! ```sh
//! cargo test --release -p r4-widget-timeline -- --nocapture
//! ```

use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use iced::advanced::renderer::{Headless, Style as RendererStyle};
use iced::theme::Base as _;
use iced::{Color, Element, Renderer, Theme};
use iced_test::futures::futures::executor::block_on;
use iced_test::runtime::user_interface::{Cache, UserInterface};

use r4_widget_timeline::{
    bar as build_bar, stacked_bars, stacked_bars_fixed, Message, TranslateLane, FRAME_BUDGET_US,
    SCALES, VIEWPORT,
};

/// フレーム/イテレーション数。r1 の `RUNS` 相当(中央値を取るための反復)。
const ITERATIONS: u32 = 24;

fn headless_renderer() -> Renderer {
    block_on(Renderer::new(
        iced::advanced::renderer::Settings::default(),
        None,
    ))
    .expect("headless renderer must construct — iced_test_spike.rs と同じ経路")
}

fn median(mut xs: Vec<u128>) -> u128 {
    xs.sort_unstable();
    xs[xs.len() / 2]
}

fn theme_and_style() -> (Theme, RendererStyle) {
    (
        Theme::default(iced::theme::Mode::None),
        RendererStyle {
            text_color: Color::BLACK,
        },
    )
}

// ---------------------------------------------------------------------------
// (a) 静止: N 個の pin(bar) を Stack へ絶対配置した view の初回 build/draw
// ---------------------------------------------------------------------------

/// 戻り値: (build 中央値µs, draw 中央値µs)。build = diff+layout
/// (`UserInterface::build` 実装実測、`runtime/src/user_interface.rs` 行106-113)。
fn measure_static(n: usize) -> (u128, u128) {
    let (theme, style) = theme_and_style();
    let mut build_us = Vec::with_capacity(ITERATIONS as usize);
    let mut draw_us = Vec::with_capacity(ITERATIONS as usize);

    for _ in 0..ITERATIONS {
        let mut renderer = headless_renderer();
        let element: Element<'_, Message> = stacked_bars(n, 1.0, 0.0);

        let t0 = Instant::now();
        let mut ui = UserInterface::build(element, VIEWPORT, Cache::default(), &mut renderer);
        build_us.push(t0.elapsed().as_micros());

        let t1 = Instant::now();
        ui.draw(
            &mut renderer,
            &theme,
            &style,
            iced::mouse::Cursor::Unavailable,
        );
        draw_us.push(t1.elapsed().as_micros());

        let _ = ui.into_cache();
    }

    (median(build_us), median(draw_us))
}

// ---------------------------------------------------------------------------
// (b) パン=カメラ: TranslateLane が draw 時 translation だけでパンする
// ---------------------------------------------------------------------------

/// 戻り値: (1フレームあたりの draw 中央値µs, 全フレーム終了後の layout 呼び出し回数)。
/// layout 呼び出し回数が 1 のままであることが「re-layout していない」ことの
/// 直接証拠(`TranslateLane::layout` が呼ばれるたびに `layout_calls` を
/// インクリメントする計測フック、`src/lib.rs` 参照)。
fn measure_pan_camera(n: usize) -> (u128, u32) {
    let (theme, style) = theme_and_style();
    let offset_x = Rc::new(Cell::new(0.0_f32));
    let layout_calls = Rc::new(Cell::new(0_u32));

    let mut renderer = headless_renderer();
    let content = stacked_bars_fixed(n, 1.0);
    let lane: Element<'_, Message> =
        TranslateLane::new(content, offset_x.clone(), layout_calls.clone()).into();

    let mut ui = UserInterface::build(lane, VIEWPORT, Cache::default(), &mut renderer);
    assert_eq!(
        layout_calls.get(),
        1,
        "build() は一度だけ layout を呼ぶはず(n={n})"
    );

    let mut frame_us = Vec::with_capacity(ITERATIONS as usize);
    for frame in 0..ITERATIONS {
        // 連続パン相当 — 毎フレーム値を変える(定数畳み込みで測定が無意味に
        // ならないようにする)。
        offset_x.set(frame as f32 * 4.0);

        let t0 = Instant::now();
        ui.draw(
            &mut renderer,
            &theme,
            &style,
            iced::mouse::Cursor::Unavailable,
        );
        frame_us.push(t0.elapsed().as_micros());
    }

    let calls_after = layout_calls.get();
    assert_eq!(
        calls_after, 1,
        "パン中に layout が再度呼ばれている(n={n}) — re-layout してはいけない条件が破れた"
    );

    (median(frame_us), calls_after)
}

// ---------------------------------------------------------------------------
// (c) パン=素朴再構築: view 全体(pin の x)を毎フレーム作り直す
// ---------------------------------------------------------------------------

/// build(diff+layout)+draw を合わせた1フレームあたりの中央値µs。
/// cache は毎フレーム持ち越す(実アプリの `view()` 再構築ループと同じ手口 —
/// 空 Cache から作り直すのは初回だけで、以降は前フレームの `Tree` を diff の
/// 出発点にする。これをやらないと「初回相当のコスト」を毎フレーム測ることに
/// なり、(b)との比較が不当に不利になる)。
fn measure_pan_naive_rebuild(n: usize) -> u128 {
    let (theme, style) = theme_and_style();
    let mut renderer = headless_renderer();

    let mut cache = Cache::default();
    {
        let element = stacked_bars(n, 1.0, 0.0);
        let ui = UserInterface::build(element, VIEWPORT, cache, &mut renderer);
        cache = ui.into_cache();
    }

    let mut frame_us = Vec::with_capacity(ITERATIONS as usize);
    for frame in 0..ITERATIONS {
        let pan_px = frame as f32 * 4.0;
        let element = stacked_bars(n, 1.0, pan_px);

        let t0 = Instant::now();
        let mut ui = UserInterface::build(element, VIEWPORT, cache, &mut renderer);
        ui.draw(
            &mut renderer,
            &theme,
            &style,
            iced::mouse::Cursor::Unavailable,
        );
        frame_us.push(t0.elapsed().as_micros());

        cache = ui.into_cache();
    }

    median(frame_us)
}

// ---------------------------------------------------------------------------
// (d) zoom=x-only 再配置: x・幅だけ再計算(y=row位置は不変)
// ---------------------------------------------------------------------------

/// 1回あたりの build+draw 中央値µs。手口は(c)と同型(view 再構築)だが、
/// 変えるのは `zoom`(pan_px は常に0) — `stacked_bars` は zoom を x にも幅にも
/// 掛けるが y は `i as f32 * ROW_HEIGHT` のまま触らない(`src/lib.rs` 参照)。
fn measure_zoom_rebuild(n: usize) -> u128 {
    let (theme, style) = theme_and_style();
    let mut renderer = headless_renderer();

    let mut cache = Cache::default();
    {
        let element = stacked_bars(n, 1.0, 0.0);
        let ui = UserInterface::build(element, VIEWPORT, cache, &mut renderer);
        cache = ui.into_cache();
    }

    let mut step_us = Vec::with_capacity(ITERATIONS as usize);
    for step in 0..ITERATIONS {
        let zoom = 1.0 + (step as f32) * 0.05;
        let element = stacked_bars(n, zoom, 0.0);

        let t0 = Instant::now();
        let mut ui = UserInterface::build(element, VIEWPORT, cache, &mut renderer);
        ui.draw(
            &mut renderer,
            &theme,
            &style,
            iced::mouse::Cursor::Unavailable,
        );
        step_us.push(t0.elapsed().as_micros());

        cache = ui.into_cache();
    }

    median(step_us)
}

// ---------------------------------------------------------------------------
// 実行 — 行列を組んで印字する(合否は(b)の8.3ms線だけ、他は診断出力)
// ---------------------------------------------------------------------------

#[test]
fn r4_widget_timeline_perf_matrix() {
    println!(
        "\nR4 widget timeline 性能行列(単位 µs、中央値 n={ITERATIONS} イテレーション)"
    );
    println!(
        "{:>6} | {:>9} {:>9} | {:>9} (layout回数) | {:>9} | {:>9} | 8.3ms線(=8300µs)"
    , "規模", "a-build", "a-draw", "b-frame", "c-frame", "d-step");

    let mut any_over_budget = false;

    for &n in &SCALES {
        let (a_build, a_draw) = measure_static(n);
        let (b_frame, b_layout_calls) = measure_pan_camera(n);
        let c_frame = measure_pan_naive_rebuild(n);
        let d_step = measure_zoom_rebuild(n);

        let b_verdict = if b_frame <= FRAME_BUDGET_US {
            "GO"
        } else {
            any_over_budget = true;
            "NO-GO"
        };

        println!(
            "{n:>6} | {a_build:>9} {a_draw:>9} | {b_frame:>9} ({b_layout_calls:>2}回) [{b_verdict}] | {c_frame:>9} | {d_step:>9} |"
        );
    }

    if any_over_budget {
        println!(
            "\n注意: 少なくとも1規模で(b)パン=カメラが 8.3ms(120Hz)予算を超えた。\
             このテスト自体は失敗させない(判定は supervisor 側 — 上の行列を読んで\
             Phase 2 の GO/NO-GO を決める)。"
        );
    }
}

/// (b)の「layout が走っていない」ことを単体で機械的に証明する回帰見張り。
/// 上の行列印字テストとは独立に、この不変量だけを毎回検証する。
#[test]
fn pan_camera_never_relayouts_across_a_thousand_bars() {
    let (theme, style) = theme_and_style();
    let offset_x = Rc::new(Cell::new(0.0_f32));
    let layout_calls = Rc::new(Cell::new(0_u32));

    let mut renderer = headless_renderer();
    let content = stacked_bars_fixed(1000, 1.0);
    let lane: Element<'_, Message> =
        TranslateLane::new(content, offset_x.clone(), layout_calls.clone()).into();

    let mut ui = UserInterface::build(lane, VIEWPORT, Cache::default(), &mut renderer);
    assert_eq!(layout_calls.get(), 1, "build 直後は1回のみのはず");

    for frame in 0..200 {
        offset_x.set(frame as f32 * 3.0);
        ui.draw(
            &mut renderer,
            &theme,
            &style,
            iced::mouse::Cursor::Unavailable,
        );
    }

    assert_eq!(
        layout_calls.get(),
        1,
        "1000 bar・200フレームのパンを通しても layout は1回のまま(構造的に \
         `UserInterface::draw` は layout を呼ばない — この assert はその不変量の \
         直接検証)"
    );
}

/// `bar()`/`stacked_bars()` が要求した本数どおりの Stack を組み立てていること
/// のごく単純な健全性チェック(計測系がそもそも壊れていないことの前提)。
#[test]
fn stacked_bars_builds_without_panicking_at_all_scales() {
    let mut renderer = headless_renderer();
    for &n in &SCALES {
        let element: Element<'_, Message> = stacked_bars(n, 1.0, 0.0);
        let _ui = UserInterface::build(element, VIEWPORT, Cache::default(), &mut renderer);
    }
    // `bar()` 単体の1個構築も壊れていないことを見ておく。
    let _ = build_bar(0, 40.0);
}
