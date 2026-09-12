//! 調査用: 平らな円に Glass(透過 100%・粗さ最大)を載せ、下の赤い角の縁がぼけるか。
use motolii_render::{doc::store::*, doc::vector::*, engine::Engine};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new()?;
    let square = |color: Rgb, size: f64, src: PathSource| ShapeNode::Leaf(Shape { source: src, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(color), ..Default::default() }) });
    let _ = size_hint;
    for (label, roughness) in [("roughness 0", 0.0), ("roughness 1", 1.0)] {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: 256, height: 256, fps: Fps::try_new(30, 1)?, duration_frames: 1, background: [1.0, 1.0, 1.0, 1.0] }))?;
        doc.apply_all([
            Intent::AddLayer(LayerId(1)),
            Intent::SetMeta { layer: LayerId(1), meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetShapes { layer: LayerId(1), shapes: vec![square(Rgb { r: 1.0, g: 0.0, b: 0.0 }, 120.0, PathSource::Rectangle { size: Point { x: 120.0, y: 120.0 } })] },
            Intent::SetAttrs { layer: LayerId(1), patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
            Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([100.0, 128.0]) },
            Intent::AddLayer(LayerId(2)),
            Intent::SetMeta { layer: LayerId(2), meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetShapes { layer: LayerId(2), shapes: vec![square(Rgb { r: 1.0, g: 1.0, b: 1.0 }, 120.0, PathSource::Ellipse { size: Point { x: 120.0, y: 120.0 } })] },
            Intent::SetAttrs { layer: LayerId(2), patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoPointFiveD), ..Default::default() } },
            Intent::SetConstant { layer: LayerId(2), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([160.0, 128.0]) },
            Intent::SetEffects { layer: LayerId(2), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] },
            Intent::SetConstant { layer: LayerId(2), property: PropertyId::effect_param(EffectId(0), "transmission").unwrap(), value: Value::F64(1.0) },
            Intent::SetConstant { layer: LayerId(2), property: PropertyId::effect_param(EffectId(0), "roughness").unwrap(), value: Value::F64(roughness) },
        ])?;
        let px = engine.render_frame(&doc.view(), RationalTime::ZERO)?;
        image::save_buffer(format!("/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/7d02b591-a384-43db-8038-a5de0fe623a3/scratchpad/glass-plane-{}.png", roughness), &px, 256, 256, image::ColorType::Rgba8)?;
        // 円の中(中心 160,128 半径 60)で赤の縁 x=160 を跨ぐ行 y=128 の値
        let row: Vec<_> = (140..180).step_by(4).map(|x| px[(128 * 256 + x) * 4..(128 * 256 + x) * 4 + 3].to_vec()).collect();
        println!("{label}: row y=128 x=140..180 {row:?}");
        for f in engine.layer_failures() { println!("  failure: {f}"); }
    }
    Ok(())
}
fn size_hint() {}
