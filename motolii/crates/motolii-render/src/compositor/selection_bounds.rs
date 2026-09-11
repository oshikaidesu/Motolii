//! 選ばれた層の画面上の広がりを、絵そのもの(outline の object-id mask)から取る。
//! 点を写して包む籠は透視・effect・変位を知らないが、mask は描いた画素そのものなので
//! 平面・網・点群・3D のどれでも 1 px で正しい。GPU で id ごとの min/max に畳み、
//! 読み戻すのは id 256 個 × 4 値 = 4 KiB だけ(mask 全体の読み戻しは 4K で 16 MB/フレーム)。

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

const IDS: usize = 256;
const WORDS: usize = IDS * 4;
const BYTES: u64 = (WORDS * 4) as u64;

// min は ~x の max として畳む: buffer を 0 に clear するだけで min・max とも初期化できる。
const SHADER: &str = r#"
@group(0) @binding(0) var mask: MASK_TEXTURE;
@group(0) @binding(1) var<storage, read_write> out: array<atomic<u32>, 1024>;
var<workgroup> tile: array<atomic<u32>, 1024>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3u, @builtin(local_invocation_index) li: u32) {
    for (var i = li; i < 1024u; i += 256u) { atomicStore(&tile[i], 0u); }
    workgroupBarrier();
    let dims = textureDimensions(mask);
    if gid.x < dims.x && gid.y < dims.y {
        let id = textureLoad(mask, vec2i(gid.xy) MASK_SAMPLE).r;
        if id != 0u {
            atomicMax(&tile[id * 4u + 0u], ~gid.x);
            atomicMax(&tile[id * 4u + 1u], ~gid.y);
            atomicMax(&tile[id * 4u + 2u], gid.x + 1u);
            atomicMax(&tile[id * 4u + 3u], gid.y + 1u);
        }
    }
    workgroupBarrier();
    for (var i = li; i < 1024u; i += 256u) {
        let v = atomicLoad(&tile[i]);
        if v != 0u { atomicMax(&out[i], v); }
    }
}
"#;

pub(crate) struct SelectionBounds {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    result: wgpu::Buffer,
    staging: wgpu::Buffer,
    /// 0 = 空、1 = map 待ち、2 = map 済み。
    mapped: Arc<AtomicU8>,
    multisampled: bool,
}

impl SelectionBounds {
    /// compute の上限が無い device(WebGL2 相当)では `None`: 籠は写した点へ戻る。
    pub(crate) fn new(device: &wgpu::Device, multisampled: bool) -> Option<Self> {
        let limits = device.limits();
        if limits.max_storage_buffers_per_shader_stage < 1
            || limits.max_compute_invocations_per_workgroup < 256
            || limits.max_compute_workgroup_size_x < 16
            || limits.max_compute_workgroup_size_y < 16
            || limits.max_compute_workgroup_storage_size < (WORDS * 4) as u32
        {
            return None;
        }
        let source = SHADER
            .replace("MASK_TEXTURE", if multisampled { "texture_multisampled_2d<u32>" } else { "texture_2d<u32>" })
            .replace("MASK_SAMPLE", if multisampled { ", 0" } else { ", 0" });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-selection-bounds"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-selection-bounds"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Uint, view_dimension: wgpu::TextureViewDimension::D2, multisampled },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-selection-bounds"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("motolii-selection-bounds"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let result = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-selection-bounds-result"),
            size: BYTES,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-selection-bounds-staging"),
            size: BYTES,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Some(Self { pipeline, layout, result, staging, mapped: Arc::new(AtomicU8::new(0)), multisampled })
    }

    /// mask を畳んで staging へ写す命令を記録する。submit の後に `schedule_map` を呼ぶ。
    pub(crate) fn record(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, mask: &wgpu::TextureView, size: [u32; 2]) {
        // 前の読み戻しがまだ map 中なら、staging へは書けない。待って取り込む。
        if self.mapped.load(Ordering::Acquire) == 1 {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
        }
        if self.mapped.load(Ordering::Acquire) == 2 {
            self.staging.unmap();
            self.mapped.store(0, Ordering::Release);
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-selection-bounds"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(mask) },
                wgpu::BindGroupEntry { binding: 1, resource: self.result.as_entire_binding() },
            ],
        });
        encoder.clear_buffer(&self.result, 0, None);
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-selection-bounds"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(size[0].div_ceil(16), size[1].div_ceil(16), 1);
        }
        encoder.copy_buffer_to_buffer(&self.result, 0, &self.staging, 0, BYTES);
    }

    /// submit 済みの staging の map を頼む。呼び手の次の `poll` で完了する。
    pub(crate) fn schedule_map(&mut self) {
        if self.mapped.load(Ordering::Acquire) != 0 {
            return;
        }
        self.mapped.store(1, Ordering::Release);
        let flag = Arc::clone(&self.mapped);
        self.staging.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            flag.store(if result.is_ok() { 2 } else { 0 }, Ordering::Release);
        });
    }

    /// 届いていれば id ごとの `[x0, y0, x1, y1]`(画素、x1・y1 は外側)。届く前なら `None`。
    pub(crate) fn take(&mut self, device: &wgpu::Device) -> Option<Vec<(u8, [f32; 4])>> {
        if self.mapped.load(Ordering::Acquire) == 1 {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
        }
        if self.mapped.load(Ordering::Acquire) != 2 {
            return None;
        }
        let out = {
            let data = self.staging.slice(..).get_mapped_range();
            let words: Vec<u32> = data.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
            (1..IDS)
                .filter_map(|id| {
                    let w = &words[id * 4..id * 4 + 4];
                    (w[2] != 0).then(|| (id as u8, [(!w[0]) as f32, (!w[1]) as f32, w[2] as f32, w[3] as f32]))
                })
                .collect()
        };
        self.staging.unmap();
        self.mapped.store(0, Ordering::Release);
        Some(out)
    }

    pub(crate) fn multisampled(&self) -> bool {
        self.multisampled
    }
}

#[cfg(test)]
mod reduction_tests {
    use super::*;

    /// id 3 を 10..20 × 5..8 に描いた mask から、その矩形がそのまま返る。他の id は無し。
    #[test]
    fn a_painted_id_comes_back_as_its_pixel_rectangle() {
        let gpu = crate::render::compositor::headless::HeadlessGpu::new().unwrap();
        let (device, queue) = (gpu.device, gpu.queue);
        let mut bounds = SelectionBounds::new(&device, false).expect("the headless device borrows compute limits");
        let (w, h) = (64u32, 32u32);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mask"), size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rg8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
        });
        let mut pixels = vec![0u8; (w * h * 2) as usize];
        for y in 5..8 { for x in 10..20 { pixels[((y * w + x) * 2) as usize] = 3; } }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 2), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        let view = texture.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        bounds.record(&device, &mut encoder, &view, [w, h]);
        queue.submit([encoder.finish()]);
        bounds.schedule_map();
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let found = bounds.take(&device).expect("mapped after the wait");
        assert_eq!(found, vec![(3u8, [10.0, 5.0, 20.0, 8.0])]);
    }
}
