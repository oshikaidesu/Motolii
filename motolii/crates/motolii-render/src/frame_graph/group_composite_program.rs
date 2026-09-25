use std::collections::{BTreeMap, BTreeSet};

use crate::doc::store::{EffectScope, LayerId, LayerProjection, LayerSource, StoreError, StoreView};

use super::scene_program::{SceneContentValue, SceneContributionValue, SceneLayerValue, ScenePlateValue};
use super::{EffectProgram, EffectValue, EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, TimeDependency, TransformValue};

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SceneFragmentValue {
    pub(super) contributions: Vec<SceneContributionValue>,
}

#[derive(Clone)]
struct Recipe {
    group: LayerId,
    child_count: usize,
    effect_start: usize,
    /// The group's first placement effect: where its copy set and effect value sit among the
    /// inputs, and its program (which child each copy picks).
    placement: Option<GroupPlacement>,
}

#[derive(Clone, Copy)]
struct GroupPlacement {
    set: usize,
    effect: usize,
    program: crate::doc::store::kind::PlacementProgram,
}

#[derive(Debug)]
pub(super) enum GroupCompositeProgramError {
    Store(StoreError),
    Cycle(LayerId),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for GroupCompositeProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for GroupCompositeProgramError {}
impl From<StoreError> for GroupCompositeProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

/// Compiles authoring Group scopes into semantic fragments. A Group node is
/// always a scope boundary, but it emits an actual Plate only when an evaluated
/// Whole effect requires one.
pub(super) struct GroupCompositeProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    group_nodes: BTreeMap<LayerId, NodeKey>,
    roots: Vec<NodeKey>,
}

impl GroupCompositeProgram {
    pub(super) fn compile(
        view: &StoreView<'_>,
        effects: &EffectProgram,
        placements: &super::PlacementProgram,
        contributions: &BTreeMap<LayerId, NodeKey>,
    ) -> Result<Self, GroupCompositeProgramError> {
        let mut metas = BTreeMap::new();
        let mut children: BTreeMap<Option<LayerId>, Vec<LayerId>> = BTreeMap::new();

        for layer in view.layers() {
            if !contributions.contains_key(&layer) { continue; }
            let Some(meta) = view.meta(layer)? else { continue };
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let parent = match attrs.parent {
                Some(parent) if view.meta(parent)?.is_some_and(|meta| meta.source == LayerSource::Group) && contributions.contains_key(&parent) => Some(parent),
                _ => None,
            };
            metas.insert(layer, meta.clone());
            children.entry(parent).or_default().push(layer);
        }
        for list in children.values_mut() {
            list.sort_by_key(|layer| (metas.get(layer).map_or(0, |meta| meta.order), layer.0));
        }

        let mut program = Self {
            nodes: BTreeMap::new(),
            recipes: BTreeMap::new(),
            group_nodes: BTreeMap::new(),
            roots: Vec::new(),
        };
        let mut visiting = BTreeSet::new();

        fn build_group(
            group: LayerId,
            program: &mut GroupCompositeProgram,
            metas: &BTreeMap<LayerId, crate::doc::store::LayerMeta>,
            children: &BTreeMap<Option<LayerId>, Vec<LayerId>>,
            effects: &EffectProgram,
            placements: &super::PlacementProgram,
            contributions: &BTreeMap<LayerId, NodeKey>,
            visiting: &mut BTreeSet<LayerId>,
        ) -> Result<NodeKey, GroupCompositeProgramError> {
            if let Some(key) = program.group_nodes.get(&group) { return Ok(*key); }
            if !visiting.insert(group) { return Err(GroupCompositeProgramError::Cycle(group)); }

            let owner = *contributions.get(&group).ok_or(GroupCompositeProgramError::InvalidInput(NodeKind::GroupComposite))?;
            let direct = children.get(&Some(group)).map(Vec::as_slice).unwrap_or(&[]);
            let mut inputs = Vec::with_capacity(1 + direct.len() + effects.binding(group).map_or(0, |binding| binding.effects.len()));
            inputs.push(owner);

            for child in direct {
                let key = if metas.get(child).is_some_and(|meta| meta.source == LayerSource::Group) {
                    build_group(*child, program, metas, children, effects, placements, contributions, visiting)?
                } else {
                    *contributions.get(child).ok_or(GroupCompositeProgramError::InvalidInput(NodeKind::GroupComposite))?
                };
                inputs.push(key);
            }
            let child_count = direct.len();
            let effect_start = inputs.len();
            let mut placement = None;
            if let Some(binding) = effects.binding(group) {
                let end = binding.effects.iter().position(|key| effects.is_placement(*key)).unwrap_or(binding.effects.len());
                inputs.extend(binding.effects[..end].iter().copied());
                if let (Some(&effect), Some(set)) = (binding.effects.get(end), placements.binding(group)) {
                    if let Some(program) = effects.placement_program(effect) {
                        placement = Some(GroupPlacement { set: inputs.len(), effect: inputs.len() + 1, program });
                        inputs.extend([set.node, effect]);
                        // Effects after the placement read its result: the copies as one picture.
                        inputs.extend(binding.effects[end + 1..].iter().copied());
                    }
                }
            }

            let mut identity = NodeIdentity::new(NodeKind::GroupComposite, inputs);
            identity.parameters.extend_from_slice(&group.0.to_be_bytes());
            identity.parameters.extend_from_slice(&(child_count as u32).to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            program.nodes.entry(key).or_insert(node);
            program.recipes.entry(key).or_insert(Recipe { group, child_count, effect_start, placement });
            program.group_nodes.insert(group, key);
            visiting.remove(&group);
            Ok(key)
        }

        let root_layers = children.get(&None).cloned().unwrap_or_default();
        for layer in root_layers {
            let key = if metas.get(&layer).is_some_and(|meta| meta.source == LayerSource::Group) {
                build_group(layer, &mut program, &metas, &children, effects, placements, contributions, &mut visiting)?
            } else {
                *contributions.get(&layer).ok_or(GroupCompositeProgramError::InvalidInput(NodeKind::GroupComposite))?
            };
            program.roots.push(key);
        }
        Ok(program)
    }

    pub(super) fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ {
        self.nodes.values().cloned()
    }

    pub(super) fn roots(&self) -> &[NodeKey] { &self.roots }

    pub(super) fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<NodeValue, GroupCompositeProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let owner = inputs.at(0)
                .and_then(|value| value.downcast_ref::<SceneContributionValue>())
                .cloned()
                .ok_or(GroupCompositeProgramError::InvalidInput(node.identity().kind))?;

            // Each direct child with its subtree, in order.
            let mut subtrees: Vec<Vec<SceneContributionValue>> = Vec::with_capacity(recipe.child_count);
            for index in 0..recipe.child_count {
                let value = inputs.at(index + 1).ok_or(GroupCompositeProgramError::InvalidInput(node.identity().kind))?;
                if let Some(contribution) = value.downcast_ref::<SceneContributionValue>() {
                    subtrees.push(vec![contribution.clone()]);
                } else if let Some(fragment) = value.downcast_ref::<SceneFragmentValue>() {
                    subtrees.push(fragment.contributions.clone());
                } else {
                    return Err(GroupCompositeProgramError::InvalidInput(node.identity().kind));
                }
            }
            let mut children: Vec<SceneContributionValue> = subtrees.iter().flatten().cloned().collect();

            let mut each = Vec::new();
            let mut whole = Vec::new();
            let mut in_whole = false;
            let effect_end = recipe.placement.map_or(node.identity().inputs.len(), |placement| placement.set);
            for index in recipe.effect_start..effect_end {
                let Some(effect) = inputs.at(index)
                    .and_then(|value| value.downcast_ref::<EffectValue>())
                    .and_then(|value| value.0.clone()) else { continue };
                if in_whole || effect.scope == EffectScope::Whole {
                    in_whole = true;
                    whole.push(effect);
                } else {
                    each.push(effect);
                }
            }

            if !each.is_empty() {
                for child in &mut children {
                    append_each(child, &each);
                }
            }

            if let Some(placed) = recipe.placement.and_then(|placement| placed_copies(placement, inputs, &owner, &subtrees, &each)) {
                let after_start = recipe.placement.map_or(0, |placement| placement.effect + 1);
                let after: Vec<_> = (after_start..node.identity().inputs.len()).filter_map(|index| inputs.at(index)
                    .and_then(|value| value.downcast_ref::<EffectValue>())
                    .and_then(|value| value.0.clone())).collect();
                // The copies stand in for the group: it is not drawn itself.
                let mut contributions = vec![SceneContributionValue { solo: owner.solo, layer: None }];
                if after.is_empty() {
                    contributions.extend(placed);
                } else {
                    contributions.push(plate(recipe.group, &owner, placed, after));
                }
                return Ok(NodeValue::new(SceneFragmentValue { contributions }));
            }

            let mut contributions = vec![owner.clone()];
            if whole.is_empty() {
                contributions.extend(children);
                return Ok(NodeValue::new(SceneFragmentValue { contributions }));
            }
            contributions.push(plate(recipe.group, &owner, children, whole));
            Ok(NodeValue::new(SceneFragmentValue { contributions }))
        })())
    }
}

/// The members composed into one picture owned by the group, with `effects` applied to that picture.
fn plate(group: LayerId, owner: &SceneContributionValue, children: Vec<SceneContributionValue>, effects: Vec<crate::picture::resolved::ResolvedEffect>) -> SceneContributionValue {
    let solo = children.iter().any(|child| child.solo);
    let seed = children.iter().find_map(|child| child.layer.as_ref()).cloned();
    if let Some(mut layer) = seed {
        let owner_layer = owner.layer.as_ref();
        layer.layer = group;
        layer.instance = 0;
        layer.content_key = None;
        layer.freeze_eligible = owner_layer.is_some_and(|owner| owner.freeze_eligible);
        layer.timing_start = owner_layer.map_or(layer.timing_start, |owner| owner.timing_start);
        layer.content = SceneContentValue::Plate(ScenePlateValue { owner: Some(group), members: children, average: false });
        layer.effects.clear();
        layer.after_effects = effects;
        layer.masks.clear();
        layer.matte = None;
        layer.clip_to_below = false;
        layer.flatten = false;
        layer.environment = false;
        layer.opacity = owner_layer.map_or(1.0, |owner| owner.opacity);
        layer.blend = owner_layer.map_or(layer.blend, |owner| owner.blend);
        layer.projection = LayerProjection::TwoD;
        layer.transform = TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY };
        SceneContributionValue { solo, layer: Some(layer) }
    } else {
        // Preserve solo participation even if every child is currently
        // hidden/out-of-range; this matches the global solo rule.
        SceneContributionValue { solo, layer: None }
    }
}


/// A placement effect on a group copies its children: with Each scope every copy picks one child
/// (and its subtree) by the program's pick; with Whole scope every copy carries all of them. A copy
/// moves its children as the group's own copy moves the group. Copies at another time are not
/// placed here yet.
fn placed_copies(
    placement: GroupPlacement,
    inputs: &NodeInputs,
    owner: &SceneContributionValue,
    subtrees: &[Vec<SceneContributionValue>],
    each: &[crate::picture::resolved::ResolvedEffect],
) -> Option<Vec<SceneContributionValue>> {
    let set = inputs.at(placement.set)?.downcast_ref::<super::PlacementSetValue>()?;
    set.selected_effect?;
    let effect = inputs.at(placement.effect)?.downcast_ref::<EffectValue>()?.0.clone()?;
    let group = owner.layer.as_ref()?;
    let whole = effect.scope == EffectScope::Whole;
    let heads: Vec<u64> = subtrees.iter().filter_map(|tree| tree.first().and_then(|c| c.layer.as_ref()).map(|l| l.layer.0)).collect();
    if heads.len() != subtrees.len() || subtrees.is_empty() { return Some(Vec::new()); }
    let picks = (placement.program.pick)(&effect.params, &heads, set.copies.len());
    // Copies of picked children stack in copy order at the children's front-most slot.
    let slot = (!whole).then(|| subtrees.iter().flatten().filter_map(|c| c.layer.as_ref().map(|l| l.order)).min()).flatten();
    let (group_affine, group_spatial) = (group.transform.affine.inverse(), group.transform.spatial.inverse());
    let mut out = Vec::new();
    for (ordinal, copy) in set.copies.iter().enumerate() {
        let Some(moved) = copy.transform else { continue };
        let (affine, spatial) = (moved.affine * group_affine, moved.spatial * group_spatial);
        let picked: Vec<&SceneContributionValue> = if whole {
            subtrees.iter().flatten().collect()
        } else {
            let Some(&pick) = picks.get(ordinal) else { continue };
            subtrees.get(pick).map(|tree| tree.iter().collect()).unwrap_or_default()
        };
        for contribution in picked {
            let mut contribution = contribution.clone();
            if let Some(layer) = contribution.layer.as_mut() {
                layer.instance = copy.index;
                layer.transform = TransformValue { affine: affine * layer.transform.affine, spatial: spatial * layer.transform.spatial };
                layer.opacity = (layer.opacity * copy.opacity).clamp(0.0, 1.0);
                layer.shape_stretch = copy.outline_stretch;
                layer.freeze_eligible = false;
                if let Some(slot) = slot { layer.order = slot; }
                layer.effects.extend(each.iter().cloned());
            }
            out.push(contribution);
        }
    }
    Some(out)
}

fn append_each(contribution: &mut SceneContributionValue, effects: &[crate::picture::resolved::ResolvedEffect]) {
    if let Some(layer) = contribution.layer.as_mut() {
        layer.effects.extend(effects.iter().cloned());
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::Value;
    use crate::doc::store::{
        Composition, EffectId, EffectInstance, Fps, LayerAttrsPatch, LayerMeta, LayerSource,
        LayerTiming, PropertyId, ShapeNode,
    };
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
    use crate::frame_graph::{
        CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor,
        SceneContentValue, SceneProgram, SceneProgramError, SceneValue,
    };
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(
            &mut self,
            node: &GraphNode,
            inputs: NodeInputs,
            context: EvaluationContext,
        ) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }

    fn rectangle() -> Vec<ShapeNode> {
        vec![ShapeNode::Leaf(Shape {
            source: PathSource::Rectangle { size: Point { x: 20.0, y: 20.0 } },
            ops: Vec::new(),
            stroke: None,
            fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }),
        })]
    }

    #[test]
    fn group_each_stays_on_children_and_whole_becomes_one_plate() {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0; 4], look: Default::default()
        })).unwrap();

        let group = LayerId(1);
        let leaf = LayerId(2);
        doc.apply_all([
            Intent::AddLayer(group),
            Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 30) } },
            Intent::AddLayer(leaf),
            Intent::SetMeta { layer: leaf, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 30) } },
            Intent::SetAttrs { layer: leaf, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } },
            Intent::SetShapes { layer: leaf, shapes: rectangle() },
        ]).unwrap();

        let leaf_effect = EffectId(10);
        doc.apply(Intent::SetEffects {
            layer: leaf,
            effects: vec![EffectInstance { id: leaf_effect, plugin_id: "leaf.own".into() }],
        }).unwrap();

        let each = EffectId(20);
        let whole = EffectId(21);
        let after = EffectId(22);
        doc.apply(Intent::SetEffects {
            layer: group,
            effects: vec![
                EffectInstance { id: each, plugin_id: "group.each".into() },
                EffectInstance { id: whole, plugin_id: "group.whole".into() },
                EffectInstance { id: after, plugin_id: "group.after".into() },
            ],
        }).unwrap();
        doc.apply(Intent::SetConstant {
            layer: group,
            property: PropertyId::effect_scope(whole),
            value: Value::Enum(EffectScope::Whole.enum_value()),
        }).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(
            &mut executor,
            crate::doc::core::RationalTime::ZERO,
            FrameQuality::Export,
            Generation::new(1),
        ).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap();

        let plate_layer = scene.layers.iter().find(|layer| matches!(layer.content, SceneContentValue::Plate(_))).expect("Group Whole must create one semantic plate");
        assert_eq!(
            plate_layer.after_effects.iter().map(|effect| effect.plugin_id.as_str()).collect::<Vec<_>>(),
            ["group.whole", "group.after"],
        );
        let SceneContentValue::Plate(plate) = &plate_layer.content else { unreachable!() };
        let member = plate.members.iter().find_map(|member| member.layer.as_ref()).expect("plate member");
        assert_eq!(
            member.effects.iter().map(|effect| effect.plugin_id.as_str()).collect::<Vec<_>>(),
            ["leaf.own", "group.each"],
        );
    }
}
