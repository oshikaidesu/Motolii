//! D1h: DocParam期待型・空トラック・AssetRef・NaN/Inf/値域の validate。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    AssetId, Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document,
    DocumentError, EffectId, EffectInstance, ItemEnvelope, KeyframeId, Track, TrackItem,
};
use motolii_eval::Interp;
use std::collections::BTreeMap;

fn valid_minimal() -> Document {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("a").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc
}

fn clip_mut(doc: &mut Document) -> &mut Clip {
    match &mut doc.tracks[0].items[0] {
        TrackItem::Clip(c) => c,
        _ => unreachable!(),
    }
}

#[test]
fn transform_position_const_color_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.transform.position = DocParam::const_color([1.0, 0.0, 0.0, 1.0]);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParamTypeMismatch { .. })
    ));
}

#[test]
fn empty_keyframes_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.transform.rotation = DocParam::Keyframes(DocKeyframeTrack::new());
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::EmptyKeyframeTrack { .. })
    ));
}

#[test]
fn keyframes_mixed_variants_fail() {
    let mut doc = valid_minimal();
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(0),
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::from_seconds(1),
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
    });
    clip_mut(&mut doc).envelope.transform.rotation = DocParam::Keyframes(track);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::KeyframeVariantMismatch { .. })
    ));
}

#[test]
fn data_fallback_wrong_type_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.opacity = DocParam::Data {
        track: "x".into(),
        fallback: DocValue::Vec2([0.0, 0.0]),
    };
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParamTypeMismatch { .. })
    ));
}

#[test]
fn opacity_nan_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.opacity = DocParam::Const(DocValue::F64(f64::NAN));
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::NonFiniteValue { .. })
    ));
}

#[test]
fn opacity_out_of_range_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.opacity = DocParam::const_f64(1.5);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ValueOutOfRange { .. })
    ));
}

#[test]
fn color_out_of_range_fails() {
    let mut doc = valid_minimal();
    let mut params = BTreeMap::new();
    params.insert("color".into(), DocParam::const_color([2.0, 0.0, 0.0, 1.0]));
    clip_mut(&mut doc).envelope.effects.push(EffectInstance {
        id: EffectId::from_raw(0),
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params,
        extra: Default::default(),
    });
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ValueOutOfRange { .. })
    ));
}

#[test]
fn dangling_asset_ref_fails() {
    let mut doc = valid_minimal();
    let mut params = BTreeMap::new();
    params.insert(
        "tex".into(),
        DocParam::Const(DocValue::AssetRef(AssetId::from_raw(999))),
    );
    clip_mut(&mut doc).source = ClipSource::Plugin {
        plugin_id: "vendor.future.plugin".into(),
        effect_version: 1,
        params,
        extra: Default::default(),
    };
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnknownAssetId { id: 999 })
    ));
}

#[test]
fn asset_ref_resolves_when_registered() {
    let mut doc = valid_minimal();
    let id = doc.assets.allocate("tex", "image/png", "h").unwrap();
    let mut params = BTreeMap::new();
    params.insert("tex".into(), DocParam::Const(DocValue::AssetRef(id)));
    clip_mut(&mut doc).source = ClipSource::Plugin {
        plugin_id: "vendor.future.plugin".into(),
        effect_version: 1,
        params,
        extra: Default::default(),
    };
    assert!(doc.validate().is_ok());
}

#[test]
fn look_at_on_rotation_ok() {
    let mut doc = valid_minimal();
    let asset = match &doc.tracks[0].items[0] {
        TrackItem::Clip(c) => match c.source {
            ClipSource::Asset { asset, .. } => asset,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let target = doc.layers.allocate("t").unwrap();
    let tid2 = doc.track_ids.allocate("V2").unwrap();
    doc.tracks.push(Track {
        id: tid2,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(target),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    clip_mut(&mut doc).envelope.transform.rotation = DocParam::LookAt {
        target,
        axis: motolii_doc::LookAtAxis::PlusY,
    };
    assert!(doc.validate().is_ok());
}

#[test]
fn look_at_on_position_fails() {
    let mut doc = valid_minimal();
    let target = doc.layers.allocate("t").unwrap();
    clip_mut(&mut doc).envelope.transform.position = DocParam::LookAt {
        target,
        axis: motolii_doc::LookAtAxis::PlusY,
    };
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::SpatialLinkNotAllowed { .. })
    ));
}

#[test]
fn vec2_axes_with_color_axis_fails() {
    let mut doc = valid_minimal();
    clip_mut(&mut doc).envelope.transform.position = DocParam::Vec2Axes {
        x: Box::new(DocParam::const_color([1.0, 0.0, 0.0, 1.0])),
        y: Box::new(DocParam::const_f64(0.0)),
    };
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParamTypeMismatch { .. })
    ));
}

#[test]
fn bezier_nan_y_fails() {
    let mut doc = valid_minimal();
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(0),
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Bezier {
            x1: 0.42,
            y1: f64::NAN,
            x2: 0.58,
            y2: 1.0,
        },
    });
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::from_seconds(1),
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    clip_mut(&mut doc).envelope.transform.rotation = DocParam::Keyframes(track);
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::NonFiniteBezier { .. })
    ));
}

#[test]
fn layer_source_clear_rejects_wrong_color_type() {
    let mut doc = valid_minimal();
    let mut params = BTreeMap::new();
    params.insert("color".into(), DocParam::const_vec2([0.0, 0.0]));
    clip_mut(&mut doc).source = ClipSource::Plugin {
        plugin_id: "core.layer_source.clear".into(),
        effect_version: 1,
        params,
        extra: Default::default(),
    };
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ParamTypeMismatch { .. })
    ));
}
