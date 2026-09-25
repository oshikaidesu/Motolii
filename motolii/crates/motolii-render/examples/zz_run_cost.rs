//! The fixed cost of a run: N tiny pictures that each start a run (2D and not, alternating), drawn
//! through the production route. GPU wait per frame against N gives the cost of one run.
//! `zz_run_cost <N> <width> <height>`
use motolii_edit::{Document, Intent};
use motolii_render::{doc::store::*, engine::Engine};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n: u64 = std::env::args().nth(1).and_then(|v| v.parse().ok()).unwrap_or(8);
    let width: u32 = std::env::args().nth(2).and_then(|v| v.parse().ok()).unwrap_or(1920);
    let height: u32 = std::env::args().nth(3).and_then(|v| v.parse().ok()).unwrap_or(1080);
    let dir = std::env::temp_dir().join("motolii-run-cost");
    std::fs::create_dir_all(&dir)?;
    let png = dir.join("dot.png");
    image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 120, 40, 255])).save(&png)?;
    let mut doc = Document::new().with_programs(motolii_render::extensions::bundled());
    let fps = Fps::try_new(60, 1).unwrap();
    doc.apply(Intent::SetComposition(Composition { width, height, fps, duration_frames: 300, background: [0.02, 0.02, 0.03, 1.0], look: Default::default() }))?;
    for i in 0..n {
        let layer = LayerId(1 + i);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: png.to_string_lossy().into_owned(), fingerprint: None }, order: i as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([width as f64 / 2.0 + i as f64 * 5.0, height as f64 / 2.0]) },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(if i % 2 == 0 { LayerProjection::TwoD } else { LayerProjection::ThreeD }), ..Default::default() } },
        ])?;
    }
    let comp = doc.view().composition()?.ok_or("composition")?;
    let mut engine = Engine::new()?;
    engine.set_realtime(true);
    let device = engine.gpu_device().clone();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("run-cost"), size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: motolii_render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    for pass in 0..2 {
        let before = engine.surface_work();
        let mut wait = Vec::new();
        for frame in 0..30 {
            let t = RationalTime::try_from_frame(frame, comp.fps)?;
            engine.render_frame_into(&doc.view(), t, &target)?;
            let submitted = Instant::now();
            device.poll(wgpu::PollType::wait_indefinitely())?;
            wait.push(submitted.elapsed().as_secs_f64() * 1000.0);
        }
        if pass == 1 {
            let after = engine.surface_work();
            wait.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("n={n} {width}x{height} gpu_wait_median_ms={:.3} main_runs/frame={} run_breaks={:?}", wait[wait.len() / 2], (after.main_runs - before.main_runs) / 30,
                after.run_breaks.iter().zip(before.run_breaks.iter()).map(|(a, b)| (a - b) / 30).collect::<Vec<_>>());
        }
    }
    Ok(())
}
