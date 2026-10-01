use super::*;

pub(crate) struct CaptureDiagnostic {
    metadata: serde_json::Value,
    buffer: wgpu::Buffer,
    ready: std::sync::mpsc::Receiver<bool>,
    width: u32,
    height: u32,
}

impl CaptureDiagnostic {
    pub(super) fn enqueue(
        ctx: &RenderContext,
        pending: &mut Vec<wgpu::CommandBuffer>,
        texture: &wgpu::Texture,
        metadata: serde_json::Value,
    ) -> Self {
        let size = texture.size();
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("diagnostic-reflection-atlas"),
            size: u64::from(size.width) * u64::from(size.height) * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );
        let (send, ready) = std::sync::mpsc::channel();
        encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, .., move |r| {
            let _ = send.send(r.is_ok());
        });
        pending.push(encoder.finish());
        Self {
            metadata,
            buffer,
            ready,
            width: size.width,
            height: size.height,
        }
    }

    pub(crate) fn save(self, directory: &std::path::Path, name: &str) -> serde_json::Value {
        assert!(
            self.ready
                .try_recv()
                .expect("render completion must flush diagnostic mapping")
        );
        let data = self.buffer.slice(..).get_mapped_range();
        image::RgbaImage::from_raw(self.width, self.height, data.to_vec())
            .unwrap()
            .save(directory.join(format!("{name}-atlas.png")))
            .unwrap();
        drop(data);
        self.buffer.unmap();
        std::fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&self.metadata).unwrap(),
        )
        .unwrap();
        self.metadata
    }
}
