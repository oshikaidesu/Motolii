//! 運転席 — Timeline ツリー行(裁定173 H2)の Shell 経路。
//!
//! アルゴリズム本体(`attrs.parent` の flatten・fold・インデント深さ・循環の
//! 投影段防衛)は `motolii-timeline-pane` crate 自身の単体テスト
//! (`projection.rs::tree_tests`)が見る — `Shell` は `attrs.parent` を書く
//! 公開の口をまだ持たない(H3「親選択 UI」がその口、今回のスコープ外
//! `NON-GOALS`)ので、ここでは親子 fixture を組めない。
//!
//! ここで見るのは Shell 経由の配線だけ:
//! - `Message::Timeline(timeline_pane::Message::ToggleFold(..))` が
//!   **shell/src の改修ゼロで** `PaneState::update` まで届き、Session の
//!   fold 状態を動かすこと(`Message::Timeline` の5例外に `ToggleFold` を
//!   含めていないので、既存の `other => self.timeline.update(..)` の受け皿が
//!   そのまま拾う — `write.rs` mod doc 参照)
//! - 導入前の見た目(フラットな1 layer = 1行)が並び・件数とも無改造で残ること
//!   (oracle「fold 既定=全展開で現行の見た目不変」)

use motolii_shell::{timeline_pane, Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

/// **オラクル**: parent を持たない既存の layer は、H2 導入前と全く同じ
/// depth=0・has_children=false・children_open=true の行になる(既存 PNG/atlas
/// を壊さない根拠 — `RowProjection` へ新設した3フィールドの既定値が導入前の
/// 見た目と一致することの直接の柵)。
#[test]
fn a_flat_layer_with_no_parent_is_unaffected_by_the_tree_fields() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::AddLayer);

    let rows = shell.timeline_rows();
    assert_eq!(rows.len(), 2, "layer数が変わっている");
    for row in &rows {
        assert_eq!(row.depth, 0, "parentの無いlayerの深さは常に0");
        assert!(!row.has_children, "子を持たないはずのlayerが矢印を出している");
        assert!(row.children_open, "既定は全展開のはず");
    }
}

/// **オラクル(赤→緑)**: `ToggleFold` は shell/src の改修なしで
/// `PaneState::update` まで届く(`Message::Timeline` の5例外に含まれない
/// ので `other` 受け皿がそのまま渡す)。子を持たない layer への ToggleFold は
/// 行を消さない(`has_children == false` の行は fold の影響を受けない)。
#[test]
fn toggle_fold_message_reaches_the_pane_state_through_the_existing_catch_all() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);
    let id = shell.timeline_rows()[0].id;
    // `AddLayer` 自体が undo 可能な Document 操作を積む — この時点の
    // `can_undo()` を基準値として持ち、ToggleFold がそれを動かさないことを
    // 確かめる(「fold は Session 状態 — Document を触らない」の柵)。
    let can_undo_before = shell.can_undo();

    let _ = shell.update(Message::Timeline(timeline_pane::Message::ToggleFold(id)));

    assert_eq!(
        shell.timeline_rows().len(),
        1,
        "子の無いlayerへのToggleFoldで行が消えてはいけない"
    );
    assert_eq!(
        shell.can_undo(),
        can_undo_before,
        "fold はSession状態 — ToggleFoldがUndoスタックを動かしてはいけない"
    );
}

/// `build_timeline_pane()`(pane 自身が実際に描く投影)と `timeline_rows()`
/// (Document 直読みの投影)は、ドラッグ中でなければ常に一致する
/// (`timeline_preview_drive.rs::pane_preview_is_a_passthrough_when_nothing_is_dragging`
/// と同じ柵を、新設したツリー系フィールドについても確認する)。
#[test]
fn pane_projection_matches_shell_projection_for_the_new_tree_fields() {
    let mut shell = shell();
    let _ = shell.update(Message::AddLayer);

    let pane_rows = shell.build_timeline_pane().rows().to_vec();
    let shell_rows = shell.timeline_rows();
    assert_eq!(pane_rows, shell_rows);
}
