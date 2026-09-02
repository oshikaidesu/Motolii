mod testkit;

use motolii::doc::store::{
    Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, Mask, MaskId, MaskMode, Path, PathVertex, RationalTime,
    Value,
};
use motolii::render::engine::Engine;

fn rectangle_path(x0: f64, y0: f64, x1: f64, y1: f64) -> Path {
    Path {
        vertices: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .into_iter()
            .map(|point| PathVertex {
                point,
                in_tangent: [0.0, 0.0],
                out_tangent: [0.0, 0.0],
            })
            .collect(),
        closed: true,
    }
}

fn path_track(path: Path) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Path(path),
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn document_with_red_image(width: u32, height: u32) -> (Document, LayerId) {
    let dir = testkit::tmp_dir("mask-coverage");
    let target_path = dir.join("target.png");
    let pixels = [255u8, 0, 0, 255]
        .into_iter()
        .cycle()
        .take((width * height * 4) as usize)
        .collect::<Vec<_>>();
    image::save_buffer(
        &target_path,
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .unwrap();

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width,
        height,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: target_path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                name: Some("masked target".to_owned()),
                ..Default::default()
            },
        },
    ])
    .unwrap();
    (doc, layer)
}

#[test]
fn add_mask_changes_the_product_alpha_inside_the_layer_frame() {
    let width = 8u32;
    let height = 8u32;
    let (mut doc, layer) = document_with_red_image(width, height);
    doc.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        shape: path_track(rectangle_path(0.0, 0.0, 4.0, 8.0)),
    })
    .unwrap();

    let frame = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let inside_alpha = frame[((height / 2 * width + 1) * 4 + 3) as usize];
    let outside_alpha = frame[((height / 2 * width + 6) * 4 + 3) as usize];
    assert!(inside_alpha > 0, "Add Mask removed its inside");
    assert_eq!(
        outside_alpha, 0,
        "Add Mask did not clear the outside alpha: {outside_alpha}"
    );
}

#[test]
fn subtract_mask_cuts_a_hole_from_the_masks_above_it() {
    let width = 8u32;
    let height = 8u32;
    let (mut doc, layer) = document_with_red_image(width, height);
    doc.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        shape: path_track(rectangle_path(1.0, 0.0, 7.0, 8.0)),
    })
    .unwrap();
    doc.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(1),
            mode: MaskMode::Subtract,
            inverted: false,
        },
        shape: path_track(rectangle_path(3.0, 0.0, 5.0, 8.0)),
    })
    .unwrap();

    let frame = Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let alpha = |x: u32| frame[((height / 2 * width + x) * 4 + 3) as usize];
    assert_eq!(alpha(0), 0, "the Add Mask outside stayed opaque");
    assert!(alpha(2) > 0, "the Add Mask body vanished");
    assert_eq!(alpha(4), 0, "the Subtract Mask did not cut its hole");
}

#[test]
fn repeated_masked_frames_keep_the_same_pixels() {
    let (mut doc, layer) = document_with_red_image(8, 8);
    doc.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(0),
            mode: MaskMode::Add,
            inverted: false,
        },
        shape: path_track(rectangle_path(0.0, 0.0, 4.0, 8.0)),
    })
    .unwrap();
    let mut engine = Engine::new().unwrap();
    let first = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let second = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(first, second);
}
