//! バインディング表・解決器・衝突検出。**背骨1(書き口1箇所)の keymap 版** —
//! (キー+修飾キー+文脈)→動詞 id の対応を、コードの match ではなく**データ**
//! として持つ(発注書 やること1)。

use crate::key::{Key, ModifierSpec, Modifiers};
use crate::verb::VerbId;

/// 発火する文脈。**現行 shell に実在する区別だけ**を型にする —
/// `inspector_pointer_event` の早期 match 腕のうち Escape/Alt+矢印/Shift+F は
/// **`status`(text_input 消費済みか)を無視して常に**発火させる
/// (`Scope::Global`)。**Backspace/Delete は同じ関数内の腕だが
/// `Scope::NavigationBundle`**(2026-08-23 修正 — 旧実装は `status` を一切
/// 見ずに常時発火しており、Timeline でキーフレームを選択したまま text_input で
/// Backspace を押すと文字削除とキー削除が二重発火した。`defaults.rs::
/// global_bindings` 冒頭 doc 参照)。`resolve_navigation_key` へ委譲する残り
/// 全部も `Scope::NavigationBundle` で、**`status == Captured` の間は発火
/// させない**(`resolve_navigation_key` 冒頭 doc 引用「テキスト入力中は
/// 横取りしない」)。この2値が今の shell の全て — pane 別の文脈はまだ shell
/// 側に実装が無い(`Shell::update` はどの pane がフォーカスされているかで
/// キー解決を分岐させていない)ので、ここでも作らない(発明しない)。
/// **将来 pane 別文脈を足す時は variant を増やすだけ**でよい設計
/// (`Scope` を見る側は `match` ではなく [`Scope::always_fires`] の1関数越しに
/// 見ているため)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// テキスト入力中(他の widget が既にそのキーを消費した状態)でも常に発火する。
    Global,
    /// テキスト入力中は無効(既定の nav/編集ショートカット文脈、
    /// `resolve_navigation_key` 全体がこれ)。
    NavigationBundle,
}

impl Scope {
    /// text_input が既にこのキーを消費した(`captured_by_text_input`)状態でも
    /// なお発火するか。
    pub fn always_fires(self) -> bool {
        matches!(self, Scope::Global)
    }
}

/// 1本のバインディング。「データとして持つ」の最小単位(発注書 やること1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    pub key: Key,
    pub modifiers: ModifierSpec,
    pub scope: Scope,
    pub verb: VerbId,
}

impl Binding {
    pub const fn new(key: Key, modifiers: ModifierSpec, scope: Scope, verb: VerbId) -> Self {
        Binding { key, modifiers, scope, verb }
    }

    /// 表示用ラベル(`Cmd+Shift+Z` 等)。[`ModifierSpec::label_prefix`] +
    /// [`Key::label`]。
    pub fn label(&self) -> String {
        format!("{}{}", self.modifiers.label_prefix(), self.key.label())
    }
}

/// 構築時に検出した衝突1件。**同じ `key` に2つの異なる動詞が、両立しうる
/// 修飾キー要求で結びついている**状態(発注書 やること3「衝突検出は構築時に
/// 検出できる形」)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict {
    pub key: Key,
    pub first: VerbId,
    pub second: VerbId,
}

/// 構築済みのキーマップ。**唯一の構築口は [`Keymap::build`]** — 衝突があれば
/// `Err` になり、衝突を含んだままの `Keymap` は作れない(発注書「構築時に検出」
/// の型的裏付け)。
#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Keymap {
    /// 衝突検出を通してから組む。**`scope` をまたいで**衝突を見る —
    /// `Global` は `captured_by_text_input` の値に関係なく発火するので、
    /// `captured_by_text_input == false` の瞬間には `Global` と
    /// `NavigationBundle` の両方が同時に「生きている」候補になる。よって
    /// scope の異同では衝突判定を割り引かない(全ペアを見る)。
    pub fn build(bindings: Vec<Binding>) -> Result<Self, Vec<Conflict>> {
        let mut conflicts = Vec::new();
        for i in 0..bindings.len() {
            for j in (i + 1)..bindings.len() {
                let a = bindings[i];
                let b = bindings[j];
                if a.key == b.key && a.verb != b.verb && a.modifiers.overlaps(b.modifiers) {
                    conflicts.push(Conflict { key: a.key, first: a.verb, second: b.verb });
                }
            }
        }
        if conflicts.is_empty() {
            Ok(Keymap { bindings })
        } else {
            Err(conflicts)
        }
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// (キー+修飾キー+文脈)→ 動詞 id。**pure** — `resolve_navigation_key` と
    /// 同じ設計意図(試験がイベントを組み立てずに直接叩ける)。表の先頭から
    /// 見て最初に一致した binding を返す(現行 shell の match 文と同じ
    /// 「先勝ち」規則 — `build()` が事前に衝突ゼロを保証しているので、通常は
    /// 「最初に一致した物」と「一致した唯一の物」が一致する)。
    pub fn resolve(&self, key: Key, modifiers: Modifiers, captured_by_text_input: bool) -> Option<VerbId> {
        self.bindings
            .iter()
            .find(|binding| {
                binding.key == key
                    && binding.modifiers.matches(modifiers)
                    && (binding.scope.always_fires() || !captured_by_text_input)
            })
            .map(|binding| binding.verb)
    }

    /// ある動詞に結びついた**最初の** binding の表示ラベル。1動詞が複数キーを
    /// 持つ場合(例: `DeleteSelectedKeys` は Backspace/Delete の2キー)は表の
    /// 先頭を正本とする。発注書 やること4「`Verb.shortcut` をこの表から
    /// 生成できるようにする」の実体 — 呼び手(将来の `motolii-verbs` 側)は
    /// この文字列をそのまま表示に使える。
    pub fn shortcut_label(&self, verb: VerbId) -> Option<String> {
        self.bindings.iter().find(|binding| binding.verb == verb).map(Binding::label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{Key, ModifierSpec};

    #[test]
    fn build_rejects_two_different_verbs_on_an_overlapping_chord() {
        let bindings = vec![
            Binding::new(
                Key::character('z'),
                ModifierSpec::ANY.command_required(true),
                Scope::NavigationBundle,
                VerbId::Undo,
            ),
            // shift の要否を指定していない(ANY) — 素の Cmd+Z と両立しうるので
            // 衝突するはず。
            Binding::new(
                Key::character('z'),
                ModifierSpec::ANY.command_required(true),
                Scope::NavigationBundle,
                VerbId::Redo,
            ),
        ];
        let err = Keymap::build(bindings).expect_err("重なる chord なのに衝突検出しなかった");
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].key, Key::character('z'));
    }

    #[test]
    fn build_accepts_disjoint_shift_requirement_on_the_same_key() {
        let bindings = vec![
            Binding::new(
                Key::character('z'),
                ModifierSpec::ANY.command_required(true).shift_required(false),
                Scope::NavigationBundle,
                VerbId::Undo,
            ),
            Binding::new(
                Key::character('z'),
                ModifierSpec::ANY.command_required(true).shift_required(true),
                Scope::NavigationBundle,
                VerbId::Redo,
            ),
        ];
        assert!(Keymap::build(bindings).is_ok(), "shift で分岐した2本を衝突扱いしている");
    }

    #[test]
    fn same_verb_from_two_different_keys_is_not_a_conflict() {
        use crate::key::NamedKey;
        let bindings = vec![
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
        ];
        assert!(Keymap::build(bindings).is_ok(), "同じ動詞への2つの入口を衝突扱いしている");
    }

    #[test]
    fn resolve_returns_none_for_navigation_bundle_while_captured_but_global_still_fires() {
        use crate::key::NamedKey;
        let bindings = vec![
            Binding::new(Key::Named(NamedKey::Escape), ModifierSpec::ANY, Scope::Global, VerbId::EscapeCancel),
            Binding::new(
                Key::Named(NamedKey::Home),
                ModifierSpec::ANY,
                Scope::NavigationBundle,
                VerbId::JumpPlayheadToStart,
            ),
        ];
        let keymap = Keymap::build(bindings).unwrap();

        assert_eq!(
            keymap.resolve(Key::Named(NamedKey::Escape), Modifiers::NONE, true),
            Some(VerbId::EscapeCancel),
            "Global が captured=true で無効化されている"
        );
        assert_eq!(
            keymap.resolve(Key::Named(NamedKey::Home), Modifiers::NONE, true),
            None,
            "NavigationBundle が captured=true なのに発火した"
        );
        assert_eq!(
            keymap.resolve(Key::Named(NamedKey::Home), Modifiers::NONE, false),
            Some(VerbId::JumpPlayheadToStart),
            "NavigationBundle が captured=false で発火しない"
        );
    }

    #[test]
    fn shortcut_label_matches_expected_format() {
        let binding = Binding::new(
            Key::character('z'),
            ModifierSpec::ANY.command_required(true).shift_required(true),
            Scope::NavigationBundle,
            VerbId::Redo,
        );
        assert_eq!(binding.label(), "Cmd+Shift+Z");
    }
}
