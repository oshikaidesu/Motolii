//! 右クリックメニュー — overlay。項目を選ぶか、外を押す / Esc で消える。
//!
//! 出す・消すの判断は呼び出し側が持つ(このメニューは自分を閉じない —
//! [`MenuEvent::Chosen`] / [`MenuEvent::Dismissed`] を言うだけで、view から
//! 外すのは消費側の update である)。
//!
//! 実装は iced の [`iced::advanced::Overlay`] — 木の上に立ち、窓端では
//! はみ出さないよう内側へ寄る。項目の当たりは [`item_position`] が公開する
//! 幾何と同じ式で、テストと消費側が同じ座標を見る。

use iced::advanced::text::{Paragraph as _, Renderer as _};
use iced::advanced::widget::Tree;
use iced::advanced::{layout, mouse, overlay, renderer, text, Layout, Shell, Widget};
use iced::{keyboard, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector};

use crate::theme::{self, Tokens};

/// メニューの1項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// 選ばれた時に [`MenuEvent::Chosen`] が運ぶ id。
    pub id: u32,
    /// 表示する字。
    pub label: String,
    /// false なら「今この文脈で無効」(Q0 — 未実装の飾りには使わない)。
    pub enabled: bool,
}

/// メニューの語彙。**この enum が公開契約**(消費側 capsule と同文)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// 有効な項目が選ばれた。
    Chosen(u32),
    /// 外を押した、または Esc。
    Dismissed,
}

/// 1項目の高さ。
pub const ITEM_H: f32 = 22.0;
/// メニューの上下 padding。
pub const MENU_PAD_Y: f32 = 4.0;
/// 項目の左右 padding。
pub const ITEM_PAD_X: f32 = 12.0;
/// メニューの最小幅。
pub const MENU_MIN_W: f32 = 120.0;
/// 字の大きさ。
const FONT_SIZE: f32 = 11.0;

/// `index` 番目の項目の当たり中心(メニューが `at` に出た時)。
/// テストと消費側が同じ幾何を見るための口。窓端 clamp が効いた場合はずれる。
pub fn item_position(at: iced::Point, index: usize) -> iced::Point {
    iced::Point::new(
        at.x + ITEM_PAD_X,
        at.y + MENU_PAD_Y + ITEM_H * (index as f32 + 0.5),
    )
}

/// メニューを1つ組む。`at` は窓座標(右クリック位置)。
pub fn context_menu<'a, M>(
    items: Vec<MenuItem>,
    at: iced::Point,
    on_event: impl Fn(MenuEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    Element::new(ContextMenu {
        items,
        at,
        on_event: Box::new(on_event),
    })
}

struct ContextMenu<'a, M> {
    items: Vec<MenuItem>,
    at: Point,
    on_event: Box<dyn Fn(MenuEvent) -> M + 'a>,
}

impl<M> Widget<M, iced::Theme, iced::Renderer> for ContextMenu<'_, M> {
    /// 木の中では場所を取らない — 絵も当たりも overlay 側が持つ。
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(0.0), Length::Fixed(0.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::ZERO)
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &iced::Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        Some(overlay::Element::new(Box::new(MenuOverlay {
            items: &self.items,
            on_event: self.on_event.as_ref(),
            position: self.at + translation,
        })))
    }
}

struct MenuOverlay<'a, M> {
    items: &'a [MenuItem],
    on_event: &'a dyn Fn(MenuEvent) -> M,
    position: Point,
}

impl<M> MenuOverlay<'_, M> {
    /// `position` に居る項目の番号(padding の余白は項目ではない)。
    fn item_at(&self, bounds: Rectangle, position: Point) -> Option<usize> {
        if !bounds.contains(position) {
            return None;
        }
        let y = position.y - bounds.y - MENU_PAD_Y;
        if y < 0.0 {
            return None;
        }
        let index = (y / ITEM_H) as usize;
        (index < self.items.len()).then_some(index)
    }

    fn row(&self, bounds: Rectangle, index: usize) -> Rectangle {
        Rectangle {
            x: bounds.x,
            y: bounds.y + MENU_PAD_Y + ITEM_H * index as f32,
            width: bounds.width,
            height: ITEM_H,
        }
    }
}

impl<M> overlay::Overlay<M, iced::Theme, iced::Renderer> for MenuOverlay<'_, M> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        // 幅は一番長い label に合わせる(切れた字の menu は読めない)。
        let mut width = MENU_MIN_W;
        for item in self.items {
            let paragraph = <iced::Renderer as text::Renderer>::Paragraph::with_text(text::Text {
                content: item.label.as_str(),
                bounds: Size::INFINITE,
                size: Pixels(FONT_SIZE),
                line_height: text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: text::Alignment::Left,
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: None,
            });
            width = width.max(paragraph.min_bounds().width + ITEM_PAD_X * 2.0);
        }
        let size = Size::new(width, MENU_PAD_Y * 2.0 + ITEM_H * self.items.len() as f32);

        // 窓端でははみ出さず内側へ寄る(読めない menu を出さない)。
        let position = Point::new(
            self.position.x.min(bounds.width - size.width).max(0.0),
            self.position.y.min(bounds.height - size.height).max(0.0),
        );
        layout::Node::new(size).move_to(position)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        shell: &mut Shell<'_, M>,
    ) {
        let bounds = layout.bounds();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(position) = cursor.position() else {
                    return;
                };
                if let Some(index) = self.item_at(bounds, position) {
                    let item = &self.items[index];
                    if item.enabled {
                        shell.publish((self.on_event)(MenuEvent::Chosen(item.id)));
                    }
                    // 無効な項目は黙る(拒否は灰の字と NotAllowed cursor が言う)。
                    shell.capture_event();
                } else if bounds.contains(position) {
                    // メニューの余白 — 何も選ばれないが、下の面へも通さない。
                    shell.capture_event();
                } else {
                    // 外を押した = 閉じる合図。押し自体も menu が飲む
                    // (閉じるためのクリックが下の面で誤発火しない)。
                    shell.publish((self.on_event)(MenuEvent::Dismissed));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // hover の帯は draw が cursor から導く — 動いたら描き直すだけでよい。
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                shell.publish((self.on_event)(MenuEvent::Dismissed));
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let Some(position) = cursor.position() else {
            return mouse::Interaction::None;
        };
        match self.item_at(bounds, position) {
            Some(index) if self.items[index].enabled => mouse::Interaction::Pointer,
            // 無効な項目の上では「押せない」と cursor が言う(Q3 拒否の報酬)。
            Some(_) => mouse::Interaction::NotAllowed,
            None => mouse::Interaction::None,
        }
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme_in: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;

        let t = Tokens::resolve(theme_in);
        let bounds = layout.bounds();
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: t.border_default,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..renderer::Quad::default()
            },
            t.surface_panel,
        );

        for (index, item) in self.items.iter().enumerate() {
            let row = self.row(bounds, index);
            // Q3 接触の報酬: 有効な項目は hover で帯が点く。
            if item.enabled && cursor.is_over(row) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: row,
                        ..renderer::Quad::default()
                    },
                    theme::mix(t.action_active, 20.0, t.surface_panel),
                );
            }
            renderer.fill_text(
                text::Text {
                    content: item.label.clone(),
                    bounds: Size::new(row.width - ITEM_PAD_X * 2.0, row.height),
                    size: Pixels(FONT_SIZE),
                    line_height: text::LineHeight::default(),
                    font: renderer.default_font(),
                    align_x: text::Alignment::Left,
                    align_y: iced::alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::None,
                    hint_factor: None,
                },
                Point::new(row.x + ITEM_PAD_X, row.center_y()),
                if item.enabled {
                    t.text_primary
                } else {
                    // 「今この文脈で無効」は灰で言う(Q0 — 死に見た目ではなく文脈 disabled)。
                    t.text_secondary
                },
                row,
            );
        }
    }
}
