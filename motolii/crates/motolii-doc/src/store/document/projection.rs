use crate::doc::core::{projection_switch_world, RationalTime};
use crate::doc::eval::Value;
use crate::doc::store::view::StoreView;
use crate::doc::store::{property, LayerAttrsPatch, LayerProjection, StoreError};

use super::group::move_translation_values;
use super::{Document, Intent, LayerId};

impl Document {
    /// Applies `patch` to each layer. A projection change keeps the picture seen at `at`
    /// under the document camera, like a parent change keeps the pose at drop time.
    /// `layers` pairs each layer with the center of its local bounds.
    pub fn set_projection(
        &mut self,
        layers: &[(LayerId, [f32; 3])],
        patch: LayerAttrsPatch,
        at: RationalTime,
    ) -> Result<(), StoreError> {
        let mut intents = Vec::new();
        {
            let view = self.view();
            for &(layer, local_center) in layers {
                if let Some(to) = patch.projection {
                    intents.extend(projection_compensation(&view, self.geometry(), layer, local_center, to, at)?);
                }
                intents.push(Intent::SetAttrs { layer, patch: patch.clone() });
            }
        }
        self.apply_all(intents)
    }
}

fn projection_compensation(
    view: &StoreView<'_>,
    geometry: crate::doc::store::geometry::Geometry,
    layer: LayerId,
    local_center: [f32; 3],
    to: LayerProjection,
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    let attrs = view.attrs(layer)?.unwrap_or_default();
    let from = attrs.projection;
    if from == to {
        return Ok(Vec::new());
    }
    let comp = view
        .composition()?
        .ok_or_else(|| StoreError::Property("No composition".into()))?
        .spec();
    let camera = (geometry.camera)(view, at)?;
    let worlds = (geometry.worlds)(view, at)?;
    let world = *worlds.get(&layer).ok_or_else(|| {
        StoreError::Property(format!("Layer {} is not present", layer.0))
    })?;
    let parent = attrs
        .parent
        .and_then(|id| worlds.get(&id).copied())
        .unwrap_or(glam::Affine3A::IDENTITY);
    let switched = projection_switch_world(comp, camera, from, to, world, local_center.into())
        .ok_or_else(|| StoreError::Property("Cannot keep this layer's place across the projection change; it is not in front of the camera".into()))?;
    // 法(2026-09-12): 札を変えても**中心は画面の同じ場所**に留める。面の向きは新しい札の意味に従う
    // (2D は画面に正対、2.5D は既定カメラに正対、3D は自分の回転)ので、回転や scale は書き換えない。
    // 動かすのは位置だけなので、位置が animate されていても全 key を同じ量ずらせる。
    let center = glam::Vec3::from(local_center);
    let before = world.transform_point3(center);
    let after = switched.transform_point3(center);
    let delta = parent.inverse().transform_vector3(after - before);
    if !delta.is_finite() {
        return Err(StoreError::Property("Projection change needs a finite transform".into()));
    }
    if delta.length() <= 1e-3 {
        return Ok(Vec::new());
    }
    move_translation_values(view, layer, delta.to_array().map(f64::from))
}
