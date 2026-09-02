
use crate::doc::core::RationalTime;
use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Value};

use crate::doc::store::view::StoreView;
use crate::doc::store::{LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming, StoreError};

use super::{Document, Intent, LayerId, PropertyId};

impl Document {
    pub fn group_layers(&mut self, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
        let view = self.view();
        let roots = outermost_present(&view, layers)?;
        if roots.is_empty() {
            return Ok(None);
        }

        let group_id = LayerId(view.next_layer_id());
        let comp_duration = view
            .composition()?
            .map(|composition| composition.duration_frames)
            .unwrap_or(0);
        let order = view
            .meta(roots[0])?
            .map(|meta| meta.order)
            .ok_or_else(|| StoreError::Property(format!("layer {} has no placement", roots[0].0)))?;
        let common_parent = common_parent(&view, &roots)?;

        let mut intents = Vec::with_capacity(roots.len() + 3);
        intents.push(Intent::AddLayer(group_id));
        intents.push(Intent::SetMeta {
            layer: group_id,
            meta: LayerMeta {
                source: LayerSource::Group,
                order,
                timing: LayerTiming::place(0, None, comp_duration),
            },
        });
        intents.push(Intent::SetAttrs {
            layer: group_id,
            patch: LayerAttrsPatch {
                name: Some("Group".to_owned()),
                parent: Some(common_parent),
                ..Default::default()
            },
        });
        for &child in &roots {
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
        let t = RationalTime::ZERO;
        let view = self.view();
        let present = view.layers();
        let mut group_candidates = Vec::new();
        for &group in groups {
            if view
                .meta(group)?
                .is_some_and(|meta| meta.source == LayerSource::Group)
            {
                group_candidates.push(group);
            }
        }
        let groups = outermost_present(&view, &group_candidates)?;
        if groups.is_empty() {
            return Ok(Vec::new());
        }

        let mut intents = Vec::new();
        let mut released = Vec::new();

        for &group in &groups {
            if view.attrs(group)?.unwrap_or_default().frozen {
                return Err(StoreError::Property(format!(
                    "layer {} は凍結中(frozen)なので ungroup できない \
                     (先に unfreeze すること)",
                    group.0
                )));
            }
            reject_animated_transform(&view, group, "group")?;
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

                if !identity {
                    reject_animated_transform(&view, child, "child")?;
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

fn outermost_present(view: &StoreView<'_>, layers: &[LayerId]) -> Result<Vec<LayerId>, StoreError> {
    let present: std::collections::HashSet<_> = view.layers().into_iter().collect();
    let mut unique = Vec::new();
    let mut selected = std::collections::HashSet::new();
    for &layer in layers {
        if present.contains(&layer) && selected.insert(layer) {
            unique.push(layer);
        }
    }

    let mut roots = Vec::new();
    for layer in unique {
        let mut parent = view.attrs(layer)?.and_then(|attrs| attrs.parent);
        let mut seen = std::collections::HashSet::new();
        let mut has_selected_ancestor = false;
        while let Some(candidate) = parent {
            if !seen.insert(candidate) {
                break;
            }
            if selected.contains(&candidate) {
                has_selected_ancestor = true;
                break;
            }
            parent = view.attrs(candidate)?.and_then(|attrs| attrs.parent);
        }
        if !has_selected_ancestor {
            roots.push(layer);
        }
    }
    Ok(roots)
}

fn common_parent(view: &StoreView<'_>, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
    let Some((&first, rest)) = layers.split_first() else {
        return Ok(None);
    };
    let parent = view.attrs(first)?.and_then(|attrs| attrs.parent);
    for &layer in rest {
        if view.attrs(layer)?.and_then(|attrs| attrs.parent) != parent {
            return Err(StoreError::Property(
                "cannot group layers from different parents without changing their world transforms"
                    .to_owned(),
            ));
        }
    }
    Ok(parent)
}

const TRANSFORM_PROPERTIES: &[&str] = &[
    crate::doc::store::property::ANCHOR,
    crate::doc::store::property::POSITION,
    crate::doc::store::property::POSITION_X,
    crate::doc::store::property::POSITION_Y,
    crate::doc::store::property::SCALE,
    crate::doc::store::property::ROTATION,
    crate::doc::store::property::SKEW,
    crate::doc::store::property::SKEW_AXIS,
];

fn reject_animated_transform(
    view: &StoreView<'_>,
    layer: LayerId,
    role: &str,
) -> Result<(), StoreError> {
    for &name in TRANSFORM_PROPERTIES {
        let property = PropertyId::new(name)?;
        if view
            .track(layer, &property)?
            .is_some_and(|track| track.keys().len() > 1)
        {
            return Err(StoreError::Property(format!(
                "cannot ungroup: {role} layer {} has animated `{name}`; keep the group or remove the animation first",
                layer.0
            )));
        }
    }
    Ok(())
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
