//! iced_test spike(タスク#15) — `iced_test`(iced公式・実験的な headless テスト crate)
//! が next/ の iced 0.14 に対して実際に動くかの実証。
//!
//! ## 実測結果
//!
//! **動く**。`iced_test = "0.14"`(crates.io、upstream `iced-rs/iced` の
//! `test`/`tester` crate)を dev-dependency に足すだけでビルドが通り、
//! `Simulator::click`/`find`/`point_at`/`simulate`/`into_messages` が
//! そのまま使える(この repo の workspace は iced を fork rev で使っているが、
//! `next/` の iced は crates.io の無印 0.14 系列なので、iced_test も同じ
//! crates.io 系列を選べば版が揃う — fork の `iced_test` rev
//! 73e686e は 0.15.0-dev で別線、next/ には引かない)。
//!
//! ## selector ベースの find/click が効く範囲(実測)
//!
//! `Simulator::find`/`click` は widget が `Widget::operate()` で自ら
//! `widget::Operation` へ登録した候補(`Candidate`)しか見えない
//! (`iced_core::widget::Operation` の `container`/`focusable`/`scrollable`/
//! `text_input`/`text`/`custom` の6種、upstream `test/src/simulator.rs`
//! 0.14.0 タグを実際に読んで確認)。
//!
//! - `button`/`row`/`column`/`container` は自分を `Candidate::Container` として
//!   登録し、中身の `text(...)` は `Candidate::Text` として登録する
//!   (`widget/src/button.rs` `fn operate` 実測)→ header の3ボタンは
//!   ラベル文字列で `find`/`click` できる
//! - **`canvas::Program` と `slider` は `operate()` を一切実装しない**
//!   (`widget/src/canvas.rs`・`widget/src/slider.rs` に `fn operate` が
//!   無いことを実測)→ Timeline pane(丸ごと1枚の canvas)と transport の
//!   scrub slider は selector に**一切現れない**。`find`/`click` の対象外
//!
//! よって Timeline pane への「見つけて click」は selector では組めない
//! (壊れているのではなく、iced 0.14 の `canvas` widget が最初から
//! selector 系 API の対象外という構造的な事実)。代わりに
//! `Simulator::point_at` で座標を直接指定して駆動する — 旧実装
//! `crates/motolii-shell-iced/tests/drive_timeline.rs` と同じ型。
//! Q0 横断柵(`tests/q0_fence.rs`)側の限界节にも同じ制約を明記してある。

use motolii_shell::timeline_pane::{frame_at_x, TimelinePane};
use motolii_shell::tokens::Tokens;
use motolii_shell::{Message, Session, Shell};

fn shell() -> Shell {
    Shell::new().0
}

// ---------------------------------------------------------------------------
// selector ベースの find/click — header の button で実証
// ---------------------------------------------------------------------------

/// **iced_test が iced 0.14 に対して実際に動くことの直接証拠**。壊れていれば
/// この1本がコンパイルすら通らない。
#[test]
fn finding_the_add_layer_button_by_label_and_clicking_it_publishes_add_layer() {
    let shell = shell();
    let mut ui = iced_test::simulator(shell.view());

    ui.click("+ Layer")
        .expect("\"+ Layer\" ボタンが text selector で見つからない");

    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        matches!(messages.as_slice(), [Message::AddLayer]),
        "click が期待した Message::AddLayer を出していない: {messages:?}"
    );
}

/// 文脈disabled(`on_press_maybe(None)`)は **selector には見つかる**(button
/// 自体は on_press の有無に関係なく `Candidate::Container` として登録される
/// — 実測)が、click しても Message は出ない。「見つかる = 押せる」ではない
/// ことの実例(Q0 柵側の判定ルールがこの区別に依存している)。
#[test]
fn a_context_disabled_button_is_findable_but_silent() {
    let shell = shell();
    assert!(!shell.can_undo(), "この前提が崩れると次のassertが無意味");

    let mut ui = iced_test::simulator(shell.view());
    ui.click("Undo")
        .expect("Undo ボタンは on_press が無くても text selector で見つかる");

    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        messages.is_empty(),
        "文脈disabledのUndoが押せてしまっている: {messages:?}"
    );
}

// ---------------------------------------------------------------------------
// Timeline canvas — selectorでは見つからないので座標を直接指定
// ---------------------------------------------------------------------------

/// comp 全体(0..300)を占有する層1枚だけの Timeline を組み立てる。
/// `Shell` を経由しない — `Shell` は `doc`/`session` を外へ出さないので、
/// pane 単体を狙うテストは `motolii_store` を直接使うしかない
/// (`Shell::update(AddLayer)` と同じ intent 列)。
fn one_layer_timeline() -> (motolii_store::Document, Session) {
    use motolii_store::{Composition, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming};

    let mut doc = motolii_store::Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 300,
    }))
    .expect("comp を置ける");
    doc.mark_undo_floor();

    let id = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(id),
        Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [80, 160, 220, 255],
                    width: 240,
                    height: 135,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .expect("layer を置ける");

    (doc, Session::default())
}

/// `Simulator::new` の既定サイズ(upstream の doc comment に明記: 1024x768)。
/// selector が効かない canvas を座標で狙うので、この数値そのものが前提になる。
const DEFAULT_WIDTH: f32 = 1024.0;

#[test]
fn clicking_a_bar_in_the_timeline_canvas_publishes_select_via_raw_coordinates() {
    let (doc, session) = one_layer_timeline();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors);

    let mut ui = iced_test::simulator(pane.view());

    // layer は 0..300(=comp全体)を占有するので、行0の中央 x はどこでも bar の上。
    let x = DEFAULT_WIDTH / 2.0;
    let y = tokens.dims.row_height + tokens.dims.row_height / 2.0; // ルーラー帯の直後、行0の中央
    ui.point_at(iced::Point::new(x, y));
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);

    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        matches!(messages.as_slice(), [Message::Select(id)] if *id == motolii_store::LayerId(1)),
        "canvas 上の bar click が Message::Select を出していない: {messages:?}"
    );
}

#[test]
fn clicking_the_ruler_band_publishes_scrub_to_via_raw_coordinates() {
    let (doc, session) = one_layer_timeline();
    let tokens = Tokens::default();
    let store = doc.view();
    let pane = TimelinePane::new(&store, &session, tokens.dims, tokens.colors);

    let x = 200.0_f32;
    // 期待値は timeline_pane 自身の純粋関数から出す(数値をここで再発明しない —
    // `frame_at_x` は drive.rs の `timeline_scrub_math_maps_pixel_to_frame` が
    // 別途単体で見ている)。
    let expected_frame = frame_at_x(x, DEFAULT_WIDTH, 300);

    let mut ui = iced_test::simulator(pane.view());
    ui.point_at(iced::Point::new(x, 5.0)); // ルーラー帯(高さ = row_height の内側)は常に scrub
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);

    let messages: Vec<_> = ui.into_messages().collect();
    assert!(
        matches!(messages.as_slice(), [Message::ScrubTo(frame)] if *frame == expected_frame),
        "canvas 上のルーラー click が期待した Message::ScrubTo({expected_frame}) を出していない: {messages:?}"
    );
}
