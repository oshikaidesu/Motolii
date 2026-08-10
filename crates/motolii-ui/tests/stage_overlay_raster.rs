use std::collections::BTreeMap;

use motolii_core::{PixelSize, RationalTime};
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, EvaluationTime, Group, ItemEnvelope, LayerId, Track,
    TrackItem, Transform2D, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_ui::{project_stage_geometry, stage_overlay_raster, StageLayerProjection};

fn rect(center: [f64; 2], size: [f64; 2]) -> ClipSource {
    ClipSource::Plugin {
        plugin_id: RECT_LAYER_SOURCE.into(),
        effect_version: 1,
        params: BTreeMap::from([
            ("center".into(), DocParam::const_vec2(center)),
            ("size".into(), DocParam::const_vec2(size)),
            ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
        ]),
        extra: Default::default(),
    }
}

fn fixture() -> (Document, LayerId, LayerId, LayerId) {
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("V1").unwrap();
    document.tracks.push(Track {
        id: track,
        items: Vec::new(),
    });
    let selected = document.layers.allocate("selected").unwrap();
    let other = document.layers.allocate("other").unwrap();
    let unavailable = document.layers.allocate("unavailable").unwrap();
    let mut other_envelope = ItemEnvelope::new(other);
    other_envelope.transform = Transform2D {
        position: DocParam::const_vec2([0.6, 0.0]),
        ..Transform2D::identity()
    };
    document.tracks[0].items = vec![
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(selected),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: Default::default(),
            source: rect([0.0, 0.0], [0.4, 0.4]),
        }),
        TrackItem::Clip(Clip {
            envelope: other_envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: Default::default(),
            source: rect([0.0, 0.0], [0.2, 0.2]),
        }),
        TrackItem::Group(Group {
            envelope: ItemEnvelope::new(unavailable),
            children: Vec::new(),
        }),
    ];
    document.validate().unwrap();
    (document, selected, other, unavailable)
}

fn raster(selected: Option<LayerId>, scale: f64) -> stage_overlay_raster::StageOverlayRaster {
    let (document, selected_layer, _, _) = fixture();
    let projection = project_stage_geometry(
        &document,
        EvaluationTime::new(RationalTime::ZERO),
        &DataTracks::new(),
    )
    .unwrap();
    stage_overlay_raster::raster_selection_outline(
        &projection,
        selected.or(Some(selected_layer)),
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        scale,
    )
}

fn alpha(raster: &stage_overlay_raster::StageOverlayRaster, x: u32, y: u32) -> u8 {
    raster.pixels[((y * raster.width + x) * 4 + 3) as usize]
}

fn min_alpha_x(raster: &stage_overlay_raster::StageOverlayRaster) -> u32 {
    (0..raster.width)
        .find(|x| (0..raster.height).any(|y| alpha(raster, *x, y) > 0))
        .unwrap()
}

#[test]
fn selected_outline_is_stroked_only_on_the_selected_rect() {
    let output = raster(None, 1.0);
    assert_eq!(output.width, 200);
    assert!(alpha(&output, 100, 60) > 0);
    assert_eq!(alpha(&output, 100, 100), 0);
    assert_eq!(alpha(&output, 160, 100), 0);
}

#[test]
fn world_translation_moves_the_outline() {
    let (document, selected, _, _) = fixture();
    let base_projection = project_stage_geometry(
        &document,
        EvaluationTime::new(RationalTime::ZERO),
        &DataTracks::new(),
    )
    .unwrap();
    let base = stage_overlay_raster::raster_selection_outline(
        &base_projection,
        Some(selected),
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        1.0,
    );

    let (mut shifted_document, shifted, _, _) = fixture();
    let TrackItem::Clip(clip) = &mut shifted_document.tracks[0].items[0] else {
        panic!("expected selected clip");
    };
    clip.envelope.transform.position = DocParam::const_vec2([0.2, 0.0]);
    let shifted_projection = project_stage_geometry(
        &shifted_document,
        EvaluationTime::new(RationalTime::ZERO),
        &DataTracks::new(),
    )
    .unwrap();
    let shifted_output = stage_overlay_raster::raster_selection_outline(
        &shifted_projection,
        Some(shifted),
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        1.0,
    );
    assert!(min_alpha_x(&shifted_output) > min_alpha_x(&base));
}

#[test]
fn selection_none_is_transparent_and_zero_output_is_safe() {
    let (document, _, _, _) = fixture();
    let projection = project_stage_geometry(
        &document,
        EvaluationTime::new(RationalTime::ZERO),
        &DataTracks::new(),
    )
    .unwrap();
    let output = stage_overlay_raster::raster_selection_outline(
        &projection,
        None,
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        1.0,
    );
    assert!(output.pixels.iter().all(|byte| *byte == 0));
    let zero = stage_overlay_raster::raster_selection_outline(
        &projection,
        None,
        PixelSize {
            width: 0.0,
            height: 1.0,
        },
        1.0,
    );
    assert_eq!((zero.width, zero.height, zero.pixels.len()), (0, 0, 0));
}

#[test]
fn scale_factor_doubles_dimensions_and_outline_physical_width() {
    let one = raster(None, 1.0);
    let two = raster(None, 2.0);
    assert_eq!((two.width, two.height), (400, 400));
    let one_thickness = (0..one.height).filter(|y| alpha(&one, 100, *y) > 0).count();
    let two_thickness = (0..two.height).filter(|y| alpha(&two, 200, *y) > 0).count();
    assert!(two_thickness > one_thickness);
}

#[test]
fn unavailable_and_unknown_selection_are_transparent() {
    let (document, _, _, unavailable) = fixture();
    let projection = project_stage_geometry(
        &document,
        EvaluationTime::new(RationalTime::ZERO),
        &DataTracks::new(),
    )
    .unwrap();
    let output = stage_overlay_raster::raster_selection_outline(
        &projection,
        Some(unavailable),
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        1.0,
    );
    assert!(output.pixels.iter().all(|byte| *byte == 0));
    assert!(projection
        .layers()
        .iter()
        .any(|(layer, projection)| *layer == unavailable
            && matches!(projection, StageLayerProjection::Unavailable(_))));

    let unknown = LayerId::from_raw(999_999);
    let output = stage_overlay_raster::raster_selection_outline(
        &projection,
        Some(unknown),
        PixelSize {
            width: 200.0,
            height: 200.0,
        },
        1.0,
    );
    assert!(output.pixels.iter().all(|byte| *byte == 0));
}
