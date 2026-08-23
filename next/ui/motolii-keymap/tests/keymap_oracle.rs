//! crate 単体の oracle。**`global_bindings`(Escape/Backspace・Delete/Alt+矢印/
//! Shift+F)は shell 側の `inspector_pointer_event` が `pub` でないため
//! 直接突き合わせられない**(RETURN の逸脱台帳参照)——ここでは自己無矛盾
//! (件数・衝突ゼロ・文脈の効き)だけを検分する。`nav_bundle_bindings` の
//! shell との一致証明は `next/shell/motolii-shell/tests/suite/
//! keymap_equivalence.rs` を見よ。

use motolii_keymap::{default_bindings, default_keymap, global_bindings, nav_bundle_bindings, Key, Modifiers, NamedKey, Scope, VerbId};

/// 発注書「既定バインディングの表: 現在shellに直書きされている割当を全部
/// この表へ写す」の件数チェック——`lib.rs` を読んで数えた実測値
/// (`defaults.rs` 冒頭 doc のコメントに1行ずつ出典を書いてある)。
#[test]
fn binding_counts_match_the_transcribed_shell_allocation() {
    // 裁定208 の無主レーンで shell 実装に追随(`defaults.rs::nav_bundle_bindings`
    // 各コメント参照): (a) Shift+Home/End(JumpToWorkAreaStart/End)で Home/End
    // がそれぞれ shift 分岐の2本に割れ 26→28 (b) JKL シャトルで旧 j/k の
    // JumpMeaningPoint 4本を退役させ ShuttleReverse/Stop の2本(j/k 各1)へ
    // 差し替え、退役した動詞は `,`/`.` の4本で受け直す(28-4+2+4=30) (c) bare
    // n(SetWorkAreaOut)を1本追加(30→31)。
    assert_eq!(global_bindings().len(), 8, "global 側の行数が実測(8)とずれている");
    // (d) 2026-08-23 C-1: 平の Save(Cmd+S・Shift無し)を1本追加(31→32)。
    //     Cmd+Shift+S=SaveAs との振り分けは shell の `resolve_navigation_key` と同型。
    assert_eq!(nav_bundle_bindings().len(), 32, "nav bundle 側の行数が実測(32)とずれている");
    assert_eq!(default_bindings().len(), 40);
}

#[test]
fn default_keymap_builds_without_panicking() {
    let _ = default_keymap();
}

/// 衝突検出の形(発注書 やること3・5「衝突検出」)。既定表を汚さず、
/// 局所的に矛盾する2本を足して `Keymap::build` が `Err` を返すことを見る。
#[test]
fn conflict_detection_catches_a_deliberately_introduced_clash() {
    use motolii_keymap::{Binding, Keymap, ModifierSpec};

    let mut bindings = nav_bundle_bindings();
    // 既存の「Cmd+Z(shift無し)→Undo」と丸かぶりする「Cmd+Z→Redo」を追加。
    bindings.push(Binding::new(
        Key::character('z'),
        ModifierSpec::ANY.command_required(true).shift_required(false),
        Scope::NavigationBundle,
        VerbId::Redo,
    ));

    let err = Keymap::build(bindings).expect_err("わざと重ねた binding が衝突検出されない");
    assert!(err.iter().any(|conflict| conflict.key == Key::character('z')));
}

/// 文脈の効き(発注書 やること5「文脈の効き・テキスト入力中の無効化」)。
#[test]
fn text_input_capture_disables_navigation_bundle_but_not_global() {
    let keymap = default_keymap();

    // Global: Escape は captured=true でも発火する。
    assert_eq!(
        keymap.resolve(Key::Named(NamedKey::Escape), Modifiers::NONE, true),
        Some(VerbId::EscapeCancel)
    );

    // NavigationBundle: Home は captured=true だと発火しない。
    assert_eq!(keymap.resolve(Key::Named(NamedKey::Home), Modifiers::NONE, true), None);
    assert_eq!(
        keymap.resolve(Key::Named(NamedKey::Home), Modifiers::NONE, false),
        Some(VerbId::JumpPlayheadToStart)
    );

    // Backspace/Delete(Timeline キー削除)は `inspector_pointer_event` の
    // 早期 match 腕でありながら `Scope::NavigationBundle`(2026-08-23 修正の
    // 回帰柵 — AX-5 実測の二重発火穴。captured=true で発火したら Document
    // 側が text_input の文字削除と同時に選択キーフレームを消す)。
    assert_eq!(keymap.resolve(Key::Named(NamedKey::Backspace), Modifiers::NONE, true), None);
    assert_eq!(keymap.resolve(Key::Named(NamedKey::Delete), Modifiers::NONE, true), None);
    assert_eq!(
        keymap.resolve(Key::Named(NamedKey::Backspace), Modifiers::NONE, false),
        Some(VerbId::DeleteSelectedKeys)
    );
    assert_eq!(
        keymap.resolve(Key::Named(NamedKey::Delete), Modifiers::NONE, false),
        Some(VerbId::DeleteSelectedKeys)
    );
}

/// `Verb.shortcut` 生成の実演(発注書 やること4)。表示ラベルが `Cmd+…` 形式で
/// 出ることを見る——`next/shell/motolii-shell/src/menu.rs` の手書き文字列との
/// 一致は shell 側の `keymap_equivalence.rs` が実データで検分する。
#[test]
fn shortcut_label_can_be_derived_from_the_table() {
    let keymap = default_keymap();
    assert_eq!(keymap.shortcut_label(VerbId::Undo).as_deref(), Some("Cmd+Z"));
    assert_eq!(keymap.shortcut_label(VerbId::Redo).as_deref(), Some("Cmd+Shift+Z"));
    assert_eq!(keymap.shortcut_label(VerbId::ResetToRenderCamera).as_deref(), Some("Shift+F"));
}
