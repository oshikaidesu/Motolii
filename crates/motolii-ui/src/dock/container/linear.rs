// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/container/linear.rs`.
//
// なぜほぼ無改変か: 移植元の egui 依存は 28 行(4%)で、そのほとんどが `ui()` の描画と掴み判定。
// `layout` と share の分配、`resize_interaction` / `shrink_shares` / `drop_zones` は写しである。

use std::collections::{HashMap, HashSet};

use super::super::behavior::{EditAction, LayoutContext};
use super::super::geom::{lerp, pos2, vec2, Rect};
use super::super::interaction::{
    splitter_response, InteractContext, ResizeHandle, SplitterId, SplitterKind, SplitterResponse,
};
use super::super::{
    Behavior, ContainerInsertion, InsertionPoint, ResizeState, SimplifyAction, TileId, Tiles, Tree,
};

// ----------------------------------------------------------------------------

/// How large of a share of space each child has, on a 1D axis.
///
/// Used for [`Linear`] containers (horizontal and vertical).
///
/// Also contains the shares for currently invisible tiles.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Shares {
    /// How large of a share each child has.
    ///
    /// For instance, the shares `[1, 2, 3]` means that the first child gets 1/6 of the space,
    /// the second gets 2/6 and the third gets 3/6.
    shares: HashMap<TileId, f32>,
}

impl Shares {
    pub fn iter(&self) -> impl Iterator<Item = (&TileId, &f32)> {
        self.shares.iter()
    }

    pub fn replace_with(&mut self, remove: TileId, new: TileId) {
        if let Some(share) = self.shares.remove(&remove) {
            self.shares.insert(new, share);
        }
    }

    pub fn set_share(&mut self, id: TileId, share: f32) {
        self.shares.insert(id, share);
    }

    /// Split the given width based on the share of the children.
    pub fn split(&self, children: &[TileId], available_width: f32) -> Vec<f32> {
        let mut num_shares = 0.0;
        for &child in children {
            num_shares += self[child];
        }
        if num_shares == 0.0 {
            num_shares = 1.0;
        }
        children
            .iter()
            .map(|&child| available_width * self[child] / num_shares)
            .collect()
    }

    pub fn retain(&mut self, keep: impl Fn(TileId) -> bool) {
        self.shares.retain(|&child, _| keep(child));
    }
}

impl<'a> IntoIterator for &'a Shares {
    type Item = (&'a TileId, &'a f32);
    type IntoIter = std::collections::hash_map::Iter<'a, TileId, f32>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.shares.iter()
    }
}

impl std::ops::Index<TileId> for Shares {
    type Output = f32;

    #[inline]
    fn index(&self, id: TileId) -> &Self::Output {
        self.shares.get(&id).unwrap_or(&1.0)
    }
}

impl std::ops::IndexMut<TileId> for Shares {
    #[inline]
    fn index_mut(&mut self, id: TileId) -> &mut Self::Output {
        self.shares.entry(id).or_insert(1.0)
    }
}

// ----------------------------------------------------------------------------

/// The direction of a [`Linear`] container. Either horizontal or vertical.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize,
)]
pub enum LinearDir {
    #[default]
    Horizontal,
    Vertical,
}

/// Horizontal or vertical container.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Linear {
    pub children: Vec<TileId>,
    pub dir: LinearDir,
    pub shares: Shares,
}

impl Linear {
    pub fn new(dir: LinearDir, children: Vec<TileId>) -> Self {
        Self {
            children,
            dir,
            ..Default::default()
        }
    }

    fn visible_children<Pane>(&self, tiles: &Tiles<Pane>) -> Vec<TileId> {
        self.children
            .iter()
            .copied()
            .filter(|&child_id| tiles.is_visible(child_id))
            .collect()
    }

    /// Create a binary split with the given split ratio in the 0.0 - 1.0 range.
    ///
    /// The `fraction` is the fraction of the total width that the first child should get.
    pub fn new_binary(dir: LinearDir, children: [TileId; 2], fraction: f32) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&fraction),
            "Fraction should be in 0.0..=1.0"
        );
        let mut slf = Self {
            children: children.into(),
            dir,
            ..Default::default()
        };
        // We multiply the shares with 2.0 because the default share size is 1.0,
        // and so we want the total share to be the same as the number of children.
        slf.shares[children[0]] = 2.0 * (fraction);
        slf.shares[children[1]] = 2.0 * (1.0 - fraction);
        slf
    }

    /// Swap out one child for another, keeping its position and its share of the space.
    ///
    /// Returns the index of the child that was swapped,
    /// or `None` if `old` was not a child of this container.
    #[must_use]
    pub(super) fn replace_child(&mut self, old: TileId, new: TileId) -> Option<usize> {
        let index = self.children.iter().position(|child| *child == old)?;
        self.children[index] = new;
        self.shares.replace_with(old, new);
        Some(index)
    }

    pub fn add_child(&mut self, child: TileId) {
        self.children.push(child);
    }

    pub(super) fn layout<Pane>(
        &mut self,
        tiles: &mut Tiles<Pane>,
        layout: &LayoutContext<'_>,
        rect: Rect,
    ) {
        // GC:
        let child_set: HashSet<TileId> = self.children.iter().copied().collect();
        self.shares.retain(|id| child_set.contains(&id));

        match self.dir {
            LinearDir::Horizontal => {
                self.layout_horizontal(tiles, layout, rect);
            }
            LinearDir::Vertical => self.layout_vertical(tiles, layout, rect),
        }
    }

    fn layout_horizontal<Pane>(
        &self,
        tiles: &mut Tiles<Pane>,
        layout: &LayoutContext<'_>,
        rect: Rect,
    ) {
        let visible_children = self.visible_children(tiles);

        let num_gaps = visible_children.len().saturating_sub(1);
        let gap_width = layout.gap_width;
        let total_gap_width = gap_width * num_gaps as f32;
        let available_width = (rect.width() - total_gap_width).max(0.0);

        let widths = self.shares.split(&visible_children, available_width);

        let mut x = rect.min.x;
        for (child, width) in visible_children.iter().zip(widths) {
            let child_rect = Rect::from_min_size(pos2(x, rect.min.y), vec2(width, rect.height()));
            tiles.layout_tile(layout, child_rect, *child);
            x += width + gap_width;
        }
    }

    fn layout_vertical<Pane>(
        &self,
        tiles: &mut Tiles<Pane>,
        layout: &LayoutContext<'_>,
        rect: Rect,
    ) {
        let visible_children = self.visible_children(tiles);

        let num_gaps = visible_children.len().saturating_sub(1);
        let gap_height = layout.gap_width;
        let total_gap_height = gap_height * num_gaps as f32;
        let available_height = (rect.height() - total_gap_height).max(0.0);

        let heights = self.shares.split(&visible_children, available_height);

        let mut y = rect.min.y;
        for (child, height) in visible_children.iter().zip(heights) {
            let child_rect = Rect::from_min_size(pos2(rect.min.x, y), vec2(rect.width(), height));
            tiles.layout_tile(layout, child_rect, *child);
            y += height + gap_height;
        }
    }

    pub(super) fn interact<Pane>(
        &mut self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        tile_id: TileId,
    ) {
        match self.dir {
            LinearDir::Horizontal => self.horizontal_interact(tree, behavior, ctx, tile_id),
            LinearDir::Vertical => self.vertical_interact(tree, behavior, ctx, tile_id),
        }
    }

    fn horizontal_interact<Pane>(
        &mut self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        parent_id: TileId,
    ) {
        let visible_children = self.visible_children(&tree.tiles);

        for &child in &visible_children {
            tree.tile_interact(behavior, ctx, child);
        }

        linear_drop_zones(ctx, tree, &self.children, self.dir, parent_id);

        // ------------------------
        // resizing:

        let resizable = behavior.is_container_resizable(&tree.tiles, parent_id);

        let parent_rect = tree.tiles.rect_or_die(parent_id);
        for (i, pair) in visible_children.windows(2).enumerate() {
            let (left, right) = (pair[0], pair[1]);
            let splitter_id = SplitterId {
                parent_id,
                kind: SplitterKind::Linear,
                index: i,
            };

            let left_rect = tree.tiles.rect_or_die(left);
            let right_rect = tree.tiles.rect_or_die(right);
            let x = lerp(left_rect.right(), right_rect.left(), 0.5);

            let line_rect = Rect::from_center_size(
                pos2(x, parent_rect.center().y),
                vec2(
                    2.0 * behavior.resize_grab_radius_side(),
                    parent_rect.height(),
                ),
            );

            if !resizable {
                ctx.handles.push(ResizeHandle {
                    id: splitter_id,
                    rect: line_rect,
                    line: x,
                    is_vertical_line: true,
                    span: parent_rect.y_range(),
                    state: ResizeState::Idle,
                });
                continue;
            }

            let mut resize_state = ResizeState::Idle;
            let response = splitter_response(splitter_id, line_rect, ctx.input, ctx.drag);
            if let Some(pointer) = ctx.input.pointer_pos {
                resize_state = resize_interaction(
                    behavior,
                    &mut self.shares,
                    &visible_children,
                    &response,
                    [left, right],
                    pointer.round_to_pixels(ctx.input.pixels_per_point).x - x,
                    i,
                    |tile_id: TileId| tree.tiles.rect_or_die(tile_id).width(),
                );
            }

            ctx.handles.push(ResizeHandle {
                id: splitter_id,
                rect: line_rect,
                line: x,
                is_vertical_line: true,
                span: parent_rect.y_range(),
                state: resize_state,
            });
        }
    }

    fn vertical_interact<Pane>(
        &mut self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        parent_id: TileId,
    ) {
        let visible_children = self.visible_children(&tree.tiles);

        for &child in &visible_children {
            tree.tile_interact(behavior, ctx, child);
        }

        linear_drop_zones(ctx, tree, &self.children, self.dir, parent_id);

        // ------------------------
        // resizing:

        let resizable = behavior.is_container_resizable(&tree.tiles, parent_id);

        let parent_rect = tree.tiles.rect_or_die(parent_id);
        for (i, pair) in visible_children.windows(2).enumerate() {
            let (top, bottom) = (pair[0], pair[1]);
            let splitter_id = SplitterId {
                parent_id,
                kind: SplitterKind::Linear,
                index: i,
            };

            let top_rect = tree.tiles.rect_or_die(top);
            let bottom_rect = tree.tiles.rect_or_die(bottom);
            let y = lerp(top_rect.bottom(), bottom_rect.top(), 0.5);

            let line_rect = Rect::from_center_size(
                pos2(parent_rect.center().x, y),
                vec2(
                    parent_rect.width(),
                    2.0 * behavior.resize_grab_radius_side(),
                ),
            );

            if !resizable {
                ctx.handles.push(ResizeHandle {
                    id: splitter_id,
                    rect: line_rect,
                    line: y,
                    is_vertical_line: false,
                    span: parent_rect.x_range(),
                    state: ResizeState::Idle,
                });
                continue;
            }

            let mut resize_state = ResizeState::Idle;
            let response = splitter_response(splitter_id, line_rect, ctx.input, ctx.drag);
            if let Some(pointer) = ctx.input.pointer_pos {
                resize_state = resize_interaction(
                    behavior,
                    &mut self.shares,
                    &visible_children,
                    &response,
                    [top, bottom],
                    pointer.round_to_pixels(ctx.input.pixels_per_point).y - y,
                    i,
                    |tile_id: TileId| tree.tiles.rect_or_die(tile_id).height(),
                );
            }

            ctx.handles.push(ResizeHandle {
                id: splitter_id,
                rect: line_rect,
                line: y,
                is_vertical_line: false,
                span: parent_rect.x_range(),
                state: resize_state,
            });
        }
    }

    pub(super) fn simplify_children(&mut self, mut simplify: impl FnMut(TileId) -> SimplifyAction) {
        self.children.retain_mut(|child| match simplify(*child) {
            SimplifyAction::Remove => false,
            SimplifyAction::Keep => true,
            SimplifyAction::Replace(new) => {
                self.shares.replace_with(*child, new);
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

#[expect(clippy::too_many_arguments)]
fn resize_interaction<Pane>(
    behavior: &mut dyn Behavior<Pane>,
    shares: &mut Shares,
    children: &[TileId],
    splitter_response: &SplitterResponse,
    [left, right]: [TileId; 2],
    dx: f32,
    i: usize,
    tile_width: impl Fn(TileId) -> f32,
) -> ResizeState {
    if splitter_response.double_clicked() {
        behavior.on_edit(EditAction::TileResized);

        // double-click to center the split between left and right:
        let mean = 0.5 * (shares[left] + shares[right]);
        shares[left] = mean;
        shares[right] = mean;
        ResizeState::Hovering
    } else if splitter_response.dragged() {
        behavior.on_edit(EditAction::TileResized);

        if dx < 0.0 {
            // Expand right, shrink stuff to the left:
            shares[right] += shrink_shares(
                behavior,
                shares,
                &children[0..=i].iter().copied().rev().collect::<Vec<_>>(),
                dx.abs(),
                tile_width,
            );
        } else {
            // Expand the left, shrink stuff to the right:
            shares[left] +=
                shrink_shares(behavior, shares, &children[i + 1..], dx.abs(), tile_width);
        }
        ResizeState::Dragging
    } else if splitter_response.hovered() {
        ResizeState::Hovering
    } else {
        ResizeState::Idle
    }
}

/// Try shrink the children by a total of `target_in_points`,
/// making sure no child gets smaller than its minimum size.
fn shrink_shares<Pane>(
    behavior: &dyn Behavior<Pane>,
    shares: &mut Shares,
    children: &[TileId],
    target_in_points: f32,
    size_in_point: impl Fn(TileId) -> f32,
) -> f32 {
    if children.is_empty() {
        return 0.0;
    }

    let mut total_shares = 0.0;
    let mut total_points = 0.0;
    for &child in children {
        total_shares += shares[child];
        total_points += size_in_point(child);
    }

    let shares_per_point = total_shares / total_points;

    let min_size_in_shares = shares_per_point * behavior.min_size();

    let target_in_shares = shares_per_point * target_in_points;
    let mut total_shares_lost = 0.0;

    for &child in children {
        let share = &mut shares[child];
        let spare_share = (*share - min_size_in_shares).max(0.0);
        let shares_needed = (target_in_shares - total_shares_lost).max(0.0);
        let shrink_by = f32::min(spare_share, shares_needed);

        *share -= shrink_by;
        total_shares_lost += shrink_by;
    }

    total_shares_lost
}

fn linear_drop_zones<Pane>(
    ctx: &mut InteractContext<'_>,
    tree: &Tree<Pane>,
    children: &[TileId],
    dir: LinearDir,
    parent_id: TileId,
) {
    let preview_thickness = 12.0;
    let dragged_index = children
        .iter()
        .position(|&child| ctx.drag.is_being_dragged(child));

    let after_rect = |rect: Rect| match dir {
        LinearDir::Horizontal => Rect::from_min_max(
            rect.right_top() - vec2(preview_thickness, 0.0),
            rect.right_bottom(),
        ),
        LinearDir::Vertical => Rect::from_min_max(
            rect.left_bottom() - vec2(0.0, preview_thickness),
            rect.right_bottom(),
        ),
    };

    let insertion = |i: usize| match dir {
        LinearDir::Horizontal => ContainerInsertion::Horizontal(i),
        LinearDir::Vertical => ContainerInsertion::Vertical(i),
    };

    let drop = &mut *ctx.drop;
    drop_zones(
        preview_thickness,
        children,
        dragged_index,
        dir,
        |tile_id| tree.tiles.rect(tile_id),
        |rect, i| {
            drop.suggest_rect(InsertionPoint::new(parent_id, insertion(i)), rect);
        },
        after_rect,
    );
}

/// Register drop-zones for a linear container.
///
/// `get_rect`: return `None` for invisible tiles.
pub(super) fn drop_zones(
    preview_thickness: f32,
    children: &[TileId],
    dragged_index: Option<usize>,
    dir: LinearDir,
    get_rect: impl Fn(TileId) -> Option<Rect>,
    mut add_drop_drect: impl FnMut(Rect, usize),
    after_rect: impl Fn(Rect) -> Rect,
) {
    let before_rect = |rect: Rect| match dir {
        LinearDir::Horizontal => Rect::from_min_max(
            rect.left_top(),
            rect.left_bottom() + vec2(preview_thickness, 0.0),
        ),
        LinearDir::Vertical => Rect::from_min_max(
            rect.left_top(),
            rect.right_top() + vec2(0.0, preview_thickness),
        ),
    };
    let between_rects = |a: Rect, b: Rect| match dir {
        LinearDir::Horizontal => Rect::from_center_size(
            a.right_center().lerp(b.left_center(), 0.5),
            vec2(preview_thickness, a.height()),
        ),
        LinearDir::Vertical => Rect::from_center_size(
            a.center_bottom().lerp(b.center_top(), 0.5),
            vec2(a.width(), preview_thickness),
        ),
    };

    let mut prev_rect: Option<Rect> = None;

    for (i, &child) in children.iter().enumerate() {
        let Some(rect) = get_rect(child) else {
            // skip invisible child
            continue;
        };

        if Some(i) == dragged_index {
            // Suggest hole as a drop-target:
            add_drop_drect(rect, i);
        } else if let Some(prev_rect) = prev_rect {
            if Some(i - 1) != dragged_index {
                // Suggest dropping between the rects:
                add_drop_drect(between_rects(prev_rect, rect), i);
            }
        } else {
            // Suggest dropping before the first child:
            add_drop_drect(before_rect(rect), 0);
        }

        prev_rect = Some(rect);
    }

    if let Some(last_rect) = prev_rect {
        // Suggest dropping after the last child (unless that's the one being dragged):
        if dragged_index != Some(children.len() - 1) {
            add_drop_drect(after_rect(last_rect), children.len());
        }
    }
}
