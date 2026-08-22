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
//! `inspector_pointer_event` が `pub` でないため直接は突き合わせられない
//! (`motolii-keymap` crate 冒頭 doc の逸脱台帳・`tests/keymap_oracle.rs` 参照)。

use iced::keyboard::{key::Named, Key as IcedKey, Modifiers as IcedModifiers};
use motolii_keymap::{nav_bundle_keymap, Key as NeutralKey, Modifiers as NeutralModifiers, VerbId};
use motolii_shell::timeline_pane::nav::{ClipEdge, JumpDirection};
use motolii_shell::{resolve_navigation_key, Message};

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
        Message::Undo => VerbId::Undo,
        Message::Redo => VerbId::Redo,
        Message::CopyLayer => VerbId::CopyLayer,
        Message::PasteLayer => VerbId::PasteLayer,
        Message::CutLayer => VerbId::CutLayer,
        Message::DuplicateLayer => VerbId::DuplicateLayer,
        Message::SelectAllLayers => VerbId::SelectAllLayers,
        Message::DeselectAllLayers => VerbId::DeselectAllLayers,
        Message::NewProjectRequested => VerbId::NewProjectRequested,
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
