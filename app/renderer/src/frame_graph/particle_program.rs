use std::collections::BTreeMap;

use crate::doc::core::{Fps, RationalTime};
use crate::doc::eval::Value;
use crate::doc::store::{particles, LayerId, LayerSource, PropertyId, StoreError, StoreView};

use super::{DynamicInput, EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Debug, PartialEq)]
struct BirthState {
    accumulated: f64,
    next_index: u32,
    births: Vec<(u32, f64)>,
}

impl Default for BirthState {
    fn default() -> Self {
        Self { accumulated: 0.0, next_index: 0, births: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleValue {
    pub particles: Vec<particles::Particle>,
    pub turbulence: particles::Turbulence,
    pub links: particles::Links,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleBinding {
    pub layer: LayerId,
    pub births: NodeKey,
    pub particles: NodeKey,
}

#[derive(Clone)]
enum Recipe {
    Births {
        start: i64,
        fps: Fps,
        rate: Option<usize>,
    },
    Particles {
        start: i64,
        fps: Fps,
        births: usize,
        rows: Vec<(&'static str, Option<usize>)>,
    },
}

#[derive(Debug)]
pub enum ParticleProgramError {
    Store(StoreError),
    Time(String),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for ParticleProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ParticleProgramError {}
impl From<StoreError> for ParticleProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct ParticleProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, ParticleBinding>,
}

impl ParticleProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram) -> Result<Self, ParticleProgramError> {
        let Some(fps) = view.composition()?.map(|composition| composition.fps) else {
            return Ok(Self { nodes: BTreeMap::new(), recipes: BTreeMap::new(), bindings: BTreeMap::new() });
        };
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if meta.source != LayerSource::Particles { continue; }

            let rate_key = properties.node_for(layer, &PropertyId::new(particles::RATE)?);
            let mut birth_inputs = Vec::new();
            let rate = rate_key.map(|key| {
                let index = birth_inputs.len();
                birth_inputs.push(key);
                index
            });
            let mut birth_identity = NodeIdentity::new(NodeKind::ParticleBirths, birth_inputs);
            birth_identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            birth_identity.parameters.extend_from_slice(&meta.timing.start.to_be_bytes());
            birth_identity.parameters.extend_from_slice(&fps.num().to_be_bytes());
            birth_identity.parameters.extend_from_slice(&fps.den().to_be_bytes());
            birth_identity.time_dependency = TimeDependency::Exact;
            let birth_node = GraphNode::new(birth_identity);
            let birth_key = birth_node.key();
            nodes.entry(birth_key).or_insert(birth_node);
            recipes.entry(birth_key).or_insert(Recipe::Births { start: meta.timing.start, fps, rate });

            let mut inputs = vec![birth_key];
            let births = 0usize;
            let mut rows = Vec::new();
            for &(name, _, _, _) in particles::ROWS {
                if name == particles::RATE { continue; }
                let input = properties.node_for(layer, &PropertyId::new(name)?).map(|key| {
                    let index = inputs.len();
                    inputs.push(key);
                    index
                });
                rows.push((name, input));
            }

            let mut identity = NodeIdentity::new(NodeKind::Particle, inputs);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&meta.timing.start.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let particle_node = GraphNode::new(identity);
            let particle_key = particle_node.key();
            nodes.entry(particle_key).or_insert(particle_node);
            recipes.entry(particle_key).or_insert(Recipe::Particles {
                start: meta.timing.start,
                fps,
                births,
                rows,
            });
            bindings.insert(layer, ParticleBinding { layer, births: birth_key, particles: particle_key });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<ParticleBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, ParticleProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Births { start, fps, .. } => (|| {
                let frame = context.time.try_to_frame_floor(*fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                if frame <= *start { return Ok(Vec::new()); }
                let previous = RationalTime::try_from_frame(frame - 1, *fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                Ok(vec![DynamicInput { node: node.key(), time: previous }])
            })(),
            Recipe::Particles { start, fps, births, rows } => (|| {
                let frame = context.time.try_to_frame_floor(*fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                if frame <= *start { return Ok(Vec::new()); }
                let life = row_number(inputs, rows, particles::LIFE).max(0.01);
                let reach = (life * fps.as_f64()).ceil() as i64 + 1;
                let first = (*start).max(frame - reach);
                let birth_node = node.identity().inputs[*births];
                let mut requests = Vec::new();
                for sample in first..frame {
                    let at = RationalTime::try_from_frame(sample, *fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                    requests.push(DynamicInput { node: birth_node, time: at });
                }
                Ok(requests)
            })(),
        })
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, ParticleProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Births { start, fps, rate } => {
                let result = (|| {
                    let frame = context.time.try_to_frame_floor(*fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                    if frame < *start { return Ok(BirthState::default()); }
                    let previous = if frame > *start {
                        inputs.at(node.identity().inputs.len())
                            .and_then(|value| value.downcast_ref::<BirthState>())
                            .cloned()
                            .ok_or(ParticleProgramError::InvalidInput(node.identity().kind))?
                    } else {
                        BirthState::default()
                    };
                    let rate = rate.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()).and_then(|value| match value {
                        Value::F64(value) if value.is_finite() => Some(*value),
                        _ => None,
                    }).or_else(|| match particles::default_of(particles::RATE) { Some(Value::F64(value)) => Some(value), _ => None }).unwrap_or(0.0).max(0.0);
                    let frame_seconds = fps.den() as f64 / fps.num() as f64;
                    let before = previous.accumulated;
                    let accumulated = before + rate * frame_seconds;
                    let count = accumulated.floor() as u64 - before.floor() as u64;
                    let at = RationalTime::try_from_frame(frame, *fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?.as_seconds_f64();
                    let mut births = Vec::with_capacity(count as usize);
                    for k in 0..count {
                        births.push((
                            previous.next_index.wrapping_add(k as u32),
                            at + frame_seconds * (k as f64 + 0.5) / count as f64,
                        ));
                    }
                    Ok(BirthState {
                        accumulated,
                        next_index: previous.next_index.wrapping_add(count as u32),
                        births,
                    })
                })();
                result.map(NodeValue::new)
            }
            Recipe::Particles { start, fps, births, rows } => {
                let result = (|| {
                    let frame = context.time.try_to_frame_floor(*fps).map_err(|error| ParticleProgramError::Time(error.to_string()))?;
                    let mut values = BTreeMap::new();
                    for &(name, index) in rows {
                        if let Some(value) = index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()).cloned() {
                            values.insert(name.to_owned(), value);
                        }
                    }

                    let mut all_births = Vec::new();
                    if frame >= *start {
                        let dynamic_start = node.identity().inputs.len();
                        for index in dynamic_start..inputs.len() {
                            let state = inputs.at(index).and_then(|value| value.downcast_ref::<BirthState>())
                                .ok_or(ParticleProgramError::InvalidInput(node.identity().kind))?;
                            all_births.extend_from_slice(&state.births);
                        }
                        let current = inputs.at(*births).and_then(|value| value.downcast_ref::<BirthState>())
                            .ok_or(ParticleProgramError::InvalidInput(node.identity().kind))?;
                        all_births.extend_from_slice(&current.births);
                    }

                    let (particles, turbulence, links) = particles::particles_from_values_and_births(
                        &values,
                        &all_births,
                        context.time.as_seconds_f64(),
                    );
                    Ok(ParticleValue { particles, turbulence, links })
                })();
                result.map(NodeValue::new)
            }
        })
    }
}

fn row_number(inputs: &NodeInputs, rows: &[(&'static str, Option<usize>)], name: &str) -> f64 {
    rows.iter().find(|(candidate, _)| *candidate == name)
        .and_then(|(_, index)| *index)
        .and_then(|index| inputs.at(index))
        .and_then(|value| value.downcast_ref::<Value>())
        .and_then(|value| match value { Value::F64(value) if value.is_finite() => Some(*value), _ => None })
        .or_else(|| match particles::default_of(name) { Some(Value::F64(value)) => Some(value), _ => None })
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, LayerMeta, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn dynamic_inputs(&mut self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<DynamicInput>, Self::Error> {
            self.0.dynamic_inputs(node, inputs, context)
        }
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }

    #[test]
    fn graph_particles_match_the_legacy_particle_oracle() {
        let fps = Fps::try_new(10, 1).unwrap();
        let mut doc = Document::new();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::SetComposition(Composition { width: 320, height: 180, fps, duration_frames: 50, background: [0.0; 4] }),
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Particles, order: 0, timing: LayerTiming::place(0, None, 50) } },
        ]).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let binding = program.particles().binding(layer).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), vec![binding.particles]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let at = RationalTime::try_new(17, 10).unwrap();
        let frame = graph.evaluate(&mut executor, at, FrameQuality::Export, Generation::new(1)).unwrap();
        let graph_value = frame.value(binding.particles).and_then(|value| value.downcast_ref::<ParticleValue>()).unwrap();
        let legacy = particles::particles_at(&doc.view(), layer, at).unwrap();

        assert_eq!(graph_value.particles, legacy.0);
        assert_eq!(graph_value.turbulence, legacy.1);
        assert_eq!(graph_value.links, legacy.2);
    }
}
