//! 絵。**モデルを書く道がここには無い**(`&Shell` しか受け取らない)。
//!
//! iced ではこれが型で保証される — egui shell の `show(&mut self, ui)` は
//! 描画の途中で製品状態を書けてしまい、それを禁じるのに走査フェンスが要った。
//!
//! ## 文言は egui 版と同じ
//!
//! 移行の途中で利用者の見る言葉が揺れないよう、スタート画面と status 帯の
//! 文字列は `blitz_shell/app.rs` からそのまま写している。違うのは**組み方**だけ:
//! egui は "New Project…   Cmd+N" を1つの label に詰めているが、ここは行(`row`)に
//! 割った。近道の表示は別の物なので別の widget にする方が iced では素直で、
//! 運転席の selector(`&str` は**完全一致**)も「押したいボタンの名前」だけを
//! 名指しできる。
//!
//! ## M-3 で帯に来た物
//!
//! Undo / Redo ボタン(編集面 = Timeline と一緒に来た)。`Cmd+Z` / `Shift+Cmd+Z`
//! と同じ `UiIntent::Undo` / `Redo` へ流れる — 経路も意味も1つ。

use iced::widget::canvas::Canvas;
use iced::widget::{
    button, column, container, mouse_area, row, rule, scrollable, space, stack, text,
};
use iced::{Center, Element, Fill};

use crate::browser::{BrowserCard, BrowserRail};
use crate::message::Message;
use crate::shell::Shell;
use crate::theme::style;
use crate::timeline::keys::interp_menu_items;
use crate::timeline::{TimelineMsg, TimelineProgram};
use crate::widgets::context_menu::{context_menu, MenuEvent};
use crate::widgets::DropEvent;
use crate::window_input::window_input;

/// スタート画面の見出し。
pub const TITLE: &str = "Motolii";
/// 見出しの下の一行。
pub const TAGLINE: &str = "Make one 3\u{2013}5 minute music video.";
/// New ボタンの名前。**運転席がこの文字列で押す**(完全一致)。
pub const NEW_PROJECT: &str = "New Project\u{2026}";
/// New ボタンに添える近道。
pub const NEW_PROJECT_SHORTCUT: &str = "Cmd+N";
/// Open ボタンの名前。**運転席がこの文字列で押す**(完全一致)。
pub const OPEN_PROJECT: &str = "Open\u{2026}";
/// Open ボタンに添える近道。
pub const OPEN_PROJECT_SHORTCUT: &str = "Cmd+O";
/// スタート画面の末尾の一行。
pub const DROP_HINT: &str = "Then just drop video and audio into this window.";
/// Browser pane の幅(既定配置の左)。panel の resize / dock 文法は
/// M-5 以降の統合 wave で来る(ui-interaction-language §3 の組み込み既定)。
pub const BROWSER_PANE_W: f32 = 316.0;
/// Inspector pane の幅(既定配置の右)。
pub const INSPECTOR_PANE_W: f32 = 300.0;
// 上段(Browser|Stage|Inspector)対 下段 Timeline の高さ比は egui shell
// (`crates/motolii-ui/src/blitz_shell/app.rs::build_initial_tree`)の中央列を
// 正典にする: そこは Stage/Timeline を `insert_vertical_tile` へ明示 share を
// 与えず渡していて、`egui_tiles::Shares::get_share` は未設定を 1.0 として扱う
// ので実測は 1:1(50:50)。「だいたい6:4」という見積りではなく、egui 側に実在
// するこの数字を `seated()` の `Length::FillPortion(1)` 2本で採る。
/// 書き出しを始めるボタン。
pub const EXPORT: &str = "Export";
/// 走っている書き出しを止めるボタン。
pub const CANCEL_EXPORT: &str = "Cancel";
/// 帯の Undo ボタン(`Cmd+Z` と同じ入口 — 経路も意味も1つ)。
pub const UNDO: &str = "Undo";
/// 帯の Redo ボタン(`Shift+Cmd+Z` と同じ入口)。
pub const REDO: &str = "Redo";

// ---------------------------------------------------------------------------
// Browser pane(M-4a)の名乗り。運転席は全部この文字列で掴む(完全一致)。
// ---------------------------------------------------------------------------

/// pane の見出し。
pub const BROWSER_HEADER: &str = "Browser";
/// source rail: 登録 folder + Document asset の統合面。
pub const BROWSER_RAIL_ALL: &str = "All media";
/// source rail: Document の asset 台帳。
pub const BROWSER_RAIL_PROJECT: &str = "Project";
/// source rail: 同じ台帳を取り込みの新しい順で。
pub const BROWSER_RAIL_RECENT: &str = "Recent";
/// 掴んだファイルが窓の上に居るあいだ、panel が受け皿であることを言う一行。
/// **取り込み自体は窓ぜんたい**(`AdmitPaths`)で、panel は表示だけ。
pub const BROWSER_DROP_TARGET: &str = "Drop anywhere in this window to add media";
/// Project rail の空状態(Q7: 空は空として、次の一手つきで言う)。
pub const BROWSER_EMPTY_PROJECT: &str =
    "No media in this project yet — drop files into this window, or double-click a card in All media.";
/// Recent rail の空状態。
pub const BROWSER_EMPTY_RECENT: &str = "Nothing has been added to this project yet.";
/// selection tray の空状態。
pub const BROWSER_NO_SELECTION: &str = "No selection";

/// All media の空状態は登録 folder を名指しする(幽霊 card を出さない)。
pub fn browser_empty_all(folder: &str) -> String {
    format!("No media found in {folder}.")
}

/// selection tray の名乗り。card の text と選択の text を運転席が
/// 別々に掴めるよう、tray は `●` を前置する。
pub fn browser_tray_label(name: &str) -> String {
    format!("\u{25cf} {name}")
}

/// 未保存の座席の名乗り方。egui shell の status 帯と**一字も違わない**。
pub fn unsaved_label(name: &str) -> String {
    format!("\u{25cf} {name} \u{2014} unsaved")
}

/// 走っている書き出しの名乗り方(進捗口が無い v0 の「まだ生きている」表示)。
pub fn exporting_label(seconds: u64) -> String {
    format!("Exporting\u{2026} {seconds}s")
}

/// 窓ぜんたい。
///
/// 一番外は [`window_input`] — 近道キー・OS ドロップ・閉じる要求はここで
/// [`Message`] になる。中身は「座席の有無で変わる本体」と「status 帯」の2段。
pub fn view(shell: &Shell) -> Element<'_, Message> {
    let body: Element<'_, Message> = if shell.is_seated() {
        seated(shell)
    } else {
        container(start_screen()).center(Fill).into()
    };

    let mut page = column![container(body).width(Fill).height(Fill)]
        .width(Fill)
        .height(Fill);

    // 帯は「座席が居るあいだ」と「言われたことがある時」に出る。空の帯を常設しない
    // (egui 版と同じ判断)。
    if let Some(band) = status_band(shell) {
        page = page.push(band);
    }

    window_input(page).into()
}

/// 座席が無いときの画面。
fn start_screen<'a>() -> Element<'a, Message> {
    column![
        text(TITLE).size(34),
        text(TAGLINE).style(style::text_secondary),
        column![
            action_button(
                NEW_PROJECT,
                NEW_PROJECT_SHORTCUT,
                Message::NewProjectPressed
            ),
            action_button(
                OPEN_PROJECT,
                OPEN_PROJECT_SHORTCUT,
                Message::OpenProjectPressed
            ),
        ]
        .spacing(8)
        .align_x(Center),
        text(DROP_HINT).style(style::text_muted),
    ]
    .spacing(18)
    .align_x(Center)
    .into()
}

/// 名前と近道を1つのボタンに並べる。名前は独立した text なので、
/// 運転席は `"New Project…"` の完全一致で掴める。
/// 近道は添え物なので muted — 名前と同格の顔をさせない。
fn action_button<'a>(name: &'a str, shortcut: &'a str, message: Message) -> Element<'a, Message> {
    button(
        row![
            text(name).size(16),
            text(shortcut).size(16).style(style::text_muted)
        ]
        .spacing(12),
    )
    .style(style::action)
    .on_press(message)
    .into()
}

/// 座席が在るときの画面 — 既定配置(ui-interaction-language §3):
/// 上段は 左 Browser(M-4a)/ 中央 Stage 島(M-2)/ 右 Inspector(M-4b)、
/// 下段は Timeline(M-3, canvas)。分割線は 1px([`style::separator`])。
///
/// 中央上は [`crate::stage_island::stage_island`]: Rerun `SpatialStage` の
/// 合成絵が shader widget として立ち、入力はブリッジ(調停つき)を通る。
/// 上段:下段の高さ比は 1:1(egui shell の中央列の実測 — [`TOP_ROW_HEIGHT`] の
/// docコメント参照)。
fn seated(shell: &Shell) -> Element<'_, Message> {
    let top = row![
        browser_panel(shell),
        rule::vertical(1).style(style::separator),
        container(crate::stage_island::stage_island(shell))
            .width(Fill)
            .height(Fill),
        rule::vertical(1).style(style::separator),
        container(crate::inspector_pane::inspector(
            shell.inspector(),
            shell.editor_status(),
        ))
        .width(INSPECTOR_PANE_W)
        .height(Fill),
    ]
    .height(iced::Length::FillPortion(1));

    column![
        top,
        rule::horizontal(1).style(style::separator),
        container(timeline_pane(shell))
            .id(TIMELINE_PANE_ID)
            .height(iced::Length::FillPortion(1)),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

/// Timeline pane を包む `container` の id。**運転席がここへ実座標を訊く**——
/// 4面合成(統合第2弾)で Timeline canvas が窓の左上 (0,0) には立たなくなった
/// ので、運転席は `Simulator::find(TIMELINE_PANE_ID)` で実 bounds を引いてから
/// canvas ローカル座標(`timeline::semantics::PaneGeometry`)へ足す
/// (`drive_timeline.rs::geometry_offset`)。
pub const TIMELINE_PANE_ID: iced::advanced::widget::Id =
    iced::advanced::widget::Id::new("motolii-timeline-pane");

/// 下段 Timeline pane(M-3)。canvas は自分の受け持つ矩形の左上を原点に取る
/// ので、運転席のポインタ座標(`timeline::semantics::PaneGeometry`)はその
/// 矩形のローカル座標のまま成立する(窓座標への変換は [`TIMELINE_PANE_ID`])。
///
/// **2026-08-19 M-6**: 右クリックで開いたキーの補間メニュー
/// (`TimelinePane::key_menu`)が在れば、canvas の上に重ねる
/// (`widgets::context_menu` — M-4w で用意した既存の overlay 部品を初めて使う)。
fn timeline_pane(shell: &Shell) -> Element<'_, Message> {
    let canvas: Element<'_, Message> = Canvas::new(TimelineProgram { shell })
        .width(Fill)
        .height(Fill)
        .into();
    let Some(menu) = shell.timeline_pane().key_menu else {
        return canvas;
    };
    let items = interp_menu_items(menu.param);
    let (layer, param, at_us) = (
        menu.layer,
        menu.param,
        motolii_ui::blitz_shell::seconds_to_us(menu.at_seconds),
    );
    let overlay = context_menu(items, menu.anchor, move |event| match event {
        MenuEvent::Chosen(id) => match crate::timeline::keys::interp_for_menu_id(id) {
            Some(interp) => Message::Timeline(TimelineMsg::KeyInterpChosen {
                layer,
                param,
                at_us,
                interp,
            }),
            None => Message::Timeline(TimelineMsg::KeyMenuDismissed),
        },
        MenuEvent::Dismissed => Message::Timeline(TimelineMsg::KeyMenuDismissed),
    });
    stack(vec![canvas, overlay]).width(Fill).height(Fill).into()
}

/// grid の列数。
const BROWSER_GRID_COLUMNS: usize = 3;

/// Browser pane 1面。上から見出し(+受け皿表示)、source rail と grid、
/// selection tray。**Document を書く道はここに無い** — カードのダブルクリックは
/// Message になり、殻が `UiIntent::AdmitPaths` へ写す。
fn browser_panel(shell: &Shell) -> Element<'_, Message> {
    let pane = shell.browser();
    let cards = shell.browser_cards();

    // selection tray の名乗り(grid が card を消費する前に写す)。
    let tray_line = pane
        .selected_id()
        .and_then(|selected| {
            cards
                .iter()
                .find(|card| card.id == selected)
                .map(|card| browser_tray_label(&card.name))
        })
        .unwrap_or_else(|| BROWSER_NO_SELECTION.to_owned());

    // source rail(機能する3席だけ = Q0)。選択中と hover の段階は
    // `theme::Tokens` の semantic role から(独自 hex を発明しない)。
    // 見た目は browser-library.css:126-147 `.locationRow` の写し
    // (この製品の色は raw hex で token 対応が無いので、role だけを合わせる:
    // 選択 = action_active の字 + surface_raised の地、hover = surface_hover)。
    let mut rail = column![].spacing(1).width(96);
    for (label, value) in [
        (BROWSER_RAIL_ALL, BrowserRail::AllMedia),
        (BROWSER_RAIL_PROJECT, BrowserRail::Project),
        (BROWSER_RAIL_RECENT, BrowserRail::Recent),
    ] {
        let selected = pane.rail() == value;
        rail = rail.push(
            button(text(label).size(11))
                .padding([3, 7])
                .style(move |theme, status| rail_button_style(theme, status, selected))
                .width(Fill)
                .on_press(Message::BrowserRailChosen(value)),
        );
    }

    // grid(空なら Q7 の正直な空状態 — 幽霊 card を出さない)。
    let body: Element<'_, Message> = if cards.is_empty() {
        let line = match pane.rail() {
            BrowserRail::AllMedia => browser_empty_all(&pane.library_root_name()),
            BrowserRail::Project => BROWSER_EMPTY_PROJECT.to_owned(),
            BrowserRail::Recent => BROWSER_EMPTY_RECENT.to_owned(),
        };
        container(text(line).size(12)).padding(8).width(Fill).into()
    } else {
        let mut grid = column![].spacing(4).width(Fill);
        let mut cells = row![].spacing(4);
        let mut in_row = 0;
        for card in cards {
            cells = cells.push(browser_card(card));
            in_row += 1;
            if in_row == BROWSER_GRID_COLUMNS {
                grid = grid.push(cells);
                cells = row![].spacing(4);
                in_row = 0;
            }
        }
        if in_row > 0 {
            // 端数行でも cell 幅を揃える(空席で埋める)。
            while in_row < BROWSER_GRID_COLUMNS {
                cells = cells.push(space::horizontal());
                in_row += 1;
            }
            grid = grid.push(cells);
        }
        scrollable(grid).height(Fill).into()
    };

    // header 帯 — browser-library.css:28-38 の写し: 太字の見出し + 右に
    // library root の名前(reference の "LOCAL LIBRARY" に当たる、既存データ)。
    let header = container(
        row![
            text(BROWSER_HEADER)
                .size(11)
                .font(bold_font(iced::font::Weight::Bold)),
            space().width(Fill),
            text(pane.library_root_name())
                .size(9)
                .style(style::text_muted),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .padding([6, 8])
    .width(Fill)
    .style(browser_band_style);

    let mut panel = column![header]
        .spacing(6)
        .width(BROWSER_PANE_W)
        .height(Fill);
    if pane.drop_hover() {
        // 受け皿の点灯(**表示だけ**。取り込みは窓ぜんたいの AdmitPaths)。
        panel = panel.push(
            container(text(BROWSER_DROP_TARGET).size(12))
                .padding(6)
                .width(Fill)
                .style(drop_hover_style),
        );
    }
    panel = panel.push(
        row![rail, body]
            .spacing(8)
            .padding(iced::padding::left(8).right(8))
            .height(Fill),
    );
    // selection tray — browser-library.css:278-293 の写し(帯 + 頭に選択の点)。
    panel = panel.push(
        container(text(tray_line).size(11))
            .padding([5, 8])
            .width(Fill)
            .style(browser_band_style),
    );

    os_file_drop_zone(
        container(panel)
            .style(browser_panel_style)
            .height(Fill)
            .into(),
        shell.is_seated(),
        |event| Message::BrowserDropHover(matches!(event, DropEvent::HoverEnter)),
    )
}

/// OS drag のファイル受け皿表示。**`crate::widgets::drop_zone` とは意味が違う** —
/// widgets 版はマウス cursor の面内滞在(`CursorMoved`/`CursorLeft`)を hover と
/// 呼ぶ汎用 widget で、Browser が要るのは OS がファイルを窓上へ運んでいる間だけ
/// 点く受け皿表示(`FileHovered` / `FilesHoveredLeft` window event)である。
/// 型は同じ [`DropEvent`] 契約を借りるが、統合 wave で widgets 版へ差し替えると
/// この意味が壊れるため、ここは Browser の局所 widget として残す
/// (`FileDropped` は捕まない — 取り込みは [`window_input`] → `AdmitPaths` のまま)。
fn os_file_drop_zone<'a, M>(
    inner: iced::Element<'a, M>,
    accepting: bool,
    on_event: impl Fn(DropEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    use iced::advanced::widget::{Operation, Tree};
    use iced::advanced::{layout, mouse, overlay, renderer, Layout, Shell as IcedShell, Widget};
    use iced::{Event, Length, Rectangle, Size, Vector};

    struct OsFileDropZone<'a, M> {
        inner: Element<'a, M>,
        accepting: bool,
        on_event: Box<dyn Fn(DropEvent) -> M + 'a>,
    }

    impl<M> Widget<M, iced::Theme, iced::Renderer> for OsFileDropZone<'_, M> {
        fn size(&self) -> Size<Length> {
            self.inner.as_widget().size()
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.inner
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn diff(&mut self, tree: &mut Tree) {
            tree.diff_children(std::slice::from_mut(&mut self.inner));
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn Operation,
        ) {
            self.inner
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &iced::Renderer,
            shell: &mut IcedShell<'_, M>,
            viewport: &Rectangle,
        ) {
            self.inner.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                shell,
                viewport,
            );

            // 点灯/消灯は事象をそのまま写す(widget 側で状態を持たない — 複数
            // zone が同じ事象を見てよい)。`FileDropped` はここで**捕まない**。
            match event {
                Event::Window(iced::window::Event::FileHovered(_)) if self.accepting => {
                    shell.publish((self.on_event)(DropEvent::HoverEnter));
                }
                Event::Window(
                    iced::window::Event::FilesHoveredLeft | iced::window::Event::FileDropped(_),
                ) => {
                    shell.publish((self.on_event)(DropEvent::HoverLeave));
                }
                _ => {}
            }
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &iced::Renderer,
        ) -> mouse::Interaction {
            self.inner.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut iced::Renderer,
            theme: &iced::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.inner.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &iced::Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
            self.inner.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }

    Element::new(OsFileDropZone {
        inner,
        accepting,
        on_event: Box::new(on_event),
    })
}

/// source rail の1行。選択中は `action_active` の字 + `surface_raised` の地、
/// 未選択は `text_secondary`。hover は `surface_hover` で段階的に応える(Q3)。
fn rail_button_style(theme: &iced::Theme, status: button::Status, selected: bool) -> button::Style {
    let t = crate::theme::Tokens::resolve(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(t.surface_hover.into()),
        _ if selected => Some(t.surface_raised.into()),
        _ => None,
    };
    button::Style {
        background,
        text_color: if selected {
            t.action_active
        } else {
            t.text_secondary
        },
        border: iced::Border::default(),
        ..button::Style::default()
    }
}

/// 受け皿の点灯面 — `action_active` の淡い地。
fn drop_hover_style(theme: &iced::Theme) -> container::Style {
    let t = crate::theme::Tokens::resolve(theme);
    container::Style {
        background: Some(
            iced::Color {
                a: 0.12,
                ..t.action_active
            }
            .into(),
        ),
        border: iced::Border {
            color: t.action_active,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// 太字 font(section 見出し・header title に使う。`inspector_pane::weight` と
/// 同じ写し — letter-spacing はこの iced fork に無いので持たない)。
fn bold_font(weight: iced::font::Weight) -> iced::Font {
    iced::Font {
        weight,
        ..iced::Font::DEFAULT
    }
}

/// header / selection tray の帯 — `surface_raised`(browser-library.css の
/// header/tray は panel 地より僅かに明るい面を使う。この製品では raw hex に
/// token 対応が無いので、意味の近い `surface_raised` を採る)。
fn browser_band_style(theme: &iced::Theme) -> container::Style {
    let t = crate::theme::Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_raised.into()),
        border: iced::Border {
            color: t.border_default,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// Browser pane の外枠 — `surface_panel` の地 + `border_default` の 1px 枠。
fn browser_panel_style(theme: &iced::Theme) -> container::Style {
    let t = crate::theme::Tokens::resolve(theme);
    container::Style {
        background: Some(t.surface_panel.into()),
        border: iced::Border {
            color: t.border_default,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// card 1枚。click=選択 / double-click=配置(Q1 の共通文法)。
fn browser_card(card: BrowserCard) -> Element<'static, Message> {
    let glyph = match card.kind {
        "video" => "\u{25b6}", // ▶(egui 版 draw_thumb と同じ割当)
        "audio" => "\u{266a}", // ♪
        _ => "\u{25e7}",       // ▧
    };
    let thumb: Element<'static, Message> = match card.thumbnail {
        // 縮小実体(既存座席の cache)を載せる。元画像へは戻さない。
        Some(path) => iced::widget::image(iced::widget::image::Handle::from_path(path))
            .width(Fill)
            .height(48)
            .into(),
        None => container(text(glyph).size(16))
            .center_x(Fill)
            .height(48)
            .into(),
    };
    let selected = card.selected;
    // caption 2行 — browser-library.css:264-267 の写し(name = bold、
    // meta = muted の小字)。
    let body = column![
        container(thumb)
            .style(move |theme: &iced::Theme| browser_thumb_style(theme, selected))
            .width(Fill)
            .height(48),
        text(card.name)
            .size(10)
            .font(bold_font(iced::font::Weight::Bold)),
        text(card.meta).size(9).style(style::text_muted),
    ]
    .spacing(2);
    mouse_area(
        container(body)
            .padding(4)
            .width(Fill)
            .style(move |theme| browser_card_style(theme, selected)),
    )
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Message::BrowserCardClicked(card.id.clone()))
    .on_double_click(Message::BrowserCardActivated(card.id))
    .into()
}

/// thumbnail 枠 — browser-library.css:240-250 の写し(既定 `border_default`、
/// hover は素通し、選択は `action_active`)。
fn browser_thumb_style(theme: &iced::Theme, selected: bool) -> container::Style {
    let t = crate::theme::Tokens::resolve(theme);
    container::Style {
        border: iced::Border {
            color: if selected {
                t.action_active
            } else {
                t.border_default
            },
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// card の面。選択の強調は `theme::Tokens` の semantic role から
/// (独自 hex は発明しない — accent = `action_active`、地は `surface_hover`)。
fn browser_card_style(theme: &iced::Theme, selected: bool) -> container::Style {
    let t = crate::theme::Tokens::resolve(theme);
    let mut style = container::rounded_box(theme);
    if selected {
        style.background = Some(t.surface_hover.into());
        style.border = style.border.color(t.action_active).width(1);
    }
    style
}

/// status 帯 — **信用の可視化と、窓が言ったことの最新1行**。
///
/// 並びは egui shell と同じ: 保存状態 → 書き出し面 → 最新の一言。
/// 言うことも座席も無ければ帯そのものを出さない。
fn status_band(shell: &Shell) -> Option<Element<'_, Message>> {
    let latest = shell.latest_report();
    if !shell.is_seated() && latest.is_none() {
        return None;
    }

    let mut band = row![].spacing(12).align_y(Center);

    // 保存状態(保存済みなら project 名、未保存なら ● 付き)。
    if let Some(name) = shell.project_name() {
        let label = if shell.is_dirty() {
            unsaved_label(&name)
        } else {
            name
        };
        band = band.push(text(label).size(13));
    }

    // Undo / Redo(M-3)。`Cmd+Z` / `Shift+Cmd+Z` と同じ入口へ流れる。
    // 台帳が空の側は押せない(触れそうで触れない物にしない — 灰色は「無効」の意味)。
    if shell.is_seated() {
        band = band.push(button(text(UNDO).size(13)).on_press_maybe(
            (shell.undo_len() > 0).then_some(Message::Timeline(TimelineMsg::UndoPressed)),
        ));
        band = band.push(button(text(REDO).size(13)).on_press_maybe(
            (shell.redo_len() > 0).then_some(Message::Timeline(TimelineMsg::RedoPressed)),
        ));
    }

    // 書き出し面。実行中は経過秒と Cancel、そうでなければ Export。
    // **実行中に Export は出さない** = 二重起動の口が無い。
    if let Some(seconds) = shell.export_elapsed_seconds() {
        band = band.push(text(exporting_label(seconds)).size(13));
        let mut cancel = button(text(CANCEL_EXPORT).size(13)).style(style::action);
        if !shell.export_cancel_requested() {
            cancel = cancel.on_press(Message::CancelExportPressed);
        }
        band = band.push(cancel);
    } else if shell.is_seated() {
        let mut export = button(text(EXPORT).size(13)).style(style::action);
        if shell.can_start_export() {
            export = export.on_press(Message::ExportPressed);
        }
        band = band.push(export);
    }

    if let Some(latest) = latest {
        band = band.push(text(latest).size(13).style(style::text_secondary));
    }

    // 帯は panel 面 + 上辺の区切り線。土台との段差は明度差と 1px の線で作る
    // (UI視覚言語「面」: 階層は小さな明度差と境界線で作る)。
    Some(
        column![
            rule::horizontal(1).style(style::separator),
            container(band)
                .padding(8)
                .width(Fill)
                .style(style::status_band),
        ]
        .into(),
    )
}
