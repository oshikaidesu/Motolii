use std::collections::BTreeSet;
use std::fmt;

use dioxus_workbench::{
    DockZone, LayoutNode, PanelId, PanelLayout, PanelPlacement, SplitAxis, SplitId, TileId,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Panel {
    Media,
    Effects,
    Create,
    Colors,
    Stage,
    Output,
    Inspector,
    Utility,
    Settings,
    Timeline,
    Ease,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    Left,
    Center,
    Right,
    Bottom,
}

struct Spec {
    panel: Panel,
    label: &'static str,
    way: &'static str,
    home: Zone,
    window: (u32, u32),
}

const PANELS: &[Spec] = &[
    Spec { panel: Panel::Media, label: "Media", way: "var(--way-browser)", home: Zone::Left, window: (420, 640) },
    Spec { panel: Panel::Effects, label: "Effects", way: "var(--way-browser)", home: Zone::Left, window: (420, 640) },
    Spec { panel: Panel::Create, label: "Create", way: "var(--way-browser)", home: Zone::Left, window: (420, 640) },
    Spec { panel: Panel::Colors, label: "Colors", way: "var(--way-browser)", home: Zone::Left, window: (420, 640) },
    Spec { panel: Panel::Stage, label: "Stage", way: "var(--way-stage)", home: Zone::Center, window: (960, 620) },
    Spec { panel: Panel::Output, label: "Output", way: "var(--way-stage)", home: Zone::Center, window: (960, 620) },
    Spec { panel: Panel::Inspector, label: "Inspector", way: "var(--way-inspector)", home: Zone::Right, window: (340, 700) },
    Spec { panel: Panel::Utility, label: "Utility", way: "var(--way-inspector)", home: Zone::Right, window: (340, 700) },
    Spec { panel: Panel::Settings, label: "Settings", way: "var(--way-inspector)", home: Zone::Right, window: (340, 700) },
    Spec { panel: Panel::Timeline, label: "Timeline", way: "var(--way-timeline)", home: Zone::Bottom, window: (1100, 420) },
    Spec { panel: Panel::Ease, label: "Ease", way: "var(--way-timeline)", home: Zone::Right, window: (420, 520) },
];

impl Panel {
    fn spec(self) -> &'static Spec {
        PANELS.iter().find(|spec| spec.panel == self).expect("panel spec")
    }

    pub(super) fn all() -> impl Iterator<Item = Panel> {
        PANELS.iter().map(|spec| spec.panel)
    }

    pub(super) fn way(self) -> &'static str {
        self.spec().way
    }

    pub(super) fn window_size(self) -> (u32, u32) {
        self.spec().window
    }

    pub(super) fn label(self) -> &'static str {
        self.spec().label
    }

    fn workbench_id(self) -> PanelId {
        PanelId::new(self.label())
    }

    fn from_workbench(id: &PanelId) -> Option<Self> {
        PANELS
            .iter()
            .find(|spec| spec.label == id.as_str())
            .map(|spec| spec.panel)
    }

    fn home(self) -> TileId {
        TileId::new(match self.spec().home {
            Zone::Left => "left",
            Zone::Center => "center",
            Zone::Right => "right",
            Zone::Bottom => "bottom",
        })
    }
}

impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Side {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl Side {
    fn workbench(self) -> DockZone {
        match self {
            Self::Left => DockZone::Left,
            Self::Right => DockZone::Right,
            Self::Top => DockZone::Top,
            Self::Bottom => DockZone::Bottom,
            Self::Center => DockZone::Center,
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct Dock {
    layout: PanelLayout,
    hidden: BTreeSet<Panel>,
    detached: BTreeSet<Panel>,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            layout: canonical_layout(),
            hidden: BTreeSet::new(),
            detached: BTreeSet::new(),
        }
    }
}

impl Dock {
    pub(super) fn root(&self) -> LayoutNode {
        self.projected_layout().root
    }

    pub(super) fn panels(&self, tile: &dioxus_workbench::Tile) -> Vec<Panel> {
        tile.panels.iter().filter_map(Panel::from_workbench).collect()
    }

    pub(super) fn active(&self, tile: &dioxus_workbench::Tile) -> Option<Panel> {
        tile.active.as_ref().and_then(Panel::from_workbench)
    }

    pub(super) fn is_visible(&self, panel: Panel) -> bool {
        !self.hidden.contains(&panel)
            && !self.detached.contains(&panel)
            && self.layout.tile_for_panel(&panel.workbench_id()).is_some()
    }

    pub(super) fn is_detached(&self, panel: Panel) -> bool {
        self.detached.contains(&panel)
    }

    pub(super) fn is_active(&self, panel: Panel) -> bool {
        if !self.is_visible(panel) {
            return false;
        }
        let projected = self.projected_layout();
        let Some(tile) = projected.tile_for_panel(&panel.workbench_id()) else {
            return false;
        };
        projected
            .tile(&tile)
            .and_then(|tile| tile.active.as_ref())
            .is_some_and(|active| active == &panel.workbench_id())
    }

    pub(super) fn set_active(&mut self, panel: Panel) {
        if self.is_visible(panel) {
            self.layout.activate(&panel.workbench_id());
        }
    }

    pub(super) fn drop_onto(&mut self, panel: Panel, target: &TileId, side: Side) {
        if self.detached.contains(&panel) || self.hidden.contains(&panel) {
            return;
        }
        self.layout
            .dock_panel(&panel.workbench_id(), target, side.workbench());
    }

    pub(super) fn set_split_ratio(&mut self, split: &SplitId, ratio: f64) {
        self.layout.set_split_ratio(split, ratio);
    }

    pub(super) fn detach(&mut self, panel: Panel) {
        if !self.is_visible(panel) {
            return;
        }
        self.hidden.remove(&panel);
        self.detached.insert(panel);
    }

    pub(super) fn reattach(&mut self, panel: Panel) {
        if !self.detached.remove(&panel) {
            return;
        }
        self.hidden.remove(&panel);
        self.set_active(panel);
    }

    pub(super) fn hide(&mut self, panel: Panel) {
        if self.detached.contains(&panel) {
            return;
        }
        self.hidden.insert(panel);
    }

    pub(super) fn toggle(&mut self, panel: Panel) {
        if self.detached.contains(&panel) {
            return;
        }
        if self.is_visible(panel) {
            self.hide(panel);
        } else {
            self.show(panel);
        }
    }

    pub(super) fn reset_layout(&mut self) {
        self.hidden.clear();
        self.layout = canonical_layout();
    }

    fn show(&mut self, panel: Panel) {
        self.hidden.remove(&panel);
        self.set_active(panel);
    }

    fn projected_layout(&self) -> PanelLayout {
        let placements = Panel::all()
            .filter(|panel| !self.hidden.contains(panel) && !self.detached.contains(panel))
            .map(|panel| PanelPlacement::new(panel.workbench_id(), panel.home()))
            .collect::<Vec<_>>();
        self.layout.reconciled(&placements)
    }

    #[cfg(test)]
    fn valid(&self) -> bool {
        self.layout.valid()
            && self.projected_layout().valid()
            && !Panel::all().any(|panel| self.is_visible(panel) && self.is_detached(panel))
    }
}

fn canonical_layout() -> PanelLayout {
    PanelLayout::new(LayoutNode::split(
        "root",
        SplitAxis::Vertical,
        0.68,
        LayoutNode::split(
            "top-left-rest",
            SplitAxis::Horizontal,
            0.22,
            LayoutNode::tile("left", ["Media", "Effects", "Create", "Colors"]),
            LayoutNode::split(
                "top-center-right",
                SplitAxis::Horizontal,
                0.56 / 0.78,
                LayoutNode::tile("center", ["Stage", "Output"]),
                LayoutNode::tile("right", ["Inspector", "Utility", "Settings", "Ease"]),
            ),
        ),
        LayoutNode::tile("bottom", ["Timeline"]),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_count(node: &LayoutNode, panel: Panel) -> usize {
        match node {
            LayoutNode::Tile(tile) => tile
                .panels
                .iter()
                .filter(|candidate| **candidate == panel.workbench_id())
                .count(),
            LayoutNode::Split { first, second, .. } => {
                panel_count(first, panel) + panel_count(second, panel)
            }
        }
    }

    #[test]
    fn any_pane_can_be_split_again() {
        let mut dock = Dock::default();
        let target = dock.layout.tile_for_panel(&Panel::Media.workbench_id()).unwrap();
        dock.drop_onto(Panel::Ease, &target, Side::Bottom);
        assert!(dock.valid());
        assert!(dock.is_visible(Panel::Ease));
    }

    #[test]
    fn dropping_in_the_middle_stacks_instead_of_splitting() {
        let mut dock = Dock::default();
        let target = dock.layout.tile_for_panel(&Panel::Stage.workbench_id()).unwrap();
        dock.drop_onto(Panel::Ease, &target, Side::Center);
        assert!(dock.is_active(Panel::Ease));
        assert!(dock.valid());
    }

    #[test]
    fn hiding_then_showing_brings_it_back() {
        let mut dock = Dock::default();
        for panel in Panel::all() {
            dock.toggle(panel);
            assert!(!dock.is_visible(panel));
            dock.toggle(panel);
            assert!(dock.is_visible(panel));
            assert!(dock.valid());
        }
    }

    #[test]
    fn every_operation_keeps_one_reachable_dock_tree() {
        let mut dock = Dock::default();
        dock.hide(Panel::Media);
        dock.show(Panel::Media);
        let timeline = dock.layout.tile_for_panel(&Panel::Timeline.workbench_id()).unwrap();
        dock.drop_onto(Panel::Media, &timeline, Side::Top);
        dock.hide(Panel::Effects);
        dock.hide(Panel::Effects);
        dock.detach(Panel::Timeline);
        assert!(dock.valid());
    }

    #[test]
    fn reset_layout_keeps_real_detached_windows_detached() {
        let mut dock = Dock::default();
        dock.detach(Panel::Inspector);
        dock.reset_layout();
        assert!(dock.is_detached(Panel::Inspector));
        assert!(!dock.is_visible(Panel::Inspector));
        assert!(dock.is_active(Panel::Media));
    }

    #[test]
    fn a_detached_panel_cannot_be_docked_while_its_window_is_still_open() {
        let mut dock = Dock::default();
        dock.detach(Panel::Inspector);
        dock.toggle(Panel::Inspector);
        assert!(dock.is_detached(Panel::Inspector));
        assert!(!dock.is_visible(Panel::Inspector));
    }

    #[test]
    fn repeated_edge_drops_do_not_shrink_a_zone_below_the_splitter_floor() {
        let mut dock = Dock::default();
        for index in 0..8 {
            let target = dock.layout.tile_for_panel(&Panel::Stage.workbench_id()).unwrap();
            dock.drop_onto(
                Panel::Media,
                &target,
                if index % 2 == 0 { Side::Left } else { Side::Right },
            );
            assert!(dock.valid());
        }
        assert!(dock
            .layout
            .split_ids()
            .iter()
            .all(|id| dock.layout.split_ratio(id).is_some_and(|ratio| (0.1..=0.9).contains(&ratio))));
    }

    #[test]
    fn moving_an_inactive_tab_does_not_change_the_visible_panel() {
        let mut dock = Dock::default();
        dock.set_active(Panel::Create);
        dock.hide(Panel::Media);
        assert!(dock.is_active(Panel::Create));
    }

    #[test]
    fn recreating_the_bottom_after_timeline_closes_keeps_every_other_panel_once() {
        for moving in Panel::all().filter(|panel| *panel != Panel::Timeline) {
            let mut dock = Dock::default();
            dock.hide(Panel::Timeline);
            let source = dock
                .layout
                .tile_for_panel(&moving.workbench_id())
                .expect("moving panel remains reachable after Timeline closes");

            dock.drop_onto(moving, &source, Side::Bottom);

            assert!(dock.valid(), "{moving} made the recreated bottom invalid");
            let projected = dock.projected_layout();
            assert_eq!(
                projected.tile_count(),
                4,
                "{moving} did not recreate one bottom branch"
            );
            for panel in Panel::all() {
                assert_eq!(
                    panel_count(&projected.root, panel),
                    usize::from(panel != Panel::Timeline),
                    "moving {moving} changed {panel} reachability"
                );
                assert_eq!(
                    panel_count(&dock.layout.root, panel),
                    1,
                    "moving {moving} corrupted stored placement for {panel}"
                );
            }
        }
    }

    #[test]
    fn every_hidden_panel_returns_to_its_previous_tile() {
        for panel in Panel::all() {
            let mut dock = Dock::default();
            let home = dock
                .layout
                .tile_for_panel(&panel.workbench_id())
                .expect("panel home");

            dock.hide(panel);
            assert_eq!(panel_count(&dock.projected_layout().root, panel), 0);
            dock.show(panel);

            assert_eq!(
                dock.layout.tile_for_panel(&panel.workbench_id()),
                Some(home),
                "{panel} forgot its placement while hidden"
            );
            assert_eq!(panel_count(&dock.projected_layout().root, panel), 1);
        }
    }
}
