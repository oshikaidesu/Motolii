//! キー・修飾キーの中立表現。**`iced::keyboard::Key`/`Modifiers` に依存しない** —
//! この crate は iced を引かない(`Cargo.toml` doc 参照)。shell 側で
//! `iced::keyboard::Key`/`Modifiers` からここへ写す変換は次切片の仕事。
//!
//! 値そのものは `next/shell/motolii-shell/src/lib.rs` の
//! `resolve_navigation_key`/`inspector_pointer_event` が現に処理しているキーの
//! 集合と1対1(新しい named key を発明しない — 発注書「実装済みの物だけ」)。

/// 名前つきキー。**現行 shell が実際に処理している物だけ**を列挙する
/// (`ArrowLeft`/`ArrowRight`/`Home`/`End`/`Space`/`Escape`/`Backspace`/`Delete`)。
/// 増やす時は「shell 側に対応する既存挙動がある」ことが前提(発明しない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    Space,
    Escape,
    Backspace,
    Delete,
}

impl NamedKey {
    /// 表示用ラベル(`Keymap::shortcut_label` が使う)。
    pub fn label(self) -> &'static str {
        match self {
            NamedKey::ArrowLeft => "Left",
            NamedKey::ArrowRight => "Right",
            NamedKey::Home => "Home",
            NamedKey::End => "End",
            NamedKey::Space => "Space",
            NamedKey::Escape => "Esc",
            NamedKey::Backspace => "Backspace",
            NamedKey::Delete => "Delete",
        }
    }
}

/// キー。`Character` は常に小文字 ASCII で保持する — 現行 shell の
/// `c.eq_ignore_ascii_case(...)` と同じ「大小無視」を、正規化1箇所(この型の
/// 構築点)へ寄せるため(`resolve()` 側は単純な `==` だけで済む)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Named(NamedKey),
    /// 常に小文字化して保持する。[`Key::character`] を経由すれば呼び手は
    /// 大小を気にしなくてよい。
    Character(char),
}

impl Key {
    /// 大小どちらで渡しても小文字へ正規化する構築子。
    pub fn character(c: char) -> Self {
        Key::Character(c.to_ascii_lowercase())
    }

    /// 表示用ラベル。
    pub fn label(self) -> String {
        match self {
            Key::Named(named) => named.label().to_owned(),
            Key::Character(c) => c.to_ascii_uppercase().to_string(),
        }
    }
}

/// 修飾キーの実際の押下状態。フィールド名は `iced::keyboard::Modifiers` の
/// アクセサ(`command()`/`shift()`/`alt()`/`control()`)に揃える —
/// shell 側の変換コード(次切片)が1対1で書けるように。`command` は
/// `iced::keyboard::Modifiers::command()` と同じ「macOS では Cmd、それ以外では
/// Ctrl」の論理キーであって物理 Ctrl とは別(iced のドキュメント参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub command: bool,
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers { command: false, shift: false, alt: false, control: false };

    pub fn command() -> Self {
        Modifiers { command: true, ..Modifiers::NONE }
    }

    pub fn shift() -> Self {
        Modifiers { shift: true, ..Modifiers::NONE }
    }

    pub fn alt() -> Self {
        Modifiers { alt: true, ..Modifiers::NONE }
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }
}

/// 1つの修飾キーに対する要求。`None` = どちらでもよい(現行 shell の
/// 「そのフラグを一切見ない」match guard に対応 — 例: `Cmd+C` は shift の
/// 有無を見ないので `Cmd+Shift+C` でも発火する。これは shell の現状の**仕様**
/// であって、この crate の役目は「今の割当を写す」ことなので、ここでも
/// `None`(どちらでもよい)として忠実に再現する)。
pub type Requirement = Option<bool>;

/// 4つの修飾キーそれぞれに [`Requirement`] を持つ、1 binding の判定式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierSpec {
    pub command: Requirement,
    pub shift: Requirement,
    pub alt: Requirement,
    pub control: Requirement,
}

impl ModifierSpec {
    /// 全フラグ「どちらでもよい」(`Space` 等 — 現行 shell がどの修飾も
    /// 見ない match guard)。
    pub const ANY: ModifierSpec =
        ModifierSpec { command: None, shift: None, alt: None, control: None };

    pub fn command_required(mut self, required: bool) -> Self {
        self.command = Some(required);
        self
    }

    pub fn shift_required(mut self, required: bool) -> Self {
        self.shift = Some(required);
        self
    }

    pub fn alt_required(mut self, required: bool) -> Self {
        self.alt = Some(required);
        self
    }

    /// 実際の押下状態がこの判定式を満たすか。
    pub fn matches(self, actual: Modifiers) -> bool {
        fn check(requirement: Requirement, actual: bool) -> bool {
            match requirement {
                None => true,
                Some(required) => required == actual,
            }
        }
        check(self.command, actual.command)
            && check(self.shift, actual.shift)
            && check(self.alt, actual.alt)
            && check(self.control, actual.control)
    }

    /// 2つの判定式を**同時に満たす** `Modifiers` が存在しうるか(衝突検出の核)。
    /// 片方が `None`(どちらでもよい)なら常に両立する。両方が `Some` なら
    /// 値が一致していないと両立しない。
    pub fn overlaps(self, other: ModifierSpec) -> bool {
        fn compatible(a: Requirement, b: Requirement) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => a == b,
                _ => true,
            }
        }
        compatible(self.command, other.command)
            && compatible(self.shift, other.shift)
            && compatible(self.alt, other.alt)
            && compatible(self.control, other.control)
    }

    /// 表示ラベルの修飾キー部分(`Cmd+Shift+` のような prefix)。**要求されて
    /// いる(`Some(true)`)フラグだけ**を出す — `None`/`Some(false)` は表記に
    /// 出さない(現行 `menu.rs` の手書き文字列と同じ省略規則、
    /// `default_bindings` 冒頭 doc 参照)。
    pub fn label_prefix(self) -> String {
        let mut parts = Vec::new();
        if self.command == Some(true) {
            parts.push("Cmd");
        }
        if self.shift == Some(true) {
            parts.push("Shift");
        }
        if self.alt == Some(true) {
            parts.push("Alt");
        }
        if self.control == Some(true) {
            parts.push("Ctrl");
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}+", parts.join("+"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_key_normalizes_case() {
        assert_eq!(Key::character('C'), Key::character('c'));
        assert_eq!(Key::character('C'), Key::Character('c'));
    }

    #[test]
    fn requirement_none_matches_either_state() {
        let spec = ModifierSpec::ANY.command_required(true);
        assert!(spec.matches(Modifiers { shift: true, ..Modifiers::command() }));
        assert!(spec.matches(Modifiers::command()));
    }

    #[test]
    fn overlap_is_false_when_a_required_flag_disagrees() {
        let cmd_no_shift = ModifierSpec::ANY.command_required(true).shift_required(false);
        let cmd_shift = ModifierSpec::ANY.command_required(true).shift_required(true);
        assert!(!cmd_no_shift.overlaps(cmd_shift), "shift 要求が逆なのに overlap している");
    }

    #[test]
    fn overlap_is_true_when_one_side_does_not_care() {
        let cmd_any_shift = ModifierSpec::ANY.command_required(true);
        let cmd_shift = ModifierSpec::ANY.command_required(true).shift_required(true);
        assert!(cmd_any_shift.overlaps(cmd_shift), "shift 側が don't-care なのに overlap しない");
    }

    #[test]
    fn label_prefix_only_shows_required_true_flags() {
        let spec = ModifierSpec::ANY.command_required(true).shift_required(false);
        assert_eq!(spec.label_prefix(), "Cmd+", "shift:Forbidden が表記に漏れている");

        let any = ModifierSpec::ANY.command_required(true);
        assert_eq!(any.label_prefix(), "Cmd+", "shift: DontCare が表記に混ざっている");
    }
}
