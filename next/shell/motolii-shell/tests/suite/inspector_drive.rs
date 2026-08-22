//! 運転席 — Inspector pane(第1波 + drag-to-scrub)。
//!
//! `drive.rs` と同じ流儀(窓を開けずに `Shell` を動かす)。見るのは発注書の3点:
//! 選択→行が出る / 編集→store が変わる / undo 1回で戻る。
//!
//! drag-to-scrub(利用者の直接依頼)は同じ流儀で、window 全体から拾う
//! `Message::Inspector(inspector_pane::Message::PointerMoved/PointerReleased)`/
//! `KeyboardModifiersChanged`/`EscapePressed` を直接送って検証する
//! (`lib.rs::inspector_pointer_event` が実際に window から拾うのと同じ形の
//! Message を、実 window を開かずに直送する)。

use motolii_shell::inspector_pane::{self, RowValue, TransformField};
use motolii_shell::{Message, Shell};
use motolii_store::{property, PropertyId};

fn shell() -> Shell {
    Shell::new().0
}

/// **選択→行が出る**。`AddLayer` は置いた layer を選択する(lib.rs)ので、
/// 直後に Transform 5行(Position/Scale/Rotation/Opacity/Anchor)と Attrs が並ぶ。
#[test]
fn selecting_a_layer_shows_transform_and_attrs_rows() {
    let mut shell = shell();
    assert!(
        shell.inspector_selection().is_none(),
        "選択が無いのに投影が出ている"
    );

    let _ = shell.update(Message::AddLayer);
    let selection = shell
        .inspector_selection()
        .expect("AddLayer 直後は選択があるので投影が出るはず");

    let labels: Vec<&str> = selection.transform.iter().map(|row| row.label).collect();
    assert_eq!(
        labels,
        vec!["Position", "Scale", "Rotation", "Opacity", "Anchor"],
        "Transform 行の並びが発注書の指定と違う"
    );
    // 新規 layer は attrs 未設定なので name は空(既定)。
    assert_eq!(selection.attrs.name, "");
    assert!(!selection.attrs.hidden);
    // ident 帯の種別 — `AddLayer`(lib.rs)は `LayerSource::Solid` を置く。
    assert_eq!(selection.kind, "solid", "ident 帯の種別ラベルが実データと違う");
}

/// **編集→store が変わる / undo 1回で戻る**(静的値、Scale X)。
#[test]
fn editing_a_static_scale_field_writes_a_single_hold_keyframe_and_undoes_in_one_step() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let before = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale) = &before.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(scale[0].value, 1.0, "Scale の既定値が1.0でない");
    assert!(scale[0].editable, "un-keyed の Scale が編集不可になっている");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::ScaleX,
        "2.5".to_owned(),
    )));
    // 打鍵だけでは store は変わらない(1 gesture = 1 undo の前提)。
    let mid = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale_mid) = &mid.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(
        scale_mid[0].value, 1.0,
        "Submit 前なのに store の値が動いている"
    );

    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(TransformField::ScaleX)));
    let after = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale_after) = &after.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(scale_after[0].value, 2.5, "Submit で store が変わらない");
    assert_eq!(
        scale_after[1].value, 1.0,
        "X を編集したら Y まで動いた(他成分を保っていない)"
    );
    assert!(shell.can_undo());

    // **1操作 = 1 undo** — AddLayer と field 編集は別操作なので、Undo 1回で
    // field 編集**だけ**が戻り、layer 自体は残る。
    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1, "field 編集の Undo が AddLayer まで戻した");
    let reverted = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale_reverted) = &reverted.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(scale_reverted[0].value, 1.0, "Undo 1回で値が戻らない");
}

/// キー持ち(2キー以上)な property も**編集可能**(Q0、2026-08-22 発注 —
/// 旧規則「animated は表示のみ」を撤去)。編集の意味は playhead へのキー
/// upsert: fixture のサビ歌詞 position(キー 510/570、playhead=900)を編集
/// すると frame 900 に3個目のキーが増え、既存キーは無傷。
#[test]
fn editing_an_animated_position_upserts_a_key_at_the_playhead() {
    let (mut shell, _) = Shell::new_fixture();
    let selection = shell
        .inspector_selection()
        .expect("fixture は1層選択済みのはず");
    let layer = selection.layer;

    let RowValue::Vector(position) = &selection.transform[0].value else {
        panic!("Position 行が Vector でない");
    };
    assert!(
        position[0].editable,
        "キー持ち(2キー)の position も編集可能のはず(Q0)"
    );
    assert!(position[1].editable);
    assert!(position[0].keyed, "実キー持ちの投影は keyed のはず(accent 表示の合図)");
    let before_y = position[1].value;

    let position_property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let keys_before = shell
        .store_view()
        .track(layer, &position_property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0);
    assert_eq!(keys_before, 2, "fixture の前提(position 2キー)が崩れている");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::PositionX,
        "999".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(TransformField::PositionX)));
    assert!(shell.status().is_none(), "編集が拒否されている: {:?}", shell.status());

    let keys_after = shell
        .store_view()
        .track(layer, &position_property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0);
    assert_eq!(keys_after, 3, "playhead(キーの無い時刻)の編集はキーを増やすはず");

    let after = shell.inspector_selection().expect("selection");
    let RowValue::Vector(position_after) = &after.transform[0].value else {
        panic!("Position 行が Vector でない");
    };
    assert_eq!(position_after[0].value, 999.0, "playhead のキー値が編集値になっていない");
    assert_eq!(position_after[1].value, before_y, "X の編集で Y まで動いた");

    // 1編集 = 1 undo。
    let _ = shell.update(Message::Undo);
    let keys_reverted = shell
        .store_view()
        .track(layer, &position_property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0);
    assert_eq!(keys_reverted, 2, "undo 1回で編集前(キー2個)へ戻らない");
}

/// **Attrs: Hidden トグル** — 即1回の Intent、undo 1回で戻る。
#[test]
fn toggling_hidden_flips_attrs_and_undoes_in_one_step() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert!(!shell.inspector_selection().unwrap().attrs.hidden);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ToggleHidden));
    assert!(shell.inspector_selection().unwrap().attrs.hidden, "toggle が効いていない");

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1, "toggle の Undo が AddLayer まで戻した");
    assert!(!shell.inspector_selection().unwrap().attrs.hidden, "Undo 1回で戻らない");
}

/// **Attrs: Name の改名** — Enter で確定、undo 1回で戻る。
#[test]
fn renaming_a_layer_commits_on_submit_and_undoes_in_one_step() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::NameInput("Rectangle".to_owned())));
    // Submit 前は store が変わらない。
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::NameSubmit));
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "Rectangle");

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1);
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");
}

/// 数値として読めない入力は**黙って消えない**(M13) — 理由が status 帯に出て、
/// store も動かない。
#[test]
fn unparseable_input_is_rejected_with_a_reason() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldInput(
        TransformField::Rotation,
        "abc".to_owned(),
    )));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::FieldSubmit(TransformField::Rotation)));

    assert!(shell.status().is_some(), "拒否理由が出ていない");
    let RowValue::Vector(rotation) = &shell.inspector_selection().unwrap().transform[2].value
    else {
        panic!("Rotation 行が Vector でない");
    };
    assert_eq!(rotation[2].value, 0.0, "書けないはずの値で store が動いている");
}

// ---------------------------------------------------------------------------
// drag-to-scrub(利用者の直接依頼)
// ---------------------------------------------------------------------------

fn scale_x(shell: &Shell) -> f64 {
    let RowValue::Vector(scale) = &shell.inspector_selection().unwrap().transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    scale[0].value
}

fn scale_y(shell: &Shell) -> f64 {
    let RowValue::Vector(scale) = &shell.inspector_selection().unwrap().transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    scale[1].value
}

/// **drag で値が変わり undo 1回で戻る**(発注書の柵そのもの)。途中の複数手
/// (最初の move は基準確定のみ・以降が実際の drag)は history を1件に畳んで
/// いる — Undo 1回で press 前まで戻ることで検証する。他成分(Y)も保つ。
#[test]
fn dragging_a_value_cell_updates_the_store_live_and_commits_on_release_with_one_undo() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert_eq!(scale_x(&shell), 1.0);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::ScaleX)));
    // 最初の move は基準確定のみ — まだ値は動かない。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(100.0, 0.0))));
    assert_eq!(scale_x(&shell), 1.0, "基準確定の move で値が動いている");

    // +10px * 0.01(Scale の感度) = +0.1、複数手でも history は畳まれて1件。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(110.0, 0.0))));
    assert!((scale_x(&shell) - 1.1).abs() < 1e-9, "1手目の drag で値が動いていない");
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(130.0, 0.0))));
    assert!((scale_x(&shell) - 1.3).abs() < 1e-9, "2手目の drag で値が動いていない");
    assert_eq!(scale_y(&shell), 1.0, "動かしていない Y まで動いた");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerReleased));
    assert!((scale_x(&shell) - 1.3).abs() < 1e-9, "release で値が変わってしまった");
    assert!(shell.can_undo());

    // **1 gesture = 1 undo**: 複数 move があっても Undo 1回で press 前まで戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1, "drag の Undo が AddLayer まで戻した");
    assert_eq!(scale_x(&shell), 1.0, "Undo 1回で drag 前の値へ戻らない");
}

/// **Shift+drag = 1/10 微調整**。
#[test]
fn shift_held_during_a_drag_uses_a_tenth_sensitivity() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::ScaleX)));
    let _ = shell.update(Message::KeyboardModifiersChanged(
        iced::keyboard::Modifiers::SHIFT,
    ));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(0.0, 0.0))));
    // +10px * 0.01 * 1/10 = +0.01。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(10.0, 0.0))));
    assert!(
        (scale_x(&shell) - 1.01).abs() < 1e-9,
        "Shift+drag が1/10感度になっていない: {}",
        scale_x(&shell)
    );
}

/// **Esc で復元**。drag 中に Esc を押すと press 前の値まで戻り、AddLayer 以外の
/// 痕跡を undo 履歴に残さない(Undo 1回で AddLayer 自体が戻る = drag は
/// history に何も足していない)。
#[test]
fn escape_during_a_drag_restores_the_original_value_and_leaves_no_undo_trace() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::PositionX)));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(0.0, 0.0))));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(50.0, 0.0))));
    let RowValue::Vector(position) = &shell.inspector_selection().unwrap().transform[0].value
    else {
        panic!("Position 行が Vector でない");
    };
    assert_eq!(position[0].value, 50.0, "drag 中なのに値が動いていない");

    let _ = shell.update(Message::EscapePressed);
    let RowValue::Vector(position_after) =
        &shell.inspector_selection().unwrap().transform[0].value
    else {
        panic!("Position 行が Vector でない");
    };
    assert_eq!(position_after[0].value, 0.0, "Esc で元の値へ戻らない");

    // drag が history に痕跡を残していなければ、Undo 1回で AddLayer 自体が戻る。
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        0,
        "Esc で中断した drag が undo 履歴に何か残している"
    );
}

/// **click(ドラッグせず release)は従来どおり type 編集へ**。move が一度も
/// 無ければ store は動かず、`inspector_field_draft` が該当 field で立つ。
#[test]
fn clicking_a_value_cell_without_dragging_enters_type_editing() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    assert!(shell.inspector_field_draft().is_none());
    // AddLayer 自体は undo 可能な1手 — click がそれ「以上」を足さないことを
    // この後の layer_count(Undo すると AddLayer まで戻る)で確かめる。
    assert!(shell.can_undo(), "AddLayer 直後は undo 可能なはず(前提)");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::Rotation)));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerReleased));

    let draft = shell
        .inspector_field_draft()
        .expect("click は type 編集への切り替えのはず");
    assert_eq!(draft.field, TransformField::Rotation);
    assert_eq!(draft.text, "0.0", "下書きの初期値が今の store 値と違う");

    // click だけでは store は動いていない — Undo 1回で AddLayer 自体が戻る
    // (click が別の undo entry を足していれば layer は残ってしまう)。
    let RowValue::Vector(rotation) = &shell.inspector_selection().unwrap().transform[2].value
    else {
        panic!("Rotation 行が Vector でない");
    };
    assert_eq!(rotation[2].value, 0.0);
    let _ = shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "click だけで別の undo entry が足されている");
}

// ---------------------------------------------------------------------------
// transient overlay 化(旧ハック撤去)の落とし穴 — 履歴の無傷性
// ---------------------------------------------------------------------------

/// **Esc → undo 履歴が完全に無傷**。旧実装の `cancel_inspector_interaction` は
/// 「同じ値で1回上書きしてから undo」して辻褄を合わせていたが、この無害化は
/// `tip`(redo 境界)を1つ先へ進めたまま残す — `can_undo` だけを見ると
/// drag 開始前と同じに見えてしまうが、`can_redo` を見ると drag していないのに
/// 「redo できる」という嘘の状態が残る(何もない所へ redo すると幽霊の
/// no-op エントリが生える)。transient overlay は `Document` の edit timeline に
/// 一切触れないので、drag→Esc の前後で `can_redo` も含めて完全に無傷のはず。
#[test]
fn escape_during_a_drag_leaves_undo_history_completely_untouched() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let can_undo_before = shell.can_undo();
    let can_redo_before = shell.can_redo();
    assert!(can_undo_before, "AddLayer 直後は undo 可能なはず(前提)");
    assert!(!can_redo_before, "drag 前から redo できてしまっている(前提が崩れている)");

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::ScaleX)));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(0.0, 0.0))));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(50.0, 0.0))));
    assert!((scale_x(&shell) - 1.5).abs() < 1e-9, "drag 中なのに値が動いていない");

    let _ = shell.update(Message::EscapePressed);

    assert_eq!(
        shell.can_undo(),
        can_undo_before,
        "Esc の後で can_undo が drag 開始前と違う"
    );
    assert_eq!(
        shell.can_redo(),
        can_redo_before,
        "Esc の後で can_redo が drag 開始前と違う — 履歴に幽霊エントリが残っている"
    );
}

/// **ドラッグ確定 → undo 1回で開始前の値へ戻り、中間値は履歴に残らない**。
/// commit 後に Undo→Redo を1往復させ、Redo で戻ってくる値が最後の move の値
/// (中間値ではなく確定値そのもの)であることまで確かめる — 途中経過が history
/// のどこにも紛れ込んでいないことの直接証拠。
#[test]
fn committing_a_drag_undoes_in_one_step_with_no_intermediate_value_in_history() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let start = scale_x(&shell);
    assert_eq!(start, 1.0);

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::ScaleX)));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(0.0, 0.0))));
    // 中間値 1.1 を経由する。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(10.0, 0.0))));
    assert!((scale_x(&shell) - 1.1).abs() < 1e-9, "中間 move で値が動いていない");
    // 最終値 1.3 で確定する。
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(30.0, 0.0))));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerReleased));
    assert!((scale_x(&shell) - 1.3).abs() < 1e-9, "release で確定値になっていない");

    // Undo 1回で drag 前まで戻る(中間値 1.1 では止まらない)。
    let _ = shell.update(Message::Undo);
    assert_eq!(scale_x(&shell), start, "Undo 1回で drag 前の値へ戻らない");

    // Redo で戻る先は確定値 1.3 そのもの — 中間値 1.1 が history に紛れて
    // いれば、ここが 1.1 になったり、Redo が複数回要る形で壊れるはず。
    let _ = shell.update(Message::Redo);
    assert!(
        (scale_x(&shell) - 1.3).abs() < 1e-9,
        "Redo で確定値(1.3)へ戻らない: {}",
        scale_x(&shell)
    );
}

/// キー持ち(2キー以上)な field も drag できる(Q0、2026-08-22 発注 — 旧規則
/// 「animated は drag も始まらない」を撤去)。確定は値セル Enter と同経路の
/// キー upsert(`inspector_pane::edited_value_track`)— playhead(fixture=900、
/// キーの無い時刻)へキーが増え、1 gesture = 1 undo。
#[test]
fn dragging_a_keyed_field_commits_an_upserted_key() {
    let (mut shell, _) = Shell::new_fixture();
    let selection = shell.inspector_selection().expect("fixture は選択済みのはず");
    let layer = selection.layer;
    let RowValue::Vector(position) = &selection.transform[0].value else {
        panic!("Position 行が Vector でない");
    };
    assert!(position[0].editable, "キー持ち field も drag の起点になるはず(Q0)");
    let before = position[0].value;

    let _ = shell.update(Message::Inspector(inspector_pane::Message::ValuePressed(TransformField::PositionX)));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(0.0, 0.0))));
    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerMoved(iced::Point::new(50.0, 0.0))));

    let RowValue::Vector(position_moved) =
        &shell.inspector_selection().unwrap().transform[0].value
    else {
        panic!("Position 行が Vector でない");
    };
    assert_eq!(
        position_moved[0].value,
        before + 50.0,
        "キー持ち field の drag が動かない(旧規則の再発)"
    );

    let _ = shell.update(Message::Inspector(inspector_pane::Message::PointerReleased));
    let position_property = PropertyId::new(property::POSITION).expect("position は予約語ではない");
    let keys_after = shell
        .store_view()
        .track(layer, &position_property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0);
    assert_eq!(keys_after, 3, "drag 確定は playhead へキーを増やすはず(2→3)");

    // 1 gesture = 1 undo。
    let _ = shell.update(Message::Undo);
    let keys_reverted = shell
        .store_view()
        .track(layer, &position_property)
        .expect("track を読めるはず")
        .map(|track| track.keys().len())
        .unwrap_or(0);
    assert_eq!(keys_reverted, 2, "undo 1回で drag 前(キー2個)へ戻らない");
}
