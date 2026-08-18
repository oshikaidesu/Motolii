//! 部品契約の**薄い stub** — 本物の widgets module は別レーンが実装中。
//!
//! // INTEGRATION: swap to widgets module
//!
//! 契約(発注文の指定どおり。名前・引数・返りを変えない):
//!
//! ```text
//! pub enum DropEvent { HoverEnter, HoverLeave }
//! pub fn drop_zone<'a, M>(inner, accepting, on_event) -> iced::Element<'a, M>
//! pub struct MenuItem { pub id: u32, pub label: String, pub enabled: bool }
//! pub enum MenuEvent { Chosen(u32), Dismissed }
//! pub fn context_menu<'a, M>(items, at, on_event) -> iced::Element<'a, M>
//! ```
//!
//! ## stub の割り切り
//!
//! - `drop_zone` は **hover の点灯/消灯を Message に写すだけ**。受け皿の見た目は
//!   呼び手(Browser pane)が `accepting` と自分の状態から描く。
//!   `FileDropped` は**捕まない** — 取り込みは窓ぜんたい(`window_input` →
//!   `AdmitPaths`)の仕事で、panel が奪わない
//! - `context_menu` は `at` へ浮かべる overlay を持たず、その場に列を出すだけ。
//!   M-4a の Browser は menu 面を持たない(egui 版と同じ)ので、この stub は
//!   契約の席を確保する以上の仕事をしない

use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{layout, mouse, overlay, renderer, Layout, Shell as IcedShell, Widget};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

/// 掴んだファイルが受け皿の上に来た/離れた。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropEvent {
    HoverEnter,
    HoverLeave,
}

/// `inner` を OS ドロップの受け皿表示つきで包む。
pub fn drop_zone<'a, M>(
    inner: iced::Element<'a, M>,
    accepting: bool,
    on_event: impl Fn(DropEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    Element::new(DropZone {
        inner,
        accepting,
        on_event: Box::new(on_event),
    })
}

struct DropZone<'a, M> {
    inner: Element<'a, M>,
    accepting: bool,
    on_event: Box<dyn Fn(DropEvent) -> M + 'a>,
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for DropZone<'_, M> {
    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.inner));
    }

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

        // 点灯/消灯は**事象をそのまま写す**(widget 側で状態を持たない)。
        // 受け手(殻の `set_drop_hover`)が冪等なので、複数 file の hover で
        // 同じ Message が重なっても意味は変わらない。widget 木の状態にしないのは、
        // 運転席が「view を作り直して1事象だけ流す」駆動でも消灯が届くため。
        match event {
            Event::Window(iced::window::Event::FileHovered(_)) if self.accepting => {
                // 捕まない: 他の受け皿(将来の複数 zone)も同じ事象を見てよい。
                shell.publish((self.on_event)(DropEvent::HoverEnter));
            }
            Event::Window(
                iced::window::Event::FilesHoveredLeft | iced::window::Event::FileDropped(_),
            ) => {
                // **`FileDropped` はここで捕まない。** 表示を畳むだけで、
                // 取り込みは窓ぜんたい(`window_input` → `AdmitPaths`)に流れる。
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

/// context menu の1行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: u32,
    pub label: String,
    pub enabled: bool,
}

/// context menu が返す事象。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    Chosen(u32),
    Dismissed,
}

/// 薄い stub: `at` へ浮かべる overlay は持たず、その場に列で出す。
/// M-4a の Browser は menu 面を持たない(egui 版と同じ)ので未使用のまま
/// 契約の席だけを確保している。本物が来たらこの module ごと差し替える。
pub fn context_menu<'a, M>(
    items: Vec<MenuItem>,
    at: iced::Point,
    on_event: impl Fn(MenuEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    // `button::on_press` が Message: Clone を要る(iced の既定)。契約の M は
    // この crate では `Message`(Clone)なので、bound を書いても席は変わらない。
    M: Clone + 'a,
{
    // 薄い stub は位置決めをしない(本物は `at` へ浮かべる)。
    let _ = at;
    let mut menu = iced::widget::column![].spacing(2);
    for item in items {
        let mut entry = iced::widget::button(iced::widget::text(item.label).size(12))
            .style(iced::widget::button::text);
        if item.enabled {
            entry = entry.on_press(on_event(MenuEvent::Chosen(item.id)));
        }
        menu = menu.push(entry);
    }
    iced::widget::container(menu)
        .style(iced::widget::container::bordered_box)
        .padding(2)
        .into()
}
