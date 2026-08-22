//! pane-local ephemeral 状態(rail scope + 検索文字列、切片 B2)。**Document
//! ではない** — `settings_pane::BackgroundFieldDraft`/
//! `timeline_pane::write::PaneState` と同じ「front だけが持つ transient」の形
//! (crate 冒頭 doc「B2 以降、rail/filter(shell-state)…」の実体はこの1本の
//! struct — `Session` を共有する必要が無い pane-local 状態なので
//! `motolii-shell-state` へは置かない、`timeline_pane::PaneState` が Document/
//! Session を触る drag 状態を持つのと違い、この pane は Document を一切
//! 書かない = 引数も `&mut self` だけで完結する)。
//!
//! `Shell` は `browser_pane::PaneState` を1個持ち、`Message::Browser` の腕を
//! `PaneState::update` へそのまま委譲する(`timeline_pane::PaneState::update`
//! と同型の委譲、`motolii-shell` 側の doc 参照)。**B3: パネルの開閉トグル**
//! (`settings_pane::Message::ToggleSettingsPanel` 相当)を追加 — ただし
//! Settings とは置き場が違う: Settings は `settings_panel_open` を `Shell`
//! 自身に持つ(pane crate が `&mut self` を持てないため、Shell 側の専用 glue
//! 関数が per-variant で分岐する)。Browser は B1/B2 から既に `PaneState` が
//! transient 状態(scope/検索欄)を1個持つ形を確立済みなので、開閉フラグも
//! **同じ `PaneState` の内側**へ足す方が「`Message::Browser` は
//! `PaneState::update` への直委譲のまま」という B1/B2 の委譲形を崩さずに済む
//! (`Shell::update` 側に per-variant 分岐を増やさない — crate 冒頭 doc の
//! 「pane split 流儀」どおり)。`Shell::view` は [`PaneState::is_open`] を読んで
//! 表示するかどうかだけ判断する。
use crate::model::{LibraryTab, RailScope};

/// pane ローカル Message(裁定160 切片以降の一貫した形 — root
/// `motolii_shell::Message::Browser(Message)` が1本で畳む)。
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// rail の scope ボタン、または filter shelf の同義チップ
    /// (`model::RAIL_SCOPES`/`model::FILTER_CHIPS`)。mock は両方が同じ
    /// `state.source`/`state.tag` を書く — 2つの入口が同じ状態を書く
    /// (Ableton可視性原理: 唯一の入口にしない、ユーザー記憶
    /// `ableton-visibility-principle.md`)。
    SelectScope(RailScope),
    /// タブ帯(mock `.libraryTabs`)のタブクリック。mock `chooseTab` の転写 —
    /// 非 media タブへ移る時は rail scope を `AllMedia` へ戻す(mock が
    /// `source='all'`/`tag=''` へ戻すのと同じ意味 — scope は media 種別の
    /// 語彙なので他タブでは意味を持たない)。検索文字列は mock 同様保持。
    SelectTab(LibraryTab),
    /// 検索欄への打鍵。**即時反映**(Settings の下書き/Enter 確定とは違う —
    /// 検索は絞り込みのプレビューそのものなので Enter を待つ理由が無い、mock
    /// `search.addEventListener('input', ...)` と同じ即時反映)。
    QueryChanged(String),
    /// filter shelf の Clear ボタン(mock `.clearFilter`)。scope を
    /// `RailScope::AllMedia` へ、検索文字列を空へ戻す。
    ClearFilters,
    /// header の "Browser" トグル(B3)。`settings_pane::Message::
    /// ToggleSettingsPanel` と同格の表示専用フラグ — Document にも undo 履歴
    /// にも乗らない。
    ToggleBrowserPanel,
}

/// Browser pane 専用の transient 状態(rail scope + 検索欄の下書き + パネル
/// 開閉、B3 でopen を追加)。**`Default` = 初期状態そのもの**
/// (`RailScope::default()` = `AllMedia`・検索欄は空 = mock の初期状態
/// `state = {source: 'all', tag: '', query: ''}` と一致・`open` は `bool` の
/// `Default` である `false` = 閉じた状態、`settings_panel_open`/
/// `edit_menu_open` と同じ「既定は閉」)。
#[derive(Default)]
pub struct PaneState {
    /// active なタブ(mock `state.tab`、既定 = `LibraryTab::Media`)。
    tab: LibraryTab,
    scope: RailScope,
    query: String,
    open: bool,
}

impl PaneState {
    pub fn new() -> Self {
        Self::default()
    }

    /// active なタブ(mock `state.tab`)。`pane_view` がタブ帯の active 表示と
    /// catalog 投影([`crate::model::catalog`])の分岐に読む。
    pub fn tab(&self) -> LibraryTab {
        self.tab
    }

    pub fn scope(&self) -> RailScope {
        self.scope
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// パネルが開いているか(B3)。`Shell::view` がこれを読んで
    /// `browser_pane::view` を木へ差し込むかどうかを決める(`settings_pane`
    /// の `Shell::settings_panel_open()` と同格の口)。
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// pane 側の唯一の書き口。
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectScope(scope) => self.scope = scope,
            Message::SelectTab(tab) => {
                self.tab = tab;
                // mock `chooseTab`: `if (tab !== 'media') { state.source = 'all';
                // state.tag = ''; }` の転写。query は mock 同様タブを跨いで保持。
                if tab != LibraryTab::Media {
                    self.scope = RailScope::AllMedia;
                }
            }
            Message::QueryChanged(text) => self.query = text,
            Message::ClearFilters => {
                self.scope = RailScope::AllMedia;
                self.query.clear();
            }
            Message::ToggleBrowserPanel => self.open = !self.open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_all_media_with_an_empty_query() {
        let state = PaneState::new();
        assert_eq!(state.scope(), RailScope::AllMedia);
        assert_eq!(state.query(), "");
    }

    #[test]
    fn select_scope_replaces_the_current_scope() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Video));
        assert_eq!(state.scope(), RailScope::Video);
    }

    #[test]
    fn query_changed_replaces_the_draft_verbatim() {
        let mut state = PaneState::new();
        state.update(Message::QueryChanged("clip".to_owned()));
        assert_eq!(state.query(), "clip");
    }

    /// `Clear` は scope も query も両方まとめて初期状態へ戻す
    /// (mock `#clear-filter` ハンドラの2行と同じ)。
    #[test]
    fn clear_filters_resets_scope_and_query_together() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Audio));
        state.update(Message::QueryChanged("tone".to_owned()));
        state.update(Message::ClearFilters);
        assert_eq!(state.scope(), RailScope::AllMedia);
        assert_eq!(state.query(), "");
    }

    // -----------------------------------------------------------------
    // B3 EXACT TARGET: パネル開閉トグル。
    // -----------------------------------------------------------------

    #[test]
    fn panel_starts_closed() {
        let state = PaneState::new();
        assert!(!state.is_open());
    }

    #[test]
    fn toggle_browser_panel_opens_then_closes() {
        let mut state = PaneState::new();
        state.update(Message::ToggleBrowserPanel);
        assert!(state.is_open(), "1回目のトグルで開かない");
        state.update(Message::ToggleBrowserPanel);
        assert!(!state.is_open(), "2回目のトグルで閉じない");
    }

    /// scope/query の操作は開閉フラグに影響しない(独立した2軸)。
    #[test]
    fn toggling_the_panel_does_not_disturb_scope_or_query() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Audio));
        state.update(Message::QueryChanged("tone".to_owned()));
        state.update(Message::ToggleBrowserPanel);
        assert_eq!(state.scope(), RailScope::Audio);
        assert_eq!(state.query(), "tone");
        assert!(state.is_open());
    }

    // -----------------------------------------------------------------
    // タブ状態(mock `state.tab`、B3 転写の取り残し回収)。
    // -----------------------------------------------------------------

    /// **ORACLE**: 初期タブは media(mock `state = {tab: 'media', ...}`)。
    #[test]
    fn initial_tab_is_media() {
        assert_eq!(PaneState::new().tab(), LibraryTab::Media);
    }

    #[test]
    fn select_tab_switches_the_tab() {
        let mut state = PaneState::new();
        state.update(Message::SelectTab(LibraryTab::Effects));
        assert_eq!(state.tab(), LibraryTab::Effects);
        state.update(Message::SelectTab(LibraryTab::Media));
        assert_eq!(state.tab(), LibraryTab::Media);
    }

    /// mock `chooseTab`: 非 media タブへ移る時は `source='all'`/`tag=''` へ
    /// 戻す(rail scope は media 専用の語彙)。検索文字列は mock 同様保持。
    #[test]
    fn selecting_a_non_media_tab_resets_the_rail_scope_but_keeps_the_query() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Audio));
        state.update(Message::QueryChanged("tone".to_owned()));
        state.update(Message::SelectTab(LibraryTab::Panels));
        assert_eq!(
            state.scope(),
            RailScope::AllMedia,
            "非 media タブで scope が残っている"
        );
        assert_eq!(
            state.query(),
            "tone",
            "検索文字列は mock 同様タブを跨いで保持のはず"
        );
    }

    /// media タブへ戻る遷移は scope を触らない(mock も `tab === 'media'` では
    /// source/tag を書かない)。
    #[test]
    fn returning_to_media_does_not_disturb_the_scope() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Video));
        state.update(Message::SelectTab(LibraryTab::Media));
        assert_eq!(state.scope(), RailScope::Video);
    }

    /// タブ切替は開閉フラグに影響しない(独立した軸)。
    #[test]
    fn selecting_a_tab_does_not_disturb_the_open_flag() {
        let mut state = PaneState::new();
        state.update(Message::ToggleBrowserPanel);
        state.update(Message::SelectTab(LibraryTab::Create));
        assert!(state.is_open());
    }
}
