//! owns: パネルのフォーカス/巡回状態(発注 2026-08-22、normal-map bundle
//! B25 の「状態側」)。
//!
//! ## 位置づけ
//! `next/ui/motolii-menubar/src/menus.rs`(MENU2、コミット `229f5cc2`)が
//! Window メニューの**項目定義**(Browser 開閉・Inspector/Stage/Timeline
//! フォーカス・Cycle Panel・Close Panel の6項目、出典
//! 1525/801/1317/1316/1503/1499)を menubar crate に作ったが、その**状態側**
//! (今どの pane にフォーカスがあるか・巡回の次はどれか)がまだ無かった。
//!
//! OWNS-JUSTIFICATION(A): 裁定160切片6(pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §2.3) — `motolii-timeline-pane`
//! が `motolii-shell` へ依存できない循環を避けるためleaf crate化する構造上の
//! 必然性を具体的に引用(裁定215 棚卸し 2026-08-23 #29)。
//! ここがその置き場——`next/ui/motolii-shell-state/src/layout.rs`(B26、
//! パネル分割木+hidden葉、コミット `f69e709e`)の**隣**に置く: どちらも
//! 「Session 水準の pane 状態」を iced 非依存の純データで表す同じ様式だが、
//! コード上は独立している(下記「layout.rs との関係」参照)。
//!
//! ## layout.rs との関係(依存ゼロ、`PaneKind` だけを橋にする)
//! `layout.rs` の `WorkspaceLayout<K>`/`LayoutNode<K>` は `K: Copy + Eq` に
//! しか依存しないジェネリック型——`layout.rs` 自体は `K` の具体形を知らない。
//! shell 側は本モジュールの [`PaneKind`] を `K` に当てて
//! `WorkspaceLayout<PaneKind>` を作る想定(`layout.rs` モジュール doc の
//! 「shell 結線手順」2 と同じ橋渡し)。そうすると
//! `WorkspaceLayout::visible_kinds() -> Vec<PaneKind>` の戻り値の型が、
//! そのまま [`FocusState::cycle_next`]/[`FocusState::cycle_prev`] の引数の
//! 型と一致し、**この `focus` モジュールが `layout` モジュールを `use` する
//! 必要が一切無い**——2つのモジュールは [`PaneKind`] という共有の語彙だけで
//! つながる(mod 間の import も依存追加も発生しない)。
//!
//! 実務上の注記: この worktree はまだ `layout.rs`(B26 レーンの成果)を
//! 持っていない(このブランチに未着地)。本ファイルはその不在に関わらず
//! 単独でコンパイル・テストできる——`layout` モジュールへの `mod`/`use` を
//! 一切書かないのはそのため(発注書「既存 layout.rs は読み専用」の対応:
//! 読む対象が今この worktree に無いので、依存を作らない設計で応じた)。
//!
//! ## `PaneKind` を独自定義する理由
//! 実在する4パネルは `next/shell/motolii-shell/src/pane_layout.rs::PaneKind`
//! /`menus::window_menu` と同じ Browser/Inspector/Stage/Timeline だが、この
//! crate は `motolii-shell`/`motolii-timeline-pane` のどちらも import
//! できない(`lib.rs` 冒頭 doc の既定の依存方向: `motolii-shell` →
//! `motolii-shell-state` の一方向のみ)。`Session`/`KeySelector` が shell 側の
//! 型を複製せず独自定義しているのと同じ理由で、ここでも独自に [`PaneKind`]
//! を定義する——値と並びだけを一致させる(`pane_layout::PaneKind`/
//! `menus::window_menu` の Browser → Inspector → Stage → Timeline 順)。
//!
//! ## B25 見送り75行の再確認(発注書ステップ1)
//! MENU2 が見送った75行(`menus.rs` モジュール doc の表)を、フォーカス/
//! 巡回で表現できる行が混ざっていないか読み直した。新たに状態を要する行は
//! 無かった——2件だけ注記が要る:
//!
//! - id 802「Active Window: Media Folders」(BMD): 概念上は Browser に最も
//!   近いが、MENU2 は明示的な「Focus Browser」項目を作らなかった
//!   (`toggle_browser` の開閉のみ)。状態側は既に [`PaneKind::Browser`] を
//!   巡回/フォーカスの対象に含めている——将来 menu 層が明示項目を足しても
//!   状態側の追加作業は不要。
//! - id 1511/1512「Maximize or restore panel under pointer」(Premiere):
//!   pane 巡回ではなく「他パネルを隠して1枚を最大化する」レイアウト操作
//!   ——`layout.rs` の hide/show(木の形)の仕事であって、本モジュールの
//!   focus/cycle の対象ではないと判断し、見送りのまま据え置く(実装は
//!   `layout.rs` 着地後の別レーンへ)。
//!
//! 他の未消化行(799/800/1445 の Active Window 系・978〜1040 の panel_window
//! 一覧・1520〜1527 の Open or close 系等)は MENU2 の doc がすでに述べる
//! 「対応する pane 種別が Motolii に実在しない」がそのまま理由——状態を
//! 足しても表現できる対象が無い(4パネル以外の pane が無い)。
//!
//! ## キーボードフォーカスとの関係(検討結果)
//! 「text_input が編集中は pane フォーカスを奪わない」という規則を型で
//! 表現できるか検討した。この crate は text_input の実体(`iced::widget::
//! text_input` 等)を持たず、持つべきでもない(持つと「iced 非依存の純
//! データ」という性質が壊れる)——**真偽そのものをこの crate 内に持ち込む
//! ことはできない**。代わりに [`KeyboardClaim`] という2値の列挙を「呼び出し
//! 口の必須引数」にし、`cycle_next`/`cycle_prev`/`close_focused` の**素の
//! 版とガード版を分離した**(`*_unless_typing` 系)。呼び手(`Shell::update`)
//! は text_input の focus 有無を見て `KeyboardClaim::TextEditing`/`Panes` を
//! 都度選ぶ——「奪わない」という規則自体をこの crate が強制することは
//! できないが、ガード版の型シグネチャに `KeyboardClaim` を要求することで、
//! 「text_input 中かどうかを一度も見ずに pane ショートカットを実行する」
//! 呼び出しを素の版と見分けやすくする(判断の明示を型で促す、が正確な
//! 表現——強制ではない)。

/// 4つのパネル種別。値・並びは `pane_layout::PaneKind`/
/// `motolii-menubar::menus::window_menu` の Browser/Inspector/Stage/Timeline
/// と一致させる(モジュール冒頭 doc「独自定義する理由」参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PaneKind {
    Browser,
    Inspector,
    Stage,
    Timeline,
}

/// 呼び出し口で明示する「今キーボード入力を誰が掴んでいるか」。pane 巡回/
/// close はショートカット駆動の操作なので、text_input 編集中はそれらを
/// 実行してはならない——この型はその判断を型シグネチャへ持ち上げる
/// (モジュール冒頭 doc「キーボードフォーカスとの関係」参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardClaim {
    /// text_input 等がキー入力を掴んでいない——pane 巡回/close を実行してよい。
    Panes,
    /// text_input 等がキー入力を掴んでいる——pane 巡回/close を実行しては
    /// ならない。
    TextEditing,
}

/// パネルフォーカス/巡回状態。**Session 水準**(Document に乗らない —
/// `next/ui/motolii-shell-state/src/lib.rs::Session::timeline_fold` や
/// `layout.rs::WorkspaceLayout` と同格の「意味を持たない純表示状態」)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusState {
    focused: Option<PaneKind>,
}

impl FocusState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 今フォーカスされている pane(未フォーカスなら `None`)。
    pub fn focused(&self) -> Option<PaneKind> {
        self.focused
    }

    /// 直接指定してフォーカスする(Window メニューの Inspector/Stage/
    /// Timeline 項目、`menus::WindowMenuMessages::focus_*` の受け口)。
    /// **可視性は呼び手の責任** — この関数自体は `kind` が今見えているかを
    /// 検証しない(hide 直後は [`FocusState::reconcile`] を呼んで補正する
    /// 設計、モジュール冒頭 doc 参照)。
    pub fn focus(&mut self, kind: PaneKind) {
        self.focused = Some(kind);
    }

    /// フォーカスを外す(誰もフォーカスしていない状態へ)。
    pub fn clear(&mut self) {
        self.focused = None;
    }

    /// 表示中の pane だけを巡回して次へ進む(Window > Cycle Panel、
    /// normal-map 1503)。`visible` は `WorkspaceLayout::visible_kinds()` の
    /// 戻り値をそのまま渡す想定(並び=分割木の DFS 順、`layout.rs` 参照)—
    /// **隠れた pane は `visible` に現れないので自動的に飛ばされる**。
    /// `visible` が空なら巡回先が無い(全 hide)ので `None` を返しフォーカスも
    /// 外す。今のフォーカスが `visible` に無ければ(隠れた/存在しない pane を
    /// 指していた場合)先頭へ安全側で倒す。
    pub fn cycle_next(&mut self, visible: &[PaneKind]) -> Option<PaneKind> {
        self.cycle(visible, 1)
    }

    /// [`cycle_next`](Self::cycle_next) の逆向き。
    pub fn cycle_prev(&mut self, visible: &[PaneKind]) -> Option<PaneKind> {
        self.cycle(visible, -1)
    }

    fn cycle(&mut self, visible: &[PaneKind], step: i64) -> Option<PaneKind> {
        if visible.is_empty() {
            self.focused = None;
            return None;
        }
        let len = visible.len() as i64;
        let current_index = self.focused.and_then(|k| visible.iter().position(|v| *v == k));
        let next_index = match current_index {
            Some(i) => (((i as i64 + step) % len + len) % len) as usize,
            // 未フォーカス、または今のフォーカスがもう `visible` に無い
            // (隠れた/閉じた pane を指していた) — 先頭から巡回を始める。
            None => 0,
        };
        let next = visible[next_index];
        self.focused = Some(next);
        Some(next)
    }

    /// hide 直後などに呼ぶ: 今のフォーカスが `visible` から消えていたら外す
    /// (「閉じた pane にフォーカスが残らない」の直接実装)。フォーカスを
    /// 実際に外したら `true`。
    pub fn reconcile(&mut self, visible: &[PaneKind]) -> bool {
        match self.focused {
            Some(kind) if !visible.contains(&kind) => {
                self.focused = None;
                true
            }
            _ => false,
        }
    }

    /// Window > Close Panel(normal-map 1499/1500、`= hide`): 今フォーカス
    /// 中の pane を返し、フォーカスを外す。**実際に pane を隠す操作
    /// (`WorkspaceLayout::hide`)は呼び手の仕事** — このモジュールは
    /// 「どの pane を閉じるか」の決定だけを持つ(モジュール冒頭 doc
    /// 「layout.rs との関係」参照、副作用の分離)。フォーカスが無ければ
    /// `None` で無変化。
    pub fn close_focused(&mut self) -> Option<PaneKind> {
        self.focused.take()
    }

    // ---- キーボード主張ガード版(モジュール冒頭 doc 参照) ----

    /// [`cycle_next`](Self::cycle_next) のガード版。`claim` が
    /// `TextEditing` の間は no-op(現在のフォーカスをそのまま返す)。
    pub fn cycle_next_unless_typing(
        &mut self,
        visible: &[PaneKind],
        claim: KeyboardClaim,
    ) -> Option<PaneKind> {
        match claim {
            KeyboardClaim::TextEditing => self.focused,
            KeyboardClaim::Panes => self.cycle_next(visible),
        }
    }

    /// [`cycle_prev`](Self::cycle_prev) のガード版。
    pub fn cycle_prev_unless_typing(
        &mut self,
        visible: &[PaneKind],
        claim: KeyboardClaim,
    ) -> Option<PaneKind> {
        match claim {
            KeyboardClaim::TextEditing => self.focused,
            KeyboardClaim::Panes => self.cycle_prev(visible),
        }
    }

    /// [`close_focused`](Self::close_focused) のガード版。`TextEditing` の
    /// 間は no-op(フォーカスは外さず `None` を返す — 「閉じなかった」ことを
    /// 呼び手が判別できるよう、フォーカス済み pane を返す素の版とは異なり
    /// 常に `None`)。
    pub fn close_focused_unless_typing(&mut self, claim: KeyboardClaim) -> Option<PaneKind> {
        match claim {
            KeyboardClaim::TextEditing => None,
            KeyboardClaim::Panes => self.close_focused(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [PaneKind; 4] =
        [PaneKind::Browser, PaneKind::Inspector, PaneKind::Stage, PaneKind::Timeline];

    // -----------------------------------------------------------------
    // 既定状態・直接フォーカス。
    // -----------------------------------------------------------------

    #[test]
    fn default_state_has_no_focus() {
        let focus = FocusState::default();
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn focus_sets_the_given_pane() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Stage);
        assert_eq!(focus.focused(), Some(PaneKind::Stage));
    }

    #[test]
    fn clear_removes_the_focus() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Inspector);
        focus.clear();
        assert_eq!(focus.focused(), None);
    }

    // -----------------------------------------------------------------
    // 巡回: 表示中の pane だけを回る。
    // -----------------------------------------------------------------

    /// 未フォーカスから始めた cycle_next は先頭(`visible[0]`)へ。
    #[test]
    fn cycle_next_from_unfocused_lands_on_the_first_visible_pane() {
        let mut focus = FocusState::new();
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Browser));
    }

    /// 巡回は宣言順(Browser→Inspector→Stage→Timeline)で進み、末尾から
    /// 先頭へ折り返す。
    #[test]
    fn cycle_next_advances_in_order_and_wraps_around() {
        let mut focus = FocusState::new();
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Browser));
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Inspector));
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Stage));
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Timeline));
        assert_eq!(focus.cycle_next(&ALL), Some(PaneKind::Browser), "末尾から先頭へ折り返していない");
    }

    /// cycle_prev は逆順に進み、先頭から末尾へ折り返す。
    #[test]
    fn cycle_prev_goes_backwards_and_wraps_around() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Browser);
        assert_eq!(focus.cycle_prev(&ALL), Some(PaneKind::Timeline), "先頭から前へ回すと末尾に折り返さない");
        assert_eq!(focus.cycle_prev(&ALL), Some(PaneKind::Stage));
    }

    /// **オラクル**: 巡回は隠れた pane を飛ばす — `visible` に含まれない
    /// pane(ここでは Stage を hide した想定)には決して止まらない。
    #[test]
    fn cycle_skips_panes_that_are_not_in_the_visible_slice() {
        let visible = [PaneKind::Browser, PaneKind::Inspector, PaneKind::Timeline];
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Inspector);

        let next = focus.cycle_next(&visible);

        assert_eq!(next, Some(PaneKind::Timeline), "Stage が隠れているのに巡回対象に含まれている");
        assert_ne!(focus.focused(), Some(PaneKind::Stage));
    }

    /// 今のフォーカスが `visible` から消えている(閉じられた)場合、巡回は
    /// panic せず先頭から始め直す(安全側フォールバック)。
    #[test]
    fn cycle_falls_back_to_the_first_visible_pane_when_current_focus_is_gone() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Stage); // Stage を直接フォーカスした後に隠されたと仮定
        let visible = [PaneKind::Browser, PaneKind::Inspector, PaneKind::Timeline];

        let next = focus.cycle_next(&visible);

        assert_eq!(next, Some(PaneKind::Browser));
    }

    /// **オラクル**: 空(全 hide)の扱い — 巡回先が無いので `None`、
    /// フォーカスも外れる(閉じた pane にフォーカスが残らない、の極端形)。
    #[test]
    fn cycle_with_no_visible_panes_returns_none_and_clears_focus() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Timeline);

        let next = focus.cycle_next(&[]);

        assert_eq!(next, None);
        assert_eq!(focus.focused(), None, "全 hide なのにフォーカスが残っている");
    }

    // -----------------------------------------------------------------
    // reconcile: hide 直後の後始末。
    // -----------------------------------------------------------------

    /// **オラクル**: 閉じた pane にフォーカスが残らない — フォーカス中の
    /// pane が `visible` から消えたら `reconcile` が外す。
    #[test]
    fn reconcile_clears_focus_when_the_focused_pane_is_no_longer_visible() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Browser);

        let changed = focus.reconcile(&[PaneKind::Inspector, PaneKind::Stage, PaneKind::Timeline]);

        assert!(changed);
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn reconcile_is_a_no_op_when_the_focused_pane_is_still_visible() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Inspector);

        let changed = focus.reconcile(&ALL);

        assert!(!changed);
        assert_eq!(focus.focused(), Some(PaneKind::Inspector));
    }

    #[test]
    fn reconcile_with_no_focus_is_a_no_op() {
        let mut focus = FocusState::new();
        assert!(!focus.reconcile(&ALL));
        assert_eq!(focus.focused(), None);
    }

    // -----------------------------------------------------------------
    // close_focused。
    // -----------------------------------------------------------------

    #[test]
    fn close_focused_returns_and_clears_the_focused_pane() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Stage);

        let closed = focus.close_focused();

        assert_eq!(closed, Some(PaneKind::Stage));
        assert_eq!(focus.focused(), None, "close_focused 後もフォーカスが残っている");
    }

    #[test]
    fn close_focused_with_no_focus_returns_none() {
        let mut focus = FocusState::new();
        assert_eq!(focus.close_focused(), None);
    }

    // -----------------------------------------------------------------
    // キーボード主張ガード版。
    // -----------------------------------------------------------------

    #[test]
    fn guarded_cycle_next_is_a_no_op_while_text_editing() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Browser);

        let result = focus.cycle_next_unless_typing(&ALL, KeyboardClaim::TextEditing);

        assert_eq!(result, Some(PaneKind::Browser), "text_input 編集中なのに巡回してしまった");
        assert_eq!(focus.focused(), Some(PaneKind::Browser));
    }

    #[test]
    fn guarded_cycle_prev_is_a_no_op_while_text_editing() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Stage);

        let result = focus.cycle_prev_unless_typing(&ALL, KeyboardClaim::TextEditing);

        assert_eq!(result, Some(PaneKind::Stage));
        assert_eq!(focus.focused(), Some(PaneKind::Stage));
    }

    #[test]
    fn guarded_close_focused_is_a_no_op_while_text_editing() {
        let mut focus = FocusState::new();
        focus.focus(PaneKind::Timeline);

        let result = focus.close_focused_unless_typing(KeyboardClaim::TextEditing);

        assert_eq!(result, None, "text_input 編集中の close は no-op なので None のはず");
        assert_eq!(focus.focused(), Some(PaneKind::Timeline), "text_input 編集中なのに閉じてしまった");
    }

    /// ガード版は `KeyboardClaim::Panes` の時、素の版と同じ結果を返す。
    #[test]
    fn guarded_methods_behave_like_the_unguarded_ones_when_panes_claim_the_keyboard() {
        let mut guarded = FocusState::new();
        let mut plain = FocusState::new();

        assert_eq!(
            guarded.cycle_next_unless_typing(&ALL, KeyboardClaim::Panes),
            plain.cycle_next(&ALL)
        );
        assert_eq!(
            guarded.cycle_prev_unless_typing(&ALL, KeyboardClaim::Panes),
            plain.cycle_prev(&ALL)
        );
        assert_eq!(
            guarded.close_focused_unless_typing(KeyboardClaim::Panes),
            plain.close_focused()
        );
    }
}
