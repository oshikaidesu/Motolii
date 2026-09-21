use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::{Engine, EngineError};

#[derive(Clone)]
pub(super) struct GpuSceneValue { pub layers: Vec<LayerWithPasses> }

impl Engine {
    pub(super) fn prepare_gpu_scene(&mut self, scene: &SceneValue, comp: CompSpec, projection_camera: ResolvedCamera) -> Result<GpuSceneValue, EngineError> {
        #[derive(Clone, Copy)]
        struct Entry {
            layer: LayerId,
            matte: Option<crate::doc::store::Matte>,
            clip_to_below: bool,
            stencil: bool,
        }

        let clip_bases: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|layer| layer.clip_to_below)
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        let mut layers = Vec::with_capacity(scene.layers.len());
        let mut entries = Vec::with_capacity(scene.layers.len());
        for source in &scene.layers {
            let key = LayerId(source.content_key.map_or(0, |key| key.as_u64()));
            let force_picture = clip_bases.contains(&source.layer);
            let (content, natural) = match &source.content {
                SceneContentValue::None => continue,
                SceneContentValue::Text(text) if force_picture => self.shape_texture_from_shapes(&text.shapes(), key, false, 0.05, comp, None, true)?,
                SceneContentValue::Text(text) => self.text_texture_from_shapes(&text.shapes(), key, comp)?,
                SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(shapes, key, !force_picture, 0.05, comp, None, true)?,
                SceneContentValue::Material(material) => self.mesh_content_for(&material.source.path, comp)?,
                SceneContentValue::Media { source: media, time } => {
                    if source.environment && crate::render::media::is_still_image_path(&media.path) {
                        self.environment_content_for(&media.path)?
                    } else {
                        self.file_content_for(&media.path, *time, source.layer, comp)?
                    }
                }
            };
            let Some(content) = content else { continue };
            let placement = LayerPlacement {
                transform: source.transform.affine,
                world_transform: Some(source.transform.spatial),
                order: i32::from(source.order),
                opacity: source.opacity,
                z: source.transform.spatial.translation.z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            };
            let passes: Vec<_> = crate::render::engine::translate::translate_effect_passes(&source.effects)
                .into_iter()
                .chain(crate::render::engine::translate::translate_plate_passes(&source.after_effects))
                .collect();
            // Stencil/Silhouette are matte sources, never compositor blend modes.
            let blend_mode = if source.blend.is_stencil() {
                CompositeBlendMode::Normal
            } else {
                crate::render::engine::translate::translate_blend_mode(source.blend)?
            };
            let layer = Layer {
                content,
                size: natural,
                placement,
                projection: source.projection,
                projection_camera,
                blend_mode,
                shading: self.compositor.surface_shading_for(&source.effects, false).map_err(EngineError::Store)?,
                displace: crate::render::engine::translate::translate_point_displace(&source.effects),
                clip: crate::render::engine::translate::translate_clip(&source.effects),
                shadow: crate::render::engine::translate::translate_cast_shadow(&source.effects),
                outline: self.outline_id(source.layer),
                frame: None,
            };
            let layer = self.apply_masks_to_layer(layer, &source.masks, natural, None)?;
            let layer = self.flatten_if_asked(comp, projection_camera, layer, source.flatten)?;
            layers.push(LayerWithPasses { layer, passes, padding: 0, pass_sources: Vec::new(), cut: Vec::new() });
            entries.push(Entry {
                layer: source.layer,
                matte: source.matte,
                clip_to_below: source.clip_to_below,
                stencil: source.blend.is_stencil(),
            });
        }

        let by_id: std::collections::HashMap<_, _> = entries.iter().enumerate()
            .map(|(index, entry)| (entry.layer, index))
            .collect();
        let mut removed = vec![false; layers.len()];

        // Clipping is source-atop onto the visible base. The upper contribution
        // never reaches the final scene as a separate layer.
        for index in 0..layers.len() {
            let entry = entries[index];
            if !entry.clip_to_below || entry.stencil {
                continue;
            }
            let Some(base_id) = entry.matte.map(|matte| matte.layer) else {
                removed[index] = true;
                continue;
            };
            let Some(base_index) = by_id.get(&base_id).copied() else {
                removed[index] = true;
                continue;
            };
            match self.clip_onto_base(layers[base_index].clone(), &layers[index].layer, &layers[index].passes)? {
                Some(clipped) => layers[base_index] = clipped,
                None => self.layer_failures.push(format!(
                    "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                    entry.layer.0
                )),
            }
            removed[index] = true;
        }

        // Track mattes and stencils consume their source. A missing source means
        // the target has no coverage and therefore contributes nothing.
        let matte_sources: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|entry| !entry.clip_to_below)
            .filter_map(|entry| entry.matte.map(|matte| matte.layer))
            .collect();

        for index in 0..layers.len() {
            if removed[index] || entries[index].clip_to_below {
                continue;
            }
            let Some(matte) = entries[index].matte else { continue };
            let Some(source_index) = by_id.get(&matte.layer).copied() else {
                removed[index] = true;
                continue;
            };
            let target = self.apply_effects_before_matte(
                comp,
                projection_camera,
                layers[index].layer.clone(),
                &layers[index].passes,
            )?;
            let source = self.apply_effects_before_matte(
                comp,
                projection_camera,
                layers[source_index].layer.clone(),
                &layers[source_index].passes,
            )?;
            let layer = self.apply_matte(comp, projection_camera, &target, &source, matte.mode)?;
            layers[index] = LayerWithPasses {
                layer,
                passes: Vec::new(),
                padding: 0,
                pass_sources: Vec::new(),
                cut: Vec::new(),
            };
        }

        let layers = layers.into_iter().enumerate()
            .filter(|(index, _)| {
                !removed[*index]
                    && !entries[*index].stencil
                    && !matte_sources.contains(&entries[*index].layer)
            })
            .map(|(_, layer)| layer)
            .collect();

        Ok(GpuSceneValue { layers })
    }
}
