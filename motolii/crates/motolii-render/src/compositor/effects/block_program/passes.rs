//! ブロックの GPU の道: 計算 pipeline、1 コマの物の世界(buffer)、付いて行く・world へ写す・紐の 3 本の pass。
//! 札の文字は読まない — 受け取るのは組み上がった全文。

use super::*;

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
    neighbor_starts: wgpu::Buffer,
    neighbor_list: wgpu::Buffer,
    /// 今の state が `states` のどちらか。
    pub(crate) current: usize,
    pub(crate) count: u32,
}

impl BlockProgram {
    pub(crate) fn new(device: &wgpu::Device, label: &str, source: &str, rounds: u32) -> Self {
        let (pipeline, layout) = pipeline(device, label, source, &[storage(0, true), storage(1, true), uniform(2), uniform(3), storage(4, false), storage(5, true), storage(6, true), storage(7, true)], "motolii_block_main");
        Self { pipeline, layout, rounds: rounds.max(1) }
    }

    /// 掛かった物(`members`)に `rounds` 回掛ける。毎回、前の state を読み、写してから書く(掛からない物はそのまま)。
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32]) {
        self.record_from(device, queue, encoder, world, time, members, params, u32::MAX);
    }

    /// 場(`SCOPE: room`)は元の物の番号を `host.source` で渡す。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_from(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32], source: u32) {
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
            let host = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-host"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let mut h = [0u8; 32];
            h[0..4].copy_from_slice(&time.to_le_bytes());
            h[4..8].copy_from_slice(&(members.len() as u32).to_le_bytes());
            h[8..12].copy_from_slice(&world.count.to_le_bytes());
            h[12..16].copy_from_slice(&round.to_le_bytes());
            h[16..20].copy_from_slice(&source.to_le_bytes());
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
                    wgpu::BindGroupEntry { binding: 6, resource: world.neighbor_starts.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 7, resource: world.neighbor_list.as_entire_binding() },
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
            neighbor_starts: buffer(8, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            neighbor_list: buffer(4, wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST),
            current: 0,
            count: 0,
        }
    }

    /// このコマの物の箱を置き、state を 0(ずれ無し)から始める。
    pub(crate) fn begin(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[BlockItem], reach: f32) {
        self.begin_from(device, queue, items, reach, &[]);
    }

    /// 始まりの state を渡して始める(外の解き手 — Rapier — が出したずれを土台にする)。
    pub(crate) fn begin_from(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, items: &[BlockItem], reach: f32, seed: &[BlockOffset]) {
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
            let mut start = vec![BlockOffset::default(); items.len()];
            for (slot, value) in start.iter_mut().zip(seed) {
                *slot = *value;
            }
            queue.write_buffer(&self.states[0], 0, &offset_bytes(&start));
        }
        let (starts, list) = neighbors(items, reach);
        let storage = |bytes: &[u8]| {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-neighbors"), size: bytes.len().max(4) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            queue.write_buffer(&buffer, 0, bytes);
            buffer
        };
        self.neighbor_starts = storage(&starts.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        self.neighbor_list = storage(&list.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
    }

    pub(crate) fn state(&self) -> &wgpu::Buffer {
        &self.states[self.current]
    }
}

/// 付いて行く(付いて置く札の相手がブロックで動いた分だけ、札も動く): 対 (札, 相手) ごとに、札の state に相手の state を足す。
/// 1 回で 1 段(札の札は次の回)。
pub(crate) struct FollowPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

const FOLLOW_PASS: &str = "struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };\n\
@group(0) @binding(0) var<storage, read> state_in: array<Offset>;\n\
@group(0) @binding(1) var<storage, read_write> state_out: array<Offset>;\n\
@group(0) @binding(2) var<storage, read> pairs: array<vec2u>;\n\
@group(0) @binding(3) var<uniform> count: vec4u;\n\
@compute @workgroup_size(64)\n\
fn main(@builtin(global_invocation_id) gid: vec3u) {\n\
    if gid.x >= count.x { return; }\n\
    let pair = pairs[gid.x];\n\
    let own = state_in[pair.x];\n\
    state_out[pair.x] = Offset(own.translate + state_in[pair.y].translate, own.rotate, own.scale, own.tint);\n\
}\n";

impl FollowPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-follow", FOLLOW_PASS, &[storage(0, true), storage(1, false), storage(2, true), uniform(3)], "main");
        Self { pipeline, layout }
    }

    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, pairs: &[(u32, u32)]) {
        if pairs.is_empty() || world.count == 0 {
            return;
        }
        let pair_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-follow-pairs"), size: (pairs.len() * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&pair_buffer, 0, &pairs.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let count = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-follow-count"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&count, 0, &[(pairs.len() as u32).to_le_bytes(), [0; 4], [0; 4], [0; 4]].concat());
        let (from, to) = (world.current, 1 - world.current);
        encoder.copy_buffer_to_buffer(&world.states[from], 0, &world.states[to], 0, u64::from(world.count) * OFFSET_BYTES);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block-follow"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: world.states[from].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.states[to].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: pair_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: count.as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-follow"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((pairs.len() as u32).div_ceil(64), 1, 1);
        }
        world.current = to;
    }
}

/// state(comp のずれ)を、物ごとの comp → world の向きで world のずれにして描く側の motion へ書く(物ごと vec4 × 4:
/// 位置と回り、真ん中と大きさ、軸、色と不透明)。
pub(crate) struct WorldPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

const WORLD_PASS: &str = r#"struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };
struct Basis { u: vec4f, v: vec4f, centre: vec4f };
@group(0) @binding(0) var<storage, read> bases: array<Basis>;
@group(0) @binding(1) var<storage, read> state: array<Offset>;
@group(0) @binding(2) var<storage, read_write> motion: array<vec4f>;
@group(0) @binding(3) var<uniform> count: vec4u;
@group(0) @binding(4) var<storage, read> links: array<vec2u>;
fn world_offset(k: u32) -> vec3f {
    let t = state[k].translate;
    return bases[k].u.xyz * t.x + bases[k].v.xyz * t.y;
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let k = gid.x;
    if k >= count.x + count.y { return; }
    var base = k * 4u;
    if k >= count.x {
        base = (count.x + (k - count.x) * 2u) * 4u;
    }
    if base + 4u > arrayLength(&motion) { return; }
    if k >= count.x {
        // A connector: entry of kind 1 — offset at A, A, B − A, offset at B (see re_renderer motion_offset).
        let link = links[k - count.x];
        let a = bases[link.x].centre.xyz;
        let b = bases[link.y].centre.xyz;
        motion[base] = vec4f(world_offset(link.x), 0.0);
        motion[base + 1u] = vec4f(a, 1.0);
        motion[base + 2u] = vec4f(b - a, 1.0);
        motion[base + 3u] = vec4f(world_offset(link.y), 1.0);
        return;
    }
    let u = bases[k].u.xyz;
    let v = bases[k].v.xyz;
    let n = cross(u, v);
    let axis = select(vec3f(0.0, 0.0, 1.0), n / length(n), length(n) > 1e-12);
    motion[base] = vec4f(world_offset(k), radians(state[k].rotate));
    motion[base + 1u] = vec4f(bases[k].centre.xyz, state[k].scale);
    motion[base + 2u] = vec4f(axis, 0.0);
    motion[base + 3u] = state[k].tint;
}
"#;

impl WorldPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-world-pass", WORLD_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3), storage(4, true)], "main");
        Self { pipeline, layout }
    }

    /// `bases` は物ごとの (comp の x の 1px の world, y の 1px の world, 物の真ん中の world)。真ん中は回る軸が通る所。
    /// `links` はつなぐ線ごとの (元の物, 先の物): 物の後ろに 2 項ずつ(kind 1 は前の 1 項、紐は 2 項)両端のずれを書く。
    /// 返すのは bases の buffer(紐の解きが同じ物を読む)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &BlockWorld, bases: &[([f32; 3], [f32; 3], [f32; 3])], links: &[(u32, u32)], motion: &wgpu::Buffer) -> Option<wgpu::Buffer> {
        if world.count == 0 {
            return None;
        }
        let basis = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-bases"), size: (bases.len().max(1) * 48) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&basis, 0, &bases.iter().flat_map(|(u, v, c)| [u[0], u[1], u[2], 0.0, v[0], v[1], v[2], 0.0, c[0], c[1], c[2], 0.0]).flat_map(f32::to_le_bytes).collect::<Vec<u8>>());
        let count = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-count"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&count, 0, &[world.count.to_le_bytes(), (links.len() as u32).to_le_bytes(), [0; 4], [0; 4]].concat());
        let link_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-block-links"), size: (links.len().max(1) * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&link_buffer, 0, &links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-block-world-pass"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: basis.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.state().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: count.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: link_buffer.as_entire_binding() },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-world-pass"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups((world.count + links.len() as u32).div_ceil(64), 1, 1);
        drop(pass);
        Some(basis)
    }
}

/// 紐(Line Path = Rope): つなぐ線の 3 次ベジェの腹の 2 点だけがばね + 減衰で遅れて付いて来る(利用者 2026-09-18
/// 「紐が物理の挙動になれば」「ベジェでいけないの?」)。状態は紐 1 本に 2 点の位置と速度、GPU に置いたまま
/// (読み戻さない)。描く側の motion の kind 2 の項(つなぐ線の 2 つ目の項)に、描いた時の腹と今の腹を書く。
pub(crate) struct RopePass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    states: [wgpu::Buffer; 2],
    capacity: u64,
    current: usize,
    /// 最後に解いたコマ(続きのコマでなければ静止から始める)。
    pub(crate) last_frame: Option<i64>,
}

const ROPE_PASS: &str = r#"struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };
struct Basis { u: vec4f, v: vec4f, centre: vec4f };
struct Host { objects: u32, ropes: u32, dt: f32, reset: u32 };
@group(0) @binding(0) var<storage, read> bases: array<Basis>;
@group(0) @binding(1) var<storage, read> state: array<Offset>;
@group(0) @binding(2) var<storage, read_write> motion: array<vec4f>;
@group(0) @binding(3) var<uniform> host: Host;
@group(0) @binding(4) var<storage, read> links: array<vec2u>;
@group(0) @binding(5) var<storage, read> ropes: array<vec4f>;
@group(0) @binding(6) var<storage, read> rope_in: array<vec4f>;
@group(0) @binding(7) var<storage, read_write> rope_out: array<vec4f>;
fn world_offset(k: u32) -> vec3f {
    let t = state[k].translate;
    return bases[k].u.xyz * t.x + bases[k].v.xyz * t.y;
}
fn controls(a: vec3f, b: vec3f, down: vec3f, sag: f32) -> array<vec3f, 2> {
    let d = b - a;
    return array<vec3f, 2>(a + d / 3.0 + down * sag, a + d * (2.0 / 3.0) + down * sag);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let r = gid.x;
    if r >= host.ropes { return; }
    let rope = ropes[r];
    let c = u32(rope.x);
    let link = links[c];
    let a = bases[link.x].centre.xyz;
    let b = bases[link.y].centre.xyz;
    let down = normalize(bases[link.x].v.xyz);
    let sag = length(b - a) * rope.y / 100.0;
    let drawn = controls(a, b, down, sag);
    let la = a + world_offset(link.x);
    let lb = b + world_offset(link.y);
    let rest = controls(la, lb, down, sag);
    var p0 = rope_in[r * 4u].xyz;
    var v0 = rope_in[r * 4u + 1u].xyz;
    var p1 = rope_in[r * 4u + 2u].xyz;
    var v1 = rope_in[r * 4u + 3u].xyz;
    if host.reset == 1u {
        p0 = rest[0]; v0 = vec3f(0.0); p1 = rest[1]; v1 = vec3f(0.0);
    }
    // ばね(硬さ rope.z)と減衰(rope.w)、4 回に刻む。時刻の関数ではなく解き手の側(記憶を持つ)。
    let dt = host.dt / 4.0;
    for (var i = 0u; i < 4u; i = i + 1u) {
        v0 = v0 + (rest[0] - p0) * rope.z * dt - v0 * rope.w * dt;
        p0 = p0 + v0 * dt;
        v1 = v1 + (rest[1] - p1) * rope.z * dt - v1 * rope.w * dt;
        p1 = p1 + v1 * dt;
    }
    rope_out[r * 4u] = vec4f(p0, 0.0);
    rope_out[r * 4u + 1u] = vec4f(v0, 0.0);
    rope_out[r * 4u + 2u] = vec4f(p1, 0.0);
    rope_out[r * 4u + 3u] = vec4f(v1, 0.0);
    let base = (host.objects + c * 2u) * 4u;
    if base + 8u > arrayLength(&motion) { return; }
    motion[base + 2u] = vec4f(b - a, 2.0);
    motion[base + 4u] = vec4f(drawn[0], 0.0);
    motion[base + 5u] = vec4f(drawn[1], 0.0);
    motion[base + 6u] = vec4f(p0, 0.0);
    motion[base + 7u] = vec4f(p1, 0.0);
}
"#;

impl RopePass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-rope-pass", ROPE_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3), storage(4, true), storage(5, true), storage(6, true), storage(7, false)], "main");
        let buffer = |device: &wgpu::Device, size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-state"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { pipeline, layout, states: [buffer(device, 64), buffer(device, 64)], capacity: 1, current: 0, last_frame: None }
    }

    /// `ropes` は (つなぐ線の番号, Slack %, 硬さ, 減衰)。`frame` が前の続きでなければ静止から始める。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &BlockWorld, bases_buffer: &wgpu::Buffer, links: &[(u32, u32)], ropes: &[(u32, f32, f32, f32)], motion: &wgpu::Buffer, frame: i64, fps: f32) {
        if ropes.is_empty() || world.count == 0 {
            self.last_frame = None;
            return;
        }
        let count = ropes.len() as u64;
        let mut reset = self.last_frame != Some(frame - 1);
        if count > self.capacity {
            self.capacity = count.next_power_of_two();
            let buffer = |size: u64| device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-state"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            self.states = [buffer(self.capacity * 64), buffer(self.capacity * 64)];
            reset = true;
        }
        self.last_frame = Some(frame);
        let link_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-links"), size: (links.len().max(1) * 8) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&link_buffer, 0, &links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let rope_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-ropes"), size: count * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&rope_buffer, 0, &ropes.iter().flat_map(|(c, slack, k, d)| [(*c as f32).to_le_bytes(), slack.to_le_bytes(), k.to_le_bytes(), d.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let host = device.create_buffer(&wgpu::BufferDescriptor { label: Some("motolii-rope-host"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&host, 0, &[world.count.to_le_bytes(), (count as u32).to_le_bytes(), (1.0 / fps.max(1.0)).to_le_bytes(), (reset as u32).to_le_bytes()].concat());
        let (from, to) = (self.current, 1 - self.current);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-rope-pass"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: bases_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: world.state().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: host.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: link_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: rope_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: self.states[from].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 7, resource: self.states[to].as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-rope-pass"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
        }
        self.current = to;
    }
}

/// 試験と計測の口: state を読み戻す(描く道は読み戻さない)。
pub fn read_state(device: &wgpu::Device, queue: &wgpu::Queue, world: &BlockWorld, encoder: wgpu::CommandEncoder) -> Vec<BlockOffset> {
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
