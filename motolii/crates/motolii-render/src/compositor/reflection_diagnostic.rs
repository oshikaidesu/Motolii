use super::*;

/// Tests only: the reflection atlas of a frame, asked of the readback belt as the frame leaves
/// it, saved once the test's render has waited for it.
pub(crate) struct CaptureDiagnostic {
    metadata: serde_json::Value,
    id: re_renderer::GpuReadbackIdentifier,
}

impl CaptureDiagnostic {
    pub(super) fn enqueue(
        ctx: &RenderContext,
        texture: &wgpu::Texture,
        metadata: serde_json::Value,
    ) -> Self {
        let id = super::readback::ask_texture(ctx, texture).expect("diagnostic readback");
        Self { metadata, id }
    }

    pub(crate) fn save(self, ctx: &RenderContext, directory: &std::path::Path, name: &str) -> serde_json::Value {
        let pixels = super::readback::take_texture(ctx, self.id)
            .expect("render completion must bring the diagnostic readback");
        image::RgbaImage::from_raw(pixels.width, pixels.height, pixels.data)
            .unwrap()
            .save(directory.join(format!("{name}-atlas.png")))
            .unwrap();
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&self.metadata).unwrap(),
        )
        .unwrap();
        self.metadata
    }
}
