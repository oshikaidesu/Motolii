//! 保存した作品を 1 フレーム描いて PNG に落とす(実窓が黒い時の切り分け用)。
//! `cargo run -p motolii-render --example render_doc -- <doc.rrd> <frame> <out.png>`
use motolii_render::{doc::store::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("document path required")?;
    let frame: i64 = args.next().as_deref().unwrap_or("0").parse()?;
    let out = args.next().unwrap_or_else(|| "render_doc.png".into());
    let mut doc = Document::load(&path)?;
    // 切り分け: MOTOLII_ONLY=<layer id> でその層だけ、MOTOLII_STRIP=<layer id> でその層の効果を外す。
    let only: Option<u64> = std::env::var("MOTOLII_ONLY").ok().and_then(|v| v.parse().ok());
    let strip: Option<u64> = std::env::var("MOTOLII_STRIP").ok().and_then(|v| v.parse().ok());
    if only.is_some() || strip.is_some() {
        let layers: Vec<(LayerId, bool)> = doc.view().resolved_layers(RationalTime::ZERO)?.iter().map(|l| (l.id, matches!(l.source, LayerSource::Camera))).collect();
        for (id, camera) in layers {
            if only.is_some_and(|o| o != u64::from(id.0)) && !camera {
                doc.apply(Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { hidden: Some(true), ..Default::default() } })?;
            }
            if strip == Some(u64::from(id.0)) { doc.apply(Intent::SetEffects { layer: id, effects: vec![] })?; }
        }
    }
    let comp = doc.view().composition()?.ok_or("composition")?;
    let t = RationalTime::try_from_frame(frame, comp.fps)?;
    let mut engine = Engine::new()?;
    if let Ok(mode) = std::env::var("MOTOLII_PRESENT") {
        // 実窓と同じ入口: presentable texture + 観測カメラ + 選択の outline。その後の通常描画が生きているか。
        let texture = engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
            label: Some("present probe"), size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: motolii_render::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        let outline: Vec<LayerId> = if mode == "outline" { vec![LayerId(1)] } else { Vec::new() };
        let camera = motolii_render::doc::core::ResolvedCamera::default();
        engine.render_frame_into_with_camera(&doc.view(), t, &texture, camera, true, &outline)?;
        engine.gpu_device().poll(wgpu::PollType::wait_indefinitely())?;
        eprintln!("present ({mode}): ok; drawn layers {}; failures {:?}", engine.drawn_layers(), engine.layer_failures());
    }
    let repeats: usize = std::env::var("MOTOLII_REPEAT").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let mut pixels = Vec::new();
    for i in 0..repeats {
        let started = std::time::Instant::now();
        pixels = engine.render_frame(&doc.view(), t)?;
        let bright = pixels.chunks_exact(4).filter(|p| p[0] > 20 || p[1] > 20 || p[2] > 20).count();
        let alpha: u64 = pixels.chunks_exact(4).map(|p| p[3] as u64).sum();
        eprintln!("  alpha sum {alpha}; first pixel {:?}", &pixels[..4]);
        eprintln!("render #{i}: {}x{} frame {frame} in {:?}; drawn layers {}; bright pixels {bright}", comp.width, comp.height, started.elapsed(), engine.drawn_layers());
        for f in engine.layer_failures() { eprintln!("  layer failure: {f}"); }
    }
    image::save_buffer(&out, &pixels, comp.width, comp.height, image::ColorType::Rgba8)?;
    Ok(())
}
