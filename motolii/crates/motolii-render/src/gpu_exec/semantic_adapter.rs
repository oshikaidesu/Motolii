use crate::frame_graph::{
    CanonicalEncoder, EffectValue, EvaluatedFrame, SceneContentValue, SceneLayerValue, SceneProgram,
    SceneValue, TransformValue,
};

// GPU versioning is deliberately value-derived. These helpers are kept here
// rather than in the semantic programs so GPU residency policy cannot leak
// back into semantic evaluation.

use super::lowerer::{GpuContributionInput, VersionedSemantic};
use super::types::GpuResourceVersion;

#[derive(Debug)]
pub(crate) enum SemanticAdapterError {
    Encode(crate::frame_graph::CanonicalError),
    MissingContribution(crate::doc::store::LayerId),
    MissingTransform(crate::doc::store::LayerId),
}

impl From<crate::frame_graph::CanonicalError> for SemanticAdapterError {
    fn from(value: crate::frame_graph::CanonicalError) -> Self { Self::Encode(value) }
}

pub(crate) struct SemanticGpuAdapter<'a> {
    program: &'a SceneProgram,
    frame: &'a EvaluatedFrame,
}

impl<'a> SemanticGpuAdapter<'a> {
    pub fn new(program: &'a SceneProgram, frame: &'a EvaluatedFrame) -> Self {
        Self { program, frame }
    }

    pub fn contributions(
        &self,
        scene: &SceneValue,
    ) -> Result<Vec<GpuContributionInput>, SemanticAdapterError> {
        scene.layers.iter().map(|layer| self.contribution(layer)).collect()
    }

    pub fn contribution(
        &self,
        layer: &SceneLayerValue,
    ) -> Result<GpuContributionInput, SemanticAdapterError> {
        let contribution = self.program.contribution(layer.layer)
            .ok_or(SemanticAdapterError::MissingContribution(layer.layer))?;
        let transform = self.program.transforms().binding(layer.layer)
            .ok_or(SemanticAdapterError::MissingTransform(layer.layer))?;

        let content_node = self.program.content().binding(layer.layer)
            .and_then(|binding| binding.content)
            .or(layer.content_key);
        let content = content_node.map(|node| VersionedSemantic {
            node,
            version: content_version(&layer.content),
        });

        let placement = VersionedSemantic {
            node: transform.world,
            version: transform_version(&layer.transform)?,
        };

        let effects = self.program.effects().binding(layer.layer)
            .map(|binding| binding.effects.iter().filter_map(|node| {
                let value = self.frame.value(*node)?.downcast_ref::<EffectValue>()?;
                Some(VersionedSemantic { node: *node, version: effect_version(value) })
            }).collect())
            .unwrap_or_default();

        let masks = self.program.masks().binding(layer.layer)
            .map(|binding| binding.masks.iter().filter_map(|node| {
                let value = self.frame.value(*node)?.downcast_ref::<crate::frame_graph::MaskValue>()?;
                Some(VersionedSemantic { node: *node, version: mask_version(value) })
            }).collect())
            .unwrap_or_default();

        Ok(GpuContributionInput {
            contribution: VersionedSemantic {
                node: contribution,
                version: contribution_version(layer)?,
            },
            instance: layer.instance,
            content,
            placement,
            effects,
            masks,
            matte_source: None,
            plate: None,
        })
    }
}

fn hash_encoded(encoded: CanonicalEncoder) -> GpuResourceVersion {
    GpuResourceVersion::from_canonical(&encoded)
}

fn transform_version(value: &TransformValue) -> Result<GpuResourceVersion, SemanticAdapterError> {
    let mut encoded = CanonicalEncoder::new();
    for component in value.affine.matrix2.to_cols_array() { encoded.f32(component)?; }
    for component in value.affine.translation.to_array() { encoded.f32(component)?; }
    for component in value.spatial.matrix3.to_cols_array() { encoded.f32(component)?; }
    for component in value.spatial.translation.to_array() { encoded.f32(component)?; }
    Ok(hash_encoded(encoded))
}

fn content_version(value: &SceneContentValue) -> GpuResourceVersion {
    let mut encoded = CanonicalEncoder::new();
    match value {
        SceneContentValue::None => { encoded.u8(0); }
        SceneContentValue::Text(text) => {
            encoded.u8(1);
            let bytes = format!("{text:?}");
            let _ = encoded.string(&bytes);
        }
        SceneContentValue::Shape(shape) => {
            encoded.u8(2);
            let bytes = serde_json::to_vec(shape).unwrap_or_default();
            let _ = encoded.bytes(&bytes);
        }
        SceneContentValue::Material(material) => {
            encoded.u8(3).u64(material.source.version);
            let _ = encoded.string(&material.source.path);
        }
        SceneContentValue::Media { source, time } => {
            encoded.u8(4).u64(source.version).rational_time(*time);
            let _ = encoded.string(&source.path);
        }
        SceneContentValue::Particles(particles) => {
            encoded.u8(5);
            let bytes = format!("{particles:?}");
            let _ = encoded.string(&bytes);
        }
        SceneContentValue::Plate(plate) => {
            encoded.u8(6).bool(plate.average);
            encoded.u64(plate.owner.map_or(0, |id| id.0));
            encoded.u64(plate.members.len() as u64);
        }
    }
    hash_encoded(encoded)
}

fn effect_version(value: &EffectValue) -> GpuResourceVersion {
    let mut encoded = CanonicalEncoder::new();
    let bytes = format!("{value:?}");
    let _ = encoded.string(&bytes);
    hash_encoded(encoded)
}

fn mask_version(value: &crate::frame_graph::MaskValue) -> GpuResourceVersion {
    let mut encoded = CanonicalEncoder::new();
    let bytes = format!("{value:?}");
    let _ = encoded.string(&bytes);
    hash_encoded(encoded)
}

fn contribution_version(layer: &SceneLayerValue) -> Result<GpuResourceVersion, SemanticAdapterError> {
    let mut encoded = CanonicalEncoder::new();
    encoded.u64(layer.layer.0).u32(layer.instance);
    encoded.u64(content_version(&layer.content).as_u64());
    encoded.u64(transform_version(&layer.transform)?.as_u64());
    encoded.f32(layer.opacity)?;
    encoded.i16(layer.order);
    encoded.f32(layer.depth)?;
    encoded.bool(layer.flatten).bool(layer.environment).bool(layer.ghost);
    Ok(hash_encoded(encoded))
}
