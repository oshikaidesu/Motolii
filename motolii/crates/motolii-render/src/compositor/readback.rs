//! Pixels and bytes the GPU hands back on a later frame, through re_renderer's readback belt.
//! The copy is recorded behind everything the frame has recorded so far (re_renderer's own
//! `schedule_read_texture` copies ahead of it, in the frame-global encoder), so what is read is
//! what the frame drew.

use re_renderer::texture_info::Texture2DBufferInfo;
use re_renderer::{GpuReadbackError, GpuReadbackIdentifier, RenderContext};

fn next_identifier() -> GpuReadbackIdentifier {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0x6d6f_746f_0000_0000);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// A texture's pixels, row padding removed.
pub(crate) struct Pixels {
    pub(crate) data: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

struct TextureShape {
    extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,
}

struct BufferSize(wgpu::BufferAddress);

/// Asks for `texture` as this frame leaves it. Taken with [`take_texture`] on a later frame.
pub(crate) fn ask_texture(ctx: &RenderContext, texture: &wgpu::Texture) -> Result<GpuReadbackIdentifier, GpuReadbackError> {
    let (extent, format) = (texture.size(), texture.format());
    let id = next_identifier();
    let mut buffer = ctx.gpu_readback_belt.lock().allocate(
        &ctx.device,
        &ctx.gpu_resources.buffers,
        Texture2DBufferInfo::new(format, extent).buffer_size_padded,
        id,
        Box::new(TextureShape { extent, format }),
    );
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-readback") });
    buffer.read_texture2d(&mut encoder, texture.as_image_copy(), extent)?;
    ctx.queue_commands([encoder.finish()]);
    Ok(id)
}

/// The pixels asked for with [`ask_texture`], once they have arrived.
pub(crate) fn take_texture(ctx: &RenderContext, id: GpuReadbackIdentifier) -> Option<Pixels> {
    ctx.gpu_readback_belt.lock().readback_next_available(id, |data: &[u8], shape: Box<TextureShape>| Pixels {
        data: Texture2DBufferInfo::new(shape.format, shape.extent).remove_padding(data).into_owned(),
        width: shape.extent.width,
        height: shape.extent.height,
    })
}

/// Asks for the first `size` bytes of `buffer` as this frame leaves it. Taken with [`take_buffer`].
pub(crate) fn ask_buffer(ctx: &RenderContext, buffer: &wgpu::Buffer, size: wgpu::BufferAddress) -> Result<GpuReadbackIdentifier, GpuReadbackError> {
    let id = next_identifier();
    let mut readback = ctx.gpu_readback_belt.lock().allocate(&ctx.device, &ctx.gpu_resources.buffers, size, id, Box::new(BufferSize(size)));
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-readback") });
    readback.read_buffer(&mut encoder, buffer, 0, size)?;
    ctx.queue_commands([encoder.finish()]);
    Ok(id)
}

/// The bytes asked for with [`ask_buffer`], once they have arrived.
pub(crate) fn take_buffer(ctx: &RenderContext, id: GpuReadbackIdentifier) -> Option<Vec<u8>> {
    ctx.gpu_readback_belt.lock().readback_next_available(id, |data: &[u8], size: Box<BufferSize>| data[..size.0 as usize].to_vec())
}
