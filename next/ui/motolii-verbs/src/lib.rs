//! 動詞レジストリ(発注 2026-08-22)。
//!
//! ## 問題
//!
//! 1つの動詞(例: `Copy`)がいま最大4箇所に手書きされている —
//! (a) メニュー項目(`motolii_menubar::menus::edit_menu`)
//! (b) 右クリック項目(`motolii_menubar::context::clip_context_items`)
//! (c) keymap(`motolii_shell::resolve_navigation_key`)
//! (d) `next/reference/normal-map.tsv` の行。
//!
//! **この crate が今回消化するのは (a)/(b) の重複だけ**(ラベル文字列・
//! shortcut 表記・並びを [`registry`] へ正本化し、[`generate`] が両方の
//! 出力形を生成する)。(c) keymap の統合と (d) map 行id からの自動生成は
//! 次切片 — この crate の [`Verb::map_ids`] は「対応関係の記録」に留まり、
//! (d) から (a)/(b) を自動生成する機構はまだ無い。
//!
//! ## S6(Ableton 可視性原理)を construction time で保証
//!
//! [`裁定195`](../../../DECISIONS.md) が実測した通り、[`Entry::Context`]/
//! [`Entry::PanelControl`](右クリック・rail glyph・Inspector swatch のような
//! 「隠れた」入口)が**その動詞の唯一の入口**になっていると Ableton 可視性
//! 原理(S6)に違反する。この crate は `Verb` の `entries` フィールドを
//! [`s6_checked`] でラップして定義することを強制し、違反があれば
//! **`static` 初期化子の const 評価がコンパイルエラーになる**(実行時
//! テストを待たずにビルドが止まる — 「型で保証」の実装としてはこれが
//! stable Rust で到達できる最も強い形。[`s6_compliant`] 自体は `const fn`
//! なので、`Verb` の `entries` は必ずコンパイル時に検査される)。
//!
//! ### 判定規則(発注書の素案からの実務的な補正)
//!
//! 発注書は素案として「entries が2つ以上、または明示的に `ShortcutOnly`」を
//! 提案していたが、そのまま実装すると **メニューにしか無い正当な動詞**
//! (`New Layer`/`Freeze`/`Unfreeze`/`Undo`/`Redo`/`Documentation` 等 —
//! メニューバーは常設で「隠れていない」ので単一入口で全く問題ない)まで
//! 違反として弾いてしまう。実装した規則は:
//!
//! > `entries` に [`Entry::Context`] または [`Entry::PanelControl`]
//! > (「隠れた」入口)が**1つも無ければ無条件で合格**。
//! > 1つでもあれば、隠れていない入口([`Entry::Menu`]/[`Entry::ShortcutOnly`]/
//! > [`Entry::ExternalMenu`])が**最低1つ**なければ不合格(単純な
//! > `entries.len() >= 2` だと `Context`+`PanelControl` のように隠れた入口が
//! > 2つ揃っただけの組を誤って合格させてしまうため、隠れている/いないを
//! > 区別して数える)。
//!
//! これは現行 `context.rs` の `assert_s6_compliant`(右クリック項目だけを
//! メニューバー/shortcut registry と突き合わせる、メニュー単体の項目は
//! 監査対象外)と同じ実効判定を、`Verb` 単体からでも下せる形に一般化した
//! もの — 実データで検証済み(`tests/registry_invariants.rs`)。
//!
//! [`Entry::ShortcutOnly`]/[`Entry::ExternalMenu`] はどちらも「隠れていない
//! 入口」として扱う — 前者は `context.rs::SHORTCUT_ONLY_REGISTRY` と同じ
//! 概念(shell 実装済みキーの転記)、後者はこの crate にはまだ無い
//! menu 項目(shell 側 `menu.rs` の未移行分、例: keyframe Interpolation)を
//! 出典つきで指す新設の逃げ道 — 転記の規律は `SHORTCUT_ONLY_REGISTRY` と
//! 同じ(出典行を文字列に埋める)。

/// メニューバーの4本(`motolii_menubar::menus` の4関数に対応)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSlot {
    Edit,
    Layer,
    Window,
    Help,
}

/// 右クリック文脈の4種(`motolii_menubar::context` の4関数に対応)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSlot {
    Clip,
    LayerRow,
    Canvas,
    Keyframe,
}

/// パネル常設コントロール(メニューでも右クリックでもない、rail の glyph
/// button や Inspector の色 swatch のような「常に見えている単一ウィジェット」
/// の入口)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSlot {
    /// timeline rail の m/s/l glyph button(`rail.rs:337-339`)。
    RailGlyph,
    /// Inspector の label color swatch(`inspector-pane/src/lib.rs:2827`)。
    InspectorSwatch,
}

/// 動詞1つが持ちうる入口の種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// メニューバーの項目として存在する。
    Menu(MenuSlot),
    /// 右クリックメニューの項目として存在する。
    Context(ContextSlot),
    /// パネル常設コントロールとして存在する。
    PanelControl(PanelSlot),
    /// メニュー項目化されていない実装済み shortcut(出典を文字列で転記 —
    /// `context.rs::SHORTCUT_ONLY_REGISTRY` と同じ規律)。
    ShortcutOnly(&'static str),
    /// この crate にはまだ無い menu 項目(shell 側の未移行実装、出典を
    /// 文字列で転記)。
    ExternalMenu(&'static str),
}

/// [`Entry::Context`]/[`Entry::PanelControl`] は「隠れた」入口 — 右クリック
/// または常設だが目立たないウィジェット。この2種**だけ**が単一入口だと
/// S6 違反になりうる(モジュール冒頭 doc の判定規則参照)。
pub const fn is_hidden(entry: &Entry) -> bool {
    matches!(entry, Entry::Context(_) | Entry::PanelControl(_))
}

/// S6 監査本体(`const fn` — `static` 初期化子から呼べる)。モジュール冒頭
/// doc「判定規則」参照。**「隠れた入口が1つでもあるなら、隠れていない入口が
/// 最低1つ要る」** — 単純な `entries.len() >= 2` だと Context+PanelControl
/// (両方とも隠れている)の2つ組が誤って合格してしまうため、隠れた/隠れて
/// いないを区別して数える(`tests::two_hidden_entries_alone_are_not_compliant`
/// が退行検知)。
pub const fn s6_compliant(entries: &[Entry]) -> bool {
    let mut has_hidden = false;
    let mut has_visible = false;
    let mut i = 0;
    while i < entries.len() {
        if is_hidden(&entries[i]) {
            has_hidden = true;
        } else {
            has_visible = true;
        }
        i += 1;
    }
    if !has_hidden {
        return true;
    }
    has_visible
}

/// `Verb::entries` の唯一の構築口。違反時は **const 評価がパニックし
/// コンパイルが失敗する**(`static VERB: Verb = Verb { entries:
/// s6_checked(&[...]), .. }` という形で使う限り、実行時チェックを待たない)。
pub const fn s6_checked(entries: &'static [Entry]) -> &'static [Entry] {
    assert!(
        s6_compliant(entries),
        "S6 違反(Ableton 可視性原理): この動詞は Context/PanelControl \
         (隠れた入口)しか持たない — Menu か ShortcutOnly か ExternalMenu の \
         いずれかで第二の入口を明示すること(裁定195参照)"
    );
    entries
}

/// 動詞1つの記述(message は持たない — 呼び手ごとに異なる具体 `Message` は
/// [`generate`] 側で `motolii_menubar::menus::XxxMessages<M>` 等から渡す)。
#[derive(Debug, Clone, Copy)]
pub struct Verb {
    /// 安定 id(crate 内一意、`"edit.undo"` のような `<所属>.<動詞>` 形)。
    pub id: &'static str,
    /// 表示ラベル(メニュー・右クリックの両方で同じ文字列を使う)。
    pub label: &'static str,
    /// 表示専用の shortcut 併記(実装済み割当のみ、S6 = 飾り禁止)。
    pub shortcut: Option<&'static str>,
    /// 入口の集合。[`s6_checked`] を通した値だけを渡すこと。
    pub entries: &'static [Entry],
    /// 対応する `next/reference/normal-map.tsv` の行id(無ければ空 —
    /// Motolii 固有動詞や rail/Inspector 発の動詞は出典ゼロ)。複数可
    /// (例: `Group` は 455/456/457 の3行が同じ動詞へ収束)。
    pub map_ids: &'static [u32],
    /// 所属 bundle(`intent-bundles.tsv` の id、出典が無ければ `None`)。
    pub bundle: Option<&'static str>,
}

pub mod generate;
pub mod registry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_only_entry_is_compliant() {
        assert!(s6_compliant(&[Entry::Menu(MenuSlot::Edit)]));
    }

    #[test]
    fn shortcut_only_alone_is_compliant() {
        // 現行データには存在しない形(SHORTCUT_ONLY_REGISTRY は常に Context と
        // 対で使われる)だが、「隠れた入口が無ければ無条件合格」という規則の
        // 境界値として確認しておく。
        assert!(s6_compliant(&[Entry::ShortcutOnly("Enter")]));
    }

    #[test]
    fn context_alone_is_not_compliant() {
        assert!(!s6_compliant(&[Entry::Context(ContextSlot::LayerRow)]));
    }

    #[test]
    fn panel_control_alone_is_not_compliant() {
        assert!(!s6_compliant(&[Entry::PanelControl(PanelSlot::RailGlyph)]));
    }

    #[test]
    fn context_plus_menu_is_compliant() {
        assert!(s6_compliant(&[
            Entry::Menu(MenuSlot::Layer),
            Entry::Context(ContextSlot::LayerRow),
        ]));
    }

    #[test]
    fn context_plus_shortcut_only_is_compliant() {
        assert!(s6_compliant(&[
            Entry::ShortcutOnly("Enter"),
            Entry::Context(ContextSlot::LayerRow),
        ]));
    }

    #[test]
    fn context_plus_external_menu_is_compliant() {
        assert!(s6_compliant(&[
            Entry::ExternalMenu("shell menu.rs:112-142"),
            Entry::Context(ContextSlot::Keyframe),
        ]));
    }

    #[test]
    fn two_hidden_entries_alone_are_not_compliant() {
        // Context+PanelControl は両方とも「隠れた」— 2つあっても目立つ入口が
        // 無ければ不合格(裁定195前夜の Hide/Solo/Lock がまさにこの形だった:
        // rail glyph のみで、メニューにも右クリックにも無かった)。
        assert!(!s6_compliant(&[
            Entry::Context(ContextSlot::LayerRow),
            Entry::PanelControl(PanelSlot::RailGlyph),
        ]));
    }
}
