use crate::frame_graph::{
    CanonicalEncoder, SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneProgram,
    SceneValue, TransformValue,
};

// GPU versioning is deliberately value-derived. These helpers are kept here
// rather than in the semantic programs so GPU residency policy cannot leak
// back into semantic evaluation.

use super::lowerer::{GpuContributionInput, GpuEffectImages, GpuImageSourceKind, VersionedImageSource, VersionedSemantic};
use super::types::GpuResourceVersion;

#[derive(Debug)]
pub(crate) enum SemanticAdapterError {
    Encode(crate::frame_graph::CanonicalError),
    MissingContribution(crate::doc::store::LayerId),
    MissingTransform(crate::doc::store::LayerId),
    EffectIdentityMismatch(crate::doc::store::LayerId),
    ImageSourceIdentityMismatch(crate::doc::store::LayerId),
    MaskIdentityMismatch(crate::doc::store::LayerId),
}

impl From<crate::frame_graph::CanonicalError> for SemanticAdapterError {
    fn from(value: crate::frame_graph::CanonicalError) -> Self { Self::Encode(value) }
}

pub(crate) struct SemanticGpuAdapter<'a> {
    program: &'a SceneProgram,
}

impl<'a> SemanticGpuAdapter<'a> {
    pub fn new(program: &'a SceneProgram) -> Self {
        Self { program }
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
            version: placement_version(layer)?,
        };

        let matte_source = layer.matte.and_then(|matte| {
            let source = self.program.contribution(matte.layer)?;
            Some(super::types::GpuResourceIdentity::semantic(
                source,
                super::types::GpuResourceClass::Composite,
                0,
            ).key())
        });

        if layer.effect_keys.len() != layer.effects.len() || layer.after_effect_keys.len() != layer.after_effects.len() {
            return Err(SemanticAdapterError::EffectIdentityMismatch(layer.layer));
        }
        let effects = layer.effect_keys.iter().copied().zip(layer.effects.iter())
            .map(|(node, value)| VersionedSemantic { node, version: resolved_effect_version(value) })
            .collect();
        let after_effects = layer.after_effect_keys.iter().copied().zip(layer.after_effects.iter())
            .map(|(node, value)| VersionedSemantic { node, version: resolved_effect_version(value) })
            .collect();

        if layer.image_source_effects.len() != layer.image_sources.len() {
            return Err(SemanticAdapterError::ImageSourceIdentityMismatch(layer.layer));
        }
        let effect_images = layer.image_source_effects.iter().copied().zip(layer.image_sources.iter())
            .map(|(effect, sources)| {
                let sources = sources.iter().map(|source| {
                    Ok(VersionedImageSource {
                        version: image_source_version(source)?,
                        lifetime: image_source_lifetime(source),
                        kind: match source {
                            SceneImageSourceValue::Content { .. } => GpuImageSourceKind::Content,
                            SceneImageSourceValue::Scene { .. } => GpuImageSourceKind::Scene,
                        },
                    })
                }).collect::<Result<Vec<_>, SemanticAdapterError>>()?;
                Ok(GpuEffectImages { effect, sources })
            }).collect::<Result<Vec<_>, SemanticAdapterError>>()?;

        if layer.mask_keys.len() != layer.masks.len() {
            return Err(SemanticAdapterError::MaskIdentityMismatch(layer.layer));
        }
        let masks = layer.mask_keys.iter().copied().zip(layer.masks.iter())
            .map(|(node, value)| VersionedSemantic { node, version: resolved_mask_version(value) })
            .collect();

        Ok(GpuContributionInput {
            contribution: VersionedSemantic {
                node: contribution,
                version: contribution_version(layer)?,
            },
            instance: layer.instance,
            content,
            placement,
            effects,
            after_effects,
            effect_images,
            masks,
            matte_source,
            plate: matches!(layer.content, SceneContentValue::Plate(_)).then_some(VersionedSemantic {
                node: contribution,
                version: content_version(&layer.content),
            }),
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

fn resolved_effect_version(value: &crate::picture::resolved::ResolvedEffect) -> GpuResourceVersion {
    let mut encoded = CanonicalEncoder::new();
    let bytes = format!("{value:?}");
    let _ = encoded.string(&bytes);
    hash_encoded(encoded)
}

fn placement_version(layer: &SceneLayerValue) -> Result<GpuResourceVersion, SemanticAdapterError> {
    let mut encoded = CanonicalEncoder::new();
    encoded.u64(transform_version(&layer.transform)?.as_u64());
    encoded.f32(layer.opacity)?;
    encoded.i16(layer.order);
    Ok(hash_encoded(encoded))
}

fn image_source_lifetime(value: &SceneImageSourceValue) -> super::types::GpuResourceLifetime {
    let namespace = match value {
        SceneImageSourceValue::Content { namespace, .. } | SceneImageSourceValue::Scene { namespace, .. } => *namespace,
    };
    if namespace == 0 {
        super::types::GpuResourceLifetime::Persistent
    } else {
        // Lookbehind snapshots are immutable and bounded. Feedback recurrence
        // uses History resources instead and is intentionally not modeled here.
        super::types::GpuResourceLifetime::Temporal { retain_generations: 8 }
    }
}

fn image_source_version(value: &SceneImageSourceValue) -> Result<GpuResourceVersion, SemanticAdapterError> {
    let mut encoded = CanonicalEncoder::new();
    match value {
        SceneImageSourceValue::Content { layer, content, time, namespace } => {
            encoded.u8(0).u64(layer.0).rational_time(*time).u64(*namespace);
            encoded.u64(content_version(content).as_u64());
        }
        SceneImageSourceValue::Scene { scene, background, time, namespace } => {
            encoded.u8(1).rational_time(*time).u64(*namespace).u64(scene.layers.len() as u64);
            for component in background { encoded.f32(*component)?; }
            for layer in &scene.layers {
                encoded.u64(layer_visual_version(layer)?.as_u64());
            }
        }
    }
    Ok(hash_encoded(encoded))
}

fn layer_visual_version(layer: &SceneLayerValue) -> Result<GpuResourceVersion, SemanticAdapterError> {
    let mut encoded = CanonicalEncoder::new();
    encoded.u64(layer.layer.0).u32(layer.instance);
    encoded.u64(content_version(&layer.content).as_u64());
    encoded.u64(transform_version(&layer.transform)?.as_u64());
    encoded.f32(layer.opacity)?;
    encoded.i16(layer.order);
    encoded.f32(layer.depth)?;
    for component in layer.shape_stretch { encoded.f32(component)?; }
    encoded.bool(layer.flatten).bool(layer.environment).bool(layer.ghost).bool(layer.clip_to_below);
    let _ = encoded.string(&format!("{:?}", layer.projection));
    let _ = encoded.string(&format!("{:?}", layer.blend));
    let _ = encoded.string(&format!("{:?}", layer.matte));
    for effect in &layer.effects { encoded.u64(resolved_effect_version(effect).as_u64()); }
    for effect in &layer.after_effects { encoded.u64(resolved_effect_version(effect).as_u64()); }
    let _ = encoded.string(&format!("{:?}", layer.masks));
    Ok(hash_encoded(encoded))
}

fn resolved_mask_version(value: &crate::picture::resolved::ResolvedMask) -> GpuResourceVersion {
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
