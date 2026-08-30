//! 同じ絵を何百フレームも描き続けても壊れないこと。
//!
//! 窓が**誰も触っていないのに5秒で落ちた**(2026-08-30)。ログには
//! `Buffer binding 1 range 201326592 exceeds max_*_buffer_binding_size limit 134217728`
//! が119回。操作ではなく**時間**で壊れている = フレームを跨いで何かが積み上がっている。
//! 1フレームだけ描くテストでは絶対に捕まらないので、ここで回し続ける。

use motolii::render::compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, ResolvedCamera,
};

const W: u32 = 128;
const H: u32 = 128;
const FRAMES: usize = 240;

fn layer(c: &mut Compositor, name: &str, blend: BlendMode, order: i16) -> Layer {
    let tex = c
        .upload_rgba(name, &vec![200u8; (64 * 64 * 4) as usize], 64, 64)
        .expect("upload");
    Layer {
        texture: tex,
        size: [64.0, 64.0],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [order as f32 * 4.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 0.0, 0.0,
            ),
            order,
            opacity: 1.0,
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: blend,
    }
}

#[test]
fn hundreds_of_frames_do_not_grow_the_gpu_state() {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let mut c =
        Compositor::with_device_using_headless_defaults(device.clone(), queue).expect("with_device");

    let layers = vec![
        layer(&mut c, "a", BlendMode::Normal, 0),
        layer(&mut c, "b", BlendMode::Multiply, 1),
        layer(&mut c, "c", BlendMode::Normal, 2),
    ];

    let guard = device.push_error_scope(wgpu::ErrorFilter::Validation);
    for _ in 0..FRAMES {
        c.render_sequential(
            CompSpec { width: W, height: H },
            ResolvedCamera::default(),
            &layers,
            [0.0, 0.0, 0.0, 1.0],
        )
        .expect("render_sequential");
    }
    if let Some(err) = pollster::block_on(guard.pop()) {
        panic!(
            "{FRAMES} フレーム回したら GPU 側が壊れた(フレームを跨いで何かが積み上がっている):\n{err}"
        );
    }
}

/// 窓と同じ経路(engine → render_frame_into)で回す。層の texture を毎フレーム
/// 作り直す所や、上流の texture manager の世代交代はここにしか無い。
#[test]
fn hundreds_of_frames_through_the_window_path_do_not_grow() {
    use motolii::doc::store::RationalTime;
    use motolii::render::engine::Engine;

    let fx = motolii::doc::fixture::build();
    let mut engine = Engine::new().expect("engine");
    let device = engine.gpu_device().clone();
    let view = fx.doc.view();

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frame-loop-target"),
        size: wgpu::Extent3d { width: 1920, height: 1080, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: motolii::render::compositor::PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        // 窓(ui/stage_widget.rs の create_target)と同じ宣言にする
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    });

    let guard = device.push_error_scope(wgpu::ErrorFilter::Validation);
    for frame in 0..FRAMES {
        let t = RationalTime::try_new(frame as i64, 30).expect("時刻");
        engine
            .render_frame_into(&view, t, &target)
            .expect("render_frame_into");
    }
    if let Some(err) = pollster::block_on(guard.pop()) {
        panic!("{FRAMES} フレーム回したら GPU 側が壊れた(窓と同じ経路):\n{err}");
    }
}
