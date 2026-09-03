use std::collections::BTreeSet;
use std::fmt;

use dioxus_workbench::{
    DockZone, LayoutNode, PanelId, PanelLayout, PanelPlacement, SplitAxis, SplitId, TileId,
};
use dioxus_dnd::prelude::{transition, GestureEffect, GestureEvent, GesturePhase, Point};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub(crate) enum Panel {
    Media,
    Effects,
    Create,
    Colors,
    Stage,
    Inspector,
    Timeline,
    Desk,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone {
    Left,
    Center,
    Right,
    Desk,
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
    Spec { panel: Panel::Inspector, label: "Inspector", way: "var(--way-inspector)", home: Zone::Right, window: (340, 700) },
    Spec { panel: Panel::Timeline, label: "Timeline", way: "var(--way-timeline)", home: Zone::Bottom, window: (1100, 420) },
    Spec { panel: Panel::Desk, label: "Desk", way: "var(--way-timeline)", home: Zone::Desk, window: (420, 520) },
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
            Zone::Desk => "desk",
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

/// 配置は人の物。作品ではなく利用者の設定へ仕舞う(別窓は窓が閉じれば戻るので仕舞わない)。
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct Dock {
    layout: PanelLayout,
    hidden: BTreeSet<Panel>,
    #[serde(skip)]
    detached: BTreeSet<Panel>,
    /// 引き出しを開けて机を広げている間、畳んでいた時の割合を覚える。
    desk_folded: Option<f64>,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            layout: canonical_layout(),
            hidden: BTreeSet::new(),
            detached: BTreeSet::new(),
            desk_folded: None,
        }
    }
}

impl Dock {
    /// 仕舞った配置。壊れていれば無かった事にして既定へ戻る。
    pub(super) fn load(path: &std::path::Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        let dock: Self = serde_json::from_str(&text).ok()?;
        // 知らない panel 名(11 面時代の Ease / Output 等)が居る配置は捨てて既定へ。空の tile が残る。
        fn all_known(node: &LayoutNode) -> bool {
            match node {
                LayoutNode::Tile(tile) => tile.panels.iter().all(|p| Panel::from_workbench(p).is_some()),
                LayoutNode::Split { first, second, .. } => all_known(first) && all_known(second),
            }
        }
        (dock.layout.valid() && all_known(&dock.layout.root)).then_some(dock)
    }

    pub(super) fn save(&self, path: &std::path::Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string(self) {
            let _ = std::fs::write(path, text);
        }
    }

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

    /// 別窓へ出す。隠れている panel も出せる(隠れは解ける)。既に窓なら出さない。
    pub(super) fn detach(&mut self, panel: Panel) -> bool {
        if self.detached.contains(&panel)
            || self.layout.tile_for_panel(&panel.workbench_id()).is_none()
        {
            return false;
        }
        self.hidden.remove(&panel);
        self.detached.insert(panel);
        true
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
        self.desk_folded = None;
        self.layout = canonical_layout();
    }

    pub(super) fn desk_open(&self) -> bool {
        self.desk_folded.is_some()
    }

    /// 引き出しは机の中に出る。開けたら机の tile が上へ広がり、閉じたら畳んでいた割合へ戻る。
    pub(super) fn open_desk(&mut self, open: bool) {
        let id = SplitId::new("right-col");
        match (open, self.desk_folded) {
            (true, None) => {
                if let Some(folded) = self.layout.split_ratio(&id) {
                    self.desk_folded = Some(folded);
                    self.layout.set_split_ratio(&id, DESK_OPEN_RATIO);
                }
            }
            (false, Some(folded)) => {
                self.desk_folded = None;
                self.layout.set_split_ratio(&id, folded);
            }
            _ => {}
        }
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

/// tab を掴んでから放すまで。相(押した・引いている)と効果(tap・drop・cancel)は dioxus-dnd の物。
pub(super) const TAB_DRAG_THRESHOLD: f64 = 6.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct TabDrag {
    pub(super) panel: Panel,
    pub(super) phase: GesturePhase,
    pub(super) cursor: Point,
}

impl TabDrag {
    pub(super) fn pressed(panel: Panel, x: f64, y: f64, pointer_id: i32) -> Self {
        let at = Point::new(x, y);
        let (phase, _) = transition(
            GesturePhase::Idle,
            GestureEvent::Down { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        Self { panel, phase, cursor: at }
    }

    pub(super) fn move_to(mut self, x: f64, y: f64, pointer_id: i32) -> (Self, GestureEffect) {
        let at = Point::new(x, y);
        let (phase, effect) = transition(
            self.phase,
            GestureEvent::Move { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        self.cursor = at;
        (self, effect)
    }

    pub(super) fn release(mut self, x: f64, y: f64, pointer_id: i32) -> (Self, GestureEffect) {
        let at = Point::new(x, y);
        let (phase, _) = transition(
            self.phase,
            GestureEvent::Move { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        let (phase, effect) = transition(
            phase,
            GestureEvent::Up { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        self.cursor = at;
        (self, effect)
    }

    pub(super) fn cancel(mut self) -> (Self, GestureEffect) {
        let (phase, effect) = transition(
            self.phase,
            GestureEvent::Cancel,
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        (self, effect)
    }

    pub(super) fn dragging(self) -> bool {
        matches!(self.phase, GesturePhase::Dragging { .. })
    }

    pub(super) fn pointer_id(self) -> i32 {
        match self.phase {
            GesturePhase::Pressed { pointer_id, .. }
            | GesturePhase::Dragging { pointer_id, .. } => pointer_id,
            GesturePhase::Idle => 0,
        }
    }
}

/// 仕切りの移動量を、測った箱の長さで割合にする。長さが無ければ動かない。
pub(super) fn splitter_delta(pointer_delta: f64, extent: f64) -> f32 {
    if extent.is_finite() && extent > 0.0 {
        (pointer_delta / extent) as f32
    } else {
        0.0
    }
}

#[cfg(test)]
#[test]
fn splitter_delta_uses_the_measured_container_extent() {
    assert!((splitter_delta(120.0, 600.0) - 0.2).abs() < f32::EPSILON);
    assert!((splitter_delta(120.0, 1_200.0) - 0.1).abs() < f32::EPSILON);
    assert_eq!(splitter_delta(120.0, 0.0), 0.0);
}

/// 引き出しが開いている間の Inspector の取り分。残りが机。
const DESK_OPEN_RATIO: f64 = 0.42;

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
                LayoutNode::tile("center", ["Stage"]),
                LayoutNode::split(
                    "right-col",
                    SplitAxis::Vertical,
                    0.86,
                    LayoutNode::tile("right", ["Inspector"]),
                    LayoutNode::tile("desk", ["Desk"]),
                ),
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
    fn a_saved_layout_comes_back_without_its_detached_windows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("layout.json");
        let mut dock = Dock::default();
        dock.hide(Panel::Colors);
        dock.set_split_ratio(&SplitId::new("root"), 0.5);
        assert!(dock.detach(Panel::Desk));
        dock.save(&path);

        let back = Dock::load(&path).unwrap();
        assert!(!back.is_visible(Panel::Colors));
        assert_eq!(back.layout.split_ratio(&SplitId::new("root")), Some(0.5));
        assert!(!back.is_detached(Panel::Desk), "a closed window came back detached");
        assert!(back.is_visible(Panel::Desk));

        std::fs::write(&path, "{ not layout").unwrap();
        assert!(Dock::load(&path).is_none());

        // 昔の panel 名を含む配置は既定へ戻る(空の tile を居座らせない)。
        let stale = serde_json::to_string(&Dock::default()).unwrap().replace("\"Media\"", "\"Ease\"");
        std::fs::write(&path, stale).unwrap();
        assert!(Dock::load(&path).is_none(), "a stale panel name survived into the layout");
    }

    #[test]
    fn any_pane_can_be_split_again() {
        let mut dock = Dock::default();
        let target = dock.layout.tile_for_panel(&Panel::Media.workbench_id()).unwrap();
        dock.drop_onto(Panel::Desk, &target, Side::Bottom);
        assert!(dock.valid());
        assert!(dock.is_visible(Panel::Desk));
    }

    #[test]
    fn dropping_in_the_middle_stacks_instead_of_splitting() {
        let mut dock = Dock::default();
        let target = dock.layout.tile_for_panel(&Panel::Stage.workbench_id()).unwrap();
        dock.drop_onto(Panel::Desk, &target, Side::Center);
        assert!(dock.is_active(Panel::Desk));
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
        let tiles = Dock::default().projected_layout().tile_count();
        for moving in Panel::all().filter(|panel| *panel != Panel::Timeline) {
            let mut dock = Dock::default();
            dock.hide(Panel::Timeline);
            let source = dock
                .layout
                .tile_for_panel(&moving.workbench_id())
                .expect("moving panel remains reachable after Timeline closes");

            let alone = dock
                .projected_layout()
                .tile(&source)
                .is_some_and(|tile| tile.panels.len() == 1);
            dock.drop_onto(moving, &source, Side::Bottom);

            assert!(dock.valid(), "{moving} made the recreated bottom invalid");
            let projected = dock.projected_layout();
            assert_eq!(
                projected.tile_count(),
                if alone { tiles - 1 } else { tiles },
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
