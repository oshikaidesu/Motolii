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

/// パネルの生まれ。**置き場ではない** —— 最初の形と、隠れた物を戻す先を
/// 決めるためだけに使う。置き場は木で、いくつでも割れる。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Zone {
    Left,
    Center,
    Right,
    Bottom,
}

/// パネル1枚の素性。**足す時はここに1行足すだけ**。名前・色・既定の置き場は
/// 全部ここから引く(散らすと足し忘れる)。中身は `app::panel_body`。
struct Spec {
    panel: Panel,
    label: &'static str,
    way: &'static str,
    home: Zone,
    /// 別窓にした時の大きさ。パネルごとに要る形が違う(帯は横長、属性は縦長)。
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

    pub(super) fn window_size(self) -> (u32, u32) {
        self.spec().window
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

/// 置き場は**木**。葉が Pane(パネル1枚)で、節は「並べる」か「重ねる」。
/// 部屋を4つに決め打ちすると、左をさらに上下に割る手が**型として存在しない**。
///
/// 語彙は egui_tiles(rerun が出資)と同じ形にしてある —— Tile = Container か Pane、
/// Container は Linear(並べる)か Tabs(重ねる)。ImGui も VS Code も骨は同じ。
pub(super) type TileId = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Dir {
    /// 横に並べる。
    Row,
    /// 縦に並べる。
    Column,
}

#[derive(Clone, PartialEq, Debug)]
pub(super) enum Tile {
    Pane(Panel),
    /// 並べる。`shares` は子と同じ長さで、合計に対する割合。
    Linear { dir: Dir, children: Vec<TileId>, shares: Vec<f32> },
    /// 重ねる。見えているのは1つ。
    Tabs { children: Vec<TileId>, active: usize },
}

/// どこへ落としたか。中央は重ねる、縁は割る。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Side {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

#[derive(Clone, PartialEq)]
pub(super) struct Dock {
    tiles: std::collections::BTreeMap<TileId, Tile>,
    root: Option<TileId>,
    next: TileId,
    /// 別窓へ出ている物。木にも居ないし、隠れてもいない。
    detached: Vec<Panel>,
}

impl Default for Dock {
    fn default() -> Self {
        let mut d = Self {
            tiles: Default::default(),
            root: None,
            next: 1,
            detached: Vec::new(),
        };
        let group = |d: &mut Self, home: Zone| {
            let panes: Vec<TileId> = PANELS
                .iter()
                .filter(|s| s.home == home)
                .map(|s| d.add(Tile::Pane(s.panel)))
                .collect();
            d.add(Tile::Tabs { children: panes, active: 0 })
        };
        let left = group(&mut d, Zone::Left);
        let center = group(&mut d, Zone::Center);
        let right = group(&mut d, Zone::Right);
        let bottom = group(&mut d, Zone::Bottom);
        let row = d.add(Tile::Linear {
            dir: Dir::Row,
            children: vec![left, center, right],
            shares: vec![0.22, 0.56, 0.22],
        });
        let root = d.add(Tile::Linear {
            dir: Dir::Column,
            children: vec![row, bottom],
            shares: vec![0.68, 0.32],
        });
        d.root = Some(root);
        d
    }
}

impl Dock {
    pub(super) fn root(&self) -> Option<TileId> {
        self.root
    }

    pub(super) fn tile(&self, id: TileId) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    fn add(&mut self, tile: Tile) -> TileId {
        let id = self.next;
        self.next += 1;
        self.tiles.insert(id, tile);
        id
    }

    fn parent_of(&self, id: TileId) -> Option<TileId> {
        self.tiles.iter().find_map(|(pid, t)| match t {
            Tile::Linear { children, .. } | Tile::Tabs { children, .. } => {
                children.contains(&id).then_some(*pid)
            }
            Tile::Pane(_) => None,
        })
    }

    pub(super) fn tile_of(&self, panel: Panel) -> Option<TileId> {
        self.tiles
            .iter()
            .find_map(|(id, t)| matches!(t, Tile::Pane(p) if *p == panel).then_some(*id))
    }

    pub(super) fn is_visible(&self, panel: Panel) -> bool {
        self.tile_of(panel).is_some()
    }

    pub(super) fn is_detached(&self, panel: Panel) -> bool {
        self.detached.contains(&panel)
    }

    /// 重ねている束の中で今見えている物。
    pub(super) fn active_of(&self, tabs: TileId) -> Option<TileId> {
        match self.tiles.get(&tabs)? {
            Tile::Tabs { children, active } => children.get(*active).copied(),
            _ => None,
        }
    }

    pub(super) fn is_active(&self, panel: Panel) -> bool {
        let Some(id) = self.tile_of(panel) else { return false };
        let Some(parent) = self.parent_of(id) else { return true };
        match self.tiles.get(&parent) {
            Some(Tile::Tabs { .. }) => self.active_of(parent) == Some(id),
            _ => true,
        }
    }

    pub(super) fn set_active(&mut self, panel: Panel) {
        let Some(id) = self.tile_of(panel) else { return };
        let Some(parent) = self.parent_of(id) else { return };
        if let Some(Tile::Tabs { children, active }) = self.tiles.get_mut(&parent) {
            if let Some(i) = children.iter().position(|c| *c == id) {
                *active = i;
            }
        }
    }

    /// パネルを別のタイルの隣(または上)へ移す。**中央なら重ね、縁なら割る。**
    /// 同じ所へ落としても並びは動かない。
    pub(super) fn drop_onto(&mut self, panel: Panel, target: TileId, side: Side) {
        if self.tile_of(panel) == Some(target) {
            return;
        }
        self.pull_out(panel);
        let moving = self.add(Tile::Pane(panel));
        self.insert_at(moving, target, side);
        self.set_active(panel);
    }

    fn insert_at(&mut self, moving: TileId, target: TileId, side: Side) {
        let Some(root) = self.root else {
            self.root = Some(moving);
            return;
        };
        // 束の中の1枚へ落としたら、束そのものを相手にする。
        let target = match self.parent_of(target) {
            Some(p) if matches!(self.tiles.get(&p), Some(Tile::Tabs { .. })) => p,
            _ => target,
        };
        if side == Side::Center {
            match self.tiles.get_mut(&target) {
                Some(Tile::Tabs { children, active }) => {
                    children.push(moving);
                    *active = children.len() - 1;
                }
                _ => {
                    let tabs = self.add(Tile::Tabs { children: vec![target, moving], active: 1 });
                    self.replace_child(target, tabs, root);
                }
            }
            return;
        }
        let (dir, before) = match side {
            Side::Left => (Dir::Row, true),
            Side::Right => (Dir::Row, false),
            Side::Top => (Dir::Column, true),
            Side::Bottom => (Dir::Column, false),
            Side::Center => unreachable!("上で返している"),
        };
        // 同じ向きの並びの中なら、割らずに間へ入れる。
        if let Some(parent) = self.parent_of(target) {
            if let Some(Tile::Linear { dir: pdir, children, shares }) = self.tiles.get_mut(&parent) {
                if *pdir == dir {
                    let i = children.iter().position(|c| *c == target).unwrap_or(0);
                    let at = if before { i } else { i + 1 };
                    let share = shares.get(i).copied().unwrap_or(1.0) * 0.5;
                    if let Some(s) = shares.get_mut(i) {
                        *s = share;
                    }
                    children.insert(at, moving);
                    shares.insert(at, share);
                    return;
                }
            }
        }
        let children = if before { vec![moving, target] } else { vec![target, moving] };
        let split = self.add(Tile::Linear { dir, children, shares: vec![0.5, 0.5] });
        self.replace_child(target, split, root);
    }

    /// `old` の居た場所へ `new` を差す。根なら根を差し替える。
    fn replace_child(&mut self, old: TileId, new: TileId, root: TileId) {
        if old == root {
            self.root = Some(new);
            return;
        }
        let Some(parent) = self.parent_of(old) else { return };
        if let Some(Tile::Linear { children, .. } | Tile::Tabs { children, .. }) =
            self.tiles.get_mut(&parent)
        {
            if let Some(slot) = children.iter_mut().find(|c| **c == old) {
                *slot = new;
            }
        }
    }

    /// 木から抜いて、空になった節を畳む。**畳まないと空の箱が残る。**
    fn pull_out(&mut self, panel: Panel) {
        self.detached.retain(|p| *p != panel);
        let Some(id) = self.tile_of(panel) else { return };
        self.tiles.remove(&id);
        if self.root == Some(id) {
            self.root = None;
            return;
        }
        let Some(parent) = self.parent_of(id) else { return };
        let mut drop_parent = false;
        if let Some(tile) = self.tiles.get_mut(&parent) {
            match tile {
                Tile::Linear { children, shares, .. } => {
                    if let Some(i) = children.iter().position(|c| *c == id) {
                        children.remove(i);
                        if i < shares.len() {
                            shares.remove(i);
                        }
                    }
                    drop_parent = children.len() <= 1;
                }
                Tile::Tabs { children, active } => {
                    if let Some(i) = children.iter().position(|c| *c == id) {
                        children.remove(i);
                        *active = (*active).min(children.len().saturating_sub(1));
                    }
                    drop_parent = children.is_empty();
                }
                Tile::Pane(_) => {}
            }
        }
        if drop_parent {
            self.collapse(parent);
        }
    }

    /// 子が1つ以下になった節を、その子で置き換える。
    fn collapse(&mut self, id: TileId) {
        let Some(root) = self.root else { return };
        let only = match self.tiles.get(&id) {
            Some(Tile::Linear { children, .. } | Tile::Tabs { children, .. }) => {
                children.first().copied()
            }
            _ => return,
        };
        match only {
            Some(child) => {
                self.replace_child(id, child, root);
                self.tiles.remove(&id);
            }
            None => {
                let parent = self.parent_of(id);
                self.tiles.remove(&id);
                if self.root == Some(id) {
                    self.root = None;
                } else if let Some(parent) = parent {
                    if let Some(
                        Tile::Linear { children, .. } | Tile::Tabs { children, .. },
                    ) = self.tiles.get_mut(&parent)
                    {
                        if let Some(i) = children.iter().position(|c| *c == id) {
                            children.remove(i);
                        }
                    }
                    self.collapse(parent);
                }
            }
        }
    }

    /// 隣り合う2つの割合を動かす。**合計は変えない** —— 変えると節ごとに
    /// 大きさが漂う。
    pub(super) fn nudge_share(&mut self, parent: TileId, index: usize, delta: f32) {
        let Some(Tile::Linear { shares, .. }) = self.tiles.get_mut(&parent) else { return };
        let (Some(a), Some(b)) = (shares.get(index).copied(), shares.get(index + 1).copied()) else {
            return;
        };
        let total = a + b;
        let next = (a + delta * total).clamp(total * 0.08, total * 0.92);
        shares[index] = next;
        shares[index + 1] = total - next;
    }

    pub(super) fn detach(&mut self, panel: Panel) {
        self.pull_out(panel);
        self.detached.push(panel);
    }

    pub(super) fn reattach(&mut self, panel: Panel) {
        self.detached.retain(|p| *p != panel);
        self.show(panel);
    }

    pub(super) fn hide(&mut self, panel: Panel) {
        self.pull_out(panel);
    }

    pub(super) fn toggle(&mut self, panel: Panel) {
        if self.is_visible(panel) {
            self.hide(panel);
        } else {
            self.show(panel);
        }
    }

    /// 隠れていた物を出す。**同じ家の仲間の隣**へ戻す(既定の置き場の代わり)。
    fn show(&mut self, panel: Panel) {
        if self.is_visible(panel) {
            return;
        }
        let home = panel.spec().home;
        let mate = PANELS
            .iter()
            .filter(|s| s.home == home && s.panel != panel)
            .find_map(|s| self.tile_of(s.panel));
        match mate {
            Some(target) => self.drop_onto(panel, target, Side::Center),
            None => {
                let Some(root) = self.root else {
                    let id = self.add(Tile::Pane(panel));
                    self.root = Some(id);
                    return;
                };
                self.drop_onto(panel, root, Side::Right);
            }
        }
    }
}

#[cfg(test)]
mod tree {
    use super::*;

    fn panels_in(d: &Dock, id: TileId, out: &mut Vec<Panel>) {
        match d.tile(id) {
            Some(Tile::Pane(p)) => out.push(*p),
            Some(Tile::Linear { children, .. } | Tile::Tabs { children, .. }) => {
                for c in children.clone() {
                    panels_in(d, c, out);
                }
            }
            None => {}
        }
    }

    fn all(d: &Dock) -> Vec<Panel> {
        let mut out = Vec::new();
        if let Some(root) = d.root() {
            panels_in(d, root, &mut out);
        }
        out
    }

    /// 木は**どの節でも割れる**。部屋を4つに決め打ちしていた時は、
    /// 左をさらに上下に割る手が型として存在しなかった。
    #[test]
    fn any_pane_can_be_split_again() {
        let mut d = Dock::default();
        let target = d.tile_of(Panel::Media).expect("Media");
        d.hide(Panel::Ease);
        d.drop_onto(Panel::Ease, target, Side::Bottom);

        assert!(d.is_visible(Panel::Ease));
        let parent = d.parent_of(d.tile_of(Panel::Ease).unwrap()).unwrap();
        assert!(
            matches!(d.tile(parent), Some(Tile::Linear { dir: Dir::Column, .. })),
            "縦に割れていない: {:?}",
            d.tile(parent)
        );
    }

    /// 中央へ落としたら重ねる。縁なら割る。
    #[test]
    fn dropping_in_the_middle_stacks_instead_of_splitting() {
        let mut d = Dock::default();
        let target = d.tile_of(Panel::Stage).expect("Stage");
        d.drop_onto(Panel::Ease, target, Side::Center);

        let parent = d.parent_of(d.tile_of(Panel::Ease).unwrap()).unwrap();
        assert!(matches!(d.tile(parent), Some(Tile::Tabs { .. })), "重なっていない");
        assert!(d.is_active(Panel::Ease), "落とした物が見えていない");
    }

    /// 抜いた後に**空の箱を残さない**。残ると触れない隙間になる。
    #[test]
    fn taking_the_last_one_out_folds_the_box_away() {
        let mut d = Dock::default();
        let before = d.tiles.len();
        d.hide(Panel::Timeline);

        assert!(!d.is_visible(Panel::Timeline));
        assert!(
            d.tiles.len() < before,
            "空の箱が残っている: {} → {}",
            before,
            d.tiles.len()
        );
        for (id, t) in &d.tiles {
            if let Tile::Linear { children, .. } | Tile::Tabs { children, .. } = t {
                assert!(!children.is_empty(), "空の節 {id} が残った");
            }
        }
    }

    /// 同じパネルが2箇所に居ることはない。
    #[test]
    fn a_panel_lives_in_exactly_one_place() {
        let mut d = Dock::default();
        let target = d.tile_of(Panel::Inspector).unwrap();
        d.drop_onto(Panel::Media, target, Side::Center);

        let seen = all(&d);
        let mut sorted = seen.clone();
        sorted.sort_by_key(|p| p.label());
        sorted.dedup();
        assert_eq!(seen.len(), sorted.len(), "同じ物が2箇所に居る: {seen:?}");
    }

    /// 隠して出すと戻ってくる。
    #[test]
    fn hiding_then_showing_brings_it_back() {
        let mut d = Dock::default();
        for panel in Panel::all() {
            d.toggle(panel);
            assert!(!d.is_visible(panel), "{panel} が隠れていない");
            d.toggle(panel);
            assert!(d.is_visible(panel), "{panel} が戻ってこない");
        }
        assert_eq!(all(&d).len(), Panel::all().count(), "数が合わない");
    }
}
