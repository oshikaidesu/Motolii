//! drop 受け皿 — DnD affordance。hover 中はハイライトし、受入可否を色で言う。
//!
//! ドロップそのものは窓の口([`crate::window_input`] の `FileDropped`)が運ぶ。
//! この widget は「いまどこに落ちるか・受け入れられるか」を**見せる**係で、
//! enter / leave を消費側へ伝える(受入判定 `accepting` の正本は消費側)。
//!
//! 中身([`iced::Element`])を1枚で包む — [`crate::window_input`] と同じ形の
//! 透過 wrapper で、絵は中身の上に縁を重ねるだけ。

use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{layout, mouse, overlay, renderer, Layout, Shell, Widget};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::widgets::palette::PALETTE;

/// hover 縁の太さ。
const EDGE_W: f32 = 2.0;

/// drop 面の語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropEvent {
    /// cursor が面に入った。
    HoverEnter,
    /// cursor が面から出た(窓から出た時も含む)。
    HoverLeave,
}

/// `inner` を drop 受け皿で包む。
pub fn drop_zone<'a, M>(
    inner: iced::Element<'a, M>,
    accepting: bool,
    on_event: impl Fn(DropEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    Element::new(DropZone {
        content: inner,
        accepting,
        on_event: Box::new(on_event),
    })
}

struct DropZone<'a, M> {
    content: Element<'a, M>,
    accepting: bool,
    on_event: Box<dyn Fn(DropEvent) -> M + 'a>,
}

/// 「いま面の上に居るか」。enter / leave は縁でだけ言うための記憶。
#[derive(Default)]
struct ZoneState {
    hovered: bool,
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for DropZone<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ZoneState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ZoneState::default())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
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
        self.content
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
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        // 中身が先(window_input と同じ順)。enter / leave の追跡は中身が
        // event を飲んでも続ける — 面の上に居る事実は奪い合いにならない。
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );

        let state: &mut ZoneState = tree.state.downcast_mut();
        let over = match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                layout.bounds().contains(*position)
            }
            Event::Mouse(mouse::Event::CursorLeft) => false,
            _ => return,
        };
        if over == state.hovered {
            return;
        }
        state.hovered = over;
        shell.publish((self.on_event)(if over {
            DropEvent::HoverEnter
        } else {
            DropEvent::HoverLeave
        }));
        shell.request_redraw();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
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
        use iced::advanced::Renderer as _;

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        // Q3: hover 中だけ縁が点く。受け入れるなら accent、受け入れないなら
        // 灰の縁(可否の表示)。hover していない面に飾りは出さない。
        let state: &ZoneState = tree.state.downcast_ref();
        if !state.hovered {
            return;
        }
        let (edge, wash) = if self.accepting {
            (
                PALETTE.accent,
                iced::Color {
                    a: 0.08,
                    ..PALETTE.accent
                },
            )
        } else {
            (PALETTE.outline, iced::Color::TRANSPARENT)
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: iced::Border {
                    color: edge,
                    width: EDGE_W,
                    radius: 4.0.into(),
                },
                ..renderer::Quad::default()
            },
            wash,
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
