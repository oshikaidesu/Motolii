//! 箱のブロック(`STAGE: block`)を 1 コマに掛ける: 層を組む時に物の箱を集め、組み終えたら GPU で解いて
//! 描く側の motion の buffer に書く。読み戻さない(利用者 2026-09-15「この天井を作るべきでない」)。

use std::collections::HashMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{LayerId, PropertyId, RationalTime, ResolvedLayer, StoreView, Value};
use crate::render::compositor::effects::block_program::{BlockItem, BlockProgram};
use crate::render::compositor::effects::isf::IsfStage;
use crate::render::compositor::Layer;
use crate::render::engine::{Engine, EngineError};

/// 同じブロックで同じ欄の値の物を 1 回の計算に束ねる。
pub(crate) struct BlockBatch {
    plugin: String,
    params: Vec<f32>,
    items: Vec<BlockItem>,
}

#[derive(Default)]
pub(crate) struct BlockState {
    /// ブロックごとの GPU の道と、組んだ時の本文(棚が読み直されたら組み直す)。
    programs: HashMap<String, (String, BlockProgram)>,
    /// 住む箱(comp の lo / hi、角丸)と、物の箱(素材座標、書類の `layer_box`: 形なら縁のにじみの余白を含まない)。層を組む前に書類から。
    rooms: HashMap<LayerId, ([f32; 4], f32, [f32; 4])>,
    batches: Vec<BlockBatch>,
    count: u32,
}

impl Engine {
    /// ブロックを持つ層の住む箱を、書類の親の Group の箱から求める(親が無ければ comp の枠)。
    pub(super) fn prepare_blocks(&mut self, view: &StoreView<'_>, comp: CompSpec, t: RationalTime, resolved: &[ResolvedLayer]) -> Result<(), EngineError> {
        let store = |e: crate::doc::store::StoreError| EngineError::Store(e.to_string());
        let blocks: Vec<String> = self.compositor.catalog.definitions.iter().filter(|d| d.manifest.stage == IsfStage::Block).map(|d| d.plugin_id().to_owned()).collect();
        let state = &mut self.blocks;
        state.rooms.clear();
        state.batches.clear();
        state.count = 0;
        if blocks.is_empty() {
            return Ok(());
        }
        for layer in resolved.iter().filter(|l| l.copy == 0 && !l.ghost && l.effects.iter().any(|e| blocks.contains(&e.plugin_id))) {
            let frame = ([0.0, 0.0, comp.width as f32, comp.height as f32], 0.0);
            let room = match view.attrs(layer.id).map_err(store)?.unwrap_or_default().parent {
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
            state.rooms.insert(layer.id, (room.0, room.1, own));
        }
        Ok(())
    }

    /// 組んだ 1 枚にブロックが掛かっていれば、描く時と同じ置き方の箱を集め、motion の番号を最後の欄に入れる。
    pub(super) fn attach_block(&mut self, layer: &ResolvedLayer, built: &mut Layer, comp: CompSpec) {
        let Some(&room) = self.blocks.rooms.get(&layer.id) else { return };
        let Some((definition, effect)) = layer.effects.iter().find_map(|e| {
            self.compositor.catalog.definitions.iter().find(|d| d.manifest.stage == IsfStage::Block && d.plugin_id() == e.plugin_id).map(|d| (d, e))
        }) else { return };
        let params: Vec<f32> = definition.manifest.param_inputs().map(|input| {
            effect.params.iter().find(|(n, _)| n == &input.name).and_then(|(_, v)| match v {
                Value::F64(v) => Some(*v as f32),
                Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                _ => None,
            }).unwrap_or(input.default[0])
        }).collect();
        let size = glam::Vec2::from(built.size);
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }
        let m = built.placement.transform;
        let own = room.2;
        let corners = [[own[0], own[1]], [own[2], own[1]], [own[0], own[3]], [own[2], own[3]]].map(|c| m.transform_point2(glam::Vec2::from(c)));
        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
        // comp の 1px が world でどちら向きか: 素材の辺の world の長さ ÷ 素材の px、を comp → 素材の逆写しに掛ける。
        let (_, u, v) = crate::render::compositor::projected_placement_corners(comp, built.projection_camera, built.projection, built.placement, glam::Vec2::ZERO, size);
        let (per_x, per_y) = (u / size.x, v / size.y);
        let inverse = if m.matrix2.determinant().abs() > 1e-12 { m.matrix2.inverse() } else { glam::Mat2::IDENTITY };
        let world = |comp_step: glam::Vec2| { let material = inverse * comp_step; per_x * material.x + per_y * material.y };
        let state = &mut self.blocks;
        let index = state.count;
        state.count += 1;
        built.shading.params[crate::render::compositor::effects::surface_program::PARAM_SLOTS - 1] = (index + 1) as f32;
        let item = BlockItem {
            lo: lo.to_array(),
            hi: hi.to_array(),
            room_lo: [room.0[0], room.0[1]],
            room_size: [room.0[2] - room.0[0], room.0[3] - room.0[1]],
            radius: room.1,
            index,
            world_u: world(glam::Vec2::X).to_array(),
            world_v: world(glam::Vec2::Y).to_array(),
        };
        match state.batches.iter_mut().find(|b| b.plugin == effect.plugin_id && b.params == params) {
            Some(batch) => batch.items.push(item),
            None => state.batches.push(BlockBatch { plugin: effect.plugin_id.clone(), params, items: vec![item] }),
        }
    }

    /// 集めた箱を GPU で解いて motion の buffer に書き、描く側へ渡す。ブロックが無ければ外す。
    pub(super) fn run_blocks(&mut self, t: RationalTime) {
        let state = &mut self.blocks;
        if state.count == 0 {
            self.compositor.motion = None;
            return;
        }
        let ctx = &self.compositor.ctx;
        let motion = re_renderer::MotionBuffer::new(ctx, u64::from(state.count));
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-blocks") });
        for batch in &state.batches {
            let Some(definition) = self.compositor.catalog.definitions.iter().find(|d| d.plugin_id() == batch.plugin) else { continue };
            let entry = state.programs.entry(batch.plugin.clone());
            let (source, program) = entry.or_insert_with(|| (definition.vertex_text.clone(), BlockProgram::new(&ctx.device, &batch.plugin, &definition.vertex_text)));
            if *source != definition.vertex_text {
                *source = definition.vertex_text.clone();
                *program = BlockProgram::new(&ctx.device, &batch.plugin, source);
            }
            program.record(&ctx.device, &ctx.queue, &mut encoder, t.as_seconds_f64() as f32, &batch.items, &batch.params, motion.buffer());
        }
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
}
