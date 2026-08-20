//! ui_scale の柵(発注書: 「fixture で 100%/150%を切り替えて崩れない柵テスト」)。
//!
//! `Shell` は `Tokens` を外から差し替える口を持たない(唯一の書き口は
//! `Message` 経由の `update()` — 発注書の背骨1どおり)。この柵は代わりに
//! `Tokens::default()` を直接組み替えて[`inspector_pane::view`] を直叩きする
//! (pane 関数は `StoreView`/`Session` ではなく投影・`Dimensions`・`Colors` しか
//! 受け取らない設計そのものが、この種の直叩きを可能にしている)。

use motolii_shell::inspector_pane;
use motolii_shell::tokens::{Colors, Dimensions, Tokens};
use motolii_shell::{Message, Shell};

/// 100%/150%それぞれの `Dimensions`(ui_scale 適用済み)と、1層選択済みの投影。
fn selection_and_dims(ui_scale: f32) -> (inspector_pane::SelectionProjection, Dimensions, Colors) {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let selection = shell
        .inspector_selection()
        .expect("AddLayer 直後は選択があるはず");

    let tokens = Tokens {
        ui_scale,
        ..Tokens::default()
    };
    let dims = tokens.dims.scaled(tokens.ui_scale);
    (selection, dims, tokens.colors)
}

/// **本命**: mock の2枚(`--s: 1.00` / `--s: 1.50`)と同じ倍率で、選択あり・
/// 選択なしの両状態を `iced_test::simulator` に通す — layout を実際に組ませて
/// panic しないことを見る(`Simulator::new` は layout を即座に走らせるので、
/// これだけで「組めない」壊れ方は拾える)。
#[test]
fn the_inspector_view_lays_out_without_panicking_at_100_and_150_percent() {
    for ui_scale in [1.0_f32, 1.5_f32] {
        let (selection, dims, colors) = selection_and_dims(ui_scale);

        let with_selection = inspector_pane::view(Some(&selection), None, None, dims, colors);
        let _ = iced_test::simulator(with_selection);

        let empty = inspector_pane::view(None, None, None, dims, colors);
        let _ = iced_test::simulator(empty);
    }
}

/// 150%は100%のちょうど1.5倍(**罫線幅だけ例外**)。適用点([`Dimensions::scaled`])
/// が発注書どおり「全寸法・全文字サイズ」を掛けていることの直接証拠。
#[test]
fn scaling_to_150_percent_grows_every_inspector_dimension_but_the_border() {
    let (_selection_100, dims_100, _colors_100) = selection_and_dims(1.0);
    let (_selection_150, dims_150, _colors_150) = selection_and_dims(1.5);

    let ratio = |scaled: f32, base: f32| (scaled - base * 1.5).abs() < 0.01;

    assert!(ratio(dims_150.title_text, dims_100.title_text));
    assert!(ratio(dims_150.body_text, dims_100.body_text));
    assert!(ratio(dims_150.caption_text, dims_100.caption_text));
    assert!(ratio(dims_150.micro_text, dims_100.micro_text));
    assert!(ratio(dims_150.inspector_panel_width, dims_100.inspector_panel_width));
    assert!(ratio(dims_150.inspector_row_height, dims_100.inspector_row_height));
    assert!(ratio(
        dims_150.inspector_section_header_height,
        dims_100.inspector_section_header_height
    ));
    assert!(ratio(dims_150.inspector_value_width, dims_100.inspector_value_width));
    assert!(ratio(dims_150.inspector_glyph_width, dims_100.inspector_glyph_width));

    // 罫線だけ物理1px床(mock `--line: 1px` — 拡大しない)。100%でも150%でも1.0。
    assert_eq!(dims_100.border_width, 1.0);
    assert_eq!(dims_150.border_width, 1.0);
}
