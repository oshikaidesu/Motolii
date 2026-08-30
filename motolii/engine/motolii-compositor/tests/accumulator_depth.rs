//! run が複数に割れた時、accumulator の板が z を持つ層を上塗りしないこと。
//!
//! 上流は矩形をカメラからの距離で並べる。板を z=0 に置くと、光軸から離れた層は
//! z が小さくても距離では板より遠くなり、板に消される。

use motolii_compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, ResolvedCamera,
};

const W: u32 = 640;
const H: u32 = 360;

fn compositor() -> Compositor {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    Compositor::with_device_using_headless_defaults(device, queue).expect("with_device")
}

#[test]
fn a_layer_with_depth_survives_the_accumulator_across_runs() {
    let mut c = compositor();
    let comp = CompSpec { width: W, height: H };

    let grey = c
        .upload_rgba("grey", &vec![90u8; (200 * 120 * 4) as usize], 200, 120)
        .expect("grey");
    let white = c
        .upload_rgba("white", &vec![255u8; (200 * 120 * 4) as usize], 200, 120)
        .expect("white");

    // 左上の隅(光軸から遠い)へ置く。ここが板との距離比べで負ける場所。
    let place = |tex: motolii_compositor::GpuTexture2D, order: i16, z: f32, blend| Layer {
        texture: tex,
        size: [200.0, 120.0],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 0.0, 0.0,
            ),
            order,
            opacity: 1.0,
            z,
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: blend,
    };

    let white_pixels = |c: &mut Compositor, z: f32| {
        // Multiply を挟んで run を割る(accumulator が実際に敷かれる状況を作る)。
        let layers = vec![
            place(grey.clone(), 0, 0.0, BlendMode::Normal),
            place(grey.clone(), 1, 0.0, BlendMode::Multiply),
            place(white.clone(), 2, z, BlendMode::Normal),
        ];
        c.render_sequential(comp, ResolvedCamera::default(), &layers, [0.0, 0.0, 0.0, 1.0])
            .expect("render_sequential")
            .chunks(4)
            .filter(|p| p[0] > 200 && p[1] > 200)
            .count()
    };

    let flat = white_pixels(&mut c, 0.0);
    let near = white_pixels(&mut c, -60.0);
    let far = white_pixels(&mut c, 60.0);

    assert!(flat > 0, "z=0 で最前面の板が出ていない");
    assert!(near > 0, "手前へ動かした板が accumulator に消された");
    assert!(far > 0, "奥へ動かした板が accumulator に消された");
}
