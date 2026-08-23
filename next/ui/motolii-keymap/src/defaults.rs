//! 既定バインディングの表(発注書 やること2)。**現在
//! `next/shell/motolii-shell/src/lib.rs` に直書きされている割当を全部ここへ
//! 写しただけ** — 新しい割当は一切発明していない。各行のコメントは移設元の
//! match 腕を指す(移行の追跡用)。
//!
//! 2つの表に分ける理由は [`crate::binding::Scope`] の doc と同じ —
//! **「どの関数が捌くか」ではなく「`captured`(text_input が既にそのキーを
//! 消費したか)を見るかどうか」で分ける**:
//! - [`global_bindings`]: `inspector_pointer_event` の早期 match 腕。
//!   Escape/Alt+矢印/Shift+F は `status` を無視して常に発火する
//!   (`Scope::Global`)。**Backspace/Delete だけは同じ関数内の腕でありながら
//!   `Scope::NavigationBundle`**(2026-08-23 修正 — 旧実装は
//!   `inspector_pointer_event` の Backspace/Delete 腕が `status` を一切見ずに
//!   常時発火しており、Timeline でキーフレームを選択したまま text_input で
//!   Backspace を押すと文字削除とキー削除が二重発火する実害があった。fork
//!   `core/src/text/input.rs:181` が Backspace を含む全 `Action::Edit` で
//!   無条件に `shell.capture_event()` を呼ぶため、text_input にフォーカスが
//!   ある間は既に `Status::Captured` になっている — Cmd+Z 等と同じ経路。
//!   `input.rs::inspector_pointer_event` 側のコメント参照)
//! - [`nav_bundle_bindings`]: `resolve_navigation_key` が受け持つ残り全部
//!   (テキスト入力中は無効)
//!
//! [`default_bindings`] はこの2つを**実際のイベント到達順**(`global` が先に
//! match され、その後に `nav_bundle` 相当へフォールスルーする)で連結する。
//! `next/shell/motolii-shell/tests/suite/keymap_equivalence.rs` は
//! [`nav_bundle_keymap`] と `motolii_shell::resolve_navigation_key` を突き合わせ、
//! 両者が同じ入力に同じ結論を返すことを検分する(移行の安全証明)。
//! **`inspector_pointer_event` は本レーンで `pub` になった**(shell 分割)ので、
//! 同ファイルは [`global_bindings`] も `inspector_pointer_event` へ直接突き
//! 合わせる(`backspace_delete_respect_capture_and_match_global_bindings`)——
//! 旧 doc が書いていた「pub でないため突き合わせられない」逸脱は解消済み。

use crate::binding::{Binding, Keymap, Scope};
use crate::key::{Key, ModifierSpec, NamedKey};
use crate::verb::VerbId;

/// `inspector_pointer_event`(`next/shell/motolii-shell/src/input.rs`)の
/// 早期 match 腕。**Backspace/Delete を除き常に発火**(`status` 無視)。
pub fn global_bindings() -> Vec<Binding> {
    vec![
        // Escape → EscapePressed
        Binding::new(Key::Named(NamedKey::Escape), ModifierSpec::ANY, Scope::Global, VerbId::EscapeCancel),
        // Backspace/Delete(Mac 主部キーは Backspace として届く) →
        // Timeline::DeleteSelectedKeys。2キーとも同じ動詞。**`Scope::
        // NavigationBundle`**(2026-08-23 修正、上のファイル冒頭 doc 参照) —
        // text_input にフォーカスがある間は既に `shell.capture_event()` 済み
        // なので、captured=true では発火してはいけない。
        Binding::new(
            Key::Named(NamedKey::Backspace),
            ModifierSpec::ANY,
            Scope::NavigationBundle,
            VerbId::DeleteSelectedKeys,
        ),
        Binding::new(
            Key::Named(NamedKey::Delete),
            ModifierSpec::ANY,
            Scope::NavigationBundle,
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
        // Shift+Home(作業範囲の先頭、Shift+Home/End が素の Home/End より先に
        // match される — 裁定208 の無主レーンで追随。shell 側の
        // `resolve_navigation_key` 実測に合わせて表を割った) → JumpToWorkAreaStart
        Binding::new(
            Key::Named(NamedKey::Home),
            ModifierSpec::ANY.shift_required(true),
            Scope::NavigationBundle,
            VerbId::JumpToWorkAreaStart,
        ),
        // 4130: Home(Shift無し) → JumpPlayheadToStart
        Binding::new(
            Key::Named(NamedKey::Home),
            ModifierSpec::ANY.shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpPlayheadToStart,
        ),
        // Shift+End(作業範囲の末尾) → JumpToWorkAreaEnd
        Binding::new(
            Key::Named(NamedKey::End),
            ModifierSpec::ANY.shift_required(true),
            Scope::NavigationBundle,
            VerbId::JumpToWorkAreaEnd,
        ),
        // 4131: End(Shift無し) → JumpPlayheadToEnd
        Binding::new(
            Key::Named(NamedKey::End),
            ModifierSpec::ANY.shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpPlayheadToEnd,
        ),
        // JKL シャトル(B21 第5波結線、`resolve_navigation_key` 5631-5644行目
        // 実測 2026-08-22・裁定208 の無主レーンで追随)。**旧割当だった
        // bare j/k(意味点ジャンプ)は `,`/`.` へ移設済み**(shift はどちらも
        // 見ない — 「連打相当」で同じ Message、下記 doc 参照)。
        Binding::new(
            Key::character('j'),
            ModifierSpec::ANY.command_required(false),
            Scope::NavigationBundle,
            VerbId::ShuttleReverse,
        ),
        Binding::new(
            Key::character('k'),
            ModifierSpec::ANY.command_required(false),
            Scope::NavigationBundle,
            VerbId::ShuttleStop,
        ),
        // JumpPrev/NextMeaningPoint の新住所(5650-5661行目実測)。Shift 付きで
        // 選択レイヤー限定(layer_only)。**shell は同じ物理キーの shift 別
        // 文字 `<`/`>` も受ける(5650/5656行目 `c == "," || c == "<"`)が、
        // この crate の `Key` は1 binding=1文字までしか持てないため
        // `,`/`.` のみ転写する**(`<`/`>` 側は未転写 — RETURN の逸脱台帳)。
        Binding::new(
            Key::character(','),
            ModifierSpec::ANY.command_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointPrev,
        ),
        Binding::new(
            Key::character(','),
            ModifierSpec::ANY.command_required(false).shift_required(true),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointPrevLayerOnly,
        ),
        Binding::new(
            Key::character('.'),
            ModifierSpec::ANY.command_required(false).shift_required(false),
            Scope::NavigationBundle,
            VerbId::JumpMeaningPointNext,
        ),
        Binding::new(
            Key::character('.'),
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
        // Mark Out(作業範囲の Out を playhead へ、5672-5676行目実測・裁定208
        // の無主レーンで追随)。**bare `b`(Mark In/SetWorkAreaIn)は未転写**
        // (候補キーに含まれず未検出だった、`VerbId::ShuttleReverse` 冒頭
        // コメント・RETURN 参照)。
        Binding::new(
            Key::character('n'),
            ModifierSpec::ANY.command_required(false),
            Scope::NavigationBundle,
            VerbId::SetWorkAreaOut,
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
        // Cmd+S(Shift無し) → SaveRequested(裁定150: 4製品とも一度保存したらパスを聞かない)
        Binding::new(
            Key::character('s'),
            ModifierSpec::ANY.command_required(true).shift_required(false),
            Scope::NavigationBundle,
            VerbId::SaveRequested,
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
