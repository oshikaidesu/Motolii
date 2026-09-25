// as: motolii/crates/motolii-render/src/compositor/transmissive.rs
// A second renderer inside Motolii, however it is named.
pub(crate) struct TransmissiveSurfaceRenderer;
impl re_renderer::renderer::Renderer for TransmissiveSurfaceRenderer {
    type RendererDrawData = TransmissiveDrawData;
}
