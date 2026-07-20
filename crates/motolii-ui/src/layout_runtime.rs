//! Motolii layout intentと`egui_tiles` runtimeのprivate投影境界。

use std::collections::{BTreeMap, HashSet};

use egui_tiles::{Container, Linear, LinearDir, Tile, TileId, Tiles, Tree};

use crate::layout::{
    normalize_runtime_shares, LayoutError, LayoutNode, PanelLayout, PanelRole, SplitAxis,
};

pub(crate) struct RuntimeLayout {
    tree: Tree<PanelRole>,
    pane_ids: BTreeMap<PanelRole, TileId>,
    separators: Vec<RuntimeSeparator>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSeparator {
    pub(crate) path: Vec<usize>,
    pub(crate) boundary: usize,
    pub(crate) axis: SplitAxis,
    pub(crate) leading: TileId,
    pub(crate) trailing: TileId,
}

impl RuntimeLayout {
    pub(crate) fn project(layout: &PanelLayout) -> Result<Self, LayoutError> {
        layout.validate()?;
        let mut tiles = Tiles::default();
        let mut pane_ids = BTreeMap::new();
        let mut separators = Vec::new();
        let root = project_node(
            layout.root(),
            &mut tiles,
            &mut pane_ids,
            &mut separators,
            &mut Vec::new(),
        );
        for role in PanelRole::ALL {
            let tile_id = pane_ids
                .get(&role)
                .copied()
                .ok_or(LayoutError::MissingRole { role })?;
            tiles.set_visible(tile_id, layout.is_visible(role));
        }
        let runtime = Self {
            tree: Tree::new("motolii-panel-layout", root, tiles),
            pane_ids,
            separators,
        };
        let projected_signature = runtime.canonical_signature()?;
        debug_assert_eq!(projected_signature, layout.canonical_signature());
        Ok(runtime)
    }

    pub(crate) fn tree_mut(&mut self) -> &mut Tree<PanelRole> {
        &mut self.tree
    }

    #[cfg(test)]
    pub(crate) fn tree(&self) -> &Tree<PanelRole> {
        &self.tree
    }

    pub(crate) fn separators(&self) -> &[RuntimeSeparator] {
        &self.separators
    }

    pub(crate) fn extract_proposal(&self) -> Result<PanelLayout, LayoutError> {
        let root = self.tree.root().ok_or(LayoutError::DanglingRuntimeTile)?;
        let mut visited = HashSet::new();
        let root = extract_node(&self.tree, root, &mut visited)?;
        let hidden = PanelRole::AUXILIARY
            .into_iter()
            .filter(|role| {
                self.pane_ids
                    .get(role)
                    .is_some_and(|tile_id| !self.tree.tiles.is_visible(*tile_id))
            })
            .collect();
        PanelLayout::from_parts(root, hidden)
    }

    pub(crate) fn canonical_signature(&self) -> Result<String, LayoutError> {
        Ok(self.extract_proposal()?.canonical_signature())
    }

    pub(crate) fn separator_response(
        &self,
        ui: &egui::Ui,
        separator: &RuntimeSeparator,
    ) -> Option<egui::Response> {
        if !self.tree.tiles.is_visible(separator.leading)
            || !self.tree.tiles.is_visible(separator.trailing)
        {
            return None;
        }
        let mut tile_id = self.tree.root()?;
        let mut parent_scope_id = ui.id();
        for index in &separator.path {
            let container_scope_id = parent_scope_id
                .with(tile_id)
                .with(egui::IdSalt::new("child"));
            let container = self.tree.tiles.get_container(tile_id)?;
            tile_id = *container.children().nth(*index)?;
            parent_scope_id = container_scope_id;
        }
        let container_scope_id = parent_scope_id
            .with(tile_id)
            .with(egui::IdSalt::new("child"));
        let container = self.tree.tiles.get_container(tile_id)?;
        let children: Vec<_> = container
            .children()
            .copied()
            .filter(|child| self.tree.is_visible(*child))
            .collect();
        let visible_boundary = children
            .windows(2)
            .position(|pair| pair == [separator.leading, separator.trailing])?;
        ui.ctx()
            .read_response(container_scope_id.with((tile_id, "resize", visible_boundary)))
    }

    #[cfg(test)]
    pub(crate) fn remove_stage_for_test(&mut self) {
        if let Some(tile_id) = self.pane_ids.get(&PanelRole::Stage).copied() {
            self.tree.tiles.remove(tile_id);
        }
    }
}

fn project_node(
    node: &LayoutNode,
    tiles: &mut Tiles<PanelRole>,
    pane_ids: &mut BTreeMap<PanelRole, TileId>,
    separators: &mut Vec<RuntimeSeparator>,
    path: &mut Vec<usize>,
) -> TileId {
    match node {
        LayoutNode::Pane(role) => {
            let tile_id = tiles.insert_pane(*role);
            pane_ids.insert(*role, tile_id);
            tile_id
        }
        LayoutNode::Split {
            axis,
            children,
            shares,
            ..
        } => {
            let child_ids: Vec<_> = children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    path.push(index);
                    let child_id = project_node(child, tiles, pane_ids, separators, path);
                    path.pop();
                    child_id
                })
                .collect();
            let mut linear = Linear::new(
                match axis {
                    SplitAxis::Horizontal => LinearDir::Horizontal,
                    SplitAxis::Vertical => LinearDir::Vertical,
                },
                child_ids.clone(),
            );
            for (child_id, share) in child_ids.iter().copied().zip(shares) {
                linear.shares.set_share(child_id, *share as f32);
            }
            for boundary in 0..child_ids.len().saturating_sub(1) {
                separators.push(RuntimeSeparator {
                    path: path.clone(),
                    boundary,
                    axis: *axis,
                    leading: child_ids[boundary],
                    trailing: child_ids[boundary + 1],
                });
            }
            tiles.insert_container(linear)
        }
        LayoutNode::Tabs { children, active } => {
            let child_ids: Vec<_> = children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    path.push(index);
                    let child_id = project_node(child, tiles, pane_ids, separators, path);
                    path.pop();
                    child_id
                })
                .collect();
            let mut tabs = egui_tiles::Tabs::new(child_ids);
            if let Some(active_id) = pane_ids.get(active).copied() {
                tabs.set_active(active_id);
            }
            tiles.insert_container(tabs)
        }
    }
}

fn extract_node(
    tree: &Tree<PanelRole>,
    tile_id: TileId,
    visited: &mut HashSet<TileId>,
) -> Result<LayoutNode, LayoutError> {
    if !visited.insert(tile_id) {
        return Err(LayoutError::RepeatedRuntimeTile);
    }
    let tile = tree
        .tiles
        .get(tile_id)
        .ok_or(LayoutError::DanglingRuntimeTile)?;
    match tile {
        Tile::Pane(role) => Ok(LayoutNode::Pane(*role)),
        Tile::Container(Container::Linear(linear)) => {
            let children = linear
                .children
                .iter()
                .copied()
                .map(|child| extract_node(tree, child, visited))
                .collect::<Result<Vec<_>, _>>()?;
            let runtime_shares: Vec<_> = linear
                .children
                .iter()
                .map(|child| linear.shares[*child])
                .collect();
            let shares = normalize_runtime_shares(&runtime_shares)?;
            Ok(LayoutNode::Split {
                axis: match linear.dir {
                    LinearDir::Horizontal => SplitAxis::Horizontal,
                    LinearDir::Vertical => SplitAxis::Vertical,
                },
                children,
                default_shares: vec![1; shares.len()],
                shares,
            })
        }
        Tile::Container(Container::Tabs(tabs)) => {
            let children = tabs
                .children
                .iter()
                .copied()
                .map(|child| extract_node(tree, child, visited))
                .collect::<Result<Vec<_>, _>>()?;
            let active_id = tabs
                .active
                .or_else(|| tabs.children.first().copied())
                .ok_or(LayoutError::EmptyContainer)?;
            let active = match tree.tiles.get(active_id) {
                Some(Tile::Pane(role)) => *role,
                Some(Tile::Container(_)) => return Err(LayoutError::NestedTabChild),
                None => return Err(LayoutError::DanglingRuntimeTile),
            };
            Ok(LayoutNode::Tabs { children, active })
        }
        Tile::Container(Container::Grid(_)) => Err(LayoutError::UnsupportedRuntimeGrid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutAction, LayoutConstraints};

    struct EmptyBehavior;

    impl egui_tiles::Behavior<PanelRole> for EmptyBehavior {
        fn pane_ui(
            &mut self,
            _ui: &mut egui::Ui,
            _tile_id: TileId,
            _pane: &mut PanelRole,
        ) -> egui_tiles::UiResponse {
            egui_tiles::UiResponse::None
        }

        fn tab_title_for_pane(&mut self, pane: &PanelRole) -> egui::WidgetText {
            pane.title().into()
        }
    }

    fn constraints() -> LayoutConstraints {
        LayoutConstraints {
            viewport_width: 1_000.0,
            stage_min_width: 320.0,
        }
    }

    #[test]
    fn two_empty_runtime_projections_have_tile_id_free_signature() {
        let layout = PanelLayout::built_in();
        let first = RuntimeLayout::project(&layout).unwrap();
        let second = RuntimeLayout::project(&layout).unwrap();
        assert_eq!(
            first.canonical_signature().unwrap(),
            second.canonical_signature().unwrap()
        );
        assert_eq!(
            first.canonical_signature().unwrap(),
            layout.canonical_signature()
        );
    }

    #[test]
    fn fixed_auxiliary_tab_and_split_project_and_extract() {
        let mut layout = PanelLayout::built_in();
        layout
            .move_tab_for_test(PanelRole::Browser, PanelRole::Inspector, constraints())
            .unwrap();
        layout
            .move_split_for_test(
                PanelRole::Timeline,
                PanelRole::Stage,
                SplitAxis::Vertical,
                false,
                constraints(),
            )
            .unwrap();
        let runtime = RuntimeLayout::project(&layout).unwrap();
        let extracted = runtime.extract_proposal().unwrap();
        assert_eq!(
            extracted.canonical_signature(),
            layout.canonical_signature()
        );
    }

    #[test]
    fn hidden_auxiliary_survives_projection_without_hiding_stage() {
        let mut layout = PanelLayout::built_in();
        layout
            .apply(LayoutAction::Hide(PanelRole::Browser), constraints())
            .unwrap();
        let runtime = RuntimeLayout::project(&layout).unwrap();
        assert!(!runtime
            .tree()
            .is_visible(runtime.pane_ids[&PanelRole::Browser]));
        assert!(runtime
            .tree()
            .is_visible(runtime.pane_ids[&PanelRole::Stage]));
        assert_eq!(
            runtime.extract_proposal().unwrap().canonical_signature(),
            layout.canonical_signature()
        );
    }

    #[test]
    fn adapter_reads_the_native_tiles_separator_response() {
        let context = egui::Context::default();
        let found_native_response = std::cell::Cell::new(false);
        let _ = context.run_ui(Default::default(), |ui| {
            let mut runtime = RuntimeLayout::project(&PanelLayout::built_in()).unwrap();
            runtime.tree_mut().ui(&mut EmptyBehavior, ui);
            let separator = runtime.separators().first().unwrap();
            let response = runtime.separator_response(ui, separator).unwrap();
            found_native_response
                .set(response.sense.senses_click() && response.sense.senses_drag());
        });
        assert!(found_native_response.get());
    }
}
