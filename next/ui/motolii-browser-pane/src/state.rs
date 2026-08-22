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
use crate::model::{CardKey, CreateKind, LibraryTab, PreviewScope, RailScope, SortKey, ViewMode};
use motolii_store::AssetId;

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
    /// 非 media タブの rail 行、または filter shelf の同義チップ
    /// (`model::preview_tags` — rail とチップが同ラベル・同 Message、
    /// [`Message::SelectScope`] と同型の「2つの入口が同じ状態を書く」形。
    /// 構造の対称化、利用者実窓指摘 2026-08-22)。
    SelectPreviewScope(PreviewScope),
    /// カタログのカード click(mock `selectCard` の単一選択のみ — Shift/⌘ の
    /// 複数選択・selection tray は予約地のまま)。media/preview の2系統を
    /// [`CardKey`] が型で分ける。
    SelectCard(CardKey),
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
    /// create タブのカードの**ダブルクリック**=「作る」(B36 実体化 —
    /// AE/Figma 慣習 S0: シングル=選択・ダブル=作成。view 側は
    /// `mouse_area::on_double_click` が発火する)。**pane-local には状態を
    /// 動かさない** — 実際のレイヤー生成は shell 結線(次波)で、`Shell` が
    /// この variant を横取りして `Intent` へ落とす形になる(`SelectCard` は
    /// ダブルクリックの1打目/2打目の press で別途 publish 済み)。drag で
    /// Stage/Timeline へ落とす経路は将来切片(発注の見送り明記)。
    CreateFromCard { kind: CreateKind },
    /// OS の file-drag が窓に入っている/出た(B08 続編: drop 先ハイライト)。
    /// shell 結線(次波)が `iced::window::Event::FileHovered`/
    /// `FilesHoveredLeft` をこの1本へ翻訳する想定 — pane は真偽だけ持ち、
    /// media タブの catalog 容器を drop 受け入れ面として塗り替える
    /// (`pane_grid` の `hovered_region` と同じ文法、`crate::drop_target_style`)。
    DropHoverChanged(bool),
    /// 取り込み(admit)直後の新規素材 id 列(B08 続編: 新規素材ハイライト)。
    /// shell 結線(次波)が `Shell::admit` の後に publish する想定。空 Vec は
    /// 「ハイライト消灯」。**Document ではない** — 台帳はハイライトを知らず、
    /// pane-local の一過性の光だけ(undo 履歴にも乗らない)。
    RecentlyAdmitted(Vec<AssetId>),
    /// create カードへの cursor 進入(`mouse_area::on_enter`)。create カード
    /// は button でなく mouse_area(ダブルクリックが要るため — button は press
    /// を capture して `on_double_click` へ届かない、fork `widget/src/
    /// mouse_area.rs::update` 実測)なので、hover の意匠だけ pane が自前で
    /// 持つ(Q0: 触れそうで触れない物は不合格 — hover 無反応にしない)。
    CardHovered(CardKey),
    /// create カードからの cursor 退出(`mouse_area::on_exit`)。隣カードへの
    /// 進入と同一 event 内で順序が前後しても hover を取りこぼさないよう、
    /// 「自分が hover 中の時だけ消す」(update 参照)。
    CardUnhovered(CardKey),
    /// 並べ替えキー選択(B08 第4切片「素材の整理」)。filter shelf のチップ
    /// (mock に類例が無い新規 UI、[`crate::model::SORT_KEYS`])から publish
    /// される。media タブのみ意味を持つ — 台帳 `AssetListItem` の実属性
    /// (名前/`AssetId`/種別)を並べ替える。preview-local カタログは宣言順を
    /// 保つ契約のまま([`crate::model::preview_visible`] doc 参照)なので、
    /// このキーは preview 系の絞り込みには一切効かない。
    SelectSortKey(SortKey),
    /// grid/list 表示形式切替(B08 第4切片。`Icon::GridView`/`Icon::ViewList`
    /// 在庫を使う icon+tooltip ペア、裁定187)。media/preview 両方の catalog
    /// grid の列数に効く(見た目の切替のみ — Document にも undo 履歴にも
    /// 乗らない、`ToggleBrowserPanel` と同格の表示専用フラグ)。
    SelectViewMode(ViewMode),
    /// 素材の置換(map 616「Replace selected footage item」/617「Replace
    /// selected source footage for selected layers」の消化 — store 側は
    /// `Intent::SetSource` が裁定112c で実装済み、UI だけが未着手だった行。
    /// 618(ドラッグ版)は Stage/Timeline 側の drop target が要る別 write-set
    /// のため対象外、`crate::model` 冒頭 doc 参照)。
    ///
    /// **この crate は `Intent::SetSource` を呼ばない**(`Settings`/`Export`
    /// pane と同じ「pane は Intent を呼ばない」分業、crate 冒頭 doc「shell
    /// 結線」参照)— supervisor が `AssetId` → `Asset` を引き、
    /// [`crate::model::asset_to_layer_source`] を呼んで `Some(source)` なら
    /// `Intent::SetSource { layer, source }` を dispatch する。
    /// [`PaneState::update`] はこの腕を no-op として扱う(pane-local な状態は
    /// 何も変わらない — supervisor が委譲の前後どちらかで副作用を差し込む)。
    ReplaceSelectedLayerSource(AssetId),
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
    /// 非 media タブの rail scope(mock `state.tag` の転写 — 既定 =
    /// `PreviewScope::All` = mock `tag: ''`。media の `scope` と独立の軸:
    /// mock は1本の `state.tag` を全タブで共有して混線し得るが、こちらは
    /// media 台帳絞り込みへ決して漏れない型の壁)。
    preview_scope: PreviewScope,
    /// カタログの単一選択(mock `state.selected` の転写、`Option` = 未選択。
    /// 複数選択・selection tray は予約地)。
    selected: Option<CardKey>,
    query: String,
    open: bool,
    /// cursor が乗っている create カード(B36 — [`Message::CardHovered`] doc
    /// 参照。button 経路のカードは iced の `button::Status::Hovered` が担う
    /// ので、ここに乗るのは mouse_area 経路の create カードだけ)。
    hovered: Option<CardKey>,
    /// OS file-drag が窓に入っているか(B08 — [`Message::DropHoverChanged`])。
    drop_hover: bool,
    /// 取り込み直後の新規素材(B08 — [`Message::RecentlyAdmitted`])。カード
    /// 選択かタブ切替で消灯する(「直後」の一過性 — 恒久の状態にしない)。
    recent: Vec<AssetId>,
    /// 並べ替えキー(B08 第4切片、既定 = `SortKey::Name`)。media タブ専用
    /// (`Message::SelectSortKey` doc 参照)だが軸としてはタブに依存しない
    /// 1個の transient 状態 — `open`/`view_mode` と同じ扱い(タブ切替で
    /// 破棄しない、`update` の `SelectTab` 腕参照)。
    sort_key: SortKey,
    /// grid/list 表示形式(B08 第4切片、既定 = `ViewMode::Grid` — mock 既定
    /// 表示 `data-view="grid"` に一致)。media/preview 両方の catalog に効く。
    view_mode: ViewMode,
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

    /// 非 media タブの rail scope(`pane_view` が rail の選択表示と
    /// [`crate::model::preview_visible`] の絞り込みに読む)。
    pub fn preview_scope(&self) -> PreviewScope {
        self.preview_scope
    }

    /// カタログの単一選択(`pane_view` がカードの選択意匠に読む)。
    pub fn selected(&self) -> Option<CardKey> {
        self.selected
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

    /// cursor が乗っている create カード(`pane_view` が hover 意匠に読む)。
    pub fn hovered(&self) -> Option<CardKey> {
        self.hovered
    }

    /// OS file-drag が窓に入っているか(`pane_view` が media タブの drop 先
    /// ハイライトに読む)。
    pub fn drop_hover(&self) -> bool {
        self.drop_hover
    }

    /// 取り込み直後の新規素材 id 列(`pane_view` がカードのハイライトに読む)。
    pub fn recently_admitted(&self) -> &[AssetId] {
        &self.recent
    }

    /// 並べ替えキー(`pane_view` が media catalog の並べ替えと sort チップの
    /// 選択表示に読む、B08 第4切片)。
    pub fn sort_key(&self) -> SortKey {
        self.sort_key
    }

    /// grid/list 表示形式(`pane_view` が catalog grid の列数と view mode
    /// トグルの選択表示に読む、B08 第4切片)。
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// pane 側の唯一の書き口。
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectScope(scope) => self.scope = scope,
            Message::SelectPreviewScope(scope) => self.preview_scope = scope,
            Message::SelectCard(key) => {
                self.selected = Some(key);
                // 新規素材ハイライトは「直後」の一過性 — カードへ触った時点で
                // 消灯する(触った=気づいた、以降は通常の選択文法へ)。
                self.recent.clear();
            }
            Message::SelectTab(tab) => {
                self.tab = tab;
                // mock `chooseTab`: `if (tab !== 'media') { state.source = 'all';
                // state.tag = ''; }` の転写。query は mock 同様タブを跨いで保持。
                // preview タグはタブ別の語彙なので、非 media タブへ入る度に
                // 全件へ戻す(mock の `state.tag = ''` と同じ意味)。
                if tab != LibraryTab::Media {
                    self.scope = RailScope::AllMedia;
                    self.preview_scope = PreviewScope::All;
                }
                // mock `chooseTab` は移動先タブの先頭カードを選択し直すが、
                // `PaneState` はカタログを持たない(view 引数で渡る)ため
                // 「未選択」へ戻す(選択の持ち越しで前タブの selection が
                // 亡霊として残るのを防ぐ — 逸脱として RETURN 記載)。
                self.selected = None;
                // hover/新規素材ハイライトもタブと一緒に流す(前タブの
                // 一過性状態を亡霊として持ち越さない — selected と同じ理由)。
                self.hovered = None;
                self.recent.clear();
            }
            Message::QueryChanged(text) => self.query = text,
            Message::ClearFilters => {
                self.scope = RailScope::AllMedia;
                self.preview_scope = PreviewScope::All;
                self.query.clear();
            }
            Message::ToggleBrowserPanel => self.open = !self.open,
            // 「作る」は pane-local に動かす状態が無い(選択はダブルクリックの
            // press が `SelectCard` で別途書く)。実生成= shell 結線(次波)が
            // この variant を `Shell::update` 側で横取りして `Intent` へ落とす。
            Message::CreateFromCard { .. } => {}
            Message::DropHoverChanged(hovering) => self.drop_hover = hovering,
            Message::RecentlyAdmitted(ids) => self.recent = ids,
            Message::CardHovered(key) => self.hovered = Some(key),
            Message::CardUnhovered(key) => {
                // 隣カードへの enter とこのカードの exit が同一 event 内で
                // 前後しても hover を取りこぼさない — 自分が hover 中の時だけ
                // 消す(stale unhover は no-op)。
                if self.hovered == Some(key) {
                    self.hovered = None;
                }
            }
            Message::SelectSortKey(key) => self.sort_key = key,
            Message::SelectViewMode(mode) => self.view_mode = mode,
            // 副作用(`Intent::SetSource` の dispatch)は supervisor の責務
            // (このメンバー doc・crate 冒頭 doc「shell 結線」参照) — pane-local
            // 状態には何も書かない。
            Message::ReplaceSelectedLayerSource(_) => {}
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

    // -----------------------------------------------------------------
    // 構造の対称化(2026-08-22): 非 media タブの rail scope + カード選択。
    // -----------------------------------------------------------------

    use crate::model::PreviewTag;
    use motolii_store::AssetId;

    /// **ORACLE**: 初期 preview scope は全件(mock `state.tag: ''`)。
    #[test]
    fn initial_preview_scope_is_all() {
        assert_eq!(PaneState::new().preview_scope(), PreviewScope::All);
    }

    #[test]
    fn select_preview_scope_replaces_the_current_preview_scope() {
        let mut state = PaneState::new();
        state.update(Message::SelectTab(LibraryTab::Effects));
        state.update(Message::SelectPreviewScope(PreviewScope::Tag(
            PreviewTag::Color,
        )));
        assert_eq!(state.preview_scope(), PreviewScope::Tag(PreviewTag::Color));
    }

    /// mock `chooseTab` の `state.tag = ''` 転写: 非 media タブへ入る度に
    /// preview scope は全件へ戻る(タグはタブ別の語彙 — effects の Color を
    /// create へ持ち越さない)。
    #[test]
    fn selecting_a_tab_resets_the_preview_scope() {
        let mut state = PaneState::new();
        state.update(Message::SelectTab(LibraryTab::Effects));
        state.update(Message::SelectPreviewScope(PreviewScope::Tag(
            PreviewTag::Color,
        )));
        state.update(Message::SelectTab(LibraryTab::Create));
        assert_eq!(state.preview_scope(), PreviewScope::All);
    }

    /// `Clear` は preview scope も併せて初期状態へ戻す(mock `data-clear-filter`
    /// が `state.tag = ''` を書くのと同じ — media の scope/query と同時)。
    #[test]
    fn clear_filters_resets_the_preview_scope_too() {
        let mut state = PaneState::new();
        state.update(Message::SelectTab(LibraryTab::Panels));
        state.update(Message::SelectPreviewScope(PreviewScope::Tag(
            PreviewTag::Notes,
        )));
        state.update(Message::ClearFilters);
        assert_eq!(state.preview_scope(), PreviewScope::All);
    }

    /// **ORACLE**: カード click は単一選択を記録する(mock `selectCard` の
    /// 単一選択パス)。
    #[test]
    fn select_card_records_the_selection() {
        let mut state = PaneState::new();
        assert_eq!(state.selected(), None, "初期状態は未選択のはず");
        state.update(Message::SelectCard(CardKey::Media(AssetId::from_raw(3))));
        assert_eq!(state.selected(), Some(CardKey::Media(AssetId::from_raw(3))));
        state.update(Message::SelectCard(CardKey::Preview("glow")));
        assert_eq!(state.selected(), Some(CardKey::Preview("glow")));
    }

    /// タブ切替で選択は未選択へ戻る(前タブの selection を亡霊として
    /// 持ち越さない — `update` の doc 参照)。
    #[test]
    fn selecting_a_tab_clears_the_card_selection() {
        let mut state = PaneState::new();
        state.update(Message::SelectCard(CardKey::Media(AssetId::from_raw(0))));
        state.update(Message::SelectTab(LibraryTab::Effects));
        assert_eq!(state.selected(), None);
    }

    // -----------------------------------------------------------------
    // B36: CreateFromCard(pane-local には状態を動かさない — shell 結線待ち)。
    // -----------------------------------------------------------------

    /// **ORACLE**: `CreateFromCard` は pane-local の状態を一切動かさない
    /// (tab/scope/選択/query 全て不変 — 実生成は shell の仕事、`Message::
    /// CreateFromCard` doc 参照)。
    #[test]
    fn create_from_card_is_pane_inert() {
        let mut state = PaneState::new();
        state.update(Message::SelectTab(LibraryTab::Create));
        state.update(Message::SelectCard(CardKey::Preview("rectangle")));
        state.update(Message::QueryChanged("rect".to_owned()));
        state.update(Message::CreateFromCard {
            kind: CreateKind::Rectangle,
        });
        assert_eq!(state.tab(), LibraryTab::Create);
        assert_eq!(state.selected(), Some(CardKey::Preview("rectangle")));
        assert_eq!(state.query(), "rect");
    }

    // -----------------------------------------------------------------
    // B08 続編: drop 先ハイライト+新規素材ハイライト。
    // -----------------------------------------------------------------

    /// **ORACLE**: 初期状態は drop-hover なし・新規素材ハイライトなし。
    #[test]
    fn initial_state_has_no_drop_hover_and_no_recent_assets() {
        let state = PaneState::new();
        assert!(!state.drop_hover());
        assert!(state.recently_admitted().is_empty());
    }

    /// `DropHoverChanged` は真偽をそのまま写す(file-drag の入/出)。
    #[test]
    fn drop_hover_follows_the_message() {
        let mut state = PaneState::new();
        state.update(Message::DropHoverChanged(true));
        assert!(state.drop_hover());
        state.update(Message::DropHoverChanged(false));
        assert!(!state.drop_hover());
    }

    /// **ORACLE**: `RecentlyAdmitted` は id 列を記録し、次の便が丸ごと
    /// 置き換える(累積しない — 「直後」だけの光)。
    #[test]
    fn recently_admitted_records_and_replaces() {
        let mut state = PaneState::new();
        state.update(Message::RecentlyAdmitted(vec![
            AssetId::from_raw(1),
            AssetId::from_raw(2),
        ]));
        assert_eq!(
            state.recently_admitted(),
            [AssetId::from_raw(1), AssetId::from_raw(2)]
        );
        state.update(Message::RecentlyAdmitted(vec![AssetId::from_raw(9)]));
        assert_eq!(state.recently_admitted(), [AssetId::from_raw(9)]);
    }

    /// カードへ触ると新規素材ハイライトは消灯する(触った=気づいた)。
    #[test]
    fn selecting_a_card_clears_the_recent_highlight() {
        let mut state = PaneState::new();
        state.update(Message::RecentlyAdmitted(vec![AssetId::from_raw(1)]));
        state.update(Message::SelectCard(CardKey::Media(AssetId::from_raw(1))));
        assert!(state.recently_admitted().is_empty());
    }

    /// タブ切替でも消灯する(selected と同じ「一過性を持ち越さない」)。
    #[test]
    fn selecting_a_tab_clears_the_recent_highlight() {
        let mut state = PaneState::new();
        state.update(Message::RecentlyAdmitted(vec![AssetId::from_raw(1)]));
        state.update(Message::SelectTab(LibraryTab::Effects));
        assert!(state.recently_admitted().is_empty());
    }

    // -----------------------------------------------------------------
    // B36: create カードの hover(mouse_area 経路の自前 hover)。
    // -----------------------------------------------------------------

    /// enter で hover が乗り、同じカードの exit で降りる。
    #[test]
    fn card_hover_enters_and_leaves() {
        let mut state = PaneState::new();
        state.update(Message::CardHovered(CardKey::Preview("solid")));
        assert_eq!(state.hovered(), Some(CardKey::Preview("solid")));
        state.update(Message::CardUnhovered(CardKey::Preview("solid")));
        assert_eq!(state.hovered(), None);
    }

    /// **ORACLE**: 隣カードへの enter の後に届く古い exit は hover を消さない
    /// (同一 event 内の enter/exit 順序が木順に依存するための防御 —
    /// `Message::CardUnhovered` doc 参照)。
    #[test]
    fn a_stale_unhover_does_not_clear_the_new_hover() {
        let mut state = PaneState::new();
        state.update(Message::CardHovered(CardKey::Preview("solid")));
        state.update(Message::CardHovered(CardKey::Preview("null")));
        state.update(Message::CardUnhovered(CardKey::Preview("solid")));
        assert_eq!(state.hovered(), Some(CardKey::Preview("null")));
    }

    /// タブ切替で hover も流す(亡霊防止 — selected/recent と同じ)。
    #[test]
    fn selecting_a_tab_clears_the_hover() {
        let mut state = PaneState::new();
        state.update(Message::CardHovered(CardKey::Preview("solid")));
        state.update(Message::SelectTab(LibraryTab::Media));
        assert_eq!(state.hovered(), None);
    }

    // -----------------------------------------------------------------
    // B08 第4切片(素材の整理): SortKey/ViewMode の transient 状態。
    // -----------------------------------------------------------------

    /// **ORACLE**: 初期 sort key は `SortKey::Name`(mock に類例が無い新規
    /// 状態の既定値、`state.rs` の `sort_key` フィールド doc 参照)。
    #[test]
    fn initial_sort_key_is_name() {
        assert_eq!(PaneState::new().sort_key(), SortKey::Name);
    }

    #[test]
    fn select_sort_key_replaces_the_current_sort_key() {
        let mut state = PaneState::new();
        state.update(Message::SelectSortKey(SortKey::Kind));
        assert_eq!(state.sort_key(), SortKey::Kind);
    }

    /// **ORACLE**: 初期表示形式は `ViewMode::Grid`(mock 既定表示
    /// `data-view="grid"` に一致)。
    #[test]
    fn initial_view_mode_is_grid() {
        assert_eq!(PaneState::new().view_mode(), ViewMode::Grid);
    }

    #[test]
    fn select_view_mode_toggles_between_grid_and_list() {
        let mut state = PaneState::new();
        state.update(Message::SelectViewMode(ViewMode::List));
        assert_eq!(state.view_mode(), ViewMode::List);
        state.update(Message::SelectViewMode(ViewMode::Grid));
        assert_eq!(state.view_mode(), ViewMode::Grid);
    }

    /// sort key/view mode はタブ切替で破棄されない(`open` と同じ独立軸 —
    /// `selecting_a_tab_does_not_disturb_the_open_flag` と同型)。
    #[test]
    fn selecting_a_tab_does_not_disturb_sort_key_or_view_mode() {
        let mut state = PaneState::new();
        state.update(Message::SelectSortKey(SortKey::AddedDate));
        state.update(Message::SelectViewMode(ViewMode::List));
        state.update(Message::SelectTab(LibraryTab::Panels));
        assert_eq!(state.sort_key(), SortKey::AddedDate);
        assert_eq!(state.view_mode(), ViewMode::List);
    }

    /// `Clear` は scope/query/preview scope だけを戻す — sort key/view mode
    /// は「フィルタ」ではなく「表示の好み」なので `ClearFilters` の対象外
    /// (mock の `data-clear-filter` にも view mode/sort の記述は無い)。
    #[test]
    fn clear_filters_does_not_reset_sort_key_or_view_mode() {
        let mut state = PaneState::new();
        state.update(Message::SelectSortKey(SortKey::Kind));
        state.update(Message::SelectViewMode(ViewMode::List));
        state.update(Message::ClearFilters);
        assert_eq!(state.sort_key(), SortKey::Kind);
        assert_eq!(state.view_mode(), ViewMode::List);
    }

    // -----------------------------------------------------------------
    // B08 map 616/617: ReplaceSelectedLayerSource は pane-local 状態への
    // no-op(`Intent::SetSource` の dispatch は supervisor の責務、
    // `Message::ReplaceSelectedLayerSource` doc 参照)。
    // -----------------------------------------------------------------

    /// **ORACLE**: この腕は scope/query/tab/selection のどれも動かさない。
    #[test]
    fn replace_selected_layer_source_does_not_touch_pane_local_state() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Audio));
        state.update(Message::QueryChanged("tone".to_owned()));
        state.update(Message::SelectCard(CardKey::Media(AssetId::from_raw(5))));
        state.update(Message::ReplaceSelectedLayerSource(AssetId::from_raw(5)));
        assert_eq!(state.scope(), RailScope::Audio);
        assert_eq!(state.query(), "tone");
        assert_eq!(state.selected(), Some(CardKey::Media(AssetId::from_raw(5))));
    }
}
