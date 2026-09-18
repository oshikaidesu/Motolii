fn main() {
    let _ = motolii_render::engine::Engine::new();
    for e in motolii_render::engine::catalog_errors() {
        println!("rejected: {e}");
    }
}
