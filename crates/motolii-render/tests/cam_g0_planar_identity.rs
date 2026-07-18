//! CAM-G0: camera接続前の既存`ViewportTransform` GPU経路のmatching-aspect identity fixture。
//! 期待値の正本は `oracles/cam_g0_planar_identity.tsv`。本ファイルは変更可能なharness。

use std::collections::BTreeMap;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_gpu::download_rgba;
use motolii_nodes::{CanonicalPoint, CanonicalSize, RectOverlay};
use motolii_render::{render_frame, RenderFrameRequest, SolidSource};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const ORACLE: &str = include_str!("oracles/cam_g0_planar_identity.tsv");

#[derive(Debug)]
struct OracleFixture {
    desc: FrameDesc,
    quality: Quality,
    timeline_time: RationalTime,
    reports_source_time: bool,
    source: SolidSource,
    overlay: RectOverlay,
    expected: Vec<u8>,
}

fn parse_f64(meta: &BTreeMap<&str, &str>, key: &str) -> f64 {
    meta.get(key)
        .unwrap_or_else(|| panic!("oracle meta missing {key}"))
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("oracle meta {key} must be f64"))
}

fn parse_u32(meta: &BTreeMap<&str, &str>, key: &str) -> u32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("oracle meta missing {key}"))
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("oracle meta {key} must be u32"))
}

fn parse_bool(meta: &BTreeMap<&str, &str>, key: &str) -> bool {
    match meta
        .get(key)
        .unwrap_or_else(|| panic!("oracle meta missing {key}"))
        .as_ref()
    {
        "true" => true,
        "false" => false,
        other => panic!("oracle meta {key} must be true/false, got {other}"),
    }
}

fn rgba8(value: &str) -> [u8; 4] {
    let values = value
        .split(',')
        .map(|component| component.parse::<u8>().expect("oracle rgba8 component"))
        .collect::<Vec<_>>();
    values
        .try_into()
        .expect("oracle rgba8 must have 4 components")
}

fn load_oracle() -> OracleFixture {
    let mut meta = BTreeMap::new();
    let mut rgba_rows = Vec::new();
    for line in ORACLE.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["meta", key, value] => {
                meta.insert(*key, *value);
            }
            ["rgba", y, x, rgba] => rgba_rows.push((
                y.parse::<u32>().expect("oracle rgba y"),
                x.parse::<u32>().expect("oracle rgba x"),
                rgba8(rgba),
            )),
            _ => panic!("malformed CAM-G0 semantic oracle line: {line}"),
        }
    }
    assert!(!meta.is_empty(), "CAM-G0 oracle meta must not be empty");
    assert!(
        !rgba_rows.is_empty(),
        "CAM-G0 oracle rgba must not be empty"
    );

    let width = parse_u32(&meta, "width");
    let height = parse_u32(&meta, "height");
    let aspect_num = parse_u32(&meta, "aspect_num");
    let aspect_den = parse_u32(&meta, "aspect_den");
    assert_eq!(
        (width, height),
        (aspect_num, aspect_den),
        "matching aspect must equal output desc rational aspect"
    );
    assert_eq!(
        meta.get("pixel_format").copied(),
        Some("Rgba8Unorm"),
        "oracle pixel_format"
    );
    assert_eq!(
        meta.get("color_space").copied(),
        Some("Srgb"),
        "oracle color_space"
    );
    assert!(parse_bool(&meta, "premultiplied"), "oracle premultiplied");
    assert_eq!(
        meta.get("quality").copied(),
        Some("FINAL"),
        "oracle quality"
    );

    let timeline_time = RationalTime::try_new(
        parse_u32(&meta, "timeline_time_num") as i64,
        parse_u32(&meta, "timeline_time_den") as i64,
    )
    .expect("oracle timeline_time");
    let reports_source_time = parse_bool(&meta, "reports_source_time");
    assert_eq!(
        meta.get("time_map").copied(),
        Some("identity"),
        "oracle time_map"
    );

    let desc = FrameDesc::packed(
        width,
        height,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );
    let source = SolidSource {
        color: [
            parse_f64(&meta, "bg_r") as f32,
            parse_f64(&meta, "bg_g") as f32,
            parse_f64(&meta, "bg_b") as f32,
            parse_f64(&meta, "bg_a") as f32,
        ],
        time_map: TimeMap::identity(),
        reports_source_time,
    };
    let overlay = RectOverlay {
        center: CanonicalPoint {
            x: parse_f64(&meta, "center_x"),
            y: parse_f64(&meta, "center_y"),
        },
        size: CanonicalSize {
            width: parse_f64(&meta, "size_w"),
            height: parse_f64(&meta, "size_h"),
        },
        color: [
            parse_f64(&meta, "fg_r") as f32,
            parse_f64(&meta, "fg_g") as f32,
            parse_f64(&meta, "fg_b") as f32,
            parse_f64(&meta, "fg_a") as f32,
        ],
    };

    rgba_rows.sort_by_key(|(y, x, _)| (*y, *x));
    let pixel_count = (width * height) as usize;
    assert_eq!(
        rgba_rows.len(),
        pixel_count,
        "oracle rgba row count must match width*height"
    );
    let mut expected = vec![0u8; pixel_count * 4];
    for (idx, (y, x, rgba)) in rgba_rows.into_iter().enumerate() {
        assert_eq!((y, x), (idx as u32 / width, idx as u32 % width));
        let i = idx * 4;
        expected[i..i + 4].copy_from_slice(&rgba);
    }

    OracleFixture {
        desc,
        quality: Quality::FINAL,
        timeline_time,
        reports_source_time,
        source,
        overlay,
        expected,
    }
}

#[test]
fn cam_g0_planar_identity_matches_semantic_oracle() {
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let fixture = load_oracle();
    let request = RenderFrameRequest {
        desc: fixture.desc,
        timeline_time: fixture.timeline_time,
        source: fixture.source,
        overlay: fixture.overlay,
    };

    let rendered = render_frame(&gpu, &request, fixture.quality).unwrap();
    assert_eq!(rendered.desc, fixture.desc);
    if fixture.reports_source_time {
        assert_eq!(rendered.source_time, fixture.timeline_time);
    }

    let actual = download_rgba(&gpu, &rendered.texture).unwrap();
    assert_rgba_close(
        "cam-g0-planar-identity",
        RgbaImageDesc {
            width: fixture.desc.width,
            height: fixture.desc.height,
        },
        &actual,
        &fixture.expected,
        tol::EXACT,
    );
}
