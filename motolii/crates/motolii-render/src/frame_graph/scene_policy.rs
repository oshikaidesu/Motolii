use std::collections::{BTreeMap, HashSet};

use crate::doc::store::{BlendMode, LayerId, LayerSource, Matte, MatteMode, StoreError, StoreView};

use super::SceneLayerValue;

#[derive(Clone)]
struct LayerPolicy {
    parent: Option<LayerId>,
    order: i16,
    clip_base: Option<LayerId>,
}

/// Revision-scoped structural rules used by SceneComposite. Runtime application
/// reads only evaluated SceneLayerValue data plus this immutable topology.
#[derive(Clone, Default)]
pub(super) struct ScenePolicy {
    layers: BTreeMap<LayerId, LayerPolicy>,
}

impl ScenePolicy {
    pub(super) fn compile(view: &StoreView<'_>) -> Result<Self, StoreError> {
        let mut layers = BTreeMap::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let attrs = view.attrs(layer)?.unwrap_or_default();
            layers.insert(layer, LayerPolicy {
                parent: attrs.parent,
                order: meta.order,
                clip_base: attrs.clip_to_below.then(|| view.clipping_base(layer)).transpose()?.flatten(),
            });
        }
        Ok(Self { layers })
    }

    /// Stencil/Silhouette are authoring blend modes, but execution meaning is a
    /// matte handed to lower layers. The stencil contribution itself remains in
    /// the SceneValue as an auxiliary source and is suppressed by GPU lowering.
    pub(super) fn hand_out_stencils(&self, layers: &mut [SceneLayerValue]) {
        let mut stencils: Vec<(LayerId, i16, bool, MatteMode, Option<LayerId>, Option<LayerId>)> = layers
            .iter()
            .filter(|layer| layer.blend.is_stencil() && !layer.ghost)
            .filter_map(|layer| {
                let policy = self.layers.get(&layer.layer)?;
                Some((
                    layer.layer,
                    policy.order,
                    layer.clip_to_below,
                    if layer.blend == BlendMode::SilhouetteAlpha { MatteMode::InvertedAlpha } else { MatteMode::Alpha },
                    policy.parent,
                    policy.clip_base,
                ))
            })
            .collect();

        // The closest (lowest order among applicable stencils) wins, matching
        // the legacy resolver's reverse walk plus overwrite.
        stencils.sort_by_key(|stencil| std::cmp::Reverse(stencil.1));
        let mut handed = HashSet::new();

        for (stencil, order, clipped, mode, parent, base) in stencils {
            for index in 0..layers.len() {
                if layers[index].layer == stencil
                    || layers[index].ghost
                    || layers[index].clip_to_below
                    || layers[index].source == LayerSource::Group
                {
                    continue;
                }
                if layers[index].matte.is_some() && !handed.contains(&layers[index].layer) {
                    continue;
                }

                let chain = self.ancestor_chain(layers[index].layer);
                let inside = if clipped {
                    base.is_some_and(|base| chain.contains(&base))
                } else {
                    let top = chain.iter().enumerate().find_map(|(index, id)| {
                        (chain.get(index + 1).copied() == parent).then_some(*id)
                    });
                    top.is_some_and(|top| {
                        top != stencil
                            && self.layers.get(&top).is_some_and(|policy| policy.order < order)
                    })
                };

                if inside {
                    layers[index].matte = Some(Matte { layer: stencil, mode });
                    handed.insert(layers[index].layer);
                }
            }

            // The stencil's clip/matte is only scope information. It must not
            // consume another matte while it acts as the matte source.
            if let Some(layer) = layers.iter_mut().find(|layer| layer.layer == stencil) {
                layer.matte = None;
            }
        }
    }

    fn ancestor_chain(&self, layer: LayerId) -> Vec<LayerId> {
        let mut chain = vec![layer];
        let mut seen = HashSet::from([layer]);
        while let Some(parent) = chain
            .last()
            .and_then(|layer| self.layers.get(layer))
            .and_then(|policy| policy.parent)
        {
            if !seen.insert(parent) || chain.len() > 64 {
                break;
            }
            chain.push(parent);
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::RationalTime;
    use crate::doc::store::{LayerAttrsPatch, LayerMeta, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &crate::frame_graph::GraphNode, inputs: crate::frame_graph::NodeInputs, context: crate::frame_graph::EvaluationContext) -> Result<crate::frame_graph::NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }

    #[test]
    fn stencil_is_handed_to_lower_siblings_and_not_to_higher_ones() {
        let mut doc = Document::new();
        for (id, order) in [(1u64, 0i16), (2, 1), (3, 2), (4, 3)] {
            let layer = LayerId(id);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order, timing: LayerTiming::place(0, None, 90) } },
            ]).unwrap();
        }
        let stencil = LayerId(3);
        doc.apply(Intent::SetAttrs {
            layer: stencil,
            patch: LayerAttrsPatch { blend_mode: Some(BlendMode::StencilAlpha), ..Default::default() },
        }).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let evaluated = graph.evaluate(&mut executor, RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = evaluated.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>()).unwrap();

        assert_eq!(scene.layers.iter().find(|layer| layer.layer == LayerId(1)).unwrap().matte, Some(Matte { layer: stencil, mode: MatteMode::Alpha }));
        assert_eq!(scene.layers.iter().find(|layer| layer.layer == LayerId(2)).unwrap().matte, Some(Matte { layer: stencil, mode: MatteMode::Alpha }));
        assert_eq!(scene.layers.iter().find(|layer| layer.layer == LayerId(4)).unwrap().matte, None);
        assert_eq!(scene.layers.iter().find(|layer| layer.layer == stencil).unwrap().matte, None);
    }
}
