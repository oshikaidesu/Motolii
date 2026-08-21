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
//! と同型の委譲、`motolii-shell` 側の doc 参照)。**パネルの開閉トグル
//! (`settings_pane::Message::ToggleSettingsPanel` 相当)はまだ無い** —
//! `Shell::view` への組み込みは B3(絵と一緒に配線、crate 冒頭 doc 参照)の
//! 範囲なので、この切片(B2)ではまだ要らない。
use crate::model::RailScope;

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
    /// 検索欄への打鍵。**即時反映**(Settings の下書き/Enter 確定とは違う —
    /// 検索は絞り込みのプレビューそのものなので Enter を待つ理由が無い、mock
    /// `search.addEventListener('input', ...)` と同じ即時反映)。
    QueryChanged(String),
    /// filter shelf の Clear ボタン(mock `.clearFilter`)。scope を
    /// `RailScope::AllMedia` へ、検索文字列を空へ戻す。
    ClearFilters,
}

/// Browser pane 専用の transient 状態(rail scope + 検索欄の下書き)。
/// **`Default` = 初期状態そのもの**(`RailScope::default()` = `AllMedia`・
/// 検索欄は空 = mock の初期状態 `state = {source: 'all', tag: '', query: ''}`
/// と一致)。
#[derive(Default)]
pub struct PaneState {
    scope: RailScope,
    query: String,
}

impl PaneState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scope(&self) -> RailScope {
        self.scope
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// pane 側の唯一の書き口。
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectScope(scope) => self.scope = scope,
            Message::QueryChanged(text) => self.query = text,
            Message::ClearFilters => {
                self.scope = RailScope::AllMedia;
                self.query.clear();
            }
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
}
