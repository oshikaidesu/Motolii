// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/container/tabs.rs`.
//
// なぜ差分が大きいか: 移植元の `tab_bar_ui` は `egui::ScrollArea` と `Button` そのものであり、
// 「書き直す」層に当たる。タブ帯の**幅の実測**は `Behavior::tab_width` へ外出しし、
// `tab_rects` を `Grid::col_ranges` と同じ扱いのレイアウト成果として持つ。
// タブ帯のスクロール(`ScrollState` と左右の矢印)は egui widget そのものなので**未移植**。

use super::super::behavior::{EditAction, LayoutContext, TabState};
use super::super::geom::{pos2, vec2, Rect};
use super::super::interaction::InteractContext;
use super::super::{
    Behavior, ContainerInsertion, InsertionPoint, SimplifyAction, TileId, Tiles, Tree,
};

/// A container with tabs. Only one tab is open (active) at a time.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Tabs {
    /// The tabs, in order.
    pub children: Vec<TileId>,

    /// The currently open tab.
    pub active: Option<TileId>,

    /// Where each visible tab button sits, recomputed during layout.
    #[serde(skip)]
    tab_rects: Vec<(TileId, Rect)>,
}

impl PartialEq for Tabs {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            children,
            active,
            tab_rects: _, // ignored because they are recomputed each frame
        } = self;

        children == &other.children && active == &other.active
    }
}

impl Tabs {
    pub fn new(children: Vec<TileId>) -> Self {
        let active = children.first().copied();
        Self {
            children,
            active,
            tab_rects: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: TileId) {
        self.children.push(child);
    }

    /// Swap out one tab for another, keeping its position and whether it was the open one.
    ///
    /// Returns the index of the tab that was swapped,
    /// or `None` if `old` was not a tab of this container.
    #[must_use]
    pub(super) fn replace_child(&mut self, old: TileId, new: TileId) -> Option<usize> {
        let index = self.children.iter().position(|child| *child == old)?;
        self.children[index] = new;
        if self.active == Some(old) {
            self.active = Some(new);
        }
        Some(index)
    }

    pub fn set_active(&mut self, child: TileId) {
        self.active = Some(child);
    }

    pub fn is_active(&self, child: TileId) -> bool {
        Some(child) == self.active
    }

    /// Where each visible tab button sits, as of the last layout pass.
    pub fn tab_rects(&self) -> &[(TileId, Rect)] {
        &self.tab_rects
    }

    pub(super) fn layout<Pane>(
        &mut self,
        tiles: &mut Tiles<Pane>,
        layout: &LayoutContext<'_>,
        rect: Rect,
    ) {
        let prev_active = self.active;
        self.ensure_active(tiles);
        if prev_active != self.active {
            layout.tab_auto_selected.set(true);
        }

        // Tab buttons run left-to-right with no spacing between them, as upstream's tab flow does
        // (`ui.spacing_mut().item_spacing.x = 0.0`).
        let tab_bar_rect = rect
            .split_top_bottom_at_y(rect.top() + layout.tab_bar_height)
            .0;
        self.tab_rects.clear();
        let mut x = tab_bar_rect.left();
        for &child_id in &self.children {
            if !tiles.is_visible(child_id) {
                continue;
            }
            let width = (layout.tab_width)(child_id);
            self.tab_rects.push((
                child_id,
                Rect::from_min_size(
                    pos2(x, tab_bar_rect.top()),
                    vec2(width, tab_bar_rect.height()),
                ),
            ));
            x += width;
        }

        let mut active_rect = rect;
        active_rect.min.y += layout.tab_bar_height;

        if let Some(active) = self.active {
            // Only lay out the active tab (saves CPU):
            tiles.layout_tile(layout, active_rect, active);
        }
    }

    pub fn next_active<Pane>(&self, tiles: &Tiles<Pane>) -> Option<TileId> {
        self.active
            .filter(|active| self.children.contains(active) && tiles.is_visible(*active))
            .or_else(|| {
                self.children
                    .iter()
                    .copied()
                    .find(|&child_id| tiles.is_visible(child_id))
            })
    }

    /// Make sure we have an active tab (or no visible tabs).
    pub fn ensure_active<Pane>(&mut self, tiles: &Tiles<Pane>) {
        self.active = self.next_active(tiles);
    }

    pub(super) fn interact<Pane>(
        &mut self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        rect: Rect,
        tile_id: TileId,
    ) {
        let next_active = self.tab_bar_interact(tree, behavior, ctx, rect, tile_id);

        if let Some(active) = self.active {
            tree.tile_interact(behavior, ctx, active);
        }

        // We have only laid out the active tab, so we need to switch active tab _after_ the pass above:
        self.active = next_active;
    }

    /// Returns the next active tab (e.g. the one clicked, or the current).
    fn tab_bar_interact<Pane>(
        &self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        rect: Rect,
        tile_id: TileId,
    ) -> Option<TileId> {
        let mut next_active = self.active;

        let tab_bar_height = behavior.tab_bar_height();
        let tab_bar_rect = rect.split_top_bottom_at_y(rect.top() + tab_bar_height).0;

        let mut dragged_index = None;

        let on_a_tab = |pos| self.tab_rects.iter().any(|(_, r)| r.contains(pos));

        // The background behind the buttons is draggable (to drag the parent container tile).
        if !tree.is_root(tile_id)
            && behavior.is_tile_draggable(&tree.tiles, tile_id)
            && ctx.drag.dragged_tile.is_none()
            && ctx.input.pointer_down
        {
            if let Some(origin) = ctx.drag.press_origin {
                if tab_bar_rect.contains(origin)
                    && !on_a_tab(origin)
                    && ctx
                        .input
                        .pointer_pos
                        .is_some_and(|p| p.distance(origin) > behavior.drag_start_distance())
                {
                    behavior.on_edit(EditAction::TileDragged);
                    ctx.drag.dragged_tile = Some(tile_id);
                }
            }
        }

        for (i, &child_id) in self.children.iter().enumerate() {
            if !tree.is_visible(child_id) {
                continue;
            }

            let Some(&(_, tab_rect)) = self
                .tab_rects
                .iter()
                .find(|(rect_id, _)| *rect_id == child_id)
            else {
                continue;
            };

            let is_being_dragged = ctx.drag.is_being_dragged(child_id);

            let _tab_state = TabState {
                active: self.is_active(child_id),
                is_being_dragged,
                closable: behavior.is_tab_closable(&tree.tiles, child_id),
            };

            let clicked = ctx.input.pointer_released
                && ctx.drag.dragged_tile.is_none()
                && ctx.drag.press_origin.is_some_and(|p| tab_rect.contains(p))
                && ctx.input.pointer_pos.is_some_and(|p| tab_rect.contains(p));

            if clicked {
                behavior.on_edit(EditAction::TabSelected);
                next_active = Some(child_id);
            }

            // A press that turns into a movement drags the tab (upstream: `Sense::click_and_drag`).
            if ctx.drag.dragged_tile.is_none()
                && ctx.input.pointer_down
                && behavior.is_tile_draggable(&tree.tiles, child_id)
            {
                if let Some(origin) = ctx.drag.press_origin {
                    if tab_rect.contains(origin)
                        && ctx
                            .input
                            .pointer_pos
                            .is_some_and(|p| p.distance(origin) > behavior.drag_start_distance())
                    {
                        behavior.on_edit(EditAction::TileDragged);
                        ctx.drag.dragged_tile = Some(child_id);
                    }
                }
            }

            if let Some(mouse_pos) = ctx.drop.mouse_pos {
                if ctx.drop.dragged_tile_id.is_some() && tab_rect.contains(mouse_pos) {
                    // Expand this tab - maybe the user wants to drop something into it!
                    behavior.on_edit(EditAction::TabSelected);
                    next_active = Some(child_id);
                }
            }

            if is_being_dragged {
                dragged_index = Some(i);
            }
        }

        // -----------
        // Drop zones:

        let preview_thickness = 6.0;
        let tab_rects = &self.tab_rects;
        let children = &self.children;
        let after_rect = |rect: Rect| {
            let dragged_size = if let Some(dragged_index) = dragged_index {
                // We actually know the size of this thing
                tab_rects
                    .iter()
                    .find(|(id, _)| *id == children[dragged_index])
                    .map_or_else(|| rect.size(), |(_, r)| r.size())
            } else {
                rect.size() // guess that the size is the same as the last button
            };
            Rect::from_min_size(rect.right_top(), dragged_size)
        };

        let drop = &mut *ctx.drop;
        super::linear::drop_zones(
            preview_thickness,
            &self.children,
            dragged_index,
            super::LinearDir::Horizontal,
            |needle| {
                tab_rects
                    .iter()
                    .find(|(id, _)| *id == needle)
                    .map(|(_, r)| *r)
            },
            |rect, i| {
                drop.suggest_rect(
                    InsertionPoint::new(tile_id, ContainerInsertion::Tabs(i)),
                    rect,
                );
            },
            after_rect,
        );

        next_active
    }

    pub(super) fn simplify_children(&mut self, mut simplify: impl FnMut(TileId) -> SimplifyAction) {
        self.children.retain_mut(|child| match simplify(*child) {
            SimplifyAction::Remove => {
                // The tab being removed may be the open one, and this is the only place that
                // still knows it happened. The `Replace` arm below already carries `active`
                // across; leaving it out here means `simplify` can return a container whose open
                // tab is a tile that no longer exists anywhere in the tree.
                //
                // `None` rather than "the next tab": which tab to open instead is a question for
                // whoever shows the container (`ensure_active` answers it at layout time from
                // what is visible), while "the open tab is gone" is a fact this pass knows.
                if self.active == Some(*child) {
                    self.active = None;
                }
                false
            }
            SimplifyAction::Keep => true,
            SimplifyAction::Replace(new) => {
                if self.active == Some(*child) {
                    self.active = Some(new);
                }
                *child = new;
                true
            }
        });
    }

    /// Returns child index, if found.
    pub(crate) fn remove_child(&mut self, needle: TileId) -> Option<usize> {
        let index = self.children.iter().position(|&child| child == needle)?;
        self.children.remove(index);
        Some(index)
    }
}
