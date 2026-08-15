// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release),
// the `mod tests` blocks of `src/tiles.rs`, `src/tree.rs` and `src/container/tabs.rs`.
//
// 加えて C4 capsule の POSITIVE ORACLE(`docs/ui-interaction-language.md:75`)
// 「分割・tab化・resize・表示/非表示・既定presetへのreset」を状態機械の上で確かめる。

use super::geom::{pos2, Pos2, Rect};
use super::{
    Behavior, Container, ContainerInsertion, ContainerKind, DockInput, Grid, GridLayout,
    InsertionPoint, ResizeState, Shares, SimplificationOptions, Tile, TileId, Tiles, Tree,
};

const AREA: Rect = Rect {
    min: Pos2 { x: 0.0, y: 0.0 },
    max: Pos2 { x: 800.0, y: 600.0 },
};

/// Keeps every pane and answers with the upstream defaults.
struct TestBehavior;

impl Behavior<&'static str> for TestBehavior {
    fn tab_title_for_pane(&mut self, pane: &&'static str) -> String {
        (*pane).to_owned()
    }
}

/// Drops one specific pane, the way an application closes a document.
struct DropPane(&'static str);

impl Behavior<&'static str> for DropPane {
    fn tab_title_for_pane(&mut self, pane: &&'static str) -> String {
        (*pane).to_owned()
    }

    fn retain_pane(&mut self, pane: &&'static str) -> bool {
        *pane != self.0
    }
}

fn insertion(parent_id: TileId, insertion: ContainerInsertion) -> InsertionPoint {
    InsertionPoint {
        parent_id,
        insertion,
    }
}

fn hover(x: f32, y: f32) -> DockInput {
    DockInput {
        pointer_pos: Some(pos2(x, y)),
        ..Default::default()
    }
}

fn width_of(tree: &Tree<&'static str>, tile: TileId) -> f32 {
    tree.tiles.rect(tile).expect("laid out").width()
}

// ----------------------------------------------------------------------------
// POSITIVE ORACLE

/// 分割: dragging a pane onto the lower half of another splits that pane vertically.
#[test]
fn oracle_split() {
    let mut behavior = TestBehavior;
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let mut tree = Tree::new("split", root, tiles);

    tree.update(&mut behavior, &DockInput::default(), AREA);

    tree.begin_drag(&behavior, b);
    // Over the lower half of `a`, which is the `Vertical(MAX)` drop zone of that pane.
    tree.update(&mut behavior, &hover(200.0, 450.0), AREA);

    let drop = DockInput {
        pointer_released: true,
        ..hover(200.0, 450.0)
    };
    tree.update(&mut behavior, &drop, AREA);
    tree.update(&mut behavior, &DockInput::default(), AREA);

    let new_root = tree.root.expect("the tree still has a root");
    let container = tree
        .tiles
        .get_container(new_root)
        .expect("the root is a container");
    assert_eq!(
        container.kind(),
        ContainerKind::Vertical,
        "dropping on the lower half should have split vertically, got {:?}",
        container.kind()
    );
    assert_eq!(container.children_vec(), vec![a, b]);
}

/// tab化: dragging a pane onto another pane's tab strip puts them in one tab container.
#[test]
fn oracle_make_tabs() {
    let mut behavior = TestBehavior;
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let mut tree = Tree::new("tabs", root, tiles);

    tree.update(&mut behavior, &DockInput::default(), AREA);

    tree.begin_drag(&behavior, b);
    // The centre of `a`'s `Tabs(MAX)` drop zone: everything below its tab bar.
    let over_tab_zone = hover(200.0, 312.0);
    tree.update(&mut behavior, &over_tab_zone, AREA);

    let drop = DockInput {
        pointer_released: true,
        ..over_tab_zone
    };
    tree.update(&mut behavior, &drop, AREA);
    tree.update(&mut behavior, &DockInput::default(), AREA);

    let new_root = tree.root.expect("the tree still has a root");
    match tree.tiles.get_container(new_root) {
        Some(Container::Tabs(tabs)) => {
            assert_eq!(tabs.children, vec![a, b]);
            assert_eq!(tabs.active, Some(b), "the dropped tab becomes the open one");
        }
        other => panic!("expected a tab container, got {other:?}"),
    }
}

/// resize: dragging the splitter between two panes moves space from one to the other.
#[test]
fn oracle_resize() {
    let mut behavior = TestBehavior;
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let mut tree = Tree::new("resize", root, tiles);

    let frame = tree.update(&mut behavior, &DockInput::default(), AREA);
    assert_eq!(frame.handles.len(), 1, "one splitter between two panes");
    let before = width_of(&tree, a);
    assert!(
        (before - width_of(&tree, b)).abs() < 1.0,
        "the two panes start out even"
    );

    let splitter_x = frame.handles[0].line;

    // Press on the splitter...
    let press = DockInput {
        pointer_down: true,
        pointer_pressed: true,
        ..hover(splitter_x, 300.0)
    };
    let frame = tree.update(&mut behavior, &press, AREA);
    assert_eq!(frame.handles[0].state, ResizeState::Dragging);

    // ...and drag it to the right.
    let drag = DockInput {
        pointer_down: true,
        ..hover(splitter_x + 100.0, 300.0)
    };
    tree.update(&mut behavior, &drag, AREA);

    let release = DockInput {
        pointer_released: true,
        ..hover(splitter_x + 100.0, 300.0)
    };
    tree.update(&mut behavior, &release, AREA);

    let after = width_of(&tree, a);
    assert!(
        after > before + 50.0,
        "the left pane should have grown: {before} -> {after}"
    );
    assert!(
        width_of(&tree, b) < before,
        "the right pane should have shrunk"
    );
}

/// 表示/非表示: an invisible tile keeps its place but takes up no space.
#[test]
fn oracle_visibility() {
    let mut behavior = TestBehavior;
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let mut tree = Tree::new("visibility", root, tiles);

    tree.update(&mut behavior, &DockInput::default(), AREA);
    assert!(tree.active_tiles().contains(&b));

    tree.set_visible(b, false);
    tree.update(&mut behavior, &DockInput::default(), AREA);

    assert!(
        !tree.active_tiles().contains(&b),
        "hidden tiles are not active"
    );
    assert!(tree.tiles.rect(b).is_none(), "a hidden tile has no rect");
    assert!(
        (width_of(&tree, a) - AREA.width()).abs() < 1.0,
        "the visible pane takes the whole width"
    );
    assert!(
        tree.tiles
            .get_container(root)
            .expect("root is a container")
            .children_vec()
            .contains(&b),
        "an invisible tile keeps its place in the hierarchy"
    );

    tree.set_visible(b, true);
    tree.update(&mut behavior, &DockInput::default(), AREA);
    assert!(tree.active_tiles().contains(&b), "and it comes back");
}

/// 既定presetへのreset: the tree takes the preset's layout and keeps its own id.
#[test]
fn oracle_reset_to_preset() {
    let preset = || {
        let mut tiles = Tiles::default();
        let a = tiles.insert_pane("a");
        let b = tiles.insert_pane("b");
        let root = tiles.insert_horizontal_tile(vec![a, b]);
        Tree::new("preset", root, tiles)
    };

    let mut behavior = TestBehavior;
    let mut tree = preset();
    tree.id = "dock".to_owned();
    let untouched = {
        let mut t = preset();
        t.id = "dock".to_owned();
        t
    };

    // Wreck the layout: hide one pane and drag the other away into a vertical split.
    let root = tree.root.expect("root");
    let children = tree
        .tiles
        .get_container(root)
        .expect("root is a container")
        .children_vec();
    tree.update(&mut behavior, &DockInput::default(), AREA);
    tree.set_visible(children[1], false);
    tree.update(&mut behavior, &DockInput::default(), AREA);
    assert_ne!(tree, untouched, "the layout really did change");

    tree.reset_to(preset());
    tree.update(&mut behavior, &DockInput::default(), AREA);

    assert_eq!(tree.id(), "dock", "reset keeps the tree's own id");
    assert_eq!(tree, untouched, "reset restores the preset layout");
}

// ----------------------------------------------------------------------------
// Upstream tests, kept because they guard the behaviour this port must not change.

/// Wrapping a tile in a new container must not disturb the wrapped tile's own [`TileId`].
#[test]
fn wrapping_a_tile_keeps_its_id() {
    for insertion_kind in [
        ContainerInsertion::Tabs(1),
        ContainerInsertion::Horizontal(1),
        ContainerInsertion::Vertical(1),
        ContainerInsertion::Grid(1),
    ] {
        let mut tiles = Tiles::default();
        let a = tiles.insert_pane("a");
        let b = tiles.insert_pane("b");
        let root = tiles.insert_horizontal_tile(vec![a, b]);
        let dropped = tiles.insert_pane("dropped");

        let wrapper = tiles
            .insert_at(insertion(a, insertion_kind), dropped)
            .expect("wrapping a pane should have created a container");

        assert_eq!(
            tiles.get(a),
            Some(&Tile::Pane("a")),
            "{insertion_kind:?}: pane `a` must still be a pane, under its original id"
        );
        assert_ne!(
            wrapper, a,
            "{insertion_kind:?}: the new container needs its own id"
        );

        let wrapper_children = tiles
            .get_container(wrapper)
            .expect("the wrapper should be a container")
            .children_vec();
        assert!(
            wrapper_children.contains(&a) && wrapper_children.contains(&dropped),
            "{insertion_kind:?}: the wrapper should hold both tiles, but holds {wrapper_children:?}"
        );

        assert_eq!(
            tiles
                .get_container(root)
                .expect("root should be a container")
                .children_vec(),
            vec![wrapper, b],
            "{insertion_kind:?}: the root should hold the wrapper in `a`'s old place"
        );
        assert_eq!(
            tiles.parent_of(a),
            Some(wrapper),
            "{insertion_kind:?}: `a` should now live inside the wrapper"
        );
    }
}

/// The wrapper takes over the wrapped tile's place, so it must inherit its share.
#[test]
fn the_wrapper_inherits_the_wrapped_tile_s_share() {
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let dropped = tiles.insert_pane("dropped");

    let mut shares = Shares::default();
    shares.set_share(a, 3.0);
    shares.set_share(b, 1.0);
    if let Some(Tile::Container(Container::Linear(linear))) = tiles.get_mut(root) {
        linear.shares = shares;
    }

    let wrapper = tiles
        .insert_at(insertion(a, ContainerInsertion::Vertical(1)), dropped)
        .expect("wrapping a pane should have created a container");

    let Some(Tile::Container(Container::Linear(linear))) = tiles.get(root) else {
        panic!("root should still be a linear container");
    };
    assert_eq!(
        linear.shares[wrapper], 3.0,
        "the wrapper should have taken over `a`'s share"
    );
    assert_eq!(linear.shares[b], 1.0, "`b`'s share should be untouched");
}

/// If the wrapped tile was the open tab, the container replacing it should be the open tab.
#[test]
fn the_wrapper_inherits_being_the_active_tab() {
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_tab_tile(vec![a, b]);
    let dropped = tiles.insert_pane("dropped");

    if let Some(Tile::Container(Container::Tabs(tabs))) = tiles.get_mut(root) {
        tabs.set_active(a);
    }

    let wrapper = tiles
        .insert_at(insertion(a, ContainerInsertion::Vertical(1)), dropped)
        .expect("wrapping a pane should have created a container");

    let Some(Tile::Container(Container::Tabs(tabs))) = tiles.get(root) else {
        panic!("root should still be a tabs container");
    };
    assert_eq!(
        tabs.active,
        Some(wrapper),
        "the wrapper took `a`'s place, so it should be the open tab"
    );
}

/// A grid's shares are positional, so the wrapper has to land in the wrapped tile's cell.
#[test]
fn the_wrapper_takes_the_wrapped_tile_s_grid_cell() {
    let mut tiles = Tiles::default();
    let panes: Vec<TileId> = ["a", "b", "c", "d"]
        .into_iter()
        .map(|pane| tiles.insert_pane(pane))
        .collect();
    let mut grid = Grid::new(panes.clone());
    grid.layout = GridLayout::Columns(2);
    let root = tiles.insert_new(Tile::Container(Container::Grid(grid)));
    let dropped = tiles.insert_pane("dropped");

    let wrapper = tiles
        .insert_at(
            insertion(panes[2], ContainerInsertion::Vertical(1)),
            dropped,
        )
        .expect("wrapping a pane should have created a container");

    assert_eq!(
        tiles
            .get_container(root)
            .expect("root should be a container")
            .children_vec(),
        vec![panes[0], panes[1], wrapper, panes[3]],
        "the wrapper should sit in the cell the wrapped pane occupied"
    );
}

/// Wrapping the root leaves the wrapped tile with its own id, so the _new_ container is the root.
#[test]
fn wrapping_the_root_makes_the_new_container_the_root() {
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let dropped = tiles.insert_pane("dropped");
    let mut tree = Tree::new("test", root, tiles);

    tree.move_tile(
        dropped,
        InsertionPoint {
            parent_id: root,
            insertion: ContainerInsertion::Vertical(1),
        },
        false,
    );

    let new_root = tree.root.expect("the tree should still have a root");
    assert_ne!(
        new_root, root,
        "the wrapping container should be the new root"
    );
    assert_eq!(
        tree.tiles
            .get_container(root)
            .expect("the old root should still be a container under its own id")
            .children_vec(),
        vec![a, b],
        "the wrapped container should be untouched"
    );
    assert_eq!(
        tree.tiles
            .get_container(new_root)
            .expect("the new root should be a container")
            .children_vec(),
        vec![root, dropped],
    );
    assert_eq!(tree.tiles.parent_of(root), Some(new_root));
}

/// A root the collector had to drop must stop being the root.
#[test]
fn gc_clears_a_root_it_had_to_drop() {
    let mut tiles = Tiles::default();
    let only = tiles.insert_pane("doomed");
    let mut tree = Tree::new("root_pane", only, tiles);

    tree.gc(&mut DropPane("doomed"));

    assert_eq!(
        tree.root, None,
        "the root tile is gone, so the tree must not still name it"
    );
    assert!(tree.is_empty(), "an empty tree has to say that it is empty");
}

/// A tile named as a child by two containers must lose the second *reference*, not itself.
#[test]
fn gc_drops_the_second_reference_to_a_shared_tile_and_not_the_tile() {
    let mut tiles = Tiles::default();
    let pane = tiles.insert_pane("shared");
    let first = tiles.insert_tab_tile(vec![pane]);
    let second = tiles.insert_tab_tile(vec![pane]); // the same tile, a second parent
    let root = tiles.insert_horizontal_tile(vec![first, second]);
    let mut tree = Tree::new("shared", root, tiles);

    tree.gc(&mut TestBehavior);

    assert!(
        tree.tiles.get(pane).is_some(),
        "the shared tile itself must survive - only one of the two references is bogus"
    );
    assert_eq!(
        tree.tiles
            .get_container(first)
            .expect("the first parent should still be a container")
            .children_vec(),
        vec![pane],
        "the first parent to reach the tile keeps it"
    );
    assert!(
        tree.tiles
            .get_container(second)
            .expect("the second parent should still be a container")
            .children_vec()
            .is_empty(),
        "the second reference is the one that has to go"
    );

    tree.simplify(&SimplificationOptions::default());
    assert!(
        tree.root.is_some(),
        "the tree must still have a root after gc+simplify"
    );
    assert!(
        tree.tiles.get(pane).is_some(),
        "the pane must still be in the tree after gc+simplify"
    );
}

/// `simplify` must let go of a tab it removes.
#[test]
fn simplify_lets_go_of_a_tab_it_removes() {
    let mut tiles = Tiles::default();
    let empty = tiles.insert_horizontal_tile(vec![]);
    let pane = tiles.insert_pane("keep");
    let other = tiles.insert_pane("keep too");
    let root = tiles.insert_tab_tile(vec![empty, pane, other]);
    let mut tree = Tree::new("simplify_active", root, tiles);

    match tree.tiles.get(root) {
        Some(Tile::Container(Container::Tabs(tabs))) => assert_eq!(
            tabs.active,
            Some(empty),
            "setup: the container about to be pruned is the open tab"
        ),
        other => panic!("expected a tab container, got {other:?}"),
    }

    tree.simplify(&SimplificationOptions::default());

    assert!(
        tree.tiles.get(empty).is_none(),
        "the empty container should have been pruned"
    );
    match tree.tiles.get(root) {
        Some(Tile::Container(Container::Tabs(tabs))) => {
            if let Some(active) = tabs.active {
                assert!(
                    tree.tiles.get(active).is_some(),
                    "the open tab must be a tile that still exists"
                );
                assert!(
                    tabs.children.contains(&active),
                    "the open tab must be one of the container's own tabs"
                );
            }
        }
        other => panic!("expected a tab container, got {other:?}"),
    }
}

/// Inserting into a container that already accepts the tile must not wrap anything.
#[test]
fn inserting_into_a_matching_container_creates_nothing() {
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let root = tiles.insert_horizontal_tile(vec![a]);
    let dropped = tiles.insert_pane("dropped");

    let wrapper = tiles.insert_at(insertion(root, ContainerInsertion::Horizontal(0)), dropped);

    assert_eq!(wrapper, None, "a horizontal container takes it directly");
    assert_eq!(
        tiles
            .get_container(root)
            .expect("root should be a container")
            .children_vec(),
        vec![dropped, a]
    );
}

/// `dock/` must not reach for egui: the whole point of the port.
#[test]
fn the_dock_holds_no_egui() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/dock");
    let mut offenders = Vec::new();

    fn walk(dir: &std::path::Path, offenders: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).expect("dock/ should be readable") {
            let path = entry.expect("readable entry").path();
            if path.is_dir() {
                walk(&path, offenders);
            } else if path.file_name().is_some_and(|name| name == "tests.rs") {
                // このファイル自身は判定文字列として `egui` を持つので除く。
                continue;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let source = std::fs::read_to_string(&path).expect("readable file");
                for (i, line) in source.lines().enumerate() {
                    // Comments name egui to say what was replaced; code must not.
                    let code = line.split("//").next().unwrap_or_default();
                    if code.contains("egui") {
                        offenders.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
                    }
                }
            }
        }
    }

    walk(&dir, &mut offenders);
    assert!(
        offenders.is_empty(),
        "dock/ must not depend on egui, found:\n{}",
        offenders.join("\n")
    );
}
