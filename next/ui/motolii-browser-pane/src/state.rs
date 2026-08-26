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
use crate::context_menu;
use crate::media_preview::PreviewMedia;
use crate::model::{
    CardKey, CreateKind, LibraryTab, PreviewScope, RailScope, ShapeOpKind, SortKey, ViewMode,
};
use motolii_store::AssetId;

/// OS の Cmd/Ctrl と Shift を pane 境界で意味へ変換した入力。
///
/// `iced` のキーボード型を pane-local state に持ち込まず、view/WIRE は
/// Cmd(macOS) と Ctrl(Windows/Linux) を同じ `toggle` へ正規化する。これで
/// state は OS を知らず、通常クリック・トグル・範囲選択の責任だけを持つ。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardSelectionModifiers {
    shift: bool,
    toggle: bool,
}

impl CardSelectionModifiers {
    /// `shift` は範囲選択、`toggle` は Cmd/Ctrl 選択を表す。
    pub const fn new(shift: bool, toggle: bool) -> Self {
        Self { shift, toggle }
    }

    /// 通常クリック。
    pub const fn plain() -> Self {
        Self::new(false, false)
    }

    /// Cmd(macOS)/Ctrl(Windows/Linux) クリック。
    pub const fn toggle() -> Self {
        Self::new(false, true)
    }

    /// Shift クリック。
    pub const fn range() -> Self {
        Self::new(true, false)
    }

    pub const fn is_shift(self) -> bool {
        self.shift
    }

    pub const fn is_toggle(self) -> bool {
        self.toggle
    }
}

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
    /// カタログの通常カード click。従来の単一選択入口を維持する。
    SelectCard(CardKey),
    /// media カードの右クリック。対象カードを pane-local context menu の
    /// anchor として渡す。Document の選択集合を暗黙に書き換えず、menu の
    /// command が対象 AssetId を明示的に運ぶ。
    OpenContextMenu(CardKey),
    /// media カードのダブルクリックによる素材単体プレビュー要求。
    ///
    /// これは Browser が持つ最後の handoff であり、`AssetId` 以外の素材情報や
    /// player state は持たない。`motolii-shell` の `Message::Browser` 境界から
    /// Source Monitor 相当の実在 owner へ結線するまで、pane-local state は
    /// 変更しない。現行 Stage preview は comp 全体の合成経路なので、ここへ
    /// それを誤接続したり、Browser に再生ボタンを作ったりはしない。
    PreviewMedia(PreviewMedia),
    /// modifier 付きカード click。
    ///
    /// `visible_cards` は現在の表示順(絞り込み後の catalog 順)を view/WIRE
    /// が渡す。Shift の時だけ anchor から target までの範囲に使い、Cmd/Ctrl
    /// toggle と通常 click では無視する。順序を `PaneState` にキャッシュ
    /// しないので、pane は Document/catalog の第二の所有者にならない。
    ///
    /// Cmd(macOS) と Ctrl(Windows/Linux) の差は、発行側が
    /// [`CardSelectionModifiers::toggle`] へ正規化する。
    SelectCardWithModifiers {
        key: CardKey,
        modifiers: CardSelectionModifiers,
        visible_cards: Vec<CardKey>,
    },
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
    /// effects タブの Mask カードのダブルクリック(裁定205 施工第2号 §A —
    /// `model::SelectionAction::AddMask` を宣言するカードから発火)。
    /// **pane-local には状態を動かさない**(`CreateFromCard` と同じ理由) —
    /// 選択中の単一レイヤーへ `Intent::AddMask` を積むのは shell 側
    /// (`Shell::add_mask_to_selected_layer`)の仕事。単一選択でない時にどう
    /// 振る舞うか(拒否/no-op)も shell 側の判断——この Message 自体は
    /// 「マスクを1枚足したい」という意図だけを運ぶ。
    AddMaskFromCard,
    /// effects タブの Glow カードのダブルクリック(裁定205 施工第2号 §B —
    /// `model::SelectionAction::ApplyEffect` を宣言するカードから発火)。
    /// plugin id はカード定義側の `&'static str`(`model::EFFECTS_PREVIEW` の
    /// Glow カード)をそのまま運ぶ — 値の正本はそこ1箇所。**pane-local には
    /// 状態を動かさない**(`AddMaskFromCard` と同型)。
    ApplyEffectFromCard { plugin_id: &'static str },
    /// effects タブの `OpKind` 演算子カードのダブルクリック(2026-08-24
    /// 「ブラウザに8枚の札」発注 — `model::SelectionAction::ApplyOp` を宣言
    /// するカードから発火)。**pane-local には状態を動かさない**
    /// (`AddMaskFromCard`/`ApplyEffectFromCard` と同型) — 選択中の単一
    /// レイヤーの shape へ演算子を1段積んで `Intent::SetShapes` で書き戻すのは
    /// shell 側(`Shell::apply_op_to_selected_layer`)の仕事。どの演算子かは
    /// `model::ShapeOpKind` の tag が運ぶ(具体的な既定パラメータは shell 側が
    /// 組む — `model::ShapeOpKind` doc 参照)。
    ApplyOpFromCard { op: ShapeOpKind },
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
    /// 素材を台帳から外す(`A01-entry.tsv` `RemoveAsset` 行 — store 側は
    /// `Intent::RemoveAsset { asset }` が実装・undo 込みでテスト済み
    /// (`next/core/motolii-store/tests/asset.rs`)、UI 側の入口が皆無だった
    /// 穴を塞ぐ)。
    ///
    /// **この crate は `Intent::RemoveAsset` を呼ばない**
    /// (`ReplaceSelectedLayerSource` doc と同じ「pane は Intent を呼ばない」
    /// 分業、crate 冒頭 doc「shell 結線」参照)— supervisor が
    /// `Intent::RemoveAsset { asset }` を dispatch する。[`PaneState::update`]
    /// はこの腕を no-op として扱う(pane-local な状態は何も変わらない —
    /// 台帳から実際に消える描画結果は Document 側の変化を経由して次の
    /// render で反映される、supervisor が委譲の前後どちらかで副作用を
    /// 差し込む)。
    RemoveAssetFromCard(AssetId),
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
    /// カタログの選択。`selected` は既存 view/API との互換用の focus、
    /// `selected_cards` が実際の pane-local 選択集合、`selection_anchor` が Shift
    /// 範囲の起点を持つ。カードの意味や Document の選択は所有しない。
    selected: Option<CardKey>,
    selected_cards: Vec<CardKey>,
    selection_anchor: Option<CardKey>,
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
    /// media カード context menu の anchor。カード選択集合とは別の一過性
    /// 状態であり、対象 AssetId の引き渡しとメニューの表示位置を1つの
    /// pane-local component に閉じ込める。
    context_menu: context_menu::State,
}

impl PaneState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Presentation の初期配置と pane-local の開閉を同じ状態から組む入口。
    /// `Default` は単体の pane 契約どおり閉じたままにし、Shell が保存済み
    /// layout を復元する時だけこの constructor を使う。これで「木は Browser
    /// 開だが pane-local は閉」という二重状態を起こさない。
    pub fn with_open(open: bool) -> Self {
        Self { open, ..Self::default() }
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

    /// 既存単一選択 API との互換用の focus。複数選択全体は
    /// [`Self::selected_cards`] を読む。
    pub fn selected(&self) -> Option<CardKey> {
        self.selected
    }

    /// 現在のカード選択集合を表示順で返す。
    pub fn selected_cards(&self) -> &[CardKey] {
        &self.selected_cards
    }

    /// カードが現在の選択集合に含まれるか。
    pub fn is_card_selected(&self, key: CardKey) -> bool {
        self.selected_cards.contains(&key)
    }

    /// Shift 範囲選択の起点。view の表示順を再構築する側が必要なら読む。
    pub fn selection_anchor(&self) -> Option<CardKey> {
        self.selection_anchor
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

    /// 現在開いている media-card menu の anchor。view はこれを card component
    /// へ渡すだけで、menu の内部状態や Document を所有しない。
    pub(crate) fn context_menu_anchor(&self) -> Option<CardKey> {
        self.context_menu.anchor()
    }

    /// pane 側の唯一の書き口。
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectScope(scope) => self.scope = scope,
            Message::SelectPreviewScope(scope) => self.preview_scope = scope,
            Message::SelectCard(key) => {
                self.context_menu.close();
                self.select_card(key, CardSelectionModifiers::plain(), &[]);
            }
            Message::OpenContextMenu(key) => {
                self.context_menu.open(key);
            }
            // PreviewMedia は外向きの typed handoff。素材/再生の正本を Browser
            // に複製しないため、pane-local state は変えない。
            Message::PreviewMedia(_) => {}
            Message::SelectCardWithModifiers {
                key,
                modifiers,
                visible_cards,
            } => {
                self.context_menu.close();
                self.select_card(key, modifiers, &visible_cards);
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
                self.clear_card_selection();
                // hover/新規素材ハイライトもタブと一緒に流す(前タブの
                // 一過性状態を亡霊として持ち越さない — selected と同じ理由)。
                self.hovered = None;
                self.recent.clear();
                self.context_menu.close();
            }
            Message::QueryChanged(text) => self.query = text,
            Message::ClearFilters => {
                self.scope = RailScope::AllMedia;
                self.preview_scope = PreviewScope::All;
                self.query.clear();
                self.context_menu.close();
            }
            Message::ToggleBrowserPanel => {
                self.open = !self.open;
                self.context_menu.close();
            }
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
            // 副作用(`Intent::RemoveAsset` の dispatch)は supervisor の責務
            // (このメンバー doc・crate 冒頭 doc「shell 結線」参照) — pane-local
            // 状態には何も書かない(選択状態も保つ — 実際に消えるかは
            // Document 側の変化を経由する、`selected` を先回りで解除しない)。
            Message::RemoveAssetFromCard(_) => self.context_menu.close(),
            // マスク追加/effect 適用も「作る」と同じく pane-local に動かす状態が
            // 無い(`CreateFromCard` と同型 — 選択は `SelectCard` が別途書く)。
            // 実際の `Intent::AddMask`/`Intent::SetEffects` は shell 結線が
            // この2 variant を `Shell::update` 側で横取りして落とす
            // (裁定205 施工第2号 §A/§B)。
            Message::AddMaskFromCard => {}
            Message::ApplyEffectFromCard { .. } => {}
            // 演算子適用も「マスク追加/effect 適用」と同型 — pane-local に
            // 動かす状態が無い。実際の `Intent::SetShapes` 書き戻しは shell
            // 結線がこの variant を `Shell::update` 側で横取りして落とす。
            Message::ApplyOpFromCard { .. } => {}
        }
    }

    /// modifier-aware なカード選択の唯一の state 書き口。
    fn select_card(
        &mut self,
        key: CardKey,
        modifiers: CardSelectionModifiers,
        visible_cards: &[CardKey],
    ) {
        let (selected_cards, selection_anchor) = resolve_card_selection(
            &self.selected_cards,
            self.selection_anchor,
            key,
            modifiers,
            visible_cards,
        );
        self.selected_cards = selected_cards;
        self.selection_anchor = selection_anchor;
        self.selected = if self.selected_cards.contains(&key) {
            Some(key)
        } else {
            self.selected_cards.last().copied()
        };
        // 新規素材ハイライトは「直後」の一過性 — カードへ触った時点で
        // 消灯する(触った=気づいた、以降は通常の選択文法へ)。
        self.recent.clear();
    }

    fn clear_card_selection(&mut self) {
        self.selected = None;
        self.selected_cards.clear();
        self.selection_anchor = None;
    }
}

/// カード選択の意味だけを計算する pure helper。
///
/// Shift は anchor と target が現在の表示順に存在する時だけ inclusive range
/// を作る。フィルタやタブ切替で anchor が見えなくなった場合は、範囲を推測せず
/// target の通常選択へフォールバックする。Shift+Cmd/Ctrl は Shift を優先する
/// (範囲選択の意味を一つにして、OS ごとの差を増やさない)。
fn resolve_card_selection(
    current: &[CardKey],
    anchor: Option<CardKey>,
    target: CardKey,
    modifiers: CardSelectionModifiers,
    visible_cards: &[CardKey],
) -> (Vec<CardKey>, Option<CardKey>) {
    if modifiers.is_shift() {
        if let Some(anchor) = anchor {
            let anchor_index = visible_cards.iter().position(|key| *key == anchor);
            let target_index = visible_cards.iter().position(|key| *key == target);
            if let (Some(anchor_index), Some(target_index)) = (anchor_index, target_index) {
                let start = anchor_index.min(target_index);
                let end = anchor_index.max(target_index);
                return (
                    visible_cards[start..=end].iter().copied().fold(
                        Vec::new(),
                        |mut selected, key| {
                            if !selected.contains(&key) {
                                selected.push(key);
                            }
                            selected
                        },
                    ),
                    Some(anchor),
                );
            }
        }
        return (vec![target], Some(target));
    }

    if modifiers.is_toggle() {
        let mut selected = current.to_vec();
        if let Some(index) = selected.iter().position(|key| *key == target) {
            selected.remove(index);
            let next_anchor = if anchor == Some(target) {
                selected.last().copied()
            } else {
                anchor.filter(|key| selected.contains(key))
            };
            return (selected, next_anchor);
        }
        selected.push(target);
        return (selected, anchor.or(Some(target)));
    }

    (vec![target], Some(target))
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
    fn presentation_can_seed_the_panel_open_without_changing_default() {
        assert!(PaneState::with_open(true).is_open());
        assert!(!PaneState::with_open(false).is_open());
        assert!(!PaneState::new().is_open());
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
        assert_eq!(
            state.selected_cards(),
            [CardKey::Media(AssetId::from_raw(3))]
        );
        state.update(Message::SelectCard(CardKey::Preview("glow")));
        assert_eq!(state.selected(), Some(CardKey::Preview("glow")));
        assert_eq!(state.selected_cards(), [CardKey::Preview("glow")]);
        assert_eq!(state.selection_anchor(), Some(CardKey::Preview("glow")));
    }

    #[test]
    fn toggle_modifier_adds_and_removes_cards_without_replacing_the_selection() {
        let first = CardKey::Media(AssetId::from_raw(1));
        let second = CardKey::Media(AssetId::from_raw(2));
        let mut state = PaneState::new();

        state.update(Message::SelectCard(first));
        state.update(Message::SelectCardWithModifiers {
            key: second,
            modifiers: CardSelectionModifiers::toggle(),
            visible_cards: vec![first, second],
        });
        assert_eq!(state.selected_cards(), [first, second]);
        assert_eq!(state.selection_anchor(), Some(first));

        state.update(Message::SelectCardWithModifiers {
            key: second,
            modifiers: CardSelectionModifiers::toggle(),
            visible_cards: vec![first, second],
        });
        assert_eq!(state.selected_cards(), [first]);
        assert_eq!(state.selected(), Some(first));
        assert_eq!(state.selection_anchor(), Some(first));
    }

    #[test]
    fn shift_modifier_selects_the_inclusive_range_from_the_anchor() {
        let cards = [
            CardKey::Media(AssetId::from_raw(1)),
            CardKey::Media(AssetId::from_raw(2)),
            CardKey::Media(AssetId::from_raw(3)),
            CardKey::Media(AssetId::from_raw(4)),
        ];
        let mut state = PaneState::new();

        state.update(Message::SelectCard(cards[1]));
        state.update(Message::SelectCardWithModifiers {
            key: cards[3],
            modifiers: CardSelectionModifiers::range(),
            visible_cards: cards.to_vec(),
        });

        assert_eq!(state.selected_cards(), [cards[1], cards[2], cards[3]]);
        assert_eq!(state.selected(), Some(cards[3]));
        assert_eq!(state.selection_anchor(), Some(cards[1]));
    }

    #[test]
    fn shift_range_follows_reverse_display_order() {
        let cards = [
            CardKey::Media(AssetId::from_raw(1)),
            CardKey::Media(AssetId::from_raw(2)),
            CardKey::Media(AssetId::from_raw(3)),
            CardKey::Media(AssetId::from_raw(4)),
        ];
        let mut state = PaneState::new();

        state.update(Message::SelectCard(cards[3]));
        state.update(Message::SelectCardWithModifiers {
            key: cards[1],
            modifiers: CardSelectionModifiers::range(),
            visible_cards: cards.to_vec(),
        });

        // 選択集合は表示順で返し、anchor は最初の plain click のまま。
        assert_eq!(state.selected_cards(), [cards[1], cards[2], cards[3]]);
        assert_eq!(state.selection_anchor(), Some(cards[3]));
    }

    #[test]
    fn shift_without_a_visible_anchor_falls_back_to_plain_selection() {
        let anchor = CardKey::Media(AssetId::from_raw(1));
        let target = CardKey::Media(AssetId::from_raw(3));
        let mut state = PaneState::new();

        state.update(Message::SelectCard(anchor));
        state.update(Message::SelectCardWithModifiers {
            key: target,
            modifiers: CardSelectionModifiers::range(),
            visible_cards: vec![target],
        });

        assert_eq!(state.selected_cards(), [target]);
        assert_eq!(state.selection_anchor(), Some(target));
    }

    #[test]
    fn selecting_a_tab_clears_the_multi_selection_and_anchor() {
        let first = CardKey::Media(AssetId::from_raw(1));
        let second = CardKey::Media(AssetId::from_raw(2));
        let mut state = PaneState::new();
        state.update(Message::SelectCard(first));
        state.update(Message::SelectCardWithModifiers {
            key: second,
            modifiers: CardSelectionModifiers::toggle(),
            visible_cards: vec![first, second],
        });

        state.update(Message::SelectTab(LibraryTab::Effects));

        assert!(state.selected_cards().is_empty());
        assert_eq!(state.selection_anchor(), None);
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

    // -----------------------------------------------------------------
    // A01-entry RemoveAsset: RemoveAssetFromCard は pane-local 状態への
    // no-op(`Intent::RemoveAsset` の dispatch は supervisor の責務、
    // `Message::RemoveAssetFromCard` doc 参照)。
    // -----------------------------------------------------------------

    /// **ORACLE**(検収条件そのもの、裁定218): この腕は scope/query/selection
    /// のどれも動かさない — publish 自体は view 側の責務なので、ここで押さえる
    /// のは「受け取った `PaneState::update` が no-op であること」の1点のみ。
    #[test]
    fn remove_asset_from_card_does_not_touch_pane_local_state() {
        let mut state = PaneState::new();
        state.update(Message::SelectScope(RailScope::Audio));
        state.update(Message::QueryChanged("tone".to_owned()));
        state.update(Message::SelectCard(CardKey::Media(AssetId::from_raw(5))));
        state.update(Message::RemoveAssetFromCard(AssetId::from_raw(5)));
        assert_eq!(state.scope(), RailScope::Audio);
        assert_eq!(state.query(), "tone");
        assert_eq!(state.selected(), Some(CardKey::Media(AssetId::from_raw(5))));
    }

    #[test]
    fn context_menu_anchor_is_separate_from_card_selection_and_closes_on_select() {
        let first = CardKey::Media(AssetId::from_raw(1));
        let second = CardKey::Media(AssetId::from_raw(2));
        let target = CardKey::Media(AssetId::from_raw(9));
        let mut state = PaneState::new();

        state.update(Message::SelectCard(first));
        state.update(Message::SelectCardWithModifiers {
            key: second,
            modifiers: CardSelectionModifiers::toggle(),
            visible_cards: vec![first, second, target],
        });
        state.update(Message::OpenContextMenu(target));

        assert_eq!(state.context_menu_anchor(), Some(target));
        assert_eq!(state.selected_cards(), [first, second]);

        state.update(Message::SelectCard(target));
        assert_eq!(state.context_menu_anchor(), None);
        assert_eq!(state.selected_cards(), [target]);
    }

    #[test]
    fn preview_card_cannot_open_a_media_context_menu() {
        let mut state = PaneState::new();

        state.update(Message::OpenContextMenu(CardKey::Preview("glow")));

        assert_eq!(state.context_menu_anchor(), None);
    }
}
