//! 運転席 — Inspector の Key 列(K1: Inspector からキーフレームを打てる)。
//!
//! `inspector_drive.rs` と同じ流儀(窓を開けずに `Shell` を動かす)。見るのは
//! 発注書の3状態それぞれ: 静的→click でキー1個の track / キー上→click で除去
//! (最後の1個は値を保って静的化)/ track 有り・キー無し→click で評価値のキー
//! 追加。いずれも 1 click = 1 `Intent::SetTrack` = 1 undo。

use motolii_shell::inspector_pane::{self, KeyCellState, KeyRow, RowValue, TransformField};
use motolii_shell::{Message, Shell};
use motolii_store::{property, PropertyId};

fn shell_with_layer() -> Shell {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    shell
}

/// 選択層の Scale track のキー個数を store から直接数える(正準静的表現 =
/// 1キー Hold @ZERO も1個と数える — 各テストが状態と併せて意味を判定する)。
fn scale_key_count(shell: &Shell) -> usize {
    let layer = shell.inspector_selection().expect("selection").layer;
    let property = PropertyId::new(property::SCALE).expect("scale は予約語ではない");
    shell
        .store_view()
        .track(layer, &property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0)
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
/// キー追加。値セルはキー数に関わらず編集可能のまま(Q0、2026-08-22 発注 —
/// 旧規則「2キー以上は表示のみ」は撤去)。
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
    assert!(editable, "2キー以上でも値セルは編集可能のまま(Q0 — 編集は playhead への upsert)");

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

// ---------------------------------------------------------------------------
// AE 作法: キー持ち track の値編集 = playhead 位置へのキー upsert
// (利用者実窓指摘「キーが1つしか打てない」の根治、2026-08-22 発注)
// ---------------------------------------------------------------------------

/// **(a) 利用者シナリオの再現**: キー打ち → playhead 移動 → 値セル編集 →
/// **キーが2個**。旧実装はここで track 全体が `single_hold_track`(静的)に
/// 置き換わってキーが消えていた(この assert が旧コードで red になる再現)。
/// (d) undo 可逆: 1 undo で編集前(キー1個)へ、redo で2個へ戻る。
#[test]
fn editing_a_value_after_moving_the_playhead_adds_a_second_key() {
    let mut shell = shell_with_layer();

    // frame 10 でキー化(既存 K1 の動詞)。
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(scale_key_count(&shell), 1);
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);

    // frame 20 へ移動して値セルを編集 — キーが**増える**(AE 文法)。
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "3.0".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(
        TransformField::ScaleX,
    )));
    assert!(shell.status().is_none(), "値編集が拒否されている: {:?}", shell.status());
    assert_eq!(
        scale_key_count(&shell),
        2,
        "キー持ち track の値編集は playhead へキーを増やすはず(track を静的に戻さない)"
    );
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey, "playhead(frame 20)にキーが有るはず");
    let (value, editable) = scale_x(&shell);
    assert_eq!(value, 3.0, "frame 20 のキー値は編集値のはず");
    assert!(editable, "値セルはキー数に関わらず常に編集可能のはず(Q0)");

    // frame 10 の既存キーは無傷。
    let _ = shell.update(Message::StepPlayhead(-10));
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);
    let (value, _) = scale_x(&shell);
    assert_eq!(value, 1.0, "frame 10 の既存キーの値が変わっている");

    // (d) 1編集 = 1 undo。redo で戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(scale_key_count(&shell), 1, "undo 1回で編集前(キー1個)へ戻るはず");
    let _ = shell.update(Message::Redo);
    assert_eq!(scale_key_count(&shell), 2, "redo でキー2個へ戻るはず");
}

/// **(b) playhead 上での値編集 = 既存キーの値更新**(個数不変・track は
/// キー持ちのまま — 静的化しない)。
#[test]
fn editing_a_value_on_an_existing_key_updates_it_in_place() {
    let mut shell = shell_with_layer();
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    assert_eq!(scale_key_count(&shell), 1);

    // playhead を動かさず値を編集 — キーは増えず、値だけ更新。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "2.5".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(
        TransformField::ScaleX,
    )));
    assert_eq!(scale_key_count(&shell), 1, "playhead 上の編集はキー個数を変えないはず");
    assert_eq!(
        key_state(&shell, "Scale"),
        KeyCellState::AtKey,
        "編集後も track はキー持ちのまま(静的化しない)のはず"
    );
    let (value, _) = scale_x(&shell);
    assert_eq!(value, 2.5, "既存キーの値が更新されているはず");
}

/// **(c) 静的プロパティの値編集はキーを生やさない**(従来どおり静的値の変更 —
/// キー化は Key 列 click が明示的に行う)。
#[test]
fn editing_a_static_value_does_not_sprout_keys() {
    let mut shell = shell_with_layer();
    let _ = shell.update(Message::StepPlayhead(15));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "4.0".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(
        TransformField::ScaleX,
    )));
    assert_eq!(
        key_state(&shell, "Scale"),
        KeyCellState::Static,
        "静的プロパティの値編集でキーが生えている"
    );
    let (value, editable) = scale_x(&shell);
    assert_eq!(value, 4.0);
    assert!(editable);
}

/// **(e) 数値ドラッグの確定も同経路**: キー持ち track の drag-to-scrub 確定は
/// playhead へのキー upsert(1 gesture = 1 undo)。旧実装は animated を
/// 編集不可に落として drag 自体が始まらなかった。
#[test]
fn dragging_a_keyed_value_commits_an_upserted_key_with_one_undo() {
    let mut shell = shell_with_layer();
    let _ = shell.update(Message::StepPlayhead(10));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
    let _ = shell.update(Message::StepPlayhead(10)); // frame 20
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::Between);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(
        TransformField::ScaleX,
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(
        iced::Point::new(0.0, 0.0),
    )));
    // +10px * 0.01(Scale の感度)= +0.1 → 1.1。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(
        iced::Point::new(10.0, 0.0),
    )));
    let (value, _) = scale_x(&shell);
    assert!((value - 1.1).abs() < 1e-9, "キー持ち track の drag が動いていない: {value}");
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerReleased));

    assert_eq!(scale_key_count(&shell), 2, "drag 確定は playhead へキーを増やすはず");
    assert_eq!(key_state(&shell, "Scale"), KeyCellState::AtKey);
    let (value, _) = scale_x(&shell);
    assert!((value - 1.1).abs() < 1e-9, "確定値が frame 20 のキーに入っていない: {value}");

    // frame 10 の既存キーは無傷。
    let _ = shell.update(Message::StepPlayhead(-10));
    let (value, _) = scale_x(&shell);
    assert_eq!(value, 1.0, "frame 10 の既存キーの値が変わっている");

    // 1 gesture = 1 undo。
    let _ = shell.update(Message::Undo);
    assert_eq!(scale_key_count(&shell), 1, "undo 1回で drag 前(キー1個)へ戻るはず");
}

/// 複数時刻での Key click がキーを積める(既存 K1 の3状態が今回の変更で
/// 壊れていないことの再確認 — 発注書やること3)。
#[test]
fn key_clicks_at_multiple_frames_stack_keys() {
    let mut shell = shell_with_layer();
    for frame in [5, 10, 15] {
        let _ = shell.update(Message::StepPlayhead(5));
        let _ = shell.update(Message::Inspector(inspector_pane::Message::KeyPressed(KeyRow::Scale)));
        assert_eq!(
            key_state(&shell, "Scale"),
            KeyCellState::AtKey,
            "frame {frame} で click がキーを打てていない"
        );
    }
    assert_eq!(scale_key_count(&shell), 3, "3回の click でキーが3個積まれるはず");
}
