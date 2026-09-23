//! Which layers the block solver moves and how they are related. Decided from
//! evaluated semantics plus the picture sizes the executor already produced.

use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::doc::store::{LayerId, RationalTime, Value};
use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::render::compositor::effects::block_program::BlockItem;
use crate::render::compositor::effects::catalog::CatalogSnapshot;
use crate::render::compositor::effects::isf::IsfStage;

/// 効果の列の何番目のブロックか・どのブロックか・欄の値が同じ物を、1 回の計算に束ねる。
pub struct BlockBatch {
    pub stage: usize,
    pub plugin: String,
    pub params: Vec<f32>,
    pub members: Vec<u32>,
    /// 場(`SCOPE: room`)の元の物の番号。掛かった層そのものが動くブロックは `u32::MAX`。
    pub source: u32,
    /// 名指しの深さ(親・anchor を辿った段数)。同じ stage では浅い方(読まれる側)が先に走る。
    pub rank: usize,
}

/// 形の層の輪郭(素材座標)。当たりは四角ではなく、この形そのもので見る
/// (利用者 2026-09-16「今のコリジョンの当たり判定は四角で変です。2d も 3d もシルエットが算出できるはず」)。
/// 2D は書類の形(`vector::resolve`)、3D の網・粒は `media::silhouette_points`(まだ箱のまま)。
fn outline_of(shapes: &[crate::doc::vector::ShapeNode], stretch: [f32; 2]) -> Option<Vec<[f32; 2]>> {
    let shapes = if stretch == [1.0, 1.0] { shapes.to_vec() } else { crate::picture::shapes_ops::stretch_outline(shapes, stretch) };
    let leaves = crate::picture::shapes_ops::flatten(&shapes).ok()?;
    let canvas = crate::picture::shapes_ops::content_canvas(&shapes).ok().flatten()?;
    let (ox, oy) = (canvas.origin_x as f32, canvas.origin_y as f32);
    let mut points = Vec::new();
    for shape in &leaves {
        if shape.fill.is_none() {
            continue;
        }
        for instance in crate::picture::shapes_ops::resolve(shape).ok()?.iter() {
            flatten_contours(&instance.path, [ox, oy], &mut points);
        }
    }
    (points.len() >= 3).then_some(points)
}

/// 輪郭の列を点に割る(曲がりに合わせて、lyon の適応分割)。文字も形も同じ。
fn flatten_contours(contours: &[crate::doc::vector::Contour], offset: [f32; 2], points: &mut Vec<[f32; 2]>) {
    let (ox, oy) = (offset[0], offset[1]);
    for contour in contours {
        let vs = &contour.vertices;
        if vs.len() < 2 {
            continue;
        }
        let last = if contour.closed { vs.len() } else { vs.len() - 1 };
        for i in 0..last {
            let (a, b) = (&vs[i], &vs[(i + 1) % vs.len()]);
            let p = |x: f64, y: f64| lyon_geom::point(x as f32, y as f32);
            let curve = lyon_geom::CubicBezierSegment {
                from: p(a.point.x, a.point.y),
                ctrl1: p(a.point.x + a.out_tangent.x, a.point.y + a.out_tangent.y),
                ctrl2: p(b.point.x + b.in_tangent.x, b.point.y + b.in_tangent.y),
                to: p(b.point.x, b.point.y),
            };
            points.push([curve.from.x + ox, curve.from.y + oy]);
            curve.for_each_flattened(0.6, &mut |line| {
                points.push([line.to.x + ox, line.to.y + oy]);
            });
        }
    }
}

/// 名指しの深さ: 相手(親・anchor)の無い物は 0、相手が居れば相手の深さ + 1。輪は 0 で切る。
/// 同じ stage の batch はこの順に走るので、読む側は読まれる側の今を見る。
fn reference_depth(items: &[BlockItem]) -> Vec<usize> {
    fn go(items: &[BlockItem], k: usize, memo: &mut [Option<usize>], visiting: &mut Vec<usize>) -> usize {
        if let Some(d) = memo[k] {
            return d;
        }
        if visiting.contains(&k) {
            return 0;
        }
        visiting.push(k);
        let d = [items[k].parent_slot, items[k].anchor_slot].into_iter()
            .filter(|j| (*j as usize) < items.len())
            .map(|j| go(items, j as usize, memo, visiting) + 1)
            .max().unwrap_or(0);
        visiting.pop();
        memo[k] = Some(d);
        d
    }
    let mut memo = vec![None; items.len()];
    (0..items.len()).map(|k| go(items, k, &mut memo, &mut Vec::new())).collect()
}

/// 書類から先に読む、物ごとの住む箱と箱。
#[derive(Clone)]
pub struct Placed {
    pub room: [f32; 4],
    pub radius: f32,
    pub own: [f32; 4],
    pub group: u32,
    pub weight: f32,
    /// 書類の間合い(CSS の margin)。物同士はこれだけ空けて当たる。
    pub margin: f32,
    /// 手触り(0 返す ↔ 0.5 吸う ↔ 1 引きずる)。
    pub hardness: f32,
    /// 形そのものの輪郭(素材座標)。無ければ箱で当たる。
    pub outline: Option<std::sync::Arc<Vec<[f32; 2]>>>,
}

/// The solver's objects for one frame, in slot order.
#[derive(Default)]
pub struct BlockPlan {
    pub lies: crate::render::engine::physics::Lies,
    pub active: bool,
    pub placed: HashMap<LayerId, Placed>,
    pub follows: HashMap<LayerId, LayerId>,
    pub object_layers: Vec<LayerId>,
    pub field_rooms: std::collections::HashSet<u32>,
    pub object_frames: Vec<i64>,
    pub fields: Vec<(usize, String, Vec<f32>, u32)>,
    pub objects: Vec<BlockItem>,
    pub bases: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    pub outlines: Vec<Option<std::sync::Arc<Vec<[f32; 2]>>>>,
    pub slots: HashMap<LayerId, u32>,
    pub connectors: Vec<(LayerId, u32, u32)>,
    pub traces: HashMap<LayerId, u32>,
    pub ropes: Vec<(u32, f32, f32, f32)>,
    pub batches: Vec<BlockBatch>,
}

pub fn plan_blocks(
    scene: &SceneValue,
    solver: &SolverPlanValue,
    catalog: &CatalogSnapshot,
    comp: CompSpec,
    t: RationalTime,
    fps: crate::doc::store::Fps,
    picture_sizes: &HashMap<LayerId, [f32; 2]>,
) -> BlockPlan {
    let block_ids: Vec<String> = catalog.definitions.iter()
        .filter(|definition| definition.manifest.stage == IsfStage::Block)
        .map(|definition| definition.plugin_id().to_owned())
        .collect();
    let definitions: Vec<(String, bool, Vec<(String, f32)>)> = catalog.definitions.iter()
        .filter(|definition| definition.manifest.stage == IsfStage::Block)
        .map(|definition| (
            definition.plugin_id().to_owned(),
            definition.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room,
            definition.manifest.param_inputs().map(|param| (param.name.clone(), param.default[0])).collect(),
        ))
        .collect();
    let field_blocks: std::collections::HashSet<String> = definitions.iter()
        .filter(|(_, field, _)| *field)
        .map(|(plugin, _, _)| plugin.clone())
        .collect();
    let lies = catalog.definitions.iter()
        .find(|definition| {
            definition.manifest.stage == IsfStage::Block
                && definition.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room
                && !definition.manifest.physics.is_empty()
        })
        .map(|definition| crate::render::engine::physics::Lies::from_manifest(&definition.manifest.physics))
        .unwrap_or_default();
    let mut world = BlockPlan { lies: lies.clone(), active: !block_ids.is_empty(), ..Default::default() };
    let state = &mut world;

    if block_ids.is_empty() {
        return world;
    }

    let mut seen = std::collections::HashSet::new();
    let primary: Vec<&SceneLayerValue> = scene.layers.iter()
        .filter(|layer| layer.instance == 0 && !layer.ghost && seen.insert(layer.layer))
        .collect();
    let primary_by_id: HashMap<LayerId, &SceneLayerValue> =
        primary.iter().map(|layer| (layer.layer, *layer)).collect();


    let semantic_size = |layer: &SceneLayerValue| -> Option<[f32; 2]> {
        let shapes: Option<std::sync::Arc<Vec<crate::doc::store::ShapeNode>>> = match &layer.content {
            SceneContentValue::Shape(shapes) => Some(shapes.clone()),
            SceneContentValue::Text(text) => Some(text.shapes()),
            _ => None,
        };
        if let Some(shapes) = shapes {
            if let Ok(Some(canvas)) = crate::picture::shapes_ops::content_canvas(&shapes) {
                return Some([canvas.width as f32, canvas.height as f32]);
            }
        }
        match &layer.content {
            SceneContentValue::Particles(value) => {
                let mut hi = glam::Vec2::ZERO;
                for particle in &value.particles {
                    hi = hi.max(glam::Vec2::new(particle.position[0], particle.position[1]));
                }
                Some([hi.x.max(1.0), hi.y.max(1.0)])
            }
            SceneContentValue::Plate(_) => Some([comp.width as f32, comp.height as f32]),
            _ => None,
        }
    };
    let size_of = |id: LayerId| -> Option<[f32; 2]> {
        solver.layers.get(&id).and_then(|layer| layer.size)
            .filter(|size| size[0] > 0.0 && size[1] > 0.0)
            .or_else(|| primary_by_id.get(&id).and_then(|layer| semantic_size(layer)))
            .or_else(|| picture_sizes.get(&id).copied())
    };

    // A field establishes a room at its nearest authored parent.
    for layer in &primary {
        let has_field = layer.effects.iter().any(|effect| field_blocks.contains(&effect.plugin_id));
        if has_field {
            let parent = solver.layers.get(&layer.layer).and_then(|value| value.relation.parent);
            state.field_rooms.insert(parent.map_or(0, |parent| parent.0 as u32));
        }
    }
    let field_rooms = state.field_rooms.clone();
    let room_ancestor = lies.room_ancestor;
    let room_of = |id: LayerId| -> Option<u32> {
        let mut at = solver.layers.get(&id).and_then(|value| value.relation.parent);
        loop {
            let key = at.map_or(0, |parent| parent.0 as u32);
            if field_rooms.contains(&key) {
                return Some(key);
            }
            if !room_ancestor {
                return None;
            }
            let parent = at?;
            at = solver.layers.get(&parent).and_then(|value| value.relation.parent);
        }
    };

    let mut needed = std::collections::HashSet::new();
    for layer in &primary {
        let Some(plan) = solver.layers.get(&layer.layer) else { continue };
        if let Some((from, to)) = plan.relation.connection {
            if primary_by_id.contains_key(&from) && primary_by_id.contains_key(&to) {
                needed.insert(from);
                needed.insert(to);
            }
        }
        if let Some((target, _)) = plan.relation.trace {
            if primary_by_id.contains_key(&target) {
                needed.insert(target);
            }
        }
    }

    let mut named = std::collections::HashSet::new();
    for layer in &primary {
        let has_block = layer.effects.iter().any(|effect| block_ids.contains(&effect.plugin_id));
        if !has_block { continue; }
        if let Some(plan) = solver.layers.get(&layer.layer) {
            if let Some(parent) = plan.relation.parent { named.insert(parent); }
            if let Some(anchor) = plan.relation.anchor { named.insert(anchor); }
        }
    }

    let mut room_ids = HashMap::new();
    let mut wanted = Vec::new();
    for layer in &primary {
        if layer.effects.iter().any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id)) {
            continue;
        }
        let plan = solver.layers.get(&layer.layer);
        let has_block = layer.effects.iter().any(|effect| block_ids.contains(&effect.plugin_id));
        if !has_block && plan.is_some_and(|plan| plan.relation.connection.is_some() || plan.relation.trace.is_some()) {
            continue;
        }
        let is_group = layer.source == crate::doc::store::LayerSource::Group;
        match room_of(layer.layer) {
            Some(room) if has_block || !is_group || !room_ancestor => {
                room_ids.insert(layer.layer, room);
                wanted.push(layer.layer);
            }
            None if has_block || needed.contains(&layer.layer) || named.contains(&layer.layer) => {
                wanted.push(layer.layer);
            }
            _ => {}
        }
    }

    // Build the semantic collision/input records. Rooms and own boxes come
    // from evaluated transforms + Flow/content extents, not Document reads.
    for id in &wanted {
        let Some(layer) = primary_by_id.get(id).copied() else { continue };
        let Some(plan) = solver.layers.get(id) else { continue };
        let frame = ([0.0, 0.0, comp.width as f32, comp.height as f32], 0.0);
        let parent = match room_ids.get(id) {
            Some(0) => None,
            Some(room) => Some(LayerId(*room as u64)),
            None => plan.relation.parent,
        };
        let room = match parent.and_then(|parent| primary_by_id.get(&parent).map(|layer| (parent, *layer))) {
            Some((parent_id, parent_layer)) => {
                let size = size_of(parent_id).unwrap_or([comp.width as f32, comp.height as f32]);
                let transform = parent_layer.transform.affine;
                let corners = [
                    glam::Vec2::ZERO,
                    glam::vec2(size[0], 0.0),
                    glam::Vec2::from(size),
                    glam::vec2(0.0, size[1]),
                ].map(|point| transform.transform_point2(point));
                let lo = corners.iter().fold(glam::Vec2::MAX, |acc, point| acc.min(*point));
                let hi = corners.iter().fold(glam::Vec2::MIN, |acc, point| acc.max(*point));
                let radius = solver.layers.get(&parent_id).map_or(0.0, |value| {
                    value.border_radius * transform.matrix2.x_axis.length()
                });
                ([lo.x, lo.y, hi.x, hi.y], radius)
            }
            None => frame,
        };

        let own_size = size_of(*id).or_else(|| named.contains(id).then_some([0.0, 0.0]));
        let Some(size) = own_size else { continue };
        let stretch = plan.stretch;
        let own = [0.0, 0.0, size[0] * stretch[0], size[1] * stretch[1]];
        let outline = match &layer.content {
            SceneContentValue::Shape(shapes) => outline_of(shapes, stretch).map(std::sync::Arc::new),
            SceneContentValue::Text(text) => outline_of(&text.shapes(), stretch).map(std::sync::Arc::new),
            _ => None,
        };
        state.placed.insert(*id, Placed {
            room: room.0,
            radius: room.1,
            own,
            group: parent.map_or(0, |parent| parent.0 as u32),
            weight: plan.weight,
            margin: plan.margin,
            hardness: plan.hardness,
            outline,
        });
    }

    // Follow is now a graph relation. It participates only when the target
    // is one of the solver objects, exactly as the legacy GPU FollowPass.
    for layer in &primary {
        let Some(plan) = solver.layers.get(&layer.layer) else { continue };
        let Some(target) = plan.relation.anchor.filter(|_| plan.relation.follow_anchor) else { continue };
        if !state.placed.contains_key(&target) { continue; }
        state.follows.insert(layer.layer, target);
        if !state.placed.contains_key(&layer.layer) {
            let size = size_of(layer.layer).unwrap_or([0.0, 0.0]);
            state.placed.insert(layer.layer, Placed {
                room: [0.0; 4],
                radius: 0.0,
                own: [0.0, 0.0, size[0], size[1]],
                group: u32::MAX,
                weight: 0.0,
                margin: 0.0,
                hardness: 0.5,
                outline: None,
            });
        }
    }

    let mut pending: Vec<(u32, Vec<(String, Vec<f32>, bool)>)> = Vec::new();
    let mut ordered = Vec::new();
    let mut late = Vec::new();
    for layer in &primary {
        let Some(placed) = state.placed.get(&layer.layer) else { continue };
        let has_block = layer.effects.iter().any(|effect| block_ids.contains(&effect.plugin_id));
        if has_block || state.follows.contains_key(&layer.layer) || state.field_rooms.contains(&placed.group) || needed.contains(&layer.layer) {
            ordered.push(*layer);
        } else if named.contains(&layer.layer) {
            late.push(*layer);
        }
    }
    ordered.extend(late);

    let current_frame = t.try_to_frame_round(fps).unwrap_or(0);
    for layer in ordered {
        let Some(placed) = state.placed.get(&layer.layer).cloned() else { continue };
        let blocks_here: Vec<(String, Vec<f32>, bool)> = layer.effects.iter().filter_map(|effect| {
            let definition = definitions.iter().find(|definition| definition.0 == effect.plugin_id)?;
            let params = definition.2.iter().map(|(name, default)| {
                effect.params.iter().find(|(candidate, _)| candidate == name).and_then(|(_, value)| match value {
                    Value::F64(value) => Some(*value as f32),
                    Value::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
                    _ => None,
                }).unwrap_or(*default)
            }).collect();
            Some((effect.plugin_id.clone(), params, definition.1))
        }).collect();
        let transform = layer.transform.affine;
        let own = placed.own;
        let corners = [
            [own[0], own[1]], [own[2], own[1]], [own[0], own[3]], [own[2], own[3]],
        ].map(|point| transform.transform_point2(glam::Vec2::from(point)));
        let lo = corners.iter().fold(glam::Vec2::MAX, |acc, point| acc.min(*point));
        let hi = corners.iter().fold(glam::Vec2::MIN, |acc, point| acc.max(*point));
        let k = state.objects.len() as u32;
        state.slots.insert(layer.layer, k);
        state.objects.push(BlockItem {
            lo: lo.to_array(),
            hi: hi.to_array(),
            room_lo: [placed.room[0], placed.room[1]],
            room_size: [placed.room[2] - placed.room[0], placed.room[3] - placed.room[1]],
            radius: placed.radius,
            group: placed.group,
            margin: placed.margin,
            weight: placed.weight,
            ..Default::default()
        });
        state.outlines.push(placed.outline.as_ref().map(|points| {
            std::sync::Arc::new(points.iter().map(|point| transform.transform_point2(glam::Vec2::from(*point)).to_array()).collect())
        }));
        state.bases.push(([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]));
        state.object_layers.push(layer.layer);
        state.object_frames.push(current_frame);
        pending.push((k, blocks_here));
    }

    for k in 0..state.objects.len() {
        let id = state.object_layers[k];
        let relation = solver.layers.get(&id).map(|value| &value.relation);
        let slot = |target: Option<LayerId>| target
            .and_then(|target| state.slots.get(&target).copied())
            .unwrap_or(crate::render::compositor::effects::block_program::NO_OBJECT);
        state.objects[k].parent_slot = slot(relation.and_then(|value| value.parent));
        state.objects[k].anchor_slot = slot(relation.and_then(|value| value.anchor));
    }

    let depth = reference_depth(&state.objects);
    for (k, blocks_here) in pending {
        let rank = depth[k as usize];
        for (stage, (plugin, params, is_field)) in blocks_here.into_iter().enumerate() {
            if is_field {
                state.fields.push((stage, plugin, params, k));
            } else {
                match state.batches.iter_mut().find(|batch| {
                    batch.stage == stage && batch.rank == rank && batch.plugin == plugin && batch.params == params && batch.source == u32::MAX
                }) {
                    Some(batch) => batch.members.push(k),
                    None => state.batches.push(BlockBatch { stage, plugin, params, members: vec![k], source: u32::MAX, rank }),
                }
            }
        }
    }
    state.batches.sort_by_key(|batch| (batch.stage, batch.rank));

    for layer in &primary {
        let Some(relation) = solver.layers.get(&layer.layer).map(|value| &value.relation) else { continue };
        if let Some((from, to)) = relation.connection {
            if let (Some(&from_slot), Some(&to_slot)) = (state.slots.get(&from), state.slots.get(&to)) {
                let index = state.connectors.len() as u32;
                if let Some(slack) = relation.rope_slack {
                    state.ropes.push((index, slack, 60.0, 6.0));
                }
                state.connectors.push((layer.layer, from_slot, to_slot));
                continue;
            }
        }
        if let Some((target, _)) = relation.trace {
            if let Some(&slot) = state.slots.get(&target) {
                state.traces.insert(layer.layer, slot);
            }
        }
    }

    world
}
