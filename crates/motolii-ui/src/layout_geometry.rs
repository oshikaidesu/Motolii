//! Host layout正本からlogical rectangleだけを導出するprivate projection。

use std::collections::BTreeMap;

use taffy::prelude::{
    AvailableSpace, Dimension, Display, FlexDirection, LengthPercentage, NodeId, Size, Style,
    TaffyTree,
};

use crate::layout::{LayoutNode, PanelLayout, PanelRole, SplitAxis};
use crate::native_host_layout::LogicalRect;

pub(crate) fn project_panel_rects(
    layout: &PanelLayout,
    width: f32,
    height: f32,
) -> Option<BTreeMap<PanelRole, LogicalRect>> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }

    let mut tree = TaffyTree::<()>::new();
    tree.disable_rounding();
    let root = add_node(&mut tree, layout, layout.root(), None)?;
    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        },
    )
    .ok()?;
    let mut output = BTreeMap::new();
    collect(&tree, layout, root, layout.root(), [0.0, 0.0], &mut output)?;
    Some(output)
}

fn add_node(
    tree: &mut TaffyTree<()>,
    layout: &PanelLayout,
    node: &LayoutNode,
    basis: Option<f32>,
) -> Option<NodeId> {
    let is_root = basis.is_none();
    let flex_basis = basis
        .map(|_| Dimension::length(0.0))
        .unwrap_or_else(Dimension::auto);
    let flex_grow = basis.unwrap_or(0.0);
    let size = if is_root {
        Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        }
    } else {
        Size {
            width: Dimension::auto(),
            height: Dimension::auto(),
        }
    };
    match node {
        LayoutNode::Pane(_) => tree
            .new_leaf(Style {
                display: Display::Flex,
                flex_basis,
                flex_grow,
                size,
                ..Default::default()
            })
            .ok(),
        LayoutNode::Split {
            axis,
            children,
            shares,
            ..
        } => {
            let visible = children
                .iter()
                .zip(shares)
                .filter(|(child, _)| node_is_visible(layout, child))
                .collect::<Vec<_>>();
            let total = visible.iter().map(|(_, share)| **share as f64).sum::<f64>();
            if !total.is_finite() || total <= 0.0 {
                return None;
            }
            let child_ids = visible
                .into_iter()
                .map(|(child, share)| {
                    add_node(tree, layout, child, Some(*share as f32 / total as f32))
                })
                .collect::<Option<Vec<_>>>()?;
            tree.new_with_children(
                Style {
                    display: Display::Flex,
                    flex_basis,
                    flex_grow,
                    flex_direction: match axis {
                        SplitAxis::Horizontal => FlexDirection::Row,
                        SplitAxis::Vertical => FlexDirection::Column,
                    },
                    size,
                    gap: Size {
                        width: LengthPercentage::length(0.0),
                        height: LengthPercentage::length(0.0),
                    },
                    ..Default::default()
                },
                &child_ids,
            )
            .ok()
        }
        LayoutNode::Tabs { children, active } => {
            let child_ids = children
                .iter()
                .filter(|child| {
                    matches!(child, LayoutNode::Pane(role) if role == active && layout.is_visible(*role))
                })
                .map(|_| {
                    tree.new_leaf(Style {
                        display: Display::Flex,
                        flex_basis: Dimension::length(0.0),
                        flex_grow: 1.0,
                        size: Size {
                            width: Dimension::auto(),
                            height: Dimension::auto(),
                        },
                        ..Default::default()
                    })
                    .ok()
                })
                .collect::<Option<Vec<_>>>()?;
            tree.new_with_children(
                Style {
                    display: Display::Flex,
                    flex_basis,
                    flex_grow,
                    size,
                    ..Default::default()
                },
                &child_ids,
            )
            .ok()
        }
    }
}

fn node_is_visible(layout: &PanelLayout, node: &LayoutNode) -> bool {
    match node {
        LayoutNode::Pane(role) => layout.is_visible(*role),
        LayoutNode::Split { children, .. } => {
            children.iter().any(|child| node_is_visible(layout, child))
        }
        LayoutNode::Tabs { active, .. } => layout.is_visible(*active),
    }
}

fn collect(
    tree: &TaffyTree<()>,
    authority: &PanelLayout,
    node_id: NodeId,
    node: &LayoutNode,
    origin: [f32; 2],
    output: &mut BTreeMap<PanelRole, LogicalRect>,
) -> Option<()> {
    let computed = tree.layout(node_id).ok()?;
    let absolute = [
        origin[0] + computed.location.x,
        origin[1] + computed.location.y,
    ];
    match node {
        LayoutNode::Pane(role) => {
            if authority.is_visible(*role) {
                output.insert(
                    *role,
                    LogicalRect {
                        x: f64::from(absolute[0]),
                        y: f64::from(absolute[1]),
                        width: f64::from(computed.size.width),
                        height: f64::from(computed.size.height),
                    },
                );
            }
        }
        LayoutNode::Split { children, .. } => {
            let child_ids = tree.children(node_id).ok()?;
            let visible_children = children
                .iter()
                .filter(|child| node_is_visible(authority, child))
                .collect::<Vec<_>>();
            if child_ids.len() != visible_children.len() {
                return None;
            }
            for (child_id, child) in child_ids.into_iter().zip(visible_children) {
                collect(tree, authority, child_id, child, absolute, output)?;
            }
        }
        LayoutNode::Tabs { children, active } => {
            let child_ids = tree.children(node_id).ok()?;
            let active_child = children.iter().find(
                |child| matches!(child, LayoutNode::Pane(role) if role == active && authority.is_visible(*role)),
            );
            if child_ids.len() != usize::from(active_child.is_some()) {
                return None;
            }
            if let (Some(child_id), Some(child)) = (child_ids.into_iter().next(), active_child) {
                collect(tree, authority, child_id, child, absolute, output)?;
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutAction, LayoutConstraints, SeparatorAction};

    fn constraints() -> LayoutConstraints {
        LayoutConstraints {
            viewport_width: 1_000.0,
            stage_min_width: 320.0,
        }
    }

    #[test]
    fn built_in_authority_projects_the_existing_product_rects() {
        let rects = project_panel_rects(&PanelLayout::built_in(), 1_000.0, 800.0).unwrap();
        assert_eq!(rects[&PanelRole::Browser].width, 200.0);
        assert_eq!(rects[&PanelRole::Stage].width, 600.0);
        assert_eq!(rects[&PanelRole::Inspector].x, 800.0);
        assert_eq!(rects[&PanelRole::Timeline].y, 600.0);
    }

    #[test]
    fn separator_change_is_read_from_authority_not_built_in_constants() {
        let mut layout = PanelLayout::built_in();
        layout
            .apply(
                LayoutAction::Separator {
                    path: vec![0],
                    boundary: 0,
                    action: SeparatorAction::IncreaseLeading,
                },
                constraints(),
            )
            .unwrap();
        let rects = project_panel_rects(&layout, 1_000.0, 800.0).unwrap();
        assert!(rects[&PanelRole::Browser].width > 200.0);
        assert!(rects[&PanelRole::Stage].x > 200.0);
    }

    #[test]
    fn invalid_viewport_does_not_produce_partial_geometry() {
        for (width, height) in [
            (0.0, 800.0),
            (1_000.0, 0.0),
            (f32::NAN, 800.0),
            (1_000.0, f32::INFINITY),
        ] {
            assert!(project_panel_rects(&PanelLayout::built_in(), width, height).is_none());
        }
    }

    #[test]
    fn hidden_auxiliary_redistributes_its_space_without_a_gap() {
        let mut layout = PanelLayout::built_in();
        layout
            .apply(LayoutAction::Hide(PanelRole::Inspector), constraints())
            .unwrap();
        let rects = project_panel_rects(&layout, 1_000.0, 800.0).unwrap();

        assert!(!rects.contains_key(&PanelRole::Inspector));
        assert_eq!(rects[&PanelRole::Browser].width, 250.0);
        assert_eq!(rects[&PanelRole::Stage].x, 250.0);
        assert_eq!(rects[&PanelRole::Stage].width, 750.0);
        assert_eq!(
            rects[&PanelRole::Stage].x + rects[&PanelRole::Stage].width,
            1_000.0
        );
    }

    #[test]
    fn tab_projection_emits_only_the_active_role_at_the_full_stack_rect() {
        let mut layout = PanelLayout::built_in();
        layout
            .move_tab_for_test(PanelRole::Browser, PanelRole::Inspector, constraints())
            .unwrap();
        layout.select_tab_for_test(PanelRole::Browser).unwrap();
        let rects = project_panel_rects(&layout, 1_000.0, 800.0).unwrap();

        assert!(rects.contains_key(&PanelRole::Browser));
        assert!(!rects.contains_key(&PanelRole::Inspector));
        assert_eq!(rects[&PanelRole::Browser].x, 750.0);
        assert_eq!(rects[&PanelRole::Browser].width, 250.0);
    }
}
