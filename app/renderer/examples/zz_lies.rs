//! 調査用(2026-09-15 2.5D の嘘): 作品を読み、名前で層に environment / Cast Shadow を足して、指定コマを PNG へ。
//! `zz_lies <doc.rrd> <out_dir> <frames,comma> [Name=env|blocks ...]`
use motolii_edit::{Document, Intent};
use motolii_render::{doc::store::*, engine::Engine, picture::resolve::resolved_layers};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut doc = Document::load(&args.next().ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let out = args.next().ok_or("out")?;
    let frames: Vec<i64> = args.next().ok_or("frames")?.split(',').map(|f| f.parse()).collect::<Result<_, _>>()?;
    let layers: Vec<(LayerId, String)> = resolved_layers(&doc.view(), RationalTime::ZERO)?.iter().map(|l| (l.id, doc.view().attrs(l.id).ok().flatten().map(|a| a.name).unwrap_or_default())).collect();
    for edit in args {
        let (name, what) = edit.split_once('=').ok_or("Name=env|blocks")?;
        let id = layers.iter().find(|(_, n)| n == name).ok_or(format!("no layer {name}"))?.0;
        match what {
            "env" => doc.apply(Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { environment: Some(true), ..Default::default() } })?,
            _ => doc.apply(Intent::SetEffects { layer: id, effects: vec![EffectInstance { id: EffectId(900), plugin_id: "motolii.cast_shadow".into() }] })?,
        }
    }
    let comp = doc.view().composition()?.ok_or("comp")?;
    let mut engine = Engine::new()?;
    for frame in frames {
        let pixels = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, comp.fps)?)?;
        for f in engine.layer_failures() { eprintln!("frame {frame}: {f}"); }
        image::save_buffer(format!("{out}/{frame:04}.png"), &pixels, comp.width, comp.height, image::ColorType::Rgba8)?;
    }
    Ok(())
}
