//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で効果の順に解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue, SolverPlanValue};
use crate::doc::store::{LayerId, RationalTime, Value};
use crate::render::compositor::effects::block_program::{BlockItem, BlockProgram, BlockWorld, FollowPass, WorldPass};
use crate::render::compositor::effects::isf::IsfStage;
use crate::render::compositor::Layer;
use crate::render::engine::{Engine, EngineError};

use crate::render_lowering::{BlockBatch, Placed};

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
    pub(crate) fn object_count(&self) -> usize {
        self.objects.len()
    }

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

    pub fn block_states(&mut self) -> Vec<[f32; 3]> {
        self.block_state_now().iter().map(|o| [o.translate[0], o.translate[1], o.rotate]).collect()
    }

    /// 計測の口: 物ごとの今のずれを全部(位置 2・回転・大きさ・色 3・不透明)。描く道は読み戻さない。
    pub fn block_offsets(&mut self) -> Vec<[f32; 8]> {
        self.block_state_now().iter().map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale, o.tint[0], o.tint[1], o.tint[2], o.tint[3]]).collect()
    }

    /// Tests and measurement only: the blocks' state as this frame leaves it, now (the frame ends and
    /// the GPU is waited for).
    fn block_state_now(&mut self) -> Vec<crate::render::compositor::effects::block_program::BlockOffset> {
        use crate::render::compositor::effects::block_program::{offsets_from_bytes, OFFSET_BYTES};
        let Some(world) = self.blocks.world.as_ref() else { return Vec::new() };
        let bytes = u64::from(world.count) * OFFSET_BYTES;
        if bytes == 0 { return Vec::new(); }
        let Ok(id) = crate::render::compositor::readback::ask_buffer(&self.compositor.ctx, world.state(), bytes) else { return Vec::new() };
        self.compositor.next_frame();
        if self.compositor.ctx.device.poll(wgpu::PollType::wait_indefinitely()).is_err() { return Vec::new(); }
        crate::render::compositor::readback::take_buffer(&self.compositor.ctx, id).map(|data| offsets_from_bytes(&data)).unwrap_or_default()
    }


    /// Installs the lowered block plan, solves physics and runs the block programs.
    pub(in crate::engine) fn prepare_frame_graph_blocks(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        prep: &super::frame_graph_scene::Preparation,
        t: RationalTime,
        fps: crate::doc::store::Fps,
        prepared: &mut super::frame_graph_scene::GpuSceneValue,
    ) -> Result<(), EngineError> {
        let comp = prep.comp;
        let mut sizes: HashMap<LayerId, [f32; 2]> = HashMap::new();
        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter()) {
            sizes.entry(id).or_insert(layer.layer.size);
        }
        let catalog = self.compositor.catalog.clone();
        let plan = crate::render_lowering::plan_blocks(scene, solver, &catalog, comp, t, fps, &sizes);
        if self.blocks.physics.lies != plan.lies {
            self.blocks.physics = Default::default();
            self.blocks.physics.lies = plan.lies.clone();
        }
        let state = &mut self.blocks;
        state.fps = fps.as_f64();
        state.last_frame = 0;
        state.now = Some(t);
        state.placed = plan.placed;
        state.follows = plan.follows;
        state.object_layers = plan.object_layers;
        state.object_frames = plan.object_frames;
        state.slots = plan.slots;
        state.connectors = plan.connectors;
        state.traces = plan.traces;
        state.ropes = plan.ropes;
        state.field_rooms = plan.field_rooms;
        state.fields = plan.fields;
        state.objects = plan.objects;
        state.bases = plan.bases;
        state.outlines = plan.outlines;
        state.batches = plan.batches;
        if !plan.active {
            self.compositor.motion = None;
            return Ok(());
        }

        self.solve_physics(t);

        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter_mut()) {
            self.attach_block_id(id, &mut layer.layer, comp, prep.seam.camera_relative_world());
        }
        self.run_blocks(t, fps.as_f64());
        Ok(())
    }

    /// `world`: where a 2D/2.5D layer stands in the solver's world (3D placement does not read it).
    pub(in crate::engine) fn attach_block_id(&mut self, layer: LayerId, built: &mut Layer, comp: CompSpec, world: crate::doc::core::ResolvedCamera) {
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
        let (origin, u, v) = crate::render::compositor::projected_placement_corners(comp, world, built.projection, built.placement, glam::Vec2::ZERO, size);
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
        // Four vec4 per entry (motion.wgsl); a rope takes two entries.
        let motion = ctx.gpu_resources.buffers.alloc(&ctx.device, &re_renderer::BufferDesc {
            label: "motion".into(),
            size: (state.objects.len() + 2 * links.len()).max(1) as u64 * 64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        state.world_pass.get_or_insert_with(|| WorldPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &state.bases, &links, &motion);
        let frame = (t.as_seconds_f64() * state.fps).round() as i64;
        state.rope_pass.get_or_insert_with(|| crate::render::compositor::effects::block_program::RopePass::new(&ctx.device))
            .record(&ctx.device, &ctx.queue, &mut encoder, world, &state.ropes, &motion, frame, state.fps as f32);
        // Recorded, not submitted: it goes out with the frame's other work, ahead of every draw.
        self.compositor.ctx.queue_commands([encoder.finish()]);
        self.compositor.motion = Some(motion);
    }
}

#[cfg(test)]
mod tests;
