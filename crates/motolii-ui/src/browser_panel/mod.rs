//! Browser native pane(egui)。
//!
//! UX の正本は `docs/mocks-ui/public/browser-library.html`(+ `.css`)。
//! 構造(header / toolbar / tabs / sidebar / catalog / selection tray)と
//! 色・寸法はそこから写し、出典は `theme.rs` が `// browser-library.css:<line>` で持つ。
//!
//! データの owner は既存 `crate::media_library`(走査・種別判定・path解決)。
//! ここでは走査を**再実装しない**。サムネイルも既存 `browser_blitz::thumbnail` の
//! 縮小実体をそのまま egui texture にする(1回 upload、毎フレーム再合成しない —
//! 旧 Blitz Browser 面の重さ(DOM 全体を毎フレーム scene へ起こす)を持ち込まない)。
//!
//! モックの COLLECTIONS(favorite 等)と Effects / Create / Panels の項目は
//! preview-local データで model が無いため、**項目を発明せず**空で出す。
//! drag&drop / tag 編集は後続レーン。
//!
//! **配置は要求として外へ出す。** カードをダブルクリックすると
//! [`BrowserRequest::PlaceFile`] が返る — このパネルは Document を1バイトも書かない。
//! 取り込むか・どこへ置くか・失敗を何と言うかを決めるのは shell 側で、経路は
//! OS ドロップ(`blitz_shell::admit_dropped_paths`)と同じ1本である。

mod theme;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::media_library::{LibraryItem, LibraryProjection, MediaLibrary};

/// pane 既定のフォルダ。`blitz_shell/pane.rs` の従来 Browser と同じ。
pub const DEFAULT_BROWSER_DIR: &str = "docs/mocks";

/// browser-library.html:31-34 の tab(`data-tab`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserTab {
    Media,
    Effects,
    Create,
    Panels,
}

impl BrowserTab {
    fn label(self) -> &'static str {
        match self {
            Self::Media => "Media",
            Self::Effects => "Effects",
            Self::Create => "Create",
            Self::Panels => "Panels",
        }
    }

    fn all_label(self) -> &'static str {
        // browser-library.html:222-223 の titles / tabTitles。
        match self {
            Self::Media => "All media",
            Self::Effects => "All effects",
            Self::Create => "All create",
            Self::Panels => "All panels",
        }
    }
}

/// browser-library.html:95-97 の view(`data-view`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserView {
    Thumbnails,
    Grid,
    List,
}

/// card 1枚ぶんの表示投影。`media_library::LibraryItem` の写しで、独自の意味を持たない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCardModel {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    /// card の2行目(browser-library.html:123 `video · B-roll` の席)。
    pub meta: String,
    pub selected: bool,
}

/// Browser が shell へ出す**型付きの要求**。
///
/// パネルは自分で import しない。カードが指しているのは project の asset ではなく
/// **フォルダの中の実ファイル**で、それを Document へ入れてよいか・どこへ置くか・
/// 入らなかった時に何と言うかは shell の判断である(Document の writer は
/// エディタの中に1つだけ = single writer)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRequest {
    /// カードのダブルクリック = 「この実ファイルを置いてくれ」。
    ///
    /// 意味は OS ドロップ1本ぶんと同じで、shell は
    /// `blitz_shell::admit_dropped_paths` と同じ経路へ流す(2つ目の import 経路を作らない)。
    PlaceFile(PathBuf),
}

/// Browser native pane 1面。
pub struct BrowserPanel {
    library: MediaLibrary,
    /// 走査結果。構築時に1回だけ scan する(毎フレーム fs を触らない)。
    projection: LibraryProjection,
    /// image kind の縮小実体(既存 thumbnail cache の path)。id → path。
    thumbnails: BTreeMap<String, PathBuf>,
    /// upload 済み texture。`None` は読めなかった項目(再試行しない)。
    textures: HashMap<String, Option<egui::TextureHandle>>,
    tab: BrowserTab,
    view: BrowserView,
    /// sidebar PLACES の directory filter(None = All)。
    directory: Option<String>,
    /// sidebar LIBRARY の kind filter(video / image / audio)。
    kind: Option<&'static str>,
    /// filter shelf の tag filter。
    tag: Option<String>,
    query: String,
    selected: Option<String>,
    /// filter shelf の開閉(browser-library.html:437-441 `#filter-toggle`)。
    shelf_open: bool,
    /// 画像を出せなかった理由の控え。**再試行の方針は変えない**(`None` を
    /// 覚えたまま作り直さない)が、なぜ glyph card なのかは1度は外へ出す
    /// (2026-08-18 外部診断 F-09)。
    ///
    /// **shell の型はここに来ない。** panel は理由を返すだけで、言うのは
    /// `blitz_shell::pane`(そこで `ShellTranscript` へ写る)。
    notices: Vec<String>,
}

impl BrowserPanel {
    /// pane 既定(`docs/mocks`)。
    pub fn default_shell() -> Self {
        Self::with_root(PathBuf::from(DEFAULT_BROWSER_DIR))
    }

    pub fn with_root(root: PathBuf) -> Self {
        let library = MediaLibrary::with_root(root);
        let projection = library.project();
        // 縮小実体は image kind だけ(動画・音声の実体生成は後続。mock も glyph card)。
        let image_items: Vec<_> = projection
            .items
            .iter()
            .filter(|item| item.kind == "image")
            .filter_map(|item| library.resolve(&item.id))
            .collect();
        // 作れなかった分は理由つきで控えへ。card は従来どおり glyph で出る。
        let mut notices = Vec::new();
        let thumbnails = crate::browser_blitz::thumbnail::prepare(&image_items)
            .into_iter()
            .zip(&image_items)
            .filter_map(|(prepared, item)| match prepared {
                Ok(path) => Some((item.id.clone(), path)),
                Err(reason) => {
                    notices.push(format!(
                        "browser-panel: {} の縮小実体を作れないので glyph card で描く: {reason}",
                        item.path.display()
                    ));
                    None
                }
            })
            .collect();
        Self {
            library,
            projection,
            thumbnails,
            textures: HashMap::new(),
            tab: BrowserTab::Media,
            view: BrowserView::Grid,
            directory: None,
            kind: None,
            tag: None,
            query: String::new(),
            selected: None,
            shelf_open: true,
            notices,
        }
    }

    /// 出せなかった画像の理由を引き取る(引き取ったら空になる)。
    ///
    /// 呼び手は shell 側で、`--status-log` に残る一言へ写す。同じ理由が
    /// 毎フレーム流れないのは、`None` cache のおかげで**1回しか積まれない**から。
    pub fn take_notices(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notices)
    }

    // ---- model 面(toolkit 非依存) ----

    pub fn tab(&self) -> BrowserTab {
        self.tab
    }

    pub fn set_tab(&mut self, tab: BrowserTab) {
        self.tab = tab;
    }

    pub fn view(&self) -> BrowserView {
        self.view
    }

    pub fn set_view(&mut self, view: BrowserView) {
        self.view = view;
    }

    pub fn set_query(&mut self, query: &str) {
        self.query = query.trim().to_lowercase();
    }

    pub fn set_kind_filter(&mut self, kind: Option<&str>) {
        self.kind = match kind {
            Some("video") => Some("video"),
            Some("image") => Some("image"),
            Some("audio") => Some("audio"),
            _ => None,
        };
    }

    pub fn set_tag_filter(&mut self, tag: Option<String>) {
        self.tag = tag;
    }

    pub fn set_directory_filter(&mut self, directory: Option<String>) {
        self.directory = directory;
    }

    pub fn select(&mut self, id: &str) {
        self.selected = Some(id.to_owned());
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// filter 済みの card 一覧(browser-library.html:288-297 の render と同じ意味)。
    pub fn visible_cards(&self) -> Vec<BrowserCardModel> {
        if self.tab != BrowserTab::Media {
            // Media 以外の catalog はまだ model が無い(項目を発明しない)。
            return Vec::new();
        }
        self.projection
            .items
            .iter()
            .filter(|item| self.item_visible(item))
            .map(|item| BrowserCardModel {
                id: item.id.clone(),
                name: item.name.clone(),
                kind: item.kind,
                meta: card_meta(item),
                selected: self.selected.as_deref() == Some(item.id.as_str()),
            })
            .collect()
    }

    pub fn result_count(&self) -> usize {
        self.visible_cards().len()
    }

    /// card 1枚が指している実ファイルの配置要求。
    ///
    /// path の解決は既存 `MediaLibrary::resolve` のままで、ここで走査し直さない。
    /// 知らない id(filter で消えた後の古い選択など)は `None` — 存在しない path を
    /// 作って shell に投げない。
    pub fn place_request(&self, id: &str) -> Option<BrowserRequest> {
        let resolved = self.library.resolve(id)?;
        Some(BrowserRequest::PlaceFile(resolved.path))
    }

    /// 縮小実体の path(image kind のみ)。元画像 path は返さない。
    pub fn thumbnail_path(&self, id: &str) -> Option<&Path> {
        self.thumbnails.get(id).map(PathBuf::as_path)
    }

    fn item_visible(&self, item: &LibraryItem) -> bool {
        let kind_matches = self.kind.is_none_or(|kind| item.kind == kind);
        let directory_matches = self
            .directory
            .as_deref()
            .is_none_or(|directory| item.directory == directory);
        let tag_matches = self
            .tag
            .as_deref()
            .is_none_or(|tag| item.tags.iter().any(|t| t == tag));
        // browser-library.html:290 haystack = search 語彙 + tags。
        let haystack = format!(
            "{} {} {}",
            item.name.to_lowercase(),
            item.tags.join(" ").to_lowercase(),
            item.directory.to_lowercase()
        );
        kind_matches && directory_matches && tag_matches && haystack.contains(&self.query)
    }

    fn selection_item(&self) -> Option<&LibraryItem> {
        let selected = self.selected.as_deref()?;
        self.projection.items.iter().find(|item| item.id == selected)
    }

    // ---- 描画 ----

    /// pane いっぱいに描き、**このフレームで人が出した要求**を返す。
    ///
    /// 返るのは今のところカードのダブルクリック 1 種だけ(`BrowserRequest::PlaceFile`)。
    /// 選択・filter・tab の切り替えはパネルの中で閉じるので何も返さない。
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> Option<BrowserRequest> {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0.0, theme::PANEL_BG);

        let mut cursor = rect.top();
        cursor = self.draw_header(&painter, rect, cursor);
        cursor = self.draw_toolbar(ui, &painter, rect, cursor);
        cursor = self.draw_tabs(ui, &painter, rect, cursor);

        // browser-library.css:99 `.browserWorkspace { display: flex }`(sidebar + catalog)
        let workspace = Rect::from_min_max(Pos2::new(rect.left(), cursor), rect.max);
        // css:348-349 `@media (max-width: 420px)`: 狭い pane では sidebar を 92px へ
        // 譲らせて catalog 側に席を返す。ここを守らないと catalog header の見出しが
        // 先に潰れる。
        let sidebar = Rect::from_min_size(
            workspace.min,
            Vec2::new(sidebar_width(rect.width()), workspace.height()),
        );
        let catalog = Rect::from_min_max(
            Pos2::new(sidebar.right(), workspace.top()),
            workspace.max,
        );
        self.draw_sidebar(ui, sidebar);
        // css:348 の `@media (max-width: 420px)` は文書幅 = pane 幅で効く。
        self.draw_catalog(ui, catalog, rect.width())
    }

    /// browser-library.css:28-38 `.browserHeader`。
    fn draw_header(&self, painter: &egui::Painter, rect: Rect, top: f32) -> f32 {
        let header = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::HEADER_H),
        );
        painter.hline(header.x_range(), header.bottom(), Stroke::new(1.0, theme::BORDER));
        painter.text(
            Pos2::new(header.left() + theme::HEADER_PAD_X, header.center().y),
            Align2::LEFT_CENTER,
            "Browser",
            FontId::proportional(theme::FS_HEADER),
            theme::TEXT,
        );
        painter.text(
            Pos2::new(header.right() - theme::HEADER_PAD_X, header.center().y),
            Align2::RIGHT_CENTER,
            "LOCAL LIBRARY",
            FontId::proportional(theme::FS_HEADER_SPAN),
            theme::HEADER_SPAN,
        );
        header.bottom()
    }

    /// browser-library.css:40-77 `.browserToolbar`(‹ › / 検索 / Filters)。
    fn draw_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
    ) -> f32 {
        let toolbar = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::TOOLBAR_H),
        );
        painter.hline(
            toolbar.x_range(),
            toolbar.bottom(),
            Stroke::new(1.0, theme::TOOLBAR_BORDER),
        );
        let button_top = toolbar.center().y - theme::BUTTON_H / 2.0;
        let mut x = toolbar.left() + theme::TOOLBAR_PAD_X;
        // css:60-61 history buttons は disabled(履歴 route は後続)。opacity .4。
        for glyph in ["‹", "›"] {
            let button = Rect::from_min_size(
                Pos2::new(x, button_top),
                Vec2::new(theme::HISTORY_W, theme::BUTTON_H),
            );
            painter.rect_filled(button, 0.0, theme::BUTTON_BG.gamma_multiply(0.4));
            painter.rect_stroke(
                button,
                0.0,
                Stroke::new(1.0, theme::BUTTON_BORDER.gamma_multiply(0.4)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                button.center(),
                Align2::CENTER_CENTER,
                glyph,
                FontId::proportional(14.0),
                theme::HISTORY_FG.gamma_multiply(0.4),
            );
            x = button.right() + theme::TOOLBAR_GAP;
        }
        // Filters toggle(css:62 toolbarButton、右端)。
        let filters_w = 34.0;
        let filters = Rect::from_min_size(
            Pos2::new(toolbar.right() - theme::TOOLBAR_PAD_X - filters_w, button_top),
            Vec2::new(filters_w, theme::BUTTON_H),
        );
        let filters_response = ui.interact(filters, ui.id().with("filters"), Sense::click());
        if filters_response.clicked() {
            self.shelf_open = !self.shelf_open;
        }
        let (chip_bg, chip_border, chip_fg) = if self.shelf_open {
            // 開いている間は selected chip と同じ強調(css:201 と同じ語彙)。
            (theme::CHIP_SELECTED_BG, theme::ACCENT, theme::CHIP_SELECTED_FG)
        } else {
            (theme::BUTTON_BG, theme::BUTTON_BORDER, theme::BUTTON_FG)
        };
        painter.rect_filled(filters, 0.0, chip_bg);
        painter.rect_stroke(
            filters,
            0.0,
            Stroke::new(1.0, chip_border),
            egui::StrokeKind::Inside,
        );
        painter.text(
            filters.center(),
            Align2::CENTER_CENTER,
            "Filters",
            FontId::proportional(theme::FS_TOOLBAR),
            chip_fg,
        );
        // 検索(css:64-77)。残り幅いっぱい。
        let search = Rect::from_min_max(
            Pos2::new(x, button_top),
            Pos2::new(filters.left() - theme::TOOLBAR_GAP, button_top + theme::BUTTON_H),
        );
        painter.rect_filled(search, 0.0, theme::SEARCH_BG);
        let mut query = self.query.clone();
        let mut search_ui = ui.new_child(egui::UiBuilder::new().max_rect(search.shrink2(Vec2::new(6.0, 0.0))));
        search_ui.style_mut().visuals.extreme_bg_color = Color32::TRANSPARENT;
        search_ui.style_mut().visuals.override_text_color = Some(theme::SEARCH_FG);
        let edit = search_ui.add_sized(
            search_ui.available_size(),
            egui::TextEdit::singleline(&mut query)
                .frame(egui::Frame::NONE)
                .font(FontId::proportional(theme::FS_SEARCH))
                .hint_text(
                    egui::RichText::new("Search files and tags")
                        .color(theme::SEARCH_PLACEHOLDER)
                        .size(theme::FS_SEARCH),
                ),
        );
        if edit.changed() {
            self.set_query(&query);
        }
        // css:77 focus 時は accent 枠。
        let search_border = if edit.has_focus() {
            theme::ACCENT
        } else {
            theme::SEARCH_BORDER
        };
        painter.rect_stroke(
            search,
            0.0,
            Stroke::new(1.0, search_border),
            egui::StrokeKind::Inside,
        );
        toolbar.bottom()
    }

    /// browser-library.css:79-97 `.libraryTabs`。
    fn draw_tabs(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
    ) -> f32 {
        let tabs = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::TABS_H),
        );
        painter.hline(tabs.x_range(), tabs.bottom(), Stroke::new(1.0, theme::BORDER));
        let tab_list = [
            BrowserTab::Media,
            BrowserTab::Effects,
            BrowserTab::Create,
            BrowserTab::Panels,
        ];
        let width = tabs.width() / tab_list.len() as f32;
        for (index, tab) in tab_list.into_iter().enumerate() {
            let cell = Rect::from_min_size(
                Pos2::new(tabs.left() + width * index as f32, tabs.top()),
                Vec2::new(width, theme::TABS_H),
            );
            let response = ui.interact(cell, ui.id().with(("tab", index)), Sense::click());
            let selected = self.tab == tab;
            if selected {
                painter.rect_filled(cell, 0.0, theme::TAB_SELECTED_BG);
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(cell.left(), cell.bottom() - 2.0),
                        cell.max,
                    ),
                    0.0,
                    theme::ACCENT,
                );
            }
            painter.text(
                cell.center(),
                Align2::CENTER_CENTER,
                tab.label(),
                FontId::proportional(theme::FS_TAB),
                if selected {
                    theme::TAB_SELECTED_FG
                } else {
                    theme::TAB_FG
                },
            );
            if response.clicked() {
                self.tab = tab;
                // browser-library.html:311-316 chooseTab: media 以外へ移ると filter を戻す。
                if tab != BrowserTab::Media {
                    self.directory = None;
                    self.tag = None;
                    self.kind = None;
                }
            }
        }
        tabs.bottom()
    }

    /// browser-library.css:101-151 `.librarySidebar`。
    fn draw_sidebar(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, 0.0, theme::SIDEBAR_BG);
        painter.vline(rect.right(), rect.y_range(), Stroke::new(1.0, theme::BORDER));

        // 行動作をまとめる小さな描き手。
        struct Row<'a> {
            label: String,
            indent: bool,
            selected: bool,
            disabled: bool,
            action: Option<SidebarAction<'a>>,
        }
        enum SidebarAction<'a> {
            All,
            Kind(&'static str),
            Directory(&'a str),
        }

        let mut rows: Vec<Row> = Vec::new();
        let mut headers: Vec<(usize, &str)> = Vec::new();
        match self.tab {
            BrowserTab::Media => {
                // browser-library.html:45-49 LIBRARY(All media / Video / Images / Audio)。
                headers.push((rows.len(), "LIBRARY"));
                let none_active =
                    self.kind.is_none() && self.directory.is_none() && self.tag.is_none();
                rows.push(Row {
                    label: "▦  All media".to_owned(),
                    indent: false,
                    selected: none_active,
                    disabled: false,
                    action: Some(SidebarAction::All),
                });
                for (glyph, label, kind) in [
                    ("▣", "Video", "video"),
                    ("▧", "Images", "image"),
                    ("♪", "Audio", "audio"),
                ] {
                    rows.push(Row {
                        label: format!("{glyph}  {label}"),
                        indent: false,
                        selected: self.kind == Some(kind),
                        disabled: false,
                        action: Some(SidebarAction::Kind(kind)),
                    });
                }
                // browser-library.html:51-55 PLACES(実フォルダ)。
                headers.push((rows.len(), "PLACES"));
                for directory in &self.projection.directories {
                    let is_root = directory.path.is_empty();
                    rows.push(Row {
                        label: format!(
                            "{}  {}",
                            if is_root { "▾" } else { "├" },
                            directory.name
                        ),
                        indent: !is_root,
                        selected: self.directory.as_deref() == Some(directory.path.as_str())
                            && !is_root
                            || (is_root && self.directory.as_deref() == Some("")),
                        disabled: false,
                        action: Some(SidebarAction::Directory(&directory.path)),
                    });
                }
                rows.push(Row {
                    label: "＋  Add folder".to_owned(),
                    indent: false,
                    selected: false,
                    disabled: true, // html:55 disabled(directory access は後続)
                    action: None,
                });
            }
            other => {
                // browser-library.html:58-85 の各 tab。項目 model が無いので All 行だけ。
                headers.push((0, other.label()));
                rows.push(Row {
                    label: format!("▦  {}", other.all_label()),
                    indent: false,
                    selected: true,
                    disabled: false,
                    action: Some(SidebarAction::All),
                });
            }
        }

        let mut y = rect.top() + 3.0; // css:105 padding-top 3px
        let mut header_iter = headers.iter().peekable();
        let mut pending: Option<(SidebarAction, ())> = None;
        for (index, row) in rows.iter().enumerate() {
            if let Some((at, title)) = header_iter.peek() {
                if *at == index {
                    let h2 = Rect::from_min_size(
                        Pos2::new(rect.left(), y),
                        Vec2::new(rect.width(), theme::SIDEBAR_H2_H),
                    );
                    painter.text(
                        Pos2::new(h2.left() + theme::ROW_PAD_X, h2.bottom() - 4.0),
                        Align2::LEFT_BOTTOM,
                        *title,
                        FontId::proportional(theme::FS_SIDEBAR_H2),
                        theme::SIDEBAR_H2_FG,
                    );
                    y = h2.bottom();
                    header_iter.next();
                }
            }
            let row_rect = Rect::from_min_size(
                Pos2::new(rect.left(), y),
                Vec2::new(rect.width(), theme::ROW_H),
            );
            y = row_rect.bottom();
            let response = ui.interact(
                row_rect,
                ui.id().with(("sidebar-row", index)),
                if row.disabled {
                    Sense::hover()
                } else {
                    Sense::click()
                },
            );
            if row.selected {
                painter.rect_filled(row_rect, 0.0, theme::ROW_SELECTED_BG);
                painter.rect_filled(
                    Rect::from_min_size(row_rect.min, Vec2::new(2.0, row_rect.height())),
                    0.0,
                    theme::ACCENT,
                );
            } else if response.hovered() && !row.disabled {
                painter.rect_filled(row_rect, 0.0, theme::ROW_HOVER_BG);
            }
            let fg = if row.disabled {
                theme::ROW_DISABLED_FG.gamma_multiply(0.45) // css:147 opacity .45
            } else if row.selected {
                theme::ROW_SELECTED_FG
            } else {
                theme::ROW_FG
            };
            let indent = if row.indent {
                theme::ROW_INDENT
            } else {
                theme::ROW_PAD_X
            };
            painter.text(
                Pos2::new(row_rect.left() + indent, row_rect.center().y),
                Align2::LEFT_CENTER,
                &row.label,
                FontId::proportional(theme::FS_ROW),
                fg,
            );
            if response.clicked() {
                if let Some(action) = &row.action {
                    pending = Some((
                        match action {
                            SidebarAction::All => SidebarAction::All,
                            SidebarAction::Kind(kind) => SidebarAction::Kind(kind),
                            SidebarAction::Directory(path) => SidebarAction::Directory(path),
                        },
                        (),
                    ));
                }
            }
        }
        // browser-library.html:334-338: source/kind を選ぶと tag filter を戻す。
        if let Some((action, ())) = pending {
            match action {
                SidebarAction::All => {
                    self.kind = None;
                    self.directory = None;
                    self.tag = None;
                }
                SidebarAction::Kind(kind) => {
                    self.kind = Some(kind);
                    self.directory = None;
                    self.tag = None;
                }
                SidebarAction::Directory(path) => {
                    self.directory = Some(path.to_owned());
                    self.kind = None;
                    self.tag = None;
                }
            }
        }
    }

    /// catalog 列(header / shelf / summary / grid / tray)。
    /// 要求が出るのは grid(カード)だけなので、返り値もそこから来る。
    fn draw_catalog(
        &mut self,
        ui: &mut egui::Ui,
        rect: Rect,
        panel_w: f32,
    ) -> Option<BrowserRequest> {
        let painter = ui.painter().with_clip_rect(rect);
        let mut cursor = rect.top();
        cursor = self.draw_catalog_header(ui, &painter, rect, cursor, panel_w);
        if self.shelf_open {
            cursor = self.draw_filter_shelf(ui, &painter, rect, cursor);
        }
        cursor = self.draw_result_summary(&painter, rect, cursor);
        let tray_top = rect.bottom() - theme::TRAY_H;
        let grid = Rect::from_min_max(
            Pos2::new(rect.left(), cursor),
            Pos2::new(rect.right(), tray_top),
        );
        let request = self.draw_grid(ui, grid);
        self.draw_selection_tray(&painter, rect, tray_top);
        request
    }

    /// browser-library.css:155-168 `.catalogHeader` + view modes。
    ///
    /// css:158 が `display: flex`、css:166 が `.viewModes { margin-left: auto }` なので、
    /// **見出しと view-mode ボタンは重ならない** — ボタンは右へ寄り、残りが見出しの席になる。
    /// egui の painter は勝手に席を割らないので、ここで先にボタンぶんの幅を引いてから
    /// 見出しと path を置き、はみ出しは `…` で畳む。
    fn draw_catalog_header(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
        panel_w: f32,
    ) -> f32 {
        let header = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::CATALOG_HEADER_H),
        );
        let layout = catalog_header_layout(header.width(), panel_w);
        painter.hline(
            header.x_range(),
            header.bottom(),
            Stroke::new(1.0, theme::CATALOG_HEADER_BORDER),
        );
        // browser-library.html:222 titles: source を選んでいれば folder 名、なければ tab 名。
        let (title, path) = match (&self.directory, self.tab) {
            (Some(directory), BrowserTab::Media) => {
                let name = self
                    .projection
                    .directories
                    .iter()
                    .find(|entry| entry.path == *directory)
                    .map(|entry| entry.name.clone())
                    .unwrap_or_else(|| directory.clone());
                let root = self
                    .projection
                    .root
                    .as_ref()
                    .map(|root| root.path.display().to_string())
                    .unwrap_or_default();
                (name, format!("{root}/{directory}"))
            }
            _ => (
                self.tab.all_label().to_owned(),
                "Library · local files".to_owned(),
            ),
        };
        // css:164 strong / css:165 span。span は css:165 の `text-overflow: ellipsis` を
        // そのまま持つ。strong に css の規定は無いが、この header は高さ 31px の1行席で
        // 折返せないので egui 慣行の `…` で畳む(切り落としにしない)。
        let title_galley = elided(
            painter,
            title,
            FontId::proportional(theme::FS_CATALOG_TITLE),
            theme::CATALOG_TITLE_FG,
            layout.text_w,
        );
        painter.galley(
            Pos2::new(
                header.left() + theme::CATALOG_HEADER_PAD_X,
                header.center().y - 6.0 - title_galley.size().y / 2.0,
            ),
            title_galley,
            theme::CATALOG_TITLE_FG,
        );
        let path_galley = elided(
            painter,
            path,
            FontId::proportional(theme::FS_CATALOG_PATH),
            theme::CATALOG_PATH_FG,
            layout.path_max_w,
        );
        painter.galley(
            Pos2::new(
                header.left() + theme::CATALOG_HEADER_PAD_X,
                header.center().y + 6.0 - path_galley.size().y / 2.0,
            ),
            path_galley,
            theme::CATALOG_PATH_FG,
        );
        // view modes(css:166-168)。glyph は html:95-97 の ▦ ▤ ☷。
        let mut x = header.left() + layout.modes_left + theme::VIEW_MODES_W;
        for (index, (glyph, view)) in [
            ("☷", BrowserView::List),
            ("▤", BrowserView::Grid),
            ("▦", BrowserView::Thumbnails),
        ]
        .into_iter()
        .enumerate()
        {
            let button = Rect::from_min_size(
                Pos2::new(
                    x - theme::VIEW_BUTTON_W,
                    header.center().y - theme::BUTTON_H / 2.0,
                ),
                Vec2::new(theme::VIEW_BUTTON_W, theme::BUTTON_H),
            );
            x = button.left() - theme::VIEW_BUTTON_GAP; // css:166 gap 2px
            let response = ui.interact(button, ui.id().with(("view", index)), Sense::click());
            if response.clicked() {
                self.view = view;
            }
            let pressed = self.view == view;
            let (bg, border, fg) = if pressed {
                (theme::VIEW_PRESSED_BG, theme::ACCENT, theme::VIEW_PRESSED_FG)
            } else {
                (theme::BUTTON_BG, theme::BUTTON_BORDER, theme::BUTTON_FG)
            };
            painter.rect_filled(button, 0.0, bg);
            painter.rect_stroke(button, 0.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);
            painter.text(
                button.center(),
                Align2::CENTER_CENTER,
                glyph,
                FontId::proportional(theme::FS_TAB),
                fg,
            );
        }
        header.bottom()
    }

    /// browser-library.css:170-202 `.filterShelf`(tag は model の派生 tag)。
    fn draw_filter_shelf(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: Rect,
        top: f32,
    ) -> f32 {
        let shelf = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::SHELF_H),
        );
        painter.rect_filled(shelf, 0.0, theme::SHELF_BG);
        painter.hline(
            shelf.x_range(),
            shelf.bottom(),
            Stroke::new(1.0, theme::SHELF_BORDER),
        );
        let mut x = shelf.left() + 5.0;
        painter.text(
            Pos2::new(x, shelf.center().y),
            Align2::LEFT_CENTER,
            "FILTERS",
            FontId::proportional(theme::FS_SHELF_LABEL),
            theme::SHELF_LABEL_FG,
        );
        x += 30.0;
        let chip_font = FontId::proportional(theme::FS_CHIP);
        let tags: Vec<(String, String)> = self
            .projection
            .tags
            .iter()
            .map(|tag| (tag.id.clone(), tag.label.clone()))
            .collect();
        let mut clicked_tag: Option<String> = None;
        for (index, (id, label)) in tags.iter().enumerate() {
            let text_w = painter
                .layout_no_wrap(label.clone(), chip_font.clone(), theme::CHIP_FG)
                .size()
                .x;
            let chip = Rect::from_min_size(
                Pos2::new(x, shelf.center().y - theme::CHIP_H / 2.0),
                Vec2::new(text_w + 10.0, theme::CHIP_H),
            );
            if chip.right() > shelf.right() - 34.0 {
                break; // 溢れる chip は出さない(横 scroll は後続)。
            }
            x = chip.right() + 3.0; // css:174 gap 3px
            let response = ui.interact(chip, ui.id().with(("chip", index)), Sense::click());
            let selected = self.tag.as_deref() == Some(id.as_str());
            let (bg, border, fg) = if selected {
                (theme::CHIP_SELECTED_BG, theme::ACCENT, theme::CHIP_SELECTED_FG)
            } else {
                (theme::CHIP_BG, theme::CHIP_BORDER, theme::CHIP_FG)
            };
            painter.rect_filled(chip, theme::CHIP_RADIUS, bg);
            painter.rect_stroke(
                chip,
                theme::CHIP_RADIUS,
                Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );
            painter.text(chip.center(), Align2::CENTER_CENTER, label, chip_font.clone(), fg);
            if response.clicked() {
                clicked_tag = Some(id.clone());
            }
        }
        // css:202 Clear(右端)。
        let clear = Rect::from_min_size(
            Pos2::new(shelf.right() - 32.0, shelf.center().y - theme::CHIP_H / 2.0),
            Vec2::new(28.0, theme::CHIP_H),
        );
        let clear_response = ui.interact(clear, ui.id().with("clear-filter"), Sense::click());
        painter.text(
            clear.center(),
            Align2::CENTER_CENTER,
            "Clear",
            chip_font,
            theme::CHIP_CLEAR_FG,
        );
        if let Some(tag) = clicked_tag {
            // browser-library.html:339-343: tag を選ぶと source を All へ戻す。toggle。
            self.tag = if self.tag.as_deref() == Some(tag.as_str()) {
                None
            } else {
                Some(tag)
            };
            self.directory = None;
            self.kind = None;
        }
        if clear_response.clicked() {
            // browser-library.html:344-350 clearFilter。
            self.tag = None;
            self.directory = None;
            self.kind = None;
            self.query.clear();
        }
        shelf.bottom()
    }

    /// browser-library.css:204-213 `.resultSummary`。
    fn draw_result_summary(&self, painter: &egui::Painter, rect: Rect, top: f32) -> f32 {
        let summary = Rect::from_min_size(
            Pos2::new(rect.left(), top),
            Vec2::new(rect.width(), theme::SUMMARY_H),
        );
        painter.text(
            Pos2::new(summary.left() + 6.0, summary.center().y),
            Align2::LEFT_CENTER,
            "Results",
            FontId::proportional(theme::FS_SUMMARY),
            theme::TEXT,
        );
        painter.text(
            Pos2::new(summary.right() - 6.0, summary.center().y),
            Align2::RIGHT_CENTER,
            self.result_count().to_string(),
            FontId::proportional(theme::FS_SUMMARY_COUNT),
            theme::SUMMARY_COUNT_FG,
        );
        summary.bottom()
    }

    /// browser-library.css:215-276 `.thumbnailGrid` + `.libraryCard`(3 view)。
    fn draw_grid(&mut self, ui: &mut egui::Ui, rect: Rect) -> Option<BrowserRequest> {
        let cards = self.visible_cards();
        // css:218 / 269 / 273: 列数は view で決まる(grid 2 / thumbnails 4 / list 1)。
        let columns = match self.view {
            BrowserView::Thumbnails => 4,
            BrowserView::Grid => 2,
            BrowserView::List => 1,
        };
        let mut clicked: Option<String> = None;
        // 「置いてくれ」と言われたカード。**選択とは別の入れ物**で持つ —
        // ダブルクリックは単クリック(選択)も同時に真になるため。
        let mut place: Option<String> = None;
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        egui::ScrollArea::vertical()
            .id_salt("browser-grid")
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                let full_w = ui.available_width() - 2.0; // css:222 padding 0 1px
                let cell_w = full_w / columns as f32;
                let row_h = match self.view {
                    // list: thumb 46px 幅で高さは 16:9 + padding(css:275)。
                    BrowserView::List => {
                        theme::LIST_THUMB_W / theme::THUMB_ASPECT + theme::CARD_PAD * 2.0
                    }
                    // grid / thumbnails: thumb 16:9 + 文字2行(thumbnails は文字なし css:272)。
                    BrowserView::Grid => {
                        (cell_w - theme::CARD_PAD * 2.0) / theme::THUMB_ASPECT
                            + theme::CARD_PAD * 2.0
                            + 18.0
                    }
                    BrowserView::Thumbnails => {
                        (cell_w - theme::CARD_PAD * 2.0) / theme::THUMB_ASPECT
                            + theme::CARD_PAD * 2.0
                    }
                };
                let rows = cards.len().div_ceil(columns);
                let (area, _) = ui.allocate_exact_size(
                    Vec2::new(full_w, rows as f32 * row_h),
                    Sense::hover(),
                );
                let painter = ui.painter().with_clip_rect(ui.clip_rect());
                for (index, card) in cards.iter().enumerate() {
                    let col = index % columns;
                    let row = index / columns;
                    let cell = Rect::from_min_size(
                        Pos2::new(
                            area.left() + 1.0 + col as f32 * cell_w,
                            area.top() + row as f32 * row_h,
                        ),
                        Vec2::new(cell_w, row_h),
                    );
                    let response =
                        ui.interact(cell, ui.id().with(("card", &card.id)), Sense::click());
                    // card は painter で描く面なので、名乗らせないと AccessKit の木に
                    // 出ない = 運転席(kittest)からも支援技術からも掴めない。
                    // ここで名前を出すのは見た目を変えず、**触れることを機械が言える**
                    // ようにするためである。
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &card.name)
                    });
                    // **ダブルクリック = 配置**(Finder / Ableton / AM と同じ既定の
                    // 「開く」の位置)。判定は同じ response の `double_clicked()` で、
                    // 単クリックの選択と同居させるため先に見る。
                    if response.double_clicked() {
                        place = Some(card.id.clone());
                    } else if response.clicked() {
                        clicked = Some(card.id.clone());
                    }
                    self.draw_card(ui.ctx(), &painter, cell, card, response.hovered());
                }
            });
        if let Some(id) = clicked {
            self.select(&id);
        }
        // 置いたカードは選ばれてもいる(触った物が selection tray に残る)。
        let id = place?;
        self.select(&id);
        self.place_request(&id)
    }

    /// card 1枚。hover / selected の視覚は css:236-250。
    fn draw_card(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        cell: Rect,
        card: &BrowserCardModel,
        hovered: bool,
    ) {
        if card.selected {
            painter.rect_filled(cell, 0.0, theme::CARD_SELECTED_BG);
        }
        let inner = cell.shrink(theme::CARD_PAD);
        let (thumb, copy) = match self.view {
            BrowserView::List => {
                let thumb = Rect::from_min_size(
                    inner.min,
                    Vec2::new(theme::LIST_THUMB_W, theme::LIST_THUMB_W / theme::THUMB_ASPECT),
                );
                (thumb, Some(Rect::from_min_max(
                    Pos2::new(thumb.right() + 5.0, inner.top()), // css:276 padding-left 5px
                    inner.max,
                )))
            }
            BrowserView::Grid => {
                let thumb = Rect::from_min_size(
                    inner.min,
                    Vec2::new(inner.width(), inner.width() / theme::THUMB_ASPECT),
                );
                (thumb, Some(Rect::from_min_max(
                    Pos2::new(inner.left(), thumb.bottom() + 2.0), // css:266 margin-top 2px
                    inner.max,
                )))
            }
            BrowserView::Thumbnails => (
                Rect::from_min_size(
                    inner.min,
                    Vec2::new(inner.width(), inner.width() / theme::THUMB_ASPECT),
                ),
                None, // css:272 thumbnails view は cardCopy を出さない
            ),
        };
        self.draw_thumb(ctx, painter, thumb, card, hovered);
        if let Some(copy) = copy {
            // css:264-265 cardCopy は `overflow: hidden; text-overflow: ellipsis;
            // white-space: nowrap` — 隣の cell へ溢れさせない(ここは clip で写す)。
            let clipped = painter.with_clip_rect(copy.intersect(painter.clip_rect()));
            clipped.text(
                Pos2::new(copy.left(), copy.top() + 5.0),
                Align2::LEFT_CENTER,
                &card.name,
                FontId::proportional(theme::FS_CARD_NAME),
                theme::CARD_NAME_FG,
            );
            clipped.text(
                Pos2::new(copy.left(), copy.top() + 13.0),
                Align2::LEFT_CENTER,
                &card.meta,
                FontId::proportional(theme::FS_CARD_META),
                theme::CARD_META_FG,
            );
        }
    }

    /// thumb 枠。画像は縮小実体の texture、その他は kind glyph(mock と同じ割当)。
    fn draw_thumb(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        thumb: Rect,
        card: &BrowserCardModel,
        hovered: bool,
    ) {
        let is_svg = card.name.to_lowercase().ends_with(".svg");
        let (bg, glyph) = match card.kind {
            "video" => (theme::THUMB_VIDEO, "▶"), // html:123
            "audio" => (theme::THUMB_AUDIO, "♪"), // html:132
            _ if is_svg => (theme::THUMB_SVG, "SVG"), // html:126
            _ => (theme::THUMB_IMAGE, ""),        // html:129
        };
        painter.rect_filled(thumb, 0.0, bg);
        if let Some(texture) = self.texture_for(ctx, &card.id) {
            // 縮小実体を contain で載せる(thumbnail.rs が既に枠へ収めた縦横比)。
            let size = texture.size_vec2();
            let scale = (thumb.width() / size.x).min(thumb.height() / size.y);
            let draw = Rect::from_center_size(thumb.center(), size * scale);
            painter.image(
                texture.id(),
                draw,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else if !glyph.is_empty() {
            painter.text(
                Pos2::new(thumb.left() + 3.0, thumb.top() + 6.0), // css:245 padding 3px 左上
                Align2::LEFT_CENTER,
                glyph,
                FontId::proportional(theme::FS_THUMB_GLYPH),
                theme::THUMB_GLYPH_FG,
            );
        }
        let border = if card.selected {
            theme::ACCENT // css:250
        } else if hovered {
            theme::THUMB_HOVER_BORDER // css:249
        } else {
            theme::THUMB_BORDER // css:246
        };
        painter.rect_stroke(thumb, 0.0, Stroke::new(1.0, border), egui::StrokeKind::Inside);
        if card.selected {
            // css:250 `box-shadow: inset 0 0 0 1px #b9a66055`
            painter.rect_stroke(
                thumb.shrink(1.0),
                0.0,
                Stroke::new(1.0, theme::ACCENT.gamma_multiply(0.33)),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// 縮小実体を1回だけ texture へ upload する。失敗は `None` を記憶して再試行しない。
    fn texture_for(&mut self, ctx: &egui::Context, id: &str) -> Option<egui::TextureHandle> {
        if let Some(entry) = self.textures.get(id) {
            return entry.clone();
        }
        let path = self.thumbnails.get(id)?.clone();
        let loaded = match load_color_image(&path) {
            Ok(image) => Some(ctx.load_texture(
                format!("browser-thumb:{id}"),
                image,
                egui::TextureOptions::LINEAR,
            )),
            Err(reason) => {
                // 再試行しない方針は変えない(`None` を覚える)。変えたのは
                // **理由が1度だけ外へ出る**ことだけ。
                self.notices.push(format!(
                    "browser-panel: {} を texture にできないので glyph card で描く: {reason}",
                    path.display()
                ));
                None
            }
        };
        self.textures.insert(id.to_owned(), loaded.clone());
        loaded
    }

    /// browser-library.css:278-295 `.selectionTray`。
    fn draw_selection_tray(&self, painter: &egui::Painter, rect: Rect, top: f32) {
        let tray = Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), top + theme::TRAY_H),
        );
        painter.rect_filled(tray, 0.0, theme::TRAY_BG);
        painter.hline(tray.x_range(), tray.top(), Stroke::new(1.0, theme::TRAY_BORDER));
        let Some(item) = self.selection_item() else {
            painter.text(
                Pos2::new(tray.left() + 6.0, tray.center().y),
                Align2::LEFT_CENTER,
                "No selection",
                FontId::proportional(theme::FS_TRAY_META),
                theme::TRAY_META_FG,
            );
            return;
        };
        let dot = Rect::from_center_size(
            Pos2::new(tray.left() + 6.0 + theme::TRAY_DOT / 2.0, tray.center().y),
            Vec2::splat(theme::TRAY_DOT),
        );
        painter.rect_filled(dot, 0.0, theme::ACCENT);
        let name_pos = Pos2::new(dot.right() + 4.0, tray.center().y);
        let name_rect = painter.text(
            name_pos,
            Align2::LEFT_CENTER,
            &item.name,
            FontId::proportional(theme::FS_TRAY_NAME),
            theme::TRAY_NAME_FG,
        );
        painter.text(
            Pos2::new(name_rect.right() + 4.0, tray.center().y),
            Align2::LEFT_CENTER,
            format!(
                "{} · {} tags",
                if item.directory.is_empty() {
                    self.projection
                        .root
                        .as_ref()
                        .map(|root| root.name.clone())
                        .unwrap_or_default()
                } else {
                    item.directory.clone()
                },
                item.tags.len()
            ),
            FontId::proportional(theme::FS_TRAY_META),
            theme::TRAY_META_FG,
        );
    }
}

/// `.catalogHeader`(browser-library.css:155-168)の横並びを先に割った結果。
///
/// css:158 `display: flex` + css:166 `.viewModes { margin-left: auto }` は、
/// **ボタン塊を右端へ寄せ、見出しには残りだけを渡す**という規則である。egui の
/// painter はこれを自動でやらないので、同じ割り方をここで数値にして持つ。
#[derive(Debug, Clone, Copy, PartialEq)]
struct CatalogHeaderLayout {
    /// view-mode ボタン塊の左端(header 左端からの相対 px)。
    modes_left: f32,
    /// 見出し行に残る幅。
    text_w: f32,
    /// path 行の上限(css:165 の `max-width: 205px`、css:353 の narrow は 110px)。
    path_max_w: f32,
}

/// browser-library.css:102-103 / css:349 `.librarySidebar`。
fn sidebar_width(panel_w: f32) -> f32 {
    if panel_w <= theme::NARROW_BREAKPOINT_W {
        theme::SIDEBAR_W_NARROW
    } else {
        theme::SIDEBAR_W
    }
}

fn catalog_header_layout(header_w: f32, panel_w: f32) -> CatalogHeaderLayout {
    let pad = theme::CATALOG_HEADER_PAD_X;
    // ボタン塊は右 padding の内側(css:160 `padding: 0 6px`)。
    let modes_left = (header_w - pad - theme::VIEW_MODES_W).max(pad);
    // css:155-162 の `.catalogHeader` は gap を持たない — flex では
    // `margin-left: auto` が寄せるだけで、見出しはボタンに接するところまで伸びてよい。
    // ただし完全に接すると読みにくいので、この rule 内で唯一定義されている間隔
    // (css:166 `.viewModes { gap: 2px }`)を見出しとボタンの間にも使う。
    // 6px 空けると狭い pane で `All media` が畳まれてしまう。
    let text_w = (modes_left - pad - theme::VIEW_BUTTON_GAP).max(0.0);
    let path_cap = if panel_w <= theme::NARROW_BREAKPOINT_W {
        theme::CATALOG_PATH_MAX_W_NARROW
    } else {
        theme::CATALOG_PATH_MAX_W
    };
    CatalogHeaderLayout {
        modes_left,
        text_w,
        path_max_w: text_w.min(path_cap),
    }
}

/// 1行に収まらない文字列を `…` で畳んだ galley。
/// `Painter::text` は幅を見ないので、隣の席へ食い込ませないためにこちらを使う。
fn elided(
    painter: &egui::Painter,
    text: String,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text, font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width);
    painter.layout_job(job)
}

/// card 2行目(browser-library.html:123 `video · B-roll` の形)。
/// 出所は model の tag(kind の次の派生 tag = 拡張子か folder 名)。
fn card_meta(item: &LibraryItem) -> String {
    let secondary = item
        .tags
        .iter()
        .find(|tag| tag.as_str() != item.kind)
        .cloned()
        .unwrap_or_else(|| item.kind.to_owned());
    format!("{} · {}", item.kind, secondary)
}

/// 縮小実体1枚を egui の絵に写す。**落ちた段(読めない/画像でない)を落とさない** —
/// `.ok()?` で畳むと「画像が出ない」だけが残り、原因が散逸する(外部診断 F-09)。
fn load_color_image(path: &Path) -> Result<egui::ColorImage, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("read: {error}"))?;
    let image = image::load_from_memory(&bytes)
        .map_err(|error| format!("decode: {error}"))?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        image.as_raw(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// repo に入っている実 media(starter kit)。動画・画像・音声が1本ずつ在る。
    fn starter_media_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/mocks-ui/starter-media/media")
            .canonicalize()
            .expect("starter media lives in the repo")
    }

    /// パネルを headless に回す運転席(GPU 不要 — 見るのは AccessKit だけ)。
    /// 返るのは harness と、そのフレームでパネルが出した要求の置き場。
    fn drive(
        root: PathBuf,
    ) -> egui_kittest::Harness<'static, (BrowserPanel, Option<BrowserRequest>)> {
        egui_kittest::Harness::builder()
            .with_size([420.0, 520.0])
            .build_ui_state(
                |ui, state: &mut (BrowserPanel, Option<BrowserRequest>)| {
                    if let Some(request) = state.0.show(ui) {
                        state.1 = Some(request);
                    }
                },
                (BrowserPanel::with_root(root), None),
            )
    }

    /// 2 連打ぶんの押下/離しを**同じフレーム**へ積む。フレームを跨ぐと kittest の
    /// `step_dt`(0.25s)が egui の double-click 判定窓(0.3s)を食い潰す。
    fn double_click_at<S>(harness: &mut egui_kittest::Harness<'_, S>, center: egui::Pos2) {
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(center));
        harness.step();
        for _ in 0..2 {
            for pressed in [true, false] {
                harness.input_mut().events.push(egui::Event::PointerButton {
                    pos: center,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                });
            }
        }
        harness.step();
    }

    /// カードの中心(AccessKit の bounds から)。名乗っていない面は掴めない。
    fn card_center<S>(harness: &egui_kittest::Harness<'_, S>, name: &str) -> egui::Pos2 {
        use egui_kittest::kittest::Queryable as _;
        harness
            .query_all_by_label_contains(name)
            .next()
            .unwrap_or_else(|| panic!("card {name:?} が AccessKit の木に出ていない"))
            .rect()
            .center()
    }

    /// **ダブルクリックは配置要求になる。** パネルは Document を書かず、
    /// 指しているのは card の id ではなく**実ファイルの path** である
    /// (shell がそのままドロップと同じ経路へ流せる形)。
    #[test]
    fn double_clicking_a_card_asks_for_that_file_to_be_placed() {
        let root = starter_media_dir();
        let mut harness = drive(root.clone());
        harness.run();

        let center = card_center(&harness, "starter-clip.mp4");
        double_click_at(&mut harness, center);

        assert_eq!(
            harness.state().1,
            Some(BrowserRequest::PlaceFile(root.join("starter-clip.mp4"))),
            "カードのダブルクリックは、その実ファイルの配置要求として返らねばならない"
        );
    }

    /// 単クリックは従来どおり**選択だけ**。要求は出ない
    /// (選ぶたびに clip が増えたら人は選べない)。
    #[test]
    fn a_single_click_only_selects_and_asks_for_nothing() {
        let root = starter_media_dir();
        let mut harness = drive(root);
        harness.run();

        let center = card_center(&harness, "starter-clip.mp4");
        harness
            .input_mut()
            .events
            .push(egui::Event::PointerMoved(center));
        harness.step();
        for pressed in [true, false] {
            harness.input_mut().events.push(egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            });
        }
        harness.step();

        let selected = harness
            .state()
            .0
            .selected_id()
            .map(str::to_owned)
            .expect("単クリックは選択する");
        assert!(
            selected.ends_with("starter-clip.mp4"),
            "選ばれたのは触ったカード。実際: {selected:?}"
        );
        assert_eq!(
            harness.state().1,
            None,
            "単クリックは配置要求を出さない(選ぶだけ)"
        );
    }

    /// 知らない id は要求を作らない — 存在しない path を shell へ投げない。
    #[test]
    fn an_unknown_card_id_yields_no_request() {
        let root = starter_media_dir();
        let panel = BrowserPanel::with_root(root.clone());
        assert!(panel.place_request("no-such-card").is_none());

        let clip = panel
            .visible_cards()
            .into_iter()
            .find(|card| card.name == "starter-clip.mp4")
            .expect("starter kit の動画カード")
            .id;
        assert_eq!(
            panel.place_request(&clip),
            Some(BrowserRequest::PlaceFile(root.join("starter-clip.mp4"))),
            "id → 実 path の解決は既存 MediaLibrary::resolve を通る"
        );
    }

    /// css:158 flex + css:166 `margin-left: auto`。
    /// **見出しの席と view-mode ボタンの席は決して重ならない。**
    #[test]
    fn the_catalog_title_never_reaches_under_the_view_modes() {
        for header_w in [20.0_f32, 90.0, 120.0, 200.0, 320.0, 640.0, 1200.0] {
            let layout = catalog_header_layout(header_w, header_w);
            let text_right = theme::CATALOG_HEADER_PAD_X + layout.text_w;
            assert!(
                text_right <= layout.modes_left,
                "header {header_w}: 見出し右端 {text_right} が modes 左端 {} を越えた",
                layout.modes_left
            );
            assert!(layout.text_w >= 0.0);
            assert!(layout.path_max_w <= layout.text_w);
        }
    }

    /// ボタン塊は右 padding の内側に丸ごと収まる(css:160 `padding: 0 6px`)。
    #[test]
    fn the_view_modes_sit_inside_the_right_padding() {
        let header_w = 320.0;
        let layout = catalog_header_layout(header_w, header_w);
        assert!(
            (layout.modes_left + theme::VIEW_MODES_W - (header_w - theme::CATALOG_HEADER_PAD_X))
                .abs()
                < 0.001,
            "modes 塊が右 padding に揃っていない"
        );
    }

    /// css:165 は 205px、css:353 の `@media (max-width: 420px)` は 110px。
    #[test]
    fn the_path_line_takes_the_narrow_cap_on_a_narrow_pane() {
        let wide = catalog_header_layout(900.0, 900.0);
        assert_eq!(wide.path_max_w, theme::CATALOG_PATH_MAX_W);
        let narrow = catalog_header_layout(400.0, 400.0);
        assert_eq!(narrow.path_max_w, theme::CATALOG_PATH_MAX_W_NARROW);
    }

    /// css:349 の narrow sidebar。狭い pane では catalog に席が返る。
    #[test]
    fn a_narrow_pane_hands_the_sidebar_width_back_to_the_catalog() {
        assert_eq!(sidebar_width(900.0), theme::SIDEBAR_W);
        assert_eq!(sidebar_width(300.0), theme::SIDEBAR_W_NARROW);
        let wide_catalog = 300.0 - sidebar_width(300.0);
        let narrow_catalog = 300.0 - theme::SIDEBAR_W;
        assert!(wide_catalog > narrow_catalog);
    }

    /// ボタンすら入らない幅でも席を負にしない(header が畳まれた時の下限)。
    #[test]
    fn a_pane_too_narrow_for_the_buttons_still_yields_a_sane_layout() {
        let layout = catalog_header_layout(20.0, 20.0);
        assert_eq!(layout.modes_left, theme::CATALOG_HEADER_PAD_X);
        assert_eq!(layout.text_w, 0.0);
        assert_eq!(layout.path_max_w, 0.0);
    }
}
