//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で効果の順に解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::doc::store::{LayerId, PropertyId, RationalTime, ResolvedLayer, StoreView, Value};
use crate::render::compositor::effects::block_program::{BlockItem, BlockProgram, BlockWorld, WorldPass};
use crate::render::compositor::effects::isf::IsfStage;
use crate::render::compositor::Layer;
use crate::render::engine::{Engine, EngineError};

/// 効果の列の何番目のブロックか・どのブロックか・欄の値が同じ物を、1 回の計算に束ねる。
pub(crate) struct BlockBatch {
    stage: usize,
    plugin: String,
    params: Vec<f32>,
    members: Vec<u32>,
}

/// 書類から先に読む、物ごとの住む箱と箱。
#[derive(Clone, Copy)]
struct Placed {
    room: [f32; 4],
    radius: f32,
    own: [f32; 4],
    group: u32,
    weight: f32,
}

#[derive(Default)]
pub(crate) struct BlockState {
    /// ブロックごとの GPU の道と、組んだ時の本文(棚が読み直されたら組み直す)。
    programs: HashMap<String, (String, BlockProgram)>,
    world: Option<BlockWorld>,
    world_pass: Option<WorldPass>,
    placed: HashMap<LayerId, Placed>,
    objects: Vec<BlockItem>,
    bases: Vec<([f32; 3], [f32; 3])>,
    batches: Vec<BlockBatch>,
}

impl Engine {
    /// ブロックを持つ層の住む箱(書類の親の Group の箱、無ければ comp の枠)と、物の箱・組・譲る比を読む。
    pub(super) fn prepare_blocks(&mut self, view: &StoreView<'_>, comp: CompSpec, t: RationalTime, resolved: &[ResolvedLayer]) -> Result<(), EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let blocks: Vec<String> = self.compositor.catalog.definitions.iter().filter(|d| d.manifest.stage == IsfStage::Block).map(|d| d.plugin_id().to_owned()).collect();
        let state = &mut self.blocks;
        state.placed.clear();
        state.objects.clear();
        state.bases.clear();
        state.batches.clear();
        if blocks.is_empty() {
            return Ok(());
        }
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost && l.effects.iter().any(|e| blocks.contains(&e.plugin_id))) {
            let frame = ([0.0, 0.0, comp.width as f32, comp.height as f32], 0.0);
            let parent = view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent;
            let room = match parent {
                Some(parent) => match (resolved.iter().find(|l| l.id == parent && l.copy == 0 && !l.ghost), view.layer_box(parent, t).map_err(store)?) {
                    (Some(owner), Some(b)) => {
                        let m = owner.placement.transform;
                        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| m.transform_point2(glam::Vec2::from(c)));
                        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
                        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
                        let radius = match view.value_at(parent, &PropertyId::new(crate::doc::store::layout::BORDER_RADIUS).map_err(store)?, t).map_err(store)? {
                            Some(Value::F64(r)) => r.max(0.0) as f32 * m.matrix2.x_axis.length(),
                            _ => 0.0,
                        };
                        ([lo.x, lo.y, hi.x, hi.y], radius)
                    }
                    _ => frame,
                },
                None => frame,
            };
            let Some(own) = view.layer_box(layer.id, t).map_err(store)? else { continue };
            let weight = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::FLEX_SHRINK).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.max(0.0) as f32,
                _ => 1.0,
            };
            state.placed.insert(layer.id, Placed { room: room.0, radius: room.1, own, group: parent.map_or(0, |p| p.0 as u32), weight });
        }
        Ok(())
    }

    /// 組んだ 1 枚にブロックが掛かっていれば、描く時と同じ置き方の箱を物として並べ、motion の番号を最後の欄に入れる。
    pub(super) fn attach_block(&mut self, layer: &ResolvedLayer, built: &mut Layer, comp: CompSpec) {
        let Some(&placed) = self.blocks.placed.get(&layer.id) else { return };
        let size = glam::Vec2::from(built.size);
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        let chain: Vec<(String, Vec<f32>)> = layer.effects.iter().filter_map(|e| {
            let d = self.compositor.catalog.definitions.iter().find(|d| d.manifest.stage == IsfStage::Block && d.plugin_id() == e.plugin_id)?;
            let params = d.manifest.param_inputs().map(|input| {
                e.params.iter().find(|(n, _)| n == &input.name).and_then(|(_, v)| match v {
                    Value::F64(v) => Some(*v as f32),
                    Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                    _ => None,
                }).unwrap_or(input.default[0])
            }).collect();
            Some((e.plugin_id.clone(), params))
        }).collect();
        if chain.is_empty() {
            return;
        }
        let m = built.placement.transform;
        let own = placed.own;
        let corners = [[own[0], own[1]], [own[2], own[1]], [own[0], own[3]], [own[2], own[3]]].map(|c| m.transform_point2(glam::Vec2::from(c)));
        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
        // comp の 1px が world でどちら向きか: 素材の辺の world の長さ ÷ 素材の px、を comp → 素材の逆写しに掛ける。
        let (_, u, v) = crate::render::compositor::projected_placement_corners(comp, built.projection_camera, built.projection, built.placement, glam::Vec2::ZERO, size);
        let (per_x, per_y) = (u / size.x, v / size.y);
        let inverse = if m.matrix2.determinant().abs() > 1e-12 { m.matrix2.inverse() } else { glam::Mat2::IDENTITY };
        let world = |comp_step: glam::Vec2| { let material = inverse * comp_step; (per_x * material.x + per_y * material.y).to_array() };
        let state = &mut self.blocks;
        let k = state.objects.len() as u32;
        built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (k + 1) as f32;
        state.objects.push(BlockItem {
            lo: lo.to_array(),
            hi: hi.to_array(),
            room_lo: [placed.room[0], placed.room[1]],
            room_size: [placed.room[2] - placed.room[0], placed.room[3] - placed.room[1]],
            radius: placed.radius,
            group: placed.group,
            margin: 0.0,
            weight: placed.weight,
        });
        state.bases.push((world(glam::Vec2::X), world(glam::Vec2::Y)));
        for (stage, (plugin, params)) in chain.into_iter().enumerate() {
            match state.batches.iter_mut().find(|b| b.stage == stage && b.plugin == plugin && b.params == params) {
                Some(batch) => batch.members.push(k),
                None => state.batches.push(BlockBatch { stage, plugin, params, members: vec![k] }),
            }
        }
    }

    /// 集めた物を GPU で効果の順に解き、world のずれにして描く側へ渡す。ブロックが無ければ外す。
    pub(super) fn run_blocks(&mut self, t: RationalTime) {
        let state = &mut self.blocks;
        if state.objects.is_empty() {
            self.compositor.motion = None;
            return;
        }
        let ctx = &self.compositor.ctx;
        let world = state.world.get_or_insert_with(|| BlockWorld::new(&ctx.device));
        world.begin(&ctx.device, &ctx.queue, &state.objects);
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
                }
                program.record(&ctx.device, &ctx.queue, &mut encoder, world, t.as_seconds_f64() as f32, &batch.members, &batch.params);
            }
        }
        let motion = re_renderer::MotionBuffer::new(ctx, state.objects.len() as u64);
        state.world_pass.get_or_insert_with(|| WorldPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &state.bases, motion.buffer());
        ctx.queue.submit([encoder.finish()]);
        self.compositor.motion = Some(motion);
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::eval::Keyframe;
    use crate::doc::store::{layout, property, Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerProjection, LayerSource, LayerTiming, PathSource, PropertyId, RationalTime, Shape, ShapeNode, Value};
    use crate::doc::vector::{Brush, Fill, Point, Rgb};
    use crate::render::engine::Engine;

    const W: u32 = 160;
    const H: u32 = 100;

    /// 灰色でない住む箱(Display の Group、角丸 0)の中を、白い四角が右下へまっすぐ漂う。`gpu` なら子に Bounce のブロック、
    /// そうでなければ親の Overflow = Bounce(書類の CPU の法)。
    fn scene(gpu: bool) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
        let (group, child) = (LayerId(1), LayerId(2));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(child),
            Intent::SetMeta { layer: child, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: child, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
        put(&mut doc, child, layout::POSITION_TYPE, Value::Enum(1));
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([30.0, 20.0]), interp: Interp::Linear, spatial: None });
        track.insert(Keyframe { t: RationalTime::try_new(3, 1).unwrap(), value: Value::Vec2([30.0 + 390.0, 20.0 + 240.0]), interp: Interp::Linear, spatial: None });
        doc.apply(Intent::SetTrack { layer: child, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        if gpu {
            doc.apply(Intent::SetEffects { layer: child, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.bounce_block".into() }] }).unwrap();
        } else {
            put(&mut doc, group, layout::OVERFLOW, Value::Enum(2));
        }
        doc
    }

    /// GPU のブロックが描く位置は、書類の CPU の Bounce と同じ(読み戻さずに描く側の頂点で動く)。
    #[test]
    fn the_bounce_block_draws_where_the_cpu_bounce_does() {
        let (cpu, gpu) = (scene(false), scene(true));
        let mut engine = Engine::new().unwrap();
        for frame in [0, 20, 45, 70] {
            let t = RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
            let expected = engine.render_frame(&cpu.view(), t).unwrap();
            let actual = engine.render_frame(&gpu.view(), t).unwrap();
            let lit = |p: &[u8]| p.chunks_exact(4).enumerate().filter(|(_, c)| c[3] > 128).map(|(i, _)| ((i as u32) % W, (i as u32) / W)).collect::<Vec<_>>();
            let (e, a) = (lit(&expected), lit(&actual));
            let centre = |v: &[(u32, u32)]| { let n = v.len().max(1) as f32; (v.iter().map(|p| p.0 as f32).sum::<f32>() / n, v.iter().map(|p| p.1 as f32).sum::<f32>() / n) };
            assert!(!e.is_empty() && (e.len() as i64 - a.len() as i64).abs() < 40, "frame {frame}: the square is drawn once in both ({} vs {} px)", e.len(), a.len());
            let (ce, ca) = (centre(&e), centre(&a));
            assert!((ce.0 - ca.0).abs() < 1.0 && (ce.1 - ca.1).abs() < 1.0, "frame {frame}: cpu centre {ce:?}, gpu centre {ca:?}");
        }
    }

    /// Push Apart を GPU のブロックにしても、書類の間合いの押し合い(CPU、32 回)と同じだけ押す。
    #[test]
    fn the_push_apart_block_pushes_like_the_margin_law() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let place = |margin: bool| {
            let mut doc = Document::new();
            doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
            let spots = [[40.0, 40.0], [52.0, 44.0], [60.0, 30.0], [100.0, 60.0], [104.0, 64.0], [20.0, 80.0]];
            for (i, at) in spots.iter().enumerate() {
                let layer = LayerId(i as u64 + 1);
                doc.apply_all([
                    Intent::AddLayer(layer),
                    Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: i as i16, timing: LayerTiming::place(0, None, 1) } },
                    Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
                    Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 20.0, y: 14.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
                    Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(*at) },
                ]).unwrap();
                if margin {
                    doc.apply(Intent::SetConstant { layer, property: PropertyId::new(layout::MARGIN).unwrap(), value: Value::F64(5.0) }).unwrap();
                }
            }
            doc
        };
        let (plain, pushed) = (place(false), place(true));
        let t = RationalTime::ZERO;
        let frame = pushed.view().layout_frame(t).unwrap();
        let view = plain.view();
        let items: Vec<BlockItem> = (1..=6).map(|i| {
            let id = LayerId(i);
            let b = view.layer_box(id, t).unwrap().unwrap();
            let m = view.local_transform(id, t).unwrap();
            let (lo, hi) = (m.transform_point2(glam::vec2(b[0], b[1])), m.transform_point2(glam::vec2(b[2], b[3])));
            BlockItem { lo: lo.to_array(), hi: hi.to_array(), room_lo: [0.0; 2], room_size: [W as f32, H as f32], radius: 0.0, group: 0, margin: 0.0, weight: 1.0 }
        }).collect();
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let program = program_for(device, include_str!("../../vism/push_apart.wgsl"));
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        program.record(device, queue, &mut encoder, &mut world, 0.0, &[0, 1, 2, 3, 4, 5], &[5.0]);
        let gpu = read_state(device, queue, &world, encoder);
        let mut moved = 0;
        for i in 0..6 {
            let cpu = frame.nudges.get(&LayerId(i as u64 + 1)).copied().unwrap_or([0.0, 0.0]);
            let g = gpu[i].translate;
            if cpu != [0.0, 0.0] { moved += 1; }
            assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {i}: gpu {g:?} cpu {cpu:?}");
        }
        assert!(moved >= 4, "the overlapping ones were pushed");
    }

    /// ブロックは効果の順につながる: 押し合って(Push Apart)から壁で折り返す(Bounce)と、全員が箱の中に収まる。
    #[test]
    fn blocks_chain_in_effect_order() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let push = program_for(device, include_str!("../../vism/push_apart.wgsl"));
        let bounce = program_for(device, include_str!("../../vism/bounce.wgsl"));
        let room = [200.0f32, 120.0];
        let mut items = Vec::new();
        for i in 0..40 {
            let x = 60.0 + (i as f32 * 37.0) % 180.0;
            let y = 20.0 + (i as f32 * 23.0) % 110.0;
            items.push(BlockItem { lo: [x, y], hi: [x + 12.0, y + 12.0], room_lo: [0.0; 2], room_size: room, radius: 0.0, group: 7, margin: 0.0, weight: 1.0 });
        }
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items);
        let members: Vec<u32> = (0..40).collect();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        push.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[2.0]);
        bounce.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[1.0]);
        let state = read_state(device, queue, &world, encoder);
        for (i, (item, o)) in items.iter().zip(&state).enumerate() {
            let (lo, hi) = ([item.lo[0] + o.translate[0], item.lo[1] + o.translate[1]], [item.hi[0] + o.translate[0], item.hi[1] + o.translate[1]]);
            assert!(lo[0] >= -0.01 && lo[1] >= -0.01 && hi[0] <= room[0] + 0.01 && hi[1] <= room[1] + 0.01, "object {i} ends inside the room: {lo:?} {hi:?}");
        }
        let spread = state.iter().filter(|o| o.translate != [0.0, 0.0]).count();
        assert!(spread > 20, "the crowd was pushed and folded: {spread} moved");
    }
}
