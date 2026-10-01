use std::collections::{BTreeMap, BTreeSet};

use crate::doc::core::{Fps, RationalTime};
use crate::doc::store::{LayerId, StoreError, StoreView};
use crate::render::compositor::effects::isf::{TimeBase, TimeOffset};
use crate::render::compositor::TimeSource;

use super::scene_program::{SceneContributionValue, ScenePlateValue};
use super::{DynamicInput, EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue, TimeDependency};

#[derive(Debug)]
pub enum LookbehindProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for LookbehindProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for LookbehindProgramError {}
impl From<StoreError> for LookbehindProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

#[derive(Clone)]
struct LayerInfo {
    parent: Option<LayerId>,
    in_point: RationalTime,
}

#[derive(Clone)]
struct ImageRequest {
    named_layers: Vec<LayerId>,
    temporal: Vec<(f32, TimeBase, TimeSource)>,
}

/// Final semantic scene decorator. SceneComposite remains recurrence-free;
/// this node asks that base scene for exact t′ values and only attaches the
/// image inputs that effects declared.
pub struct LookbehindProgram {
    node: GraphNode,
    scene: NodeKey,
    fps: Fps,
    background: [f32; 4],
    layers: BTreeMap<LayerId, LayerInfo>,
}

impl LookbehindProgram {
    pub fn compile(view: &StoreView<'_>, scene: NodeKey) -> Result<Self, LookbehindProgramError> {
        let composition = view.composition()?;
        let fps = composition.as_ref().map_or(Fps::try_new(30, 1).expect("valid fallback fps"), |composition| composition.fps);
        let background = composition.as_ref().map_or([0.0; 4], |composition| composition.background);
        let mut layers = BTreeMap::new();
        let mut identity = NodeIdentity::new(NodeKind::EffectImages, vec![scene]);
        identity.time_dependency = TimeDependency::Exact;
        identity.parameters.extend_from_slice(&fps.num().to_be_bytes());
        identity.parameters.extend_from_slice(&fps.den().to_be_bytes());
        for component in background {
            identity.parameters.extend_from_slice(&component.to_bits().to_be_bytes());
        }
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let parent = view.attrs(layer)?.unwrap_or_default().parent;
            let in_point = RationalTime::try_from_frame(meta.timing.start, fps)
                .map_err(|error| StoreError::Property(error.to_string()))?;
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&parent.map_or(0, |parent| parent.0).to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.start.to_be_bytes());
            layers.insert(layer, LayerInfo { parent, in_point });
        }
        Ok(Self {
            node: GraphNode::new(identity),
            scene,
            fps,
            background,
            layers,
        })
    }

    pub fn node(&self) -> GraphNode { self.node.clone() }
    pub fn key(&self) -> NodeKey { self.node.key() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, LookbehindProgramError>> {
        if node.key() != self.node.key() { return None; }
        let scene = match inputs.at(0).and_then(|value| value.downcast_ref::<SceneValue>()) {
            Some(scene) => scene,
            None => return Some(Err(LookbehindProgramError::InvalidInput(node.identity().kind))),
        };
        let times = self.requested_times(scene, context.time);
        Some(Ok(times.into_iter()
            .filter(|time| *time != context.time)
            .map(|time| DynamicInput { node: self.scene, time })
            .collect()))
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, LookbehindProgramError>> {
        if node.key() != self.node.key() { return None; }
        Some((|| {
            let current = inputs.at(0)
                .and_then(|value| value.downcast_ref::<SceneValue>())
                .cloned()
                .ok_or(LookbehindProgramError::InvalidInput(node.identity().kind))?;
            let times = self.requested_times(&current, context.time);
            let mut scenes = BTreeMap::new();
            scenes.insert(context.time, current.clone());
            let mut dynamic = 1usize;
            for time in times {
                if time == context.time { continue; }
                let scene = inputs.at(dynamic)
                    .and_then(|value| value.downcast_ref::<SceneValue>())
                    .cloned()
                    .ok_or(LookbehindProgramError::InvalidInput(node.identity().kind))?;
                dynamic += 1;
                scenes.insert(time, scene);
            }

            let mut out = current;
            self.attach_scene_sources(&mut out, context.time, &scenes);
            Ok(NodeValue::new(out))
        })())
    }

    fn requested_times(&self, scene: &SceneValue, now: RationalTime) -> Vec<RationalTime> {
        let mut times = BTreeSet::new();
        visit_layers(scene, &mut |layer| {
            for request in image_requests(layer) {
                if !request.named_layers.is_empty() { continue; }
                for (offset, base, _) in request.temporal {
                    times.insert(self.time_for(layer.layer, now, offset, base));
                }
            }
        });
        times.into_iter().collect()
    }

    fn attach_scene_sources(
        &self,
        scene: &mut SceneValue,
        now: RationalTime,
        scenes: &BTreeMap<RationalTime, SceneValue>,
    ) {
        let snapshot = scene.clone();
        self.attach_layers(&mut scene.layers, &snapshot, now, scenes);
    }

    fn attach_layers(
        &self,
        layers: &mut [SceneLayerValue],
        current: &SceneValue,
        now: RationalTime,
        scenes: &BTreeMap<RationalTime, SceneValue>,
    ) {
        for layer in layers {
            self.attach_layer(layer, current, now, scenes);
        }
    }

    fn attach_layer(
        &self,
        layer: &mut SceneLayerValue,
        current: &SceneValue,
        now: RationalTime,
        scenes: &BTreeMap<RationalTime, SceneValue>,
    ) {
        let requests = image_requests(layer);
        let mut rows = Vec::with_capacity(requests.len());
        for request in requests {
            let row = if !request.named_layers.is_empty() {
                request.named_layers.iter().map(|target| {
                    if *target == layer.layer { return None; }
                    find_layer(current, *target, false).map(|target| content_source(target, now, 0))
                }).collect::<Option<Vec<_>>>().unwrap_or_default()
            } else {
                request.temporal.iter().map(|(offset, base, source)| {
                    let at = self.time_for(layer.layer, now, *offset, *base);
                    let then = scenes.get(&at)?;
                    self.temporal_source(layer, then, *source, at, temporal_namespace(at, *source))
                }).collect::<Option<Vec<_>>>().unwrap_or_default()
            };
            rows.push(row);
        }
        layer.image_sources = rows;

        if let SceneContentValue::Plate(plate) = &mut layer.content {
            for member in &mut plate.members {
                if let Some(member) = member.layer.as_mut() {
                    self.attach_layer(member, current, now, scenes);
                }
            }
        }
    }

    fn temporal_source(
        &self,
        consumer: &SceneLayerValue,
        then: &SceneValue,
        source: TimeSource,
        time: RationalTime,
        namespace: u64,
    ) -> Option<SceneImageSourceValue> {
        match source {
            TimeSource::Own => {
                let wants_plate = matches!(consumer.content, SceneContentValue::Plate(_));
                find_layer(then, consumer.layer, wants_plate).map(|layer| content_source(layer, time, namespace))
            }
            TimeSource::Below => {
                let filtered = scene_before(then, consumer.layer)?;
                Some(SceneImageSourceValue::Scene { scene: filtered, background: self.background, time, namespace })
            }
            TimeSource::Comp => {
                let filtered = filter_scene(then, &|id| id != consumer.layer);
                Some(SceneImageSourceValue::Scene { scene: filtered, background: self.background, time, namespace })
            }
            TimeSource::Group => {
                let group = match &consumer.content {
                    SceneContentValue::Plate(plate) => plate.owner.or_else(|| self.layers.get(&consumer.layer).and_then(|info| info.parent)),
                    _ => self.layers.get(&consumer.layer).and_then(|info| info.parent),
                }?;
                let filtered = filter_scene(then, &|id| id != consumer.layer && self.belongs_to_group(id, group));
                Some(SceneImageSourceValue::Scene { scene: filtered, background: [0.0; 4], time, namespace })
            }
        }
    }

    fn belongs_to_group(&self, mut layer: LayerId, group: LayerId) -> bool {
        if layer == group { return true; }
        for _ in 0..64 {
            let Some(parent) = self.layers.get(&layer).and_then(|info| info.parent) else { return false };
            if parent == group { return true; }
            layer = parent;
        }
        false
    }

    fn time_for(&self, layer: LayerId, now: RationalTime, offset: f32, base: TimeBase) -> RationalTime {
        match base {
            TimeBase::Offset => shifted_seconds(now, offset),
            TimeBase::At => shifted_seconds(self.layers.get(&layer).map_or(RationalTime::ZERO, |info| info.in_point), offset),
            TimeBase::Frames => {
                let frames = offset.round() as i64;
                let now_frame = now.try_to_frame_round(self.fps).unwrap_or(0);
                RationalTime::try_from_frame((now_frame + frames).max(0), self.fps).unwrap_or(RationalTime::ZERO)
            }
        }
    }
}

fn image_requests(layer: &SceneLayerValue) -> Vec<ImageRequest> {
    let catalog = crate::render::engine::known_effects();
    let mut out = Vec::new();
    for (effect, plate_chain) in layer.effects.iter().map(|effect| (effect, false))
        .chain(layer.after_effects.iter().map(|effect| (effect, true)))
    {
        let Some(descriptor) = catalog.iter().find(|descriptor| descriptor.plugin_id == effect.plugin_id) else { continue };
        let accepted = if plate_chain {
            matches!(descriptor.stage, crate::render::compositor::EffectStage::Pass | crate::render::compositor::EffectStage::Warp)
        } else {
            descriptor.stage == crate::render::compositor::EffectStage::Pass
        };
        if !accepted { continue; }

        let named_layers = descriptor.image_layer_fields.iter().filter_map(|field| {
            effect.params.iter().find(|(name, _)| name == field).and_then(|(_, value)| match value {
                crate::doc::store::Value::LayerId(id) if *id != 0 => Some(LayerId(*id)),
                _ => None,
            })
        }).collect();

        let temporal = descriptor.image_time_offsets.iter().enumerate().map(|(index, offset)| {
            let value = match offset {
                TimeOffset::Fixed(seconds) => *seconds,
                TimeOffset::Param(name) => effect.params.iter()
                    .find(|(field, _)| field == name)
                    .and_then(|(_, value)| match value { crate::doc::store::Value::F64(value) => Some(*value as f32), _ => None })
                    .or_else(|| descriptor.params.iter().find(|param| &param.name == name).map(|param| param.default as f32))
                    .unwrap_or(0.0),
            };
            (
                value,
                descriptor.image_time_bases.get(index).copied().unwrap_or_default(),
                descriptor.image_time_sources.get(index).copied().unwrap_or_default(),
            )
        }).collect();

        out.push(ImageRequest { named_layers, temporal });
    }
    out
}

fn content_source(layer: &SceneLayerValue, time: RationalTime, namespace: u64) -> SceneImageSourceValue {
    SceneImageSourceValue::Content { layer: layer.layer, content: layer.content.clone(), time, namespace }
}

fn temporal_namespace(time: RationalTime, source: TimeSource) -> u64 {
    use std::hash::{Hash, Hasher};
    let absolute_ms = (time.as_seconds_f64() * 1000.0).round() as i64;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    absolute_ms.hash(&mut hasher);
    (source as u8).hash(&mut hasher);
    hasher.finish() | 1
}

fn find_layer(scene: &SceneValue, id: LayerId, prefer_plate: bool) -> Option<&SceneLayerValue> {
    fn scan_layer<'a>(
        layer: &'a SceneLayerValue,
        id: LayerId,
        prefer_plate: bool,
        fallback: &mut Option<&'a SceneLayerValue>,
    ) -> Option<&'a SceneLayerValue> {
        if layer.layer == id {
            if prefer_plate == matches!(layer.content, SceneContentValue::Plate(_)) {
                return Some(layer);
            }
            if fallback.is_none() { *fallback = Some(layer); }
        }
        if let SceneContentValue::Plate(plate) = &layer.content {
            for member in &plate.members {
                let Some(member) = member.layer.as_ref() else { continue };
                if let Some(found) = scan_layer(member, id, prefer_plate, fallback) {
                    return Some(found);
                }
            }
        }
        None
    }

    let mut fallback = None;
    for layer in &scene.layers {
        if let Some(found) = scan_layer(layer, id, prefer_plate, &mut fallback) {
            return Some(found);
        }
    }
    fallback
}

fn filter_scene(scene: &SceneValue, keep: &dyn Fn(LayerId) -> bool) -> SceneValue {
    SceneValue { layers: scene.layers.iter().filter_map(|layer| filter_layer(layer, keep)).collect() }
}

fn filter_layer(layer: &SceneLayerValue, keep: &dyn Fn(LayerId) -> bool) -> Option<SceneLayerValue> {
    let mut layer = layer.clone();
    if let SceneContentValue::Plate(plate) = &mut layer.content {
        let members = std::mem::take(&mut plate.members);
        plate.members = members.into_iter().filter_map(|mut member| {
            let inner = member.layer.take()?;
            member.layer = filter_layer(&inner, keep);
            member.layer.as_ref()?;
            Some(member)
        }).collect();
        if !keep(layer.layer) && plate.members.is_empty() { return None; }
        return Some(layer);
    }
    keep(layer.layer).then_some(layer)
}

fn scene_before(scene: &SceneValue, target: LayerId) -> Option<SceneValue> {
    let mut stopped = false;
    let mut found = false;
    let mut layers = Vec::new();
    for layer in &scene.layers {
        if stopped { break; }
        if layer.layer == target {
            found = true;
            break;
        }
        if let SceneContentValue::Plate(plate) = &layer.content {
            if plate_contains(plate, target) {
                let mut clone = layer.clone();
                if let SceneContentValue::Plate(cloned) = &mut clone.content {
                    cloned.members = members_before(&plate.members, target, &mut found);
                }
                if !matches!(&clone.content, SceneContentValue::Plate(p) if p.members.is_empty()) {
                    layers.push(clone);
                }
                stopped = true;
                continue;
            }
        }
        layers.push(layer.clone());
    }
    found.then_some(SceneValue { layers })
}

fn members_before(members: &[SceneContributionValue], target: LayerId, found: &mut bool) -> Vec<SceneContributionValue> {
    let mut out = Vec::new();
    for member in members {
        let Some(layer) = member.layer.as_ref() else { continue };
        if layer.layer == target {
            *found = true;
            break;
        }
        if let SceneContentValue::Plate(plate) = &layer.content {
            if plate_contains(plate, target) {
                let mut clone = member.clone();
                if let Some(layer) = clone.layer.as_mut() {
                    if let SceneContentValue::Plate(inner) = &mut layer.content {
                        inner.members = members_before(&plate.members, target, found);
                    }
                }
                out.push(clone);
                break;
            }
        }
        out.push(member.clone());
    }
    out
}

fn plate_contains(plate: &ScenePlateValue, target: LayerId) -> bool {
    plate.members.iter().any(|member| member.layer.as_ref().is_some_and(|layer| {
        layer.layer == target || matches!(&layer.content, SceneContentValue::Plate(inner) if plate_contains(inner, target))
    }))
}

fn visit_layers(scene: &SceneValue, visitor: &mut dyn FnMut(&SceneLayerValue)) {
    fn visit_layer(layer: &SceneLayerValue, visitor: &mut dyn FnMut(&SceneLayerValue)) {
        visitor(layer);
        if let SceneContentValue::Plate(plate) = &layer.content {
            for member in &plate.members {
                if let Some(layer) = member.layer.as_ref() { visit_layer(layer, visitor); }
            }
        }
    }
    for layer in &scene.layers { visit_layer(layer, visitor); }
}

fn shifted_seconds(time: RationalTime, offset: f32) -> RationalTime {
    const DEN: i64 = 1000;
    let delta = RationalTime::try_new((offset as f64 * DEN as f64).round() as i64, DEN).ok();
    delta.and_then(|delta| time.try_add(delta).ok())
        .filter(|time| *time >= RationalTime::ZERO)
        .unwrap_or(RationalTime::ZERO)
}
