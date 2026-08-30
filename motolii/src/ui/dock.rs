use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Panel {
    Media,
    Effects,
    Create,
    Colors,
    Stage,
    Inspector,
    Utility,
    Timeline,
}

impl Panel {
    pub(super) const ALL: [Panel; 8] = [
        Panel::Media,
        Panel::Effects,
        Panel::Create,
        Panel::Colors,
        Panel::Stage,
        Panel::Inspector,
        Panel::Utility,
        Panel::Timeline,
    ];

    /// 部屋ごとの色。帯を畳んだので、これはタブが引き継ぐ。
    pub(super) fn way(self) -> &'static str {
        match self {
            Panel::Media | Panel::Effects | Panel::Create | Panel::Colors => "var(--way-browser)",
            Panel::Stage => "var(--way-stage)",
            Panel::Inspector | Panel::Utility => "var(--way-inspector)",
            Panel::Timeline => "var(--way-timeline)",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Panel::Media => "Media",
            Panel::Effects => "Effects",
            Panel::Create => "Create",
            Panel::Colors => "Colors",
            Panel::Stage => "Stage",
            Panel::Inspector => "Inspector",
            Panel::Utility => "Utility",
            Panel::Timeline => "Timeline",
        }
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
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            zones: [
                vec![Panel::Media, Panel::Effects, Panel::Create, Panel::Colors],
                vec![Panel::Stage],
                vec![Panel::Inspector, Panel::Utility],
                vec![Panel::Timeline],
            ],
            active: [0; 4],
        }
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
        self.detach(panel);
        let z = zone.ix();
        self.zones[z].push(panel);
        self.active[z] = self.zones[z].len() - 1;
    }

    pub(super) fn hide(&mut self, panel: Panel) {
        self.detach(panel);
    }

    /// 隠れていれば元/既定の置き場へ出し、出ていれば隠す。
    pub(super) fn toggle(&mut self, panel: Panel) {
        match self.zone_of(panel) {
            Some(_) => self.hide(panel),
            None => self.place(panel, default_zone(panel)),
        }
    }

    fn detach(&mut self, panel: Panel) {
        for z in 0..4 {
            if let Some(i) = self.zones[z].iter().position(|p| *p == panel) {
                self.zones[z].remove(i);
                self.active[z] = self.active[z].min(self.zones[z].len().saturating_sub(1));
            }
        }
    }
}

fn default_zone(panel: Panel) -> Zone {
    match panel {
        Panel::Media | Panel::Effects | Panel::Create | Panel::Colors => Zone::Left,
        Panel::Stage => Zone::Center,
        Panel::Inspector | Panel::Utility => Zone::Right,
        Panel::Timeline => Zone::Bottom,
    }
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

        assert_eq!(dock.active(Zone::Right), Some(Panel::Inspector));
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
    fn an_empty_zone_has_nothing_selected() {
        let mut dock = Dock::default();
        dock.hide(Panel::Stage);
        assert!(dock.panels(Zone::Center).is_empty());
        assert_eq!(dock.active(Zone::Center), None);
    }

    #[test]
    fn hiding_then_showing_puts_it_back_somewhere_reachable() {
        let mut dock = Dock::default();
        for panel in Panel::ALL {
            dock.toggle(panel);
            assert!(!dock.is_visible(panel));
            dock.toggle(panel);
            assert!(dock.is_visible(panel), "{panel} が出せなくなった");
        }
    }
}
