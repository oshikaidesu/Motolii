//! 運転席 — Inspector pane(第1波)。
//!
//! `drive.rs` と同じ流儀(窓を開けずに `Shell` を動かす)。見るのは発注書の3点:
//! 選択→行が出る / 編集→store が変わる / undo 1回で戻る。

use motolii_shell::inspector_pane::{RowValue, TransformField};
use motolii_shell::{Message, Shell};

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

    shell.update(Message::AddLayer);
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
    shell.update(Message::AddLayer);

    let before = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale) = &before.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(scale[0].value, 1.0, "Scale の既定値が1.0でない");
    assert!(scale[0].editable, "un-keyed の Scale が編集不可になっている");

    shell.update(Message::InspectorFieldInput(
        TransformField::ScaleX,
        "2.5".to_owned(),
    ));
    // 打鍵だけでは store は変わらない(1 gesture = 1 undo の前提)。
    let mid = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale_mid) = &mid.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(
        scale_mid[0].value, 1.0,
        "Submit 前なのに store の値が動いている"
    );

    shell.update(Message::InspectorFieldSubmit(TransformField::ScaleX));
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
    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1, "field 編集の Undo が AddLayer まで戻した");
    let reverted = shell.inspector_selection().expect("selection");
    let RowValue::Vector(scale_reverted) = &reverted.transform[1].value else {
        panic!("Scale 行が Vector でない");
    };
    assert_eq!(scale_reverted[0].value, 1.0, "Undo 1回で値が戻らない");
}

/// animated(2キー以上)な property は**表示のみ** — 発注書の指示どおり、
/// 理由つき disabled ではなく「そもそも編集できない」ことを書き口自体でも守る。
#[test]
fn animated_position_gets_the_fixture_keys_and_stays_display_only() {
    let (mut shell, _) = Shell::new_fixture();
    let selection = shell
        .inspector_selection()
        .expect("fixture は1層選択済みのはず");

    let RowValue::Vector(position) = &selection.transform[0].value else {
        panic!("Position 行が Vector でない");
    };
    assert!(
        !position[0].editable,
        "fixture の選択層(サビ歌詞)は position に2キーあるので animated のはず"
    );
    assert!(!position[1].editable);

    // 書き口自体も拒む(UI が control を出していなくても、二重の柵として)。
    let before = position[0].value;
    shell.update(Message::InspectorFieldInput(
        TransformField::PositionX,
        "999".to_owned(),
    ));
    shell.update(Message::InspectorFieldSubmit(TransformField::PositionX));
    let after = shell.inspector_selection().expect("selection");
    let RowValue::Vector(position_after) = &after.transform[0].value else {
        panic!("Position 行が Vector でない");
    };
    assert_eq!(
        position_after[0].value, before,
        "animated な property が書き込めてしまっている"
    );
    assert!(
        shell.status().is_some(),
        "拒否したのに理由が出ていない(M13)"
    );
}

/// **Attrs: Hidden トグル** — 即1回の Intent、undo 1回で戻る。
#[test]
fn toggling_hidden_flips_attrs_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    assert!(!shell.inspector_selection().unwrap().attrs.hidden);

    shell.update(Message::InspectorToggleHidden);
    assert!(shell.inspector_selection().unwrap().attrs.hidden, "toggle が効いていない");

    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1, "toggle の Undo が AddLayer まで戻した");
    assert!(!shell.inspector_selection().unwrap().attrs.hidden, "Undo 1回で戻らない");
}

/// **Attrs: Name の改名** — Enter で確定、undo 1回で戻る。
#[test]
fn renaming_a_layer_commits_on_submit_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");

    shell.update(Message::InspectorNameInput("Rectangle".to_owned()));
    // Submit 前は store が変わらない。
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");

    shell.update(Message::InspectorNameSubmit);
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "Rectangle");

    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 1);
    assert_eq!(shell.inspector_selection().unwrap().attrs.name, "");
}

/// 数値として読めない入力は**黙って消えない**(M13) — 理由が status 帯に出て、
/// store も動かない。
#[test]
fn unparseable_input_is_rejected_with_a_reason() {
    let mut shell = shell();
    shell.update(Message::AddLayer);

    shell.update(Message::InspectorFieldInput(
        TransformField::Rotation,
        "abc".to_owned(),
    ));
    shell.update(Message::InspectorFieldSubmit(TransformField::Rotation));

    assert!(shell.status().is_some(), "拒否理由が出ていない");
    let RowValue::Vector(rotation) = &shell.inspector_selection().unwrap().transform[2].value
    else {
        panic!("Rotation 行が Vector でない");
    };
    assert_eq!(rotation[2].value, 0.0, "書けないはずの値で store が動いている");
}
