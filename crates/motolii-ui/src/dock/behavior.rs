// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/behavior.rs`.
//
// なぜ書き直しか: 移植元の `behavior.rs` は「差し替え口(設計上の継ぎ目)」であり、
// 中身の大半は `egui::{Ui, Painter, Visuals, Stroke, Color32}` による描画である。
// Blitz では描画はCSS側が持つので、ここには**寸法と可否の問い合わせだけ**を残す。
// 数値の既定は移植元および `egui::Style` の既定からそのまま写している。決めていない。

use super::geom::{Rect, Vec2};
use super::{SimplificationOptions, Tile, TileId, Tiles};

/// The kind of edit that triggered the call to [`Behavior::on_edit`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditAction {
    /// A tile was resized by dragging or double-clicking a boundary.
    TileResized,

    /// A drag with a tile started.
    TileDragged,

    /// A tile was dropped and its position changed accordingly.
    TileDropped,

    /// A tab was selected by a click, or by hovering a dragged tile over it,
    /// or there was no active tab and one was picked arbitrarily.
    TabSelected,
}

/// The state of a tab, used to inform the rendering of the tab.
#[derive(Clone, Debug, Default)]
pub struct TabState {
    /// Is the tab currently selected?
    pub active: bool,

    /// Is the tab currently being dragged?
    pub is_being_dragged: bool,

    /// Should the tab have a close button?
    pub closable: bool,
}

/// Everything the layout pass needs from a [`Behavior`], with the pane type erased.
///
/// The layout pass never looks at a pane's contents — it only needs a handful of numbers.
/// Gathering them up front keeps `Pane` out of the layout signatures entirely, which lets the
/// same code lay out any [`Tiles`], whatever it happens to store in its panes.
pub(crate) struct LayoutContext<'a> {
    pub gap_width: f32,
    pub tab_bar_height: f32,
    pub grid_auto_column_count: &'a dyn Fn(usize, Rect, f32) -> usize,

    /// The width of each tab button, in order, for a [`crate::dock::Tabs`] container.
    ///
    /// なぜ必要か: 元は egui の tab bar `Ui` が実測していた。Blitz では文字幅を持つのは
    /// ホスト側なので、`Behavior::tab_width` で受けてレイアウト段で確定させる。
    pub tab_width: &'a dyn Fn(TileId) -> f32,

    /// Set by the layout pass if it had to pick an active tab for a [`crate::dock::Tabs`] container.
    ///
    /// Reported back to the caller rather than straight to [`Behavior::on_edit`]: laying out
    /// the tree is not the place to be emitting user-visible edit events from.
    pub tab_auto_selected: &'a std::cell::Cell<bool>,
}

/// Lay out `tiles` starting at `root`, using only the pane-agnostic parts of `behavior`.
///
/// Generic over the pane type of `tiles`, which need not be the pane type `behavior` is for.
///
/// Returns `true` if the pass had to auto-select an active tab, in which case the caller
/// should report [`EditAction::TabSelected`].
pub(crate) fn layout_tiles<Pane, TilesPane>(
    tiles: &mut Tiles<TilesPane>,
    root: Option<TileId>,
    behavior: &dyn Behavior<Pane>,
    rect: Rect,
) -> bool {
    let Some(root) = root else {
        return false;
    };

    let grid_auto_column_count = |num_visible_children, rect, gap| {
        behavior.grid_auto_column_count(num_visible_children, rect, gap)
    };
    let tab_width = |tile_id| behavior.tab_width(tile_id);
    let tab_auto_selected = std::cell::Cell::new(false);

    let layout = LayoutContext {
        gap_width: behavior.gap_width(),
        tab_bar_height: behavior.tab_bar_height(),
        grid_auto_column_count: &grid_auto_column_count,
        tab_width: &tab_width,
        tab_auto_selected: &tab_auto_selected,
    };

    tiles.layout_tile(&layout, rect, root);

    tab_auto_selected.get()
}

/// Trait defining how the [`super::Tree`] and its panes should be shown.
pub trait Behavior<Pane> {
    /// The title of a pane tab.
    fn tab_title_for_pane(&mut self, pane: &Pane) -> String;

    /// Should the tab have a close-button?
    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        false
    }

    /// Called when the close-button on a tab is pressed.
    ///
    /// Return `false` to abort the closing of a tab (e.g. after showing a message box).
    fn on_tab_close(&mut self, _tiles: &mut Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    /// The title of a general tab.
    ///
    /// The default implementation calls [`Self::tab_title_for_pane`] for panes and
    /// uses the name of the [`super::ContainerKind`] for [`super::Container`]s.
    fn tab_title_for_tile(&mut self, tiles: &Tiles<Pane>, tile_id: TileId) -> String {
        if let Some(tile) = tiles.get(tile_id) {
            match tile {
                Tile::Pane(pane) => self.tab_title_for_pane(pane),
                Tile::Container(container) => format!("{:?}", container.kind()),
            }
        } else {
            "MISSING TILE".to_owned()
        }
    }

    /// The width of one tab button, in points.
    ///
    /// The default mirrors the arithmetic of the upstream `tab_ui`: the title's width
    /// plus [`Self::tab_title_spacing`] on both sides, but without a real text measurement —
    /// hosts that can measure text should override this.
    fn tab_width(&self, _tile_id: TileId) -> f32 {
        96.0
    }

    /// The height of the bar holding tab titles.
    fn tab_bar_height(&self) -> f32 {
        24.0
    }

    /// Extra spacing to left and right of tab titles.
    fn tab_title_spacing(&self) -> f32 {
        8.0
    }

    /// Width of the gap between tiles in a horizontal or vertical layout,
    /// and between rows/columns in a grid layout.
    fn gap_width(&self) -> f32 {
        1.0
    }

    /// How far from a splitter the pointer may be and still grab it.
    ///
    /// Copied from `egui::style::Interaction::resize_grab_radius_side`'s default.
    fn resize_grab_radius_side(&self) -> f32 {
        3.0
    }

    /// How far the pointer must travel with the button down before a press becomes a drag.
    ///
    /// Copied from `egui::InputOptions::max_click_dist`'s default.
    fn drag_start_distance(&self) -> f32 {
        6.0
    }

    /// No child should shrink below this width nor height.
    fn min_size(&self) -> f32 {
        32.0
    }

    /// What are the rules for simplifying the tree?
    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions::default()
    }

    /// Return `false` if a given pane should be removed from its parent.
    fn retain_pane(&mut self, _pane: &Pane) -> bool {
        true
    }

    /// How many columns should we use for a [`super::Grid`] put into [`super::GridLayout::Auto`]?
    ///
    /// The default heuristic tries to find a good column count that results in a per-tile
    /// aspect-ratio of [`Self::ideal_tile_aspect_ratio`].
    ///
    /// The `rect` is the available space for the grid,
    /// and `gap` is the distance between each column and row.
    fn grid_auto_column_count(&self, num_visible_children: usize, rect: Rect, gap: f32) -> usize {
        num_columns_heuristic(
            num_visible_children,
            rect.size(),
            gap,
            self.ideal_tile_aspect_ratio(),
        )
    }

    /// When using [`super::GridLayout::Auto`], what is the ideal aspect ratio of a tile?
    fn ideal_tile_aspect_ratio(&self) -> f32 {
        4.0 / 3.0
    }

    /// Can this tile be dragged?
    ///
    /// If `false`, the tile cannot be dragged by the user.
    /// This affects both tab dragging and pane dragging.
    ///
    /// Default: `true` (all tiles are draggable).
    fn is_tile_draggable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    /// Can the children of this container be resized by dragging the separator?
    ///
    /// Only applies to [`super::Linear`] and [`super::Grid`] containers.
    ///
    /// Default: `true` (all containers are resizable).
    fn is_container_resizable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    // Callbacks:

    /// Called if the user edits the tree somehow, e.g. changes the size of some container,
    /// clicks a tab, or drags a tile.
    fn on_edit(&mut self, _edit_action: EditAction) {}
}

/// How many columns should we use to fit `n` children in a grid?
fn num_columns_heuristic(n: usize, size: Vec2, gap: f32, desired_aspect: f32) -> usize {
    let mut best_loss = f32::INFINITY;
    let mut best_num_columns = 1;

    for ncols in 1..=n {
        if 4 <= n && ncols == n - 1 {
            // Don't suggest 7 columns when n=8 - that produces an ugly orphan on a single row.
            continue;
        }

        let nrows = n.div_ceil(ncols);

        let cell_width = (size.x - gap * (ncols as f32 - 1.0)) / (ncols as f32);
        let cell_height = (size.y - gap * (nrows as f32 - 1.0)) / (nrows as f32);

        let cell_aspect = cell_width / cell_height;
        let aspect_diff = (desired_aspect - cell_aspect).abs();
        let num_empty_cells = ncols * nrows - n;

        let loss = aspect_diff * n as f32 + 2.0 * num_empty_cells as f32;

        if loss < best_loss {
            best_loss = loss;
            best_num_columns = ncols;
        }
    }

    best_num_columns
}

#[test]
fn test_num_columns_heuristic() {
    // Four tiles should always be in a 1x4, 2x2, or 4x1 grid - NEVER 2x3 or 3x2.

    let n = 4;
    let gap = 0.0;
    let ideal_tile_aspect_ratio = 4.0 / 3.0;

    for i in 0..=100 {
        let height = super::geom::lerp(1.0, 1000.0, i as f32 / 100.0);
        let size = Vec2::new(100.0, height);

        let ncols = num_columns_heuristic(n, size, gap, ideal_tile_aspect_ratio);
        assert!(
            ncols == 1 || ncols == 2 || ncols == 4,
            "Size {size:?} got {ncols} columns"
        );
    }
}
