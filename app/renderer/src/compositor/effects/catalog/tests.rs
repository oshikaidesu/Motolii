use super::*;

fn blur(text: String) -> Result<VismDefinition, String> {
    prepare(VismSource { name: "blur".into(), extension: "wgsl".into(), source: text.into() }, "", &[])
}

/// 貼った Shadertoy が、同梱の効果と同じ道で棚に出る(棚に出る条件 = ここを抜けること)。
/// 関数の棚: module の関数を block が WESL の import で呼べる。棚の側で直せば引いた札全部に届く。
#[test]
fn a_block_can_call_a_library_function() {
    let modules = vec![("lib".to_owned(), "fn lib_twice(x: f32) -> f32 { return x * 2.0; }".to_owned())];
    let body = "/*{ \"ID\": \"t.lib\", \"STAGE\": \"block\", \"INPUTS\": [] }*/\nimport package::lib::lib_twice;\nfn block(k: u32, p: BlockParams) -> Offset { return Offset(vec2f(lib_twice(1.0), 0.0), 0.0, 1.0, vec4f(1.0)); }";
    let ok = prepare(VismSource { name: "lib_user".into(), extension: "wgsl".into(), source: body.into() }, "", &modules);
    assert!(ok.is_ok(), "{:?}", ok.err());
    let missing = prepare(VismSource { name: "lib_user".into(), extension: "wgsl".into(), source: body.into() }, "", &[]);
    assert!(missing.is_err(), "without the module the import has nothing to resolve");
}

/// override の札は頭の JSON が無くても札(module ではない)。頭も `fn block(` も無い .wgsl は module のまま。
#[test]
fn an_override_block_is_a_card_not_a_module() {
    let modules = vec![("lib".to_owned(), "fn lib_twice(x: f32) -> f32 { return x * 2.0; }".to_owned())];
    let body = "import package::lib::lib_twice;\n@label(\"Gain\") @range(0.0, 4.0)\noverride gain: f32 = 1.0;\nfn block(k: u32) -> Offset { return Offset(vec2f(lib_twice(gain), 0.0), 0.0, 1.0, vec4f(1.0)); }";
    let card = VismSource { name: "gainer".into(), extension: "wgsl".into(), source: body.into() };
    assert!(!is_module(&card));
    assert!(is_module(&VismSource { name: "lib".into(), extension: "wgsl".into(), source: modules[0].1.clone().into() }));
    let definition = prepare(card, "", &modules).unwrap();
    assert_eq!(definition.plugin_id(), "motolii.gainer");
    assert_eq!(definition.manifest.label.as_deref(), Some("Gainer"));
    assert_eq!(definition.manifest.stage, isf::IsfStage::Block);
    assert_eq!(definition.manifest.inputs.iter().map(|p| p.label.as_deref()).collect::<Vec<_>>(), [Some("Gain")]);
}

#[test]
fn a_pasted_shadertoy_becomes_an_effect() {
    let source = "void mainImage(out vec4 c, in vec2 p) { c = vec4(p / iResolution.xy, sin(iTime), 1.0); }";
    let definition = prepare(VismSource { name: "city".into(), extension: "frag".into(), source: source.into() }, "", &[]).unwrap();
    assert_eq!(definition.plugin_id(), "import.city");
    assert!(matches!(definition.manifest.stage, isf::IsfStage::Pass));
    // manifest の無い素の GLSL は、理由を名前つきで断る。
    let plain = prepare(VismSource { name: "bare".into(), extension: "frag".into(), source: "void main() { gl_FragColor = vec4(1.0); }".into() }, "", &[]).err().expect("manifest の無い GLSL は断る");
    assert!(plain.contains("bare") && plain.contains("manifest"), "{plain}");
}

#[test]
fn default_range_and_body_changes_preserve_the_shader_interface_and_handles() {
    let source = include_str!("../../../../vism/blur.wgsl");
    let old = blur(source.into()).unwrap();
    let new = blur(source.replace("\"DEFAULT\": 8.0", "\"DEFAULT\": 12.0")
        .replace("\"MAX\": 128.0", "\"MAX\": 96.0")
        .replace("radius * 0.5", "radius * 0.6")).unwrap();
    assert_eq!(old.interface, new.interface);
    assert_eq!(old.paths(), new.paths());
    assert_eq!(descriptors(&[old])[0].params[0].default, 8.0);
    let descriptors = descriptors(&[new]);
    assert_eq!(descriptors[0].params[0].default, 12.0);
    assert_eq!(descriptors[0].params[0].range, Some((0.0, 96.0)));
}

#[test]
fn incompatible_bindings_and_missing_entry_points_fail_before_staging() {
    let source = include_str!("../../../../vism/blur.wgsl");
    assert!(blur(source.replace("@group(1)", "@group(2)")).is_err());
    assert!(blur(source.replace("fn fs_main", "fn other_main")).is_err());
    assert!(blur(source.replace("return blur(blur_h_tex", "return missing(blur_h_tex")).is_err());
    let old = blur(source.into()).unwrap();
    let renamed = blur(source.replace("radius", "spread")).unwrap();
    assert_ne!(old.interface, renamed.interface);
}

#[test]
fn copied_entrypoint_rebinds_the_watcher_runtime_and_last_good_snapshot() {
    let runtime = CatalogRuntime::default();
    let source = include_str!("../../../../vism/blur.wgsl");
    let good = blur(source.into()).unwrap();
    let snapshot = Arc::new(CatalogSnapshot {
        generation: 7,
        descriptors: descriptors(std::slice::from_ref(&good)),
        definitions: vec![good].into(),
        errors: vec!["invalid replacement retained the valid program".into()],
    });
    runtime.0.owner.lock().unwrap().snapshot = Some(snapshot.clone());
    runtime.0.generation.store(7, Ordering::Release);
    runtime.0.dirty.store(false, Ordering::Release);
    let copied_slot = Mutex::new(None);
    assert_eq!(runtime_at(&copied_slot).generation(), 0);
    bind_runtime_at(&copied_slot, &runtime);
    let rebound = runtime_at(&copied_slot);
    assert!(blur(source.replace("return blur(blur_h_tex", "return missing(blur_h_tex")).is_err());
    runtime.0.dirty.store(true, Ordering::Release);
    assert_eq!(rebound.identity(), runtime.identity());
    assert_eq!(rebound.generation(), 7);
    assert!(rebound.0.dirty.load(Ordering::Acquire));
    assert!(Arc::ptr_eq(rebound.0.owner.lock().unwrap().snapshot.as_ref().unwrap(), &snapshot));
}

/// 貼った ISF に、点・色の**全成分**と時計が届く(実 GPU、program 1 つで 2×2 を描いて読む)。
/// `name` は shader の本文の置き場になる — 並走する test 同士で同じ名前を使うと上書きし合う。
fn run_once(name: &str, source: &str, params: &[(&str, f32)]) -> [u8; 4] {
    let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
    let definition = prepare(VismSource { name: name.into(), extension: "fs".into(), source: source.into() }, "", &[]).unwrap();
    // shader の本文は棚が書く。ここは棚を通らないので自分で書く(radiance の参照 test と同じ)。
    definition.stage().unwrap();
    let building = compositor.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let program = super::super::EffectProgram::compile_for(&compositor.ctx, &definition, wgpu::TextureFormat::Rgba8Unorm);
    // pipeline は次の frame の頭で組まれる。組ませてから記録し、転送を流してから submit。
    compositor.ctx.before_submit();
    compositor.ctx.begin_frame();
    let error = pollster::block_on(building.pop());
    assert!(error.is_none(), "pipeline が組めない: {error:?}");
    let ctx = &compositor.ctx;
    let texture = |usage| ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: 2, height: 2, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage, view_formats: &[],
    });
    let src = texture(wgpu::TextureUsages::TEXTURE_BINDING);
    let dst = texture(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let mut scratch = super::super::EffectScratch::default();
    let params: Vec<(String, f32)> = params.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect();
    let scope = ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    program.record(ctx, &mut encoder, &mut scratch, &[&src.create_view(&Default::default())], &dst.create_view(&Default::default()), &params, [2.0, 2.0]);
    let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256 * 2, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    encoder.copy_texture_to_buffer(dst.as_image_copy(), wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: Some(2) } }, wgpu::Extent3d { width: 2, height: 2, depth_or_array_layers: 1 });
    let commands = encoder.finish();
    drop(ctx);
    compositor.ctx.before_submit();
    let ctx = &compositor.ctx;
    ctx.queue.submit([commands]);
    let error = pollster::block_on(scope.pop());
    assert!(error.is_none(), "GPU の検証に落ちた: {error:?}");
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    crate::compositor::device::wait_for_gpu(&ctx.device, "catalog-test-readback").unwrap();
    let data = slice.get_mapped_range();
    [data[0], data[1], data[2], data[3]]
}

#[test]
fn a_point_and_a_colour_arrive_with_every_component() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"p\",\"TYPE\":\"point2D\",\"DEFAULT\":[0.0,0.0]}, {\"NAME\":\"c\",\"TYPE\":\"color\",\"DEFAULT\":[0.0,0.0,0.0,0.0]}] }*/\n\
                  void main() { gl_FragColor = vec4(p.x / 64.0, p.y / 64.0, c.b, c.a); }";
    let px = run_once("contract-point", source, &[("p", 16.0), ("p.1", 32.0), ("c.2", 1.0), ("c.3", 0.5)]);
    let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
    assert!(near(px[0], 64) && near(px[1], 128) && near(px[2], 255) && near(px[3], 128), "{px:?}: 成分が欠けている");
}

#[test]
fn the_clock_reaches_the_shader() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/\n\
                  void main() { gl_FragColor = vec4(TIME / 10.0, float(FRAMEINDEX) / 255.0, TIMEDELTA * 10.0, 1.0); }";
    let px = run_once("contract-clock", source, &[("TIME", 2.0), ("FRAMEINDEX", 48.0), ("TIMEDELTA", 0.1)]);
    let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
    assert!(near(px[0], 51) && near(px[1], 48) && near(px[2], 255), "{px:?}: 時計が届いていない");
}

#[cfg(load_shaders_from_disk)]
#[test]
fn catalog_refresh_keeps_the_open_renderer_frame_and_device() {
    let mut compositor = crate::render::compositor::Compositor::headless().unwrap();
    compositor.ctx.before_submit();
    compositor.ctx.begin_frame();
    let current = catalog_snapshot();
    compositor.catalog = Arc::new(CatalogSnapshot {
        generation: current.generation.wrapping_sub(1),
        definitions: current.definitions.clone(),
        descriptors: current.descriptors.clone(),
        errors: Vec::new(),
    });
    let frame = compositor.ctx.active_frame_idx();
    let device = compositor.ctx.device.clone();
    let shaders = compositor.ctx.gpu_resources.shader_modules.num_resources();
    compositor.refresh_catalog_programs();
    assert_eq!(compositor.ctx.active_frame_idx(), frame);
    assert_eq!(compositor.ctx.device, device);
    assert_eq!(compositor.ctx.gpu_resources.shader_modules.num_resources(), shaders);
}
