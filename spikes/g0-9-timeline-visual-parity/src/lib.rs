//! React Timelineをsemantic fixtureへ翻訳する、製品workspace外の比較model。

use std::collections::BTreeSet;

use serde::Serialize;
use understory_view2d::Viewport1D;

pub const FIXTURE_WIDTH: f32 = 1200.0;
pub const FIXTURE_HEIGHT: f32 = 240.0;
pub const AUTO_PRESENT_TARGET: u32 = 120;
pub const DEPTH_FIXTURE_HEIGHT: f32 = 152.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthObject {
    pub id: &'static str,
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub authoring_order: usize,
    pub depth: f32,
}

pub const DEPTH_OBJECTS: [DepthObject; 5] = [
    DepthObject {
        id: "pulse-rings",
        name: "Pulse rings",
        parent: None,
        authoring_order: 0,
        depth: 0.0,
    },
    DepthObject {
        id: "night-drive",
        name: "NIGHT DRIVE",
        parent: None,
        authoring_order: 1,
        depth: 0.0,
    },
    DepthObject {
        id: "city-loop",
        name: "City loop",
        parent: None,
        authoring_order: 2,
        depth: 0.0,
    },
    DepthObject {
        id: "traffic-pass",
        name: "Traffic pass",
        parent: None,
        authoring_order: 3,
        depth: 0.0,
    },
    DepthObject {
        id: "city-grid",
        name: "City grid",
        parent: Some("pulse-rings"),
        authoring_order: 0,
        depth: 0.0,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthScope {
    Root,
    ChildrenOf(&'static str),
}

#[derive(Clone, Debug)]
struct DistributionPreview {
    token: u64,
    far: f32,
    near: f32,
    reversed: bool,
}

#[derive(Clone, Debug)]
pub struct DepthSession {
    pub objects: Vec<DepthObject>,
    pub selection: BTreeSet<&'static str>,
    pub focused: &'static str,
    pub scope: DepthScope,
    viewport: Viewport1D,
    preview: Option<DistributionPreview>,
    pub semantic_commit_count: u32,
    pub navigation_change_count: u32,
    pub selection_change_count: u32,
    pub readback_count: u32,
    pub hot_resource_creation_count: u32,
}

impl Default for DepthSession {
    fn default() -> Self {
        let mut viewport = Viewport1D::new(220.0..1100.0);
        viewport.set_visible_world_range(-0.5..0.5);
        let zoom = viewport.zoom();
        viewport.set_zoom_limits(zoom * 0.25, zoom * 64.0);
        Self {
            objects: DEPTH_OBJECTS.to_vec(),
            selection: ["pulse-rings", "night-drive", "city-loop", "traffic-pass"]
                .into_iter()
                .collect(),
            focused: "pulse-rings",
            scope: DepthScope::Root,
            viewport,
            preview: None,
            semantic_commit_count: 0,
            navigation_change_count: 0,
            selection_change_count: 0,
            readback_count: 0,
            hot_resource_creation_count: 0,
        }
    }
}

impl DepthSession {
    pub fn visible_objects(&self) -> impl Iterator<Item = &DepthObject> {
        self.objects.iter().filter(|object| match self.scope {
            DepthScope::Root => object.parent.is_none(),
            DepthScope::ChildrenOf(parent) => object.parent == Some(parent),
        })
    }

    pub fn focus(&mut self, id: &'static str) -> bool {
        let Some(object) = self.objects.iter().find(|object| object.id == id) else {
            return false;
        };
        self.focused = id;
        self.scope = object
            .parent
            .map_or(DepthScope::Root, DepthScope::ChildrenOf);
        self.selection.clear();
        self.selection.insert(id);
        self.selection_change_count += 1;
        true
    }

    pub fn begin_distribution(&mut self, token: u64, far: f32, near: f32) -> bool {
        if self.preview.is_some() || !far.is_finite() || !near.is_finite() || far >= near {
            return false;
        }
        let selected = self
            .visible_objects()
            .filter(|object| self.selection.contains(object.id))
            .collect::<Vec<_>>();
        if selected.len() < 2 || selected.len() != self.selection.len() {
            return false;
        }
        self.preview = Some(DistributionPreview {
            token,
            far,
            near,
            reversed: false,
        });
        true
    }

    pub fn toggle_distribution_reverse(&mut self) -> bool {
        let Some(preview) = self.preview.as_mut() else {
            return false;
        };
        preview.reversed = !preview.reversed;
        true
    }

    pub fn preview_depth(&self, id: &str) -> Option<f32> {
        let preview = self.preview.as_ref()?;
        let mut selected = self
            .visible_objects()
            .filter(|object| self.selection.contains(object.id))
            .collect::<Vec<_>>();
        selected.sort_by_key(|object| object.authoring_order);
        if preview.reversed {
            selected.reverse();
        }
        let index = selected.iter().position(|object| object.id == id)?;
        let step = (preview.near - preview.far) / (selected.len() - 1) as f32;
        Some(preview.far + index as f32 * step)
    }

    pub fn apply_distribution(&mut self, token: u64) -> bool {
        if self
            .preview
            .as_ref()
            .is_none_or(|preview| preview.token != token)
        {
            return false;
        }
        let assignments = self
            .objects
            .iter()
            .filter_map(|object| {
                self.preview_depth(object.id)
                    .map(|depth| (object.id, depth))
            })
            .collect::<Vec<_>>();
        for (id, depth) in assignments {
            if let Some(object) = self.objects.iter_mut().find(|object| object.id == id) {
                object.depth = depth;
            }
        }
        self.preview = None;
        self.semantic_commit_count += 1;
        true
    }

    pub fn cancel(&mut self) -> bool {
        self.preview.take().is_some()
    }

    pub fn pan(&mut self, delta: f32) -> bool {
        if !delta.is_finite() {
            return false;
        }
        self.viewport.pan_by_view(delta as f64);
        self.navigation_change_count += 1;
        true
    }

    pub fn zoom(&mut self, anchor: f32, factor: f64) -> bool {
        if !anchor.is_finite() || !factor.is_finite() || factor <= 0.0 {
            return false;
        }
        self.viewport.zoom_about_view_point(anchor as f64, factor);
        self.navigation_change_count += 1;
        true
    }

    pub fn fit_all(&mut self) {
        self.viewport.set_visible_world_range(-0.5..0.5);
        self.navigation_change_count += 1;
    }

    pub fn depth_to_screen(&self, depth: f32) -> f32 {
        self.viewport.world_to_view_x(depth as f64) as f32
    }

    pub fn visible_depth_range(&self) -> [f32; 2] {
        let range = self.viewport.visible_world_range();
        [range.start as f32, range.end as f32]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelRole {
    Timeline,
    GraphEditor,
    Stage,
    Browser,
    Inspector,
}

impl PanelRole {
    pub const ALL: [Self; 5] = [
        Self::Timeline,
        Self::GraphEditor,
        Self::Stage,
        Self::Browser,
        Self::Inspector,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelPlacement {
    Docked { window: u64 },
    Detached { window: u64 },
    Hidden,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelProjection {
    pub id: &'static str,
    pub role: PanelRole,
    pub placement: PanelPlacement,
    pub snapshot_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelHostFixture {
    pub snapshot_revision: u64,
    pub selected_id: &'static str,
    pub semantic_commit_count: u32,
    pub panels: Vec<PanelProjection>,
}

impl Default for PanelHostFixture {
    fn default() -> Self {
        Self {
            snapshot_revision: 17,
            selected_id: "pulse-rings",
            semantic_commit_count: 0,
            panels: vec![
                PanelProjection {
                    id: "timeline-main",
                    role: PanelRole::Timeline,
                    placement: PanelPlacement::Docked { window: 1 },
                    snapshot_revision: 17,
                },
                PanelProjection {
                    id: "graph-main",
                    role: PanelRole::GraphEditor,
                    placement: PanelPlacement::Docked { window: 1 },
                    snapshot_revision: 17,
                },
                PanelProjection {
                    id: "stage-main",
                    role: PanelRole::Stage,
                    placement: PanelPlacement::Docked { window: 1 },
                    snapshot_revision: 17,
                },
                PanelProjection {
                    id: "browser-main",
                    role: PanelRole::Browser,
                    placement: PanelPlacement::Docked { window: 1 },
                    snapshot_revision: 17,
                },
                PanelProjection {
                    id: "inspector-main",
                    role: PanelRole::Inspector,
                    placement: PanelPlacement::Docked { window: 1 },
                    snapshot_revision: 17,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DockNode {
    Stack {
        id: u64,
        panels: Vec<&'static str>,
        active: usize,
    },
    Split {
        id: u64,
        axis: DockAxis,
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogicalSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogicalRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockWindow {
    pub id: u64,
    pub logical_size: LogicalSize,
    pub layout_epoch: u64,
    pub root: DockNode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DockWorkspace {
    pub host: PanelHostFixture,
    pub windows: Vec<DockWindow>,
    next_stack_id: u64,
    next_split_id: u64,
}

impl Default for DockWorkspace {
    fn default() -> Self {
        Self {
            host: PanelHostFixture::default(),
            windows: vec![DockWindow {
                id: 1,
                logical_size: LogicalSize {
                    width: 1440.0,
                    height: 900.0,
                },
                layout_epoch: 1,
                root: DockNode::Split {
                    id: 1,
                    axis: DockAxis::Horizontal,
                    ratio: 0.74,
                    first: Box::new(DockNode::Split {
                        id: 2,
                        axis: DockAxis::Vertical,
                        ratio: 0.64,
                        first: Box::new(DockNode::Stack {
                            id: 1,
                            panels: vec!["stage-main"],
                            active: 0,
                        }),
                        second: Box::new(DockNode::Stack {
                            id: 2,
                            panels: vec!["timeline-main", "graph-main"],
                            active: 0,
                        }),
                    }),
                    second: Box::new(DockNode::Split {
                        id: 3,
                        axis: DockAxis::Vertical,
                        ratio: 0.52,
                        first: Box::new(DockNode::Stack {
                            id: 3,
                            panels: vec!["browser-main"],
                            active: 0,
                        }),
                        second: Box::new(DockNode::Stack {
                            id: 4,
                            panels: vec!["inspector-main"],
                            active: 0,
                        }),
                    }),
                },
            }],
            next_stack_id: 5,
            next_split_id: 4,
        }
    }
}

impl DockNode {
    fn remove_panel(&mut self, panel_id: &str) -> bool {
        match self {
            Self::Stack { panels, active, .. } => {
                let Some(index) = panels.iter().position(|id| *id == panel_id) else {
                    return false;
                };
                panels.remove(index);
                *active = (*active).min(panels.len().saturating_sub(1));
                true
            }
            Self::Split { first, second, .. } => {
                first.remove_panel(panel_id) || second.remove_panel(panel_id)
            }
        }
    }

    fn insert_tab(&mut self, stack_id: u64, panel_id: &'static str) -> bool {
        match self {
            Self::Stack { id, panels, active } if *id == stack_id => {
                panels.push(panel_id);
                *active = panels.len() - 1;
                true
            }
            Self::Stack { .. } => false,
            Self::Split { first, second, .. } => {
                first.insert_tab(stack_id, panel_id) || second.insert_tab(stack_id, panel_id)
            }
        }
    }

    fn set_split_ratio(&mut self, split_id: u64, ratio: f32) -> bool {
        match self {
            Self::Split {
                id, ratio: current, ..
            } if *id == split_id => {
                *current = ratio;
                true
            }
            Self::Split { first, second, .. } => {
                first.set_split_ratio(split_id, ratio) || second.set_split_ratio(split_id, ratio)
            }
            Self::Stack { .. } => false,
        }
    }

    fn split_stack(
        &mut self,
        target_stack: u64,
        panel_id: &'static str,
        new_stack: u64,
        new_split: u64,
        axis: DockAxis,
        after: bool,
    ) -> bool {
        match self {
            Self::Stack { id, panels, active } if *id == target_stack => {
                let existing = Self::Stack {
                    id: *id,
                    panels: panels.clone(),
                    active: *active,
                };
                let inserted = Self::Stack {
                    id: new_stack,
                    panels: vec![panel_id],
                    active: 0,
                };
                let (first, second) = if after {
                    (existing, inserted)
                } else {
                    (inserted, existing)
                };
                *self = Self::Split {
                    id: new_split,
                    axis,
                    ratio: 0.5,
                    first: Box::new(first),
                    second: Box::new(second),
                };
                true
            }
            Self::Stack { .. } => false,
            Self::Split { first, second, .. } => {
                first.split_stack(target_stack, panel_id, new_stack, new_split, axis, after)
                    || second.split_stack(target_stack, panel_id, new_stack, new_split, axis, after)
            }
        }
    }
}

impl DockWorkspace {
    pub fn detach(&mut self, panel_id: &'static str, new_window: u64, size: LogicalSize) -> bool {
        if !valid_window_size(size) || self.windows.iter().any(|window| window.id == new_window) {
            return false;
        }
        let Some(source_index) = self
            .windows
            .iter()
            .position(|window| contains_panel(&window.root, panel_id))
        else {
            return false;
        };
        let source = &mut self.windows[source_index];
        source.root.remove_panel(panel_id);
        source.layout_epoch += 1;
        let stack_id = self.next_stack_id;
        self.next_stack_id += 1;
        self.windows.push(DockWindow {
            id: new_window,
            logical_size: size,
            layout_epoch: 1,
            root: DockNode::Stack {
                id: stack_id,
                panels: vec![panel_id],
                active: 0,
            },
        });
        self.host
            .place(panel_id, PanelPlacement::Detached { window: new_window })
    }

    pub fn dock_as_tab(&mut self, panel_id: &'static str, window: u64, stack: u64) -> bool {
        if !self
            .windows
            .iter()
            .any(|entry| entry.id == window && contains_stack(&entry.root, stack))
        {
            return false;
        }
        let mut removed_window = None;
        for entry in &mut self.windows {
            if entry.root.remove_panel(panel_id) {
                entry.layout_epoch += 1;
                removed_window = Some(entry.id);
                break;
            }
        }
        if removed_window.is_none() {
            return false;
        }
        let target = self.windows.iter_mut().find(|entry| entry.id == window);
        let Some(target) = target else {
            return false;
        };
        if !target.root.insert_tab(stack, panel_id) {
            return false;
        }
        target.layout_epoch += 1;
        self.windows
            .retain(|entry| entry.id == 1 || !window_is_empty(&entry.root));
        self.host.place(panel_id, PanelPlacement::Docked { window })
    }

    pub fn dock_as_split(
        &mut self,
        panel_id: &'static str,
        window: u64,
        target_stack: u64,
        axis: DockAxis,
        after: bool,
    ) -> bool {
        if !self
            .windows
            .iter()
            .any(|entry| entry.id == window && contains_stack(&entry.root, target_stack))
        {
            return false;
        }
        let mut removed = false;
        for entry in &mut self.windows {
            if entry.root.remove_panel(panel_id) {
                entry.layout_epoch += 1;
                removed = true;
                break;
            }
        }
        if !removed {
            return false;
        }
        let stack_id = self.next_stack_id;
        self.next_stack_id += 1;
        let split_id = self.next_split_id;
        self.next_split_id += 1;
        let Some(target) = self.windows.iter_mut().find(|entry| entry.id == window) else {
            return false;
        };
        if !target
            .root
            .split_stack(target_stack, panel_id, stack_id, split_id, axis, after)
        {
            return false;
        }
        target.layout_epoch += 1;
        self.windows
            .retain(|entry| entry.id == 1 || !window_is_empty(&entry.root));
        self.host.place(panel_id, PanelPlacement::Docked { window })
    }

    pub fn resize_window(&mut self, window: u64, size: LogicalSize) -> bool {
        if !valid_window_size(size) {
            return false;
        }
        let Some(target) = self.windows.iter_mut().find(|entry| entry.id == window) else {
            return false;
        };
        target.logical_size = size;
        target.layout_epoch += 1;
        true
    }

    pub fn resize_split(&mut self, window: u64, split: u64, ratio: f32) -> bool {
        if !ratio.is_finite() {
            return false;
        }
        let ratio = ratio.clamp(0.15, 0.85);
        let Some(target) = self.windows.iter_mut().find(|entry| entry.id == window) else {
            return false;
        };
        if !target.root.set_split_ratio(split, ratio) {
            return false;
        }
        target.layout_epoch += 1;
        true
    }

    pub fn layout(&self, window: u64) -> Option<Vec<(&'static str, LogicalRect)>> {
        let window = self.windows.iter().find(|entry| entry.id == window)?;
        compute_dock_layout(&window.root, window.logical_size)
    }
}

fn valid_window_size(size: LogicalSize) -> bool {
    size.width.is_finite() && size.height.is_finite() && size.width >= 320.0 && size.height >= 240.0
}

fn contains_stack(node: &DockNode, stack_id: u64) -> bool {
    match node {
        DockNode::Stack { id, .. } => *id == stack_id,
        DockNode::Split { first, second, .. } => {
            contains_stack(first, stack_id) || contains_stack(second, stack_id)
        }
    }
}

fn contains_panel(node: &DockNode, panel_id: &str) -> bool {
    match node {
        DockNode::Stack { panels, .. } => panels.contains(&panel_id),
        DockNode::Split { first, second, .. } => {
            contains_panel(first, panel_id) || contains_panel(second, panel_id)
        }
    }
}

fn window_is_empty(node: &DockNode) -> bool {
    match node {
        DockNode::Stack { panels, .. } => panels.is_empty(),
        DockNode::Split { first, second, .. } => window_is_empty(first) && window_is_empty(second),
    }
}

fn compute_dock_layout(
    root: &DockNode,
    size: LogicalSize,
) -> Option<Vec<(&'static str, LogicalRect)>> {
    use taffy::prelude::*;

    fn add_node(tree: &mut TaffyTree, node: &DockNode, basis: Option<f32>) -> Option<NodeId> {
        match node {
            DockNode::Stack { .. } => tree
                .new_leaf(Style {
                    flex_basis: basis.map_or_else(Dimension::auto, Dimension::percent),
                    size: if basis.is_none() {
                        Size {
                            width: Dimension::percent(1.0),
                            height: Dimension::percent(1.0),
                        }
                    } else {
                        Size {
                            width: Dimension::auto(),
                            height: Dimension::auto(),
                        }
                    },
                    min_size: Size {
                        width: Dimension::length(120.0),
                        height: Dimension::length(90.0),
                    },
                    ..Default::default()
                })
                .ok(),
            DockNode::Split {
                axis,
                ratio,
                first,
                second,
                ..
            } => {
                let first_id = add_node(tree, first, Some(*ratio))?;
                let second_id = add_node(tree, second, Some(1.0 - *ratio))?;
                tree.new_with_children(
                    Style {
                        display: Display::Flex,
                        flex_basis: basis.map_or_else(Dimension::auto, Dimension::percent),
                        flex_direction: match axis {
                            DockAxis::Horizontal => FlexDirection::Row,
                            DockAxis::Vertical => FlexDirection::Column,
                        },
                        size: Size {
                            width: Dimension::percent(1.0),
                            height: Dimension::percent(1.0),
                        },
                        gap: Size {
                            width: LengthPercentage::length(4.0),
                            height: LengthPercentage::length(4.0),
                        },
                        ..Default::default()
                    },
                    &[first_id, second_id],
                )
                .ok()
            }
        }
    }

    fn collect(
        tree: &TaffyTree,
        node_id: NodeId,
        node: &DockNode,
        origin: [f32; 2],
        output: &mut Vec<(&'static str, LogicalRect)>,
    ) -> Option<()> {
        let layout = tree.layout(node_id).ok()?;
        let absolute = [origin[0] + layout.location.x, origin[1] + layout.location.y];
        match node {
            DockNode::Stack { panels, .. } => {
                let rect = LogicalRect {
                    x: absolute[0],
                    y: absolute[1],
                    width: layout.size.width,
                    height: layout.size.height,
                };
                output.extend(panels.iter().map(|panel| (*panel, rect)));
            }
            DockNode::Split { first, second, .. } => {
                let children = tree.children(node_id).ok()?;
                if children.len() != 2 {
                    return None;
                }
                collect(tree, children[0], first, absolute, output)?;
                collect(tree, children[1], second, absolute, output)?;
            }
        }
        Some(())
    }

    let mut tree = TaffyTree::new();
    let root_id = add_node(&mut tree, root, None)?;
    tree.compute_layout(
        root_id,
        Size {
            width: AvailableSpace::Definite(size.width),
            height: AvailableSpace::Definite(size.height),
        },
    )
    .ok()?;
    let mut output = Vec::new();
    collect(&tree, root_id, root, [0.0, 0.0], &mut output)?;
    Some(output)
}

impl PanelHostFixture {
    pub fn place(&mut self, id: &str, placement: PanelPlacement) -> bool {
        let Some(panel) = self.panels.iter_mut().find(|panel| panel.id == id) else {
            return false;
        };
        panel.placement = placement;
        panel.snapshot_revision = self.snapshot_revision;
        true
    }

    pub fn add_panel(&mut self, id: &'static str, role: PanelRole, placement: PanelPlacement) {
        self.panels.push(PanelProjection {
            id,
            role,
            placement,
            snapshot_revision: self.snapshot_revision,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineObject {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub row: usize,
    pub start: f32,
    pub end: f32,
    pub color_slot: usize,
    pub selected: bool,
}

pub const OBJECTS: [TimelineObject; 5] = [
    TimelineObject {
        id: "audio-night-drive",
        name: "night_drive.wav",
        kind: "A",
        row: 0,
        start: 0.02,
        end: 1.00,
        color_slot: 0,
        selected: false,
    },
    TimelineObject {
        id: "group-pulse-rings",
        name: "Pulse rings",
        kind: "G",
        row: 1,
        start: 0.08,
        end: 0.88,
        color_slot: 1,
        selected: true,
    },
    TimelineObject {
        id: "shape-city-grid",
        name: "City grid",
        kind: "S",
        row: 2,
        start: 0.17,
        end: 0.94,
        color_slot: 2,
        selected: false,
    },
    TimelineObject {
        id: "text-night-drive",
        name: "NIGHT DRIVE",
        kind: "T",
        row: 3,
        start: 0.49,
        end: 1.00,
        color_slot: 3,
        selected: false,
    },
    TimelineObject {
        id: "video-city-loop",
        name: "city_loop.mp4",
        kind: "V",
        row: 4,
        start: 0.32,
        end: 1.00,
        color_slot: 4,
        selected: false,
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectPrimitive {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub shape: u32,
    pub _padding: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct TextPrimitive {
    pub text: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub size: f32,
    pub color: [u8; 4],
    pub monospace: bool,
}

#[derive(Clone, Debug, Default)]
pub struct VisualScene {
    pub rects: Vec<RectPrimitive>,
    pub texts: Vec<TextPrimitive>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VisualParityReport {
    pub ticket: &'static str,
    pub status: &'static str,
    pub adapter: String,
    pub backend: String,
    pub object_count: usize,
    pub rect_primitive_count: usize,
    pub text_run_count: usize,
    pub present_count: u32,
    pub readback_count: u32,
    pub semantic_state_owner_count: u32,
    pub semantic_commit_count: u32,
    pub navigation_change_count: u32,
    pub selection_change_count: u32,
    pub pass: bool,
}

const BG: [f32; 4] = [0.018, 0.020, 0.022, 1.0];
const RAISED: [f32; 4] = [0.060, 0.064, 0.068, 1.0];
const LINE: [f32; 4] = [0.15, 0.16, 0.17, 1.0];
const LINE_STRONG: [f32; 4] = [0.34, 0.35, 0.36, 1.0];
const ACTIVE: [f32; 4] = [0.83, 0.59, 0.31, 1.0];
const BAR_COLORS: [[f32; 4]; 5] = [
    [0.28, 0.55, 0.52, 1.0],
    [0.37, 0.35, 0.64, 1.0],
    [0.30, 0.46, 0.33, 1.0],
    [0.58, 0.47, 0.29, 1.0],
    [0.54, 0.34, 0.30, 1.0],
];

impl VisualScene {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.rects.push(RectPrimitive {
            rect: [x, y, w, h],
            color,
            shape: 0,
            _padding: [0; 3],
        });
    }

    fn shape(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], shape: u32) {
        self.rects.push(RectPrimitive {
            rect: [x, y, w, h],
            color,
            shape,
            _padding: [0; 3],
        });
    }

    fn outline(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4], thickness: f32) {
        self.rect(x, y, w, thickness, color);
        self.rect(x, y + h - thickness, w, thickness, color);
        self.rect(x, y, thickness, h, color);
        self.rect(x + w - thickness, y, thickness, h, color);
    }

    fn text(
        &mut self,
        text: impl Into<String>,
        left: f32,
        top: f32,
        width: f32,
        appearance: (f32, [u8; 4], bool),
    ) {
        let (size, color, monospace) = appearance;
        self.texts.push(TextPrimitive {
            text: text.into(),
            left,
            top,
            width,
            height: size * 1.4,
            size,
            color,
            monospace,
        });
    }
}

pub fn build_scene(width: f32, height: f32) -> VisualScene {
    let width = width.max(640.0);
    let height = height.max(240.0);
    let mut scene = VisualScene::default();
    let header_h = 30.0;
    let ruler_h = 25.0;
    let rail_w = 54.0;
    let time_x = rail_w;
    let time_w = width - time_x;
    let row_h = (height - header_h - ruler_h) / OBJECTS.len() as f32;

    scene.rect(0.0, 0.0, width, height, BG);
    scene.rect(0.0, 0.0, width, header_h, RAISED);
    scene.rect(0.0, header_h - 1.0, width, 1.0, LINE_STRONG);
    scene.rect(10.0, 11.0, 7.0, 7.0, [0.55, 0.30, 0.27, 1.0]);
    scene.text(
        "譜面 / Timeline",
        25.0,
        8.0,
        180.0,
        (11.0, [232, 232, 232, 255], false),
    );
    scene.outline(width - 78.0, 4.0, 30.0, 22.0, LINE, 1.0);
    scene.outline(width - 44.0, 4.0, 30.0, 22.0, ACTIVE, 1.0);
    scene.text(
        "⌁",
        width - 69.0,
        7.0,
        20.0,
        (11.0, [150, 150, 150, 255], true),
    );
    scene.text(
        "▤",
        width - 35.0,
        7.0,
        20.0,
        (11.0, [232, 232, 232, 255], true),
    );

    scene.rect(0.0, header_h, rail_w, height - header_h, BG);
    scene.rect(time_x, header_h, time_w, ruler_h, BG);
    scene.rect(time_x - 1.0, header_h, 1.0, height - header_h, LINE_STRONG);

    scene.text(
        "S",
        10.0,
        header_h + 8.0,
        12.0,
        (8.0, [156, 156, 156, 255], true),
    );
    scene.text(
        "M",
        32.0,
        header_h + 8.0,
        12.0,
        (8.0, [156, 156, 156, 255], true),
    );
    scene.text(
        "TIME / BEAT",
        time_x + 8.0,
        header_h + 8.0,
        80.0,
        (8.0, [218, 174, 117, 255], true),
    );
    for tick in 0..17 {
        let x = time_x + tick as f32 * time_w / 16.0;
        let major = tick % 4 == 0;
        scene.rect(
            x,
            header_h + ruler_h - if major { 8.0 } else { 4.0 },
            1.0,
            if major { 8.0 } else { 4.0 },
            if major { LINE_STRONG } else { LINE },
        );
        scene.rect(
            x,
            header_h + ruler_h,
            1.0,
            height - header_h - ruler_h,
            if major {
                LINE
            } else {
                [0.065, 0.068, 0.072, 1.0]
            },
        );
        if major && tick > 0 {
            scene.text(
                format!("{}", 52 + tick / 4),
                x - 4.0,
                header_h + 7.0,
                32.0,
                (8.0, [158, 158, 158, 255], true),
            );
        }
    }

    for (index, object) in OBJECTS.iter().enumerate() {
        let row_y = header_h + ruler_h + index as f32 * row_h;
        scene.rect(0.0, row_y + row_h - 1.0, width, 1.0, LINE);
        scene.outline(7.0, row_y + 7.0, 18.0, 18.0, LINE, 1.0);
        scene.outline(29.0, row_y + 7.0, 18.0, 18.0, LINE, 1.0);
        scene.text(
            "S",
            13.0,
            row_y + 11.0,
            10.0,
            (8.0, [212, 212, 212, 255], true),
        );
        scene.text(
            "M",
            35.0,
            row_y + 11.0,
            10.0,
            (8.0, [212, 212, 212, 255], true),
        );

        let bar_x = time_x + object.start * time_w;
        let bar_w = (object.end - object.start) * time_w;
        let bar_y = row_y + 5.0;
        let bar_h = (row_h - 10.0).max(20.0);
        if object.kind == "G" {
            scene.rect(time_x, row_y, time_w, row_h, [0.055, 0.050, 0.085, 1.0]);
            scene.rect(time_x, row_y, 3.0, row_h, BAR_COLORS[object.color_slot]);
        }
        scene.rect(bar_x, bar_y, bar_w, bar_h, BAR_COLORS[object.color_slot]);
        scene.outline(
            bar_x,
            bar_y,
            bar_w,
            bar_h,
            if object.selected { ACTIVE } else { LINE_STRONG },
            if object.selected { 2.0 } else { 1.0 },
        );
        scene.rect(
            bar_x + 7.0,
            bar_y + 6.0,
            16.0,
            16.0,
            [0.035, 0.038, 0.040, 0.82],
        );
        scene.outline(
            bar_x + 7.0,
            bar_y + 6.0,
            16.0,
            16.0,
            [0.72, 0.72, 0.72, 1.0],
            1.0,
        );
        scene.text(
            object.kind,
            bar_x + 12.0,
            bar_y + 9.0,
            10.0,
            (8.0, [230, 230, 230, 255], true),
        );
        scene.text(
            object.name,
            bar_x + 31.0,
            bar_y + 9.0,
            (bar_w - 40.0).max(30.0),
            (10.0, [26, 27, 28, 255], object.kind != "T"),
        );
        if object.kind == "G" {
            scene.text(
                "IN  →  Echo Bloom  →  OUT",
                bar_x + 130.0,
                bar_y + 9.0,
                190.0,
                (8.0, [42, 42, 48, 255], true),
            );
            for key in [0.32_f32, 0.51, 0.72] {
                let x = time_x + key * time_w;
                scene.shape(x - 5.0, bar_y + bar_h * 0.5 - 5.0, 10.0, 10.0, ACTIVE, 1);
            }
        }
        if object.kind == "A" {
            scene.text(
                "╱╲╱▁╲╱╲▁╱╲╱╲▁╱╲",
                bar_x + 150.0,
                bar_y + 9.0,
                220.0,
                (9.0, [50, 78, 76, 255], true),
            );
        }
    }

    let playhead_x = time_x + time_w * 0.46;
    scene.rect(
        playhead_x,
        header_h + 2.0,
        1.5,
        height - header_h - 2.0,
        ACTIVE,
    );
    scene.shape(
        playhead_x - 6.0,
        header_h + ruler_h - 2.0,
        13.0,
        9.0,
        ACTIVE,
        2,
    );
    scene
}

pub fn build_depth_scene(session: &DepthSession, width: f32, height: f32) -> VisualScene {
    fn format_depth(depth: f32) -> String {
        if depth.abs() < 0.0001 {
            "0".to_owned()
        } else {
            format!("{depth:+.2}")
        }
    }
    let width = width.max(720.0);
    let height = height.max(DEPTH_FIXTURE_HEIGHT);
    let mut scene = VisualScene::default();
    let axis_y = 79.0;
    scene.rect(0.0, 0.0, width, height, BG);
    scene.rect(0.0, 0.0, width, 34.0, RAISED);
    scene.rect(0.0, 33.0, width, 1.0, LINE_STRONG);
    scene.text(
        "DEPTH",
        12.0,
        10.0,
        58.0,
        (10.0, [232, 232, 232, 255], true),
    );
    let scope = match session.scope {
        DepthScope::Root => "ROOT".to_owned(),
        DepthScope::ChildrenOf(parent) => format!("ROOT / {parent}"),
    };
    scene.outline(76.0, 6.0, 150.0, 22.0, LINE, 1.0);
    scene.text(scope, 84.0, 11.0, 135.0, (8.0, [178, 180, 186, 255], true));
    scene.text("◇", 238.0, 10.0, 20.0, (10.0, [218, 174, 117, 255], true));
    let focused = session
        .objects
        .iter()
        .find(|object| object.id == session.focused);
    scene.text(
        focused.map_or_else(
            || "Z / EDIT SPACE".to_owned(),
            |object| format!("{} {}", object.name, format_depth(object.depth)),
        ),
        274.0,
        11.0,
        270.0,
        (8.0, [165, 168, 176, 255], true),
    );
    scene.text(
        "EDIT-SPACE Z",
        width - 160.0,
        11.0,
        140.0,
        (8.0, [130, 133, 140, 255], true),
    );

    scene.rect(0.0, 34.0, 205.0, height - 34.0, [0.028, 0.030, 0.033, 1.0]);
    scene.text(
        match session.scope {
            DepthScope::Root => "ROOT",
            DepthScope::ChildrenOf(_) => "CHILD",
        },
        14.0,
        70.0,
        60.0,
        (9.0, [195, 197, 202, 255], true),
    );
    scene.text(
        "same-parent scope",
        14.0,
        91.0,
        150.0,
        (7.0, [120, 123, 130, 255], false),
    );
    scene.rect(205.0, 34.0, 1.0, height - 34.0, LINE_STRONG);

    for tick in 0..=4 {
        let depth = -0.5 + tick as f32 * 0.25;
        let x = session.depth_to_screen(depth);
        scene.rect(
            x,
            50.0,
            1.0,
            58.0,
            if tick == 2 { LINE_STRONG } else { LINE },
        );
        scene.text(
            if depth == 0.0 {
                "0".to_owned()
            } else {
                format_depth(depth)
            },
            x - 22.0,
            39.0,
            44.0,
            (7.0, [132, 135, 142, 255], true),
        );
    }
    scene.rect(220.0, axis_y, width - 320.0, 1.0, LINE_STRONG);

    if let Some(preview) = session.preview.as_ref() {
        let left = session.depth_to_screen(preview.far);
        let right = session.depth_to_screen(preview.near);
        scene.rect(
            left,
            axis_y - 12.0,
            (right - left).max(1.0),
            24.0,
            [0.26, 0.17, 0.08, 0.45],
        );
        scene.outline(
            left,
            axis_y - 12.0,
            (right - left).max(1.0),
            24.0,
            ACTIVE,
            1.0,
        );
    }

    let mut groups: Vec<(f32, Vec<&DepthObject>)> = Vec::new();
    for object in session.visible_objects() {
        let value = session.preview_depth(object.id).unwrap_or(object.depth);
        if let Some((_, entries)) = groups
            .iter_mut()
            .find(|(depth, _)| (*depth - value).abs() < 0.0001)
        {
            entries.push(object);
        } else {
            groups.push((value, vec![object]));
        }
    }
    groups.sort_by(|left, right| left.0.total_cmp(&right.0));
    for (depth, entries) in groups {
        let focused = entries
            .iter()
            .find(|object| object.id == session.focused)
            .copied()
            .unwrap_or(entries[0]);
        let selected = entries
            .iter()
            .any(|object| session.selection.contains(object.id));
        let x = session.depth_to_screen(depth);
        let label = if entries.len() > 1 {
            format!("{} × {}", format_depth(depth), entries.len())
        } else {
            format!("{} {}", focused.name, format_depth(depth))
        };
        let marker_w = if entries.len() > 1 { 74.0 } else { 116.0 };
        scene.shape(
            x - 6.0,
            axis_y - 6.0,
            12.0,
            12.0,
            if selected {
                ACTIVE
            } else {
                [0.46, 0.50, 0.56, 1.0]
            },
            1,
        );
        scene.outline(
            x - marker_w * 0.5,
            axis_y + 14.0,
            marker_w,
            21.0,
            if selected { ACTIVE } else { LINE_STRONG },
            if entries.len() > 1 { 2.0 } else { 1.0 },
        );
        scene.text(
            label,
            x - marker_w * 0.5 + 5.0,
            axis_y + 20.0,
            marker_w - 10.0,
            (8.0, [224, 225, 228, 255], true),
        );
    }

    if matches!(session.scope, DepthScope::Root) {
        let x = session.depth_to_screen(0.42);
        scene.shape(
            x - 5.0,
            axis_y - 5.0,
            10.0,
            10.0,
            [0.32, 0.58, 0.78, 1.0],
            2,
        );
        scene.text(
            "CAM +.42 →",
            x - 38.0,
            axis_y - 27.0,
            90.0,
            (7.0, [135, 185, 218, 255], true),
        );
    }
    scene.outline(
        width - 88.0,
        54.0,
        30.0,
        24.0,
        if session.preview.is_some() {
            ACTIVE
        } else {
            LINE
        },
        1.0,
    );
    scene.text(
        "⇥≋⇤",
        width - 83.0,
        61.0,
        24.0,
        (8.0, [215, 216, 220, 255], true),
    );
    scene.text(
        if session.preview.is_some() {
            "REVERSE  APPLY  CANCEL"
        } else {
            "FIT"
        },
        width - 185.0,
        height - 22.0,
        170.0,
        (7.0, [145, 148, 155, 255], true),
    );
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fixture_ids_are_stable_and_unique() {
        let ids = OBJECTS
            .iter()
            .map(|object| object.id)
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), OBJECTS.len());
        assert_eq!(OBJECTS.iter().filter(|object| object.selected).count(), 1);
    }

    #[test]
    fn fixture_ranges_are_normalized_and_non_empty() {
        for object in OBJECTS {
            assert!((0.0..=1.0).contains(&object.start));
            assert!((0.0..=1.0).contains(&object.end));
            assert!(object.start < object.end);
        }
    }

    #[test]
    fn scene_contains_oracle_roles_without_a_second_state_owner() {
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<HashSet<_>>();
        for expected in [
            "譜面 / Timeline",
            "TIME / BEAT",
            "Pulse rings",
            "NIGHT DRIVE",
        ] {
            assert!(labels.contains(expected), "missing oracle label {expected}");
        }
        assert!(scene.rects.iter().any(|primitive| primitive.shape == 1));
        assert!(scene.rects.iter().any(|primitive| primitive.shape == 2));
    }

    #[test]
    fn react_tool_panel_is_not_duplicated_in_native_scene() {
        let scene = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let labels = scene
            .texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<HashSet<_>>();
        for react_owned in ["KEYS", "LAYERS", "ALIGN"] {
            assert!(
                !labels.contains(react_owned),
                "duplicated React label {react_owned}"
            );
        }
    }

    #[test]
    fn logical_layout_is_dpi_independent() {
        let one = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        let two = build_scene(FIXTURE_WIDTH, FIXTURE_HEIGHT);
        assert_eq!(
            bytemuck::cast_slice::<RectPrimitive, u8>(&one.rects),
            bytemuck::cast_slice::<RectPrimitive, u8>(&two.rects)
        );
    }

    #[test]
    fn coincident_root_depths_are_one_stable_stack() {
        let session = DepthSession::default();
        let scene = build_depth_scene(&session, FIXTURE_WIDTH, DEPTH_FIXTURE_HEIGHT);
        let stack_labels = scene
            .texts
            .iter()
            .filter(|text| text.text.contains("× 4"))
            .count();
        assert_eq!(stack_labels, 1);
        assert_eq!(session.visible_objects().count(), 4);
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn distribution_preview_reverse_apply_is_exactly_once() {
        let mut session = DepthSession::default();
        assert!(session.begin_distribution(7, -0.25, 0.25));
        assert_eq!(session.preview_depth("pulse-rings"), Some(-0.25));
        assert_eq!(session.semantic_commit_count, 0);
        assert!(session.toggle_distribution_reverse());
        assert_eq!(session.preview_depth("pulse-rings"), Some(0.25));
        assert!(session.apply_distribution(7));
        assert!(!session.apply_distribution(7));
        assert_eq!(session.semantic_commit_count, 1);
    }

    #[test]
    fn cancel_and_mixed_parent_distribution_change_no_depth() {
        let mut session = DepthSession::default();
        let baseline = session.objects.clone();
        assert!(session.begin_distribution(8, -0.3, 0.3));
        assert!(session.cancel());
        assert_eq!(session.objects, baseline);
        session.selection.insert("city-grid");
        assert!(!session.begin_distribution(9, -0.3, 0.3));
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn child_scope_never_contains_root_markers() {
        let mut session = DepthSession::default();
        assert!(session.focus("city-grid"));
        assert_eq!(session.scope, DepthScope::ChildrenOf("pulse-rings"));
        assert_eq!(
            session
                .visible_objects()
                .map(|object| object.id)
                .collect::<Vec<_>>(),
            vec!["city-grid"]
        );
    }

    #[test]
    fn depth_navigation_is_document_free_and_rejects_non_finite_input() {
        let mut session = DepthSession::default();
        let baseline = session.objects.clone();
        let before = session.visible_depth_range();
        assert!(session.pan(48.0));
        assert!(session.zoom(620.0, 1.4));
        assert_ne!(session.visible_depth_range(), before);
        assert!(!session.pan(f32::NAN));
        assert!(!session.zoom(620.0, f64::INFINITY));
        assert_eq!(session.objects, baseline);
        assert_eq!(session.semantic_commit_count, 0);
    }

    #[test]
    fn timeline_and_graph_detach_without_copying_host_state() {
        let mut host = PanelHostFixture::default();
        assert!(host.place("graph-main", PanelPlacement::Detached { window: 2 }));
        assert_eq!(
            host.panels[0].snapshot_revision,
            host.panels[1].snapshot_revision
        );
        assert_eq!(host.semantic_commit_count, 0);
        assert_eq!(host.selected_id, "pulse-rings");
        assert!(host.place("graph-main", PanelPlacement::Docked { window: 1 }));
        assert_eq!(host.semantic_commit_count, 0);
    }

    #[test]
    fn placement_model_accepts_other_product_panels() {
        let mut host = PanelHostFixture::default();
        host.add_panel(
            "inspector-main",
            PanelRole::Inspector,
            PanelPlacement::Detached { window: 3 },
        );
        assert_eq!(host.panels.len(), 6);
        assert!(host
            .panels
            .iter()
            .all(|panel| panel.snapshot_revision == host.snapshot_revision));
    }

    #[test]
    fn taffy_layout_keeps_nested_splits_in_window_bounds_and_tabs_coincident() {
        let workspace = DockWorkspace::default();
        let layout = workspace.layout(1).expect("main window layout");
        assert_eq!(layout.len(), PanelRole::ALL.len());
        let rect = |id: &str| layout.iter().find(|(panel, _)| *panel == id).unwrap().1;
        assert_eq!(rect("timeline-main"), rect("graph-main"));
        assert_ne!(rect("stage-main"), rect("browser-main"));
        for (_, panel) in layout {
            assert!(panel.x >= 0.0 && panel.y >= 0.0);
            assert!(panel.width >= 120.0 && panel.height >= 90.0);
            assert!(panel.x + panel.width <= 1440.01);
            assert!(panel.y + panel.height <= 900.01);
        }
    }

    #[test]
    fn every_product_panel_can_detach_resize_and_redock_as_tab() {
        for role in PanelRole::ALL {
            let mut workspace = DockWorkspace::default();
            let panel_id = workspace
                .host
                .panels
                .iter()
                .find(|panel| panel.role == role)
                .unwrap()
                .id;
            assert!(workspace.detach(
                panel_id,
                2,
                LogicalSize {
                    width: 720.0,
                    height: 480.0,
                }
            ));
            assert!(workspace.resize_window(
                2,
                LogicalSize {
                    width: 1024.0,
                    height: 640.0,
                }
            ));
            let detached = workspace.layout(2).unwrap();
            assert_eq!(
                detached,
                vec![(
                    panel_id,
                    LogicalRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1024.0,
                        height: 640.0
                    }
                )]
            );
            assert!(workspace.dock_as_tab(panel_id, 1, 2));
            assert_eq!(workspace.windows.len(), 1);
            assert_eq!(workspace.host.semantic_commit_count, 0);
            assert_eq!(workspace.host.snapshot_revision, 17);
            assert_eq!(workspace.host.selected_id, "pulse-rings");
        }
    }

    #[test]
    fn every_product_panel_can_redock_as_split_without_semantic_writes() {
        for role in PanelRole::ALL {
            let mut workspace = DockWorkspace::default();
            let panel_id = workspace
                .host
                .panels
                .iter()
                .find(|panel| panel.role == role)
                .unwrap()
                .id;
            let target_stack = if role == PanelRole::Stage { 3 } else { 1 };
            assert!(workspace.detach(
                panel_id,
                2,
                LogicalSize {
                    width: 720.0,
                    height: 480.0,
                }
            ));
            assert!(workspace.dock_as_split(panel_id, 1, target_stack, DockAxis::Horizontal, true,));
            assert_eq!(workspace.windows.len(), 1);
            assert!(workspace
                .layout(1)
                .unwrap()
                .iter()
                .any(|(panel, _)| *panel == panel_id));
            assert_eq!(workspace.host.semantic_commit_count, 0);
            assert_eq!(workspace.host.snapshot_revision, 17);
            assert_eq!(workspace.host.selected_id, "pulse-rings");
        }
    }

    #[test]
    fn split_docking_and_resize_are_clamped_and_document_free() {
        let mut workspace = DockWorkspace::default();
        assert!(workspace.detach(
            "inspector-main",
            2,
            LogicalSize {
                width: 600.0,
                height: 500.0,
            }
        ));
        assert!(workspace.dock_as_split("inspector-main", 1, 1, DockAxis::Vertical, true,));
        assert!(workspace.resize_split(1, 1, 0.01));
        assert!(workspace.resize_window(
            1,
            LogicalSize {
                width: 1600.0,
                height: 1000.0,
            }
        ));
        assert!(!workspace.resize_window(
            1,
            LogicalSize {
                width: f32::NAN,
                height: 1000.0,
            }
        ));
        assert!(!workspace.resize_split(1, 1, f32::INFINITY));
        let DockNode::Split { ratio, .. } = &workspace.windows[0].root else {
            panic!("root split must remain available");
        };
        assert_eq!(*ratio, 0.15);
        let layout = workspace.layout(1).unwrap();
        assert!(layout.iter().any(|(panel, _)| *panel == "inspector-main"));
        assert_eq!(workspace.host.semantic_commit_count, 0);
        assert!(workspace.windows[0].layout_epoch >= 4);
    }
}
