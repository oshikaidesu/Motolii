//! BL4(非分離4種: Hue/Saturation/Color/Luminosity)の数値検証(ORACLE
//! 「非分離4の数式を WGSL と別実装で照合」)。`tests/blend_separable.rs` と同型——
//! 2枚の**不透明**単色 layer(base=backdrop・top=source)を `Engine::render_frame`
//! で描き、center pixel を「このテストが独立に書いた W3C Compositing 3.7節の
//! SetLum/SetSat/ClipColor 擬似コード」の計算値と突き合わせる。両方 opaque
//! (αs=αb=1)なので一般合成式は `Co = B(Cb, Cs)` へ潰れる。
//!
//! **色**: 非分離4種は RGB を1単位として扱う(Hue/Saturation が意味を持つには
//! 彩度のある色が要る)ので、`blend_separable.rs` の gray×gray とは違い、
//! base/top ともに彩度のある異なる色を使う。
//!
//! gamma: `blend_separable.rs` と同じ前提(sRGB 8bit 素材・blend は linear 空間)。

use motolii_engine::Engine;
use motolii_store::{
    BlendMode, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, RationalTime,
};

const W: u32 = 8;
const H: u32 = 8;
/// `blend_separable.rs` と同じ許容差(8bit 量子化 + GPU sRGB ハードウェア変換近似)。
const TOLERANCE: i32 = 4;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// W3C Compositing and Blending Level 1、3.7節(Non-separable blend modes)の
/// `Lum`/`ClipColor`/`SetLum`/`Sat`/`SetSat` 擬似コード。`motolii-compositor` の
/// WGSL(`blend.rs` の `nonseparable_blend` 等)とは独立実装。
fn lum(c: [f64; 3]) -> f64 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c_in: [f64; 3]) -> [f64; 3] {
    let l = lum(c_in);
    let n = c_in[0].min(c_in[1]).min(c_in[2]);
    let x = c_in[0].max(c_in[1]).max(c_in[2]);
    let mut c = c_in;
    if n < 0.0 {
        for v in c.iter_mut() {
            *v = l + (*v - l) * (l / (l - n));
        }
    }
    if x > 1.0 {
        for v in c.iter_mut() {
            *v = l + (*v - l) * ((1.0 - l) / (x - l));
        }
    }
    c
}

fn set_lum(c: [f64; 3], l: f64) -> [f64; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn sat(c: [f64; 3]) -> f64 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn set_sat(c: [f64; 3], s: f64) -> [f64; 3] {
    let cmax = c[0].max(c[1]).max(c[2]);
    let cmin = c[0].min(c[1]).min(c[2]);
    if cmax > cmin {
        let scale = s / (cmax - cmin);
        [(c[0] - cmin) * scale, (c[1] - cmin) * scale, (c[2] - cmin) * scale]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn nonseparable_blend(mode: BlendMode, cb: [f64; 3], cs: [f64; 3]) -> [f64; 3] {
    match mode {
        BlendMode::Hue => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        BlendMode::Saturation => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        BlendMode::Color => set_lum(cs, lum(cb)),
        BlendMode::Luminosity => set_lum(cb, lum(cs)),
        other => panic!("{other:?} は非分離 blend ではない(このテストの対象外)"),
    }
}

fn place_solid(doc: &mut Document, layer: LayerId, rgb: [u8; 3], order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [rgb[0], rgb[1], rgb[2], 255],
                width: W,
                height: H,
            },
            order,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

/// base(backdrop、先に描く)/top(source、blend_mode を持つ)。彩度のある異なる色
/// (Hue/Saturation が no-op にならないよう、色相・彩度とも base と top で変える)。
const BASE_RGB: [u8; 3] = [200, 60, 60]; // 赤寄り、高彩度
const TOP_RGB: [u8; 3] = [50, 90, 200]; // 青寄り、高彩度

const MODES: &[BlendMode] = &[
    BlendMode::Hue,
    BlendMode::Saturation,
    BlendMode::Color,
    BlendMode::Luminosity,
];

/// **ORACLE**: 4モード全部、単色×単色の手計算(独立実装、W3C 3.7節)と GPU 出力が
/// 一致する。
#[test]
fn all_four_nonseparable_modes_match_independent_w3c_reference_calc() {
    let cb = [
        srgb_to_linear(BASE_RGB[0] as f64 / 255.0),
        srgb_to_linear(BASE_RGB[1] as f64 / 255.0),
        srgb_to_linear(BASE_RGB[2] as f64 / 255.0),
    ];
    let cs = [
        srgb_to_linear(TOP_RGB[0] as f64 / 255.0),
        srgb_to_linear(TOP_RGB[1] as f64 / 255.0),
        srgb_to_linear(TOP_RGB[2] as f64 / 255.0),
    ];

    let mut failures = Vec::new();

    for &mode in MODES {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: W,
            height: H,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();

        let (base, top) = (LayerId(1), LayerId(2));
        place_solid(&mut doc, base, BASE_RGB, 0);
        place_solid(&mut doc, top, TOP_RGB, 1);
        doc.apply(Intent::SetAttrs {
            layer: top,
            patch: LayerAttrsPatch {
                blend_mode: Some(mode),
                ..Default::default()
            },
        })
        .unwrap();

        let mut engine = Engine::new().expect("engine");
        let frame = engine
            .render_frame(&doc.view(), t(0))
            .unwrap_or_else(|e| panic!("{mode:?} が engine に拒まれた: {e}"));
        let actual = pixel(&frame, W / 2, H / 2);

        let expected_linear = nonseparable_blend(mode, cb, cs);
        let expected: Vec<i32> = expected_linear
            .iter()
            .map(|&v| (linear_to_srgb(v) * 255.0).round() as i32)
            .collect();

        for (ch, &got_u8) in actual.iter().take(3).enumerate() {
            let got = got_u8 as i32;
            if (got - expected[ch]).abs() > TOLERANCE {
                failures.push(format!(
                    "{mode:?}: channel {ch} got={got} expected≈{} actual_pixel={actual:?}",
                    expected[ch]
                ));
            }
        }
        if actual[3] != 255 {
            failures.push(format!(
                "{mode:?}: opaque base+top のはずが alpha={} (不透明でない)",
                actual[3]
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "非分離 blend の数値検証で不一致:\n{}",
        failures.join("\n")
    );
}

/// Hue/Saturation/Color/Luminosity は Normal と違う絵を出す(no-op になっていない
/// ことの健全性確認——数値の正しさは上のテストが個別に縛る)。
#[test]
fn nonseparable_modes_change_the_output_from_normal() {
    for &mode in MODES {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: W,
            height: H,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();

        let (base, top) = (LayerId(1), LayerId(2));
        place_solid(&mut doc, base, BASE_RGB, 0);
        place_solid(&mut doc, top, TOP_RGB, 1);

        let mut engine = Engine::new().expect("engine");
        let normal = engine.render_frame(&doc.view(), t(0)).unwrap();

        doc.apply(Intent::SetAttrs {
            layer: top,
            patch: LayerAttrsPatch {
                blend_mode: Some(mode),
                ..Default::default()
            },
        })
        .unwrap();
        let blended = engine
            .render_frame(&doc.view(), t(0))
            .unwrap_or_else(|e| panic!("{mode:?} が engine に拒まれた: {e}"));

        assert_ne!(
            pixel(&normal, W / 2, H / 2),
            pixel(&blended, W / 2, H / 2),
            "{mode:?} が Normal と同じ絵になっている(no-op 疑い)"
        );
    }
}
