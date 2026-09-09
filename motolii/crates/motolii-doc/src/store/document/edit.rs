use super::{validate, Document, Intent, LayerId, PropertyId};
use crate::doc::store::{
    property, Interp, Keyframe, PropertyBase, RationalTime, StoreError, Value,
};

impl Document {
    /// 値を置く。`animate` が入っていれば今の時刻のキーになる(無ければ 1 キー目)。
    /// 切れていれば、キーのある属性は全部のキーに同じ差を足して形を保ち、無ければ値が変わるだけ。
    pub fn place_checked(
        &self,
        layer: LayerId,
        property: &PropertyId,
        value: Value,
        at: RationalTime,
        animate: bool,
    ) -> Result<Option<Intent>, StoreError> {
        let view = self.view().without_transients();
        if !view.has_layer(layer) {
            return Err(StoreError::Property(format!(
                "Layer {} no longer exists",
                layer.0
            )));
        }
        validate::check_not_locked(&view, layer)?;
        validate::check_not_frozen(&view, layer)?;
        let source = view.property_source(layer, property)?;
        if let Some(reason) = view.property_write_rejection(layer, property)? {
            return Err(StoreError::Property(reason.into()));
        }
        let current = view
            .value_at(layer, property, at)?
            .or(view.default_value(layer, property)?);
        if let Some(current) = &current {
            if std::mem::discriminant(current) != std::mem::discriminant(&value) {
                return Err(StoreError::Property(format!(
                    "Value type differs for {}",
                    property.name()
                )));
            }
        }
        match source.and_then(|source| source.base) {
            Some(PropertyBase::Slot(_)) => Err(StoreError::Property(
                "Edit the shared slot explicitly".into(),
            )),
            Some(PropertyBase::Track(mut track)) => {
                if !animate {
                    let Some(from) = current else {
                        return Err(StoreError::Property(format!("{} has no value to move", property.name())));
                    };
                    if from == value {
                        return Ok(None);
                    }
                    let mut moved = crate::doc::eval::KeyframeTrack::new();
                    for key in track.keys() {
                        moved.insert(Keyframe { value: shifted(&key.value, &from, &value), ..key.clone() });
                    }
                    return Ok(Some(Intent::SetTrack { layer, property: property.clone(), track: moved }));
                }
                if let Some(key) = track.keys().iter().find(|key| key.t == at) {
                    if key.value == value {
                        return Ok(None);
                    }
                    track.insert(Keyframe {
                        value,
                        ..key.clone()
                    });
                } else {
                    // Adding a key is a state change even when its evaluated value is unchanged.
                    track.insert(Keyframe {
                        t: at,
                        value,
                        interp: Interp::Linear,
                        spatial: None,
                    });
                }
                Ok(Some(Intent::SetTrack {
                    layer,
                    property: property.clone(),
                    track,
                }))
            }
            Some(PropertyBase::Constant(_)) | None => {
                if animate {
                    let mut track = crate::doc::eval::KeyframeTrack::new();
                    track.insert(Keyframe { t: at, value, interp: Interp::Linear, spatial: None });
                    return Ok(Some(Intent::SetTrack { layer, property: property.clone(), track }));
                }
                if current.as_ref() == Some(&value) {
                    return Ok(None);
                }
                Ok(Some(Intent::SetConstant {
                    layer,
                    property: property.clone(),
                    value,
                }))
            }
        }
    }

    pub fn begin_preview(&mut self) -> u64 {
        self.clear_all_transients();
        self.preview_owner
    }

    pub fn preview_is_current(&self, owner: u64) -> bool {
        owner != 0 && self.preview_owner == owner
    }

    pub fn preview_edits(&mut self, owner: u64, edits: &[Intent]) -> Result<(), StoreError> {
        if !self.preview_is_current(owner) {
            return Err(StoreError::Property(
                "This interaction has been superseded".into(),
            ));
        }
        let view = self.view().without_transients();
        for edit in edits {
            if let Intent::SetCameraConstant { .. } = edit {
                continue;
            }
            if let Intent::SetCameraTrack { track, .. } = edit {
                track
                    .validate()
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                continue;
            }
            let layer = match edit {
                Intent::SetTiming { layer, timing } => {
                    if timing.duration <= 0 {
                        return Err(StoreError::Property(
                            "A clip must keep a positive duration".into(),
                        ));
                    }
                    *layer
                }
                Intent::SetTrack { layer, track, .. } => {
                    track
                        .validate()
                        .map_err(|e| StoreError::Property(e.to_string()))?;
                    *layer
                }
                Intent::SetTextDocument { layer, document } => {
                    crate::doc::store::text::validate(document)?;
                    *layer
                }
                Intent::SetConstant { layer, .. } | Intent::SetShapes { layer, .. } => *layer,
                // 層属性の下書き(Sequence のゴーストの遅れ等)。attrs() が patch を重ねて読む。
                Intent::SetAttrs { layer, .. } => *layer,
                _ => {
                    return Err(StoreError::Property(
                        "This edit has no preview projection".into(),
                    ))
                }
            };
            if !view.has_layer(layer) {
                return Err(StoreError::Property(format!(
                    "Layer {} no longer exists",
                    layer.0
                )));
            }
            validate::check_not_locked(&view, layer)?;
            validate::check_not_frozen(&view, layer)?;
        }
        self.preview_edits = edits.to_vec();
        self.bump_transient_generation();
        Ok(())
    }

    pub fn clear_preview_edits(&mut self, owner: u64) -> bool {
        if !self.preview_is_current(owner) {
            return false;
        }
        self.clear_all_transients();
        true
    }
}

impl crate::doc::store::StoreView<'_> {
    pub fn property_write_rejection(&self, layer: LayerId, id: &PropertyId) -> Result<Option<&'static str>, StoreError> {
        let source = self.clone().without_transients().property_source(layer, id)?;
        Ok(match source {
            Some(source) if !source.modulators.is_empty() => Some("Edit the driver before changing a driven value"),
            Some(source) if matches!(source.base, Some(PropertyBase::Slot(_))) => Some("Edit the shared slot explicitly"),
            _ => None,
        })
    }

    pub fn default_value(
        &self,
        _layer: LayerId,
        id: &PropertyId,
    ) -> Result<Option<Value>, StoreError> {
        Ok(match id.name() {
            property::POSITION | property::ANCHOR => Some(Value::Vec2([0.0, 0.0])),
            property::SCALE => Some(Value::Vec2([1.0, 1.0])),
            property::OPACITY => Some(Value::F64(1.0)),
            property::ROTATION
            | property::ROTATION_X
            | property::ROTATION_Y
            | property::POSITION_X
            | property::POSITION_Y
            | property::POSITION_Z
            | property::SKEW
            | property::SKEW_AXIS
            | property::PAN
            | property::FADE_IN
            | property::FADE_OUT => Some(Value::F64(0.0)),
            _ => None,
        })
    }
}

/// `from` を `to` へ動かした差を `key` に足す。足せない型は `to` に置き換える。
fn shifted(key: &Value, from: &Value, to: &Value) -> Value {
    match (key, from, to) {
        (Value::F64(k), Value::F64(a), Value::F64(b)) => Value::F64(k + (b - a)),
        (Value::Vec2(k), Value::Vec2(a), Value::Vec2(b)) => Value::Vec2([k[0] + (b[0] - a[0]), k[1] + (b[1] - a[1])]),
        (Value::Color(k), Value::Color(a), Value::Color(b)) => {
            Value::Color(std::array::from_fn(|i| (k[i] + (b[i] - a[i])).clamp(0.0, 1.0)))
        }
        _ => to.clone(),
    }
}
