// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/tree.rs`.
//
// なぜ差分が出るか:
// - `egui::Id` の代わりに `String`。ドラッグ状態は `DragState`(HashMap)を `Tree` が持つ
//   (元は egui のメモリストア。`serde(skip)` で永続化しないところまで同じ)
// - `ui()` は `update()` へ。描画は返り値 `DockFrame` を受けたホストが CSS で行う

use super::behavior::{layout_tiles, EditAction};
use super::drag::DragState;
use super::geom::{Pos2, Rect};
use super::interaction::{DockInput, InteractContext, ResizeHandle};
use super::{ContainerInsertion, ContainerKind};

use super::{
    Behavior, Container, DropContext, InsertionPoint, SimplificationOptions, SimplifyAction, Tile,
    TileId, Tiles,
};

/// What one [`Tree::update`] produced for the host to draw.
#[derive(Clone, Debug, Default)]
pub struct DockFrame {
    /// Every splitter, in tree order, with the state it ended the pass in.
    pub handles: Vec<ResizeHandle>,

    /// The tile being dragged, if any.
    pub dragged_tile: Option<TileId>,

    /// Where to put the drag ghost (upstream draws it centered on the pointer).
    pub drag_ghost_pos: Option<Pos2>,

    /// Where the dragged tile would land, smoothed over time.
    pub drag_preview_rect: Option<Rect>,

    /// The container it would land in.
    pub drag_parent_rect: Option<Rect>,

    /// The smoothing is still in flight, so another frame is needed.
    pub needs_repaint: bool,
}

/// The top level type. Contains all persistent state, including layouts and sizes.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Tree<Pane> {
    /// The constant, globally unique id of this tree.
    pub(crate) id: String,

    /// None = empty tree
    pub root: Option<TileId>,

    /// All the tiles in the tree.
    pub tiles: Tiles<Pane>,

    /// When finite, this value contains the exact height of this tree
    #[serde(
        serialize_with = "serialize_f32_infinity_as_null",
        deserialize_with = "deserialize_f32_null_as_infinity"
    )]
    height: f32,

    /// When finite, this value contains the exact width of this tree
    #[serde(
        serialize_with = "serialize_f32_infinity_as_null",
        deserialize_with = "deserialize_f32_null_as_infinity"
    )]
    width: f32,

    /// Transient drag state. Never serialized, exactly as upstream never serialized it.
    #[serde(default, skip)]
    drag: DragState,
}

// Workaround for JSON which doesn't support infinity, because JSON is stupid.
fn serialize_f32_infinity_as_null<S: serde::Serializer>(
    t: &f32,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if t.is_infinite() {
        serializer.serialize_none()
    } else {
        serializer.serialize_some(t)
    }
}

fn deserialize_f32_null_as_infinity<'de, D: serde::Deserializer<'de>>(
    des: D,
) -> Result<f32, D::Error> {
    use serde::Deserialize as _;
    Ok(Option::<f32>::deserialize(des)?.unwrap_or(f32::INFINITY))
}

impl<Pane: PartialEq> PartialEq for Tree<Pane> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            id,
            root,
            tiles,
            height,
            width,
            drag: _, // ignore transient state
        } = self;

        id == &other.id
            && root == &other.root
            && tiles == &other.tiles
            && height == &other.height
            && width == &other.width
    }
}

impl<Pane: std::fmt::Debug> std::fmt::Debug for Tree<Pane> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print a hierarchical view of the tree:
        fn format_tile<Pane: std::fmt::Debug>(
            f: &mut std::fmt::Formatter<'_>,
            tiles: &Tiles<Pane>,
            indent: usize,
            tile_id: TileId,
        ) -> std::fmt::Result {
            write!(f, "{} {tile_id:?}: ", "  ".repeat(indent))?;
            if let Some(tile) = tiles.get(tile_id) {
                match tile {
                    Tile::Pane(pane) => writeln!(f, "Pane {pane:?}"),
                    Tile::Container(container) => {
                        writeln!(
                            f,
                            "{}",
                            match container {
                                Container::Tabs(_) => "Tabs",
                                Container::Linear(_) => "Linear",
                                Container::Grid(_) => "Grid",
                            }
                        )?;
                        for &child in container.children() {
                            format_tile(f, tiles, indent + 1, child)?;
                        }
                        Ok(())
                    }
                }
            } else {
                writeln!(f, "DANGLING")
            }
        }

        if let Some(root) = self.root {
            writeln!(f, "Tree {{")?;
            writeln!(f, "    id: {:?}", self.id)?;
            writeln!(f, "    width: {:?}", self.width)?;
            writeln!(f, "    height: {:?}", self.height)?;
            format_tile(f, &self.tiles, 1, root)?;
            write!(f, "}}")
        } else {
            writeln!(f, "Tree {{ }}")
        }
    }
}

// ----------------------------------------------------------------------------

impl<Pane> Tree<Pane> {
    /// Construct an empty tree.
    ///
    /// The `id` must be _globally_ unique (!).
    pub fn empty(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            root: None,
            tiles: Default::default(),
            width: f32::INFINITY,
            height: f32::INFINITY,
            drag: Default::default(),
        }
    }

    /// The most flexible constructor, allowing you to set up the tiles however you want.
    ///
    /// The `id` must be _globally_ unique (!).
    pub fn new(id: impl Into<String>, root: TileId, tiles: Tiles<Pane>) -> Self {
        Self {
            id: id.into(),
            root: Some(root),
            tiles,
            width: f32::INFINITY,
            height: f32::INFINITY,
            drag: Default::default(),
        }
    }

    /// Create a top-level [`super::Tabs`] container with the given panes.
    pub fn new_tabs(id: impl Into<String>, panes: Vec<Pane>) -> Self {
        Self::new_container(id, ContainerKind::Tabs, panes)
    }

    /// Create a top-level horizontal [`super::Linear`] container with the given panes.
    pub fn new_horizontal(id: impl Into<String>, panes: Vec<Pane>) -> Self {
        Self::new_container(id, ContainerKind::Horizontal, panes)
    }

    /// Create a top-level vertical [`super::Linear`] container with the given panes.
    pub fn new_vertical(id: impl Into<String>, panes: Vec<Pane>) -> Self {
        Self::new_container(id, ContainerKind::Vertical, panes)
    }

    /// Create a top-level [`super::Grid`] container with the given panes.
    pub fn new_grid(id: impl Into<String>, panes: Vec<Pane>) -> Self {
        Self::new_container(id, ContainerKind::Grid, panes)
    }

    /// Create a top-level container with the given panes.
    pub fn new_container(id: impl Into<String>, kind: ContainerKind, panes: Vec<Pane>) -> Self {
        let mut tiles = Tiles::default();
        let tile_ids = panes
            .into_iter()
            .map(|pane| tiles.insert_pane(pane))
            .collect();
        let root = tiles.insert_new(Tile::Container(Container::new(kind, tile_ids)));
        Self::new(id, root, tiles)
    }

    /// Throw away the current layout and take the one from `preset`, keeping this tree's id.
    ///
    /// なぜここに要るか: `docs/ui-interaction-language.md:75` の「既定presetへreset」。
    /// 既定presetが**何であるか**は製品側の決定なので、ここでは受け取るだけで作らない。
    pub fn reset_to(&mut self, preset: Self) {
        let id = std::mem::take(&mut self.id);
        *self = preset;
        self.id = id;
    }

    /// Remove the given tile and all child tiles, recursively.
    ///
    /// This also removes the tile id from the parent's list of children.
    ///
    /// All removed tiles are returned in unspecified order.
    pub fn remove_recursively(&mut self, id: TileId) -> Vec<Tile<Pane>> {
        // Remove the top-most tile_id from its parent
        self.remove_tile_id_from_parent(id);

        let mut removed_tiles = vec![];
        self.remove_recursively_impl(id, &mut removed_tiles);
        removed_tiles
    }

    fn remove_recursively_impl(&mut self, id: TileId, removed_tiles: &mut Vec<Tile<Pane>>) {
        // We can safely use the raw `tiles.remove` API here because either the parent was cleaned
        // up explicitly from `remove_recursively` or the parent is also being removed so there's
        // no reason to clean it up.
        if let Some(tile) = self.tiles.remove(id) {
            if let Tile::Container(container) = &tile {
                for &child_id in container.children() {
                    self.remove_recursively_impl(child_id, removed_tiles);
                }
            }
            removed_tiles.push(tile);
        }
    }

    /// The globally unique id used by this `Tree`.
    #[inline]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if [`Self::root`] is [`None`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[inline]
    pub fn root(&self) -> Option<TileId> {
        self.root
    }

    #[inline]
    pub fn is_root(&self, tile: TileId) -> bool {
        self.root == Some(tile)
    }

    /// The transient drag state, in place of egui's memory store.
    #[inline]
    pub fn drag(&self) -> &DragState {
        &self.drag
    }

    /// Tiles are visible by default.
    ///
    /// Invisible tiles still retain their place in the tile hierarchy.
    pub fn is_visible(&self, tile_id: TileId) -> bool {
        self.tiles.is_visible(tile_id)
    }

    /// Tiles are visible by default.
    ///
    /// Invisible tiles still retain their place in the tile hierarchy.
    pub fn set_visible(&mut self, tile_id: TileId, visible: bool) {
        self.tiles.set_visible(tile_id, visible);
    }

    /// All visible tiles.
    ///
    /// This excludes all tiles that are invisible or are inactive tabs, recursively.
    ///
    /// The order of the returned tiles is arbitrary.
    pub fn active_tiles(&self) -> Vec<TileId> {
        let mut tiles = vec![];
        if let Some(root) = self.root {
            if self.is_visible(root) {
                self.tiles.collect_active_tiles(root, &mut tiles);
            }
        }
        tiles
    }

    /// All non-visible tiles.
    ///
    /// This includes all tiles that are invisible or are inactive tabs. Uses `active_tiles`.
    ///
    /// The order of the returned tiles is arbitrary.
    pub fn inactive_tiles(&self) -> Vec<TileId> {
        let active_tiles = self.active_tiles();
        self.tiles
            .tile_ids()
            .filter(|id| !active_tiles.contains(id))
            .collect()
    }

    /// The host says a drag started on this tile (upstream: [`super::UiResponse::DragStarted`]).
    pub fn begin_drag(&mut self, behavior: &dyn Behavior<Pane>, tile_id: TileId) {
        if !self.is_root(tile_id) && behavior.is_tile_draggable(&self.tiles, tile_id) {
            self.drag.dragged_tile = Some(tile_id);
        }
    }

    /// One frame: simplify, gc, lay out, and run the drag/resize state machine.
    ///
    /// In place of the upstream `Tree::ui`. The tree will use up all of `rect` -
    /// nothing more, nothing less.
    pub fn update(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        input: &DockInput,
        rect: Rect,
    ) -> DockFrame {
        self.simplify(&behavior.simplification_options());

        self.gc(behavior);

        self.tiles.rects.clear();

        let mut drag = std::mem::take(&mut self.drag);
        drag.needs_repaint = false;
        if input.pointer_pressed {
            drag.press_origin = input.pointer_pos;
        }

        // Check if anything is being dragged:
        let mut drop_context = DropContext {
            enabled: true,
            dragged_tile_id: self.dragged_id(input, &mut drag),
            mouse_pos: input.pointer_pos,
            best_dist_sq: f32::INFINITY,
            best_insertion: None,
            preview_rect: None,
        };

        let mut rect = rect;
        if self.height.is_finite() {
            rect.set_height(self.height);
        }
        if self.width.is_finite() {
            rect.set_width(self.width);
        }
        if layout_tiles(&mut self.tiles, self.root, behavior, rect) {
            behavior.on_edit(EditAction::TabSelected);
        }

        let mut handles = Vec::new();
        {
            let mut ctx = InteractContext {
                input,
                drop: &mut drop_context,
                drag: &mut drag,
                handles: &mut handles,
            };
            if let Some(root) = self.root {
                self.tile_interact(behavior, &mut ctx, root);
            }
        }

        let mut frame = self.preview_dragged_tile(behavior, &drop_context, &mut drag, input);
        frame.handles = handles;

        if input.pointer_released {
            drag.active_splitter = None;
            drag.press_origin = None;
        }

        frame.needs_repaint = drag.needs_repaint;
        self.drag = drag;

        frame
    }

    /// Sets the exact height that can be used by the tree.
    pub fn set_height(&mut self, height: f32) {
        if height.is_sign_positive() && height.is_finite() {
            self.height = height;
        } else {
            self.height = f32::INFINITY;
        }
    }

    /// Sets the exact width that can be used by the tree.
    pub fn set_width(&mut self, width: f32) {
        if width.is_sign_positive() && width.is_finite() {
            self.width = width;
        } else {
            self.width = f32::INFINITY;
        }
    }

    pub(crate) fn tile_interact(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        ctx: &mut InteractContext<'_>,
        tile_id: TileId,
    ) {
        if !self.is_visible(tile_id) {
            return;
        }
        // NOTE: important that we get the rect and tile in two steps,
        // otherwise we could lose the tile when there is no rect.
        let Some(rect) = self.tiles.rect(tile_id) else {
            return;
        };
        let Some(mut tile) = self.tiles.remove(tile_id) else {
            return;
        };

        let drop_context_was_enabled = ctx.drop.enabled;
        if Some(tile_id) == ctx.drop.dragged_tile_id {
            // Can't drag a tile onto self or any children
            ctx.drop.enabled = false;
        }
        ctx.drop
            .on_tile(behavior.tab_bar_height(), tile_id, rect, &tile);

        if let Tile::Container(container) = &mut tile {
            container.interact(self, behavior, ctx, rect, tile_id);
        }

        self.tiles.insert(tile_id, tile);
        ctx.drop.enabled = drop_context_was_enabled;
    }

    /// Recursively "activate" the ancestors of the tiles that matches the given predicate.
    ///
    /// This means making the matching tiles and its ancestors the active tab in any tab layout.
    ///
    /// Returns `true` if a tab was made active.
    pub fn make_active(
        &mut self,
        mut should_activate: impl FnMut(TileId, &Tile<Pane>) -> bool,
    ) -> bool {
        if let Some(root) = self.root {
            self.tiles.make_active(root, &mut should_activate)
        } else {
            false
        }
    }

    fn preview_dragged_tile(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        drop_context: &DropContext,
        drag: &mut DragState,
        input: &DockInput,
    ) -> DockFrame {
        let mut frame = DockFrame::default();

        let (Some(mouse_pos), Some(dragged_tile_id)) =
            (drop_context.mouse_pos, drop_context.dragged_tile_id)
        else {
            return frame;
        };

        frame.dragged_tile = Some(dragged_tile_id);
        frame.drag_ghost_pos = Some(mouse_pos);

        if let Some(preview_rect) = drop_context.preview_rect {
            let preview_rect =
                drag.smooth_preview_rect(dragged_tile_id, preview_rect, input.stable_dt);

            frame.drag_parent_rect = drop_context
                .best_insertion
                .and_then(|insertion_point| self.tiles.rect(insertion_point.parent_id));
            frame.drag_preview_rect = Some(preview_rect);
        }

        if input.pointer_released {
            if let Some(insertion_point) = drop_context.best_insertion {
                behavior.on_edit(EditAction::TileDropped);
                self.move_tile(dragged_tile_id, insertion_point, false);
            }
            drag.clear_smooth_preview_rect(dragged_tile_id);
            drag.dragged_tile = None;
        }

        frame
    }

    /// Simplify and normalize the tree using the given options.
    ///
    /// This is also called at the start of [`Self::update`].
    pub fn simplify(&mut self, options: &SimplificationOptions) {
        if let Some(root) = self.root {
            match self.tiles.simplify(options, root, None) {
                SimplifyAction::Keep => {}
                SimplifyAction::Remove => {
                    self.root = None;
                }
                SimplifyAction::Replace(new_root) => {
                    self.root = Some(new_root);
                }
            }

            if options.all_panes_must_have_tabs {
                if let Some(tile_id) = self.root {
                    if let Some(new_root) =
                        self.tiles.make_all_panes_children_of_tabs(false, tile_id)
                    {
                        // The root was a bare pane, and is now wrapped in a tab container.
                        self.root = Some(new_root);
                    }
                }
            }
        }
    }

    /// Simplify all of the children of the given container tile recursively.
    pub fn simplify_children_of_tile(&mut self, tile_id: TileId, options: &SimplificationOptions) {
        if let Some(Tile::Container(mut container)) = self.tiles.remove(tile_id) {
            let kind = container.kind();
            container.simplify_children(|child| self.tiles.simplify(options, child, Some(kind)));
            self.tiles.insert(tile_id, Tile::Container(container));
        }
    }

    /// Garbage-collect tiles that are no longer reachable from the root tile.
    ///
    /// This is also called by [`Self::update`], so usually you don't need to call this yourself.
    pub fn gc(&mut self, behavior: &mut dyn Behavior<Pane>) {
        if !self.tiles.gc_root(behavior, self.root) {
            self.root = None;
        }
    }

    /// Move a tile to a new container, at the specified insertion index.
    ///
    /// If the insertion index is greater than the current number of children, the tile is
    /// appended at the end.
    ///
    /// See upstream for why `reflow_grid` exists:
    /// - when drag-and-dropping from a 2D representation of the grid, set `reflow_grid = false`
    /// - when drag-and-dropping from a 1D representation of the grid, set `reflow_grid = true`
    pub fn move_tile_to_container(
        &mut self,
        moved_tile_id: TileId,
        destination_container: TileId,
        mut insertion_index: usize,
        reflow_grid: bool,
    ) {
        // find target container
        if let Some(Tile::Container(target_container)) = self.tiles.get(destination_container) {
            let num_children = target_container.num_children();
            if insertion_index > num_children {
                insertion_index = num_children;
            }

            let container_insertion = match target_container.kind() {
                ContainerKind::Tabs => ContainerInsertion::Tabs(insertion_index),
                ContainerKind::Horizontal => ContainerInsertion::Horizontal(insertion_index),
                ContainerKind::Vertical => ContainerInsertion::Vertical(insertion_index),
                ContainerKind::Grid => ContainerInsertion::Grid(insertion_index),
            };

            self.move_tile(
                moved_tile_id,
                InsertionPoint {
                    parent_id: destination_container,
                    insertion: container_insertion,
                },
                reflow_grid,
            );
        }
    }

    /// Move the given tile to the given insertion point.
    ///
    /// See [`Self::move_tile_to_container()`] for details on `reflow_grid`.
    pub(crate) fn move_tile(
        &mut self,
        moved_tile_id: TileId,
        insertion_point: InsertionPoint,
        reflow_grid: bool,
    ) {
        if let Some((prev_parent_id, source_index)) = self.remove_tile_id_from_parent(moved_tile_id)
        {
            // Check to see if we are moving a tile within the same container:

            if prev_parent_id == insertion_point.parent_id {
                let parent_tile = self.tiles.get_mut(prev_parent_id);

                if let Some(Tile::Container(container)) =
                    parent_tile.filter(|tile| tile.kind() == Some(insertion_point.insertion.kind()))
                {
                    let dest_index = insertion_point.insertion.index();
                    // lets swap the two indices

                    let adjusted_index = if source_index < dest_index {
                        // We removed an earlier element, so we need to adjust the index:
                        dest_index - 1
                    } else {
                        dest_index
                    };

                    match container {
                        Container::Tabs(tabs) => {
                            let insertion_index = adjusted_index.min(tabs.children.len());
                            tabs.children.insert(insertion_index, moved_tile_id);
                            tabs.active = Some(moved_tile_id);
                        }
                        Container::Linear(linear) => {
                            let insertion_index = adjusted_index.min(linear.children.len());
                            linear.children.insert(insertion_index, moved_tile_id);
                        }
                        Container::Grid(grid) => {
                            if reflow_grid {
                                self.insert_at(insertion_point, moved_tile_id);
                            } else {
                                let dest_tile = grid.replace_at(dest_index, moved_tile_id);
                                if let Some(dest) = dest_tile {
                                    grid.insert_at(source_index, dest);
                                }
                            }
                        }
                    }
                    return; // done
                }
            }
        }

        // Moving to a new parent
        self.insert_at(insertion_point, moved_tile_id);
    }

    /// [`Tiles::insert_at`], plus the root fix-up that only the [`Tree`] can do.
    fn insert_at(&mut self, insertion_point: InsertionPoint, inserted_id: TileId) {
        if let Some(wrapper_id) = self.tiles.insert_at(insertion_point, inserted_id) {
            if self.root == Some(insertion_point.parent_id) {
                // The root got wrapped in a new container, which is now the root.
                self.root = Some(wrapper_id);
            }
        }
    }

    /// Find the currently dragged tile, if any.
    ///
    /// なぜ `&mut DragState` を取るか: 元は `ctx.stop_dragging()` で egui の状態を落としていた。
    fn dragged_id(&self, input: &DockInput, drag: &mut DragState) -> Option<TileId> {
        let tile_id = drag.dragged_tile?;

        if self.is_root(tile_id) {
            return None; // not allowed to drag root
        }

        // Abort drags on escape:
        if input.escape_pressed {
            drag.dragged_tile = None;
            return None;
        }

        Some(tile_id)
    }

    /// This removes the given tile from the parents list of children.
    ///
    /// The [`Tile`] itself is not removed from [`Self::tiles`].
    ///
    /// Performs no simplifications.
    ///
    /// If found, the parent tile and the child's index is returned.
    pub(crate) fn remove_tile_id_from_parent(
        &mut self,
        remove_me: TileId,
    ) -> Option<(TileId, usize)> {
        let mut result = None;

        for (parent_id, parent) in self.tiles.iter_mut() {
            if let Tile::Container(container) = parent {
                if let Some(child_index) = container.remove_child(remove_me) {
                    result = Some((*parent_id, child_index));
                }
            }
        }

        // Make sure that if we drag away the active some tabs,
        // that the tab container gets assigned another active tab.
        // If the tab is dragged to the same container, then it will become active again,
        // since all tabs become active when dragged, wherever they end up.
        if let Some((parent_id, _)) = result {
            if let Some(mut tile) = self.tiles.remove(parent_id) {
                if let Tile::Container(Container::Tabs(tabs)) = &mut tile {
                    tabs.ensure_active(&self.tiles);
                }
                self.tiles.insert(parent_id, tile);
            }
        }

        result
    }
}
