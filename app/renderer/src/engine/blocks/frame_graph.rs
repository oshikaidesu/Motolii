use super::*;

impl Engine {
    /// FrameGraph path: build block/Follow/field/physics state from evaluated
    /// scene + solver values only. No StoreView/document reads are allowed here.
    pub(in crate::engine) fn prepare_frame_graph_blocks(
        &mut self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        t: RationalTime,
        fps: crate::doc::store::Fps,
        prepared: &mut crate::engine::frame_graph_scene::GpuSceneValue,
    ) -> Result<(), EngineError> {
        let block_ids: Vec<String> = self.compositor.catalog.definitions.iter()
            .filter(|definition| definition.manifest.stage == IsfStage::Block)
            .map(|definition| definition.plugin_id().to_owned())
            .collect();
        let definitions: Vec<(String, bool, Vec<(String, f32)>)> = self.compositor.catalog.definitions.iter()
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
        let lies = self.compositor.catalog.definitions.iter()
            .find(|definition| {
                definition.manifest.stage == IsfStage::Block
                    && definition.manifest.scope == crate::render::compositor::effects::isf::IsfScope::Room
                    && !definition.manifest.physics.is_empty()
            })
            .map(|definition| crate::render::engine::physics::Lies::from_manifest(&definition.manifest.physics))
            .unwrap_or_default();
        if self.blocks.physics.lies != lies {
            self.blocks.physics = Default::default();
            self.blocks.physics.lies = lies.clone();
        }

        let state = &mut self.blocks;
        state.fps = fps.as_f64();
        state.last_frame = 0;
        state.now = Some(t);
        state.placed.clear();
        state.follows.clear();
        state.object_layers.clear();
        state.object_frames.clear();
        state.slots.clear();
        state.connectors.clear();
        state.traces.clear();
        state.ropes.clear();
        state.field_rooms.clear();
        state.fields.clear();
        state.objects.clear();
        state.bases.clear();
        state.outlines.clear();
        state.batches.clear();

        if block_ids.is_empty() {
            self.compositor.motion = None;
            return Ok(());
        }

        let mut seen = std::collections::HashSet::new();
        let primary: Vec<&SceneLayerValue> = scene.layers.iter()
            .filter(|layer| layer.instance == 0 && !layer.ghost && seen.insert(layer.layer))
            .collect();
        let primary_by_id: HashMap<LayerId, &SceneLayerValue> =
            primary.iter().map(|layer| (layer.layer, *layer)).collect();

        let mut gpu_sizes: HashMap<LayerId, [f32; 2]> = HashMap::new();
        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter()) {
            gpu_sizes.entry(id).or_insert(layer.layer.size);
        }

        let semantic_size = |layer: &SceneLayerValue| -> Option<[f32; 2]> {
            let shapes: Option<Vec<crate::doc::store::ShapeNode>> = match &layer.content {
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
                .or_else(|| gpu_sizes.get(&id).copied())
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
                _ => self.keyed_outlines.get(id).cloned(),
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

        self.solve_physics(t);

        for (id, layer) in prepared.layer_ids.iter().copied().zip(prepared.layers.iter_mut()) {
            self.attach_block_id(id, &mut layer.layer, comp);
        }
        self.run_blocks(t, fps.as_f64());
        Ok(())
    }
}
