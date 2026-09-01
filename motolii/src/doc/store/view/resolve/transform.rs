
use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{property, LayerId, LayerPlacement, PropertyId, StoreError};

use super::super::StoreView;

impl<'a> StoreView<'a> {
    pub(super) fn local_placement_transform(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine2, StoreError> {
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        let vec2 = |name: &str, default: [f32; 2]| -> Result<[f32; 2], StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に2成分でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        Ok(LayerPlacement::from_transform(
            vec2(property::ANCHOR, [0.0, 0.0])?,
            self.resolve_position(layer, t)?,
            vec2(property::SCALE, [1.0, 1.0])?,
            scalar(property::ROTATION, 0.0)?,
            scalar(property::SKEW, 0.0)?,
            scalar(property::SKEW_AXIS, 0.0)?,
        ))
    }

    pub fn local_transform(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        self.local_placement_transform(layer, t)
    }

    pub(super) fn world_affine(
        &self,
        layer: LayerId,
        t: RationalTime,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<glam::Affine2, StoreError> {
        if let Some(world) = memo.get(&layer) {
            return Ok(*world);
        }

        let local = self.local_placement_transform(layer, t)?;

        let parent = self
            .attrs(layer)?
            .unwrap_or_default()
            .parent
            .filter(|p| present.contains(p));
        let Some(parent) = parent else {
            memo.insert(layer, local);
            return Ok(local);
        };

        if !visiting.insert(layer) {
            return Ok(local);
        }
        let parent_world = self.world_affine(parent, t, present, memo, visiting)?;
        visiting.remove(&layer);

        let world = parent_world * local;
        memo.insert(layer, world);
        Ok(world)
    }

    pub(super) fn resolve_position(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let position = PropertyId::new(property::POSITION)?;
        match self.value_at(layer, &position, t)? {
            Some(Value::Vec2(v)) => return Ok([v[0] as f32, v[1] as f32]),
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::POSITION
                )))
            }
            None => {}
        }

        let x = self.split_position_component(layer, property::POSITION_X, t)?;
        let y = self.split_position_component(layer, property::POSITION_Y, t)?;
        Ok([x.unwrap_or(0.0), y.unwrap_or(0.0)])
    }

    pub(super) fn split_position_component(
        &self,
        layer: LayerId,
        name: &str,
        t: RationalTime,
    ) -> Result<Option<f32>, StoreError> {
        let property = PropertyId::new(name)?;
        match self.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(Some(v as f32)),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(None),
        }
    }
}

