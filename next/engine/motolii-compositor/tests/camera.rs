//! `Composition.camera` の GPU 側の縛る試験(裁定113/115、裁定116 実装)。
//!
//! `motolii-core::camera` の単体試験(GPU 無し)がカメラの投影数学そのものを
//! 機械精度で縛っている。ここは**実際にラスタライズした画素**で同じことを確かめる —
//! 数学が正しくても `re_renderer` への詰め替え(`Projection::Perspective` /
//! `macaw::IsoTransform` への変換)を間違えれば画は狂うので、その繋ぎ目を GPU 込みで縛る。

use motolii_compositor::{CompSpec, Compositor, Layer, LayerPlacement, ResolvedCamera};

fn solid(rgba: [u8; 4], w: u32, h: u32) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((w * h * 4) as usize)
        .collect()
}

fn pixel(buffer: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * width + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn flat_layer(
    texture: motolii_compositor::GpuTexture2D,
    size: [f32; 2],
    position: [f32; 2],
    z: f32,
) -> Layer {
    Layer {
        texture,
        size,
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [0.0, 0.0],
                position,
                [1.0, 1.0],
                0.0,
                0.0,
                0.0,
            ),
            order: 0,
            opacity: 1.0,
            z,
        },
        pinned: false,
    }
}

/// 分析的に「正しい」ピクセルを計算する — 軸に沿った未回転の矩形を敷いただけの
/// シーンなら、答えは `if 矩形の内側 { 矩形の色 } else { 背景色 }` で決まる。
/// GPU の実測とこれを比べれば、「正射影と一致する」を**測れる数値**にできる。
fn expected_axis_aligned_frame(
    width: u32,
    height: u32,
    background: [u8; 4],
    rects: &[([u32; 2], [u32; 2], [u8; 4])],
) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let mut color = background;
            for &([x0, y0], [x1, y1], c) in rects {
                if x >= x0 && x < x1 && y >= y0 && y < y1 {
                    color = c;
                }
            }
            let i = ((y * width + x) * 4) as usize;
            buf[i..i + 4].copy_from_slice(&color);
        }
    }
    buf
}

fn max_channel_diff(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// **縛る試験1**: 全層 z=0・カメラ既定のとき、絵が現行の正射影(裁定14)と一致する。
///
/// 軸に沿った未回転の矩形は座標が分析的に決まるので、GPU の実測をそれと画素比較する。
/// **実測**: 内部(境界から1px 離れた領域)は byte 一致(diff=0)。境界ぎりぎりの1px帯だけ
/// 透視の tan/atan を経由する分だけ ortho の厳密な線形写像とわずかに丸めがずれ得るので、
/// 境界帯は別に測って明記する(このコメントの下の `assert` がその実測値)。
#[test]
fn default_camera_all_z0_matches_orthographic_pixel_mapping() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 200,
        height: 200,
    };

    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], 60, 60), 60, 60)
        .unwrap();
    let green = compositor
        .upload_rgba("green", &solid([0, 255, 0, 255], 40, 40), 40, 40)
        .unwrap();

    let layers = [
        flat_layer(red.clone(), [60.0, 60.0], [20.0, 20.0], 0.0),
        flat_layer(green.clone(), [40.0, 40.0], [120.0, 130.0], 0.0),
    ];

    let actual = compositor
        .render(comp, ResolvedCamera::default(), &layers)
        .unwrap();

    let expected = expected_axis_aligned_frame(
        comp.width,
        comp.height,
        [0, 0, 0, 255],
        &[
            ([20, 20], [80, 80], [255, 0, 0, 255]),
            ([120, 130], [160, 170], [0, 255, 0, 255]),
        ],
    );

    // 内部(境界から離れた場所)は完全一致。
    let interior_diff = max_channel_diff(&pixel(&actual, comp.width, 40, 40), &[255, 0, 0, 255]);
    assert_eq!(interior_diff, 0, "矩形内部は byte 一致するはず");
    let interior_diff_bg =
        max_channel_diff(&pixel(&actual, comp.width, 100, 100), &[0, 0, 0, 255]);
    assert_eq!(interior_diff_bg, 0, "背景は byte 一致するはず");

    // 画面全体の最大差(境界の丸めを含む)。**この値が「正射影との一致」の実測**
    // (`cargo test -- --nocapture` で見える。裁定116 の終了報告に転記する数値)。
    let max_diff = max_channel_diff(&actual, &expected);
    println!("default_camera_all_z0_matches_orthographic_pixel_mapping: max_diff(全画素)={max_diff}");
    // 境界1px帯を除いた領域では厳密に一致することを別途確かめる(丸めが境界だけに
    // 閉じ込められていることの実証)。
    let mut max_diff_excluding_edges: u8 = 0;
    for y in 2..comp.height - 2 {
        for x in 2..comp.width - 2 {
            // 矩形境界(±1px)は除く。
            let near_boundary = [(20i32, 20i32, 80, 80), (120, 130, 160, 170)]
                .iter()
                .any(|&(x0, y0, x1, y1)| {
                    let xi = x as i32;
                    let yi = y as i32;
                    (xi - x0).abs() <= 1
                        || (xi - x1).abs() <= 1
                        || (yi - y0).abs() <= 1
                        || (yi - y1).abs() <= 1
                });
            if near_boundary {
                continue;
            }
            let i = ((y * comp.width + x) * 4) as usize;
            let d = max_channel_diff(&actual[i..i + 4], &expected[i..i + 4]);
            max_diff_excluding_edges = max_diff_excluding_edges.max(d);
        }
    }
    assert_eq!(
        max_diff_excluding_edges, 0,
        "境界1px帯を除けば byte 一致するはず(measured={max_diff_excluding_edges})"
    );
}

/// **縛る試験2**: 層2枚(z 違い)+カメラ center 移動 → 近い層が遠い層より大きく動く。
#[test]
fn panning_the_camera_moves_nearer_layers_more_than_farther_layers() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 400,
        height: 400,
    };
    let texture = compositor
        .upload_rgba("marker", &solid([255, 0, 0, 255], 40, 40), 40, 40)
        .unwrap();

    let default_camera = ResolvedCamera::default();
    let panned_camera = ResolvedCamera {
        center: [80.0, 0.0],
        ..Default::default()
    };

    // z=-150(カメラに近い)と z=+150(カメラから遠い)。base_distance(400 高さ)は
    // 約384px なので、どちらも near plane を割り込まない。
    let near = flat_layer(texture.clone(), [40.0, 40.0], [180.0, 180.0], -150.0);
    let far = flat_layer(texture.clone(), [40.0, 40.0], [180.0, 180.0], 150.0);

    let near_before = compositor.render(comp, default_camera, &[near.clone()]).unwrap();
    let near_after = compositor.render(comp, panned_camera, &[near.clone()]).unwrap();
    let far_before = compositor.render(comp, default_camera, &[far.clone()]).unwrap();
    let far_after = compositor.render(comp, panned_camera, &[far.clone()]).unwrap();

    let edge_x = |buffer: &[u8], y: u32| -> Option<i64> {
        (0..comp.width).find(|&x| pixel(buffer, comp.width, x, y)[0] > 128).map(i64::from)
    };

    let y = 200;
    let near_edge_before = edge_x(&near_before, y).expect("near layer visible before pan");
    let near_edge_after = edge_x(&near_after, y).expect("near layer visible after pan");
    let far_edge_before = edge_x(&far_before, y).expect("far layer visible before pan");
    let far_edge_after = edge_x(&far_after, y).expect("far layer visible after pan");

    let near_delta = (near_edge_before - near_edge_after).abs();
    let far_delta = (far_edge_before - far_edge_after).abs();

    assert!(
        near_delta > far_delta,
        "近い層(delta={near_delta})が遠い層(delta={far_delta})より大きく動くはず"
    );
}

/// **縛る試験3前半**: zoom で z=0 の板が等方に拡大する。
#[test]
fn zoom_scales_the_z0_layer_isotropically() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 400,
        height: 400,
    };
    let texture = compositor
        .upload_rgba("marker", &solid([255, 0, 0, 255], 40, 40), 40, 40)
        .unwrap();
    // comp 中心(200,200)を中心に置いた矩形。
    let layer = flat_layer(texture, [40.0, 40.0], [180.0, 180.0], 0.0);

    let default_frame = compositor
        .render(comp, ResolvedCamera::default(), &[layer.clone()])
        .unwrap();
    let zoomed_camera = ResolvedCamera {
        zoom: 2.0,
        ..Default::default()
    };
    let zoomed_frame = compositor.render(comp, zoomed_camera, &[layer]).unwrap();

    let width_at = |buffer: &[u8], y: u32| -> u32 {
        (0..comp.width)
            .filter(|&x| pixel(buffer, comp.width, x, y)[0] > 128)
            .count() as u32
    };
    let height_at = |buffer: &[u8], x: u32| -> u32 {
        (0..comp.height)
            .filter(|&y| pixel(buffer, comp.width, x, y)[0] > 128)
            .count() as u32
    };

    let default_w = width_at(&default_frame, 200);
    let zoomed_w = width_at(&zoomed_frame, 200);
    let default_h = height_at(&default_frame, 200);
    let zoomed_h = height_at(&zoomed_frame, 200);

    assert!(
        zoomed_w > default_w,
        "zoom=2 で幅が拡大するはず: default={default_w} zoomed={zoomed_w}"
    );
    assert!(
        zoomed_h > default_h,
        "zoom=2 で高さが拡大するはず: default={default_h} zoomed={zoomed_h}"
    );
    // **等方**: 幅と高さがほぼ同じ比率で拡大する(縦だけ/横だけ拡大しない)。
    let width_ratio = zoomed_w as f32 / default_w as f32;
    let height_ratio = zoomed_h as f32 / default_h as f32;
    assert!(
        (width_ratio - height_ratio).abs() < 0.15,
        "等方でない: width_ratio={width_ratio} height_ratio={height_ratio}"
    );
}

/// **縛る試験3後半**: roll で z=0 の板が回る。
#[test]
fn roll_rotates_the_z0_layer() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 400,
        height: 400,
    };
    let texture = compositor
        .upload_rgba("marker", &solid([255, 0, 0, 255], 20, 20), 20, 20)
        .unwrap();
    // comp 中心から右(+X)に離れた位置に小さな矩形を置く。
    let layer = flat_layer(texture, [20.0, 20.0], [280.0, 190.0], 0.0);

    let default_frame = compositor
        .render(comp, ResolvedCamera::default(), &[layer.clone()])
        .unwrap();
    let rolled_camera = ResolvedCamera {
        roll_degrees: 90.0,
        ..Default::default()
    };
    let rolled_frame = compositor.render(comp, rolled_camera, &[layer]).unwrap();

    let centroid = |buffer: &[u8]| -> (i64, i64) {
        let mut sum = (0i64, 0i64);
        let mut count = 0i64;
        for y in 0..comp.height {
            for x in 0..comp.width {
                if pixel(buffer, comp.width, x, y)[0] > 128 {
                    sum.0 += x as i64;
                    sum.1 += y as i64;
                    count += 1;
                }
            }
        }
        assert!(count > 0, "layer が見つからない");
        (sum.0 / count, sum.1 / count)
    };

    let (default_x, default_y) = centroid(&default_frame);
    let (rolled_x, rolled_y) = centroid(&rolled_frame);

    // roll=90(時計回り)で「comp 中心から右」にあったマーカーは「comp 中心から下」へ
    // 来るはず(motolii-core の純粋数学試験と同じ主張を画素で確かめる)。
    assert!(
        (rolled_x - 200).abs() < (default_x - 200).abs(),
        "roll 後は x が中心へ寄るはず: default_x={default_x} rolled_x={rolled_x}"
    );
    assert!(
        rolled_y > default_y + 20,
        "roll 後は y が中心より下へ移るはず: default_y={default_y} rolled_y={rolled_y}"
    );
}

/// **縛る試験4**: pinned 層はカメラをどう動かしても画面上不動。
#[test]
fn pinned_layers_ignore_the_camera() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 200,
        height: 200,
    };
    let texture = compositor
        .upload_rgba("marker", &solid([255, 0, 0, 255], 30, 30), 30, 30)
        .unwrap();

    let mut pinned = flat_layer(texture, [30.0, 30.0], [60.0, 60.0], 40.0);
    pinned.pinned = true;

    let baseline = compositor
        .render(comp, ResolvedCamera::default(), &[pinned.clone()])
        .unwrap();

    let cameras = [
        ResolvedCamera {
            center: [50.0, -30.0],
            ..Default::default()
        },
        ResolvedCamera {
            zoom: 2.5,
            ..Default::default()
        },
        ResolvedCamera {
            roll_degrees: 35.0,
            ..Default::default()
        },
        ResolvedCamera {
            center: [-20.0, 15.0],
            zoom: 0.6,
            roll_degrees: 200.0,
        },
    ];

    for camera in cameras {
        let frame = compositor.render(comp, camera, &[pinned.clone()]).unwrap();
        assert_eq!(
            frame, baseline,
            "pinned 層はカメラ {camera:?} でも動いてはいけない"
        );
    }
}

/// pinned 層でも、カメラを一切動かさなければ(既定カメラ)通常層と同じ絵になる
/// ("画面に張り付く" が「常に既定カメラでの見え方」であることの確認)。
#[test]
fn pinned_layer_at_default_camera_matches_an_unpinned_layer() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let comp = CompSpec {
        width: 200,
        height: 200,
    };
    let texture = compositor
        .upload_rgba("marker", &solid([10, 200, 90, 255], 30, 30), 30, 30)
        .unwrap();

    let unpinned = flat_layer(texture.clone(), [30.0, 30.0], [60.0, 60.0], 0.0);
    let mut pinned = flat_layer(texture, [30.0, 30.0], [60.0, 60.0], 0.0);
    pinned.pinned = true;

    let a = compositor
        .render(comp, ResolvedCamera::default(), &[unpinned])
        .unwrap();
    let b = compositor
        .render(comp, ResolvedCamera::default(), &[pinned])
        .unwrap();
    assert_eq!(a, b);
}
