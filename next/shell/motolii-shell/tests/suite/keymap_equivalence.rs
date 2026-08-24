//! keymap 層発注書の「移行の安全証明」: `motolii_shell::resolve_navigation_key`
//! (現行の直書き実装)と `motolii_keymap::nav_bundle_keymap()`(新設 crate が
//! 写し取った表)が、**同じ入力に同じ結論**を返すことを網羅的に検分する。
//!
//! `motolii-keymap` は `motolii-shell` の **dev-dependency**(`Cargo.toml`
//! doc 参照)——production の `resolve_navigation_key`/`inspector_pointer_event`
//! はこの波では一切書き換えていない(発注書 EXACT TARGET「shell への適用は
//! 次切片」)。この試験1本だけが両者を橋渡しする。
//!
//! `global_bindings`(Escape/Backspace・Delete/Alt+矢印/Shift+F)側は
//! **本レーンで `inspector_pointer_event` が `pub` になったので直接突き
//! 合わせる**(`backspace_delete_respect_capture_and_match_global_bindings`)。
//! 旧 doc の「pub でないため対象外」という逸脱は解消済み — 残る自己無矛盾
//! 検分は `motolii-keymap` crate 内 `tests/keymap_oracle.rs` を見よ。

use iced::keyboard::{key::Named, Key as IcedKey, Modifiers as IcedModifiers};
use motolii_keymap::{global_bindings, nav_bundle_keymap, Key as NeutralKey, Keymap, Modifiers as NeutralModifiers, NamedKey, VerbId};
use motolii_shell::timeline_pane::nav::{ClipEdge, JumpDirection};
use motolii_shell::timeline_pane::{Message as TimelinePaneMessage, ShuttleCommand};
use motolii_shell::{inspector_pointer_event, resolve_navigation_key, Message};

/// `resolve_navigation_key` が実際に処理する候補キー全部
/// (`motolii-keymap` の `nav_bundle_bindings()` が写した集合と同じ)。
fn candidate_keys() -> Vec<(IcedKey, NeutralKey)> {
    use motolii_keymap::NamedKey;
    vec![
        (IcedKey::Named(Named::ArrowLeft), NeutralKey::Named(NamedKey::ArrowLeft)),
        (IcedKey::Named(Named::ArrowRight), NeutralKey::Named(NamedKey::ArrowRight)),
        (IcedKey::Named(Named::Home), NeutralKey::Named(NamedKey::Home)),
        (IcedKey::Named(Named::End), NeutralKey::Named(NamedKey::End)),
        (IcedKey::Named(Named::Space), NeutralKey::Named(NamedKey::Space)),
        (IcedKey::Character("j".into()), NeutralKey::character('j')),
        (IcedKey::Character("k".into()), NeutralKey::character('k')),
        (IcedKey::Character("i".into()), NeutralKey::character('i')),
        (IcedKey::Character("o".into()), NeutralKey::character('o')),
        (IcedKey::Character("z".into()), NeutralKey::character('z')),
        (IcedKey::Character("c".into()), NeutralKey::character('c')),
        (IcedKey::Character("v".into()), NeutralKey::character('v')),
        (IcedKey::Character("x".into()), NeutralKey::character('x')),
        (IcedKey::Character("d".into()), NeutralKey::character('d')),
        (IcedKey::Character("a".into()), NeutralKey::character('a')),
        (IcedKey::Character("n".into()), NeutralKey::character('n')),
        (IcedKey::Character("s".into()), NeutralKey::character('s')),
        (IcedKey::Character("q".into()), NeutralKey::character('q')),
        (IcedKey::Character("g".into()), NeutralKey::character('g')),
        // JKL シャトル移設後の新住所(裁定208 の無主レーンで追随、
        // `defaults.rs::nav_bundle_bindings` 参照)。`<`/`>`(shift 付きで
        // 実際に届きうる別文字)は候補に含めない——この試験の候補キー集合は
        // 「1 IcedKey に1 NeutralKey」の1対1写像でしか組めず、shell 側だけが
        // 持つ `,`/`<` の二重受理をここで再現する筋ではないため。
        (IcedKey::Character(",".into()), NeutralKey::character(',')),
        (IcedKey::Character(".".into()), NeutralKey::character('.')),
        // 対照実験: どのバインディングも持たないキー。両者とも常に `None` を
        // 返すはず(取りこぼしが無いことの負例)。
        (IcedKey::Character("p".into()), NeutralKey::character('p')),
    ]
}

fn modifier_pair(command: bool, shift: bool, alt: bool) -> (IcedModifiers, NeutralModifiers) {
    let mut iced_mods = IcedModifiers::default();
    if command {
        iced_mods |= IcedModifiers::COMMAND;
    }
    if shift {
        iced_mods |= IcedModifiers::SHIFT;
    }
    if alt {
        iced_mods |= IcedModifiers::ALT;
    }
    let neutral = NeutralModifiers { command, shift, alt, control: false };
    (iced_mods, neutral)
}

/// `resolve_navigation_key` が返しうる `Message` を [`VerbId`] へ写す
/// (`motolii-keymap` 側の動詞 id と1対1、`defaults.rs` の表と同じ対応)。
fn expected_verb(message: &Message) -> VerbId {
    match message {
        Message::StepPlayhead(1) => VerbId::StepPlayheadForward,
        Message::StepPlayhead(10) => VerbId::StepPlayheadForwardFast,
        Message::StepPlayhead(-1) => VerbId::StepPlayheadBack,
        Message::StepPlayhead(-10) => VerbId::StepPlayheadBackFast,
        Message::JumpPlayheadToStart => VerbId::JumpPlayheadToStart,
        Message::JumpPlayheadToEnd => VerbId::JumpPlayheadToEnd,
        Message::JumpToWorkAreaStart => VerbId::JumpToWorkAreaStart,
        Message::JumpToWorkAreaEnd => VerbId::JumpToWorkAreaEnd,
        Message::JumpMeaningPoint { direction: JumpDirection::Prev, layer_only: false } => {
            VerbId::JumpMeaningPointPrev
        }
        Message::JumpMeaningPoint { direction: JumpDirection::Prev, layer_only: true } => {
            VerbId::JumpMeaningPointPrevLayerOnly
        }
        Message::JumpMeaningPoint { direction: JumpDirection::Next, layer_only: false } => {
            VerbId::JumpMeaningPointNext
        }
        Message::JumpMeaningPoint { direction: JumpDirection::Next, layer_only: true } => {
            VerbId::JumpMeaningPointNextLayerOnly
        }
        Message::JumpClipEdge(ClipEdge::In) => VerbId::JumpClipEdgeIn,
        Message::JumpClipEdge(ClipEdge::Out) => VerbId::JumpClipEdgeOut,
        // JKL シャトル(裁定208 の無主レーンで追随、`VerbId::ShuttleReverse`
        // 冒頭コメント参照)。
        Message::Timeline(TimelinePaneMessage::Shuttle(ShuttleCommand::Reverse)) => VerbId::ShuttleReverse,
        Message::Timeline(TimelinePaneMessage::Shuttle(ShuttleCommand::Stop)) => VerbId::ShuttleStop,
        Message::Timeline(TimelinePaneMessage::SetWorkAreaOut) => VerbId::SetWorkAreaOut,
        // Split(GOALS M6・E-1 結線)。Cmd+K → `SplitAtPlayhead`。
        Message::Timeline(TimelinePaneMessage::SplitAtPlayhead) => VerbId::SplitAtPlayhead,
        Message::Undo => VerbId::Undo,
        Message::Redo => VerbId::Redo,
        Message::CopyLayer => VerbId::CopyLayer,
        Message::PasteLayer => VerbId::PasteLayer,
        Message::CutLayer => VerbId::CutLayer,
        Message::DuplicateLayer => VerbId::DuplicateLayer,
        Message::SelectAllLayers => VerbId::SelectAllLayers,
        Message::DeselectAllLayers => VerbId::DeselectAllLayers,
        Message::NewProjectRequested => VerbId::NewProjectRequested,
        Message::SaveRequested => VerbId::SaveRequested,
        Message::SaveAsRequested => VerbId::SaveAsRequested,
        Message::QuitRequested => VerbId::QuitRequested,
        Message::GroupLayers => VerbId::GroupLayers,
        Message::UngroupLayers => VerbId::UngroupLayers,
        Message::TogglePlayback => VerbId::TogglePlayback,
        other => panic!("resolve_navigation_key が想定外の Message を出した: {other:?}"),
    }
}

/// 本体: 全候補キー × cmd/shift/alt の全8通り × captured 2通りを総当たりし、
/// `resolve_navigation_key` と `nav_bundle_keymap().resolve(...)` が同じ結論を
/// 返すことを見る(発注書「現在の shell の割当と解決器の出力が一致すること」)。
#[test]
fn resolver_output_matches_resolve_navigation_key_across_the_full_modifier_grid() {
    let keymap = nav_bundle_keymap();
    let mut checked = 0usize;

    for (iced_key, neutral_key) in candidate_keys() {
        for command in [false, true] {
            for shift in [false, true] {
                for alt in [false, true] {
                    let (iced_mods, neutral_mods) = modifier_pair(command, shift, alt);
                    for captured in [false, true] {
                        checked += 1;
                        let shell_result = resolve_navigation_key(&iced_key, iced_mods, captured);
                        let keymap_result = keymap.resolve(neutral_key, neutral_mods, captured);

                        match (shell_result, keymap_result) {
                            (None, None) => {}
                            (Some(message), Some(verb)) => {
                                assert_eq!(
                                    expected_verb(&message),
                                    verb,
                                    "{iced_key:?}+cmd={command}/shift={shift}/alt={alt}/captured={captured}: \
                                     shell={message:?} keymap={verb:?}"
                                );
                            }
                            (shell_result, keymap_result) => panic!(
                                "{iced_key:?}+cmd={command}/shift={shift}/alt={alt}/captured={captured} で\
                                 一致しない: shell={shell_result:?} keymap={keymap_result:?}"
                            ),
                        }
                    }
                }
            }
        }
    }

    // 総当たりが本当に回っていることの保険(検分自体が骨抜きになっていないか)。
    assert_eq!(checked, candidate_keys().len() * 2 * 2 * 2 * 2);
}

/// `iced::keyboard::Key::Named(Backspace/Delete)` の `KeyPressed` を組み立てる。
/// `physical_key`/`location`/`text`/`modified_key` は `inspector_pointer_event`
/// の Backspace/Delete 腕(および `resolve_navigation_key` 全体)が一切読まない
/// フィールドなので、実イベントを模す必要のない既定値で埋める。
fn key_pressed_event(key: IcedKey, modifiers: IcedModifiers) -> iced::Event {
    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced::keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

/// **本レーンの回帰柵**(AX-5 実測・M20「TextInput 中はテキスト優先」):
/// `inspector_pointer_event` は本日 `pub` になった — `global_bindings` を
/// 自己申告の自己無矛盾(`keymap_oracle.rs`)止まりにせず、実際の shell 関数と
/// 直接突き合わせる。**captured=true(text_input にフォーカスがある)間は
/// Backspace/Delete が `Message::Timeline(DeleteSelectedKeys)` を発火しては
/// いけない**(でなければ rename 中の文字削除とキーフレーム削除が二重発火する
/// — 旧実装の実害)。Escape は対照実験として引き続き captured を無視する
/// (`Scope::Global` のまま、Esc-cancel は typing 中でも意味を持つ)。
#[test]
fn backspace_delete_respect_capture_and_match_global_bindings() {
    let keymap = Keymap::build(global_bindings()).expect("global_bindings に衝突がある");
    let window = iced::window::Id::unique();

    let cases = [
        (IcedKey::Named(Named::Backspace), NeutralKey::Named(NamedKey::Backspace)),
        (IcedKey::Named(Named::Delete), NeutralKey::Named(NamedKey::Delete)),
    ];

    for (iced_key, neutral_key) in cases {
        for captured in [false, true] {
            let status = if captured {
                iced::event::Status::Captured
            } else {
                iced::event::Status::Ignored
            };
            let shell_result = inspector_pointer_event(
                key_pressed_event(iced_key.clone(), IcedModifiers::default()),
                status,
                window,
            );
            let keymap_result = keymap.resolve(neutral_key, NeutralModifiers::default(), captured);

            match (shell_result, keymap_result) {
                (None, None) => {}
                (Some(Message::DeleteSelectionRequested), Some(VerbId::DeleteSelectedKeys)) => {}
                (shell_result, keymap_result) => panic!(
                    "{iced_key:?}/captured={captured}: shell={shell_result:?} \
                     keymap={keymap_result:?}(一致しない、または captured=true なのに発火した)"
                ),
            }
        }
    }

    // 核心の1点(RETURN 参照): captured=true では Document 側が一切発火しない。
    for iced_key in [IcedKey::Named(Named::Backspace), IcedKey::Named(Named::Delete)] {
        let result = inspector_pointer_event(
            key_pressed_event(iced_key.clone(), IcedModifiers::default()),
            iced::event::Status::Captured,
            window,
        );
        assert!(
            result.is_none(),
            "{iced_key:?}: captured=true(text_input 編集中)なのに Timeline 削除が発火している: {result:?}"
        );
    }

    // 対照実験(境界の明示、追加テストではなく同じ試験内の1アサート):
    // Escape は `Scope::Global` のまま、captured=true でも発火する — 今回
    // 修正したのは Backspace/Delete だけであることを確認する。
    let escape_result = inspector_pointer_event(
        key_pressed_event(IcedKey::Named(Named::Escape), IcedModifiers::default()),
        iced::event::Status::Captured,
        window,
    );
    assert!(
        matches!(escape_result, Some(Message::EscapePressed)),
        "Escape は captured=true でも発火するはず(Esc-cancel は typing 中も意味を持つ): {escape_result:?}"
    );
}
