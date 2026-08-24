//! 運転席 — 第6波 shell 結線(B19: `timeline::markers` 統合手順のうち
//! **keymap M=AddAtPlayhead** と **Message::Marker の畳み+JumpTo 先取り**)。
//!
//! **未結線(RETURN 参照)**: canvas 差し替え(`draw_locators` への置き換え)・
//! input 優先順位(ループ帯の後・scrub の前)・実際の drag(`MarkerMessage::
//! Grabbed`/`DragMoved`/`DragReleased`/`DragCancelled` を publish する側)は
//! `motolii-timeline-pane` の `canvas.rs`/`input.rs` が `pub(crate)` のため
//! shell からは触れない。この試験は「意味の配線」(Document 書き込みの
//! 正しさ)だけを検分する — 上記3点は API 要求として RETURN する。
//!
//! fixture(`Shell::new_fixture()`)実測値: marker 3個(Aメロ@150・サビ@510・
//! ラスサビ@1200)、既定 playhead 900(どのマーカーとも重ならない)。

use motolii_shell::{timeline, timeline_pane, Message, Shell};

#[test]
fn m_key_resolves_to_add_at_playhead() {
    use iced::keyboard::{Key, Modifiers};
    let key = Key::Character("m".into());
    let message = motolii_shell::resolve_navigation_key(&key, Modifiers::default(), false);
    assert!(
        matches!(
            message,
            Some(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead))
        ),
        "M キーが Message::Marker(AddAtPlayhead) を出さない: {message:?}"
    );
}

#[test]
fn m_key_is_not_stolen_while_typing() {
    use iced::keyboard::{Key, Modifiers};
    let key = Key::Character("m".into());
    assert!(
        motolii_shell::resolve_navigation_key(&key, Modifiers::default(), true).is_none(),
        "captured=true(rename 等の text 編集中相当)なのに M が発火している"
    );
}

#[test]
fn cmd_m_is_not_stolen_by_the_bare_marker_key() {
    use iced::keyboard::{Key, Modifiers};
    let key = Key::Character("m".into());
    assert!(
        motolii_shell::resolve_navigation_key(&key, Modifiers::COMMAND, false).is_none(),
        "Cmd+M を裸キーの AddAtPlayhead が奪ってしまっている(将来の Cmd+M と衝突する)"
    );
}

#[test]
fn add_at_playhead_adds_exactly_one_marker() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.session().playhead, 900, "fixture の既定 playhead が想定と違う");
    let before = shell.markers().len();

    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead));

    assert_eq!(shell.markers().len(), before + 1, "マーカーが1つ増えていない");
}

#[test]
fn add_at_playhead_undoes_in_one_step() {
    // S2 発注 #22 検収条件(裁定220): 「M キー相当でプレイヘッド位置に
    // マーカーが1つ増え、undo 1回で戻る」。`AddAtPlayhead` は
    // `Intent::SetMarkers` 1回で完結する(`update_marker` 参照)ので
    // undo も1回で足りるはず。
    let mut shell = Shell::new_fixture().0;
    let before = shell.markers().len();

    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead));
    assert_eq!(shell.markers().len(), before + 1, "マーカーが1つ増えていない");

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.markers().len(), before, "undo 1回でマーカー追加前に戻らない");
}

#[test]
fn ruler_right_click_entry_adds_at_playhead_and_undoes_in_one_step() {
    // S2 発注 #22 の2入口目(S6 併存、裁定195): ルーラ locator lane
    // 右クリックが publish する `timeline_pane::Message::AddMarkerAt` は
    // shell の `Message::Timeline` 例外腕(`AddAtPlayhead` と同じ
    // `update_marker` 経路)へ畳まれる。`ruler.rs` は常に `self.playhead`
    // (Premiere/Resolve 先例どおり、クリック位置ではない)を渡すので、この
    // 試験も playhead をそのまま渡して同じ検収条件を確認する。
    let mut shell = Shell::new_fixture().0;
    let before = shell.markers().len();
    let playhead = shell.session().playhead;

    let _ = shell.update(Message::Timeline(timeline_pane::Message::AddMarkerAt(playhead)));
    assert_eq!(
        shell.markers().len(),
        before + 1,
        "ルーラ右クリック相当(AddMarkerAt)でマーカーが1つ増えていない"
    );

    let _ = shell.update(Message::Undo);
    assert_eq!(shell.markers().len(), before, "undo 1回でマーカー追加前に戻らない");
}

#[test]
fn add_at_playhead_twice_on_the_same_frame_collapses_into_one() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.markers().len();

    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead));
    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::AddAtPlayhead));

    assert_eq!(
        shell.markers().len(),
        before + 1,
        "同一フレーム連打が畳まれていない(正典 §5 の明文)"
    );
}

#[test]
fn jump_to_writes_the_playhead_directly() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::JumpTo(510)));
    assert_eq!(shell.session().playhead, 510, "JumpTo が playhead を書いていない(先取りの配線)");
}

#[test]
fn remove_deletes_the_marker_at_the_given_index() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.markers().len();
    let name_at_zero = shell.markers()[0].name.clone();

    let _ = shell.update(Message::Marker(timeline::markers::MarkerMessage::Remove(0)));

    assert_eq!(shell.markers().len(), before - 1, "Remove がマーカーを削除していない");
    assert!(
        shell.markers().iter().all(|m| m.name != name_at_zero || name_at_zero.is_empty()),
        "削除したはずのマーカーがまだ残っている"
    );
}

#[test]
fn marker_panel_rename_commits_and_undoes_in_one_step() {
    let mut shell = Shell::new_fixture().0;
    let before = shell.markers();

    let _ = shell.update(Message::Timeline(timeline_pane::Message::Marker(
        timeline::markers::MarkerMessage::RenameBegin(0),
    )));
    let _ = shell.update(Message::Timeline(timeline_pane::Message::Marker(
        timeline::markers::MarkerMessage::RenameEdited("Chorus".to_owned()),
    )));
    let _ = shell.update(Message::Timeline(timeline_pane::Message::Marker(
        timeline::markers::MarkerMessage::RenameCommit,
    )));

    assert_eq!(shell.markers()[0].name, "Chorus");
    let _ = shell.update(Message::Undo);
    assert_eq!(shell.markers(), before, "改名が undo 1回で元に戻らない");
}
