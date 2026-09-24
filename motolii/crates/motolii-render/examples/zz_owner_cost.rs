//! Ablation by layer name on a saved document: hide the layers whose name contains each argument,
//! render the same frames through the production route, report GPU wait per frame.
//! `zz_owner_cost <doc> [name,name...] [no_effects]`
use motolii_edit::{Document, Intent};
use motolii_render::{doc::store::*, engine::Engine};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("document path required")?;
    let hide: Vec<String> = std::env::args().nth(2).map(|s| s.split(',').filter(|s| !s.is_empty()).map(str::to_owned).collect()).unwrap_or_default();
    let mode = std::env::args().nth(3).unwrap_or_default();
    let strip = mode == "no_effects";
    let drop: Vec<String> = mode.strip_prefix("drop=").map(|s| s.split(',').map(str::to_owned).collect()).unwrap_or_default();
    let mut doc = Document::load(&path)?.with_programs(motolii_render::extensions::bundled());
    let comp = doc.view().composition()?.ok_or("composition")?;
    let mut hidden = Vec::new();
    for id in doc.view().layers() {
        let name = doc.view().attrs(id)?.map(|a| a.name.clone()).unwrap_or_default();
        if hide.iter().any(|h| name.contains(h.as_str())) {
            doc.apply(Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() } })?;
            hidden.push(name);
        } else if strip {
            doc.apply(Intent::SetEffects { layer: id, effects: vec![] })?;
        } else if !drop.is_empty() {
            let effects: Vec<_> = doc.view().effects(id)?.iter().filter(|e| !drop.iter().any(|d| e.plugin_id.contains(d.as_str()))).cloned().collect();
            doc.apply(Intent::SetEffects { layer: id, effects })?;
        }
    }
    let mut engine = Engine::new()?;
    engine.set_realtime(true);
    let device = engine.gpu_device().clone();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("owner-cost"), size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: motolii_render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let mut wait = Vec::new();
    for pass in 0..2 {
        let before = engine.surface_work();
        wait.clear();
        for frame in (0..300).step_by(10) {
            let t = RationalTime::try_from_frame(frame, comp.fps)?;
            engine.render_frame_into(&doc.view(), t, &target)?;
            let submitted = Instant::now();
            device.poll(wgpu::PollType::wait_indefinitely())?;
            wait.push(submitted.elapsed().as_secs_f64() * 1000.0);
        }
        if pass == 1 {
            let after = engine.surface_work();
            wait.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("hide={:?} mode={mode} gpu_wait_median_ms={:.2} layer_views={} main_runs={} backdrop_copies={}",
                hide, wait[wait.len() / 2], (after.layer_views - before.layer_views) / 30, (after.main_runs - before.main_runs) / 30, (after.backdrop_copies - before.backdrop_copies) / 30);
        }
    }
    let _ = hidden;
    Ok(())
}
