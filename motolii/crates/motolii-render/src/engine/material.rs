use std::sync::Arc;

use crate::doc::core::CompSpec;
use crate::doc::store::{LayerId, LayerProjection, ResolvedLayer};
use crate::render::compositor::{BlendMode, Compositor, CompositorError, EffectStage, GpuModelData, GpuTexture2D, Layer, LayerContent, LayerWithPasses};
use crate::render::compositor::effects::vism::ImageFrame;
use super::{Engine, EngineError};

pub(super) struct MaterialCache {
    source: GpuTexture2D,
    source_frame: i64,
    normalized: GpuTexture2D,
    key: String,
    output: GpuTexture2D,
    frame: ImageFrame,
    mesh: Option<Arc<GpuModelData>>,
}

fn image_layer(texture: GpuTexture2D, size: [f32; 2]) -> Layer {
    Layer {
        content: LayerContent::LinearTexture(texture), size, placement: Default::default(),
        projection: LayerProjection::TwoD, projection_camera: Default::default(),
        blend_mode: BlendMode::Normal, shading: Default::default(), displace: Default::default(),
        clip: None, blocks_light: false, outline: 0, frame: None,
    }
}

impl Compositor {
    fn normalized_material(&mut self, texture: &GpuTexture2D) -> Result<GpuTexture2D, CompositorError> {
        if texture.format().is_srgb() { return Ok(texture.clone()); }
        let source = self.ctx.gpu_resources.textures.get_from_handle(texture.handle()).map_err(|e| CompositorError::Effect(e.to_string()))?.texture.clone();
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("material normalization") });
        let out = self.convert_image_encoding(&mut encoder, &source, true);
        self.pending.push(encoder.finish()); self.flush_pending();
        self.import_premultiplied(&out)
    }

    fn material_plane(&mut self, texture: GpuTexture2D, natural: [f32; 2], frame: ImageFrame) -> Result<Arc<GpuModelData>, CompositorError> {
        let min = glam::Vec2::from(frame.origin);
        let max = min + glam::Vec2::from(frame.size);
        let cells = frame.pixels.map(|n| n.div_ceil(8).clamp(1,256));
        let mesh = re_renderer::textured_plane_grid("material field plane", min, max, cells, texture);
        let vertices = Arc::new(mesh.vertex_positions.clone());
        let instances = re_renderer::CpuModel::from_single_mesh(mesh).into_gpu_meshes(&self.ctx)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        Ok(Arc::new(GpuModelData {
            revision: super::super::compositor::mesh::next_model_revision(),
            planar_size: Some(natural),
            bounds: crate::render::media::SpatialBounds { min: [min.x,min.y,0.0], max: [max.x,max.y,0.0] },
            instances: Arc::new(instances), vertices,
        }))
    }
}

impl Engine {
    pub(super) fn apply_material_domains(&mut self, mut layer: Layer, resolved: &ResolvedLayer, natural: [f32; 2], source_frame: Option<ImageFrame>) -> Result<Layer, EngineError> {
        let mut spatial_seen = false;
        for effect in &resolved.effects {
            match self.compositor.catalog.descriptors.iter().find(|d| d.plugin_id == effect.plugin_id).map(|d| d.stage) {
                Some(EffectStage::Field | EffectStage::Surface) => spatial_seen = true,
                Some(EffectStage::Warp) if spatial_seen => return Err(EngineError::Store("2D warps must precede spatial effects in one material".into())),
                _ => {},
            }
        }
        let warps = super::translate::translate_image_effects(&resolved.effects, EffectStage::Warp);
        let spatial = self.compositor.catalog.descriptors.iter().any(|d| d.stage == EffectStage::Field && resolved.effects.iter().any(|e| e.plugin_id == d.plugin_id));
        if warps.is_empty() && !spatial && source_frame.is_none() { self.materials.remove(&resolved.id); return Ok(layer); }
        let Some(source) = layer.content.texture().cloned() else {
            if !warps.is_empty() { return Err(EngineError::Store("2D warp requires a planar material".into())); }
            self.materials.remove(&resolved.id);
            return Ok(layer);
        };
        let source_tick = if matches!(resolved.source, crate::doc::store::LayerSource::File { .. }) { resolved.source_frame } else { 0 };
        let key = format!("{natural:?}|{warps:?}|{spatial}|{source_tick}|{source_frame:?}");
        let mut cached = self.materials.remove(&resolved.id);
        let same_source = cached.as_ref().is_some_and(|c| c.source.handle() == source.handle() && (!matches!(resolved.source, crate::doc::store::LayerSource::File { .. }) || c.source_frame == resolved.source_frame));
        if !same_source {
            let normalized = self.compositor.normalized_material(&source)?;
            let frame = source_frame.unwrap_or(ImageFrame { size: natural, origin: [0.0;2], pixels: normalized.width_height() });
            cached = Some(MaterialCache { source: source.clone(), source_frame: resolved.source_frame, normalized: normalized.clone(), key: String::new(), output: normalized, frame, mesh: None });
        }
        let mut cached = cached.expect("material source");
        if cached.key != key {
            let mut texture = cached.normalized.clone();
            let mut frame = source_frame.unwrap_or(ImageFrame { size: natural, origin: [0.0;2], pixels: texture.width_height() });
            for pass in warps {
                let input = LayerWithPasses { layer: image_layer(texture, frame.size), passes: vec![pass] };
                let (mut outputs,padding,_spills,_owned_outputs) = self.compositor.effective_layer_textures_in_frame(&[input], Some(frame))?;
                texture = outputs.remove(0).texture().expect("image effect output").clone();
                frame = frame.padded(padding[0]);
            }
            cached.mesh = if spatial { Some(self.compositor.material_plane(texture.clone(), natural, frame)?) } else { None };
            cached.output = texture;
            cached.frame = frame;
            cached.key = key;
        }
        if let Some(mesh) = &cached.mesh {
            layer.content = LayerContent::Model(mesh.clone());
        } else {
            let scale = [layer.size[0]/natural[0].max(1.0), layer.size[1]/natural[1].max(1.0)];
            let offset = glam::vec2(cached.frame.origin[0]*scale[0], cached.frame.origin[1]*scale[1]);
            layer.placement.transform *= glam::Affine2::from_translation(offset);
            if let Some(world) = &mut layer.placement.world_transform { *world *= glam::Affine3A::from_translation(offset.extend(0.0)); }
            layer.size = [cached.frame.size[0]*scale[0],cached.frame.size[1]*scale[1]];
            layer.content = LayerContent::LinearTexture(cached.output.clone());
            layer.frame = Some(cached.frame);
        }
        self.materials.insert(resolved.id, cached);
        Ok(layer)
    }
}

#[cfg(test)]
mod domain_contract {
    use super::*;
    use crate::doc::store::*;
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};

    fn document(source: LayerSource, extent: u32, position: [f64;2]) -> Document {
        let mut doc = blank_project();
        let mut comp = doc.view().composition().unwrap().unwrap();
        comp.width=extent; comp.height=extent; comp.background=[0.0;4];
        doc.apply_all([
            Intent::SetComposition(comp), Intent::AddLayer(LayerId(1)),
            Intent::SetMeta { layer: LayerId(1), meta: LayerMeta { source, order:0, timing:LayerTiming::place(0,None,60) } },
            Intent::SetAttrs { layer: LayerId(1), patch: LayerAttrsPatch { projection:Some(LayerProjection::TwoD), ..Default::default() } },
            Intent::SetConstant { layer:LayerId(1), property:PropertyId::new(property::POSITION).unwrap(), value:Value::Vec2(position) },
        ]).unwrap();
        doc
    }

    fn shape(doc: &mut Document) {
        doc.apply(Intent::SetShapes { layer:LayerId(1), shapes:vec![ShapeNode::Leaf(Shape {
            source:PathSource::Rectangle { size:Point{x:32.0,y:32.0} }, ops:Vec::new(), stroke:None,
            fill:Some(Fill { brush:Brush::Solid(Rgb{r:0.0,g:0.2,b:1.0}), ..Default::default() }),
        })] }).unwrap();
    }

    fn effect(doc: &mut Document, id:u32, plugin:&str, params:&[(&str,f64)]) {
        let mut effects=doc.view().effects(LayerId(1)).unwrap();
        effects.push(EffectInstance{id:EffectId(id),plugin_id:plugin.into()});
        doc.apply(Intent::SetEffects{layer:LayerId(1),effects}).unwrap();
        for (name,v) in params { doc.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::effect_param(EffectId(id),name).unwrap(),value:Value::F64(*v)}).unwrap(); }
    }

    fn differing(a:&[u8],b:&[u8]) -> usize { a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a,b)| a.iter().zip(b.iter()).any(|(a,b)| a.abs_diff(*b)>3)).count() }

    #[test]
    fn the_same_material_warps_the_same_as_pixels_or_paths_and_survives_spatial_placement() {
        let mut engine=Engine::new().unwrap();
        let mut source=document(LayerSource::Shape,34,[0.0,0.0]); shape(&mut source);
        let pixels=engine.render_frame(&source.view(),RationalTime::ZERO).unwrap();
        let dir=tempfile::tempdir().unwrap(); let file=dir.path().join("material.png");
        image::save_buffer(&file,&pixels,34,34,image::ColorType::Rgba8).unwrap();
        for projection in [LayerProjection::TwoD,LayerProjection::TwoPointFiveD,LayerProjection::ThreeD] {
            let mut path=document(LayerSource::Shape,128,[40.0,40.0]); shape(&mut path);
            let mut image=document(LayerSource::File{path:file.to_string_lossy().into_owned(),fingerprint:None},128,[40.0,40.0]);
            for doc in [&mut path,&mut image] {
                doc.apply(Intent::SetAttrs{layer:LayerId(1),patch:LayerAttrsPatch{projection:Some(projection),..Default::default()}}).unwrap();
                effect(doc,0,"motolii.turbulent_warp",&[("amount",6.0),("size",20.0),("evolution",0.7)]);
            }
            let a=engine.render_frame(&path.view(),RationalTime::ZERO).unwrap();
            let b=engine.render_frame(&image.view(),RationalTime::ZERO).unwrap();
            assert!(differing(&a,&b)<30,"source representation changed the warp ({projection:?}): {} pixels",differing(&a,&b));
            image.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::new(property::POSITION).unwrap(),value:Value::Vec2([52.0,40.0])}).unwrap();
            let shifted=engine.render_frame(&image.view(),RationalTime::ZERO).unwrap();
            let mut moved=0;
            for y in 0..128 { for x in 0..116 { let i=(y*128+x)*4; let j=(y*128+x+12)*4; if b[i..i+4].iter().zip(&shifted[j..j+4]).any(|(a,b)|a.abs_diff(*b)>3){moved+=1;} } }
            assert!(moved<30,"placement changed material-local warp: {moved}");
        }
    }

    #[test]
    fn logical_warp_amount_is_independent_of_raster_density() {
        let mut compositor=Compositor::headless().unwrap();
        let mut rendered=Vec::new();
        for density in [1u32,2] {
            let extent=64*density;
            let mut bytes=Vec::new();
            for y in 0..extent { for x in 0..extent {
                bytes.extend_from_slice(&[((x as f32+0.5)/extent as f32*255.0) as u8,((y as f32+0.5)/extent as f32*255.0) as u8,128,255]);
            } }
            let raw=compositor.cached_rgba(density as u64,"density contract",||Ok::<_,std::convert::Infallible>((bytes,extent,extent))).unwrap();
            let source=compositor.normalized_material(&raw).unwrap();
            let effect=ResolvedEffect{plugin_id:"motolii.turbulent_warp".into(),params:vec![("amount".into(),Value::F64(6.0)),("size".into(),Value::F64(20.0))],..Default::default()};
            let passes=super::super::translate::translate_image_effects(&[effect],EffectStage::Warp);
            let frame=ImageFrame{size:[64.0;2],origin:[0.0;2],pixels:[extent;2]};
            let input=LayerWithPasses{layer:image_layer(source,frame.size),passes};
            let (mut output,padding,_spills,_owned)=compositor.effective_layer_textures_in_frame(&[input],Some(frame)).unwrap();
            let frame=frame.padded(padding[0]);
            let texture=output.remove(0).texture().unwrap().clone();
            let layer=LayerWithPasses{layer:image_layer(texture,frame.size),passes:Vec::new()};
            rendered.push(compositor.render_with_effects(CompSpec{width:76,height:76},Default::default(),&[layer],[0.0;4]).unwrap());
        }
        let mut total=0usize;let mut error=0usize;
        for (a,b) in rendered[0].chunks_exact(4).zip(rendered[1].chunks_exact(4)) {
            if a[3]>250 && b[3]>250 {total+=3;error+=a[..3].iter().zip(&b[..3]).map(|(a,b)|a.abs_diff(*b) as usize).sum::<usize>();}
        }
        assert!(total>6000 && error<total*2,"pixel density changed the material-local warp: {error}/{total}");
    }

    /// 溢れの法: Glow の halo(coverage の外)は層の Blend が Normal でも screen で下へ乗る。
    /// 白の上では白のまま(screen は白を変えない)、黒の上では光る。Normal の over なら白が halo の色に濁る。
    #[test]
    fn a_glow_halo_spills_as_light_independent_of_the_layer_blend() {
        let scene = |background: [f32; 4]| {
            let mut doc = document(LayerSource::Shape, 96, [48.0, 48.0]);
            let mut comp = doc.view().composition().unwrap().unwrap();
            comp.background = background;
            doc.apply(Intent::SetComposition(comp)).unwrap();
            shape(&mut doc);
            effect(&mut doc, 0, "motolii.glow", &[("threshold", 0.0), ("intensity", 3.0), ("radius", 6.0)]);
            doc
        };
        let mut engine = Engine::new().unwrap();
        let on_white = engine.render_frame(&scene([1.0; 4]).view(), RationalTime::ZERO).unwrap();
        let on_black = engine.render_frame(&scene([0.0, 0.0, 0.0, 1.0]).view(), RationalTime::ZERO).unwrap();
        let plain = engine.render_frame(&{ let mut d = document(LayerSource::Shape, 96, [48.0, 48.0]); shape(&mut d); d }.view(), RationalTime::ZERO).unwrap();
        let mut tinted_white = 0;
        let mut lit_black = 0;
        for (i, px) in plain.chunks_exact(4).enumerate() {
            if px[3] != 0 { continue; } // 素材の中は見ない
            let w = &on_white[i * 4..i * 4 + 4];
            if w[0] < 250 || w[1] < 250 || w[2] < 250 { tinted_white += 1; }
            let b = &on_black[i * 4..i * 4 + 4];
            if b[2] > 24 { lit_black += 1; }
        }
        assert!(lit_black > 200, "the halo must light the black background: {lit_black} pixels");
        assert!(tinted_white < 20, "a Normal layer's halo must screen over white, not tint it: {tinted_white} pixels");
    }

    #[test]
    fn neutral_warp_is_identity_and_evolution_and_seed_have_distinct_results() {
        let mut doc=document(LayerSource::Shape,128,[40.0,40.0]);shape(&mut doc);
        let mut engine=Engine::new().unwrap();let plain=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        effect(&mut doc,0,"motolii.turbulent_warp",&[("amount",0.0),("size",20.0)]);
        let zero=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        assert!(differing(&plain,&zero)<20,"neutral warp changed the material");
        let mut results=Vec::new();
        for (evolution,seed) in [(0.0,0.0),(1.0,0.0),(0.0,3.0)] {
            for (name,value) in [("amount",6.0),("evolution",evolution),("seed",seed)] {
                doc.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::effect_param(EffectId(0),name).unwrap(),value:Value::F64(value)}).unwrap();
            }
            results.push(engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap());
        }
        assert!(differing(&results[0],&results[1])>30 && differing(&results[0],&results[2])>30);
    }

    #[test]
    fn two_dimensional_warp_and_spatial_displacement_can_be_stacked() {
        let mut doc=document(LayerSource::Shape,128,[40.0,40.0]); shape(&mut doc);
        effect(&mut doc,0,"motolii.turbulent_warp",&[("amount",6.0),("size",20.0)]);
        let mut engine=Engine::new().unwrap();
        let warp=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        effect(&mut doc,1,"motolii.turbulent_displace",&[("amount",0.0),("size",20.0),("along",1.0)]);
        let neutral=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        if let Some(dir)=std::env::var_os("MOTOLII_DOMAIN_EVIDENCE") {
            let dir=std::path::PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();
            image::save_buffer(dir.join("warp.png"),&warp,128,128,image::ColorType::Rgba8).unwrap();
            image::save_buffer(dir.join("neutral.png"),&neutral,128,128,image::ColorType::Rgba8).unwrap();
        }
        assert!(differing(&warp,&neutral)<40,"zero spatial displacement changed the material: {} pixels",differing(&warp,&neutral));
        doc.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::effect_param(EffectId(1),"amount").unwrap(),value:Value::F64(20.0)}).unwrap();
        let spatial=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        assert!(differing(&neutral,&spatial)>50,"spatial field must deform the warped plane");
        assert!(engine.materials[&LayerId(1)].mesh.is_some(),"a spatial field must receive geometry");
        effect(&mut doc,2,"motolii.gain",&[("gain",2.0)]);
        let bright=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
        assert!(differing(&spatial,&bright)>50,"an image pass after the spatial plane must not be ignored");
        let mut effects=doc.view().effects(LayerId(1)).unwrap();effects.retain(|e|e.id!=EffectId(2));
        doc.apply(Intent::SetEffects{layer:LayerId(1),effects}).unwrap();
        if let Some(dir)=std::env::var_os("MOTOLII_DOMAIN_EVIDENCE") {
            let dir=std::path::PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();
            let mut comp=doc.view().composition().unwrap().unwrap();comp.width=512;comp.height=512;comp.background=[0.025,0.03,0.04,1.0];
            doc.apply_all([
                Intent::SetComposition(comp),
                Intent::SetConstant{layer:LayerId(1),property:PropertyId::new(property::POSITION).unwrap(),value:Value::Vec2([188.0,188.0])},
                Intent::SetConstant{layer:LayerId(1),property:PropertyId::new(property::SCALE).unwrap(),value:Value::Vec2([4.0,4.0])},
                Intent::SetAttrs{layer:LayerId(1),patch:LayerAttrsPatch{name:Some("2D warp + 3D displacement".into()),projection:Some(LayerProjection::TwoPointFiveD),..Default::default()}},
            ]).unwrap();
            doc.save(dir.join("warp-and-field.rrd")).unwrap();
            let pixels=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
            image::save_buffer(dir.join("warp-and-field.png"),&pixels,512,512,image::ColorType::Rgba8).unwrap();
            doc.apply(Intent::SetConstant{layer:LayerId(1),property:PropertyId::effect_param(EffectId(1),"amount").unwrap(),value:Value::F64(0.0)}).unwrap();
            doc.save(dir.join("warp-only.rrd")).unwrap();
            let pixels=engine.render_frame(&doc.view(),RationalTime::ZERO).unwrap();
            image::save_buffer(dir.join("warp-only.png"),&pixels,512,512,image::ColorType::Rgba8).unwrap();
        }

    }
}
