
use crate::doc::core::RationalTime;
use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Value};

use crate::doc::store::view::StoreView;
use crate::doc::store::{LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming, StoreError};

use super::{Document, Intent, LayerId, PropertyId};

impl Document {
    pub fn group_layers(&mut self, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
        if layers.is_empty() {
            return Ok(None);
        }

        let view = self.view();
        let group_id = LayerId(view.next_layer_id());
        let comp_duration = view
            .composition()?
            .map(|composition| composition.duration_frames)
            .unwrap_or(0);

        let mut intents = Vec::with_capacity(layers.len() + 2);
        intents.push(Intent::AddLayer(group_id));
        intents.push(Intent::SetMeta {
            layer: group_id,
            meta: LayerMeta {
                source: LayerSource::Group,
                order: group_id.0 as i16,
                timing: LayerTiming::place(0, None, comp_duration),
            },
        });
        for &child in layers {
            intents.push(Intent::SetAttrs {
                layer: child,
                patch: LayerAttrsPatch {
                    parent: Some(Some(group_id)),
                    ..Default::default()
                },
            });
        }

        self.apply_all(intents)?;
        Ok(Some(group_id))
    }

    pub fn ungroup_layers(&mut self, groups: &[LayerId]) -> Result<Vec<LayerId>, StoreError> {
        if groups.is_empty() {
            return Ok(Vec::new());
        }

        let t = RationalTime::ZERO;
        let view = self.view();
        let present = view.layers();

        let mut intents = Vec::new();
        let mut released = Vec::new();

        for &group in groups {
            let Some(meta) = view.meta(group)? else {
                continue;
            };
            if meta.source != LayerSource::Group {
                continue;
            }
            if view.attrs(group)?.unwrap_or_default().frozen {
                return Err(StoreError::Property(format!(
                    "layer {} は凍結中(frozen)なので ungroup できない \
                     (先に unfreeze すること)",
                    group.0
                )));
            }
            let new_parent = view.attrs(group)?.and_then(|attrs| attrs.parent);
            let group_local = view.local_transform(group, t)?;
            let identity = affine2_is_identity(group_local);

            for &child in &present {
                let Some(child_attrs) = view.attrs(child)? else {
                    continue;
                };
                if child_attrs.parent != Some(group) {
                    continue;
                }

                intents.push(Intent::SetAttrs {
                    layer: child,
                    patch: LayerAttrsPatch {
                        parent: Some(new_parent),
                        ..Default::default()
                    },
                });

                if !identity {
                    let anchor = read_vec2(&view, child, crate::doc::store::property::ANCHOR, [0.0, 0.0], t)?;
                    let child_local = view.local_transform(child, t)?;
                    let baked = bake_child_local(group_local, child_local, anchor);
                    intents.extend(baked.into_intents(child)?);
                }

                released.push(child);
            }

            intents.push(Intent::RemoveLayer(group));
        }

        self.apply_all(intents)?;
        Ok(released)
    }

}

struct BakedChildTransform {
    position: [f64; 2],
    rotation_degrees: f64,
    scale: [f64; 2],
    skew_degrees: f64,
    skew_axis_degrees: f64,
}

impl BakedChildTransform {
    fn into_intents(self, layer: LayerId) -> Result<Vec<Intent>, StoreError> {
        Ok(vec![
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::POSITION)?,
                track: still(Value::Vec2(self.position)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::ROTATION)?,
                track: still(Value::F64(self.rotation_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SCALE)?,
                track: still(Value::Vec2(self.scale)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SKEW)?,
                track: still(Value::F64(self.skew_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SKEW_AXIS)?,
                track: still(Value::F64(self.skew_axis_degrees)),
            },
        ])
    }
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn bake_child_local(
    group_local: glam::Affine2,
    child_local: glam::Affine2,
    anchor: [f32; 2],
) -> BakedChildTransform {
    use glam::{Affine2, Mat2, Vec2};

    let x = child_local * Affine2::from_translation(Vec2::new(anchor[0], anchor[1]));
    let x_prime = group_local * x;

    let position = x_prime.translation;
    let linear = x_prime.matrix2;

    let col0 = linear.x_axis;
    let sx = col0.length();
    let theta = if sx > 1e-6 { col0.y.atan2(col0.x) } else { 0.0 };

    let rest = Mat2::from_angle(-theta) * linear;
    let sy = rest.y_axis.y;
    let skew_tan = if sy.abs() > 1e-6 { rest.y_axis.x / sy } else { 0.0 };

    BakedChildTransform {
        position: [position.x as f64, position.y as f64],
        rotation_degrees: theta.to_degrees() as f64,
        scale: [sx as f64, sy as f64],
        skew_degrees: skew_tan.atan().to_degrees() as f64,
        skew_axis_degrees: 0.0,
    }
}

fn affine2_is_identity(m: glam::Affine2) -> bool {
    const EPS: f32 = 1e-4;
    m.translation.length() < EPS
        && (m.matrix2.x_axis - glam::Vec2::X).length() < EPS
        && (m.matrix2.y_axis - glam::Vec2::Y).length() < EPS
}

fn read_vec2(
    view: &StoreView,
    layer: LayerId,
    name: &str,
    default: [f32; 2],
    t: RationalTime,
) -> Result<[f32; 2], StoreError> {
    let property = PropertyId::new(name)?;
    match view.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
        Some(other) => Err(StoreError::Property(format!(
            "{name} に2成分でない値が入っている: {other:?}"
        ))),
        None => Ok(default),
    }
}
