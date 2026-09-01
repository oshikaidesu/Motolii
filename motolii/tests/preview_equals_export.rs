mod testkit;


use motolii::render::compositor::{LayerContent, 
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, LayerWithPasses,
    ResolvedCamera, PRESENTABLE_FORMAT,
};

const W: u32 = 32;
const H: u32 = 32;

fn comp() -> CompSpec {
    CompSpec { width: W, height: H }
}

fn layers(c: &mut Compositor, mode: BlendMode) -> Vec<LayerWithPasses> {
    let under = c
        .upload_rgba("under", &vec![200u8; (W * H * 4) as usize], W, H)
        .expect("under");
    let over = c
        .upload_rgba("over", &vec![120u8; (W * H * 4) as usize], W, H)
        .expect("over");
    let place = |texture, order, blend_mode| LayerWithPasses {
        layer: Layer {
            content: LayerContent::Texture(texture),
            size: [W as f32, H as f32],
            placement: LayerPlacement { order, ..LayerPlacement::default() },
            pinned: false,
            blend_mode,
        },
        passes: vec![],
    };
    vec![place(under, 0, BlendMode::Normal), place(over, 1, mode)]
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bpr = (W * 4).div_ceil(align) * align;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out = data[..4].to_vec();
    drop(data);
    buf.unmap();
    out
}

#[test]
fn render_into_matches_render_with_effects_for_separable_blends() {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let mut c = Compositor::with_device_using_headless_defaults(device.clone(), queue.clone())
        .expect("with_device");

    let mut mismatched = Vec::new();
    for mode in [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen, BlendMode::Difference] {
        let ls = layers(&mut c, mode);
        let cpu = c
            .render_with_effects(comp(), ResolvedCamera::default(), &ls, motolii::render::compositor::NO_BACKGROUND)
            .expect("render_with_effects")[..4]
            .to_vec();

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        c.render_into(&target, comp(), ResolvedCamera::default(), &ls, motolii::render::compositor::NO_BACKGROUND)
            .expect("render_into");
        let gpu = readback(&device, &queue, &target);

        if cpu.iter().zip(&gpu).any(|(a, b)| (*a as i32 - *b as i32).abs() > 2) {
            mismatched.push(format!("{mode:?}: export={cpu:?} stage={gpu:?}"));
        }
    }

    assert!(
        mismatched.is_empty(),
        "Stage と export の絵が違う blend mode がある:\n{}",
        mismatched.join("\n")
    );
}

/// 同じ時刻の絵は、そこへどう辿り着いても同じでなければならない。
///
/// デコーダが間に合わない時に前のコマを返していると、窓(毎フレーム回る)と
/// 書き出し(1コマ1回)で違う絵が出る。憲法「評価経路を2本にしない」の実物。
#[test]
fn the_same_time_gives_the_same_picture_however_you_got_there() {
    use motolii::doc::store::{
        Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
        LayerTiming, RationalTime,
    };
    use motolii::render::engine::Engine;

    if !testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = testkit::tmp_dir("preview-equals-export-video");
    let clip = dir.join("clip.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-v", "error", "-y", "-f", "lavfi", "-i",
            "testsrc=size=64x64:rate=30:duration=1", "-pix_fmt", "yuv420p",
        ])
        .arg(&clip)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");

    let build = || {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: clip.to_str().unwrap().to_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 30),
            },
        })
        .unwrap();
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { name: Some("clip".into()), ..Default::default() },
        })
        .unwrap();
        doc
    };

    let at = RationalTime::try_new(10, 30).unwrap();

    // 層が描かれなかった時の絵(背景だけ)。これと一致したら「出ていない」。
    let empty_doc = {
        let mut d = Document::new();
        d.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        d
    };
    let black = Engine::new()
        .unwrap()
        .render_frame(&empty_doc.view(), at)
        .unwrap();

    // 書き出しの一手目 —— 冷えたまま、その時刻を1回だけ頼む。
    let export_doc = build();
    let mut export = Engine::new().unwrap();
    let from_export = export.render_frame(&export_doc.view(), at).unwrap();

    // 窓 —— 実時間で回り続けて、デコーダが温まってから同じ時刻を見る。
    // 温まるのに要るのは呼んだ回数ではなく**実時間**(FFmpeg の起動)。
    let preview_doc = build();
    let mut preview = Engine::new().unwrap();
    let mut from_preview = preview.render_frame(&preview_doc.view(), at).unwrap();
    for _ in 0..150 {
        if from_preview != black {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        from_preview = preview.render_frame(&preview_doc.view(), at).unwrap();
    }
    assert_ne!(from_preview, black, "窓が温まっても絵が出ない");

    assert!(
        export.layer_failures().is_empty(),
        "書き出しで絵が落ちている: {:?}",
        export.layer_failures()
    );
    let differing = from_export
        .iter()
        .zip(&from_preview)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        differing, 0,
        "同じ時刻なのに、辿り着き方で絵が違う({differing}/{} バイト、書き出しは背景のまま={})",
        from_export.len(),
        from_export == black
    );
}
