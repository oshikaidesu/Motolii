//! 既定バインディングの表(発注書 やること2)。**現在
//! `next/shell/motolii-shell/src/lib.rs` に直書きされている割当を全部ここへ
//! 写しただけ** — 新しい割当は一切発明していない。各行のコメントは移設元の
//! match 腕を指す(移行の追跡用)。
//!
//! 2つの表に分ける理由は [`crate::binding::Scope`] の doc と同じ:
//! - [`global_bindings`]: `inspector_pointer_event` が `status` を無視して
//!   常に発火させる腕(Escape/Backspace・Delete/Alt+矢印/Shift+F)
//! - [`nav_bundle_bindings`]: `resolve_navigation_key` が受け持つ残り全部
//!   (テキスト入力中は無効)
//!
//! [`default_bindings`] はこの2つを**実際のイベント到達順**(`global` が先に
//! match され、その後に `nav_bundle` 相当へフォールスルーする)で連結する。
//! `next/shell/motolii-shell/tests/suite/keymap_equivalence.rs` は
//! [`nav_bundle_keymap`] と `motolii_shell::resolve_navigation_key` を突き合わせ、
//! 両者が同じ入力に同じ結論を返すことを検分する(移行の安全証明)。
//! `global_bindings` 側は `inspector_pointer_event` が `pub` でないため
//! shell と直接突き合わせられない——crate 内試験(`tests/keymap_oracle.rs`)
//! だけで自己無矛盾を確認している(RETURN の逸脱台帳に明記)。

use crate::binding::{Binding, Keymap, Scope};
use crate::key::{Key, ModifierSpec, NamedKey};
use crate::verb::VerbId;

/// `inspector_pointer_event`(`lib.rs` 4002行目〜)の早期 match 腕。**常に発火**
/// (`status` 無視)。
pub fn global_bindings() -> Vec<Binding> {
    vec![
        // 4017-4020: Escape → EscapePressed
        Binding::new(Key::Named(NamedKey::Escape), ModifierSpec::ANY, Scope::Global, VerbId::EscapeCancel),
        // 4028-4035: Backspace/Delete(Mac 主部キーは Backspace として届く) →
        // Timeline::DeleteSelectedKeys。2キーとも同じ動詞。
        Binding::new(
            Key::Named(NamedKey::Backspace),
            ModifierSpec::ANY,
            Scope::Global,
            VerbId::DeleteSelectedKeys,
        ),
        Binding::new(
            Key::Named(NamedKey::Delete),
            ModifierSpec::ANY,
            Scope::Global,
            VerbId::DeleteSelectedKeys,
        ),
        // 4040-4047: Alt+←(+Shift で10フレーム) → NudgeKeyframe(-1/-10)
        Binding::new(
            Key::Named(NamedKey::ArrowLeft),
            ModifierSpec::ANY.alt_required(true).shift_required(false),
            Scope::Global,
            VerbId::NudgeKeyframeBack,
        ),
        Binding::new(
            Key::Named(NamedKey::ArrowLeft),
            ModifierSpec::ANY.alt_required(true).shift_required(true),
            Scope::Global,
            VerbId::NudgeKeyframeBackFast,
        ),
        // 4048-4055: Alt+→(+Shift で10フレーム) → NudgeKeyframe(1/10)
        Binding::new(
            Key::Named(NamedKey::ArrowRight),
            ModifierSpec::ANY.alt_required(true).shift_required(false),
            Scope::Global,
            VerbId::NudgeKeyframeForward,
        ),
        Binding::new(
            Key::Named(NamedKey::ArrowRight),
            ModifierSpec::ANY.alt_required(true).shift_required(true),
            Scope::Global,
            VerbId::NudgeKeyframeForwardFast,
        ),
        // 4060-4065: Shift+F(cmd/alt は問わない — 元コードは shift しか
        // 見ていない) → Stage::ResetToRenderCamera
        Binding::new(
            Key::character('f'),
            ModifierSpec::ANY.shift_required(true),
            Scope::Global,
            VerbId::ResetToRenderCamera,
        ),
    ]
}

/// `resolve_navigation_key`(`lib.rs` 4109行目〜)の全 match 腕。**テキスト
/// 入力中は無効**(`captured` ガード)。
///
/// **shift を見ない腕がある点に注意**(元コードの実際の仕様であって、この
/// crate が作った不整合ではない): `i`/`o`(JumpClipEdge)・`c`/`v`/`x`/`d`
/// (Copy/Paste/Cut/Duplicate)・`q`(Quit)・`Space`(TogglePlayback)は shift の
/// 有無を一切見ていない — 例えば元コードは `Cmd+Shift+C` も
/// `Message::CopyLayer` を返す(`c.eq_ignore_ascii_case` の guard に shift
/// 条件が無い)。ここでは「今の割当を写す」ため `shift: ANY`(don't-care)で
/// 忠実に再現する。直す(shift を弾く)なら別途の意味論裁定が要る。
pub fn nav_bundle_bindings() -> Vec<Binding> {
    vec![
        // 4122-4125: ←(Alt無し、素=1フレーム)
        Binding::new(
            Key::Named(NamedKey::ArrowLeft),
            ModifierSpec::ANY.alt_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::StepPlayheadBack,
        ),
        // 4122-4125: Shift+←(Alt無し、10フレーム)
        Binding::new(
            Key::Named(NamedKey::ArrowLeft),
            ModifierSpec::ANY.alt_required(false).shift_required(true),
            Scope::NavigationBundle,
            VerbId::StepPlayheadBackFast,
        ),
        // 4126-4129: →(Alt無し、素=1フレーム)
        Binding::new(
            Key::Named(NamedKey::ArrowRight),
            ModifierSpec::ANY.alt_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::StepPlayheadForward,
        ),
        // 4126-4129: Shift+→(Alt無し、10フレーム)
        Binding::new(
            Key::Named(NamedKey::ArrowRight),
            ModifierSpec::ANY.alt_required(false).shift_required(true),
            Scope::NavigationBundle,
            VerbId::StepPlayheadForwardFast,
        ),
        // 4130: Home → JumpPlayheadToStart
        Binding::new(Key::Named(NamedKey::Home), ModifierSpec::ANY, Scope::NavigationBundle, VerbId::JumpPlayheadToStart),
        // 4131: End → JumpPlayheadToEnd
        Binding::new(Key::Named(NamedKey::End), ModifierSpec::ANY, Scope::NavigationBundle, VerbId::JumpPlayheadToEnd),
        // 4133-4138: j(Cmd無し、素) → JumpMeaningPoint{Prev, layer_only:false}
        Binding::new(
            Key::character('j'),
            ModifierSpec::ANY.command_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointPrev,
        ),
        // 4133-4138: Shift+j(Cmd無し) → layer_only:true
        Binding::new(
            Key::character('j'),
            ModifierSpec::ANY.command_required(false).shift_required(true),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointPrevLayerOnly,
        ),
        // 4139-4144: k(Cmd無し、素) → JumpMeaningPoint{Next, layer_only:false}
        Binding::new(
            Key::character('k'),
            ModifierSpec::ANY.command_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointNext,
        ),
        // 4139-4144: Shift+k(Cmd無し) → layer_only:true
        Binding::new(
            Key::character('k'),
            ModifierSpec::ANY.command_required(false).shift_required(true),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointNextLayerOnly,
        ),
        // 4146-4148: i(Cmd無し、shift 不問) → JumpClipEdge(In)
        Binding::new(
            Key::character('i'),
            ModifierSpec::ANY.command_required(false),
            Scope::NavigationBundle,
            VerbId::JumpClipEdgeIn,
        ),
        // 4149-4151: o(Cmd無し、shift 不問) → JumpClipEdge(Out)
        Binding::new(
            Key::character('o'),
            ModifierSpec::ANY.command_required(false),
            Scope::NavigationBundle,
            VerbId::JumpClipEdgeOut,
        ),
        // 4159-4161: Cmd+Z(Shift無し) → Undo
        Binding::new(
            Key::character('z'),
            ModifierSpec::ANY.command_required(true).shift_required(false),
            Scope::NavigationBundle,
            VerbId::Undo,
        ),
        // 4162-4164: Cmd+Shift+Z → Redo
        Binding::new(
            Key::character('z'),
            ModifierSpec::ANY.command_required(true).shift_required(true),
            Scope::NavigationBundle,
            VerbId::Redo,
        ),
        // 4165-4167: Cmd+C(shift 不問) → CopyLayer
        Binding::new(
            Key::character('c'),
            ModifierSpec::ANY.command_required(true),
            Scope::NavigationBundle,
            VerbId::CopyLayer,
        ),
        // 4168-4170: Cmd+V(shift 不問) → PasteLayer
        Binding::new(
            Key::character('v'),
            ModifierSpec::ANY.command_required(true),
            Scope::NavigationBundle,
            VerbId::PasteLayer,
        ),
        // 4171-4173: Cmd+X(shift 不問) → CutLayer
        Binding::new(
            Key::character('x'),
            ModifierSpec::ANY.command_required(true),
            Scope::NavigationBundle,
            VerbId::CutLayer,
        ),
        // 4174-4176: Cmd+D(shift 不問) → DuplicateLayer
        Binding::new(
            Key::character('d'),
            ModifierSpec::ANY.command_required(true),
            Scope::NavigationBundle,
            VerbId::DuplicateLayer,
        ),
        // 4177-4179: Cmd+A(Shift無し) → SelectAllLayers
        Binding::new(
            Key::character('a'),
            ModifierSpec::ANY.command_required(true).shift_required(false),
            Scope::NavigationBundle,
            VerbId::SelectAllLayers,
        ),
        // 4180-4182: Cmd+Shift+A → DeselectAllLayers
        Binding::new(
            Key::character('a'),
            ModifierSpec::ANY.command_required(true).shift_required(true),
            Scope::NavigationBundle,
            VerbId::DeselectAllLayers,
        ),
        // 4191-4193: Cmd+N(Shift無し) → NewProjectRequested
        Binding::new(
            Key::character('n'),
            ModifierSpec::ANY.command_required(true).shift_required(false),
            Scope::NavigationBundle,
            VerbId::NewProjectRequested,
        ),
        // 4194-4196: Cmd+Shift+S → SaveAsRequested
        Binding::new(
            Key::character('s'),
            ModifierSpec::ANY.command_required(true).shift_required(true),
            Scope::NavigationBundle,
            VerbId::SaveAsRequested,
        ),
        // 4197-4199: Cmd+Q(shift 不問) → QuitRequested
        Binding::new(
            Key::character('q'),
            ModifierSpec::ANY.command_required(true),
            Scope::NavigationBundle,
            VerbId::QuitRequested,
        ),
        // 4202-4204: Cmd+G(Shift無し) → GroupLayers
        Binding::new(
            Key::character('g'),
            ModifierSpec::ANY.command_required(true).shift_required(false),
            Scope::NavigationBundle,
            VerbId::GroupLayers,
        ),
        // 4205-4207: Cmd+Shift+G → UngroupLayers
        Binding::new(
            Key::character('g'),
            ModifierSpec::ANY.command_required(true).shift_required(true),
            Scope::NavigationBundle,
            VerbId::UngroupLayers,
        ),
        // 4213: Space(修飾キー不問 — 元コードは一切見ていない) → TogglePlayback
        Binding::new(Key::Named(NamedKey::Space), ModifierSpec::ANY, Scope::NavigationBundle, VerbId::TogglePlayback),
    ]
}

/// 両方の表を実際のイベント到達順(`global` が先)で連結した、shell 全体の
/// 既定割当。
pub fn default_bindings() -> Vec<Binding> {
    let mut bindings = global_bindings();
    bindings.extend(nav_bundle_bindings());
    bindings
}

/// [`default_bindings`] から組んだ [`Keymap`]。**衝突があれば `panic`** —
/// 既定表は「今の shell の実装」を写しただけなので衝突が無いことは実測済み
/// (`tests/keymap_oracle.rs::default_keymap_has_no_conflicts` が同じ主張を
/// panic なしで確認する)。
pub fn default_keymap() -> Keymap {
    Keymap::build(default_bindings()).expect("既定バインディング表に衝突がある")
}

/// [`nav_bundle_bindings`] だけを組んだ [`Keymap`]。`resolve_navigation_key`
/// と1対1で突き合わせる用(`tests/suite/keymap_equivalence.rs` 参照) —
/// `global_bindings` を混ぜると Alt+矢印等が `resolve_navigation_key` には
/// 存在しない扱いになり比較が成立しない(`nav_bundle_bindings` 冒頭 doc 参照)。
pub fn nav_bundle_keymap() -> Keymap {
    Keymap::build(nav_bundle_bindings()).expect("nav bundle バインディング表に衝突がある")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_build_without_conflicts() {
        assert!(Keymap::build(default_bindings()).is_ok());
    }

    #[test]
    fn every_verb_id_has_at_least_one_binding() {
        let bindings = default_bindings();
        for verb in VerbId::ALL {
            assert!(
                bindings.iter().any(|binding| binding.verb == *verb),
                "{verb:?} に対応する binding が defaults に無い"
            );
        }
    }

    #[test]
    fn every_binding_verb_is_a_known_verb_id() {
        // ALL に無い動詞を binding だけに書いてしまう typo を防ぐ。
        let bindings = default_bindings();
        for binding in &bindings {
            assert!(VerbId::ALL.contains(&binding.verb), "{:?} が VerbId::ALL に無い", binding.verb);
        }
    }
}
