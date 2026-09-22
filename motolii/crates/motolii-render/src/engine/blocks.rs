//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で効果の順に解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::doc::store::{LayerId, MaskFrame, PropertyId, RationalTime, StoreView, Value};
use crate::render::compositor::effects::block_program::{BlockItem, BlockProgram, BlockWorld, FollowPass, WorldPass};
use crate::render::compositor::effects::isf::IsfStage;
use crate::render::compositor::Layer;
use crate::render::engine::{Engine, EngineError};

/// 効果の列の何番目のブロックか・どのブロックか・欄の値が同じ物を、1 回の計算に束ねる。
pub(crate) struct BlockBatch {
    stage: usize,
    plugin: String,
    params: Vec<f32>,
    members: Vec<u32>,
    /// 場(`SCOPE: room`)の元の物の番号。掛かった層そのものが動くブロックは `u32::MAX`。
    source: u32,
    /// 名指しの深さ(親・anchor を辿った段数)。同じ stage では浅い方(読まれる側)が先に走る。
    rank: usize,
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
struct Placed {
    room: [f32; 4],
    radius: f32,
    own: [f32; 4],
    group: u32,
    weight: f32,
    /// 書類の間合い(CSS の margin)。物同士はこれだけ空けて当たる。
    margin: f32,
    /// 手触り(0 返す ↔ 0.5 吸う ↔ 1 引きずる)。
    hardness: f32,
    /// 形そのものの輪郭(素材座標)。無ければ箱で当たる。
    outline: Option<std::sync::Arc<Vec<[f32; 2]>>>,
}

#[derive(Default)]
pub(crate) struct BlockState {
    /// ブロックごとの GPU の道と、組んだ時の本文(棚が読み直されたら組み直す)。
    programs: HashMap<String, (String, BlockProgram)>,
    world: Option<BlockWorld>,
    world_pass: Option<WorldPass>,
    follow_pass: Option<FollowPass>,
    placed: HashMap<LayerId, Placed>,
    /// 付いて置く札 → 相手(相手がブロックを持つ時だけ)。
    follows: HashMap<LayerId, LayerId>,
    /// 物の番号 → 層(付いて行く対を番号にする)。
    object_layers: Vec<LayerId>,
    /// 場が立っている住む箱の組(そこに居る物は、効果を持たなくても動く物として並べる)。
    field_rooms: std::collections::HashSet<u32>,
    /// comp の最後のコマ(集まる時の「終わり」)。
    last_frame: i64,
    /// 物ごとの時刻(順番の札でずれたコマ)。object_layers と同じ並び。
    object_frames: Vec<i64>,
    /// 場の元(物の番号)と、その効果の順・欄。動く相手は物が揃ってから決める。
    fields: Vec<(usize, String, Vec<f32>, u32)>,
    objects: Vec<BlockItem>,
    /// 外の解き手(Rapier)。前のコマの力を覚えている。
    physics: crate::render::engine::physics::Physics,
    /// このコマの秒あたりのコマ数と時刻(可視の層が描く直前に解くために覚える)。
    fps: f64,
    now: Option<RationalTime>,
    bases: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    /// 物ごとの輪郭(comp の px、`objects` と同じ順)。
    outlines: Vec<Option<std::sync::Arc<Vec<[f32; 2]>>>>,
    /// 層 → 物の番号(層を組む時に motion の番号を配るため)。
    slots: HashMap<LayerId, u32>,
    /// つなぐ線 → (元の物, 先の物)。物の後ろに並ぶ motion(kind 1)で両端に付いて行く。
    connectors: Vec<(LayerId, u32, u32)>,
    /// なぞる形 → 相手の物。相手の motion をそのまま読む。
    traces: HashMap<LayerId, u32>,
    /// 紐(Line Path = Rope)のつなぐ線: (connectors の番号, Slack %, 硬さ, 減衰)。
    ropes: Vec<(u32, f32, f32, f32)>,
    rope_pass: Option<crate::render::compositor::effects::block_program::RopePass>,
    batches: Vec<BlockBatch>,
}

impl BlockState {
    /// 解き手が動かす物か(描く前に間引かないため)。
    pub(crate) fn moves(&self, layer: LayerId) -> bool {
        self.slots.contains_key(&layer)
    }
}

impl Engine {
    /// 今のコマの物ごとのずれ(震えを測る道具のため。読み戻すので描画では使わない)。
    /// 測り用: 当たりに使っている輪郭の外接(comp の px、解き手が動かした後)。輪郭の無い物は箱。
    pub fn physics_outline_bounds(&self) -> Vec<[f32; 4]> {
        let hulls = self.physics_hulls();
        let mut out = Vec::new();
        for (k, it) in self.blocks.objects.iter().enumerate() {
            let layer = self.blocks.object_layers[k];
            let Some((shift, _)) = self.blocks.physics.offset(layer) else { continue };
            let bounds = match hulls.get(k).filter(|h| h.len() >= 3) {
                Some(h) => h.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, p| [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])]),
                None => [it.lo[0] + shift[0], it.lo[1] + shift[1], it.hi[0] + shift[0], it.hi[1] + shift[1]],
            };
            out.push(bounds);
        }
        out
    }

    /// 可視のモードが読む: 物理の物の箱(comp の px、解き手が動かした後)。
    pub(crate) fn physics_marks(&self) -> Vec<crate::doc::store::analysis::BlobMark> {
        self.blocks.objects.iter().enumerate().filter_map(|(k, it)| {
            let layer = *self.blocks.object_layers.get(k)?;
            let (shift, _) = self.blocks.physics.offset(layer)?;
            Some(crate::doc::store::analysis::BlobMark {
                id: k as u32,
                center: [(it.lo[0] + it.hi[0]) * 0.5 + shift[0], (it.lo[1] + it.hi[1]) * 0.5 + shift[1]],
                size: [it.hi[0] - it.lo[0], it.hi[1] - it.lo[1]],
                age: 0,
            })
        }).collect()
    }

    /// 可視のモードが読む: 物ごとのずれ、触れ合いの線、場の元と届く輪。
    pub(crate) fn physics_offset(&self, layer: LayerId) -> Option<([f32; 2], f32)> {
        self.blocks.physics.offset(layer)
    }

    pub(crate) fn physics_links(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.contact_lines()
    }

    pub(crate) fn physics_wells(&self) -> Vec<([f32; 2], f32, [f32; 2])> {
        self.blocks.physics.wells()
    }

    pub(crate) fn physics_contacts_at(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.contact_marks()
    }

    pub(crate) fn physics_velocities(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.blocks.physics.velocities()
    }

    /// 当たりに使っている輪郭(解き手が動かした後の場所へ移したもの)。
    pub(crate) fn physics_hulls(&self) -> Vec<Vec<[f32; 2]>> {
        self.blocks.objects.iter().enumerate().filter_map(|(k, _)| {
            let layer = *self.blocks.object_layers.get(k)?;
            let outline = self.blocks.outlines.get(k)?.as_ref()?;
            let (shift, turn) = self.blocks.physics.offset(layer)?;
            let (sin, cos) = turn.to_radians().sin_cos();
            let it = self.blocks.objects.get(k)?;
            let mid = glam::vec2((it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5);
            Some(outline.iter().map(|p| {
                let d = glam::Vec2::from(*p) - mid;
                let turned = glam::vec2(d.x * cos - d.y * sin, d.x * sin + d.y * cos);
                (mid + turned + glam::Vec2::from(shift)).to_array()
            }).collect())
        }).collect()
    }

    /// 触れ合っている組の数(測り用)。
    pub fn physics_contacts(&self) -> usize {
        self.blocks.physics.contacts()
    }

    pub fn block_states(&self) -> Vec<[f32; 3]> {
        let Some(world) = self.blocks.world.as_ref() else { return Vec::new() };
        let ctx = &self.compositor.ctx;
        let encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-block-read") });
        crate::render::compositor::effects::block_program::read_state(&ctx.device, &ctx.queue, world, encoder)
            .iter().map(|o| [o.translate[0], o.translate[1], o.rotate]).collect()
    }

    /// 計測の口: 物ごとの今のずれを全部(位置 2・回転・大きさ・色 3・不透明)。描く道は読み戻さない。
    pub fn block_offsets(&self) -> Vec<[f32; 8]> {
        let Some(world) = self.blocks.world.as_ref() else { return Vec::new() };
        let ctx = &self.compositor.ctx;
        let encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-block-read") });
        crate::render::compositor::effects::block_program::read_state(&ctx.device, &ctx.queue, world, encoder)
            .iter().map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale, o.tint[0], o.tint[1], o.tint[2], o.tint[3]]).collect()
    }

    /// 祖先の箱の切り(`MaskFrame::Box`)を、ブロックのずれの後に箱の枠で掛ける層か: 解き手が動かす、平らに置かれた、
    /// 本番の組み(辿り直しの中でない)。
    pub(super) fn cut_after_motion(&self, layer: &ResolvedLayer) -> bool {
        self.blocks.moves(layer.id)
            && layer.placement.z == 0.0 && layer.placement.rotation_x == 0.0 && layer.placement.rotation_y == 0.0
            && layer.masks.iter().any(|m| m.frame == MaskFrame::Box)
    }

    /// 箱の切りを comp の px へ(素材座標 → 置き方)。ずれの後の絵は comp 大なので、そこで掛ける。
    pub(super) fn box_cut_in_comp(&self, layer: &ResolvedLayer, built: &Layer) -> Vec<ResolvedMask> {
        if !self.cut_after_motion(layer) {
            return Vec::new();
        }
        let origin = built.frame.as_ref().map_or(glam::Vec2::ZERO, |f| glam::Vec2::from(f.origin));
        let to = built.placement.transform * glam::Affine2::from_translation(-origin);
        let scale = (to.matrix2.x_axis.length() + to.matrix2.y_axis.length()) * 0.5;
        layer.masks.iter().filter(|m| m.frame == MaskFrame::Box).map(|m| {
            let mut mask = m.clone();
            for vertex in &mut mask.shape.vertices {
                let p = to.transform_point2(glam::vec2(vertex.point[0] as f32, vertex.point[1] as f32));
                vertex.point = [f64::from(p.x), f64::from(p.y)];
                for tangent in [&mut vertex.in_tangent, &mut vertex.out_tangent] {
                    let d = to.transform_vector2(glam::vec2(tangent[0] as f32, tangent[1] as f32));
                    *tangent = [f64::from(d.x), f64::from(d.y)];
                }
            }
            mask.expansion *= f64::from(scale);
            mask
        }).collect()
    }

    /// 組んだ 1 枚にブロックが掛かっていれば、描く時と同じ置き方の箱を物として並べ、motion の番号を最後の欄に入れる。
    pub(super) fn attach_block(&mut self, layer: &ResolvedLayer, built: &mut Layer, comp: CompSpec) {
        self.attach_block_id(layer.id, built, comp);
    }

    pub(crate) fn attach_block_id(&mut self, layer: LayerId, built: &mut Layer, comp: CompSpec) {
        // 物は既に決まっている(層を組む前に揃えて解いた)。ここでするのは、描く側へ渡す番号と、
        // その物の comp → world の向き(組んだ素材の大きさが要るのでここでしか作れない)。
        let Some(&k) = self.blocks.slots.get(&layer) else {
            if let Some(c) = self.blocks.connectors.iter().position(|(id, _, _)| *id == layer) {
                built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (self.blocks.objects.len() + 2 * c + 1) as f32;
                if self.blocks.ropes.iter().any(|(index, ..)| *index as usize == c) {
                    built.shading.grid_hint = 48;
                }
            } else if let Some(&target) = self.blocks.traces.get(&layer) {
                built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (target + 1) as f32;
            }
            return;
        };
        let size = glam::Vec2::from(built.size);
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (k + 1) as f32;
        let m = built.placement.transform;
        let (origin, u, v) = crate::render::compositor::projected_placement_corners(comp, built.projection_camera, built.projection, built.placement, glam::Vec2::ZERO, size);
        let (per_x, per_y) = (u / size.x, v / size.y);
        let inverse = if m.matrix2.determinant().abs() > 1e-12 { m.matrix2.inverse() } else { glam::Mat2::IDENTITY };
        let world = |comp_step: glam::Vec2| { let material = inverse * comp_step; (per_x * material.x + per_y * material.y).to_array() };
        let own = self.blocks.placed.get(&layer).map(|p| p.own).unwrap_or([0.0; 4]);
        let middle = glam::Vec2::new((own[0] + own[2]) * 0.5, (own[1] + own[3]) * 0.5);
        let centre = origin + u * (middle.x / size.x) + v * (middle.y / size.y);
        if let Some(slot) = self.blocks.bases.get_mut(k as usize) {
            *slot = (world(glam::Vec2::X), world(glam::Vec2::Y), centre.to_array());
        }
    }

    /// 意図(場・箱)を外の解き手へ渡してこのコマまで進める。同じコマなら何度呼んでも進まない。
    /// 可視の層は、絵を組む途中でこれを呼んでから中身を読む。
    pub(crate) fn solve_physics_now(&mut self) {
        if let Some(t) = self.blocks.now {
            self.solve_physics(t);
        }
    }

    pub(crate) fn solve_physics(&mut self, t: RationalTime) {
        let fps = self.blocks.fps.max(1.0);
        let state = &mut self.blocks;
        if state.objects.is_empty() {
            return;
        }
        // 場と箱は外の解き手(Rapier)に渡す。ここが持つのは意図からの訳だけで、解き方は持たない
        // (利用者 2026-09-16「物理演算を 1 から作るんじゃなくてこれも外部に揃ったやつがあるだろ」)。
        use crate::render::engine::physics::{Body, Room, Well};
        let mut rooms: HashMap<u32, Room> = HashMap::new();
        let mut gathering = false;
        for (stage, plugin, params, source) in std::mem::take(&mut state.fields) {
            let _ = (stage, &plugin);
            let item = state.objects[source as usize];
            let (turn, spread, angle, strength, reach, hold, gather) = (
                params.first().copied().unwrap_or(0.0).to_radians(),
                params.get(1).copied().unwrap_or(0.0).clamp(0.0, 1.0),
                params.get(2).copied().unwrap_or(90.0).to_radians(),
                params.get(3).copied().unwrap_or(0.0),
                params.get(4).copied().unwrap_or(0.0),
                params.get(5).copied().unwrap_or(0.0).clamp(0.0, 1.0),
                params.get(6).copied().unwrap_or(0.0) >= 0.5 && state.physics.lies.gather,
            );
            gathering |= gather;
            let room = rooms.entry(item.group).or_insert_with(|| Room {
                group: item.group,
                rect: [item.room_lo[0], item.room_lo[1], item.room_lo[0] + item.room_size[0], item.room_lo[1] + item.room_size[1]],
                round: item.radius * 2.0 >= item.room_size[0].min(item.room_size[1]) - 1e-3,
                gravity: [0.0, 0.0],
                wells: Vec::new(),
            });
            room.gravity[0] += angle.cos() * strength * spread;
            room.gravity[1] += angle.sin() * strength * spread;
            if spread < 1.0 {
                room.wells.push(Well {
                    at: [(item.lo[0] + item.hi[0]) * 0.5, (item.lo[1] + item.hi[1]) * 0.5],
                    pull: turn.cos() * strength * (1.0 - spread),
                    swirl: turn.sin() * strength * (1.0 - spread),
                    reach,
                    hold,
                });
            }
        }
        let frame = (t.as_seconds_f64() * fps).round() as i64;
        let bodies: Vec<Body> = state.objects.iter().enumerate()
            .filter(|(_, it)| rooms.contains_key(&it.group))
            .map(|(k, it)| Body {
                layer: state.object_layers[k],
                group: it.group,
                frame: state.object_frames.get(k).copied().unwrap_or(frame),
                centre: [(it.lo[0] + it.hi[0]) * 0.5, (it.lo[1] + it.hi[1]) * 0.5],
                half: [(it.hi[0] - it.lo[0]) * 0.5, (it.hi[1] - it.lo[1]) * 0.5],
                round: false,
                margin: it.margin,
                weight: it.weight,
                hardness: state.placed.get(&state.object_layers[k]).map_or(0.5, |p| p.hardness),
                outline: state.outlines.get(k).cloned().flatten(),
            })
            .collect();
        // 箱の順は毎コマ同じに(当たりの組が箱の順で決まる)。
        let mut rooms: Vec<Room> = rooms.into_values().collect();
        rooms.sort_by_key(|r| r.group);
        // 集まる: 終わりは構図。comp の最後まで前向きに解いて焼き、最後から逆に読む。
        let last = state.last_frame;
        state.physics.solve(&rooms, &bodies, frame, fps, gathering.then_some(last));
    }

    /// 集めた物を GPU で効果の順に解き、world のずれにして描く側へ渡す。ブロックが無ければ外す。
    pub(super) fn run_blocks(&mut self, t: RationalTime, _fps: f64) {
        let state = &mut self.blocks;
        if state.objects.is_empty() {
            self.compositor.motion = None;
            return;
        }
        self.solve_physics(t);
        let state = &mut self.blocks;
        let seed: Vec<crate::render::compositor::effects::block_program::BlockOffset> = state.object_layers.iter()
            .map(|layer| match state.physics.offset(*layer) {
                Some((translate, rotate)) => crate::render::compositor::effects::block_program::BlockOffset { translate, rotate, ..Default::default() },
                None => crate::render::compositor::effects::block_program::BlockOffset::default(),
            })
            .collect();
        let ctx = &self.compositor.ctx;
        // 近くの物の升目は、ブロックが宣言した届く距離(`REACH` の欄)の一番大きい値だけ広げる。
        let reach = state.batches.iter().filter_map(|batch| {
            let d = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin)?;
            let name = d.manifest.reach.as_ref()?;
            d.manifest.param_inputs().position(|p| &p.name == name).and_then(|i| batch.params.get(i).copied())
        }).fold(0.0f32, f32::max);
        let world = state.world.get_or_insert_with(|| BlockWorld::new(&ctx.device));
        world.begin_from(&ctx.device, &ctx.queue, &state.objects, reach, &seed);
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-blocks") });
        let stages = state.batches.iter().map(|b| b.stage + 1).max().unwrap_or(0);
        for stage in 0..stages {
            for batch in state.batches.iter().filter(|b| b.stage == stage) {
                let Some(definition) = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin) else { continue };
                let (source, program) = state.programs.entry(batch.plugin.clone())
                    .or_insert_with(|| (definition.vertex_text.clone(), BlockProgram::new(&ctx.device, &batch.plugin, &definition.vertex_text, definition.manifest.rounds)));
                if *source != definition.vertex_text {
                    *source = definition.vertex_text.clone();
                    *program = BlockProgram::new(&ctx.device, &batch.plugin, source, definition.manifest.rounds);
                    // 前の道の bind group は誰も呼ばない。世界に溜めない。
                    world.forget_all();
                }
                program.record_from(&ctx.device, &ctx.queue, &mut encoder, world, t.as_seconds_f64() as f32, &batch.members, &batch.params, batch.source);
            }
        }
        let index_of = |id: LayerId| state.object_layers.iter().position(|l| *l == id).map(|k| k as u32);
        let pairs: Vec<(u32, u32)> = state.object_layers.iter().enumerate()
            .filter_map(|(k, id)| state.follows.get(id).and_then(|target| index_of(*target)).map(|target| (k as u32, target)))
            .collect();
        state.follow_pass.get_or_insert_with(|| FollowPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &pairs);
        let links: Vec<(u32, u32)> = state.connectors.iter().map(|(_, a, b)| (*a, *b)).collect();
        let motion = re_renderer::MotionBuffer::new(ctx, (state.objects.len() + 2 * links.len()) as u64);
        state.world_pass.get_or_insert_with(|| WorldPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &state.bases, &links, motion.buffer());
        let frame = (t.as_seconds_f64() * state.fps).round() as i64;
        state.rope_pass.get_or_insert_with(|| crate::render::compositor::effects::block_program::RopePass::new(&ctx.device))
            .record(&ctx.device, &ctx.queue, &mut encoder, world, &state.ropes, motion.buffer(), frame, state.fps as f32);
        ctx.queue.submit([encoder.finish()]);
        self.compositor.motion = Some(motion);
    }
}

#[cfg(test)]
mod tests;
