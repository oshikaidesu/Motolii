//! 借りた式(reference/vello-blend.wgsl)と BlendMode が正しい番号で繋がっていること。
//! 白と黒だけを使う — この組の答えは色空間に依らないので、線形かガンマかを
//! 判定に持ち込まずにモードの取り違えだけを刺せる。

use motolii::render::compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, ResolvedCamera,
};

const W: u32 = 64;
const H: u32 = 64;

fn compositor() -> Compositor {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    Compositor::with_device_using_headless_defaults(device, queue).expect("with_device")
}

/// 背景(下)と前景(上)を全面に重ねて、真ん中の画素を返す。
fn blended(c: &mut Compositor, backdrop: u8, source: u8, mode: BlendMode) -> [u8; 4] {
    let comp = CompSpec { width: W, height: H };
    let make = |c: &mut Compositor, name: &str, v: u8| {
        let mut px = vec![255u8; (W * H * 4) as usize];
        for p in px.chunks_mut(4) {
            p[0] = v;
            p[1] = v;
            p[2] = v;
        }
        c.upload_rgba(name, &px, W, H).expect("upload")
    };
    let under = make(c, "under", backdrop);
    let over = make(c, "over", source);
    let place = |tex, order: i16, blend| Layer {
        texture: tex,
        size: [W as f32, H as f32],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 0.0, 0.0,
            ),
            order,
            opacity: 1.0,
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: blend,
    };
    let layers = vec![place(under, 0, BlendMode::Normal), place(over, 1, mode)];
    let px = c
        .render_sequential(comp, ResolvedCamera::default(), &layers, [0.0, 0.0, 0.0, 1.0])
        .expect("render_sequential");
    let i = (((H / 2) * W + W / 2) * 4) as usize;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

fn rgb(p: [u8; 4]) -> [u8; 3] {
    [p[0], p[1], p[2]]
}

#[test]
fn multiply_with_white_keeps_the_backdrop_and_with_black_goes_black() {
    let mut c = compositor();
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Multiply)), [0, 0, 0], "黒 × 白 = 黒");
    assert_eq!(rgb(blended(&mut c, 255, 0, BlendMode::Multiply)), [0, 0, 0], "白 × 黒 = 黒");
    assert_eq!(rgb(blended(&mut c, 255, 255, BlendMode::Multiply)), [255, 255, 255], "白 × 白 = 白");
}

#[test]
fn screen_with_white_goes_white() {
    let mut c = compositor();
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Screen)), [255, 255, 255], "黒 ∪ 白 = 白");
    assert_eq!(rgb(blended(&mut c, 255, 0, BlendMode::Screen)), [255, 255, 255], "白 ∪ 黒 = 白");
    assert_eq!(rgb(blended(&mut c, 0, 0, BlendMode::Screen)), [0, 0, 0], "黒 ∪ 黒 = 黒");
}

#[test]
fn darken_takes_the_darker_and_lighten_the_lighter() {
    let mut c = compositor();
    assert_eq!(rgb(blended(&mut c, 255, 0, BlendMode::Darken)), [0, 0, 0]);
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Darken)), [0, 0, 0]);
    assert_eq!(rgb(blended(&mut c, 255, 0, BlendMode::Lighten)), [255, 255, 255]);
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Lighten)), [255, 255, 255]);
}

#[test]
fn difference_of_black_and_white_is_white() {
    let mut c = compositor();
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Difference)), [255, 255, 255]);
    assert_eq!(rgb(blended(&mut c, 255, 255, BlendMode::Difference)), [0, 0, 0], "同じ物の差は0");
}

#[test]
fn luminosity_takes_the_source_brightness() {
    let mut c = compositor();
    // 非分離モード。白い層の輝度を黒い背景へ載せる = 白。
    assert_eq!(rgb(blended(&mut c, 0, 255, BlendMode::Luminosity)), [255, 255, 255]);
    assert_eq!(rgb(blended(&mut c, 255, 0, BlendMode::Luminosity)), [0, 0, 0]);
}
