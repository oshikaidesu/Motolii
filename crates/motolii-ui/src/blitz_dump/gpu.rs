//! ヘッドレスwgpu面と、PNG書き出し。
//!
//! すべて `spikes/blitz-probe/src/bin/alpha_composite.rs` の**写し**:
//!   - `init_gpu` (:440-479) → [`Gpu::new`]
//!   - `read_texture` (:789-833) → [`read_texture`]
//!   - `unpremultiply` (:399-410) → [`unpremultiply`]
//!   - `write_png` / `chunk` / `crc32` / `adler32` (:838-905) → 以下同名
//!
//! ## 実測済みの罠(同probe)
//! - テクスチャformatは `Rgba8Unorm`。`Rgba8UnormSrgb` にすると色が浮く(最大誤差73)
//! - 出力はプリマルチプライドアルファ。PNGへ書く前に unpremultiply する
//! - `body` に背景色を置くと viewport 全面が塗り潰される。よってパネル側のCSSは
//!   背景を持たず、面の外は透過で出る。**確認用の下地はここで別レイヤとして敷く**
//!   ([`over_checkerboard`])。パネル自身のCSSには触らない。

use std::io::Write;

/// 面のテクスチャformat。`Rgba8UnormSrgb` にしないこと(罠2)。
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub struct Gpu {
    /// `BlitzSurface` は自分で `Instance` を作るので描画には使わないが、
    /// adapter/device の出所として生かしておく。
    #[allow(dead_code)]
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    /// alpha_composite.rs:440-479 の写し。窓は開かない。
    pub fn new(runtime: &tokio::runtime::Runtime) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = runtime
            .block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .expect("no adapter");
        eprintln!("blitz-dump: backend={:?}", adapter.get_info().backend);
        let (device, queue) = runtime
            .block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("motolii-blitz-dump"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            }))
            .expect("no device");
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    /// 読み戻せる描画先。`COPY_SRC` を足す以外は
    /// `blitz_ui/surface.rs:235-250` と同じ記述。
    pub fn target(&self, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-blitz-dump-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}

/// alpha_composite.rs:789-833 の写し。行のpaddingを外して詰め直す。
pub fn read_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let bpr = (width * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
                bytes_per_row: Some(bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let padded = slice.get_mapped_range().to_vec();
    let mut out = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let start = (y * bpr) as usize;
        out.extend_from_slice(&padded[start..start + (width * 4) as usize]);
    }
    out
}

/// alpha_composite.rs:399-410 の写し。
pub fn unpremultiply(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for i in (0..src.len()).step_by(4) {
        let a = src[i + 3] as f32;
        if a > 0.0 {
            for c in 0..3 {
                out[i + c] = ((src[i + c] as f32 * 255.0 / a).round()).clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// 確認用の下地。**パネルのCSSではなくここで敷く**(罠1)。
///
/// 市松はUIの色ではない。透過している場所が一目で分かることだけが目的で、
/// パネルの見た目に足す意図は無い。入力はプリマルチプライド済みなので
/// `dst = src + bg * (1 - a)` で重ねる。
pub fn over_checkerboard(premultiplied: &[u8], width: u32, height: u32) -> Vec<u8> {
    const SQUARE: u32 = 16;
    const LIGHT: [u8; 3] = [0xa0, 0xa0, 0xa0];
    const DARK: [u8; 3] = [0x80, 0x80, 0x80];
    let mut out = vec![0u8; premultiplied.len()];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            let bg = if ((x / SQUARE) + (y / SQUARE)) % 2 == 0 {
                LIGHT
            } else {
                DARK
            };
            let inv = 255 - premultiplied[i + 3] as u32;
            for c in 0..3 {
                let value = premultiplied[i + c] as u32 + bg[c] as u32 * inv / 255;
                out[i + c] = value.min(255) as u8;
            }
            out[i + 3] = 255;
        }
    }
    out
}

/// alpha_composite.rs:838-870 の写し。無圧縮zlibのRGBA PNG。
pub fn write_png(path: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    let mut raw = Vec::with_capacity(((w * 4 + 1) * h) as usize);
    for y in 0..h {
        raw.push(0u8); // filter: None
        let s = (y * w * 4) as usize;
        raw.extend_from_slice(&rgba[s..s + (w * 4) as usize]);
    }

    let mut z = vec![0x78u8, 0x01]; // zlib header, no compression
    let mut i = 0usize;
    while i < raw.len() {
        let n = (raw.len() - i).min(65535);
        let last = if i + n == raw.len() { 1u8 } else { 0u8 };
        z.push(last);
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[i..i + n]);
        i += n;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);

    let mut file = std::fs::File::create(path).expect("png create");
    file.write_all(&png).expect("png write");
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
