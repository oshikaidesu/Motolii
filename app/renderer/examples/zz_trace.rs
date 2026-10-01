use motolii_edit::Document;
use motolii_render::doc::store::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Document::load(&std::env::args().nth(1).ok_or("doc")?)?.with_programs(motolii_render::extensions::bundled());
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let t = RationalTime::try_from_frame(120, comp.fps)?;
    for id in view.layers() {
        let name = view.meta(id)?.map(|m| format!("{:?}", m.source)).unwrap_or_default();
        let from = view.value_at(id, &PropertyId::new(layout::CONNECT_FROM)?, t)?;
        if from.is_none() { continue; }
        let shapes = motolii_render::picture::shapes::shapes_at(&view, id, t)?;
        let points: usize = shapes.iter().map(|_| 1).sum();
        println!("{id:?} {name} connect_from={from:?} shapes={points}");
        if let Some(motolii_render::doc::vector::ShapeNode::Leaf(leaf)) = shapes.first() {
            println!("   source={:?}", std::mem::discriminant(&leaf.source));
            if let motolii_render::doc::vector::PathSource::Bezier(cs) = &leaf.source {
                for c in cs { println!("   contour: {} pts {:?}", c.vertices.len(), c.vertices.iter().map(|v| (v.point.x as f32, v.point.y as f32)).collect::<Vec<_>>()); }
            }
        }
    }
    Ok(())
}
