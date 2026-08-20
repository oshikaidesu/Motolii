//! ui_scale の柵(発注書: 「fixture で 100%/150%を切り替えて崩れない柵テスト」)。
//!
//! `Shell` は `Tokens` を外から差し替える口を持たない(唯一の書き口は
//! `Message` 経由の `update()` — 発注書の背骨1どおり)。この柵は代わりに
//! `Tokens::default()` を直接組み替えて[`inspector_pane::view`] を直叩きする
//! (pane 関数は `StoreView`/`Session` ではなく投影・`Dimensions`・`Colors` しか
//! 受け取らない設計そのものが、この種の直叩きを可能にしている)。

use motolii_shell::inspector_pane;
use motolii_shell::settings_pane;
use motolii_shell::tokens::{self, Colors, Dimensions, Tokens};
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

// ---------------------------------------------------------------------------
// Settings パネル(タスク#18): ui_scale の書き戻し
// ---------------------------------------------------------------------------
//
// **`tokens/dimensions.json`(正本)そのものは書き換えない** — 複数 worktree・
// 並列試験間で共有される delicate なファイル(`../reference/KNOWN.md`「レーン
// 運用」)。正本の内容を `motolii_testkit::tmp_dir()` の隔離コピーへ写してから
// `tokens::write_ui_scale_to_path` を叩く — 書き戻しロジック([`tokens::
// replace_ui_scale`] のテキスト surgical replace)そのものは正本と同じ形の
// JSON で検分できる。

/// **本命**: 書き戻し後も `_note_*`(コメント代わり、`Dimensions` 構造体に
/// フィールドが無い)を含む他の全キーが1つも消えない。
#[test]
fn writing_ui_scale_to_a_copy_of_the_source_of_truth_preserves_every_other_key() {
    let dir = motolii_testkit::tmp_dir("shell-ui-scale-writeback");
    let path = dir.join("dimensions.json");
    let original = std::fs::read_to_string(Dimensions::debug_source_path())
        .expect("正本 tokens/dimensions.json を読めない(コピー元)");
    std::fs::write(&path, &original).unwrap();

    tokens::write_ui_scale_to_path(&path, 1.5).expect("ui_scale を書き戻せない");

    let rewritten = std::fs::read_to_string(&path).unwrap();
    assert!(
        rewritten.contains("\"ui_scale\": 1.50"),
        "新しい値が反映されていない: {rewritten}"
    );
    assert!(
        rewritten.contains("_note_ui_scale"),
        "_note_ui_scale が消えている(struct 経由の再シリアライズに戻っている疑い)"
    );
    assert!(
        rewritten.contains("_note_row_height"),
        "他フィールドの note が消えている"
    );

    let dims = Dimensions::load_from_path(&path).expect("書き戻した JSON を読めない");
    assert_eq!(dims.ui_scale, 1.5, "書いた ui_scale が反映されていない");

    let baseline = Dimensions::parse(&original).expect("元 JSON を読めない");
    assert_eq!(dims.row_height, baseline.row_height, "無関係なフィールドが変わった");
    assert_eq!(
        dims.inspector_panel_width, baseline.inspector_panel_width,
        "無関係なフィールドが変わった"
    );
}

/// 入力文字列(%) → `ui_scale` への写像: 50..200 クランプ・1%刻み丸め。
/// [`settings_pane::view`] の数値欄が最終的にこの関数を通る
/// (`crate::Shell::commit_ui_scale`)。
#[test]
fn ui_scale_percent_parsing_clamps_and_rounds_to_whole_percent() {
    assert_eq!(settings_pane::parse_ui_scale_percent("100"), Some(1.0));
    assert_eq!(settings_pane::parse_ui_scale_percent("10"), Some(0.5));
    assert_eq!(settings_pane::parse_ui_scale_percent("500"), Some(2.0));
    assert_eq!(settings_pane::parse_ui_scale_percent("garbage"), None);
}
