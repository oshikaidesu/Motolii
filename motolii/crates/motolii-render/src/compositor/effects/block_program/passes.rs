//! ブロックの GPU の道: 計算 pipeline、1 コマの物の世界(buffer)、付いて行く・world へ写す・紐の 3 本の pass。
//! 札の文字は読まない — 受け取るのは組み上がった全文。
//!
//! pipeline・buffer・bind group は re_renderer の pool から取る(shader は棚と同じ shader の file system に置く)。
//! buffer は**伸びた時だけ**取り直す(古い方は、記録済みの pass の bind group が持つ限り生き、pool は次のフレーム以降に使い回す)。
//! 回ごと・ブロックごとに違う uniform は 1 本の buffer の升目に並べて dynamic offset で指す(submit では同じ場所の最後の書き込みが勝つ)。
//! dispatch は device の `max_compute_workgroups_per_dimension` で切って、続きをもう 1 回投げる(黙って切らない)。

use re_renderer::{BindGroupEntry, GpuBindGroup, GpuBindGroupLayoutHandle, GpuBuffer, GpuComputePipelineHandle, RenderContext};

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

/// buffer 全体を指す binding。
fn whole(buffer: &GpuBuffer) -> BindGroupEntry {
    BindGroupEntry::Buffer { handle: buffer.handle, offset: 0, size: None }
}

/// 升 1 つ分だけを指す binding(dynamic offset を使う binding は大きさを言わないといけない)。
fn slot_of(buffer: &GpuBuffer, size: u64) -> BindGroupEntry {
    BindGroupEntry::Buffer { handle: buffer.handle, offset: 0, size: wgpu::BufferSize::new(size) }
}

fn bind(ctx: &RenderContext, label: &str, layout: GpuBindGroupLayoutHandle, entries: impl IntoIterator<Item = BindGroupEntry>) -> GpuBindGroup {
    ctx.gpu_resources.bind_groups.alloc(&ctx.device, &ctx.gpu_resources, &re_renderer::BindGroupDesc {
        label: label.into(),
        entries: entries.into_iter().collect(),
        layout,
    })
}

/// 計算 pipeline を pool から取る。全文は中身で名付けた file に置く(同じ全文は同じ pipeline、変われば別の pipeline)。
fn pipeline(ctx: &RenderContext, label: &str, source: &str, entries: &[wgpu::BindGroupLayoutEntry], entry: &str) -> (GpuComputePipelineHandle, GpuBindGroupLayoutHandle) {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    let path = super::super::vism::catalog_stage_path(&format!("{label}-{:016x}", hasher.finish()), "compute");
    super::super::vism::write_catalog_stage(&path, source).expect("the block's WGSL is written where the renderer reads shaders");
    let resources = &ctx.gpu_resources;
    let module = resources.shader_modules.get_or_create(ctx, &re_renderer::ShaderModuleDesc { label: label.into(), source: path, extra_workaround_replacements: Vec::new() });
    let layout = resources.bind_group_layouts.get_or_create(&ctx.device, &re_renderer::BindGroupLayoutDesc { label: label.into(), entries: entries.to_vec() });
    let pipeline_layout = resources.pipeline_layouts.get_or_create(ctx, &re_renderer::PipelineLayoutDesc { label: label.into(), entries: vec![layout] });
    let pipeline = resources.compute_pipelines.get_or_create(ctx, &re_renderer::ComputePipelineDesc { label: label.into(), pipeline_layout, entrypoint: entry.to_owned(), shader_handle: module })
        .expect("the layout and module were just taken from the pools");
    (pipeline, layout)
}

/// `pipeline` で 1 回 dispatch する。
fn dispatch(ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, label: &str, pipeline: GpuComputePipelineHandle, bind: &GpuBindGroup, offsets: &[u32], take: u32) {
    let pipelines = ctx.gpu_resources.compute_pipelines.resources();
    let Ok(pipeline) = pipelines.get(pipeline) else { return };
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some(label), timestamp_writes: None });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind, offsets);
    pass.dispatch_workgroups(take.div_ceil(GROUP), 1, 1);
}

/// dispatch を 1 軸の天井で切る: (頭の番号, その回で見る数)。天井を超えても黙って切らず、続きを投げる。
fn spans(total: u32, max_groups: u32) -> impl Iterator<Item = (u32, u32)> {
    let per = max_groups.saturating_mul(GROUP).max(GROUP);
    (0..total.div_ceil(per)).map(move |i| (i * per, per.min(total - i * per)))
}

/// 伸びた時だけ pool から取り直す buffer。毎コマするのは `write_buffer` だけ。
struct Growing {
    buffer: GpuBuffer,
    label: &'static str,
    usage: wgpu::BufferUsages,
}

impl Growing {
    fn new(ctx: &RenderContext, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> Self {
        Self { buffer: Self::alloc(ctx, label, usage, size), label, usage }
    }

    fn alloc(ctx: &RenderContext, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> GpuBuffer {
        ctx.gpu_resources.buffers.alloc(&ctx.device, &re_renderer::BufferDesc { label: label.into(), size: size.max(4).next_power_of_two(), usage, mapped_at_creation: false })
    }

    /// 要る大きさに足りなければ取り直す。取り直したら true(前の中身は無い)。
    fn fit(&mut self, ctx: &RenderContext, bytes: u64) -> bool {
        if self.buffer.size() >= bytes {
            return false;
        }
        self.buffer = Self::alloc(ctx, self.label, self.usage, bytes);
        true
    }
}

/// 1 本のブロックの GPU の道。
pub(crate) struct BlockProgram {
    pipeline: GpuComputePipelineHandle,
    layout: GpuBindGroupLayoutHandle,
    pub(crate) rounds: u32,
}

/// 1 コマの物の世界。物の箱・state の 2 本・近くの物・掛かった物の番号・uniform の升目を持ち、
/// どれも**物が増えた時だけ**取り直す。
pub(crate) struct BlockWorld {
    objects: Growing,
    states: [Growing; 2],
    neighbor_starts: Growing,
    neighbor_list: Growing,
    /// このコマの全ての batch の「掛かった物の番号」を続けて並べる(batch ごとの buffer を作らない)。
    members: Growing,
    members_used: u64,
    /// このコマの uniform(host・欄・数)を升目に並べる。回ごと・block ごとに升を変える。
    uniforms: Growing,
    uniform_used: u64,
    /// 1 升の byte(device の `min_uniform_buffer_offset_alignment` の倍数)。
    stride: u64,
    /// 物ごとの comp → world の向きと、つなぐ線の両端(world へ写す道と紐が同じ物を読む)。
    bases: Growing,
    links: Growing,
    /// dispatch の 1 軸の天井。
    max_groups: u32,
    /// 今の state が `states` のどちらか。
    pub(crate) current: usize,
    pub(crate) count: u32,
}

impl BlockProgram {
    pub(crate) fn new(ctx: &RenderContext, label: &str, source: &str, rounds: u32) -> Self {
        let (pipeline, layout) = pipeline(ctx, label, source, &[storage(0, true), storage(1, true), uniform(2, HOST_BYTES), uniform(3, PARAMS_BYTES), storage(4, false), storage(5, true), storage(6, true), storage(7, true)], "motolii_block_main");
        Self { pipeline, layout, rounds: rounds.max(1) }
    }

    /// 掛かった物(`members`)に `rounds` 回掛ける。毎回、前の回の state を読み、書く先は反対側
    /// (掛からない物はこのブロックが 1 度も書かないので、写すのは呼び出しごとに 1 回で足りる)。
    pub(crate) fn record(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32]) {
        self.record_from(ctx, encoder, world, time, members, params, u32::MAX);
    }

    /// 場(`SCOPE: room`)は元の物の番号を `host.source` で渡す。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_from(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, time: f32, members: &[u32], params: &[f32], source: u32) {
        if members.is_empty() || world.count == 0 {
            return;
        }
        // 掛かった物の番号は、このコマの並びの後ろへ足すだけ(頭の番号を host で渡す)。
        let base = world.members_span(ctx, members);
        let chunks: Vec<(u32, u32)> = spans(members.len() as u32, world.max_groups).collect();
        world.reserve(ctx, 1 + u64::from(self.rounds) * chunks.len() as u64);
        let mut slots = [0.0f32; PARAM_SLOTS];
        for (slot, value) in slots.iter_mut().zip(params) {
            *slot = *value;
        }
        let at_params = world.slot(ctx, &slots.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        let bytes = u64::from(world.count) * OFFSET_BYTES;
        encoder.copy_buffer_to_buffer(&world.states[world.current].buffer, 0, &world.states[1 - world.current].buffer, 0, bytes);
        // ping-pong の側ごとに 1 つ(この呼び出しの中で指し先は変わらない)。
        let mut binds: [Option<GpuBindGroup>; 2] = [None, None];
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
                let at_host = world.slot(ctx, &h);
                let bind = binds[from].get_or_insert_with(|| bind(ctx, "motolii-block", self.layout, [
                    whole(&world.objects.buffer),
                    whole(&world.states[from].buffer),
                    slot_of(&world.uniforms.buffer, HOST_BYTES),
                    slot_of(&world.uniforms.buffer, PARAMS_BYTES),
                    whole(&world.states[to].buffer),
                    whole(&world.members.buffer),
                    whole(&world.neighbor_starts.buffer),
                    whole(&world.neighbor_list.buffer),
                ]));
                dispatch(ctx, encoder, "motolii-block", self.pipeline, bind, &[at_host, at_params], take);
            }
            world.current = to;
        }
    }
}

impl BlockWorld {
    pub(crate) fn new(ctx: &RenderContext) -> Self {
        let store = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let state = store | wgpu::BufferUsages::COPY_SRC;
        let limits = ctx.device.limits();
        let align = u64::from(limits.min_uniform_buffer_offset_alignment);
        Self {
            objects: Growing::new(ctx, "motolii-block-objects", store, ITEM_BYTES),
            states: [Growing::new(ctx, "motolii-block-state", state, OFFSET_BYTES), Growing::new(ctx, "motolii-block-state", state, OFFSET_BYTES)],
            neighbor_starts: Growing::new(ctx, "motolii-block-neighbor-starts", store, 8),
            neighbor_list: Growing::new(ctx, "motolii-block-neighbor-list", store, 4),
            members: Growing::new(ctx, "motolii-block-members", store, 4),
            members_used: 0,
            uniforms: Growing::new(ctx, "motolii-block-uniforms", wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, align.max(PARAMS_BYTES)),
            uniform_used: 0,
            stride: PARAMS_BYTES.div_ceil(align) * align,
            bases: Growing::new(ctx, "motolii-block-bases", store, 48),
            links: Growing::new(ctx, "motolii-block-links", store, 8),
            max_groups: limits.max_compute_workgroups_per_dimension,
            current: 0,
            count: 0,
        }
    }

    /// このコマの物の箱を置き、state を 0(ずれ無し)から始める。
    pub(crate) fn begin(&mut self, ctx: &RenderContext, items: &[BlockItem], reach: f32) {
        self.begin_from(ctx, items, reach, &[]);
    }

    /// 始まりの state を渡して始める(外の解き手 — Rapier — が出したずれを土台にする)。
    pub(crate) fn begin_from(&mut self, ctx: &RenderContext, items: &[BlockItem], reach: f32, seed: &[BlockOffset]) {
        self.members_used = 0;
        self.uniform_used = 0;
        let count = items.len().max(1) as u64;
        self.objects.fit(ctx, count * ITEM_BYTES);
        self.states[0].fit(ctx, count * OFFSET_BYTES);
        self.states[1].fit(ctx, count * OFFSET_BYTES);
        self.count = items.len() as u32;
        self.current = 0;
        if !items.is_empty() {
            ctx.queue.write_buffer(&self.objects.buffer, 0, &item_bytes(items));
            let mut start = vec![BlockOffset::default(); items.len()];
            for (slot, value) in start.iter_mut().zip(seed) {
                *slot = *value;
            }
            ctx.queue.write_buffer(&self.states[0].buffer, 0, &offset_bytes(&start));
        }
        let (starts, list) = neighbors(items, reach);
        let starts: Vec<u8> = starts.iter().flat_map(|v| v.to_le_bytes()).collect();
        let list: Vec<u8> = list.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.neighbor_starts.fit(ctx, starts.len() as u64);
        self.neighbor_list.fit(ctx, list.len() as u64);
        if !starts.is_empty() {
            ctx.queue.write_buffer(&self.neighbor_starts.buffer, 0, &starts);
        }
        if !list.is_empty() {
            ctx.queue.write_buffer(&self.neighbor_list.buffer, 0, &list);
        }
    }

    pub(crate) fn state(&self) -> &wgpu::Buffer {
        &self.states[self.current].buffer
    }

    /// 掛かった物の番号をこのコマの並びの後ろへ置く。返すのはその頭(番号の位置)。
    /// 取り直すと前の番号は新しい buffer に無い: このコマで先に並べた分は、記録済みの pass が前の buffer から読む。
    fn members_span(&mut self, ctx: &RenderContext, members: &[u32]) -> u32 {
        let start = self.members_used;
        self.members.fit(ctx, (start + members.len() as u64) * 4);
        ctx.queue.write_buffer(&self.members.buffer, start * 4, &members.iter().flat_map(|m| m.to_le_bytes()).collect::<Vec<u8>>());
        self.members_used = start + members.len() as u64;
        start as u32
    }

    /// uniform の升を `slots` 個ぶん先に確かめる(取り始めた後に取り直すと、先に書いた升が迷子になる)。
    fn reserve(&mut self, ctx: &RenderContext, slots: u64) {
        self.uniforms.fit(ctx, self.uniform_used + slots * self.stride);
    }

    /// 升を 1 つ取って書く。返すのは `set_bind_group` に渡す dynamic offset。
    fn slot(&mut self, ctx: &RenderContext, bytes: &[u8]) -> u32 {
        let at = self.uniform_used;
        debug_assert!(at + self.stride <= self.uniforms.buffer.size(), "升を取る前に reserve する");
        ctx.queue.write_buffer(&self.uniforms.buffer, at, bytes);
        self.uniform_used = at + self.stride;
        at as u32
    }

    /// 物ごとの comp → world の向き。world へ写す道と紐が同じ物を読む。
    fn write_bases(&mut self, ctx: &RenderContext, bases: &[([f32; 3], [f32; 3], [f32; 3])]) {
        let bytes: Vec<u8> = bases.iter().flat_map(|(u, v, c)| [u[0], u[1], u[2], 0.0, v[0], v[1], v[2], 0.0, c[0], c[1], c[2], 0.0]).flat_map(f32::to_le_bytes).collect();
        self.bases.fit(ctx, bytes.len() as u64);
        if !bytes.is_empty() {
            ctx.queue.write_buffer(&self.bases.buffer, 0, &bytes);
        }
    }

    fn write_links(&mut self, ctx: &RenderContext, links: &[(u32, u32)]) {
        let bytes: Vec<u8> = links.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect();
        self.links.fit(ctx, bytes.len() as u64);
        if !bytes.is_empty() {
            ctx.queue.write_buffer(&self.links.buffer, 0, &bytes);
        }
    }
}

/// 付いて行く(付いて置く札の相手がブロックで動いた分だけ、札も動く): 対 (札, 相手) ごとに、札の state に相手の state を足す。
/// 1 回で 1 段(札の札は次の回)。
pub(crate) struct FollowPass {
    pipeline: GpuComputePipelineHandle,
    layout: GpuBindGroupLayoutHandle,
    pairs: Growing,
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
    pub(crate) fn new(ctx: &RenderContext) -> Self {
        let (pipeline, layout) = pipeline(ctx, "motolii-block-follow", FOLLOW_PASS, &[storage(0, true), storage(1, false), storage(2, true), uniform(3, COUNT_BYTES)], "main");
        Self { pipeline, layout, pairs: Growing::new(ctx, "motolii-block-follow-pairs", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, 8) }
    }

    pub(crate) fn record(&mut self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, pairs: &[(u32, u32)]) {
        if pairs.is_empty() || world.count == 0 {
            return;
        }
        self.pairs.fit(ctx, (pairs.len() * 8) as u64);
        ctx.queue.write_buffer(&self.pairs.buffer, 0, &pairs.iter().flat_map(|(a, b)| [a.to_le_bytes(), b.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let chunks: Vec<(u32, u32)> = spans(pairs.len() as u32, world.max_groups).collect();
        world.reserve(ctx, chunks.len() as u64);
        let (from, to) = (world.current, 1 - world.current);
        encoder.copy_buffer_to_buffer(&world.states[from].buffer, 0, &world.states[to].buffer, 0, u64::from(world.count) * OFFSET_BYTES);
        let mut binding = None;
        for (at, take) in chunks {
            // 番号は全体で数える(`count.x` は対の総数、`count.y` はこの回の頭)。
            let at_count = world.slot(ctx, &[(pairs.len() as u32).to_le_bytes(), at.to_le_bytes(), [0; 4], [0; 4]].concat());
            let bind = binding.get_or_insert_with(|| bind(ctx, "motolii-block-follow", self.layout, [
                whole(&world.states[from].buffer),
                whole(&world.states[to].buffer),
                whole(&self.pairs.buffer),
                slot_of(&world.uniforms.buffer, COUNT_BYTES),
            ]));
            dispatch(ctx, encoder, "motolii-block-follow", self.pipeline, bind, &[at_count], take);
        }
        world.current = to;
    }
}

/// state(comp のずれ)を、物ごとの comp → world の向きで world のずれにして描く側の motion へ書く(物ごと vec4 × 4:
/// 位置と回り、真ん中と大きさ、軸、色と不透明)。
pub(crate) struct WorldPass {
    pipeline: GpuComputePipelineHandle,
    layout: GpuBindGroupLayoutHandle,
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
    pub(crate) fn new(ctx: &RenderContext) -> Self {
        let (pipeline, layout) = pipeline(ctx, "motolii-block-world-pass", WORLD_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3, COUNT_BYTES), storage(4, true)], "main");
        Self { pipeline, layout }
    }

    /// `bases` は物ごとの (comp の x の 1px の world, y の 1px の world, 物の真ん中の world)。真ん中は回る軸が通る所。
    /// `links` はつなぐ線ごとの (元の物, 先の物): 物の後ろに 2 項ずつ(kind 1 は前の 1 項、紐は 2 項)両端のずれを書く。
    /// どちらも世界に置く(紐の解きが同じ物を読む)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, bases: &[([f32; 3], [f32; 3], [f32; 3])], links: &[(u32, u32)], motion: &GpuBuffer) {
        if world.count == 0 {
            return;
        }
        world.write_bases(ctx, bases);
        world.write_links(ctx, links);
        let chunks: Vec<(u32, u32)> = spans(world.count + links.len() as u32, world.max_groups).collect();
        world.reserve(ctx, chunks.len() as u64);
        let side = world.current;
        let mut binding = None;
        for (at, take) in chunks {
            let at_count = world.slot(ctx, &[world.count.to_le_bytes(), (links.len() as u32).to_le_bytes(), at.to_le_bytes(), [0; 4]].concat());
            let bind = binding.get_or_insert_with(|| bind(ctx, "motolii-block-world-pass", self.layout, [
                whole(&world.bases.buffer),
                whole(&world.states[side].buffer),
                whole(motion),
                slot_of(&world.uniforms.buffer, COUNT_BYTES),
                whole(&world.links.buffer),
            ]));
            dispatch(ctx, encoder, "motolii-block-world-pass", self.pipeline, bind, &[at_count], take);
        }
    }
}

/// 紐(Line Path = Rope): つなぐ線の 3 次ベジェの腹の 2 点だけがばね + 減衰で遅れて付いて来る(利用者 2026-09-18
/// 「紐が物理の挙動になれば」「ベジェでいけないの?」)。状態は紐 1 本に 2 点の位置と速度、GPU に置いたまま
/// (読み戻さない)。描く側の motion の kind 2 の項(つなぐ線の 2 つ目の項)に、描いた時の腹と今の腹を書く。
pub(crate) struct RopePass {
    pipeline: GpuComputePipelineHandle,
    layout: GpuBindGroupLayoutHandle,
    states: [Growing; 2],
    ropes: Growing,
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
    pub(crate) fn new(ctx: &RenderContext) -> Self {
        let (pipeline, layout) = pipeline(ctx, "motolii-rope-pass", ROPE_PASS, &[storage(0, true), storage(1, true), storage(2, false), uniform(3, ROPE_HOST_BYTES), storage(4, true), storage(5, true), storage(6, true), storage(7, false)], "main");
        let store = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        Self {
            pipeline,
            layout,
            states: [Growing::new(ctx, "motolii-rope-state", store, 64), Growing::new(ctx, "motolii-rope-state", store, 64)],
            ropes: Growing::new(ctx, "motolii-ropes", store, 16),
            current: 0,
            last_frame: None,
        }
    }

    /// `ropes` は (つなぐ線の番号, Slack %, 硬さ, 減衰)。`frame` が前の続きでなければ静止から始める。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&mut self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, world: &mut BlockWorld, ropes: &[(u32, f32, f32, f32)], motion: &GpuBuffer, frame: i64, fps: f32) {
        if ropes.is_empty() || world.count == 0 {
            self.last_frame = None;
            return;
        }
        let count = ropes.len() as u64;
        let mut reset = self.last_frame != Some(frame - 1);
        let mut fresh = self.states[0].fit(ctx, count * 64);
        fresh |= self.states[1].fit(ctx, count * 64);
        if fresh {
            // 前の紐の記憶は新しい buffer に無い。静止から始める。
            reset = true;
        }
        self.ropes.fit(ctx, count * 16);
        self.last_frame = Some(frame);
        ctx.queue.write_buffer(&self.ropes.buffer, 0, &ropes.iter().flat_map(|(c, slack, k, d)| [(*c as f32).to_le_bytes(), slack.to_le_bytes(), k.to_le_bytes(), d.to_le_bytes()].concat()).collect::<Vec<u8>>());
        let chunks: Vec<(u32, u32)> = spans(count as u32, world.max_groups).collect();
        world.reserve(ctx, chunks.len() as u64);
        let (from, to) = (self.current, 1 - self.current);
        let mut binding = None;
        for (at, take) in chunks.iter().copied() {
            let host = [world.count.to_le_bytes(), (count as u32).to_le_bytes(), (1.0 / fps.max(1.0)).to_le_bytes(), u32::from(reset).to_le_bytes(), at.to_le_bytes()].concat();
            let at_host = world.slot(ctx, &host);
            let bind = binding.get_or_insert_with(|| bind(ctx, "motolii-rope-pass", self.layout, [
                whole(&world.bases.buffer),
                whole(&world.states[world.current].buffer),
                whole(motion),
                slot_of(&world.uniforms.buffer, ROPE_HOST_BYTES),
                whole(&world.links.buffer),
                whole(&self.ropes.buffer),
                whole(&self.states[from].buffer),
                whole(&self.states[to].buffer),
            ]));
            dispatch(ctx, encoder, "motolii-rope-pass", self.pipeline, bind, &[at_host], take);
        }
        self.current = to;
    }
}

/// 試験の口: `buffer` の頭 `size` byte を、`encoder` の記録ごと 1 フレーム流して読む(描く道は読み戻さない)。
#[cfg(test)]
pub fn read_back(compositor: &mut crate::render::compositor::Compositor, buffer: &wgpu::Buffer, size: u64, encoder: wgpu::CommandEncoder) -> Vec<u8> {
    use crate::render::compositor::readback;
    compositor.ctx.queue_commands([encoder.finish()]);
    let id = readback::ask_buffer(&compositor.ctx, buffer, size).expect("block readback");
    compositor.next_frame();
    compositor.wait_offline().unwrap();
    readback::take_buffer(&compositor.ctx, id).expect("the readback arrives once the frame is waited for")
}

/// 試験の口: state を読み戻す。
#[cfg(test)]
pub fn read_state(compositor: &mut crate::render::compositor::Compositor, world: &BlockWorld, encoder: wgpu::CommandEncoder) -> Vec<BlockOffset> {
    offsets_from_bytes(&read_back(compositor, world.state(), u64::from(world.count) * OFFSET_BYTES, encoder))
}
