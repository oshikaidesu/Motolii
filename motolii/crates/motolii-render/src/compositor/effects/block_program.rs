//! 箱のブロック(`"STAGE": "block"`): 物の箱の並びと、全員の今のずれ(state)を GPU の storage buffer に置き、
//! ブロックが state を少しずつ書き換える計算シェーダー(提案 2026-09-15、利用者「わたしの推奨は gpu」「この天井を作るべきでない」)。
//!
//! 作者の file は `fn block(k: u32, p: BlockParams) -> Offset` だけを書く。`k` は物の番号、返すのは足すずれ。
//! 読める物: `objects[k]`(箱・住む箱・間合い・重み・組)、`host`(時刻・物の数・何回目)、`now_lo(k)` / `now_hi(k)`(今のずれ込みの箱)。
//! 欄の struct と、掛かった物(`members`)全員に掛ける外枠、ずれを state へ足す所はここが manifest から組む。
//! `"ROUNDS": n` なら n 回続けて解く(毎回、前の回の state を読む)。

use super::isf::IsfManifest;

/// 欄の slot の数(uniform の `array<vec4f, 6>`)。
pub(crate) const PARAM_SLOTS: usize = 24;

/// 1 つの物の箱(comp の座標、描く時と同じ置き方)。`room_*` は住む箱(並べる Group の箱)。
/// `group` は同じ住む箱に居る物の印(押し合いは同じ組の中だけ)、`margin` は間合い、`weight` は譲る比。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockItem {
    pub lo: [f32; 2],
    pub hi: [f32; 2],
    pub room_lo: [f32; 2],
    pub room_size: [f32; 2],
    pub radius: f32,
    pub group: u32,
    pub margin: f32,
    pub weight: f32,
}

/// 物ごとのずれ(位置の差・回転の差(度)・大きさの倍率)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockOffset {
    pub translate: [f32; 2],
    pub rotate: f32,
    pub scale: f32,
}

impl Default for BlockOffset {
    fn default() -> Self {
        Self { translate: [0.0; 2], rotate: 0.0, scale: 1.0 }
    }
}

/// 1 つの物の byte(WGSL の `Item` と同じ並び)。
pub(crate) const ITEM_BYTES: u64 = 48;
pub(crate) const OFFSET_BYTES: u64 = 16;

pub(crate) fn item_bytes(items: &[BlockItem]) -> Vec<u8> {
    let mut out = Vec::with_capacity(items.len() * ITEM_BYTES as usize);
    for it in items {
        for v in [it.lo[0], it.lo[1], it.hi[0], it.hi[1], it.room_lo[0], it.room_lo[1], it.room_size[0], it.room_size[1], it.radius] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&it.group.to_le_bytes());
        out.extend_from_slice(&it.margin.to_le_bytes());
        out.extend_from_slice(&it.weight.to_le_bytes());
    }
    out
}

pub(crate) fn offset_bytes(offsets: &[BlockOffset]) -> Vec<u8> {
    offsets.iter().flat_map(|o| [o.translate[0], o.translate[1], o.rotate, o.scale]).flat_map(f32::to_le_bytes).collect()
}

pub(crate) fn offsets_from_bytes(bytes: &[u8]) -> Vec<BlockOffset> {
    bytes.chunks_exact(16).map(|c| {
        let f = |i: usize| f32::from_le_bytes([c[i], c[i + 1], c[i + 2], c[i + 3]]);
        BlockOffset { translate: [f(0), f(4)], rotate: f(8), scale: f(12) }
    }).collect()
}

/// 作者も外枠も同じ型と束ねを読む。
pub(crate) const PRELUDE: &str = "struct Item { lo: vec2f, hi: vec2f, room_lo: vec2f, room_size: vec2f, radius: f32, group: u32, margin: f32, weight: f32 };\n\
struct Offset { translate: vec2f, rotate: f32, scale: f32 };\n\
struct BlockHost { time: f32, members: u32, objects: u32, round: u32 };\n\
@group(0) @binding(0) var<storage, read> objects: array<Item>;\n\
@group(0) @binding(1) var<storage, read> state_in: array<Offset>;\n\
@group(0) @binding(2) var<uniform> host: BlockHost;\n\
@group(0) @binding(3) var<uniform> block_params: array<vec4f, 6>;\n\
@group(0) @binding(4) var<storage, read_write> state_out: array<Offset>;\n\
@group(0) @binding(5) var<storage, read> members: array<u32>;\n\
const NO_OFFSET: Offset = Offset(vec2f(0.0), 0.0, 1.0);\n\
fn now_lo(k: u32) -> vec2f { return objects[k].lo + state_in[k].translate; }\n\
fn now_hi(k: u32) -> vec2f { return objects[k].hi + state_in[k].translate; }\n\n";

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
    let mut out = String::from(PRELUDE);
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
        "\n\n@compute @workgroup_size(64)\nfn motolii_block_main(@builtin(global_invocation_id) gid: vec3u) {{\n\
         \x20   if gid.x >= host.members {{ return; }}\n\
         \x20   let k = members[gid.x];\n\
         \x20   let d = block(k, BlockParams({}));\n\
         \x20   let s = state_in[k];\n\
         \x20   state_out[k] = Offset(s.translate + d.translate, s.rotate + d.rotate, s.scale * d.scale);\n}}\n",
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

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn uniform(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn pipeline(device: &wgpu::Device, label: &str, source: &str, entries: &[wgpu::BindGroupLayoutEntry], entry: &str) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some(label), source: wgpu::ShaderSource::Wgsl(source.into()) });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some(entry),
        compilation_options: Default::default(),
        cache: None,
    });
    (pipeline, layout)
}

/// 1 本のブロックの GPU の道。
pub(crate) struct BlockProgram {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    pub(crate) rounds: u32,
}

/// 1 コマの物の世界(全員の箱・state の 2 本・掛かった物の番号)。buffer は物の数が増えた時だけ作り直す。
pub(crate) struct BlockWorld {
    capacity: u64,
    objects: wgpu::Buffer,
    states: [wgpu::Buffer; 2],
    /// 今の state が `states` のどちらか。
    pub(crate) current: usize,
    pub(crate) count: u32,
}

impl BlockProgram {
    pub(crate) fn new(device: &wgpu::Device, label: &str, source: &str, rounds: u32) -> Self {
        let (pipeline, layout) = pipeline(device, label, source, &[storage(0, true), storage(1, true), uniform(2), uniform(3), storage(4, false), storage(5, true)], "motolii_block_main");
        Self { pipeline, layout, rounds: rounds.max(1) }
    }

    /// 掛かった物(`members`)に `rounds` 回掛ける。毎回、前の state を読み、写してから書く(掛からない物はそのまま)。
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32]) {
        if members.is_empty() || world.count == 0 {
            return;
        }
        // 掛かった物の番号も同じ理由でブロックごとに別の buffer。
        let members_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-members"), size: (members.len() * 4) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&members_buffer, 0, &members.iter().flat_map(|m| m.to_le_bytes()).collect::<Vec<u8>>());
        let mut slots = [0.0f32; PARAM_SLOTS];
        for (slot, value) in slots.iter_mut().zip(params) {
            *slot = *value;
        }
        // uniform の書き込みは submit の時にまとめて届き、同じ buffer は最後の値が勝つ。ブロックごとの欄は別の buffer で渡す。
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-params"), size: 96, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&params_buffer, 0, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let bytes = u64::from(world.count) * OFFSET_BYTES;
        for round in 0..self.rounds {
            // uniform は submit 前の最後の書き込みが勝つので、回ごとに違う host は別の小さな buffer で渡す。
            let host = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-host"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let mut h = [0u8; 16];
            h[0..4].copy_from_slice(&time.to_le_bytes());
            h[4..8].copy_from_slice(&(members.len() as u32).to_le_bytes());
            h[8..12].copy_from_slice(&world.count.to_le_bytes());
            h[12..16].copy_from_slice(&round.to_le_bytes());
            queue.write_buffer(&host, 0, &h);
            let (from, to) = (world.current, 1 - world.current);
            encoder.copy_buffer_to_buffer(&world.states[from], 0, &world.states[to], 0, bytes);
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("motolii-block"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: world.objects.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: world.states[from].as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: host.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: params_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: world.states[to].as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 5, resource: members_buffer.as_entire_binding() },
                ],
            });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block"), timestamp_writes: None });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups((members.len() as u32).div_ceil(64), 1, 1);
            }
            world.current = to;
        }
    }
}

impl BlockWorld {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let buffer = |size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-world"), size, usage, mapped_at_creation: false });
        let state = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        Self {
            capacity: 1,
            objects: buffer(ITEM_BYTES, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            states: [buffer(OFFSET_BYTES, state), buffer(OFFSET_BYTES, state)],
            current: 0,
            count: 0,
        }
    }

    /// このコマの物の箱を置き、state を 0(ずれ無し)から始める。
    pub(crate) fn begin(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[BlockItem]) {
        let count = items.len().max(1) as u64;
        if count > self.capacity {
            self.capacity = count.next_power_of_two();
            let buffer = |size: u64, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-world"), size, usage, mapped_at_creation: false });
            let state = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
            self.objects = buffer(self.capacity * ITEM_BYTES, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST);
            self.states = [buffer(self.capacity * OFFSET_BYTES, state), buffer(self.capacity * OFFSET_BYTES, state)];
        }
        self.count = items.len() as u32;
        self.current = 0;
        if !items.is_empty() {
            queue.write_buffer(&self.objects, 0, &item_bytes(items));
            queue.write_buffer(&self.states[0], 0, &offset_bytes(&vec![BlockOffset::default(); items.len()]));
        }
    }

    pub(crate) fn state(&self) -> &wgpu::Buffer {
        &self.states[self.current]
    }
}

/// state(comp のずれ)を、物ごとの comp → world の向きで world のずれにして描く側の motion へ書く。
pub(crate) struct WorldPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

const WORLD_PASS: &str = "struct Offset { translate: vec2f, rotate: f32, scale: f32 };\n\
struct Basis { u: vec4f, v: vec4f };\n\
@group(0) @binding(0) var<storage, read> bases: array<Basis>;\n\
@group(0) @binding(1) var<storage, read> state: array<Offset>;\n\
@group(0) @binding(2) var<storage, read_write> motion: array<vec4f>;\n\
@group(0) @binding(3) var<uniform> count: vec4u;\n\
@compute @workgroup_size(64)\n\
fn main(@builtin(global_invocation_id) gid: vec3u) {\n\
    let k = gid.x;\n\
    if k >= count.x || k >= arrayLength(&motion) { return; }\n\
    let t = state[k].translate;\n\
    motion[k] = vec4f(bases[k].u.xyz * t.x + bases[k].v.xyz * t.y, 0.0);\n\
}\n";

impl WorldPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-world-pass", WORLD_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3)], "main");
        Self { pipeline, layout }
    }

    /// `bases` は物ごとの (comp の x の 1px の world, y の 1px の world)。
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &BlockWorld, bases: &[([f32; 3], [f32; 3])], motion: &wgpu::Buffer) {
        if world.count == 0 {
            return;
        }
        let basis = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-bases"), size: (bases.len().max(1) * 32) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&basis, 0, &bases.iter().flat_map(|(u, v)| [u[0], u[1], u[2], 0.0, v[0], v[1], v[2], 0.0]).flat_map(f32::to_le_bytes).collect::<Vec<u8>>());
        let count = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-count"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&count, 0, &[world.count.to_le_bytes(), [0; 4], [0; 4], [0; 4]].concat());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block-world-pass"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: basis.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.state().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: count.as_entire_binding() },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-world-pass"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(world.count.div_ceil(64), 1, 1);
    }
}

/// 試験の口: state を読み戻す(描く道は読み戻さない)。
#[cfg(test)]
pub(crate) fn read_state(device: &wgpu::Device, queue: &wgpu::Queue, world: &BlockWorld, encoder: wgpu::CommandEncoder) -> Vec<BlockOffset> {
    let mut encoder = encoder;
    let bytes = u64::from(world.count) * OFFSET_BYTES;
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-read"), size: bytes, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    encoder.copy_buffer_to_buffer(world.state(), 0, &staging, 0, bytes);
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let out = offsets_from_bytes(&staging.slice(..).get_mapped_range());
    staging.unmap();
    out
}

#[cfg(test)]
pub(crate) fn program_for(device: &wgpu::Device, file: &str) -> BlockProgram {
    let (manifest, body) = super::isf::parse_isf_source(file).unwrap();
    assert_eq!(manifest.stage, super::isf::IsfStage::Block);
    let source = module_source(&manifest, &body).unwrap();
    validate(&source).unwrap();
    BlockProgram::new(device, "block-test", &source, manifest.rounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu() -> crate::render::engine::Engine {
        crate::render::engine::Engine::new().unwrap()
    }

    /// Bounce を GPU のブロックに移しても、doc の CPU の折り返し(`layout::bounced`)と同じずれになる。
    #[test]
    fn the_bounce_block_folds_boxes_like_the_cpu_law() {
        let engine = gpu();
        let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let program = program_for(device, include_str!("../../../vism/bounce.wgsl"));
        let m = 1.0; // doc の CANVAS_MARGIN(箱の素材座標の原点)
        let mut items = Vec::new();
        let mut rng = 7u32;
        let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
        for index in 0..4000u32 {
            let (size, radius) = if index % 2 == 0 { ([900.0, 520.0], 40.0) } else { ([420.0, 420.0], 210.0) };
            let w = 10.0 + next() * 60.0;
            let x = m - 800.0 + next() * 2400.0;
            let y = m - 800.0 + next() * 2400.0;
            items.push(BlockItem { lo: [x, y], hi: [x + w, y + w], room_lo: [m, m], room_size: size, radius, group: 0, margin: 0.0, weight: 1.0 });
        }
        let mut world = BlockWorld::new(device);
        world.begin(device, queue, &items);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
        let members: Vec<u32> = (0..items.len() as u32).collect();
        program.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[1.0]);
        let gpu = read_state(device, queue, &world, encoder);
        let error = pollster::block_on(scope.pop());
        assert!(error.is_none(), "the block pipeline and its buffers validate: {error:?}");
        for (k, (item, offset)) in items.iter().zip(&gpu).enumerate() {
            let cpu = crate::doc::store::layout::bounced(item.lo, item.hi, item.room_size, item.radius);
            let d = (offset.translate[0] - cpu[0]).abs().max((offset.translate[1] - cpu[1]).abs());
            assert!(d < 0.05, "item {k}: gpu {:?} cpu {cpu:?}", offset.translate);
        }
    }

    /// ブロックは fx と同じ棚に、段 Block として並ぶ(置き場・頭の JSON・読み直しが同じ)。
    #[test]
    fn a_block_sits_on_the_shelf_like_an_effect() {
        let refresh = crate::render::compositor::effects::catalog::refresh_effect_catalog();
        assert!(!refresh.errors.iter().any(|e| e.contains("bounce") || e.contains("push_apart")), "{:?}", refresh.errors);
        let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
        let bounce = catalog.descriptors.iter().find(|d| d.plugin_id == "motolii.bounce_block").expect("on the shelf");
        assert_eq!(bounce.stage, crate::render::compositor::effects::catalog::EffectStage::Block);
        assert_eq!(bounce.params.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(), ["Strength"]);
    }
}
