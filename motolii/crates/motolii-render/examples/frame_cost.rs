//! Saved-document ablation using the preview target and GPU completion boundary.
use motolii_render::{doc::store::*, engine::Engine};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("document path required")?;
    for case in ["no_glass", "no_radiance", "no_text_glass", "no_torus_glass", "all", "no_effects", "no_text", "no_shapes", "no_files", "text_only", "shapes_only", "files_only", "empty"] {
        if std::env::args().nth(2).is_some_and(|wanted| wanted != case) { continue; }
        let mut doc = Document::load(&path)?;
        if std::env::args().nth(3).as_deref() == Some("enable") {
            let ids = doc.view().resolved_layers(RationalTime::ZERO)?.iter().map(|l| l.id).collect::<Vec<_>>();
            for id in ids {
                let effects = doc.view().effects(id)?.to_vec();
                for effect in effects {
                    doc.apply(Intent::SetConstant { layer: id, property: PropertyId::effect_enabled(effect.id), value: Value::Bool(true) })?;
                }
            }
        }
        let comp = doc.view().composition()?.ok_or("composition")?;
        let layers = doc.view().resolved_layers(RationalTime::ZERO)?;
        for l in &layers {
            if case == "all" { eprintln!("layer {:?}: {:?}, effects={:?}", l.id, l.source, l.effects.iter().map(|e| &e.plugin_id).collect::<Vec<_>>()); }
            let text = l.source == LayerSource::Text;
            let shape = l.source == LayerSource::Shape;
            let file = matches!(l.source, LayerSource::File { .. });
            let hide = match case {
                "no_text" => text, "no_shapes" => shape, "no_files" => file,
                "text_only" => shape || file, "shapes_only" => text || file,
                "files_only" => text || shape, "empty" => text || shape || file, _ => false,
            };
            if hide { doc.apply(Intent::SetAttrs { layer: l.id, patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() } })?; }
            if case == "no_effects" { doc.apply(Intent::SetEffects { layer: l.id, effects: vec![] })?; }
            let remove = match case { "no_glass" => Some("motolii.glass"), "no_radiance" => Some("motolii.radiance"), "no_text_glass" if text => Some("motolii.glass"), "no_torus_glass" if file => Some("motolii.glass"), _ => None };
            if let Some(plugin) = remove {
                let effects = doc.view().effects(l.id)?.iter().filter(|e| e.plugin_id != plugin).cloned().collect();
                doc.apply(Intent::SetEffects { layer: l.id, effects })?;
            }
        }
        let mut engine = Engine::new()?;
        engine.set_realtime(true);
        let device = engine.gpu_device().clone();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame-cost"), size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: motolii_render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        for pass in 0..2 {
            let before = engine.surface_work();
            let mut cpu = Vec::new(); let mut wait = Vec::new();
            for frame in (0..150).step_by(5) {
                let t = RationalTime::try_from_frame(frame, comp.fps)?;
                let start = Instant::now();
                engine.render_frame_into(&doc.view(), t, &target)?;
                let submitted = Instant::now();
                device.poll(wgpu::PollType::wait_indefinitely())?;
                cpu.push(submitted.duration_since(start).as_secs_f64() * 1000.0);
                wait.push(submitted.elapsed().as_secs_f64() * 1000.0);
            }
            let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
            println!("{case} pass={pass} cpu_ms={:.3} wait_ms={:.3} total_ms={:.3}", mean(&cpu), mean(&wait), mean(&cpu)+mean(&wait));
            let after = engine.surface_work();
            println!("{case} captures={} main_runs={} mesh_upload_bytes={} draw_prepare_us={}", after.scene_captures-before.scene_captures, after.main_runs-before.main_runs, after.mesh_instance_upload_bytes-before.mesh_instance_upload_bytes, after.draw_data_prepare_us-before.draw_data_prepare_us);
            if !engine.layer_failures().is_empty() { eprintln!("failures: {:?}", engine.layer_failures()); }
        }
    }
    Ok(())
}
