//! How evaluated scene contributions combine: clipping groups, track mattes and
//! stencils. This is the only owner of those rules; executors follow the plan.

use std::collections::{HashMap, HashSet};

use crate::doc::store::{LayerId, MatteMode};

use super::SceneValue;

/// A base contribution with the layers clipped onto it (source-atop, in order).
#[derive(Clone, Debug, PartialEq)]
pub struct ClipGroup {
    pub base: usize,
    pub clips: Vec<usize>,
}

/// A matte reads everything its source layer draws: every instance of it.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedMatte {
    pub sources: Vec<PlannedContribution>,
    pub mode: MatteMode,
}

/// One visible scene contribution. Indices address `SceneValue::layers`.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedContribution {
    pub group: ClipGroup,
    pub matte: Option<PlannedMatte>,
}

impl PlannedContribution {
    /// Every scene index this contribution reads, base first.
    pub fn indices(&self) -> Vec<usize> {
        let mut out = vec![self.group.base];
        out.extend(&self.group.clips);
        if let Some(matte) = &self.matte { for source in &matte.sources { out.extend(source.indices()); } }
        out
    }
}

/// Visible contributions in scene order.
pub fn plan_composite(scene: &SceneValue) -> Vec<PlannedContribution> {
    let layers = &scene.layers;
    let by_id: HashMap<LayerId, usize> = layers.iter().enumerate().map(|(index, layer)| (layer.layer, index)).collect();
    let mut instances: HashMap<LayerId, Vec<usize>> = HashMap::new();
    for (index, layer) in layers.iter().enumerate() { instances.entry(layer.layer).or_default().push(index); }

    let mut clips: Vec<Vec<usize>> = vec![Vec::new(); layers.len()];
    for (index, layer) in layers.iter().enumerate() {
        if !layer.clip_to_below || layer.blend.is_stencil() { continue; }
        if let Some(base) = layer.matte.and_then(|matte| by_id.get(&matte.layer)) {
            clips[*base].push(index);
        }
    }

    let matte_sources: HashSet<LayerId> = layers.iter()
        .filter(|layer| !layer.clip_to_below)
        .filter_map(|layer| layer.matte.map(|matte| matte.layer))
        .collect();

    let planned: Vec<Option<PlannedContribution>> = (0..layers.len())
        .map(|index| (!layers[index].clip_to_below).then(|| contribution(scene, &instances, &clips, index, &mut Vec::new())).flatten())
        .collect();

    planned.into_iter().enumerate()
        .filter(|(index, _)| !layers[*index].blend.is_stencil() && !matte_sources.contains(&layers[*index].layer))
        .filter_map(|(_, plan)| plan)
        .collect()
}

/// A matte source is read as it appears, including its own matte. A matte
/// cycle is cut where it closes: that source is read without its matte.
fn contribution(
    scene: &SceneValue,
    instances: &HashMap<LayerId, Vec<usize>>,
    clips: &[Vec<usize>],
    index: usize,
    path: &mut Vec<usize>,
) -> Option<PlannedContribution> {
    let layer = &scene.layers[index];
    let group = ClipGroup { base: index, clips: if layer.clip_to_below { Vec::new() } else { clips[index].clone() } };
    let matte = match layer.matte.filter(|_| !layer.clip_to_below) {
        Some(matte) if !path.contains(&index) => {
            let sources = instances.get(&matte.layer)?;
            path.push(index);
            let planned: Vec<_> = sources.iter().filter_map(|&source| contribution(scene, instances, clips, source, path)).collect();
            path.pop();
            if planned.is_empty() { return None; }
            Some(PlannedMatte { sources: planned, mode: matte.mode })
        }
        _ => None,
    };
    Some(PlannedContribution { group, matte })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{BlendMode, LayerProjection, LayerSource, Matte};
    use crate::frame_graph::{SceneContentValue, SceneLayerValue, TransformValue};

    fn layer(id: u64) -> SceneLayerValue {
        SceneLayerValue {
            layer: LayerId(id), instance: 0, source: LayerSource::Null,
            transform: TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY },
            content_key: None, content: SceneContentValue::None, effects: Vec::new(), after_effects: Vec::new(),
            image_sources: Vec::new(), masks: Vec::new(), matte: None, clip_to_below: false,
            flatten: false, environment: false, ghost: false, freeze_eligible: false,
            timing_start: 0, opacity: 1.0, projection: LayerProjection::TwoD,
            blend: BlendMode::Normal, order: 0, shape_stretch: [1.0, 1.0], depth: 0.0,
        }
    }
    fn matted(id: u64, source: u64) -> SceneLayerValue {
        SceneLayerValue { matte: Some(Matte { layer: LayerId(source), mode: MatteMode::Alpha }), ..layer(id) }
    }
    fn plain(base: usize) -> PlannedContribution {
        PlannedContribution { group: ClipGroup { base, clips: Vec::new() }, matte: None }
    }
    fn masked(target: PlannedContribution, source: PlannedContribution) -> PlannedContribution {
        PlannedContribution { matte: Some(PlannedMatte { sources: vec![source], mode: MatteMode::Alpha }), ..target }
    }

    #[test]
    fn a_matte_source_is_consumed_and_cuts_its_target() {
        let scene = SceneValue { layers: vec![layer(1), matted(2, 3), layer(3)] };
        assert_eq!(plan_composite(&scene), vec![plain(0), masked(plain(1), plain(2))]);
    }

    #[test]
    fn a_missing_matte_source_leaves_no_coverage() {
        let scene = SceneValue { layers: vec![layer(1), matted(2, 9)] };
        assert_eq!(plan_composite(&scene), vec![plain(0)]);
    }

    #[test]
    fn a_matte_source_is_read_with_its_own_matte_wherever_it_sits() {
        let above = SceneValue { layers: vec![matted(1, 2), matted(2, 3), layer(3)] };
        assert_eq!(plan_composite(&above), vec![masked(plain(0), masked(plain(1), plain(2)))]);
        let below = SceneValue { layers: vec![layer(3), matted(2, 3), matted(1, 2)] };
        assert_eq!(plan_composite(&below), vec![masked(plain(2), masked(plain(1), plain(0)))]);
    }

    #[test]
    fn a_matte_cycle_is_cut_where_it_closes() {
        let scene = SceneValue { layers: vec![matted(1, 2), matted(2, 1)] };
        assert_eq!(plan_composite(&scene), Vec::new());
        let seen = contribution(&scene, &[(LayerId(1), vec![0]), (LayerId(2), vec![1])].into(), &[Vec::new(), Vec::new()], 0, &mut Vec::new());
        assert_eq!(seen, Some(masked(plain(0), masked(plain(1), plain(0)))));
    }

    #[test]
    fn clipped_layers_fold_into_their_base_and_stencils_are_never_drawn() {
        let clip = |id| SceneLayerValue { clip_to_below: true, ..matted(id, 1) };
        let stencil = SceneLayerValue { blend: BlendMode::StencilAlpha, ..layer(4) };
        let scene = SceneValue { layers: vec![layer(1), clip(2), clip(3), stencil] };
        assert_eq!(plan_composite(&scene), vec![PlannedContribution { group: ClipGroup { base: 0, clips: vec![1, 2] }, matte: None }]);
    }

    #[test]
    fn a_matte_reads_every_instance_of_its_source() {
        let copy = |instance| SceneLayerValue { instance, ..layer(3) };
        let scene = SceneValue { layers: vec![matted(2, 3), copy(0), copy(1)] };
        let planned = plan_composite(&scene);
        assert_eq!(planned, vec![PlannedContribution { group: ClipGroup { base: 0, clips: Vec::new() }, matte: Some(PlannedMatte { sources: vec![plain(1), plain(2)], mode: MatteMode::Alpha }) }]);
    }
}
