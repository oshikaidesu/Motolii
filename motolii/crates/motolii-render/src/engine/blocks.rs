//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で効果の順に解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

use std::collections::HashMap;

use crate::doc::core::CompSpec;
use crate::doc::store::{LayerId, PropertyId, RationalTime, ResolvedLayer, StoreView, Value};
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
}

/// 書類から先に読む、物ごとの住む箱と箱。
#[derive(Clone, Copy)]
struct Placed {
    room: [f32; 4],
    radius: f32,
    own: [f32; 4],
    group: u32,
    weight: f32,
    /// 書類の間合い(CSS の margin)。物同士はこれだけ空けて当たる。
    margin: f32,
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
    /// 壁を持つ住む箱の組(`Overflow = Bounce`)。中の物は最後に箱の中へ折り返される。
    wall_rooms: std::collections::HashSet<u32>,
    /// 場の元(物の番号)と、その効果の順・欄。動く相手は物が揃ってから決める。
    fields: Vec<(usize, String, Vec<f32>, u32)>,
    objects: Vec<BlockItem>,
    bases: Vec<([f32; 3], [f32; 3])>,
    batches: Vec<BlockBatch>,
}

impl Engine {
    /// ブロックを持つ層の住む箱(書類の親の Group の箱、無ければ comp の枠)と、物の箱・組・譲る比を読む。
    pub(super) fn prepare_blocks(&mut self, view: &StoreView<'_>, comp: CompSpec, t: RationalTime, resolved: &[ResolvedLayer]) -> Result<(), EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let blocks: Vec<String> = self.compositor.catalog.definitions.iter().filter(|d| d.manifest.stage == IsfStage::Block).map(|d| d.plugin_id().to_owned()).collect();
        let field_blocks: Vec<String> = self.compositor.catalog.definitions.iter()
            .filter(|d| d.manifest.stage == IsfStage::Block && d.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room)
            .map(|d| d.plugin_id().to_owned()).collect();
        let state = &mut self.blocks;
        state.placed.clear();
        state.follows.clear();
        state.object_layers.clear();
        state.field_rooms.clear();
        state.wall_rooms.clear();
        state.fields.clear();
        state.objects.clear();
        state.bases.clear();
        state.batches.clear();
        if blocks.is_empty() {
            return Ok(());
        }
        // 場が立っている住む箱を先に見る: その箱に居る物は、効果を持たなくても場に動かされる。
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost && l.effects.iter().any(|e| field_blocks.contains(&e.plugin_id))) {
            let parent = view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent;
            state.field_rooms.insert(parent.map_or(0, |p| p.0 as u32));
        }
        let field_rooms = state.field_rooms.clone();
        let in_field_room = |view: &StoreView<'_>, id: LayerId| -> Result<bool, EngineError> {
            let parent = view.attrs(id).map_err(store)?.unwrap_or_default().parent;
            Ok(field_rooms.contains(&parent.map_or(0, |p| p.0 as u32)))
        };
        let mut wanted = Vec::new();
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if layer.effects.iter().any(|e| blocks.contains(&e.plugin_id)) || in_field_room(view, layer.id)? {
                wanted.push(layer.id);
            }
        }
        let state = &mut self.blocks;
        for layer in resolved.iter().filter(|l| wanted.contains(&l.id)) {
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
            // 壁は箱の持ち物(`Overflow = Bounce`)。GPU はその法の速い解き手で、物ごとの効果は要らない(提案 2026-09-16)。
            if let Some(parent) = parent {
                let wall = match view.value_at(parent, &PropertyId::new(crate::doc::store::layout::OVERFLOW).map_err(store)?, t).map_err(store)? {
                    Some(Value::Enum(v)) => v == 2,
                    Some(Value::F64(v)) => v.round() as i64 == 2,
                    _ => false,
                };
                if wall {
                    state.wall_rooms.insert(parent.0 as u32);
                }
            }
            let margin = match view.value_at(layer.id, &PropertyId::new(crate::doc::store::layout::MARGIN).map_err(store)?, t).map_err(store)? {
                Some(Value::F64(v)) => v.max(0.0) as f32,
                _ => 0.0,
            };
            state.placed.insert(layer.id, Placed { room: room.0, radius: room.1, own, group: parent.map_or(0, |p| p.0 as u32), weight, margin });
        }
        // 付いて置く札の相手がブロックで動くなら、札も物として並べて付いて行かせる(CSS の transform を読まない anchor() とは違う、利用者 2026-09-15「付いていく方が自然」)。
        let anchor_row = PropertyId::new(crate::doc::store::layout::POSITION_ANCHOR).map_err(store)?;
        let area_row = PropertyId::new(crate::doc::store::layout::POSITION_AREA).map_err(store)?;
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
            if !matches!(view.value_at(layer.id, &area_row, t).map_err(store)?, Some(Value::Enum(a)) if a > 0) {
                continue;
            }
            let target = match view.value_at(layer.id, &anchor_row, t).map_err(store)? {
                Some(Value::LayerId(id)) if id != 0 => LayerId(id),
                Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
                _ => continue,
            };
            if state.placed.contains_key(&target) {
                state.follows.insert(layer.id, target);
                if !state.placed.contains_key(&layer.id) {
                    let own = view.layer_box(layer.id, t).map_err(store)?.unwrap_or([0.0; 4]);
                    state.placed.insert(layer.id, Placed { room: [0.0; 4], radius: 0.0, own, group: u32::MAX, weight: 0.0, margin: 0.0 });
                }
            }
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
        let chain: Vec<(String, Vec<f32>, bool)> = layer.effects.iter().filter_map(|e| {
            let d = self.compositor.catalog.definitions.iter().find(|d| d.manifest.stage == IsfStage::Block && d.plugin_id() == e.plugin_id)?;
            let params = d.manifest.param_inputs().map(|input| {
                e.params.iter().find(|(n, _)| n == &input.name).and_then(|(_, v)| match v {
                    Value::F64(v) => Some(*v as f32),
                    Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                    _ => None,
                }).unwrap_or(input.default[0])
            }).collect();
            Some((e.plugin_id.clone(), params, d.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room))
        }).collect();
        if chain.is_empty() && !self.blocks.follows.contains_key(&layer.id) && !self.blocks.field_rooms.contains(&placed.group) {
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
            margin: placed.margin,
            weight: placed.weight,
        });
        state.bases.push((world(glam::Vec2::X), world(glam::Vec2::Y)));
        state.object_layers.push(layer.id);
        for (stage, (plugin, params, is_field)) in chain.into_iter().enumerate() {
            if is_field {
                // 場は元。動く相手(同じ住む箱の他の全員)は、物が全部揃ってから決める。
                state.fields.push((stage, plugin, params, k));
                continue;
            }
            match state.batches.iter_mut().find(|b| b.stage == stage && b.plugin == plugin && b.params == params && b.source == u32::MAX) {
                Some(batch) => batch.members.push(k),
                None => state.batches.push(BlockBatch { stage, plugin, params, members: vec![k], source: u32::MAX }),
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
        // 場の相手: 元と同じ住む箱に居る、元でない物 全員。
        for (stage, plugin, params, source) in std::mem::take(&mut state.fields) {
            let group = state.objects[source as usize].group;
            let members: Vec<u32> = state.objects.iter().enumerate()
                .filter(|(k, it)| *k as u32 != source && it.group == group)
                .map(|(k, _)| k as u32).collect();
            if members.is_empty() {
                continue;
            }
            state.batches.push(BlockBatch { stage, plugin, params, members, source });
        }
        // 物同士の当たりは既定(利用者 2026-09-16「お互いの箱にぶつかるとかの方がよく使う」)。場の立つ箱では、
        // 場の後に、重なった物を書類の間合い(Margin)だけ空くまで押し合う。
        let stage = state.batches.iter().map(|b| b.stage + 1).max().unwrap_or(0);
        let touching: Vec<u32> = state.objects.iter().enumerate()
            .filter(|(_, it)| state.field_rooms.contains(&it.group))
            .map(|(k, _)| k as u32).collect();
        if !touching.is_empty() {
            state.batches.push(BlockBatch { stage, plugin: "motolii.push_apart".into(), params: vec![0.0], members: touching, source: u32::MAX });
        }
        // 壁は箱が頼んだ時だけ(`Overflow = Bounce`)。既定ではない。
        if !state.wall_rooms.is_empty() {
            let members: Vec<u32> = state.objects.iter().enumerate()
                .filter(|(_, it)| state.wall_rooms.contains(&it.group))
                .map(|(k, _)| k as u32).collect();
            if !members.is_empty() {
                state.batches.push(BlockBatch { stage: stage + 1, plugin: "motolii.bounce_block".into(), params: vec![1.0], members, source: u32::MAX });
            }
        }
        let ctx = &self.compositor.ctx;
        // 近くの物の升目は、ブロックが宣言した届く距離(`REACH` の欄)の一番大きい値だけ広げる。
        let reach = state.batches.iter().filter_map(|batch| {
            let d = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin)?;
            let name = d.manifest.reach.as_ref()?;
            d.manifest.param_inputs().position(|p| &p.name == name).and_then(|i| batch.params.get(i).copied())
        }).fold(0.0f32, f32::max);
        let world = state.world.get_or_insert_with(|| BlockWorld::new(&ctx.device));
        world.begin(&ctx.device, &ctx.queue, &state.objects, reach);
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
                program.record_from(&ctx.device, &ctx.queue, &mut encoder, world, t.as_seconds_f64() as f32, &batch.members, &batch.params, batch.source);
            }
        }
        let index_of = |id: LayerId| state.object_layers.iter().position(|l| *l == id).map(|k| k as u32);
        let pairs: Vec<(u32, u32)> = state.object_layers.iter().enumerate()
            .filter_map(|(k, id)| state.follows.get(id).and_then(|target| index_of(*target)).map(|target| (k as u32, target)))
            .collect();
        state.follow_pass.get_or_insert_with(|| FollowPass::new(&ctx.device)).record(&ctx.device, &ctx.queue, &mut encoder, world, &pairs);
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

    /// 場(`SCOPE: room`)は、掛かった層でなく**同じ住む箱に居る他の全員**を動かす。相手は効果を 1 枚も持たない。
    #[test]
    fn a_field_moves_everyone_in_its_room_who_carries_no_effect() {
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 60, background: [0.0; 4] })).unwrap();
        let (group, ball, field) = (LayerId(1), LayerId(2), LayerId(3));
        let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
        let square = |size: f32, fill: Rgb| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size as f64, y: size as f64 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(fill), ..Default::default() }) });
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 60) } },
            Intent::SetAttrs { layer: group, patch: two_d.clone() },
            Intent::AddLayer(ball),
            Intent::SetMeta { layer: ball, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 60) } },
            Intent::SetAttrs { layer: ball, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
            Intent::SetShapes { layer: ball, shapes: vec![square(16.0, Rgb { r: 1.0, g: 1.0, b: 1.0 })] },
            Intent::AddLayer(field),
            Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 60) } },
            Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
            Intent::SetShapes { layer: field, shapes: vec![square(4.0, Rgb { r: 1.0, g: 0.0, b: 0.0 })] },
        ]).unwrap();
        let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
        put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
        put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
        put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
        put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
        for layer in [ball, field] {
            put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
        }
        put(&mut doc, ball, property::POSITION, Value::Vec2([30.0, 15.0]));
        put(&mut doc, field, property::POSITION, Value::Vec2([100.0, 15.0]));
        // 一様(Spread 1)の場を下(+90°)へ、強さ 40: 1 秒で 0.5 * 40 * 1² = 20px 下がる。
        doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
        for (name, value) in [("spread", 1.0), ("turn", 0.0), ("angle", 90.0), ("strength", 40.0), ("reach", 0.0)] {
            doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
        }
        let mut engine = Engine::new().unwrap();
        // 白い四角(場の元は赤)の真ん中。
        let centre = |pixels: &[u8]| {
            let hits: Vec<(f32, f32)> = pixels.chunks_exact(4).enumerate()
                .filter(|(_, c)| c[3] > 128 && c[2] > 128)
                .map(|(i, _)| ((i as u32 % W) as f32, (i as u32 / W) as f32)).collect();
            let n = hits.len().max(1) as f32;
            (hits.len(), hits.iter().map(|p| p.0).sum::<f32>() / n, hits.iter().map(|p| p.1).sum::<f32>() / n)
        };
        let (n0, x0, y0) = centre(&engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap());
        let (n1, x1, y1) = centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(30, fps).unwrap()).unwrap());
        assert!(n0 > 100 && n1 > 100, "白い四角が両方のコマに在る ({n0} / {n1} px)");
        assert!((x1 - x0).abs() < 1.0, "横には動かない ({x0} → {x1})");
        assert!((y1 - y0 - 20.0).abs() < 1.5, "1 秒で 20px 下がる ({y0} → {y1})");
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
        world.begin(device, queue, &items, 0.0);
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
        world.begin(device, queue, &items, 0.0);
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

    /// ブロックごとに自分の欄を読む(1 コマに送る前の書き込みで、後のブロックの欄が前のブロックに混ざらない)。
    #[test]
    fn each_block_reads_its_own_params() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let bounce = program_for(device, include_str!("../../vism/bounce.wgsl"));
        let push = program_for(device, include_str!("../../vism/push_apart.wgsl"));
        let items = [BlockItem { lo: [300.0, 20.0], hi: [310.0, 30.0], room_lo: [0.0; 2], room_size: [200.0, 100.0], radius: 0.0, group: 1, margin: 0.0, weight: 1.0 }];
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        bounce.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.0]);
        push.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[9.0]);
        let state = read_state(device, queue, &world, encoder);
        assert_eq!(state[0].translate, [0.0, 0.0], "Bounce at strength 0 does not fold, whatever the next block's margin is");
    }

    /// 調べる口: `MOTOLII_BLOCK_DOC` の書類の `MOTOLII_BLOCK_FRAME` コマの物の箱とブロックの結果を出す。
    #[test]
    #[ignore]
    fn dump_blocks() {
        use crate::render::compositor::effects::block_program::read_state;
        let doc = Document::load(std::env::var("MOTOLII_BLOCK_DOC").unwrap()).unwrap();
        let frame: i64 = std::env::var("MOTOLII_BLOCK_FRAME").unwrap().parse().unwrap();
        let view = doc.view();
        let fps = view.composition().unwrap().unwrap().fps;
        let t = RationalTime::try_from_frame(frame, fps).unwrap();
        let mut engine = Engine::new().unwrap();
        let scope = engine.gpu_device().push_error_scope(wgpu::ErrorFilter::Validation);
        engine.render_frame(&view, t).unwrap();
        eprintln!("gpu validation: {:?}", pollster::block_on(scope.pop()));
        eprintln!("layer failures: {:?}", engine.layer_failures());
        for (k, o) in engine.blocks.objects.iter().enumerate() { eprintln!("object {k}: {o:?}"); }
        for b in &engine.blocks.batches { eprintln!("batch stage {} {} {:?} {:?}", b.stage, b.plugin, b.params, b.members); }
        let Some(world) = engine.blocks.world.as_ref() else { return };
        let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for (k, o) in read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder).iter().enumerate() { eprintln!("state {k}: {o:?}"); }
    }

    /// 付いて置く札は、相手がブロックで動いた分だけ一緒に動く(GPU の中で、読み戻さずに)。
    #[test]
    fn an_anchored_label_follows_what_a_block_moved() {
        use crate::render::compositor::effects::block_program::read_state;
        let mut doc = scene(true);
        let label = LayerId(3);
        doc.apply_all([
            Intent::AddLayer(label),
            Intent::SetMeta { layer: label, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer: label, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
            Intent::SetShapes { layer: label, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 6.0, y: 4.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), ..Default::default() }) })] },
            Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_ANCHOR).unwrap(), value: Value::LayerId(2) },
            Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_AREA).unwrap(), value: Value::Enum(2) },
        ]).unwrap();
        let mut engine = Engine::new().unwrap();
        let t = RationalTime::try_from_frame(45, Fps::try_new(30, 1).unwrap()).unwrap();
        engine.render_frame(&doc.view(), t).unwrap();
        let layers = engine.blocks.object_layers.clone();
        let world = engine.blocks.world.as_ref().unwrap();
        let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let state = read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder);
        let at = |id: u64| state[layers.iter().position(|l| l.0 == id).expect("an object")].translate;
        assert!(at(2) != [0.0, 0.0], "the square was folded by Bounce");
        assert_eq!(at(3), at(2), "its label moved with it");
    }

    /// Wave: 時刻と物の順で縦の正弦波。Wavelength 個離れた物は同じ高さ、半分なら逆。
    #[test]
    fn the_wave_block_travels_along_things_in_order() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let wave = program_for(device, include_str!("../../vism/wave.wgsl"));
        let items: Vec<BlockItem> = (0..8).map(|i| BlockItem { lo: [i as f32 * 20.0, 0.0], hi: [i as f32 * 20.0 + 10.0, 10.0], weight: 1.0, ..Default::default() }).collect();
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        wave.record(device, queue, &mut encoder, &mut world, 0.125, &(0..8).collect::<Vec<u32>>(), &[10.0, 2.0, 4.0]);
        let y: Vec<f32> = read_state(device, queue, &world, encoder).iter().map(|o| o.translate[1]).collect();
        let expect = |k: f32| 10.0 * (std::f32::consts::TAU * (2.0 * 0.125 - k / 4.0)).sin();
        for (k, v) in y.iter().enumerate() {
            assert!((v - expect(k as f32)).abs() < 1e-3, "object {k}: {v} vs {}", expect(k as f32));
        }
        assert!((y[0] - y[4]).abs() < 1e-3 && (y[0] + y[2]).abs() < 1e-3, "a wavelength apart: same; half: opposite {y:?}");
    }

    /// 近くの物だけ見る押し合い(升目)は、全組を見る同じ手順と同じ結果になる(散らばった 2000 個、4 組)。
    #[test]
    fn push_apart_over_neighbours_matches_all_pairs_at_scale() {
        use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
        let mut rng = 3u32;
        let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
        let items: Vec<BlockItem> = (0..2000).map(|k| {
            let (x, y, w, h) = (next() * 3000.0, next() * 3000.0, 6.0 + next() * 14.0, 6.0 + next() * 14.0);
            BlockItem { lo: [x, y], hi: [x + w, y + h], room_size: [3000.0, 3000.0], group: k % 4, weight: 0.5 + next(), ..Default::default() }
        }).collect();
        let margin = 3.0f32;
        // CPU: 同じ手順を全組で(32 回、全員を同時に測って動かす)。
        let mut lo: Vec<[f32; 2]> = items.iter().map(|i| i.lo).collect();
        let mut hi: Vec<[f32; 2]> = items.iter().map(|i| i.hi).collect();
        for _ in 0..32 {
            let mut step = vec![[0.0f32; 2]; items.len()];
            for k in 0..items.len() {
                for j in 0..items.len() {
                    if j == k || items[j].group != items[k].group { continue; }
                    let (alo, ahi) = ([lo[k][0] - margin, lo[k][1] - margin], [hi[k][0] + margin, hi[k][1] + margin]);
                    let (blo, bhi) = ([lo[j][0] - margin, lo[j][1] - margin], [hi[j][0] + margin, hi[j][1] + margin]);
                    let gap = [(blo[0] + bhi[0]) * 0.5 - (alo[0] + ahi[0]) * 0.5, (blo[1] + bhi[1]) * 0.5 - (alo[1] + ahi[1]) * 0.5];
                    let len = (gap[0] * gap[0] + gap[1] * gap[1]).sqrt();
                    let dir = if len > 1e-4 { [gap[0] / len, gap[1] / len] } else if k < j { [1.0, 0.0] } else { [-1.0, 0.0] };
                    let half = [((ahi[0] - alo[0]) + (bhi[0] - blo[0])) * 0.5, ((ahi[1] - alo[1]) + (bhi[1] - blo[1])) * 0.5];
                    if half[0] - gap[0].abs() <= 0.0 || half[1] - gap[1].abs() <= 0.0 { continue; }
                    let need = |a: usize| if dir[a].abs() >= 1e-6 { ((half[a] - gap[a].abs()) / dir[a].abs()).max(0.0) } else { 1e30 };
                    let depth = need(0).min(need(1));
                    if depth <= 0.0 || depth >= 1e29 { continue; }
                    let w = items[k].weight / (items[k].weight + items[j].weight);
                    step[k][0] -= dir[0] * depth * w * 0.5;
                    step[k][1] -= dir[1] * depth * w * 0.5;
                }
            }
            for k in 0..items.len() {
                for a in 0..2 { lo[k][a] += step[k][a]; hi[k][a] += step[k][a]; }
            }
        }
        let engine = Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let push = program_for(device, include_str!("../../vism/push_apart.wgsl"));
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items, 0.0);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        push.record(device, queue, &mut encoder, &mut world, 0.0, &(0..items.len() as u32).collect::<Vec<u32>>(), &[margin]);
        let gpu = read_state(device, queue, &world, encoder);
        let mut moved = 0;
        assert_eq!(push_reach(), Some("margin".to_owned()), "Push Apart declares its margin as its reach");
        for k in 0..items.len() {
            let cpu = [lo[k][0] - items[k].lo[0], lo[k][1] - items[k].lo[1]];
            if cpu != [0.0, 0.0] { moved += 1; }
            let g = gpu[k].translate;
            assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {k}: gpu {g:?} cpu {cpu:?}");
        }
        assert!(moved > 20, "some of them overlapped and were pushed: {moved}");
    }

    fn push_reach() -> Option<String> {
        crate::render::compositor::effects::isf::parse_isf_source(include_str!("../../vism/push_apart.wgsl")).unwrap().0.reach
    }

    /// Margin が箱よりずっと大きくても、届く距離の分だけ升目を広げるので相手を取りこぼさない。
    #[test]
    fn a_wide_margin_still_finds_its_neighbours() {
        use crate::render::compositor::effects::block_program::{neighbors, BlockItem};
        let items = [
            BlockItem { lo: [0.0, 0.0], hi: [4.0, 4.0], weight: 1.0, ..Default::default() },
            BlockItem { lo: [60.0, 0.0], hi: [64.0, 4.0], weight: 1.0, ..Default::default() },
        ];
        let (_, blind) = neighbors(&items, 0.0);
        assert!(blind.is_empty(), "without the reach, 60 px apart is out of a 8 px cell's 3×3");
        let (starts, list) = neighbors(&items, 40.0);
        assert_eq!((starts, list), (vec![0, 1, 2], vec![1, 0]), "with a 40 px margin they see each other");
    }
}
