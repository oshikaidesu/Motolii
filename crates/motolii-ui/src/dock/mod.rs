// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/lib.rs`.
//
// # 移植の範囲(C4 capsule)
//
// - そのまま:   `tiles.rs` / `container/mod.rs` / `tile.rs`
// - ほぼそのまま: `container/{grid,linear,tabs}.rs` / `tree.rs`(`Rect`/`Vec2` は `geom.rs` へ)
// - 書き直し:   `behavior.rs` と各 `ui()`(描画・カーソル・ヒット判定は元から差し替え前提)
//
// なぜこの形か: 挙動は egui_tiles を写す。改善しない。egui への依存は 0 件に保つ。
//
// 移植で落としたもの(意味を変えないための機械的置換):
// - `log::{trace,debug,warn}`: `motolii-ui` に `log` 依存が無く、Cargo.toml は ALLOWLIST 外
// - `ahash`: `std::collections::{HashMap, HashSet}` へ(元から順序非依存)
// - `itertools`: `Box<dyn Iterator>` と `slice::windows(2)` へ
// - `egui::Id`: ツリーIDは `String`、ドラッグ状態は `dock::drag::DragState`(HashMap)へ

// なぜ: 移植元と同じく unsafe を使わない
#![forbid(unsafe_code)]

use self::geom::{Pos2, Rect};

mod behavior;
mod container;
pub mod css;
pub mod drag;
pub mod geom;
mod interaction;
mod tile;
mod tiles;
mod tree;

#[cfg(test)]
mod tests;

pub use behavior::{Behavior, EditAction, TabState};
pub use container::{Container, ContainerKind, Grid, GridLayout, Linear, LinearDir, Shares, Tabs};
pub use drag::DragState;
pub use interaction::{DockInput, ResizeHandle, SplitterId, SplitterKind, SplitterResponse};
pub use tile::{Tile, TileId};
pub use tiles::Tiles;
pub use tree::{DockFrame, Tree};

// ----------------------------------------------------------------------------

/// The response from the host for a pane.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiResponse {
    #[default]
    None,

    /// The viewer is being dragged via some element in the Pane
    DragStarted,
}

/// What are the rules for simplifying the tree?
///
/// Drag-dropping tiles can often leave containers empty, or with only a single child.
/// The [`SimplificationOptions`] specifies what simplifications are allowed.
///
/// The [`Tree`] will run a simplification pass each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimplificationOptions {
    /// Remove empty [`Tabs`] containers?
    pub prune_empty_tabs: bool,

    /// Remove empty containers (that aren't [`Tabs`])?
    pub prune_empty_containers: bool,

    /// Remove [`Tabs`] containers with only a single child?
    ///
    /// Even if `true`, [`Self::all_panes_must_have_tabs`] will be respected.
    pub prune_single_child_tabs: bool,

    /// Prune containers (that aren't [`Tabs`]) with only a single child?
    pub prune_single_child_containers: bool,

    /// If true, each pane will have a [`Tabs`] container as a parent.
    ///
    /// This will win out over [`Self::prune_single_child_tabs`].
    pub all_panes_must_have_tabs: bool,

    /// If a horizontal container contain another horizontal container, join them?
    /// Same for vertical containers. Does NOT apply to grid container or tab containers.
    pub join_nested_linear_containers: bool,

    /// Flatten tabs in tabs?
    pub flatten_tabs_in_tabs: bool,
}

impl SimplificationOptions {
    /// [`SimplificationOptions`] with all simplifications turned off.
    pub const OFF: Self = Self {
        prune_empty_tabs: false,
        prune_empty_containers: false,
        prune_single_child_tabs: false,
        prune_single_child_containers: false,
        all_panes_must_have_tabs: false,
        join_nested_linear_containers: false,
        flatten_tabs_in_tabs: false,
    };
}

impl Default for SimplificationOptions {
    fn default() -> Self {
        Self {
            prune_empty_tabs: true,
            prune_single_child_tabs: true,
            prune_empty_containers: true,
            prune_single_child_containers: true,
            all_panes_must_have_tabs: false,
            join_nested_linear_containers: true,
            flatten_tabs_in_tabs: false,
        }
    }
}

/// The current state of a resize handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResizeState {
    Idle,

    /// The user is hovering over the resize handle.
    Hovering,

    /// The user is dragging the resize handle.
    Dragging,
}

// ----------------------------------------------------------------------------

/// An insertion point in a specific container.
///
/// Specifies the expected container layout type, and where to insert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContainerInsertion {
    Tabs(usize),
    Horizontal(usize),
    Vertical(usize),
    Grid(usize),
}

impl ContainerInsertion {
    /// Where in the parent (in what order among its children).
    fn index(self) -> usize {
        match self {
            Self::Tabs(index)
            | Self::Horizontal(index)
            | Self::Vertical(index)
            | Self::Grid(index) => index,
        }
    }

    fn kind(self) -> ContainerKind {
        match self {
            Self::Tabs(_) => ContainerKind::Tabs,
            Self::Horizontal(_) => ContainerKind::Horizontal,
            Self::Vertical(_) => ContainerKind::Vertical,
            Self::Grid(_) => ContainerKind::Grid,
        }
    }
}

/// Where in the tree to insert a tile.
#[derive(Clone, Copy, Debug)]
pub(crate) struct InsertionPoint {
    pub parent_id: TileId,

    /// Where in the parent?
    pub insertion: ContainerInsertion,
}

impl InsertionPoint {
    pub fn new(parent_id: TileId, insertion: ContainerInsertion) -> Self {
        Self {
            parent_id,
            insertion,
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum GcAction {
    Keep,
    Remove,
}

#[must_use]
pub(crate) enum SimplifyAction {
    Remove,
    Keep,
    Replace(TileId),
}

// ----------------------------------------------------------------------------

/// Context used for drag-and-dropping of tiles.
///
/// This is passed down during the interaction pass.
/// Each tile registers itself with this context.
pub(crate) struct DropContext {
    pub enabled: bool,
    pub dragged_tile_id: Option<TileId>,
    pub mouse_pos: Option<Pos2>,

    pub best_insertion: Option<InsertionPoint>,
    pub best_dist_sq: f32,
    pub preview_rect: Option<Rect>,
}

impl DropContext {
    /// なぜ `tab_bar_height` を直接受けるか: 元は `behavior.tab_bar_height(style)` だが、
    /// `egui::Style` を持たないため呼び出し側で解決して渡す。値の出所は変えていない。
    pub(crate) fn on_tile<Pane>(
        &mut self,
        tab_bar_height: f32,
        parent_id: TileId,
        rect: Rect,
        tile: &Tile<Pane>,
    ) {
        if !self.enabled {
            return;
        }

        if tile.kind() != Some(ContainerKind::Horizontal) {
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Horizontal(0)),
                rect.split_left_right_at_fraction(0.5).0,
            );
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Horizontal(usize::MAX)),
                rect.split_left_right_at_fraction(0.5).1,
            );
        }

        if tile.kind() != Some(ContainerKind::Vertical) {
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Vertical(0)),
                rect.split_top_bottom_at_fraction(0.5).0,
            );
            self.suggest_rect(
                InsertionPoint::new(parent_id, ContainerInsertion::Vertical(usize::MAX)),
                rect.split_top_bottom_at_fraction(0.5).1,
            );
        }

        self.suggest_rect(
            InsertionPoint::new(parent_id, ContainerInsertion::Tabs(usize::MAX)),
            rect.split_top_bottom_at_y(rect.top() + tab_bar_height).1,
        );
    }

    pub(crate) fn suggest_rect(&mut self, insertion: InsertionPoint, preview_rect: Rect) {
        if !self.enabled {
            return;
        }
        let target_point = preview_rect.center();
        if let Some(mouse_pos) = self.mouse_pos {
            let dist_sq = mouse_pos.distance_sq(target_point);
            if dist_sq < self.best_dist_sq {
                self.best_dist_sq = dist_sq;
                self.best_insertion = Some(insertion);
                self.preview_rect = Some(preview_rect);
            }
        }
    }
}
