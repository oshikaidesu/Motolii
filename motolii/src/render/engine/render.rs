
use std::collections::{HashMap, HashSet};

use crate::render::compositor::{Layer, LayerWithPasses};
use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_effect_passes, translate_matte_mode,
};
use crate::render::engine::{Engine, EngineError};

impl Engine {
    pub(crate) fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        self.layer_failures.clear();
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let camera = match camera_override {
            Some(camera) => camera,
            None => view
                .resolve_camera(t)
                .map_err(|e| EngineError::Store(e.to_string()))?,
        };
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
        let matte_sources: HashSet<LayerId> = resolved
            .iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        for layer in &resolved {
            if matte_sources.contains(&layer.id) {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let passes = translate_effect_passes(&layer.effects);

            let (content, natural) = self.texture_for_layer(view, layer, t, comp)?;
            let Some(content) = content else {
                continue;
            };
            let built = self.flatten_if_asked(comp, camera, Layer {
                content,
                size: layer_size(layer, natural),
                placement: layer.placement,
                pinned: layer.pinned,
                blend_mode,
            }, layer.flatten)?;

            let final_layer = match layer.matte {
                None => built,
                Some(matte) => {
                    let Some(source) = by_id.get(&matte.layer).copied() else {
                        continue;
                    };
                    let (source_content, source_natural) =
                        self.texture_for_layer(view, source, t, comp)?;
                    let Some(source_content) = source_content else {
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let source_layer = Layer {
                        content: source_content,
                        size: layer_size(source, source_natural),
                        placement: source.placement,
                        pinned: source.pinned,
                        blend_mode: source_blend,
                    };
                    self.apply_matte(comp, camera, &built, &source_layer, matte.mode)?
                }
            };

            layers.push(LayerWithPasses {
                layer: final_layer,
                passes,
            });
        }

        let background_color = if include_background {
            composition.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        Ok(self
            .compositor
            .render_with_effects(comp, camera, &layers, background_color)?)
    }

    fn layers_from_resolved(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        self.layer_failures.clear();
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        let by_id: HashMap<LayerId, &ResolvedLayer> =
            resolved.iter().map(|layer| (layer.id, layer)).collect();
        let matte_sources: HashSet<LayerId> = resolved
            .iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();

        for layer in resolved {
            if matte_sources.contains(&layer.id) {
                continue;
            }

            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let passes = translate_effect_passes(&layer.effects);

            let (content, natural) =
                self.texture_for_resolved(layer, text_documents, shape_documents, t, comp)?;
            let Some(content) = content else {
                continue;
            };
            let built = self.flatten_if_asked(comp, camera, Layer {
                content,
                size: layer_size(layer, natural),
                placement: layer.placement,
                pinned: layer.pinned,
                blend_mode,
            }, layer.flatten)?;

            let final_layer = match layer.matte {
                None => built,
                Some(matte) => {
                    let Some(source) = by_id.get(&matte.layer).copied() else {
                        continue;
                    };
                    let (source_content, source_natural) = self.texture_for_resolved(
                        source,
                        text_documents,
                        shape_documents,
                        t,
                        comp,
                    )?;
                    let Some(source_content) = source_content else {
                        continue;
                    };
                    let source_blend = translate_blend_mode(source.blend_mode)?;
                    let source_layer = Layer {
                        content: source_content,
                        size: layer_size(source, source_natural),
                        placement: source.placement,
                        pinned: source.pinned,
                        blend_mode: source_blend,
                    };
                    self.apply_matte(comp, camera, &built, &source_layer, matte.mode)?
                }
            };

            layers.push(LayerWithPasses {
                layer: final_layer,
                passes,
            });
        }

        Ok(layers)
    }

    pub fn render_resolved_to_texture(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_resolved_to_texture_with_shapes(
            comp,
            background,
            camera,
            t,
            resolved,
            text_documents,
            &HashMap::new(),
        )
    }

    pub fn render_resolved_to_texture_with_shapes(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        t: RationalTime,
        resolved: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers =
            self.layers_from_resolved(comp, camera, t, resolved, text_documents, shape_documents)?;
        Ok(self
            .compositor
            .render_to_texture(comp, camera, &layers, background)?)
    }

    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let camera = view
            .resolve_camera(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved)?;
        self.render_resolved_to_texture_with_shapes(
            comp,
            composition.background,
            camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )
    }

    pub fn render_frame_into(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
    ) -> Result<(), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let camera = view
            .resolve_camera(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved)?;
        let layers = self.layers_from_resolved(
            comp,
            camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;
        Ok(self.compositor.render_into(
            target,
            comp,
            camera,
            &layers,
            composition.background,
        )?)
    }

    /// 指定したカメラで撮る。**窓の視点はここへ来ない** —— 視点は撮れた絵を
    /// 2Dで動かすだけなので、撮るのは常に書き出しのカメラ(裁定 2026-09-01)。
    pub fn render_frame_into_with_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
    ) -> Result<(), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let text_documents = collect_text_documents(view, &resolved, t)?;
        let shape_documents = collect_shape_documents(view, &resolved)?;
        let layers = self.layers_from_resolved(
            comp,
            camera,
            t,
            &resolved,
            &text_documents,
            &shape_documents,
        )?;
        let background_color = if include_background {
            composition.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        Ok(self
            .compositor
            .render_into(target, comp, camera, &layers, background_color)?)
    }

    /// 平面へ収める。3D の素材を comp の絵へ一度焼き、以後は板として扱う
    /// (裁定 2026-08-30「平面に収めるのは選択肢」)。焼いた層にも blend・matte・
    /// エフェクトは今まで通り効く。
    fn flatten_if_asked(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        flatten: bool,
    ) -> Result<Layer, EngineError> {
        if !flatten || layer.content.texture().is_some() {
            return Ok(layer);
        }
        let mut baked_placement = layer.placement;
        baked_placement.opacity = 1.0;
        let source = LayerWithPasses {
            layer: Layer { placement: baked_placement, ..layer.clone() },
            passes: Vec::new(),
        };
        let (texture, _view) = self.compositor.render_to_texture(
            comp,
            camera,
            std::slice::from_ref(&source),
            crate::render::compositor::NO_BACKGROUND,
        )?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Layer {
            content: crate::render::compositor::LayerContent::Texture(imported),
            size: [comp.width as f32, comp.height as f32],
            // 焼いた絵は既に comp の座標に居るので、もう一度動かさない。
            placement: crate::doc::core::LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                ..layer.placement
            },
            pinned: true,
            blend_mode: layer.blend_mode,
        })
    }

    pub fn apply_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        target: &Layer,
        matte_source: &Layer,
        mode: crate::doc::store::MatteMode,
    ) -> Result<Layer, EngineError> {
        Ok(self.compositor.matte_layer(
            comp,
            camera,
            target,
            matte_source,
            translate_matte_mode(mode),
        )?)
    }
}

fn collect_text_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
    t: RationalTime,
) -> Result<HashMap<LayerId, TextDocument>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Text {
            if let Some(document) = view
                .resolved_text_document(layer.id, t)
                .map_err(|e| EngineError::Store(e.to_string()))?
            {
                documents.insert(layer.id, document);
            }
        }
    }
    Ok(documents)
}

fn collect_shape_documents(
    view: &StoreView<'_>,
    resolved: &[ResolvedLayer],
) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = view
                .shapes(layer.id)
                .map_err(|e| EngineError::Store(e.to_string()))?;
            documents.insert(layer.id, shapes);
        }
    }
    Ok(documents)
}

pub(crate) fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 {
            layer.declared_size[0]
        } else {
            natural[0]
        },
        if layer.declared_size[1] > 0.0 {
            layer.declared_size[1]
        } else {
            natural[1]
        },
    ]
}
