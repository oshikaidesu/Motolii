//! 実測: 1080p で feedback(RGB Trail)と別時刻の合成(Background Delay)の重さ。
//! `cargo run --release -p motolii-render --example feedback_cost -- <clip>`
use motolii_render::{doc::store::*, doc::vector::*, engine::Engine};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clip = std::env::args().nth(1).ok_or("clip required")?;
    let (w, h) = (1920u32, 1080u32);
    let fps = Fps::try_new(24, 1)?;
    let frames = 120i64;
    let build = |trail: bool, delay: bool| -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: w, height: h, fps, duration_frames: frames, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: clip.clone(), fingerprint: None }, order: 0, timing: LayerTiming { source_in: 24 * 70, ..LayerTiming::place(0, None, frames) } } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
            Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([1.5, 1.5]) },
        ]).unwrap();
        if trail {
            doc.apply_all([
                Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.rgb_trail".into() }] },
                Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "drift").unwrap(), value: Value::Vec2([6.0, 0.0]) },
            ]).unwrap();
        }
        if delay {
            let lens = LayerId(2);
            doc.apply_all([
                Intent::AddLayer(lens),
                Intent::SetMeta { layer: lens, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, frames) } },
                Intent::SetShapes { layer: lens, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Ellipse { size: Point { x: 700.0, y: 500.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
                Intent::SetConstant { layer: lens, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([600.0, 300.0]) },
                Intent::SetEffects { layer: lens, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.background_delay".into() }] },
                Intent::SetConstant { layer: lens, property: PropertyId::effect_param(EffectId(0), "offset").unwrap(), value: Value::F64(-0.25) },
            ]).unwrap();
        }
        doc
    };
    let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1000.0;
    for (name, trail, delay) in [("clip only", false, false), ("+ RGB Trail", true, false), ("+ RGB Trail + Background Delay", true, true)] {
        let doc = build(trail, delay);
        let mut engine = Engine::new()?;
        engine.render_frame(&doc.view(), at(0))?; // 初回(復号器の起動)は数えない
        let t = Instant::now();
        for f in 1..=48 { engine.render_frame(&doc.view(), at(f))?; }
        let sequential = ms(t) / 48.0;
        // 飛び: 60 へ(状態は 48 = 直近の写し 30 から 30 歩)
        let t = Instant::now(); engine.render_frame(&doc.view(), at(60))?; let jump_fwd = ms(t);
        // 戻り: 40 へ(写し 30 から 10 歩)
        let t = Instant::now(); engine.render_frame(&doc.view(), at(40))?; let jump_back = ms(t);
        // 遠くへ: 110 へ(写し 60 から 50 歩、途中で写し 90)
        let t = Instant::now(); engine.render_frame(&doc.view(), at(110))?; let jump_far = ms(t);
        // 編集: 欄を回す → 状態を捨てて入点から 110 まで
        let mut doc = doc;
        if trail { doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "reach").unwrap(), value: Value::F64(150.0) })?; }
        else { doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([1.0, 0.0]) })?; }
        let t = Instant::now(); engine.render_frame(&doc.view(), at(110))?; let edit = ms(t);
        let failures = engine.layer_failures().len();
        println!("{name:32} | sequential {sequential:6.1} ms/frame | jump 48→60 {jump_fwd:7.1} ms | back 48→40 {jump_back:7.1} ms | far 40→110 {jump_far:7.1} ms | edit@110 {edit:7.1} ms | failures {failures}");
    }
    Ok(())
}
