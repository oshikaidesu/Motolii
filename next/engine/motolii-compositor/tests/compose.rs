//! 合成の契約 — 重ね順・不透明度・**preview と export が同じ絵になること**。

use motolii_compositor::{CompSpec, Compositor, Layer};

const W: u32 = 64;
const H: u32 = 64;

fn solid(rgba: [u8; 4], w: u32, h: u32) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((w * h * 4) as usize)
        .collect()
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

#[test]
fn layers_stack_by_order_and_position() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();
    let green = compositor
        .upload_rgba("green", &solid([0, 255, 0, 255], 32, 32), 32, 32)
        .unwrap();

    let frame = compositor
        .render(
            comp(),
            &[
                Layer {
                    texture: red,
                    top_left: [0.0, 0.0],
                    size: [W as f32, H as f32],
                    order: 0,
                    opacity: 1.0,
                },
                Layer {
                    texture: green,
                    // comp 座標は左上原点。左上の 1/4 を覆う。
                    top_left: [0.0, 0.0],
                    size: [32.0, 32.0],
                    order: 1,
                    opacity: 1.0,
                },
            ],
        )
        .unwrap();

    assert_eq!(frame.len(), (W * H * 4) as usize);

    let covered = pixel(&frame, 10, 10);
    let uncovered = pixel(&frame, 50, 50);

    assert!(
        covered[1] > covered[0],
        "手前の layer が覆っている場所は緑が勝つはず: {covered:?}"
    );
    assert!(
        uncovered[0] > uncovered[1],
        "覆われていない場所は奥の赤が見えるはず: {uncovered:?}"
    );
}

/// **決定性**の試験。同じ入力から同じ絵が出ること。
///
/// **これは「経路が1本であること」の試験ではない**(2026-08-20 の敵対的レビュー)。
/// 同じ関数を2回呼んでいるだけなので、第二経路が生えても絶対に落ちない。
/// 以前は `preview` / `export` という変数名で試験のふりをしていたので、名前を正した。
/// 経路が1本であることは**依存グラフ**が守る — `motolii-export` は
/// `motolii-compositor` を引いておらず、第二の `Compositor` を建てられない。
#[test]
fn render_is_deterministic() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();

    let layer = Layer {
        texture: red,
        top_left: [0.0, 0.0],
        size: [W as f32, H as f32],
        order: 0,
        opacity: 1.0,
    };

    let first = compositor.render(comp(), std::slice::from_ref(&layer)).unwrap();
    let second = compositor.render(comp(), std::slice::from_ref(&layer)).unwrap();

    assert_eq!(first, second, "同じ入力から違う絵が出る(決定的でない)");
}

#[test]
fn opacity_dims_the_layer() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let white = compositor
        .upload_rgba("white", &solid([255, 255, 255, 255], W, H), W, H)
        .unwrap();

    let make = |opacity: f32| Layer {
        texture: white.clone(),
        top_left: [0.0, 0.0],
        size: [W as f32, H as f32],
        order: 0,
        opacity,
    };

    let full = compositor.render(comp(), &[make(1.0)]).unwrap();
    let half = compositor.render(comp(), &[make(0.5)]).unwrap();
    let none = compositor.render(comp(), &[make(0.0)]).unwrap();

    let full_px = pixel(&full, 32, 32);
    let half_px = pixel(&half, 32, 32);
    let none_px = pixel(&none, 32, 32);

    assert!(
        none_px[0] < half_px[0] && half_px[0] < full_px[0],
        "不透明度は単調に効くはず: 0.0={none_px:?} 0.5={half_px:?} 1.0={full_px:?}"
    );
    assert_eq!(full_px[0], 255, "1.0 は元の白のまま");
}

#[test]
fn empty_comp_is_the_clear_color() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let frame = compositor.render(comp(), &[]).unwrap();
    assert_eq!(frame.len(), (W * H * 4) as usize);

    let px = pixel(&frame, 32, 32);
    assert_eq!([px[0], px[1], px[2]], [0, 0, 0], "layer が無ければ RGB は clear 色");
}

/// **現状の限界を固定する試験**(R0-A と同じ役割)。
///
/// `ScreenshotProcessor` は compositing phase の結果を撮るので、**alpha は clear 色へ
/// 潰れて常に 255 になる**。preview はこれで良いが、alpha 付き書き出し
/// (ProRes 4444 / PNG 連番)はこの経路では出せない。
///
/// 直す時は `ViewBuilder` の main target を composite 前に読む経路を足すことになる
/// (fork seam は要らない見込み)。**その日にこの試験が落ちるので、忘れずに気付ける**。
#[test]
fn alpha_is_flattened_by_the_composite_step() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let empty = compositor.render(comp(), &[]).unwrap();
    assert_eq!(
        pixel(&empty, 32, 32)[3],
        255,
        "layer が1枚も無い comp でも alpha は 255 になる(透明が出せない)"
    );

    let half_alpha = compositor
        .upload_rgba("half-alpha", &solid([128, 0, 0, 128], W, H), W, H)
        .unwrap();
    let out = compositor
        .render(
            comp(),
            &[Layer {
                texture: half_alpha,
                top_left: [0.0, 0.0],
                size: [W as f32, H as f32],
                order: 0,
                opacity: 1.0,
            }],
        )
        .unwrap();
    let px = pixel(&out, 32, 32);
    assert_eq!(px[3], 255, "半透明 layer を1枚置いても alpha は 255 になる");
    assert!(px[0] > 0 && px[0] < 255, "RGB 側は正しく混ざっている: {px:?}");
}


/// **preview を iced の device で、export を別 device で回しても同じ絵か**。
///
/// iced の `shader::Primitive` は `prepare(&self, .., device: &wgpu::Device, queue: &wgpu::Queue, ..)`
/// と `render(.., encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, ..)` を渡すので、
/// re_renderer を **iced の device の上に**建てられる(= 読み戻し無しの埋め込み)。
/// その時 preview と export は別 device になりうる。背骨2(preview = export)が
/// **device を跨いでも**成り立つかは仮定してはいけないので、ここで測る。
#[test]
fn two_devices_produce_the_same_frame() {
    let make = |compositor: &mut Compositor| {
        let tex = compositor
            .upload_rgba("checker", &solid([200, 40, 90, 255], W, H), W, H)
            .unwrap();
        let small = compositor
            .upload_rgba("small", &solid([20, 180, 60, 128], 32, 32), 32, 32)
            .unwrap();
        vec![
            Layer {
                texture: tex,
                top_left: [0.0, 0.0],
                size: [W as f32, H as f32],
                order: 0,
                opacity: 1.0,
            },
            Layer {
                texture: small,
                top_left: [8.0, 12.0],
                size: [32.0, 32.0],
                order: 1,
                opacity: 0.5,
            },
        ]
    };

    let mut a = Compositor::headless().expect("device A");
    let layers_a = make(&mut a);
    let frame_a = a.render(comp(), &layers_a).unwrap();

    let mut b = Compositor::headless().expect("device B");
    let layers_b = make(&mut b);
    let frame_b = b.render(comp(), &layers_b).unwrap();

    assert_eq!(
        frame_a, frame_b,
        "別 device で絵が変わるなら、preview(iced の device)と export(別 device)を\
         同じと言えなくなる"
    );
}
