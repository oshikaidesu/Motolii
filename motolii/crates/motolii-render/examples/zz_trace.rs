use motolii_render::doc::store::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Document::load(&std::env::args().nth(1).ok_or("doc")?)?;
    let view = doc.view();
    let comp = view.composition()?.ok_or("comp")?;
    let t = RationalTime::try_from_frame(100, comp.fps)?;
    let resolved = view.resolved_layers(t)?;
    for l in resolved.iter().filter(|l| l.copy == 0 && !l.ghost) {
        if l.effects.iter().any(|e| overlay::is_track_overlay(&e.plugin_id)) {
            let scope = view.overlay_scope(l.id, &resolved, t)?;
            println!("trace layer {:?}: scope {} boxes", l.id, scope.len());
        }
    }
    Ok(())
}
