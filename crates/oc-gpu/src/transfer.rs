use oc_core::FrameDesc;

use crate::{GpuCtx, GpuRuntimeError};

use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

/// 対話系・単発ダウンロード向けの既定タイムアウト。
pub const DEFAULT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(5);
const POLL_STEP: Duration = Duration::from_millis(10);

/// パック系RGBAのCPUデータをVRAMのテクスチャへアップロードする。
/// 性能方針: 以後このテクスチャはVRAM常駐でエフェクトチェーンを流れ、CPUには戻さない。
pub fn upload_rgba(gpu: &GpuCtx, desc: &FrameDesc, data: &[u8]) -> wgpu::Texture {
    assert_eq!(data.len(), desc.data_size(), "upload_rgba: size mismatch");
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("rgba-upload"),
        size: wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(desc.stride),
            rows_per_image: Some(desc.height),
        },
        wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: 1,
        },
    );
    texture
}

/// RGBA8テクスチャをCPUへダウンロードする(テスト・書き出し用)。
/// wgpuの256バイト行アラインを吸収してタイトなRGBAを返す。
/// 単発呼び出し向けの薄いラッパー。書き出しループ等、同一寸法を繰り返し
/// ダウンロードする場合は`RgbaDownloader`を使い、毎フレームの確保を避けること。
pub fn download_rgba(gpu: &GpuCtx, texture: &wgpu::Texture) -> Result<Vec<u8>, GpuRuntimeError> {
    RgbaDownloader::new().download(gpu, texture, DEFAULT_DOWNLOAD_TIMEOUT)
}

/// RGBA8ダウンロード用ステージングバッファ(MAP_READ)を使い回すダウンローダ。
/// performance-model 原則3「確保・解放を毎フレームやらない」の実装。
/// 書き出しループのように同一寸法のテクスチャを繰り返しダウンロードする用途向け。
#[derive(Default)]
pub struct RgbaDownloader {
    buffer: Option<wgpu::Buffer>,
    /// 現在保持しているバッファの実バイト数。必要サイズがこれと一致する間は
    /// バッファを使い回し、変わったら(=解像度が変わったら)作り直す。
    capacity: u64,
}

impl RgbaDownloader {
    pub fn new() -> Self {
        Self {
            buffer: None,
            capacity: 0,
        }
    }

    /// RGBA8テクスチャをCPUへダウンロードする。
    /// 必要バイト数が前回と同じならステージングバッファを使い回す
    /// (map→読み取り→unmapのサイクルは毎回行うが、確保・解放は行わない)。
    pub fn download(
        &mut self,
        gpu: &GpuCtx,
        texture: &wgpu::Texture,
        timeout: Duration,
    ) -> Result<Vec<u8>, GpuRuntimeError> {
        let width = texture.width();
        let height = texture.height();
        let unpadded = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let required = (padded * height) as u64;

        if self.capacity != required {
            self.buffer = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("rgba-download"),
                size: required,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
            self.capacity = required;
        }
        // ハンドルをclone(Arcの複製)しておき、エラー時にself.bufferを捨てられるようにする
        let buffer = self
            .buffer
            .clone()
            .expect("buffer ensured just above");
        let buffer = &buffer;

        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([enc.finish()]);

        let slice = buffer.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result.map_err(|e| e.to_string()));
        });
        if let Err(e) = wait_for_map(gpu, rx, timeout) {
            // map要求が残留したバッファは再map時に検証エラーになるため再利用しない
            self.buffer = None;
            self.capacity = 0;
            return Err(e);
        }

        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity((unpadded * height) as usize);
        for row in 0..height {
            let start = (row * padded) as usize;
            out.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        buffer.unmap();
        Ok(out)
    }
}

fn wait_for_map(
    gpu: &GpuCtx,
    rx: mpsc::Receiver<Result<(), String>>,
    timeout: Duration,
) -> Result<(), GpuRuntimeError> {
    let start = Instant::now();
    loop {
        gpu.check_health()?;
        match rx.try_recv() {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(e)) => return Err(GpuRuntimeError::Map(e)),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(GpuRuntimeError::Map(
                    "buffer map callback channel disconnected".into(),
                ));
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(GpuRuntimeError::Timeout(timeout));
        }
        let remaining = timeout - elapsed;
        let step = remaining.min(POLL_STEP);
        match gpu.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(step),
        }) {
            Ok(_) | Err(wgpu::PollError::Timeout) => {}
            Err(e) => return Err(GpuRuntimeError::Poll(e.to_string())),
        }
    }
}
