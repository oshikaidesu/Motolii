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
    Settings,
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
    Spec { panel: Panel::Settings, label: "Settings", way: "var(--way-inspector)", home: Zone::Right },
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
