// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/tiles.rs`.
//
// なぜほぼ無改変か: 移植元でも egui 依存は 2 行(0%)しか無い層である。
// 変更は `ahash` → `std::collections`、`log::*` の削除、`egui::Rect` → `dock::geom::Rect` だけ。

use std::collections::{HashMap, HashSet};

use super::behavior::LayoutContext;
use super::geom::{Pos2, Rect};

use super::{
    Behavior, Container, ContainerInsertion, ContainerKind, GcAction, InsertionPoint, Linear,
    LinearDir, SimplificationOptions, SimplifyAction, Tabs, Tile, TileId,
};

/// Contains all tile state, but no root.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Tiles<Pane> {
    next_tile_id: u64,

    tiles: HashMap<TileId, Tile<Pane>>,

    /// Tiles are visible by default, so we only store the invisible ones.
    invisible: HashSet<TileId>,

    /// Filled in by the layout step at the start of each frame.
    #[serde(default, skip)]
    pub(crate) rects: HashMap<TileId, Rect>,
}

impl<Pane: PartialEq> PartialEq for Tiles<Pane> {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            next_tile_id: _, // ignored
            tiles,
            invisible,
            rects: _, // ignore transient state
        } = self;
        tiles == &other.tiles && invisible == &other.invisible
    }
}

impl<Pane> Default for Tiles<Pane> {
    fn default() -> Self {
        Self {
            next_tile_id: 1,
            tiles: Default::default(),
            invisible: Default::default(),
            rects: Default::default(),
        }
    }
}

// ----------------------------------------------------------------------------

impl<Pane> Tiles<Pane> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// The number of tiles, including invisible tiles.
    #[inline]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn get(&self, tile_id: TileId) -> Option<&Tile<Pane>> {
        self.tiles.get(&tile_id)
    }

    /// Get the pane instance for a given [`TileId`]
    pub fn get_pane(&self, tile_id: &TileId) -> Option<&Pane> {
        match self.tiles.get(tile_id)? {
            Tile::Pane(pane) => Some(pane),
            Tile::Container(_) => None,
        }
    }

    /// Get the container instance for a given [`TileId`]
    pub fn get_container(&self, tile_id: TileId) -> Option<&Container> {
        match self.tiles.get(&tile_id)? {
            Tile::Container(container) => Some(container),
            Tile::Pane(_) => None,
        }
    }

    pub fn get_mut(&mut self, tile_id: TileId) -> Option<&mut Tile<Pane>> {
        self.tiles.get_mut(&tile_id)
    }

    /// Get the screen-space rectangle of where a tile is shown.
    ///
    /// This is updated by [`super::Tree::update`], so you need to call that first.
    ///
    /// If the tile isn't visible, or is in an inactive tab, this returns `None`.
    pub fn rect(&self, tile_id: TileId) -> Option<Rect> {
        if self.is_visible(tile_id) {
            self.rects.get(&tile_id).copied()
        } else {
            None
        }
    }

    pub(crate) fn rect_or_die(&self, tile_id: TileId) -> Rect {
        let rect = self.rect(tile_id);
        debug_assert!(rect.is_some(), "Failed to find rect for {tile_id:?}");
        rect.unwrap_or(Rect::from_min_max(Pos2::ZERO, Pos2::ZERO))
    }

    /// All tiles, in arbitrary order
    pub fn iter(&self) -> impl Iterator<Item = (&TileId, &Tile<Pane>)> + '_ {
        self.tiles.iter()
    }

    /// All tiles, in arbitrary order
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&TileId, &mut Tile<Pane>)> + '_ {
        self.tiles.iter_mut()
    }

    /// All [`TileId`]s, in arbitrary order
    pub fn tile_ids(&self) -> impl Iterator<Item = TileId> + '_ {
        self.tiles.keys().copied()
    }

    /// All [`Tile`]s in arbitrary order
    pub fn tiles(&self) -> impl Iterator<Item = &Tile<Pane>> + '_ {
        self.tiles.values()
    }

    /// All [`Tile`]s in arbitrary order
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile<Pane>> + '_ {
        self.tiles.values_mut()
    }

    /// Tiles are visible by default.
    ///
    /// Invisible tiles still retain their place in the tile hierarchy.
    pub fn is_visible(&self, tile_id: TileId) -> bool {
        !self.invisible.contains(&tile_id)
    }

    /// Tiles are visible by default.
    ///
    /// Invisible tiles still retain their place in the tile hierarchy.
    pub fn set_visible(&mut self, tile_id: TileId, visible: bool) {
        if visible {
            self.invisible.remove(&tile_id);
        } else {
            self.invisible.insert(tile_id);
        }
    }

    pub fn toggle_visibility(&mut self, tile_id: TileId) {
        self.set_visible(tile_id, !self.is_visible(tile_id));
    }

    /// This excludes all tiles that are invisible or are inactive tabs, recursively.
    pub(crate) fn collect_active_tiles(&self, tile_id: TileId, tiles: &mut Vec<TileId>) {
        if !self.is_visible(tile_id) {
            return;
        }
        tiles.push(tile_id);

        if let Some(Tile::Container(container)) = self.get(tile_id) {
            for child_id in container.active_children(self) {
                self.collect_active_tiles(child_id, tiles);
            }
        }
    }

    pub fn insert(&mut self, id: TileId, tile: Tile<Pane>) {
        self.tiles.insert(id, tile);
    }

    /// Remove the tile with the given id from the tiles container.
    ///
    /// Note that this does not actually remove the tile from the tree and may
    /// leave dangling references. If you want to permanently remove the tile
    /// consider calling [`super::Tree::remove_recursively`].
    pub fn remove(&mut self, id: TileId) -> Option<Tile<Pane>> {
        self.tiles.remove(&id)
    }

    pub fn next_free_id(&mut self) -> TileId {
        let mut id = TileId::from_u64(self.next_tile_id);

        // Make sure it doesn't collide with an existing id
        while self.tiles.contains_key(&id) {
            self.next_tile_id += 1;
            id = TileId::from_u64(self.next_tile_id);
        }

        // Final increment the next_id
        self.next_tile_id += 1;

        id
    }

    /// Recomputes `next_tile_id` so that newly allocated tiles
    /// will not collide with any existing `TileId` or any holes
    /// left in the numbering.
    ///
    /// Note: this sets `next_tile_id` to the maximum existing `TileId`+1.
    pub fn recompute_next_tile_id(&mut self) {
        self.next_tile_id = self
            .tiles
            .keys()
            .map(|tile_id| tile_id.0 + 1)
            .max()
            .unwrap_or(self.next_tile_id);
    }

    #[must_use]
    pub fn insert_new(&mut self, tile: Tile<Pane>) -> TileId {
        let id = self.next_free_id();
        self.tiles.insert(id, tile);
        id
    }

    #[must_use]
    pub fn insert_pane(&mut self, pane: Pane) -> TileId {
        self.insert_new(Tile::Pane(pane))
    }

    #[must_use]
    pub fn insert_container(&mut self, container: impl Into<Container>) -> TileId {
        self.insert_new(Tile::Container(container.into()))
    }

    #[must_use]
    pub fn insert_tab_tile(&mut self, children: Vec<TileId>) -> TileId {
        self.insert_new(Tile::Container(Container::new_tabs(children)))
    }

    #[must_use]
    pub fn insert_horizontal_tile(&mut self, children: Vec<TileId>) -> TileId {
        self.insert_new(Tile::Container(Container::new_linear(
            LinearDir::Horizontal,
            children,
        )))
    }

    #[must_use]
    pub fn insert_vertical_tile(&mut self, children: Vec<TileId>) -> TileId {
        self.insert_new(Tile::Container(Container::new_linear(
            LinearDir::Vertical,
            children,
        )))
    }

    #[must_use]
    pub fn insert_grid_tile(&mut self, children: Vec<TileId>) -> TileId {
        self.insert_new(Tile::Container(Container::new_grid(children)))
    }

    pub fn parent_of(&self, child_id: TileId) -> Option<TileId> {
        // Each tile can only have one parent
        for (tile_id, tile) in &self.tiles {
            if let Tile::Container(container) = tile {
                if container.has_child(child_id) {
                    return Some(*tile_id);
                }
            }
        }
        None
    }

    pub fn is_root(&self, tile_id: TileId) -> bool {
        self.parent_of(tile_id).is_none()
    }

    /// Insert `inserted_id` into the tree at `insertion_point`.
    ///
    /// When the insertion point's parent is not already a container of the required kind, that
    /// parent gets wrapped in a new container holding both it and `inserted_id`. The parent keeps
    /// its own [`TileId`] and the *new* container is the one given a freshly allocated id, so a
    /// `TileId` always keeps referring to the same tile. Applications that key their own state off
    /// `TileId`s depend on that.
    ///
    /// Returns the id of the new container, when one was needed. Since [`Tiles`] does not know
    /// what the tree's root is, the caller must re-point the root at it if the wrapped tile
    /// happened to be the root.
    #[must_use]
    pub(crate) fn insert_at(
        &mut self,
        insertion_point: InsertionPoint,
        inserted_id: TileId,
    ) -> Option<TileId> {
        let InsertionPoint {
            parent_id,
            insertion,
        } = insertion_point;

        let mut parent_tile = self.tiles.remove(&parent_id)?;

        // Can the parent take the tile as-is, or does it need wrapping in a new container?
        let inserted_into_parent = match (&mut parent_tile, insertion) {
            (Tile::Container(Container::Tabs(tabs)), ContainerInsertion::Tabs(index)) => {
                let index = index.min(tabs.children.len());
                tabs.children.insert(index, inserted_id);
                tabs.set_active(inserted_id);
                true
            }

            (
                Tile::Container(Container::Linear(
                    linear @ Linear {
                        dir: LinearDir::Horizontal,
                        ..
                    },
                )),
                ContainerInsertion::Horizontal(index),
            )
            | (
                Tile::Container(Container::Linear(
                    linear @ Linear {
                        dir: LinearDir::Vertical,
                        ..
                    },
                )),
                ContainerInsertion::Vertical(index),
            ) => {
                let index = index.min(linear.children.len());
                linear.children.insert(index, inserted_id);
                true
            }

            (Tile::Container(Container::Grid(grid)), ContainerInsertion::Grid(index)) => {
                grid.insert_at(index, inserted_id);
                true
            }

            _ => false,
        };

        // Either way the parent goes back exactly where it was, under its original id.
        self.tiles.insert(parent_id, parent_tile);

        if inserted_into_parent {
            return None;
        }

        // Look up the grandparent _before_ creating the wrapper, or `parent_of` would find the
        // wrapper itself.
        let grandparent_id = self.parent_of(parent_id);

        let mut children = vec![parent_id];
        children.insert(insertion.index().min(1), inserted_id);

        let container = match insertion {
            ContainerInsertion::Tabs(_) => {
                let mut tabs = Tabs::new(children);
                tabs.set_active(inserted_id);
                Container::Tabs(tabs)
            }
            ContainerInsertion::Horizontal(_) => {
                Container::new_linear(LinearDir::Horizontal, children)
            }
            ContainerInsertion::Vertical(_) => Container::new_linear(LinearDir::Vertical, children),
            ContainerInsertion::Grid(_) => Container::new_grid(children),
        };
        let wrapper_id = self.insert_new(Tile::Container(container));

        // Whoever referred to the parent now refers to the container wrapping it.
        // `grandparent_id` is `None` when the parent is the root, which the caller handles.
        if let Some(grandparent_id) = grandparent_id {
            if let Some(Tile::Container(grandparent)) = self.get_mut(grandparent_id) {
                // `grandparent_id` came from `parent_of`, so a miss here would be a bug.
                let _replaced: Option<usize> = grandparent.replace_child(parent_id, wrapper_id);
            }
        }

        Some(wrapper_id)
    }

    /// Detect cycles, duplications, and other invalid state, and fix it.
    ///
    /// Will also call [`Behavior::retain_pane`] to check if a user wants to remove a pane.
    ///
    /// Finally free up any tiles that are no longer reachable from the root.
    ///
    /// Returns whether the root survived.
    #[must_use]
    pub(crate) fn gc_root(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        root_id: Option<TileId>,
    ) -> bool {
        let mut visited = Default::default();

        let root_kept = match root_id {
            Some(root_id) => self.gc_tile_id(behavior, &mut visited, root_id) == GcAction::Keep,
            None => true,
        };

        self.invisible.retain(|tile_id| visited.contains(tile_id));
        self.tiles.retain(|tile_id, _| visited.contains(tile_id));

        root_kept
    }

    /// Detect cycles, duplications, and other invalid state, and remove them.
    fn gc_tile_id(
        &mut self,
        behavior: &mut dyn Behavior<Pane>,
        visited: &mut HashSet<TileId>,
        tile_id: TileId,
    ) -> GcAction {
        let Some(mut tile) = self.tiles.remove(&tile_id) else {
            return GcAction::Remove;
        };
        if !visited.insert(tile_id) {
            // Put the tile back before telling the caller to drop it.
            //
            // What is duplicated is the *reference*, not the tile: we got here through a second
            // parent, and the tile is still alive under the first one. Returning without the
            // re-insert deletes it from the arena outright, and then the first parent names a
            // child that no longer exists - damage strictly worse than the sharing it was meant
            // to repair. `simplify` unravels it from there: the parent looks empty, gets pruned,
            // its parent looks empty, and a tree that was merely ill-formed ends up gone.
            self.tiles.insert(tile_id, tile);
            return GcAction::Remove;
        }

        match &mut tile {
            Tile::Pane(pane) => {
                if !behavior.retain_pane(pane) {
                    return GcAction::Remove;
                }
            }
            Tile::Container(container) => {
                container
                    .retain(|child| self.gc_tile_id(behavior, visited, child) == GcAction::Keep);
            }
        }
        self.tiles.insert(tile_id, tile);
        GcAction::Keep
    }

    pub(crate) fn layout_tile(&mut self, layout: &LayoutContext<'_>, rect: Rect, tile_id: TileId) {
        let Some(mut tile) = self.tiles.remove(&tile_id) else {
            return;
        };
        self.rects.insert(tile_id, rect);

        if let Tile::Container(container) = &mut tile {
            container.layout(self, layout, rect);
        }

        self.tiles.insert(tile_id, tile);
    }

    /// Simplify the tree, perhaps culling empty containers,
    /// and/or merging single-child containers into their parent.
    ///
    /// Drag-dropping tiles can often leave containers empty, or with only a single child.
    /// This is often undesired, so this function can be used to clean up the tree.
    ///
    /// What simplifications are allowed is controlled by the [`SimplificationOptions`].
    pub(crate) fn simplify(
        &mut self,
        options: &SimplificationOptions,
        it: TileId,
        parent_kind: Option<ContainerKind>,
    ) -> SimplifyAction {
        let Some(mut tile) = self.tiles.remove(&it) else {
            return SimplifyAction::Remove;
        };

        if let Tile::Container(container) = &mut tile {
            let kind = container.kind();
            container.simplify_children(|child| self.simplify(options, child, Some(kind)));

            if kind == ContainerKind::Tabs {
                if options.prune_empty_tabs && container.is_empty() {
                    return SimplifyAction::Remove;
                }

                if options.prune_single_child_tabs {
                    if let Some(only_child) = container.only_child() {
                        let child_is_pane = matches!(self.get(only_child), Some(Tile::Pane(_)));

                        if options.all_panes_must_have_tabs
                            && child_is_pane
                            && parent_kind != Some(ContainerKind::Tabs)
                        {
                            // Keep it, even though we only have one child
                        } else {
                            return SimplifyAction::Replace(only_child);
                        }
                    }
                }

                if options.flatten_tabs_in_tabs {
                    let was_active = match container {
                        Container::Tabs(tabs) => tabs.active,
                        Container::Linear(_) | Container::Grid(_) => None,
                    };

                    let mut found = false;
                    let mut new_children = Vec::new();
                    let mut new_active = None;

                    for &child_id in container.children_vec().iter() {
                        if let Some(Tile::Container(Container::Tabs(child_tabs))) =
                            self.get(child_id)
                        {
                            if was_active == Some(child_id) {
                                // The open tab is the container being flattened away, so the
                                // user is looking at whichever of _its_ tabs was open.
                                new_active = child_tabs.active;
                            }
                            new_children.extend(child_tabs.children.iter().copied());
                            found = true;
                        } else {
                            if was_active == Some(child_id) {
                                new_active = Some(child_id);
                            }
                            new_children.push(child_id);
                        }
                    }

                    if found {
                        // Keep this container's own id: flattening changes what it holds,
                        // not which container it is. For the same reason, keep showing the tab
                        // the user had open.
                        let mut tabs = Tabs::new(new_children);
                        if let Some(new_active) = new_active {
                            tabs.set_active(new_active);
                        }
                        *container = Container::Tabs(tabs);
                    }
                }
            } else {
                if options.join_nested_linear_containers
                    && matches!(container, Container::Linear(_))
                {
                    let Container::Linear(parent) = &mut *container else {
                        unreachable!("just matched")
                    };
                    let mut new_children = Vec::with_capacity(parent.children.len());
                    for child_id in parent.children.drain(..) {
                        if let Some(Tile::Container(Container::Linear(child))) =
                            &mut self.tiles.get_mut(&child_id)
                        {
                            if parent.dir == child.dir {
                                // absorb the child
                                let mut child_share_sum = 0.0;
                                for &grandchild in &child.children {
                                    child_share_sum += child.shares[grandchild];
                                }
                                let share_normalizer = parent.shares[child_id] / child_share_sum;
                                let grandchildren: Vec<(TileId, f32)> = child
                                    .children
                                    .iter()
                                    .map(|&grandchild| (grandchild, child.shares[grandchild]))
                                    .collect();
                                for (grandchild, share) in grandchildren {
                                    new_children.push(grandchild);
                                    parent.shares[grandchild] = share * share_normalizer;
                                }

                                self.tiles.remove(&child_id);
                            } else {
                                // keep the child
                                new_children.push(child_id);
                            }
                        } else {
                            new_children.push(child_id);
                        }
                    }
                    parent.children = new_children;
                }

                if options.prune_empty_containers && container.is_empty() {
                    return SimplifyAction::Remove;
                }
                if options.prune_single_child_containers {
                    if let Some(only_child) = container.only_child() {
                        return SimplifyAction::Replace(only_child);
                    }
                }
            }
        }

        self.tiles.insert(it, tile);
        SimplifyAction::Keep
    }

    /// Returns the id of a new container that should take `it`'s place in its parent, if `it` was
    /// a pane that had to be wrapped in one.
    ///
    /// The pane keeps its own [`TileId`]; it is the new container that gets a fresh one. See
    /// [`Self::insert_at`] for why that matters.
    #[must_use]
    pub(crate) fn make_all_panes_children_of_tabs(
        &mut self,
        parent_is_tabs: bool,
        it: TileId,
    ) -> Option<TileId> {
        let mut tile = self.tiles.remove(&it)?;

        match &mut tile {
            Tile::Pane(_) => {
                if !parent_is_tabs {
                    // Add tabs to this pane:
                    self.tiles.insert(it, tile);
                    let tabs = Container::new_tabs(vec![it]);
                    return Some(self.insert_new(Tile::Container(tabs)));
                }
            }
            Tile::Container(container) => {
                let is_tabs = container.kind() == ContainerKind::Tabs;
                let children: Vec<TileId> = container.children().copied().collect();
                for child in children {
                    if let Some(new_child) = self.make_all_panes_children_of_tabs(is_tabs, child) {
                        let _replaced: Option<usize> = container.replace_child(child, new_child);
                    }
                }
            }
        }

        self.tiles.insert(it, tile);
        None
    }

    /// Returns true if the active tile was found in this tree.
    pub(crate) fn make_active(
        &mut self,
        it: TileId,
        should_activate: &mut dyn FnMut(TileId, &Tile<Pane>) -> bool,
    ) -> bool {
        let Some(mut tile) = self.tiles.remove(&it) else {
            return false;
        };

        let mut activate = should_activate(it, &tile);

        if let Tile::Container(container) = &mut tile {
            let mut active_child = None;
            for child in container.children_vec() {
                if self.make_active(child, should_activate) {
                    active_child = Some(child);
                }
            }

            if let Some(active_child) = active_child {
                if let Container::Tabs(tabs) = container {
                    tabs.set_active(active_child);
                }
            }

            activate |= active_child.is_some();
        }

        self.tiles.insert(it, tile);
        activate
    }
}

impl<Pane: PartialEq> Tiles<Pane> {
    /// Find the tile with the given pane.
    pub fn find_pane(&self, needle: &Pane) -> Option<TileId> {
        self.tiles
            .iter()
            .find(|(_, tile)| {
                if let Tile::Pane(pane) = *tile {
                    pane == needle
                } else {
                    false
                }
            })
            .map(|(tile_id, _)| *tile_id)
    }
}
