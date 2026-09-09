//! Blend の札の絵。W3C Compositing and Blending の式を **CPU で 1 画素**解く。
//! 空の四角(C4)を、実際にその式で混ざった色にする。GPU を回すほどの物ではない。

use crate::doc::store::BlendMode;

/// 下地(bottom)の上に top をその mode で置いた色。線形光(0..1)。
pub(crate) fn blend(mode: BlendMode, top: [f32; 3], bottom: [f32; 3]) -> [f32; 3] {
    use BlendMode::*;
    let sep = |f: fn(f32, f32) -> f32| [f(top[0], bottom[0]), f(top[1], bottom[1]), f(top[2], bottom[2])];
    match mode {
        Normal => top,
        Add => sep(|s, b| (s + b).min(1.0)),
        Multiply => sep(|s, b| s * b),
        Screen => sep(|s, b| b + s - b * s),
        Overlay => sep(|s, b| hard_light(b, s)),
        Darken => sep(|s, b| s.min(b)),
        Lighten => sep(|s, b| s.max(b)),
        ColorDodge => sep(|s, b| {
            if b == 0.0 {
                0.0
            } else if s >= 1.0 {
                1.0
            } else {
                (b / (1.0 - s)).min(1.0)
            }
        }),
        ColorBurn => sep(|s, b| {
            if b >= 1.0 {
                1.0
            } else if s <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - b) / s).min(1.0)
            }
        }),
        HardLight => sep(hard_light),
        SoftLight => sep(|s, b| {
            if s <= 0.5 {
                b - (1.0 - 2.0 * s) * b * (1.0 - b)
            } else {
                let d = if b <= 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() };
                b + (2.0 * s - 1.0) * (d - b)
            }
        }),
        Difference => sep(|s, b| (b - s).abs()),
        Exclusion => sep(|s, b| b + s - 2.0 * b * s),
        Hue => set_lum(set_sat(top, sat(bottom)), lum(bottom)),
        Saturation => set_lum(set_sat(bottom, sat(top)), lum(bottom)),
        Color => set_lum(top, lum(bottom)),
        Luminosity => set_lum(bottom, lum(top)),
    }
}

fn hard_light(s: f32, b: f32) -> f32 {
    if s <= 0.5 {
        b * 2.0 * s
    } else {
        let s2 = 2.0 * s - 1.0;
        b + s2 - b * s2
    }
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut out = c;
    if n < 0.0 {
        for v in &mut out {
            *v = l + (*v - l) * l / (l - n).max(1e-6);
        }
    }
    if x > 1.0 {
        for v in &mut out {
            *v = l + (*v - l) * (1.0 - l) / (x - l).max(1e-6);
        }
    }
    out
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn sat(c: [f32; 3]) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let max = c[0].max(c[1]).max(c[2]);
    let min = c[0].min(c[1]).min(c[2]);
    let mut out = [0.0; 3];
    if max > min {
        for i in 0..3 {
            out[i] = (c[i] - min) * s / (max - min);
        }
    }
    out
}

/// 札に並べる下地。暗・明・青・暖の 4 つで、mode の性格が一目で分かる。
pub(crate) const BEDS: [[f32; 3]; 4] = [[0.18, 0.18, 0.18], [0.85, 0.85, 0.85], [0.25, 0.45, 0.85], [0.9, 0.55, 0.3]];

pub(crate) fn css(c: [f32; 3]) -> String {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c[0]), b(c[1]), b(c[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_formulas_behave_like_the_spec_says() {
        let top = [0.5, 0.5, 0.5];
        assert_eq!(blend(BlendMode::Multiply, top, [0.5, 0.5, 0.5]), [0.25, 0.25, 0.25]);
        assert_eq!(blend(BlendMode::Screen, top, [0.5, 0.5, 0.5]), [0.75, 0.75, 0.75]);
        assert_eq!(blend(BlendMode::Darken, [0.2, 0.9, 0.5], [0.6, 0.1, 0.5]), [0.2, 0.1, 0.5]);
        assert_eq!(blend(BlendMode::Difference, [1.0, 0.0, 0.5], [0.0, 1.0, 0.5]), [1.0, 1.0, 0.0]);
        let lum_kept = blend(BlendMode::Color, [1.0, 0.0, 0.0], [0.5, 0.5, 0.5]);
        assert!((lum(lum_kept) - 0.5).abs() < 1e-4, "{lum_kept:?}");
        assert_eq!(css([1.0, 0.5, 0.0]), "#ff8000");
    }
}

/// The bed is a black→light ramp tinted cool at the top and warm at the bottom, so the
/// separable, contrast and component (Hue/Saturation/Color/Luminosity) families all read
/// differently; the disc on top sweeps cyan→red.
fn specimen_colors(x:u32,y:u32) -> ([f32;3],[f32;3],bool) {
    let u=x as f32/127.0;let v=y as f32/79.0;
    let tint=[0.3+0.7*v,0.5+0.1*v,1.0-0.7*v];
    let bottom=[u*(0.6+0.4*tint[0]),u*(0.6+0.4*tint[1]),u*(0.6+0.4*tint[2])];
    let t=((y as f32-10.0)/60.0).clamp(0.0,1.0);
    let top=[0.15+0.8*t,0.75-0.5*t,0.85-0.7*t];
    let front=(x as f32-84.0).powi(2)+(y as f32-40.0).powi(2)<30.0_f32.powi(2);
    (bottom,top,front)
}

/// Fixed blend-behavior specimens: no Document, media decoding, GPU or Stage rendering.
pub(crate) fn specimen(mode: BlendMode) -> Result<serde_json::Value, String> {
    use base64::Engine as _;
    static IMAGES: std::sync::OnceLock<Vec<Result<serde_json::Value,String>>> = std::sync::OnceLock::new();
    IMAGES.get_or_init(|| (0..17).map(|index| {
        let mode=BlendMode::from_enum_value(index).unwrap();
        let image=image::RgbaImage::from_fn(128,80,|x,y| {
            let (bottom,top,front)=specimen_colors(x,y);
            let color=if front{blend(mode,top,bottom)}else{bottom};
            image::Rgba([(color[0].clamp(0.0,1.0)*255.0).round() as u8,(color[1].clamp(0.0,1.0)*255.0).round() as u8,(color[2].clamp(0.0,1.0)*255.0).round() as u8,255])
        });
        let mut bytes=std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image).write_to(&mut bytes,image::ImageFormat::Png).map_err(|e|e.to_string())?;
        Ok(serde_json::json!({"image":base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())}))
    }).collect())[mode.to_enum_value() as usize].clone()
}

#[cfg(test)]
mod specimen_tests {
    use super::*;
    use base64::Engine as _;
    #[test]
    fn behavior_specimens_are_cached_and_use_the_existing_blend_equations() {
        for i in 0..17 {
            let mode=BlendMode::from_enum_value(i).unwrap();
            let first=specimen(mode).unwrap();
            assert_eq!(first,specimen(mode).unwrap());
            let png=base64::engine::general_purpose::STANDARD.decode(first["image"].as_str().unwrap()).unwrap();
            let image=image::load_from_memory(&png).unwrap().to_rgba8();
            assert_eq!(image.dimensions(),(128,80));
            let (bottom,top,front)=specimen_colors(84,40);
            assert!(front);
            let expected=blend(mode,top,bottom);
            assert_eq!(&image.get_pixel(84,40).0[..3], &expected.map(|v|(v.clamp(0.0,1.0)*255.0).round() as u8));
            assert_ne!(image.get_pixel(84,40).0, image.get_pixel(20,40).0, "{mode:?}: the disc must read against the bed");
        }
    }
}
