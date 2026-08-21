//! 運転席 — layer クリップボード(普通地図 消化第1波 U1、正典 §4)。
//!
//! `drive.rs` と同じ流儀(窓を開けずに `Shell` を動かす)。`clipboard.rs` の
//! `#[cfg(test)]`(capture/instantiate が track・mask・effect・shape・text・slot
//! 参照を keyframe 込みで正しく複製すること)は既にそちらで固定済み — ここで見るのは
//! **`Shell::update` 経由の配線**: 1操作 = 1 undo・選択の遷移・M13(拒否は理由つき)。

use motolii_shell::{Message, Shell};
use motolii_store::LayerId;

fn shell() -> Shell {
    Shell::new().0
}

/// (a) Copy → Paste: 新しい layer が増え、**1 undo** で消える。
#[test]
fn copy_then_paste_adds_one_layer_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    shell.update(Message::CopyLayer);
    assert_eq!(shell.status(), None, "Copy で Document を触っていない=拒否は出ないはず");

    shell.update(Message::PasteLayer);
    assert_eq!(shell.status(), None, "Paste が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 2, "Paste で layer が増えていない");

    shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        1,
        "Undo 1回で Paste が消えない = 1操作が複数 undo になっている"
    );
}

/// (c) Paste 後の選択 = 新しい layer(元の layer ではない)。
#[test]
fn paste_selects_the_new_layer() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let original = shell.session().selection.expect("AddLayer は選択する");

    shell.update(Message::CopyLayer);
    shell.update(Message::PasteLayer);

    let after = shell.session().selection.expect("Paste は選択する");
    assert_ne!(after, original, "Paste 後も元の layer が選ばれたまま");
    assert_eq!(after, LayerId(2), "採番の正本(next_layer_id)を経由していない");
}

/// (b) Cut: layer が消え、**1 undo** で戻る。
#[test]
fn cut_removes_the_layer_and_undoes_in_one_step() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1);

    shell.update(Message::CutLayer);
    assert_eq!(shell.status(), None, "Cut が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 0, "Cut で layer が消えていない");
    assert!(
        shell.session().selection.is_none(),
        "切り取った layer をまだ選んだままにしている"
    );

    shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        1,
        "Undo 1回で Cut が戻らない = 1操作が複数 undo になっている"
    );
}

/// Cut が捕まえた中身は Paste で戻ってくる(Copy と同じ capture 経路を使う)。
#[test]
fn cut_then_paste_restores_a_copy_of_the_layer() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    shell.update(Message::CutLayer);
    assert_eq!(shell.layer_count(), 0);

    shell.update(Message::PasteLayer);
    assert_eq!(shell.status(), None, "Cut 後の Paste が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 1, "Cut した中身が Paste で戻ってこない");
}

/// (e) locked な layer の Cut は拒否+理由(M13)。**Document もクリップボードも
/// 変わらない** — 拒否された Copy 分だけが成立する中途半端を作らない。
#[test]
fn cut_on_a_locked_layer_is_rejected_with_a_reason_and_does_not_touch_the_clipboard() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let layer = shell.session().selection.expect("AddLayer は選択する");
    shell.update(Message::LaneBarToggleLock(layer));
    assert_eq!(shell.status(), None, "lock 自体は常に通るはず");

    shell.update(Message::CutLayer);
    assert!(
        shell.status().is_some(),
        "locked layer の Cut が理由なく通っている(M13 違反)"
    );
    assert_eq!(shell.layer_count(), 1, "locked layer が切り取れてしまっている");

    // クリップボードも書き換わっていないことを、Paste が空を理由に拒否することで確かめる。
    shell.update(Message::PasteLayer);
    assert_eq!(
        shell.status(),
        Some("クリップボードが空"),
        "拒否された Cut のコピーだけがクリップボードへ残ってしまっている"
    );
}

/// Duplicate(Cmd+D): その場複製。**1 undo**、増えた方を選ぶ(正典 §4)。
/// クリップボードには触れない(Copy の中身を上書きしない)。
#[test]
fn duplicate_adds_a_layer_selects_it_and_undoes_in_one_step_without_touching_the_clipboard() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let original = shell.session().selection.expect("AddLayer は選択する");

    // 先に別の layer を Copy しておき、Duplicate がそれを上書きしないことを確かめる。
    shell.update(Message::CopyLayer);

    shell.update(Message::DuplicateLayer);
    assert_eq!(shell.status(), None, "Duplicate が拒否されている: {:?}", shell.status());
    assert_eq!(shell.layer_count(), 2, "Duplicate で layer が増えていない");
    let duplicated = shell.session().selection.expect("Duplicate は選択する");
    assert_ne!(duplicated, original, "複製後に増えた方を選んでいない");

    shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        1,
        "Undo 1回で Duplicate が消えない = 1操作が複数 undo になっている"
    );

    // クリップボードは Copy(original)のままのはず — Paste すれば original の複製が出る。
    shell.update(Message::PasteLayer);
    assert_eq!(shell.status(), None, "Duplicate 後もクリップボードが生きているはず");
    assert_eq!(shell.layer_count(), 2);
}

/// (d) Select All は present な全 layer を選ぶ(fold 未実装なので今は「全部」= 「見えている
/// 全部」)。低レベルの契約(渡された集合しか選ばない)は `clipboard::select_all` の
/// unit test が固定済み — ここでは `Shell` 配線が実際に全 layer を渡していることを見る。
#[test]
fn select_all_selects_every_present_layer() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    shell.update(Message::AddLayer);
    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 3);

    shell.update(Message::SelectAllLayers);
    let mut selected = shell.session().selected_layers.clone();
    selected.sort();
    assert_eq!(selected, vec![LayerId(1), LayerId(2), LayerId(3)]);
    assert!(
        shell.session().selection.is_none(),
        "複数選択に入ったのに単一 focus が残っている"
    );
}

/// Deselect All(正典: 空白クリックと同義)。単一 focus・複数選択の両方を解く。
#[test]
fn deselect_all_clears_both_single_and_multi_selection() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    shell.update(Message::SelectAllLayers);
    assert!(!shell.session().selected_layers.is_empty());

    shell.update(Message::DeselectAllLayers);
    assert!(shell.session().selected_layers.is_empty());
    assert!(shell.session().selection.is_none());
}
