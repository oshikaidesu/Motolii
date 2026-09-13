//! 使い捨ての見本: 実写の上に群(動く 3 つの形)、群に Whole で RGB Trail、その上に Background Delay の形。
//! 96 コマを順に描いて PNG 列に、ffmpeg で mp4 に。`cargo run --example rgb_trail_demo -- <clip> <out dir>`
use motolii_render::{doc::store::*, doc::vector::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    let mut args = std::env::args().skip(1);
    let clip = args.next().ok_or("clip required")?;
    let dir = args.next().unwrap_or_else(|| ".".into());
    std::fs::create_dir_all(&dir)?;
    let (w, h) = (1280u32, 534u32);
    let fps = Fps::try_new(24, 1)?;
    let frames = 96i64;
    let refresh = motolii_render::compositor::refresh_effect_catalog();
    for e in &refresh.errors { eprintln!("catalog: {e}"); }

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition { width: w, height: h, fps, duration_frames: frames, background: [0.0, 0.0, 0.0, 1.0] }))?;
    // 下: 実写。
    let clip_layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(clip_layer),
        Intent::SetMeta { layer: clip_layer, meta: LayerMeta { source: LayerSource::File { path: clip.clone(), fingerprint: None }, order: 0, timing: LayerTiming { source_in: 24 * 70, ..LayerTiming::place(0, None, frames) } } },
        Intent::SetConstant { layer: clip_layer, property: PropertyId::new(property::POSITION)?, value: Value::Vec2([0.0, 0.0]) },
    ])?;
    // 群: 動く 3 つの形。群に RGB Trail(Whole)。
    let group = LayerId(10);
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 1, timing: LayerTiming::place(0, None, frames) } },
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.rgb_trail".into() }] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_scope(EffectId(0)), value: Value::Enum(EffectScope::Whole.enum_value()) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(EffectId(0), "drift")?, value: Value::Vec2([4.0, -1.0]) },
    ])?;
    let track = |from: [f64; 2], to: [f64; 2]| {
        let mut t = KeyframeTrack::new();
        t.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2(from), interp: Interp::Linear, spatial: None });
        t.insert(Keyframe { t: RationalTime::try_from_frame(frames, fps).unwrap(), value: Value::Vec2(to), interp: Interp::Linear, spatial: None });
        t
    };
    let shapes: [(u64, PathSource, Rgb, [f64; 2], [f64; 2]); 3] = [
        // 座標は左上原点(comp 1280 × 534)。
        (11, PathSource::Ellipse { size: Point { x: 120.0, y: 120.0 } }, Rgb { r: 1.0, g: 0.95, b: 0.9 }, [120.0, 140.0], [1100.0, 380.0]),
        (12, PathSource::PolyStar { points: 5.0, outer_radius: 80.0, inner_radius: 36.0, star_type: StarType::Star }, Rgb { r: 0.2, g: 0.9, b: 1.0 }, [1120.0, 90.0], [200.0, 330.0]),
        (13, PathSource::Rectangle { size: Point { x: 140.0, y: 60.0 } }, Rgb { r: 1.0, g: 0.3, b: 0.5 }, [300.0, 440.0], [900.0, 60.0]),
    ];
    for (id, source, color, from, to) in shapes {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, frames) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } },
            Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(color), ..Default::default() }) })] },
            Intent::SetTrack { layer, property: PropertyId::new(property::POSITION)?, track: track(from, to) },
        ])?;
    }
    // 上: 形の中に 0.2 秒前の下の合成(Background Delay)。
    let lens = LayerId(20);
    doc.apply_all([
        Intent::AddLayer(lens),
        Intent::SetMeta { layer: lens, meta: LayerMeta { source: LayerSource::Shape, order: 20, timing: LayerTiming::place(0, None, frames) } },
        Intent::SetShapes { layer: lens, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 360.0, y: 200.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
        Intent::SetTrack { layer: lens, property: PropertyId::new(property::POSITION)?, track: track([860.0, 300.0], [80.0, 300.0]) },
        Intent::SetEffects { layer: lens, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.background_delay".into() }] },
        Intent::SetConstant { layer: lens, property: PropertyId::effect_param(EffectId(0), "offset")?, value: Value::F64(-0.2) },
    ])?;

    let mut engine = Engine::new()?;
    let started = std::time::Instant::now();
    for f in 0..frames {
        let t = RationalTime::try_from_frame(f, fps)?;
        let pixels = engine.render_frame(&doc.view(), t)?;
        for e in engine.layer_failures() { eprintln!("frame {f}: {e}"); }
        image::save_buffer(format!("{dir}/rgb-trail-{f:03}.png"), &pixels, w, h, image::ColorType::Rgba8)?;
    }
    eprintln!("{frames} frames in {:.1}s", started.elapsed().as_secs_f64());
    // 飛んで来ても同じ絵か(スクラブの審判を見本でも 1 回)。
    let jumped = Engine::new()?.render_frame(&doc.view(), RationalTime::try_from_frame(60, fps)?)?;
    let walked = image::open(format!("{dir}/rgb-trail-060.png"))?.to_rgba8();
    let differing = jumped.chunks_exact(4).zip(walked.as_raw().chunks_exact(4)).filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 2)).count();
    eprintln!("jump to 60 vs walked: {differing} px differ");
    let status = std::process::Command::new("ffmpeg").args(["-v", "error", "-y", "-framerate", "24", "-i"]).arg(format!("{dir}/rgb-trail-%03d.png"))
        .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18"]).arg(format!("{dir}/rgb-trail.mp4")).status()?;
    eprintln!("mp4: {status}");
    Ok(())
}
