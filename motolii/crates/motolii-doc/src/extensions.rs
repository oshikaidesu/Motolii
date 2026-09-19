//! Bundled semantic effect implementations; persistence contracts live in store.
pub mod placement;
pub mod pathop;
pub mod blob;
pub mod motion;
pub mod solid;
pub mod overlay;

pub fn placement_program(plugin_id: &str) -> Option<crate::store::kind::PlacementProgram> {
    placement::program(plugin_id).or_else(|| blob::program(plugin_id))
}

pub fn sampling_program(plugin_id: &str) -> Option<crate::store::kind::SamplingProgram> {
    motion::program(plugin_id)
}
