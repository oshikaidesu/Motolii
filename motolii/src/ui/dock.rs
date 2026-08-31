use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Panel {
    Media,
    Effects,
    Create,
    Colors,
    Stage,
    Output,
    Inspector,
    Utility,
    Timeline,
    Ease,
}

/// パネル1枚の素性。**足す時はここに1行足すだけ**。名前・色・既定の置き場は
/// 全部ここから引く(散らすと足し忘れる)。中身は `app::panel_body`。
struct Spec {
    panel: Panel,
    label: &'static str,
    way: &'static str,
    home: Zone,
}

const PANELS: &[Spec] = &[
    Spec { panel: Panel::Media, label: "Media", way: "var(--way-browser)", home: Zone::Left },
    Spec { panel: Panel::Effects, label: "Effects", way: "var(--way-browser)", home: Zone::Left },
    Spec { panel: Panel::Create, label: "Create", way: "var(--way-browser)", home: Zone::Left },
    Spec { panel: Panel::Colors, label: "Colors", way: "var(--way-browser)", home: Zone::Left },
    Spec { panel: Panel::Stage, label: "Stage", way: "var(--way-stage)", home: Zone::Center },
    Spec { panel: Panel::Output, label: "Output", way: "var(--way-stage)", home: Zone::Center },
    Spec { panel: Panel::Inspector, label: "Inspector", way: "var(--way-inspector)", home: Zone::Right },
    Spec { panel: Panel::Utility, label: "Utility", way: "var(--way-inspector)", home: Zone::Right },
    Spec { panel: Panel::Timeline, label: "Timeline", way: "var(--way-timeline)", home: Zone::Bottom },
    Spec { panel: Panel::Ease, label: "Ease", way: "var(--way-timeline)", home: Zone::Right },
];

impl Panel {
    fn spec(self) -> &'static Spec {
        PANELS
            .iter()
            .find(|s| s.panel == self)
            .expect("PANELS に載っていないパネル")
    }

    pub(super) fn all() -> impl Iterator<Item = Panel> {
        PANELS.iter().map(|s| s.panel)
    }

    /// 部屋ごとの色。帯を畳んだので、これはタブが引き継ぐ。
    pub(super) fn way(self) -> &'static str {
        self.spec().way
    }

    pub(super) fn label(self) -> &'static str {
        self.spec().label
    }
}

impl fmt::Display for Panel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Zone {
    Left,
    Center,
    Right,
    Bottom,
}

impl Zone {
    pub(super) const ALL: [Zone; 4] = [Zone::Left, Zone::Center, Zone::Right, Zone::Bottom];

    fn ix(self) -> usize {
        match self {
            Zone::Left => 0,
            Zone::Center => 1,
            Zone::Right => 2,
            Zone::Bottom => 3,
        }
    }
}

/// どのパネルがどの置き場に居るか。**どこにも居ない = 隠れている**。
/// 矩形は CSS(Taffy)が出すので、ここは並びだけを持つ。
#[derive(Clone, PartialEq)]
pub(super) struct Dock {
    zones: [Vec<Panel>; 4],
    active: [usize; 4],
    /// 別窓へ出ている物。置き場にも居ないし、隠れてもいない。
    detached: Vec<Panel>,
}

impl Default for Dock {
    fn default() -> Self {
        let mut zones: [Vec<Panel>; 4] = Default::default();
        for spec in PANELS {
            zones[spec.home.ix()].push(spec.panel);
        }
        Self { zones, active: [0; 4], detached: Vec::new() }
    }
}

impl Dock {
    pub(super) fn panels(&self, zone: Zone) -> &[Panel] {
        &self.zones[zone.ix()]
    }

    pub(super) fn active(&self, zone: Zone) -> Option<Panel> {
        let z = &self.zones[zone.ix()];
        z.get(self.active[zone.ix()]).copied()
    }

    pub(super) fn is_active(&self, zone: Zone, panel: Panel) -> bool {
        self.active(zone) == Some(panel)
    }

    pub(super) fn set_active(&mut self, zone: Zone, panel: Panel) {
        if let Some(i) = self.zones[zone.ix()].iter().position(|p| *p == panel) {
            self.active[zone.ix()] = i;
        }
    }

    pub(super) fn is_detached(&self, panel: Panel) -> bool {
        self.detached.contains(&panel)
    }

    /// 別窓へ出す。置き場からは抜ける。
    pub(super) fn detach(&mut self, panel: Panel) {
        self.remove(panel);
        self.detached.push(panel);
    }

    /// 別窓を閉じた。元の置き場へ戻す。
    pub(super) fn reattach(&mut self, panel: Panel) {
        self.detached.retain(|p| *p != panel);
        self.place(panel, default_zone(panel));
    }

    pub(super) fn zone_of(&self, panel: Panel) -> Option<Zone> {
        Zone::ALL
            .into_iter()
            .find(|z| self.zones[z.ix()].contains(&panel))
    }

    pub(super) fn is_visible(&self, panel: Panel) -> bool {
        self.zone_of(panel).is_some()
    }

    /// 置き場へ移す。元の場所からは抜ける(同じパネルが2箇所に居ることはない)。
    /// 移った先では選択中になる。既に居る置き場へ落としても並びは動かない。
    pub(super) fn place(&mut self, panel: Panel, zone: Zone) {
        if self.zone_of(panel) == Some(zone) {
            self.set_active(zone, panel);
            return;
        }
        self.remove(panel);
        let z = zone.ix();
        self.zones[z].push(panel);
        self.active[z] = self.zones[z].len() - 1;
    }

    pub(super) fn hide(&mut self, panel: Panel) {
        self.remove(panel);
    }

    /// 隠れていれば元/既定の置き場へ出し、出ていれば隠す。
    pub(super) fn toggle(&mut self, panel: Panel) {
        match self.zone_of(panel) {
            Some(_) => self.hide(panel),
            None => self.place(panel, default_zone(panel)),
        }
    }

    fn remove(&mut self, panel: Panel) {
        self.detached.retain(|p| *p != panel);
        for z in 0..4 {
            if let Some(i) = self.zones[z].iter().position(|p| *p == panel) {
                self.zones[z].remove(i);
                self.active[z] = self.active[z].min(self.zones[z].len().saturating_sub(1));
            }
        }
    }
}

fn default_zone(panel: Panel) -> Zone {
    panel.spec().home
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_placed(dock: &Dock) -> Vec<Panel> {
        Zone::ALL
            .into_iter()
            .flat_map(|z| dock.panels(z).to_vec())
            .collect()
    }

    #[test]
    fn a_panel_never_lives_in_two_places() {
        let mut dock = Dock::default();
        dock.place(Panel::Inspector, Zone::Left);
        dock.place(Panel::Inspector, Zone::Bottom);

        let placed = all_placed(&dock);
        assert_eq!(
            placed.iter().filter(|p| **p == Panel::Inspector).count(),
            1,
            "同じパネルが2箇所に居る"
        );
        assert_eq!(dock.zone_of(Panel::Inspector), Some(Zone::Bottom));
    }

    #[test]
    fn moving_the_selected_panel_out_leaves_a_selection_behind() {
        let mut dock = Dock::default();
        dock.set_active(Zone::Right, Panel::Utility);
        dock.place(Panel::Utility, Zone::Left);

        let left_behind = dock.active(Zone::Right).expect("右の置き場に何も選ばれていない");
        assert_ne!(left_behind, Panel::Utility, "抜けたはずのパネルが選ばれている");
        assert_eq!(dock.active(Zone::Left), Some(Panel::Utility));
    }

    #[test]
    fn dropping_a_tab_back_where_it_came_from_does_not_reorder() {
        let mut dock = Dock::default();
        let before = dock.panels(Zone::Right).to_vec();
        dock.place(Panel::Inspector, Zone::Right);
        assert_eq!(dock.panels(Zone::Right), before.as_slice());
        assert_eq!(dock.active(Zone::Right), Some(Panel::Inspector));
    }

    #[test]
    fn a_detached_panel_is_in_no_zone_and_comes_back_when_its_window_closes() {
        let mut dock = Dock::default();
        dock.detach(Panel::Stage);
        assert!(dock.is_detached(Panel::Stage));
        assert_eq!(dock.zone_of(Panel::Stage), None, "別窓のパネルが置き場にも居る");
        assert!(!dock.is_visible(Panel::Stage));

        dock.reattach(Panel::Stage);
        assert!(!dock.is_detached(Panel::Stage));
        assert_eq!(dock.zone_of(Panel::Stage), Some(Zone::Center));
    }

    #[test]
    fn putting_a_detached_panel_back_in_a_zone_closes_the_detachment() {
        let mut dock = Dock::default();
        dock.detach(Panel::Colors);
        dock.place(Panel::Colors, Zone::Bottom);
        assert!(!dock.is_detached(Panel::Colors), "置き場に戻したのに別窓のままになっている");
    }

    #[test]
    fn an_empty_zone_has_nothing_selected() {
        let mut dock = Dock::default();
        for panel in dock.panels(Zone::Center).into_iter().copied().collect::<Vec<_>>() {
            dock.hide(panel);
        }
        assert!(dock.panels(Zone::Center).is_empty());
        assert_eq!(dock.active(Zone::Center), None);
    }

    #[test]
    fn hiding_then_showing_puts_it_back_somewhere_reachable() {
        let mut dock = Dock::default();
        for panel in Panel::all() {
            dock.toggle(panel);
            assert!(!dock.is_visible(panel));
            dock.toggle(panel);
            assert!(dock.is_visible(panel), "{panel} が出せなくなった");
        }
    }
}
