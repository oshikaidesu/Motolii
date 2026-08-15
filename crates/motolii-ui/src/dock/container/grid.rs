// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/container/grid.rs`.
//
// なぜほぼ無改変か: 移植元の egui 依存は 32 行(4%)で、そのほとんどが `ui()` の描画と掴み判定。
// 格子の割り付け(`sizes_from_shares` / `col_ranges` / `row_ranges`)は写しである。

use super::super::behavior::{EditAction, LayoutContext};
use super::super::geom::{lerp, pos2, vec2, Rangef, Rect};
use super::super::interaction::{
    splitter_response, InteractContext, ResizeHandle, SplitterId, SplitterKind, SplitterResponse,
};
use super::super::{
    Behavior, ContainerInsertion, InsertionPoint, ResizeState, SimplifyAction, TileId, Tiles, Tree,
};

/// How to lay out the children of a grid.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum GridLayout {
    /// Place children in a grid, with a dynamic number of columns and rows.
    /// Resizing the window may change the number of columns and rows.
    #[default]
    Auto,

    /// Place children in a grid with this many columns,
    /// and as many rows as needed.
    Columns(usize),
}

/// A grid of tiles.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Grid {
    /// The order of the children, row-major.
    ///
    /// We allow holes (for easier drag-dropping).
    /// We collapse all holes if they become too numerous.
    pub(super) children: Vec<Option<TileId>>,

    /// Determines the number of columns.
    pub layout: GridLayout,

    /// Share of the available width assigned to each column.
    pub col_shares: Vec<f32>,

    /// Share of the available height assigned to each row.
    pub row_shares: Vec<f32>,

    /// ui point x ranges for each column, recomputed during layout
    #[serde(skip)]
    col_ranges: Vec<Rangef>,

    /// ui point y ranges for each row, recomputed during layout
    #[serde(skip)]
    row_ranges: Vec<Rangef>,
}

impl PartialEq for Grid {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            children,
            layout,
            col_shares,
            row_shares,
            col_ranges: _, // ignored because they are recomputed each frame
            row_ranges: _, // ignored because they are recomputed each frame
        } = self;

        layout == &other.layout
            && children == &other.children
            && col_shares == &other.col_shares
            && row_shares == &other.row_shares
    }
}

impl Grid {
    pub fn new(children: Vec<TileId>) -> Self {
        Self {
            children: children.into_iter().map(Some).collect(),
            ..Default::default()
        }
    }

    pub fn num_children(&self) -> usize {
        self.children().count()
    }

    /// Includes invisible children.
    pub fn children(&self) -> impl Iterator<Item = &TileId> {
        self.children.iter().filter_map(|c| c.as_ref())
    }

    pub fn add_child(&mut self, child: TileId) {
        self.children.push(Some(child));
    }

    pub fn insert_at(&mut self, index: usize, child: TileId) {
        if let Some(slot) = self.children.get_mut(index) {
            if slot.is_none() {
                // put it in the empty hole
                slot.replace(child);
            } else {
                // put it before
                self.children.insert(index, Some(child));
            }
        } else {
            // put it last
            self.children.push(Some(child));
        }
    }

    /// Returns the child already at the given index, if any.
    #[must_use]
    pub fn replace_at(&mut self, index: usize, child: TileId) -> Option<TileId> {
        if let Some(slot) = self.children.get_mut(index) {
            slot.replace(child)
        } else {
            // put it last
            self.children.push(Some(child));
            None
        }
    }

    /// Swap out one child for another, keeping its cell.
    ///
    /// The column and row shares are positional, so they need no fixing up.
    ///
    /// Returns the index of the cell that was swapped,
    /// or `None` if `old` was not a child of this grid.
    #[must_use]
    pub(super) fn replace_child(&mut self, old: TileId, new: TileId) -> Option<usize> {
        let index = self.children.iter().position(|child| *child == Some(old))?;
        self.children[index] = Some(new);
        Some(index)
    }

    fn collapse_holes(&mut self) {
        self.children.retain(|child| child.is_some());
    }

    pub(super) fn visible_children_and_holes<Pane>(
        &self,
        tiles: &Tiles<Pane>,
    ) -> Vec<Option<TileId>> {
        self.children
            .iter()
            .filter(|id| id.is_none_or(|id| tiles.is_visible(id)))
            .copied()
            .collect()
    }

    /// The x ranges of each column, as of the last layout pass.
    pub fn col_ranges(&self) -> &[Rangef] {
        &self.col_ranges
    }

    /// The y ranges of each row, as of the last layout pass.
    pub fn row_ranges(&self) -> &[Rangef] {
        &self.row_ranges
    }

    pub(super) fn layout<Pane>(
        &mut self,
        tiles: &mut Tiles<Pane>,
        layout: &LayoutContext<'_>,
        rect: Rect,
    ) {
        // clean up any empty holes at the end
        while self.children.last() == Some(&None) {
            self.children.pop();
        }

        let gap = layout.gap_width;

        let visible_children_and_holes = self.visible_children_and_holes(tiles);

        // Calculate grid dimensions:
        let (num_cols, num_rows) = {
            let num_visible_children = visible_children_and_holes.len();

            let num_cols = match self.layout {
                GridLayout::Auto => {
                    (layout.grid_auto_column_count)(num_visible_children, rect, gap)
                }
                GridLayout::Columns(num_columns) => num_columns,
            };
            let num_cols = num_cols.max(1);
            let num_rows = num_visible_children.div_ceil(num_cols);
            (num_cols, num_rows)
        };

        debug_assert!(
            visible_children_and_holes.len() <= num_cols * num_rows,
            "Bug in dock::Grid::layout"
        );

        // Figure out where each column and row goes:
        self.col_shares.resize(num_cols, 1.0);
        self.row_shares.resize(num_rows, 1.0);

        let col_widths = sizes_from_shares(&self.col_shares, rect.width(), gap);
        let row_heights = sizes_from_shares(&self.row_shares, rect.height(), gap);

        debug_assert_eq!(col_widths.len(), num_cols, "Bug in dock::Grid::layout");
        debug_assert_eq!(row_heights.len(), num_rows, "Bug in dock::Grid::layout");

        {
            let mut x = rect.left();
            self.col_ranges.clear();
            for &width in &col_widths {
                self.col_ranges.push(Rangef::new(x, x + width));
                x += width + gap;
            }
        }
        {
            let mut y = rect.top();
            self.row_ranges.clear();
            for &height in &row_heights {
                self.row_ranges.push(Rangef::new(y, y + height));
                y += height + gap;
            }
        }

        debug_assert_eq!(self.col_ranges.len(), num_cols, "Bug in dock::Grid::layout");
        debug_assert_eq!(self.row_ranges.len(), num_rows, "Bug in dock::Grid::layout");

        // Layout each child:
        for (i, &child) in visible_children_and_holes.iter().enumerate() {
            if let Some(child) = child {
                let col = i % num_cols;
                let row = i / num_cols;
                let child_rect = Rect::from_x_y_ranges(self.col_ranges[col], self.row_ranges[row]);
                tiles.layout_tile(layout, child_rect, child);
            }
        }

        // Check if we should collapse some holes:
        {
            let num_holes = visible_children_and_holes
                .iter()
                .filter(|c| c.is_none())
                .count()
                + (num_cols * num_rows - visible_children_and_holes.len());

            if num_cols.min(num_rows) <= num_holes {
                // More holes than there are columns or rows - let's collapse all holes
                // so that we can shrink for next frame:
                self.collapse_holes();
            }
        }
    }

    pub(super) fn interact<Pane>(
        &mut self,
        tree: &mut Tree<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        tile_id: TileId,
    ) {
        for child in self.children.clone() {
            if let Some(child) = child {
                if tree.is_visible(child) {
                    tree.tile_interact(behavior, ctx, child);
                }
            }
        }

        // Register drop-zones:
        for i in 0..(self.col_ranges.len() * self.row_ranges.len()) {
            let col = i % self.col_ranges.len();
            let row = i / self.col_ranges.len();
            let child_rect = Rect::from_x_y_ranges(self.col_ranges[col], self.row_ranges[row]);
            ctx.drop.suggest_rect(
                InsertionPoint::new(tile_id, ContainerInsertion::Grid(i)),
                child_rect,
            );
        }

        self.resize_columns(&tree.tiles, behavior, ctx, tile_id);
        self.resize_rows(&tree.tiles, behavior, ctx, tile_id);
    }

    fn resize_columns<Pane>(
        &mut self,
        tiles: &Tiles<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        parent_id: TileId,
    ) {
        let resizable = behavior.is_container_resizable(tiles, parent_id);

        let parent_rect = tiles.rect_or_die(parent_id);
        let ranges = self.col_ranges.clone();
        for (i, pair) in ranges.windows(2).enumerate() {
            let (left, right) = (pair[0], pair[1]);
            let splitter_id = SplitterId {
                parent_id,
                kind: SplitterKind::GridColumn,
                index: i,
            };

            let x = lerp(left.max, right.min, 0.5);

            let line_rect = Rect::from_center_size(
                pos2(x, parent_rect.center().y),
                vec2(
                    2.0 * behavior.resize_grab_radius_side(),
                    parent_rect.height(),
                ),
            );

            let mut resize_state = ResizeState::Idle;
            if resizable {
                let response = splitter_response(splitter_id, line_rect, ctx.input, ctx.drag);
                if let Some(pointer) = ctx.input.pointer_pos {
                    resize_state = resize_interaction(
                        behavior,
                        &ranges,
                        &mut self.col_shares,
                        &response,
                        pointer.round_to_pixels(ctx.input.pixels_per_point).x - x,
                        i,
                    );
                }
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

    fn resize_rows<Pane>(
        &mut self,
        tiles: &Tiles<Pane>,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        parent_id: TileId,
    ) {
        let resizable = behavior.is_container_resizable(tiles, parent_id);

        let parent_rect = tiles.rect_or_die(parent_id);
        let ranges = self.row_ranges.clone();
        for (i, pair) in ranges.windows(2).enumerate() {
            let (top, bottom) = (pair[0], pair[1]);
            let splitter_id = SplitterId {
                parent_id,
                kind: SplitterKind::GridRow,
                index: i,
            };

            let y = lerp(top.max, bottom.min, 0.5);

            let line_rect = Rect::from_center_size(
                pos2(parent_rect.center().x, y),
                vec2(
                    parent_rect.width(),
                    2.0 * behavior.resize_grab_radius_side(),
                ),
            );

            let mut resize_state = ResizeState::Idle;
            if resizable {
                let response = splitter_response(splitter_id, line_rect, ctx.input, ctx.drag);
                if let Some(pointer) = ctx.input.pointer_pos {
                    resize_state = resize_interaction(
                        behavior,
                        &ranges,
                        &mut self.row_shares,
                        &response,
                        pointer.round_to_pixels(ctx.input.pixels_per_point).y - y,
                        i,
                    );
                }
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
        for child_opt in &mut self.children {
            if let Some(child) = *child_opt {
                match simplify(child) {
                    SimplifyAction::Remove => {
                        *child_opt = None;
                    }
                    SimplifyAction::Keep => {}
                    SimplifyAction::Replace(new) => {
                        *child_opt = Some(new);
                    }
                }
            }
        }
    }

    pub(super) fn retain(&mut self, mut retain: impl FnMut(TileId) -> bool) {
        for child_opt in &mut self.children {
            if let Some(child) = *child_opt {
                if !retain(child) {
                    *child_opt = None;
                }
            }
        }
    }

    /// Returns child index, if found.
    pub(crate) fn remove_child(&mut self, needle: TileId) -> Option<usize> {
        let index = self
            .children
            .iter()
            .position(|&child| child == Some(needle))?;
        self.children[index] = None;
        Some(index)
    }
}

fn resize_interaction<Pane>(
    behavior: &mut dyn Behavior<Pane>,
    ranges: &[Rangef],
    shares: &mut [f32],
    splitter_response: &SplitterResponse,
    dx: f32,
    i: usize,
) -> ResizeState {
    assert_eq!(ranges.len(), shares.len(), "Bug in dock::Grid");
    let num = ranges.len();
    let tile_width = |i: usize| ranges[i].span();

    let left = i;
    let right = i + 1;

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
                &(0..=i).rev().collect::<Vec<_>>(),
                dx.abs(),
                tile_width,
            );
        } else {
            // Expand the left, shrink stuff to the right:
            shares[left] += shrink_shares(
                behavior,
                shares,
                &(i + 1..num).collect::<Vec<_>>(),
                dx.abs(),
                tile_width,
            );
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
    shares: &mut [f32],
    children: &[usize],
    target_in_points: f32,
    size_in_point: impl Fn(usize) -> f32,
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

fn sizes_from_shares(shares: &[f32], available_size: f32, gap_width: f32) -> Vec<f32> {
    if shares.is_empty() {
        return vec![];
    }

    let available_size = available_size - gap_width * (shares.len() - 1) as f32;
    let available_size = available_size.max(0.0);

    let total_share: f32 = shares.iter().sum();
    if total_share <= 0.0 {
        vec![available_size / shares.len() as f32; shares.len()]
    } else {
        shares
            .iter()
            .map(|&share| share / total_share * available_size)
            .collect()
    }
}
