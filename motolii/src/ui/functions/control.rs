use super::atom;
use crate::doc::store::{StoreError, Value};

pub(crate) fn axis_delta(
    value: &Value,
    axis: usize,
    delta: f64,
    range: Option<(f64, f64)>,
) -> Result<Value, StoreError> {
    let current = match value {
        Value::F64(value) => *value,
        Value::Vec2(value) => *value
            .get(axis)
            .ok_or_else(|| StoreError::Property("Invalid value axis".into()))?,
        _ => return Err(StoreError::Property("This value is not numeric".into())),
    };
    axis_absolute(value, axis, current + delta, range)
}

pub(crate) fn axis_absolute(
    value: &Value,
    axis: usize,
    number: f64,
    range: Option<(f64, f64)>,
) -> Result<Value, StoreError> {
    let number =
        atom::bounded(number, range).map_err(|reason| StoreError::Property(reason.into()))?;
    match value {
        Value::F64(_) => Ok(Value::F64(number)),
        Value::Vec2(original) => {
            let mut changed = *original;
            *changed
                .get_mut(axis)
                .ok_or_else(|| StoreError::Property("Invalid value axis".into()))? = number;
            Ok(Value::Vec2(changed))
        }
        _ => Err(StoreError::Property("This value is not numeric".into())),
    }
}
