use motolii_edit::{Animate, Document, Intent};
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use super::*;
use super::bounds::group_local_bounds;

/// Display P3 の ICC が埋まった PNG は、Finder / Preview と同じく
/// profile を適用して読む(適用しないと (224,64,32) が (206,76,46) 程に沈む)。
#[test]
fn still_with_embedded_icc_is_mapped_to_srgb() {
    use image::ImageEncoder;
    let srgb = [224u8, 64, 32, 255];
    let p3 = moxcms::ColorProfile::new_display_p3();
    let to_p3 = moxcms::ColorProfile::new_srgb()
        .create_transform_8bit(
            moxcms::Layout::Rgba,
            &p3,
            moxcms::Layout::Rgba,
            moxcms::TransformOptions::default(),
        )
        .unwrap();
    let mut in_p3 = [0u8; 4];
    to_p3.transform(&srgb, &mut in_p3).unwrap();
    assert_ne!(in_p3[..3], srgb[..3], "P3 の数字は sRGB と違うはず");

    let dir = std::env::temp_dir().join(format!("motolii-icc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p3.png");
    let mut encoder =
        image::codecs::png::PngEncoder::new(std::fs::File::create(&path).unwrap());
    encoder.set_icc_profile(p3.encode().unwrap()).unwrap();
    encoder
        .write_image(&in_p3.repeat(4), 2, 2, image::ExtendedColorType::Rgba8)
        .unwrap();

    let (rgba, w, h) = decode_still_srgb(path.to_str().unwrap()).unwrap();
    assert_eq!((w, h), (2, 2));
    for c in 0..3 {
        assert!(
            (rgba[c] as i32 - srgb[c] as i32).abs() <= 2,
            "channel {c}: got {} want {}",
            rgba[c],
            srgb[c]
        );
    }
    assert_eq!(rgba[3], 255);
    std::fs::remove_dir_all(&dir).ok();
}


#[test]
fn group_bounds_use_offset_descendants_in_group_space_and_empty_groups_have_no_box() {
    use crate::doc::store::{LayerMeta, LayerTiming, LayerAttrsPatch, PropertyId, Value};
    let mut doc = motolii_edit::blank_project();
    for (id, source, parent, position) in [
        (1, LayerSource::Group, None, [100.0, 200.0]),
        (2, LayerSource::Shape, Some(1), [10.0, 20.0]),
        (3, LayerSource::Group, Some(1), [30.0, 40.0]),
        (4, LayerSource::Shape, Some(3), [5.0, -10.0]),
        (5, LayerSource::Shape, None, [-100.0, -200.0]),
        (6, LayerSource::Group, None, [0.0, 0.0]),
    ] {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta {
                source, order: id as i16, timing: LayerTiming::place(0, None, 30),
            } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch {
                parent: Some(parent.map(LayerId)), ..Default::default()
            } },
            Intent::SetConstant { layer, property: PropertyId::new("position").unwrap(), value: Value::Vec2(position) },
        ]).unwrap();
    }
    let view = doc.view();
    let mut resolved = crate::picture::resolve::resolved_layers(&view, RationalTime::ZERO).unwrap();
    let leaf = |layer: &ResolvedLayer| Some(if layer.id == LayerId(4) {
        crate::render::media::SpatialBounds { min: [1.0, 2.0, -1.0], max: [3.0, 4.0, 1.0] }
    } else {
        crate::render::media::SpatialBounds { min: [0.0; 3], max: [4.0, 6.0, 0.0] }
    });
    assert_eq!(group_local_bounds(&view, &resolved, LayerId(1), leaf), Some(crate::render::media::SpatialBounds {
        min: [10.0, 20.0, -1.0], max: [38.0, 34.0, 1.0],
    }));
    assert_eq!(group_local_bounds(&view, &resolved, LayerId(3), leaf), Some(crate::render::media::SpatialBounds {
        min: [6.0, -8.0, -1.0], max: [8.0, -6.0, 1.0],
    }));
    assert!(group_local_bounds(&view, &resolved, LayerId(6), leaf).is_none());
    resolved.iter_mut().find(|layer| layer.id == LayerId(1)).unwrap().placement.world_transform = Some(glam::Affine3A::from_scale(glam::vec3(0.0, 1.0, 1.0)));
    assert!(group_local_bounds(&view, &resolved, LayerId(1), leaf).is_none());
}
