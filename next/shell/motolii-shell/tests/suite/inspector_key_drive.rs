//! 運転席 — Inspector の Key 列(K1: Inspector からキーフレームを打てる)。
//!
//! `inspector_drive.rs` と同じ流儀(窓を開けずに `Shell` を動かす)。見るのは
//! 発注書の3状態それぞれ: 静的→click でキー1個の track / キー上→click で除去
//! (最後の1個は値を保って静的化)/ track 有り・キー無し→click で評価値のキー
//! 追加。いずれも 1 click = 1 `Intent::SetTrack` = 1 undo。

use motolii_shell::inspector_pane::{self, KeyCellState, KeyRow, RowValue, TransformField};
use motolii_shell::{Message, Shell};

fn shell_with_layer() -> Shell {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    shell
}

/// 選択層の投影から `label` 行の Key 状態を読む。
fn key_state(shell: &Shell, label: &str) -> KeyCellState {
    let selection = shell.inspector_selection().expect("selection");
    selection
        .transform
        .iter()
        .find(|row| row.label == label)
        .unwrap_or_else(|| panic!("{label} 行が投影に無い"))
        .key
        .state
}

/// 選択層の投影から Scale X の(表示単位の)値と editable を読む。
fn scale_x(shell: &Shell) -> (f64, bool) {
    let selection = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale) = &selection.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    (scale[0].value, scale[0].editable)
}

/// **状態1**: 静的(track 無し)→ click = 現在の静的値で playhead 時刻に
/// キー1個の track。undo 1回で静的へ戻る。
#[test]
fn key_click_on_a_static_row_creates_one_key_at_the_playhead_and_undoes_in_one_step() {
    let mut shell = shell_with_layer();
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Static, "新規 layer は静的のはず");

    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));

    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey, "click 直後は playhead 上にキーが有るはず");
    let (value, _) = scale_x(&shell);
    assert_eq!(value, 1.0, "キーの値は現在の静的値(Scale 既定 1.0)のはず");

    // playhead を離れると Between(track は有る・キーは無い)。
    let _ = shell.update(Message::StepPlayhead(5));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Between);

    // 1 click = 1 undo。
    let _ = shell.update(Message::Undo);
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Static, "undo 1回で静的へ戻るはず");
    assert_eq!(shell.layer_count(), 1, "undo が AddLayer まで戻した");
}

/// **状態3**: track 有り・playhead 上にキー無し → click = 現在の評価値で
/// キー追加。2キー以上になった行の値セルは第1波どおり表示のみへ倒れる。
#[test]
fn key_click_between_keys_inserts_the_evaluated_value() {
    let mut shell = shell_with_layer();

    // Scale X を 3.0 へ(静的 hold track)→ frame 10 でキー化。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "3.0".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(
        TransformField::ScaleX,
    )));
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);
    let (value, editable) = scale_x(&shell);
    assert_eq!(value, 3.0, "キー化は現在値を保つはず");
    assert!(editable, "1キーの track は第1波の規則どおり編集可のまま");

    // frame 20 へ動いて2個目 — 評価値(clamp で端の 3.0)のキーが入る。
    let _ = shell.update(Message::StepPlayhead(10));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Between, "キーの外は Between のはず");
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);
    let (value, editable) = scale_x(&shell);
    assert_eq!(value, 3.0);
    assert!(!editable, "2キー以上(animated)の値セルは第1波どおり表示のみのはず");

    // undo 1回で2個目のキーだけが戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Between, "undo 後も1個目のキーは残るはず");
}

/// **状態2**: playhead 上のキーを click で除去。最後の1個なら track ごと
/// 静的化(値は保つ)。undo で戻る。
#[test]
fn key_click_on_the_key_removes_it_and_the_last_removal_goes_static_keeping_the_value() {
    let mut shell = shell_with_layer();

    // Scale X = 2.5 の静的値 → frame 10 でキー化。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "2.5".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(
        TransformField::ScaleX,
    )));
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);

    // 最後の1個を除去 → 静的化・値は 2.5 のまま。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Static, "最後の1個の除去は静的化のはず");
    let (value, editable) = scale_x(&shell);
    assert_eq!(value, 2.5, "静的化は値を保つはず(AE のストップウォッチ解除と等価)");
    assert!(editable, "静的化後は編集可へ戻るはず");

    // undo 1回でキーが戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey, "undo 1回でキーが戻るはず");
}

/// 5行すべての Key 列が結線されている(Q0: 触れそうな物は触れる)。scalar 行
/// (Opacity)も vector 行も同じ動詞。
#[test]
fn every_transform_row_key_cell_is_wired() {
    let mut shell = shell_with_layer();
    let _ = shell.update(Message::StepPlayhead(3));
    for (label, row) in [
        ("Position", KeyRow::Position),
        ("Scale", KeyRow::Scale),
        ("Rotation", KeyRow::Rotation),
        ("Opacity", KeyRow::Opacity),
        ("Anchor", KeyRow::Anchor),
    ] {
        assert_eq!(key_state(&shell, label), KeyCellState::Static, "{label} の初期状態");
        let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(row)));
        assert_eq!(key_state(&shell, label), KeyCellState::AtKey, "{label} が click でキー化しない");
    }
}

/// 選択が無ければ何もしない(黙って無視 — `commit_inspector_field` と同じ柵)。
#[test]
fn key_click_without_a_selection_is_a_no_op() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Position)));
    assert!(!shell.can_undo(), "選択なしの Key click が Intent を出している");
}
