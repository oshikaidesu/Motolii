//! BL3(分離可能 blend 11 モード)の数値検証(ORACLE「モードごとの数値検証」)。
//!
//! 単色 layer 2枚(base=backdrop・top=source、両方**不透明**)を `Engine::render_frame`
//! で描き、center pixel を「このテストが独立に書いた W3C Compositing 3.6節の式」の
//! 計算値と突き合わせる。両方 opaque(αs=αb=1)なので一般合成式
//! (`motolii_compositor::blend` モジュール doc)は `Co = B(Cb, Cs)` へ潰れ、
//! per-channel blend 関数だけを比較すれば足りる——WGSL 側(`blend.rs` の
//! `blend_channel`)とは**別実装**(Rust で式を書き直す)なので、shader のタイポや
//! 符号ミスを検出できる独立照合になる。
//!
//! gamma: 素材は sRGB 8bit(`upload_rgba`)、blend 演算は linear 空間
//! (`motolii-compositor` module doc「色空間の注意」)。このテストも同じ
//! sRGB⇄linear 変換式(IEC 61966-2-1)を使い、8bit 量子化+GPU ハードウェアの
//! sRGB 変換近似の誤差ぶんだけ許容差(`TOLERANCE`)を持たせる。

use motolii_engine::Engine;
use motolii_store::{
    BlendMode, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, RationalTime,
};

const W: u32 = 8;
const H: u32 = 8;
/// 8bit 量子化 + GPU sRGB ハードウェア変換近似の誤差を吸収する許容差(LSB)。
/// `finalize_readback` が screenshot 経路で追加のガンマ round-trip を1回踏む分も
/// ここに含む(`motolii-compositor` module doc「色空間の注意」参照)。
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

/// W3C Compositing and Blending Level 1、3.6節の separable blend 関数。
/// `motolii-compositor` の WGSL(`blend.rs` の `blend_channel`)とは独立実装。
fn blend_channel(mode: BlendMode, cb: f64, cs: f64) -> f64 {
    match mode {
        BlendMode::Multiply => cs * cb,
        BlendMode::Screen => cs + cb - cs * cb,
        BlendMode::Overlay => {
            if cb <= 0.5 {
                2.0 * cs * cb
            } else {
                1.0 - 2.0 * (1.0 - cs) * (1.0 - cb)
            }
        }
        BlendMode::Darken => cb.min(cs),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::ColorDodge => {
            if cb <= 0.0 {
                0.0
            } else if cs >= 1.0 {
                1.0
            } else {
                (cb / (1.0 - cs)).min(1.0)
            }
        }
        BlendMode::ColorBurn => {
            if cb >= 1.0 {
                1.0
            } else if cs <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - cb) / cs).min(1.0)
            }
        }
        BlendMode::HardLight => {
            if cs <= 0.5 {
                2.0 * cs * cb
            } else {
                1.0 - 2.0 * (1.0 - cs) * (1.0 - cb)
            }
        }
        BlendMode::SoftLight => {
            if cs <= 0.5 {
                cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
            } else {
                let d = if cb <= 0.25 {
                    ((16.0 * cb - 12.0) * cb + 4.0) * cb
                } else {
                    cb.sqrt()
                };
                cb + (2.0 * cs - 1.0) * (d - cb)
            }
        }
        BlendMode::Difference => (cb - cs).abs(),
        BlendMode::Exclusion => cs + cb - 2.0 * cs * cb,
        other => panic!("{other:?} は分離可能 blend ではない(このテストの対象外)"),
    }
}

fn place_solid_gray(doc: &mut Document, layer: LayerId, gray: u8, order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [gray, gray, gray, 255],
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

/// base(backdrop、先に描く)/top(source、blend_mode を持つ)の固定 gray 値。
/// 0.6/0.3 相当(straight, 8bit)——中庸値で、ColorDodge/ColorBurn の一方は
/// 飽和(0 or 1)する境界も含めて検証する(境界も正しい clamp を確認する対象)。
const BASE_GRAY: u8 = 153; // ≈ 0.6 * 255
const TOP_GRAY: u8 = 77; // ≈ 0.3 * 255

const MODES: &[BlendMode] = &[
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Overlay,
    BlendMode::Darken,
    BlendMode::Lighten,
    BlendMode::ColorDodge,
    BlendMode::ColorBurn,
    BlendMode::HardLight,
    BlendMode::SoftLight,
    BlendMode::Difference,
    BlendMode::Exclusion,
];

/// **ORACLE**: 11モード全部、単色×単色の手計算(独立実装)と GPU 出力が一致する。
#[test]
fn all_eleven_separable_modes_match_independent_w3c_reference_calc() {
    let cb = srgb_to_linear(BASE_GRAY as f64 / 255.0);
    let cs = srgb_to_linear(TOP_GRAY as f64 / 255.0);

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
        place_solid_gray(&mut doc, base, BASE_GRAY, 0);
        place_solid_gray(&mut doc, top, TOP_GRAY, 1);
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

        let expected_linear = blend_channel(mode, cb, cs);
        let expected = (linear_to_srgb(expected_linear) * 255.0).round() as i32;

        for (ch, &got_u8) in actual.iter().take(3).enumerate() {
            let got = got_u8 as i32;
            if (got - expected).abs() > TOLERANCE {
                failures.push(format!(
                    "{mode:?}: channel {ch} got={got} expected≈{expected}(gray) actual_pixel={actual:?}"
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
        "分離可能 blend の数値検証で不一致:\n{}",
        failures.join("\n")
    );
}

/// 2 mode 重ね(base=Normal, mid=Multiply, top=Screen)の**決定論**——同じ Document を
/// 2回描いて同じフレームが返ることを縛る(ORACLE「2 mode 重ねの決定論」)。
/// 数値の正しさは上のテストが個別に縛るので、ここは「同じ入力→同じ出力」だけを見る。
#[test]
fn two_stacked_separable_modes_are_deterministic() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    let (base, mid, top) = (LayerId(1), LayerId(2), LayerId(3));
    place_solid_gray(&mut doc, base, 220, 0);
    place_solid_gray(&mut doc, mid, 90, 1);
    place_solid_gray(&mut doc, top, 40, 2);

    doc.apply(Intent::SetAttrs {
        layer: mid,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Multiply),
            ..Default::default()
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Screen),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let first = engine.render_frame(&doc.view(), t(0)).unwrap();
    let second = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_eq!(
        first, second,
        "同じ2mode重ねの Document が毎回同じ結果にならない"
    );

    // 白(220)への Multiply→Screen は完全な no-op にはならない(ここが変化していない
    // となると accumulator が固まって何も描いていない可能性がある、という健全性確認)。
    let background_only = {
        let mut only_base = Document::new();
        only_base
            .apply(Intent::SetComposition(Composition {
                width: W,
                height: H,
                fps: Fps::try_new(30, 1).unwrap(),
                duration_frames: 60,
                background: [0.0, 0.0, 0.0, 1.0],
            }))
            .unwrap();
        place_solid_gray(&mut only_base, base, 220, 0);
        let mut e = Engine::new().expect("engine");
        e.render_frame(&only_base.view(), t(0)).unwrap()
    };
    assert_ne!(
        pixel(&first, W / 2, H / 2),
        pixel(&background_only, W / 2, H / 2),
        "mid/top の blend が base を一切変えていない(accumulator が進んでいない疑い)"
    );
}
