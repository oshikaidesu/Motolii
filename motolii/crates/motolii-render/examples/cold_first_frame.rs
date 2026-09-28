//! GAP-30 measurement: what the first frame costs when the pipelines are cold, against the same frame warm.
//! Per document: `Engine::new()` (device + compositor), the first frame at the playhead (every pipeline it needs is
//! compiled now), the same frame again (warm), and a second `Engine` in the same process — what export and freeze pay,
//! since each makes its own device and compiles everything again. Run it twice to see what the driver keeps between
//! processes. `cargo run --release -p motolii-render --example cold_first_frame -- <doc.rrd | fixture> ...`
use motolii_edit::Document;
use motolii_render::{doc::store::*, engine::Engine};
use std::time::Instant;

fn ms(since: Instant) -> f64 {
    since.elapsed().as_secs_f64() * 1000.0
}

fn frame(engine: &mut Engine, doc: &Document, t: RationalTime, target: &wgpu::Texture) -> Result<f64, Box<dyn std::error::Error>> {
    let start = Instant::now();
    engine.render_frame_into(&doc.view(), t, target)?;
    engine.gpu_device().poll(wgpu::PollType::wait_indefinitely())?;
    Ok(ms(start))
}

fn target(engine: &Engine, comp: &Composition) -> wgpu::Texture {
    engine.gpu_device().create_texture(&wgpu::TextureDescriptor {
        label: Some("cold-first-frame"),
        size: wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: motolii_render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("doc\tengine_new_ms\tfirst_frame_ms\twarm_same_ms\twarm_next_ms\tengine2_new_ms\tengine2_first_ms");
    for arg in std::env::args().skip(1) {
        let (doc, at) = if arg == "fixture" {
            let f = motolii_edit::fixture::build();
            (f.doc.with_programs(motolii_render::extensions::bundled()), f.playhead)
        } else {
            (Document::load(&arg)?.with_programs(motolii_render::extensions::bundled()), 0)
        };
        let comp = doc.view().composition()?.ok_or("composition")?;
        let t = RationalTime::try_from_frame(at, comp.fps)?;
        let next = RationalTime::try_from_frame(at + 1, comp.fps)?;

        let start = Instant::now();
        let mut engine = Engine::new()?;
        let engine_new = ms(start);
        let tex = target(&engine, &comp);
        let first = frame(&mut engine, &doc, t, &tex)?;
        let warm_same = frame(&mut engine, &doc, t, &tex)?;
        let warm_next = frame(&mut engine, &doc, next, &tex)?;

        let start = Instant::now();
        let mut second = Engine::new()?;
        let engine2_new = ms(start);
        let tex2 = target(&second, &comp);
        let second_first = frame(&mut second, &doc, t, &tex2)?;

        let name = std::path::Path::new(&arg).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(arg.clone());
        println!("{name}\t{engine_new:.1}\t{first:.1}\t{warm_same:.1}\t{warm_next:.1}\t{engine2_new:.1}\t{second_first:.1}");
        if !engine.layer_failures().is_empty() {
            eprintln!("{name}: layer failures {:?}", engine.layer_failures());
        }
    }
    Ok(())
}
