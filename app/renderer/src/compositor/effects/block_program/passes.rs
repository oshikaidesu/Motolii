//! ブロックの GPU の道: 計算 pipeline、1 コマの物の世界(buffer)、付いて行く・world へ写す・紐の 3 本の pass。
//! 札の文字は読まない — 受け取るのは組み上がった全文。
//!
//! buffer と bind group は世界が持ち、**伸びた時だけ**作り直す。回ごと・ブロックごとに違う uniform は
//! 1 本の buffer の升目に並べて dynamic offset で指す(submit では同じ場所の最後の書き込みが勝つ)。
//! dispatch は device の `max_compute_workgroups_per_dimension` で切って、続きをもう 1 回投げる(黙って切らない)。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

/// 1 つの workgroup の invocation(全ての計算シェーダーの `@workgroup_size(64)`)。
const GROUP: u32 = 64;
/// 欄の uniform(`array<vec4f, 6>`)。
const PARAMS_BYTES: u64 = 96;
/// `BlockHost`(uniform の struct は 16 の倍数に丸まる)。
const HOST_BYTES: u64 = 32;
/// `vec4u` 1 つ(数と、切った時の頭)。
const COUNT_BYTES: u64 = 16;
/// 紐の `Host`。
const ROPE_HOST_BYTES: u64 = 32;

/// 道ごとの番号(世界に bind group を覚えさせる鍵)。
fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

/// 升目に並べた uniform の 1 升(指す場所は `set_bind_group` の dynamic offset で決める)。
fn uniform(binding: u32, size: u64) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: wgpu::BufferSize::new(size) },
        count: None,
    }
}

/// 升 1 つ分だけを指す binding(dynamic offset を使う binding は大きさを言わないといけない)。
fn slot_of(buffer: &wgpu::Buffer, size: u64) -> wgpu::BindingResource<'_> {
    wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer, offset: 0, size: wgpu::BufferSize::new(size) })
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

/// dispatch を 1 軸の天井で切る: (頭の番号, その回で見る数)。天井を超えても黙って切らず、続きを投げる。
fn spans(total: u32, max_groups: u32) -> impl Iterator<Item = (u32, u32)> {
    let per = max_groups.saturating_mul(GROUP).max(GROUP);
    (0..total.div_ceil(per)).map(move |i| (i * per, per.min(total - i * per)))
}

fn raw(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false })
}

/// 伸びた時だけ作り直す buffer。毎コマするのは `write_buffer` だけ。
struct Pool {
    buffer: wgpu::Buffer,
    label: &'static str,
    usage: wgpu::BufferUsages,
}

impl Pool {
    fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> Self {
        Self { buffer: raw(device, label, usage, size.max(4).next_power_of_two()), label, usage }
    }

    /// 要る大きさに足りなければ作り直す。作り直したら true — その buffer を指していた bind group は捨てる印。
    fn fit(&mut self, device: &wgpu::Device, bytes: u64, retired: &mut Vec<wgpu::Buffer>) -> bool {
        if self.buffer.size() >= bytes {
            return false;
        }
        let grown = raw(device, self.label, self.usage, bytes.max(4).next_power_of_two());
        retired.push(std::mem::replace(&mut self.buffer, grown));
        true
    }
}

/// 1 本のブロックの GPU の道。
pub(crate) struct BlockProgram {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    id: u64,
    pub(crate) rounds: u32,
}

/// 1 コマの物の世界。物の箱・state の 2 本・近くの物・掛かった物の番号・uniform の升目を持ち、
/// どれも**物が増えた時だけ**作り直す。bind group も同じ — 指し先が変わらない限り使い回す。
pub(crate) struct BlockWorld {
    objects: Pool,
    states: [Pool; 2],
    neighbor_starts: Pool,
    neighbor_list: Pool,
    /// このコマの全ての batch の「掛かった物の番号」を続けて並べる(batch ごとの buffer を作らない)。
    members: Pool,
    members_used: u64,
    /// このコマの uniform(host・欄・数)を升目に並べる。回ごと・block ごとに升を変える。
    uniforms: Pool,
    uniform_used: u64,
    /// 1 升の byte(device の `min_uniform_buffer_offset_alignment` の倍数)。
    stride: u64,
    /// 物ごとの comp → world の向きと、つなぐ線の両端(world へ写す道と紐が同じ物を読む)。
    bases: Pool,
    links: Pool,
    /// 作り直した buffer は、同じコマに記録済みの pass がまだ読む。submit まで生かす。
    retired: Vec<wgpu::Buffer>,
    /// 指し先が変わらない限り使い回す bind group((道の番号, ping-pong の側))。
    binds: HashMap<(u64, usize), wgpu::BindGroup>,
    /// dispatch の 1 軸の天井。
    max_groups: u32,
    /// 今の state が `states` のどちらか。
    pub(crate) current: usize,
    pub(crate) count: u32,
}

impl BlockProgram {
    pub(crate) fn new(device: &wgpu::Device, label: &str, source: &str, rounds: u32) -> Self {
        let (pipeline, layout) = pipeline(device, label, source, &[storage(0, true), storage(1, true), uniform(2, HOST_BYTES), uniform(3, PARAMS_BYTES), storage(4, false), storage(5, true), storage(6, true), storage(7, true)], "motolii_block_main");
        Self { pipeline, layout, id: next_id(), rounds: rounds.max(1) }
    }

    /// 掛かった物(`members`)に `rounds` 回掛ける。毎回、前の回の state を読み、書く先は反対側
    /// (掛からない物はこのブロックが 1 度も書かないので、写すのは呼び出しごとに 1 回で足りる)。
    pub(crate) fn record(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32]) {
        self.record_from(device, queue, encoder, world, time, members, params, u32::MAX);
    }

    /// 場(`SCOPE: room`)は元の物の番号を `host.source` で渡す。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_from(&self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32], source: u32) {
        if members.is_empty() || world.count == 0 {
            return;
        }
        // 掛かった物の番号は、このコマの並びの後ろへ足すだけ(頭の番号を host で渡す)。
        let base = world.members_span(device, queue, members);
        let chunks: Vec<(u32, u32)> = spans(members.len() as u32, world.max_groups).collect();
        world.reserve(device, 1 + u64::from(self.rounds) * chunks.len() as u64);
        let mut slots = [0.0f32; PARAM_SLOTS];
        for (slot, value) in slots.iter_mut().zip(params) {
            *slot = *value;
        }
        let at_params = world.slot(queue, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let bytes = u64::from(world.count) * OFFSET_BYTES;
        encoder.copy_buffer_to_buffer(&world.states[world.current].buffer, 0, &world.states[1 - world.current].buffer, 0, bytes);
        for round in 0..self.rounds {
            let (from, to) = (world.current, 1 - world.current);
            for (at, take) in chunks.iter().copied() {
                // `members` は掛かった物の数(札が読む意味は変わらない)。`base` と `last` は共有の並びの中の、この回の頭と終わり。
                let mut h = [0u8; 28];
                h[0..4].copy_from_slice(&time.to_le_bytes());
                h[4..8].copy_from_slice(&(members.len() as u32).to_le_bytes());
                h[8..12].copy_from_slice(&world.count.to_le_bytes());
                h[12..16].copy_from_slice(&round.to_le_bytes());
                h[16..20].copy_from_slice(&source.to_le_bytes());
                h[20..24].copy_from_slice(&(base + at).to_le_bytes());
                h[24..28].copy_from_slice(&(base + members.len() as u32).to_le_bytes());
                let at_host = world.slot(queue, &h);
                let bind = match world.bind((self.id, from)) {
                    Some(bind) => bind,
                    None => {
                        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("motolii-block"),
                            layout: &self.layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: world.objects.buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 1, resource: world.states[from].buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 2, resource: slot_of(&world.uniforms.buffer, HOST_BYTES) },
                                wgpu::BindGroupEntry { binding: 3, resource: slot_of(&world.uniforms.buffer, PARAMS_BYTES) },
                                wgpu::BindGroupEntry { binding: 4, resource: world.states[to].buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 5, resource: world.members.buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 6, resource: world.neighbor_starts.buffer.as_entire_binding() },
                                wgpu::BindGroupEntry { binding: 7, resource: world.neighbor_list.buffer.as_entire_binding() },
                            ],
                        });
                        world.remember((self.id, from), bind)
                    }
                };
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block"), timestamp_writes: None });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind, &[at_host, at_params]);
                pass.dispatch_workgroups(take.div_ceil(GROUP), 1, 1);
            }
            world.current = to;
        }
    }
}

impl BlockWorld {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let store = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let state = store | wgpu::BufferUsages::COPY_SRC;
        let limits = device.limits();
        let align = u64::from(limits.min_uniform_buffer_offset_alignment);
        Self {
            objects: Pool::new(device, "motolii-block-objects", store, ITEM_BYTES),
            states: [Pool::new(device, "motolii-block-state", state, OFFSET_BYTES), Pool::new(device, "motolii-block-state", state, OFFSET_BYTES)],
            neighbor_starts: Pool::new(device, "motolii-block-neighbor-starts", store, 8),
            neighbor_list: Pool::new(device, "motolii-block-neighbor-list", store, 4),
            members: Pool::new(device, "motolii-block-members", store, 4),
            members_used: 0,
            uniforms: Pool::new(device, "motolii-block-uniforms", wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, align.max(PARAMS_BYTES)),
            uniform_used: 0,
            stride: PARAMS_BYTES.div_ceil(align) * align,
            bases: Pool::new(device, "motolii-block-bases", store, 48),
            links: Pool::new(device, "motolii-block-links", store, 8),
            retired: Vec::new(),
            binds: HashMap::new(),
            max_groups: limits.max_compute_workgroups_per_dimension,
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
        self.retired.clear();
        self.members_used = 0;
        self.uniform_used = 0;
        let count = items.len().max(1) as u64;
        let mut fresh = self.objects.fit(device, count * ITEM_BYTES, &mut self.retired);
        fresh |= self.states[0].fit(device, count * OFFSET_BYTES, &mut self.retired);
        fresh |= self.states[1].fit(device, count * OFFSET_BYTES, &mut self.retired);
        self.count = items.len() as u32;
        self.current = 0;
        if !items.is_empty() {
            queue.write_buffer(&self.objects.buffer, 0, &item_bytes(items));
            let mut start = vec![BlockOffset::default(); items.len()];
            for (slot, value) in start.iter_mut().zip(seed) {
                *slot = *value;
            }
            queue.write_buffer(&self.states[0].buffer, 0, &offset_bytes(&start));
        }
        let (starts, list) = neighbors(items, reach);
        let starts: Vec<u8> = starts.iter().flat_map(|v| v.to_le_bytes()).collect();
        let list: Vec<u8> = list.iter().flat_map(|v| v.to_le_bytes()).collect();
        fresh |= self.neighbor_starts.fit(device, starts.len() as u64, &mut self.retired);
        fresh |= self.neighbor_list.fit(device, list.len() as u64, &mut self.retired);
        if !starts.is_empty() {
            queue.write_buffer(&self.neighbor_starts.buffer, 0, &starts);
        }
        if !list.is_empty() {
            queue.write_buffer(&self.neighbor_list.buffer, 0, &list);
        }
        if fresh {
            self.binds.clear();
        }
    }

    pub(crate) fn state(&self) -> &wgpu::Buffer {
        &self.states[self.current].buffer
    }

    /// 掛かった物の番号をこのコマの並びの後ろへ置く。返すのはその頭(番号の位置)。
    fn members_span(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, members: &[u32]) -> u32 {
        let start = self.members_used;
        if self.members.fit(device, (start + members.len() as u64) * 4, &mut self.retired) {
            self.binds.clear();
        }
        queue.write_buffer(&self.members.buffer, start * 4, &members.iter().flat_map(|m| m.to_le_bytes()).collect::<Vec<u8>>());
        self.members_used = start + members.len() as u64;
        start as u32
    }

    /// uniform の升を `slots` 個ぶん先に確かめる(取り始めた後に作り直すと、先に書いた升が迷子になる)。
    fn reserve(&mut self, device: &wgpu::Device, slots: u64) {
        if self.uniforms.fit(device, self.uniform_used + slots * self.stride, &mut self.retired) {
            self.binds.clear();
        }
    }

    /// 升を 1 つ取って書く。返すのは `set_bind_group` に渡す dynamic offset。
    fn slot(&mut self, queue: &wgpu::Queue, bytes: &[u8]) -> u32 {
        let at = self.uniform_used;
        debug_assert!(at + self.stride <= self.uniforms.buffer.size(), "升を取る前に reserve する");
        queue.write_buffer(&self.uniforms.buffer, at, bytes);
        self.uniform_used = at + self.stride;
        at as u32
    }

    /// 物ごとの comp → world の向き。world へ写す道と紐が同じ物を読む。
    fn write_bases(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, bases: &[([f32; 3], [f32; 3], [f32; 3])]) {
        let bytes: Vec<u8> = bases.iter().flat_map(|(u, v, c)| [u[0], u[1], u[2], 0.0, v[0], v[1], v[2], 0.0, c[0], c[1], c[2], 0.0]).flat_map(f32::to_le_bytes).collect();
        if self.bases.fit(device, bytes.len() as u64, &mut self.retired) {
            self.binds.clear();
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.bases.buffer, 0, &bytes);
        }
    }

    fn write_links(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, links: &[(u32, u32)]) {
        let bytes: Vec<u8> = links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect();
        if self.links.fit(device, bytes.len() as u64, &mut self.retired) {
            self.binds.clear();
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.links.buffer, 0, &bytes);
        }
    }

    fn bind(&self, key: (u64, usize)) -> Option<wgpu::BindGroup> {
        self.binds.get(&key).cloned()
    }

    fn remember(&mut self, key: (u64, usize), bind: wgpu::BindGroup) -> wgpu::BindGroup {
        self.binds.insert(key, bind.clone());
        bind
    }

    /// 覚えた bind group を全部捨てる(棚が読み直されて道が組み直された時)。
    pub(crate) fn forget_all(&mut self) {
        self.binds.clear();
    }

    /// この道の bind group を捨てる(世界の外の buffer — 描く側の motion — が入れ替わった時)。
    fn forget(&mut self, id: u64) {
        self.binds.retain(|(owner, _), _| *owner != id);
    }
}

/// 付いて行く(付いて置く札の相手がブロックで動いた分だけ、札も動く): 対 (札, 相手) ごとに、札の state に相手の state を足す。
/// 1 回で 1 段(札の札は次の回)。
pub(crate) struct FollowPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    pairs: Pool,
    id: u64,
}

const FOLLOW_PASS: &str = "struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };\n\
@group(0) @binding(0) var<storage, read> state_in: array<Offset>;\n\
@group(0) @binding(1) var<storage, read_write> state_out: array<Offset>;\n\
@group(0) @binding(2) var<storage, read> pairs: array<vec2u>;\n\
@group(0) @binding(3) var<uniform> count: vec4u;\n\
@compute @workgroup_size(64)\n\
fn main(@builtin(global_invocation_id) gid: vec3u) {\n\
    let i = gid.x + count.y;\n\
    if i >= count.x { return; }\n\
    let pair = pairs[i];\n\
    let own = state_in[pair.x];\n\
    state_out[pair.x] = Offset(own.translate + state_in[pair.y].translate, own.rotate, own.scale, own.tint);\n\
}\n";

impl FollowPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let (pipeline, layout) = pipeline(device, "motolii-block-follow", FOLLOW_PASS, &[storage(0, true), storage(1, false), storage(2, true), uniform(3, COUNT_BYTES)], "main");
        Self { pipeline, layout, pairs: Pool::new(device, "motolii-block-follow-pairs", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, 8), id: next_id() }
    }

    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, pairs: &[(u32, u32)]) {
        if pairs.is_empty() || world.count == 0 {
            return;
        }
        if self.pairs.fit(device, (pairs.len() * 8) as u64, &mut world.retired) {
            world.binds.clear();
        }
        queue.write_buffer(&self.pairs.buffer, 0, &pairs.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let chunks: Vec<(u32, u32)> = spans(pairs.len() as u32, world.max_groups).collect();
        world.reserve(device, chunks.len() as u64);
        let (from, to) = (world.current, 1 - world.current);
        encoder.copy_buffer_to_buffer(&world.states[from].buffer, 0, &world.states[to].buffer, 0, u64::from(world.count) * OFFSET_BYTES);
        for (at, take) in chunks {
            // 番号は全体で数える(`count.x` は対の総数、`count.y` はこの回の頭)。
            let at_count = world.slot(queue, &[(pairs.len() as u32).to_le_bytes(), at.to_le_bytes(), [0; 4], [0; 4]].concat());
            let bind = match world.bind((self.id, from)) {
                Some(bind) => bind,
                None => {
                    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("motolii-block-follow"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: world.states[from].buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 1, resource: world.states[to].buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 2, resource: self.pairs.buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 3, resource: slot_of(&world.uniforms.buffer, COUNT_BYTES) },
                        ],
                    });
                    world.remember((self.id, from), bind)
                }
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-follow"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[at_count]);
            pass.dispatch_workgroups(take.div_ceil(GROUP), 1, 1);
        }
        world.current = to;
    }
}

/// state(comp のずれ)を、物ごとの comp → world の向きで world のずれにして描く側の motion へ書く(物ごと vec4 × 4:
/// 位置と回り、真ん中と大きさ、軸、色と不透明)。
pub(crate) struct WorldPass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    /// 前のコマ描いた先(上流の pool が別の buffer を寄越したら bind group は組み直す)。
    motion: Option<wgpu::Buffer>,
    id: u64,
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
    let k = gid.x + count.z;
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
        let (pipeline, layout) = pipeline(device, "motolii-block-world-pass", WORLD_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3, COUNT_BYTES), storage(4, true)], "main");
        Self { pipeline, layout, motion: None, id: next_id() }
    }

    /// `bases` は物ごとの (comp の x の 1px の world, y の 1px の world, 物の真ん中の world)。真ん中は回る軸が通る所。
    /// `links` はつなぐ線ごとの (元の物, 先の物): 物の後ろに 2 項ずつ(kind 1 は前の 1 項、紐は 2 項)両端のずれを書く。
    /// どちらも世界に置く(紐の解きが同じ物を読む)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, bases: &[([f32; 3], [f32; 3], [f32; 3])], links: &[(u32, u32)], motion: &wgpu::Buffer) {
        if world.count == 0 {
            return;
        }
        world.write_bases(device, queue, bases);
        world.write_links(device, queue, links);
        if self.motion.as_ref() != Some(motion) {
            self.motion = Some(motion.clone());
            world.forget(self.id);
        }
        let chunks: Vec<(u32, u32)> = spans(world.count + links.len() as u32, world.max_groups).collect();
        world.reserve(device, chunks.len() as u64);
        let side = world.current;
        for (at, take) in chunks {
            let at_count = world.slot(queue, &[world.count.to_le_bytes(), (links.len() as u32).to_le_bytes(), at.to_le_bytes(), [0; 4]].concat());
            let bind = match world.bind((self.id, side)) {
                Some(bind) => bind,
                None => {
                    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("motolii-block-world-pass"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: world.bases.buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 1, resource: world.states[side].buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 3, resource: slot_of(&world.uniforms.buffer, COUNT_BYTES) },
                            wgpu::BindGroupEntry { binding: 4, resource: world.links.buffer.as_entire_binding() },
                        ],
                    });
                    world.remember((self.id, side), bind)
                }
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-block-world-pass"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[at_count]);
            pass.dispatch_workgroups(take.div_ceil(GROUP), 1, 1);
        }
    }
}

/// 紐(Line Path = Rope): つなぐ線の 3 次ベジェの腹の 2 点だけがばね + 減衰で遅れて付いて来る(利用者 2026-09-18
/// 「紐が物理の挙動になれば」「ベジェでいけないの?」)。状態は紐 1 本に 2 点の位置と速度、GPU に置いたまま
/// (読み戻さない)。描く側の motion の kind 2 の項(つなぐ線の 2 つ目の項)に、描いた時の腹と今の腹を書く。
pub(crate) struct RopePass {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    states: [Pool; 2],
    ropes: Pool,
    motion: Option<wgpu::Buffer>,
    id: u64,
    current: usize,
    /// 最後に解いたコマ(続きのコマでなければ静止から始める)。
    pub(crate) last_frame: Option<i64>,
}

const ROPE_PASS: &str = r#"struct Offset { translate: vec2f, rotate: f32, scale: f32, tint: vec4f };
struct Basis { u: vec4f, v: vec4f, centre: vec4f };
struct Host { objects: u32, ropes: u32, dt: f32, reset: u32, base: u32 };
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
    let r = gid.x + host.base;
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
        let (pipeline, layout) = pipeline(device, "motolii-rope-pass", ROPE_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3, ROPE_HOST_BYTES), storage(4, true), storage(5, true), storage(6, true), storage(7, false)], "main");
        let store = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        Self {
            pipeline,
            layout,
            states: [Pool::new(device, "motolii-rope-state", store, 64), Pool::new(device, "motolii-rope-state", store, 64)],
            ropes: Pool::new(device, "motolii-ropes", store, 16),
            motion: None,
            id: next_id(),
            current: 0,
            last_frame: None,
        }
    }

    /// `ropes` は (つなぐ線の番号, Slack %, 硬さ, 減衰)。`frame` が前の続きでなければ静止から始める。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, ropes: &[(u32, f32, f32, f32)], motion: &wgpu::Buffer, frame: i64, fps: f32) {
        if ropes.is_empty() || world.count == 0 {
            self.last_frame = None;
            return;
        }
        let count = ropes.len() as u64;
        let mut reset = self.last_frame != Some(frame - 1);
        let mut fresh = self.states[0].fit(device, count * 64, &mut world.retired);
        fresh |= self.states[1].fit(device, count * 64, &mut world.retired);
        if fresh {
            // 前の紐の記憶は新しい buffer に無い。静止から始める。
            reset = true;
        }
        fresh |= self.ropes.fit(device, count * 16, &mut world.retired);
        if fresh {
            world.binds.clear();
        }
        self.last_frame = Some(frame);
        queue.write_buffer(&self.ropes.buffer, 0, &ropes.iter().flat_map(|(c, slack, k, d)| [(*c as f32).to_le_bytes(), slack.to_le_bytes(), k.to_le_bytes(), d.to_le_bytes()].concat()).collect::<Vec<u8>>());
        if self.motion.as_ref() != Some(motion) {
            self.motion = Some(motion.clone());
            world.forget(self.id);
        }
        let chunks: Vec<(u32, u32)> = spans(count as u32, world.max_groups).collect();
        world.reserve(device, chunks.len() as u64);
        let (from, to) = (self.current, 1 - self.current);
        let side = world.current * 2 + from;
        for (at, take) in chunks.iter().copied() {
            let host = [world.count.to_le_bytes(), (count as u32).to_le_bytes(), (1.0 / fps.max(1.0)).to_le_bytes(), u32::from(reset).to_le_bytes(), at.to_le_bytes()].concat();
            let at_host = world.slot(queue, &host);
            let bind = match world.bind((self.id, side)) {
                Some(bind) => bind,
                None => {
                    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("motolii-rope-pass"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry { binding: 0, resource: world.bases.buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 1, resource: world.states[world.current].buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 2, resource: motion.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 3, resource: slot_of(&world.uniforms.buffer, ROPE_HOST_BYTES) },
                            wgpu::BindGroupEntry { binding: 4, resource: world.links.buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 5, resource: self.ropes.buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 6, resource: self.states[from].buffer.as_entire_binding() },
                            wgpu::BindGroupEntry { binding: 7, resource: self.states[to].buffer.as_entire_binding() },
                        ],
                    });
                    world.remember((self.id, side), bind)
                }
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("motolii-rope-pass"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[at_host]);
            pass.dispatch_workgroups(take.div_ceil(GROUP), 1, 1);
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
    let _ = crate::compositor::device::wait_for_gpu(device, "block-state-readback");
    let out = offsets_from_bytes(&staging.slice(..).get_mapped_range());
    staging.unmap();
    out
}
