use dioxus_workbench::TileId;
use crate::ui::dock::Side;
use crate::ui::app::TileNodes;

pub(super) fn dock_side_at(x: f64, y: f64, width: f64, height: f64) -> Side {
    if y < height * 0.25 {
        Side::Top
    } else if y > height * 0.75 {
        Side::Bottom
    } else if x < width * 0.25 {
        Side::Left
    } else if x > width * 0.75 {
        Side::Right
    } else {
        Side::Center
    }
}

pub(super) fn node_has_id(node: &blitz_dom::Node, expected: &str) -> bool {
    node.attr(blitz_dom::local_name!("id")).is_some_and(|id| id == expected)
}

pub(super) fn dock_target_at(tile_nodes: &TileNodes, x: f64, y: f64) -> Option<(TileId, Side)> {
    let mounted = tile_nodes
        .borrow()
        .iter()
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect::<Vec<_>>();
    for (id, handle) in mounted {
        let Some(doc) = handle.try_doc() else { continue };
        let Some(node) = doc.get_node(handle.node_id()) else { continue };
        // 消えた箱の id は次に作られた節へ再利用される。本人でなければ古い取っ手。
        if !node_has_id(node, &format!("tile-{}", id.as_str())) {
            continue;
        }
        let origin = node.absolute_position(0.0, 0.0);
        let size = node.final_layout().size;
        let local_x = x - f64::from(origin.x);
        let local_y = y - f64::from(origin.y);
        let width = f64::from(size.width);
        let height = f64::from(size.height);
        if local_x >= 0.0 && local_y >= 0.0 && local_x <= width && local_y <= height {
            return Some((id, dock_side_at(local_x, local_y, width, height)));
        }
    }
    None
}
