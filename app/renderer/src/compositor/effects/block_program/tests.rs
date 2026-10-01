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
    let program = program_for(device, "bounce");
    let m = 1.0; // doc の CANVAS_MARGIN(箱の素材座標の原点)
    let mut items = Vec::new();
    let mut rng = 7u32;
    let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
    for index in 0..4000u32 {
        let (size, radius) = if index % 2 == 0 { ([900.0, 520.0], 40.0) } else { ([420.0, 420.0], 210.0) };
        let w = 10.0 + next() * 60.0;
        let x = m - 800.0 + next() * 2400.0;
        let y = m - 800.0 + next() * 2400.0;
        items.push(BlockItem { lo: [x, y], hi: [x + w, y + w], room_lo: [m, m], room_size: size, radius, group: 0, margin: 0.0, weight: 1.0, ..Default::default() });
    }
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
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

/// 口: ブロックは位置のずれだけでなく、物の component(回転・大きさ・色と不透明)へ書ける。
/// 位置と回転は足す、大きさと色は掛ける — 同じブロックを 2 回掛けると 2 倍でなく 2 乗になる(棚の札は要らない、test の中の式)。
#[test]
fn a_block_writes_scale_and_tint_and_they_compose_by_multiplying() {
    let engine = gpu();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let source = "/*{ \"ID\": \"test.dim\", \"STAGE\": \"block\", \"INPUTS\": [ {\"NAME\":\"opacity\",\"TYPE\":\"float\",\"DEFAULT\":0.5} ] }*/\n\
        fn block(k: u32, p: BlockParams) -> Offset { return Offset(vec2f(1.0, 0.0), 10.0, 0.5, vec4f(1.0, 0.5, 0.25, p.opacity)); }";
    let (manifest, body) = super::super::isf::parse_isf_source(source).unwrap();
    let full = module_source(&manifest, &body, &[]).unwrap();
    validate(&full).unwrap();
    let program = BlockProgram::new(device, "block-test-dim", &full, 1);
    let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5]);
    program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5]);
    let state = read_state(device, queue, &world, encoder);
    assert!(pollster::block_on(scope.pop()).is_none());
    let o = state[0];
    assert_eq!(o.translate, [2.0, 0.0], "position adds");
    assert_eq!(o.rotate, 20.0, "rotation adds");
    assert_eq!(o.scale, 0.25, "scale multiplies");
    assert_eq!(o.tint, [1.0, 0.25, 0.0625, 0.25], "tint and opacity multiply");
}

/// 名指しの口(利用者 2026-09-19、CSS の `anchor()` / AE の parent): `parent(k)` / `anchor(k)` は相手の今の Offset
/// (同じコマで先に走った block の結果込み)、相手が居なければ `NO_OFFSET`。同じ stage でも読まれる側が先に走る。
#[test]
fn a_block_reads_its_parents_and_anchors_current_offset_and_none_without_one() {
    let engine = gpu();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let build = |name: &str, source: &str| {
        let (manifest, body) = wgsl_manifest(name, source).unwrap();
        let full = module_source(&manifest, &body, &[]).unwrap();
        validate(&full).unwrap();
        BlockProgram::new(device, name, &full, 1)
    };
    let mover = build("mover", "fn block(k: u32) -> Offset { return Offset(vec2f(5.0, -3.0), 12.0, 2.0, vec4f(0.5, 1.0, 1.0, 1.0)); }");
    let child = build("child", "fn block(k: u32) -> Offset { let p = parent(k); return Offset(p.translate, p.rotate, 1.0, vec4f(1.0)); }");
    let anchored = build("anchored", "fn block(k: u32) -> Offset { let a = anchor(k); return Offset(a.translate, 0.0, a.scale, a.tint); }");
    let items = [
        BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], ..Default::default() },
        BlockItem { lo: [20.0, 0.0], hi: [30.0, 10.0], parent_slot: 0, ..Default::default() },
        BlockItem { lo: [40.0, 0.0], hi: [50.0, 10.0], anchor_slot: 0, ..Default::default() },
        BlockItem { lo: [60.0, 0.0], hi: [70.0, 10.0], ..Default::default() },
    ];
    assert_eq!(item_bytes(&items).len(), items.len() * ITEM_BYTES as usize, "the host packs what the WGSL Item declares");
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    mover.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[]);
    child.record(device, queue, &mut encoder, &mut world, 0.0, &[1, 3], &[]);
    anchored.record(device, queue, &mut encoder, &mut world, 0.0, &[2], &[]);
    let state = read_state(device, queue, &world, encoder);
    assert!(pollster::block_on(scope.pop()).is_none());
    assert_eq!((state[1].translate, state[1].rotate), ([5.0, -3.0], 12.0), "the child moves and turns with its parent");
    assert_eq!((state[2].translate, state[2].scale, state[2].tint), ([5.0, -3.0], 2.0, [0.5, 1.0, 1.0, 1.0]), "the anchored thing reads the hub's whole offset");
    assert_eq!(state[3], BlockOffset::default(), "a thing with neither parent nor anchor reads NO_OFFSET");
}

/// override の札: 欄は `override`(`@label` `@range` `@options`)、本文は名前のまま読む。manifest は JSON の頭と同じ形になり、
/// 組んだ計算シェーダーは naga を通り、JSON の札(上の test)と同じ結果を出す。
#[test]
fn an_override_block_declares_its_inputs_in_wgsl() {
    use super::super::isf::{IsfInputType, IsfScope};
    let source = "@id(\"test.dim\") @description(\"dims\")\n\
        @rounds(2) @scope(\"room\") @physics(\"ROOM\", \"ancestor\") @physics(\"SUBSTEPS\", 4)\n\n\
        @label(\"Opacity\") @range(0.0, 1.0)\n\
        override opacity: f32 = 0.5;\n\
        @label(\"Shape\") @options(\"Sphere\", \"Box\", \"In Out\")\n\
        override shape: u32 = 1;\n\
        @reach\n\
        override reach: f32 = -80.0;\n\
        override lit: bool = true;\n\n\
        // a comment between\n\
        fn block(k: u32) -> Offset {\n\
            var a = opacity;\n\
            if shape != 1u || !lit { a = 0.0; }\n\
            return Offset(vec2f(1.0, 0.0), 10.0, 0.5, vec4f(1.0, 0.5, 0.25, a));\n}";
    let (manifest, body) = wgsl_manifest("dim_test", source).unwrap();
    assert_eq!(manifest.id.as_deref(), Some("test.dim"));
    assert_eq!(manifest.label.as_deref(), Some("Dim Test"), "label falls back to the file stem in Title Case");
    assert_eq!(manifest.description.as_deref(), Some("dims"));
    assert_eq!((manifest.rounds, manifest.scope, manifest.reach.as_deref()), (2, IsfScope::Room, Some("reach")));
    assert_eq!(manifest.physics.get("ROOM").and_then(|v| v.as_str()), Some("ancestor"));
    assert_eq!(manifest.physics.get("SUBSTEPS").and_then(|v| v.as_f64()), Some(4.0));
    let inputs: Vec<_> = manifest.inputs.iter().map(|i| (i.name.as_str(), i.label.as_deref(), i.ty, i.default[0], i.min.map(|m| m[0]), i.max.map(|m| m[0]), i.labels.clone())).collect();
    assert_eq!(inputs, [
        ("opacity", Some("Opacity"), IsfInputType::Float, 0.5, Some(0.0), Some(1.0), None),
        ("shape", Some("Shape"), IsfInputType::Long, 1.0, None, None, Some(vec!["Sphere".into(), "Box".into(), "In Out".into()])),
        ("reach", None, IsfInputType::Float, -80.0, None, None, None),
        ("lit", None, IsfInputType::Bool, 1.0, None, None, None),
    ]);
    assert!(body.contains("var<private> shape: u32 = 1;") && body.contains("shape = u32(block_params[0][1]);") && body.contains("lit = (block_params[0][3] != 0.0);"), "{body}");
    assert!(!body.contains('@'), "no Motolii attribute survives into the WESL text: {body}");
    let full = module_source(&manifest, &body, &[]).unwrap();
    validate(&full).unwrap();
    let engine = gpu();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let program = BlockProgram::new(device, "block-test-override", &full, 1);
    let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5, 1.0, 0.0, 1.0]);
    program.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.5, 1.0, 0.0, 1.0]);
    let state = read_state(device, queue, &world, encoder);
    assert!(pollster::block_on(scope.pop()).is_none());
    let o = state[0];
    assert_eq!((o.translate, o.rotate, o.scale, o.tint), ([2.0, 0.0], 20.0, 0.25, [1.0, 0.25, 0.0625, 0.25]), "same as the JSON-head block");
    assert!(wgsl_manifest("x", "override a: f32 = 1.0;\nfn main() {}").is_err(), "no fn block, no 札");
    assert!(wgsl_manifest("x", "@range(0.0, 1.0)\noverride a: f32 = 1.0;\n@rounds(3)\nfn block(k: u32) -> Offset { return NO_OFFSET; }").is_ok(), "attributes on fn block are the file's");
}

/// 描く側へ渡す motion は物ごと vec4 × 4: (位置, 回り)・(真ん中, 大きさ)・(軸, _)・(色, 不透明)。大きさと色が落ちずに届く。
#[test]
fn the_world_pass_hands_scale_and_tint_to_the_drawing_side() {
    let engine = gpu();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let items = [BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }];
    let mut world = BlockWorld::new(device);
    world.begin_from(device, queue, &items, 0.0, &[BlockOffset { translate: [3.0, 0.0], rotate: 0.0, scale: 0.5, tint: [1.0, 0.5, 0.25, 0.5] }]);
    let motion = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-motion"), size: 64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    WorldPass::new(device).record(device, queue, &mut encoder, &mut world, &[([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [5.0, 5.0, 0.0])], &[], &motion);
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-read"), size: 64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    encoder.copy_buffer_to_buffer(&motion, 0, &staging, 0, 64);
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    crate::compositor::device::wait_for_gpu(device, "block-program-test").unwrap();
    let words: Vec<f32> = staging.slice(..).get_mapped_range().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    staging.unmap();
    assert_eq!(&words[0..3], &[3.0, 0.0, 0.0], "offset in world");
    assert_eq!(&words[4..8], &[5.0, 5.0, 0.0, 0.5], "centre, then scale");
    assert_eq!(&words[12..16], &[1.0, 0.5, 0.25, 0.5], "tint, then opacity");
}

/// 紐: つなぐ線の 2 つ目の項に、描いた時の腹(端の間の 1/3・2/3 を Slack だけ下げた点)と今の腹が書かれ、種は 2。
/// 端が動いていなければ今の腹は描いた腹と同じ(静止から始める)。
#[test]
fn the_rope_pass_writes_the_bellies_of_a_cubic_behind_the_things() {
    let engine = gpu();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let items = [
        BlockItem { lo: [0.0, 0.0], hi: [10.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() },
        BlockItem { lo: [30.0, 0.0], hi: [40.0, 10.0], room_lo: [0.0, 0.0], room_size: [100.0, 100.0], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() },
    ];
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let bases = [([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]), ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [30.0, 0.0, 0.0])];
    let motion = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-motion"), size: 4 * 64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    WorldPass::new(device).record(device, queue, &mut encoder, &mut world, &bases, &[(0, 1)], &motion);
    RopePass::new(device).record(device, queue, &mut encoder, &mut world, &[(0, 20.0, 60.0, 6.0)], &motion, 0, 30.0);
    let staging = device.create_buffer(&wgpu::BufferDescriptor { label: Some("test-read"), size: 4 * 64, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    encoder.copy_buffer_to_buffer(&motion, 0, &staging, 0, 4 * 64);
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    crate::compositor::device::wait_for_gpu(device, "block-program-test").unwrap();
    let w: Vec<f32> = staging.slice(..).get_mapped_range().chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    staging.unmap();
    let v = |i: usize| [w[i * 4], w[i * 4 + 1], w[i * 4 + 2], w[i * 4 + 3]];
    assert_eq!(v(9), [0.0, 0.0, 0.0, 1.0], "A, then the connector's own flag");
    assert_eq!(v(10), [30.0, 0.0, 0.0, 2.0], "B − A, kind 2 = rope");
    assert_eq!(v(12)[..3], [10.0, 6.0, 0.0], "drawn c0: a third along, sagging by 20 % of the span");
    assert_eq!(v(13)[..3], [20.0, 6.0, 0.0], "drawn c1");
    assert_eq!(v(14)[..3], [10.0, 6.0, 0.0], "live c0 starts at rest");
    assert_eq!(v(15)[..3], [20.0, 6.0, 0.0], "live c1 starts at rest");
}

/// 棚の block の一覧(切り分け用): 今日置いた札が段 Block で載っているか。
#[test]
fn todays_packages_are_blocks_on_the_shelf() {
    let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
    let blocks: Vec<(String, String)> = catalog.definitions.iter().filter(|d| d.manifest.stage == super::super::isf::IsfStage::Block).map(|d| (d.source.name.clone(), d.plugin_id().to_owned())).collect();
    eprintln!("blocks: {blocks:?}\nerrors: {:?}", catalog.errors);
    for name in ["concentrick", "falloff_reveal", "formations", "zero_gravity", "wave"] {
        assert!(blocks.iter().any(|(n, _)| n == name), "{name} is not a block on the shelf: {blocks:?} / {:?}", catalog.errors);
    }
}

/// ブロックは fx と同じ棚に、段 Block として並ぶ(置き場・頭の JSON・読み直しが同じ)。
#[test]
fn a_block_sits_on_the_shelf_like_an_effect() {
    let refresh = crate::render::compositor::effects::catalog::refresh_effect_catalog();
    assert!(!refresh.errors.iter().any(|e| ["bounce", "push_apart", "four_color_gradient", "inner_shadow"].iter().any(|n| e.contains(n))), "{:?}", refresh.errors);
    let catalog = crate::render::compositor::effects::catalog::catalog_snapshot();
    let bounce = catalog.descriptors.iter().find(|d| d.plugin_id == "motolii.bounce_block").expect("on the shelf");
    assert_eq!(bounce.stage, crate::render::compositor::effects::catalog::EffectStage::Block);
    assert_eq!(bounce.params.iter().map(|p| p.label.as_str()).collect::<Vec<_>>(), ["Strength"]);
    for fx in ["motolii.four_color_gradient", "motolii.inner_shadow"] {
        assert!(catalog.descriptors.iter().any(|d| d.plugin_id == fx), "{fx} on the shelf");
    }
}
