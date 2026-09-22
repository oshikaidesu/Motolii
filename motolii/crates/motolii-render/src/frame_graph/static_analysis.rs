use std::collections::{BTreeMap, BTreeSet};

use super::{
    GraphNode, GraphTopology, InputTime, NodeKey, NodeKind, QualityDependency, TimeDependency,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StaticDomain {
    Property,
    Content,
    Layout,
    Spatial,
    Effect,
    Coverage,
    Analysis,
    Solver,
    Scene,
    View,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StaticDynamicClass {
    None,
    CrossNode,
    TemporalSample,
    Recurrence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StaticClusterSignature {
    pub domain: StaticDomain,
    pub time: TimeDependency,
    pub quality: QualityDependency,
    /// At least one compiler-declared input is sampled at another time.
    pub temporal_edge: bool,
    /// Runtime-added dependency shape. This is deliberately conservative:
    /// static analysis marks capability, not whether a particular value will
    /// actually request an extra input on this frame.
    pub dynamic: StaticDynamicClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StaticClusterId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticCluster {
    pub id: StaticClusterId,
    pub signature: StaticClusterSignature,
    pub nodes: Vec<NodeKey>,
    pub kinds: BTreeSet<NodeKind>,
    pub upstream: BTreeSet<StaticClusterId>,
    pub downstream: BTreeSet<StaticClusterId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticClusterReport {
    pub clusters: Vec<StaticCluster>,
    pub node_cluster: BTreeMap<NodeKey, StaticClusterId>,
    pub absent_builtin_kinds: BTreeSet<NodeKind>,
    pub merge_candidates: Vec<(NodeKey, NodeKey)>,
}

impl StaticClusterReport {
    pub fn cluster_of(&self, node: NodeKey) -> Option<&StaticCluster> {
        let id = self.node_cluster.get(&node)?;
        self.clusters.get(id.0)
    }

    pub fn clusters_with_kind(&self, kind: NodeKind) -> impl Iterator<Item = &StaticCluster> {
        self.clusters.iter().filter(move |cluster| cluster.kinds.contains(&kind))
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::from(
            "| Cluster | Domain | Time | Temporal edge | Dynamic | Quality | Kinds | Nodes | Upstream | Downstream |\n|---:|---|---|---|---|---|---|---:|---|---|\n",
        );
        for cluster in &self.clusters {
            let kinds = cluster.kinds.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>().join(", ");
            let upstream = cluster.upstream.iter().map(|id| id.0.to_string()).collect::<Vec<_>>().join(", ");
            let downstream = cluster.downstream.iter().map(|id| id.0.to_string()).collect::<Vec<_>>().join(", ");
            out.push_str(&format!(
                "| {} | {:?} | {:?} | {} | {} | {:?} | {} | {} | {} | {} |\n",
                cluster.id.0,
                cluster.signature.domain,
                cluster.signature.time,
                cluster.signature.temporal_edge,
                format!("{:?}", cluster.signature.dynamic),
                cluster.signature.quality,
                kinds,
                cluster.nodes.len(),
                upstream,
                downstream,
            ));
        }
        if !self.absent_builtin_kinds.is_empty() {
            out.push_str("\n## Builtin NodeKind absent from this topology\n\n");
            for kind in &self.absent_builtin_kinds {
                out.push_str(&format!("- {kind:?}\n"));
            }
        }
        if !self.merge_candidates.is_empty() {
            out.push_str("\n## Linear merge candidates\n\n");
            for (from, to) in &self.merge_candidates {
                out.push_str(&format!("- {from:?} -> {to:?}\n"));
            }
        }
        out
    }
}

/// Cluster a compiled topology without executing it.
///
/// Nodes with the same scheduling signature are grouped together. Edges are
/// then lifted to cluster-to-cluster edges. This intentionally does not try to
/// predict runtime DynamicInput values; it marks nodes that *may* request them.
pub fn cluster_topology(
    topology: &GraphTopology,
    dynamic_class: impl Fn(&GraphNode) -> StaticDynamicClass,
) -> StaticClusterReport {
    let reachable = topology.reachable();
    let mut grouped: BTreeMap<StaticClusterSignature, Vec<NodeKey>> = BTreeMap::new();

    for key in &reachable {
        let node = topology.node(*key).expect("reachable node");
        let signature = StaticClusterSignature {
            domain: domain(node.identity().kind),
            time: node.identity().time_dependency,
            quality: node.identity().quality_dependency,
            temporal_edge: node.identity().input_times.iter().any(|time| !matches!(time, InputTime::Same)),
            dynamic: dynamic_class(node),
        };
        grouped.entry(signature).or_default().push(*key);
    }

    build_report(topology, reachable, grouped)
}

fn build_report(
    topology: &GraphTopology,
    reachable: Vec<NodeKey>,
    grouped: BTreeMap<StaticClusterSignature, Vec<NodeKey>>,
) -> StaticClusterReport {
    let mut clusters = Vec::with_capacity(grouped.len());
    let mut node_cluster = BTreeMap::new();
    for (signature, nodes) in grouped {
        let id = StaticClusterId(clusters.len());
        let kinds = nodes.iter().filter_map(|key| topology.node(*key))
            .map(|node| node.identity().kind).collect();
        for key in &nodes { node_cluster.insert(*key, id); }
        clusters.push(StaticCluster {
            id,
            signature,
            nodes,
            kinds,
            upstream: BTreeSet::new(),
            downstream: BTreeSet::new(),
        });
    }

    for key in reachable {
        let Some(&to) = node_cluster.get(&key) else { continue };
        let node = topology.node(key).expect("reachable node");
        for input in &node.identity().inputs {
            let Some(&from) = node_cluster.get(input) else { continue };
            if from == to { continue; }
            clusters[to.0].upstream.insert(from);
            clusters[from.0].downstream.insert(to);
        }
    }

    let used: BTreeSet<_> = node_cluster.keys()
        .filter_map(|key| topology.node(*key))
        .map(|node| node.identity().kind)
        .collect();
    let absent_builtin_kinds = NodeKind::BUILTINS.iter().copied()
        .filter(|kind| !used.contains(kind))
        .collect();

    let mut upstream_count: BTreeMap<NodeKey, usize> = BTreeMap::new();
    let mut downstream_count: BTreeMap<NodeKey, usize> = BTreeMap::new();
    for key in node_cluster.keys().copied() {
        let node = topology.node(key).expect("clustered node");
        upstream_count.insert(key, node.identity().inputs.iter().filter(|input| node_cluster.contains_key(input)).count());
        for input in &node.identity().inputs {
            if node_cluster.contains_key(input) {
                *downstream_count.entry(*input).or_default() += 1;
            }
        }
    }
    let roots: BTreeSet<_> = topology.roots().iter().copied().collect();
    let mut merge_candidates = Vec::new();
    for (&to, &to_cluster) in &node_cluster {
        if roots.contains(&to) { continue; }
        let node = topology.node(to).expect("clustered node");
        if upstream_count.get(&to).copied().unwrap_or(0) != 1 { continue; }
        let Some(&from) = node.identity().inputs.iter().find(|input| node_cluster.contains_key(input)) else { continue };
        if downstream_count.get(&from).copied().unwrap_or(0) != 1 { continue; }
        let Some(&from_cluster) = node_cluster.get(&from) else { continue };
        if from_cluster != to_cluster { continue; }
        merge_candidates.push((from, to));
    }

    StaticClusterReport { clusters, node_cluster, absent_builtin_kinds, merge_candidates }
}

/// Conservative static capability map for SceneProgram.
///
/// Some node kinds only request dynamic inputs for a subset of compiled
/// recipes. Marking the entire kind is intentional: this report is a safety
/// tool for finding scheduling boundaries, so false negatives are worse than
/// false positives.
pub fn scene_dynamic_class(kind: NodeKind) -> StaticDynamicClass {
    match kind {
        NodeKind::AnalysisRequest => StaticDynamicClass::CrossNode,
        NodeKind::AnalysisBlob
        | NodeKind::AnalysisOverlay
        | NodeKind::ParticleBirths => StaticDynamicClass::Recurrence,
        NodeKind::PlacementSet
        | NodeKind::MotionMeasure
        | NodeKind::MotionSamples
        | NodeKind::TextShape
        | NodeKind::TextFlow
        | NodeKind::EffectImages
        | NodeKind::Particle
        | NodeKind::CompositeContribution => StaticDynamicClass::TemporalSample,
        _ => StaticDynamicClass::None,
    }
}

pub fn scene_node_may_request_dynamic_inputs(kind: NodeKind) -> bool {
    scene_dynamic_class(kind) != StaticDynamicClass::None
}

pub fn scene_static_clusters(topology: &GraphTopology) -> StaticClusterReport {
    cluster_topology(topology, |node| scene_dynamic_class(node.identity().kind))
}

pub fn domain(kind: NodeKind) -> StaticDomain {
    match kind {
        NodeKind::PropertyConstant
        | NodeKind::PropertyTrack
        | NodeKind::PropertyLink
        | NodeKind::PropertySum
        | NodeKind::Visibility
        | NodeKind::Relation
        | NodeKind::RelationSet => StaticDomain::Property,

        NodeKind::ResolvedWorld
        | NodeKind::TextDocuments
        | NodeKind::ShapeDocuments
        | NodeKind::SharedScene
        | NodeKind::TextContent
        | NodeKind::TextStyle
        | NodeKind::ShapeGeometry
        | NodeKind::ShapeMesh
        | NodeKind::MediaExtent
        | NodeKind::MediaFrame
        | NodeKind::MeshSource
        | NodeKind::Material
        | NodeKind::ParticleBirths
        | NodeKind::Particle => StaticDomain::Content,

        NodeKind::Group
        | NodeKind::Layout
        | NodeKind::FlowWindow
        | NodeKind::GroupBackground
        | NodeKind::TextObstacle
        | NodeKind::TextShape
        | NodeKind::TextFlow => StaticDomain::Layout,

        NodeKind::Transform
        | NodeKind::WorldTransform
        | NodeKind::PlacementSet
        | NodeKind::MotionMeasure
        | NodeKind::MotionSamples
        | NodeKind::TemporalCopy => StaticDomain::Spatial,

        NodeKind::Effect | NodeKind::EffectImages => StaticDomain::Effect,
        NodeKind::Mask => StaticDomain::Coverage,

        NodeKind::AnalysisRequest
        | NodeKind::AnalysisBlob
        | NodeKind::AnalysisOverlay
        | NodeKind::OverlaySet => StaticDomain::Analysis,

        NodeKind::SolverPlan => StaticDomain::Solver,

        NodeKind::CompositeContribution
        | NodeKind::GroupComposite
        | NodeKind::SceneComposite => StaticDomain::Scene,

        NodeKind::DocumentCamera | NodeKind::Camera | NodeKind::CameraProjection => StaticDomain::View,

        NodeKind::Custom(_) => StaticDomain::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::RationalTime;
    use crate::frame_graph::{GraphNode, NodeIdentity};

    fn node(kind: NodeKind, inputs: Vec<NodeKey>) -> GraphNode {
        GraphNode::new(NodeIdentity::new(kind, inputs))
    }

    #[test]
    fn clusters_static_exact_temporal_and_dynamic_boundaries() {
        let property = node(NodeKind::PropertyConstant, vec![]);

        let mut transform_id = NodeIdentity::new(NodeKind::Transform, vec![property.key()]);
        transform_id.time_dependency = TimeDependency::Exact;
        let transform = GraphNode::new(transform_id);

        let mut ghost_id = NodeIdentity::new(NodeKind::TemporalCopy, vec![transform.key()])
            .with_input_times(vec![InputTime::Offset {
                delta: RationalTime::try_new(-1, 30).unwrap(),
                clamp_to_zero: true,
            }]);
        ghost_id.time_dependency = TimeDependency::Exact;
        let ghost = GraphNode::new(ghost_id);

        let mut scene_id = NodeIdentity::new(NodeKind::CompositeContribution, vec![ghost.key()]);
        scene_id.time_dependency = TimeDependency::Exact;
        let scene = GraphNode::new(scene_id);

        let topology = GraphTopology::try_new(
            [property.clone(), transform.clone(), ghost.clone(), scene.clone()],
            vec![scene.key()],
        ).unwrap();

        let report = scene_static_clusters(&topology);
        assert_eq!(report.cluster_of(property.key()).unwrap().signature.domain, StaticDomain::Property);
        assert!(!report.cluster_of(transform.key()).unwrap().signature.temporal_edge);
        assert!(report.cluster_of(ghost.key()).unwrap().signature.temporal_edge);
        assert_eq!(report.cluster_of(scene.key()).unwrap().signature.dynamic, StaticDynamicClass::TemporalSample);
        assert_ne!(report.node_cluster[&ghost.key()], report.node_cluster[&transform.key()]);
    }

    #[test]
    fn audit_reports_absent_kinds_and_only_linear_same_cluster_merge_candidates() {
        let a = node(NodeKind::PropertyConstant, vec![]);
        let b = node(NodeKind::PropertyConstant, vec![a.key()]);
        let cnode = node(NodeKind::PropertyConstant, vec![b.key()]);
        let topology = GraphTopology::try_new([a.clone(), b.clone(), cnode.clone()], vec![cnode.key()]).unwrap();
        let report = cluster_topology(&topology, |_| StaticDynamicClass::None);
                assert!(report.absent_builtin_kinds.contains(&NodeKind::ShapeMesh));
        assert!(report.merge_candidates.contains(&(a.key(), b.key())));
        assert!(!report.merge_candidates.contains(&(b.key(), cnode.key())), "root consumer is not a merge candidate");
    }

    #[test]
    fn markdown_report_is_stable_and_contains_cluster_metadata() {
        let source = node(NodeKind::PropertyConstant, vec![]);
        let mut effect_id = NodeIdentity::new(NodeKind::EffectImages, vec![source.key()]);
        effect_id.time_dependency = TimeDependency::Exact;
        let effect = GraphNode::new(effect_id);
        let topology = GraphTopology::try_new([source, effect.clone()], vec![effect.key()]).unwrap();
        let markdown = scene_static_clusters(&topology).to_markdown();
        assert!(markdown.contains("| Cluster | Domain | Time |"));
        assert!(markdown.contains("Effect"));
        assert!(markdown.contains("TemporalSample"));
    }

    #[test]
    fn mask_and_solver_are_distinct_static_domains() {
        let mask = node(NodeKind::Mask, vec![]);
        let solver = node(NodeKind::SolverPlan, vec![]);
        let scene = node(NodeKind::SceneComposite, vec![mask.key(), solver.key()]);
        let topology = GraphTopology::try_new([mask.clone(), solver.clone(), scene.clone()], vec![scene.key()]).unwrap();
        let report = scene_static_clusters(&topology);
        assert_eq!(report.cluster_of(mask.key()).unwrap().signature.domain, StaticDomain::Coverage);
        assert_eq!(report.cluster_of(solver.key()).unwrap().signature.domain, StaticDomain::Solver);
        assert_ne!(report.node_cluster[&mask.key()], report.node_cluster[&solver.key()]);
    }
}
