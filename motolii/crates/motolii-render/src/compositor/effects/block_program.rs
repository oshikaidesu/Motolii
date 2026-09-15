//! 箱のブロック(`"STAGE": "block"`): 箱の並びを GPU の storage buffer に置き、物ごとのずれを計算シェーダーで解く
//! (提案 2026-09-15、利用者「わたしの推奨は gpu」「この天井を作るべきでない」)。
//!
//! 作者の file は `fn block(i: u32, p: BlockParams) -> Offset` だけを書く。箱の並び(`items`)・時刻と数(`host`)・
//! 欄の struct と、全員に掛ける外枠はここが manifest から組む。欄は float / long / bool で 24 個まで(fx の hook と同じ)。

use super::isf::IsfManifest;

/// 欄の slot の数(uniform の `array<vec4f, 6>`)。
pub(crate) const PARAM_SLOTS: usize = 24;

/// 1 つの物の箱。座標は親の空間。`room_*` は住む箱(Bounce の壁、並べる Group の箱)。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockItem {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub room_lo: [f32; 2],
    pub room_size: [f32; 2],
    pub radius: f32,
    pub index: u32,
    pub _pad: [f32; 2],
}

/// 物ごとのずれ(位置の差・回転の差(度)・大きさの倍率)。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockOffset {
    pub translate: [f32; 2],
    pub rotate: f32,
    pub scale: f32,
}

/// 箱の並びを storage buffer の byte に(`Item` の 48 byte、WGSL の並びと同じ)。
fn item_bytes(items: &[BlockItem]) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * 48);
    for it in items {
        for v in [it.lo[0], it.lo[1], it.hi[0], it.hi[1], it.room_lo[0], it.room_lo[1], it.room_size[0], it.room_size[1], it.radius] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&it.index.to_le_bytes());
        out.extend_from_slice(&[0u8; 8]);
    }
    out
}

fn wgsl_ident(name: &str) -> String {
    let mut out: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if out.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// 作者の本体に、型・束ね・欄の struct・外枠を足した計算シェーダーの全文。
pub(crate) fn module_source(manifest: &IsfManifest, body: &str) -> Result<String, String> {
    let names: Vec<String> = manifest.param_inputs().map(|p| wgsl_ident(&p.name)).collect();
    if names.len() > PARAM_SLOTS {
        return Err(format!("block の欄は {PARAM_SLOTS} 個まで"));
    }
    let mut out = String::from(
        "struct Item { lo: vec2f, hi: vec2f, room_lo: vec2f, room_size: vec2f, radius: f32, index: u32, _pad: vec2f };\n\
         struct Offset { translate: vec2f, rotate: f32, scale: f32 };\n\
         struct BlockHost { time: f32, count: u32, _pad: vec2f };\n\
         @group(0) @binding(0) var<storage, read> items: array<Item>;\n\
         @group(0) @binding(1) var<storage, read_write> offsets: array<Offset>;\n\
         @group(0) @binding(2) var<uniform> host: BlockHost;\n\
         @group(0) @binding(3) var<uniform> block_params: array<vec4f, 6>;\n\
         const NO_OFFSET: Offset = Offset(vec2f(0.0), 0.0, 1.0);\n\n",
    );
    out.push_str("struct BlockParams {\n");
    if names.is_empty() {
        out.push_str("    _unused: f32,\n");
    }
    for name in &names {
        out.push_str(&format!("    {name}: f32,\n"));
    }
    out.push_str("};\n\n");
    out.push_str(body);
    let args: Vec<String> = if names.is_empty() { vec!["0.0".into()] } else { (0..names.len()).map(|s| format!("block_params[{}][{}]", s / 4, s % 4)).collect() };
    out.push_str(&format!(
        "\n\n@compute @workgroup_size(64)\nfn motolii_block_main(@builtin(global_invocation_id) gid: vec3u) {{\n    \
         let i = gid.x;\n    if i >= host.count {{ return; }}\n    offsets[i] = block(i, BlockParams({}));\n}}\n",
        args.join(", ")
    ));
    Ok(out)
}

/// 計算シェーダーとして通るかを naga で確かめる(棚に載せる前)。
pub(crate) fn validate(source: &str) -> Result<(), String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| e.to_string())?;
    if !module.entry_points.iter().any(|e| e.name == "motolii_block_main" && e.stage == naga::ShaderStage::Compute) {
        return Err("block: 外枠が組めない".into());
    }
    Ok(())
}

/// 1 本のブロックの GPU の道。buffer は物の数が増えた時だけ作り直す。
pub(crate) struct BlockProgram {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    capacity: u64,
    items: Option<wgpu::Buffer>,
    offsets: Option<wgpu::Buffer>,
    host: wgpu::Buffer,
    params: wgpu::Buffer,
}

impl BlockProgram {
    pub(crate) fn new(device: &wgpu::Device, label: &str, source: &str) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(label), source: wgpu::ShaderSource::Wgsl(source.into()) });
        let storage = |binding, read_only| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let uniform = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries: &[storage(0, true), storage(1, false), uniform(2), uniform(3)] });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("motolii_block_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let uniform_buffer = |size| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { pipeline, layout, capacity: 0, items: None, offsets: None, host: uniform_buffer(16), params: uniform_buffer(96) }
    }

    /// 箱の並びと欄を書き、計算を記録する。結果は `offsets()` の buffer に残る(描く側が読み戻さずに束ねる)。
    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, time: f32, items: &[BlockItem], params: &[f32]) {
        let count = items.len().max(1) as u64;
        if count > self.capacity {
            let capacity = count.next_power_of_two();
            let buffer = |size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block"), size, usage, mapped_at_creation: false });
            self.items = Some(buffer(capacity * 48, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST));
            self.offsets = Some(buffer(capacity * 16, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC));
            self.capacity = capacity;
        }
        let (Some(item_buffer), Some(offset_buffer)) = (&self.items, &self.offsets) else { return };
        if !items.is_empty() {
            queue.write_buffer(item_buffer, 0, &item_bytes(items));
        }
        let mut host = [0u8; 16];
        host[0..4].copy_from_slice(&time.to_le_bytes());
        host[4..8].copy_from_slice(&(items.len() as u32).to_le_bytes());
        queue.write_buffer(&self.host, 0, &host);
        let mut slots = [0.0f32; PARAM_SLOTS];
        for (slot, value) in slots.iter_mut().zip(params) {
            *slot = *value;
        }
        queue.write_buffer(&self.params, 0, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: item_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: offset_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: self.host.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: self.params.as_entire_binding() },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((items.len() as u32).div_ceil(64).max(1), 1, 1);
    }

    /// 物ごとのずれの buffer(`record` の後)。
    pub(crate) fn offsets(&self) -> Option<&wgpu::Buffer> {
        self.offsets.as_ref()
    }
}

/// 試験の口: 計算して読み戻す(描く道は読み戻さない)。
#[cfg(test)]
pub(crate) fn run_and_read(device: &wgpu::Device, queue: &wgpu::Queue, program: &mut BlockProgram, time: f32, items: &[BlockItem], params: &[f32]) -> Vec<BlockOffset> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-block-test") });
    program.record(device, queue, &mut encoder, time, items, params);
    let bytes = (items.len() * 16) as u64;
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-read"), size: bytes, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    encoder.copy_buffer_to_buffer(program.offsets().unwrap(), 0, &staging, 0, bytes);
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let out = staging.slice(..).get_mapped_range().chunks_exact(16).map(|c| {
        let f = |i: usize| f32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]]);
        BlockOffset { translate: [f(0), f(4)], rotate: f(8), scale: f(12) }
    }).collect();
    staging.unmap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ブロックは fx と同じ棚に、段 Block として並ぶ(置き場・頭の JSON・読み直しが同じ)。
    #[test]
    fn a_block_sits_on_the_shelf_like_an_effect() {
        let refresh = crate::render::compositor::effects::catalog::refresh_effect_catalog();
        assert!(!refresh.errors.iter().any(|e| e.contains("bounce")), "{:?}", refresh.errors);
        let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
        let bounce = catalog.descriptors.iter().find(|d| d.plugin_id == "motolii.bounce_block").expect("on the shelf");
        assert_eq!(bounce.stage, crate::render::compositor::effects::catalog::EffectStage::Block);
        assert_eq!(bounce.params.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(), ["Strength"]);
    }

    /// Bounce を GPU のブロックに移しても、doc の CPU の折り返し(`layout::bounced`)と同じずれになる。
    #[test]
    fn the_bounce_block_folds_boxes_like_the_cpu_law() {
        let text = include_str!("../../../vism/bounce.wgsl");
        let (manifest, body) = super::super::isf::parse_isf_source(text).unwrap();
        assert_eq!(manifest.stage, super::super::isf::IsfStage::Block);
        let source = module_source(&manifest, &body).unwrap();
        validate(&source).unwrap();
        let engine = crate::render::engine::Engine::new().unwrap();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut program = BlockProgram::new(device, "bounce-test", &source);
        let m = 1.0; // doc の CANVAS_MARGIN(箱の素材座標の原点)
        let mut items = Vec::new();
        let mut rng = 7u32;
        let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
        for index in 0..4000u32 {
            let (size, radius) = if index % 2 == 0 { ([900.0, 520.0], 40.0) } else { ([420.0, 420.0], 210.0) };
            let w = 10.0 + next() * 60.0;
            let x = m - 800.0 + next() * 2400.0;
            let y = m - 800.0 + next() * 2400.0;
            items.push(BlockItem { lo: [x, y], hi: [x + w, y + w], room_lo: [m, m], room_size: size, radius, index, _pad: [0.0; 2] });
        }
        let gpu = run_and_read(device, queue, &mut program, 0.0, &items, &[1.0]);
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "the block pipeline and its buffers validate: {error:?}");
        for (item, offset) in items.iter().zip(&gpu) {
            let cpu = crate::doc::store::layout::bounced(item.lo, item.hi, item.room_size, item.radius);
            let d = (offset.translate[0] - cpu[0]).abs().max((offset.translate[1] - cpu[1]).abs());
            assert!(d < 0.05, "item {}: gpu {:?} cpu {cpu:?}", item.index, offset.translate);
        }
    }
}
