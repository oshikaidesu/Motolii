//! `TaffyBox` — taffy に layout を委譲する iced container widget。
//!
//! 構造は fork の `iced_widget::Row` を鏡写しにしている(diff / operate / update /
//! mouse_interaction / draw / overlay の子委譲)。`layout` だけが Row と違い、
//! `layout::flex::resolve` の代わりに taffy の `compute_layout_with_measure` を使う:
//! 子 widget の intrinsic 測定は measure 閉包から `Widget::layout` を呼んで得る
//! (iced_taffy / egui_taffy と同型のパターン。probe:
//! docs/reviews/2026-08-22-weblike-layout-survey.md §2)。
//!
//! 既知の近似(probe と同一・調査 §2 リスク欄):
//! - measure 中の `AvailableSpace::MinContent/MaxContent` は親の limits.max へ潰す。
//!   折返しテキストを多用する面では要検分。
//! - taffy ツリーは layout のたびに構築する(pane 規模なら µs 桁 — 調査 §2)。
//!   `taffy::Cache` の持ち込みは実測で律速になってから。

use iced_core::layout::{Layout, Limits, Node};
use iced_core::mouse;
use iced_core::overlay;
use iced_core::renderer;
use iced_core::widget::{Operation, Tree};
use iced_core::{Element, Event, Length, Point, Rectangle, Shell, Size, Vector};

/// 子 Element 群を `taffy::Style`(flex/grid)で並べる container widget。
///
/// - 自身の style(`display`・`justify-content`・`gap`・`padding`・
///   `grid-template-*` …)は [`TaffyBox::new`] で受ける。
/// - 子ごとの style(`width`/`height`・`flex-grow`・`flex-basis` …)は
///   [`TaffyBox::push`] で子と対にして受ける。
///
/// CSS 字面から style を作るには [`crate::style_from_css_decl`] を使う。
pub struct TaffyBox<'a, Message, Theme, Renderer> {
    style: taffy::Style,
    child_styles: Vec<taffy::Style>,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> TaffyBox<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// container 自身の `taffy::Style` から空の [`TaffyBox`] を作る。
    pub fn new(style: taffy::Style) -> Self {
        Self {
            style,
            child_styles: Vec::new(),
            children: Vec::new(),
        }
    }

    /// 子を、その子自身の `taffy::Style` と対にして追加する。
    pub fn push(
        mut self,
        style: taffy::Style,
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.child_styles.push(style);
        self.children.push(child.into());
        self
    }
}

impl<Message, Theme, Renderer> iced_core::Widget<Message, Theme, Renderer>
    for TaffyBox<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut self.children);
    }

    fn size(&self) -> Size<Length> {
        // 親(iced 素の flex)への寸法ヒント。実寸は taffy 側 style が決める —
        // pane root として枠いっぱいを解くのが本用途なので Fill を申告する。
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let mut taffy = taffy::TaffyTree::<usize>::new();

        let kids: Vec<taffy::NodeId> = self
            .child_styles
            .iter()
            .enumerate()
            .map(|(i, s)| {
                taffy
                    .new_leaf_with_context(s.clone(), i)
                    .expect("taffy leaf 構築は失敗しない")
            })
            .collect();

        let max = limits.max();
        let min = limits.min();

        // 軸が tight(min == max — 親が寸法を確定済み: 入れ子 TaffyBox の最終確定
        // pass や、iced 素の flex が Fill 子へ枠を割り当てた場合)なら、auto の
        // root 寸法をその確定値へ差し替えて解く。こうしないと taffy は auto root を
        // content 寸法(fit-content)で解いてしまい、space-between 等の配分が
        // 親のくれた枠でなく content 幅を分母にしてしまう。
        // 軸が loose なら CSS どおり: style の寸法が勝ち、auto は content 寸法。
        let mut root_style = self.style.clone();
        if min.width == max.width && root_style.size.width == taffy::Dimension::auto() {
            root_style.size.width = taffy::Dimension::length(max.width);
        }
        if min.height == max.height && root_style.size.height == taffy::Dimension::auto() {
            root_style.size.height = taffy::Dimension::length(max.height);
        }

        let root = taffy
            .new_with_children(root_style, &kids)
            .expect("taffy root 構築は失敗しない");
        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(max.width),
            height: taffy::AvailableSpace::Definite(max.height),
        };

        {
            let children = &mut self.children;
            let trees = &mut tree.children;
            taffy
                .compute_layout_with_measure(
                    root,
                    available,
                    |known, space, _id, ctx: Option<&mut usize>, _style| {
                        // context 無し = 子 index を持たないノード(root)。測定不要。
                        let Some(&mut i) = ctx else {
                            return taffy::Size::ZERO;
                        };
                        // MinContent/MaxContent は親 limits の max へ潰す近似
                        // (probe 同様 — モジュール冒頭の既知の近似を参照)。
                        let w = known.width.unwrap_or(match space.width {
                            taffy::AvailableSpace::Definite(w) => w,
                            _ => max.width,
                        });
                        let h = known.height.unwrap_or(match space.height {
                            taffy::AvailableSpace::Definite(h) => h,
                            _ => max.height,
                        });
                        let child_limits = Limits::new(Size::ZERO, Size::new(w, h));
                        let size = children[i]
                            .as_widget_mut()
                            .layout(&mut trees[i], renderer, &child_limits)
                            .size();
                        taffy::Size {
                            width: size.width,
                            height: size.height,
                        }
                    },
                )
                .expect("taffy compute_layout は測定閉包がある限り失敗しない");
        }

        // 最終確定: taffy が解いた矩形で子をもう一度 layout し、位置を写す。
        // taffy::Layout{location,size} と iced の Node 座標系は無変換で一致(probe 実測)。
        let children = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(&kids)
            .map(|((child, tree), kid)| {
                let solved = taffy.layout(*kid).expect("解いた子の layout は必ず在る");
                let child_limits = Limits::new(
                    Size::new(solved.size.width, solved.size.height),
                    Size::new(solved.size.width, solved.size.height),
                );
                child
                    .as_widget_mut()
                    .layout(tree, renderer, &child_limits)
                    .move_to(Point::new(solved.location.x, solved.location.y))
            })
            .collect();

        let solved_root = taffy.layout(root).expect("解いた root の layout は必ず在る");
        Node::with_children(
            Size::new(solved_root.size.width, solved_root.size.height),
            children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(tree, event, layout, cursor, renderer, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<TaffyBox<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(widget: TaffyBox<'a, Message, Theme, Renderer>) -> Self {
        Element::new(widget)
    }
}
