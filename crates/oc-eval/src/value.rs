use serde::{Deserialize, Serialize};

/// パラメータ値。補間は同一バリアント間のみ定義される。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    F64(f64),
    Vec2([f64; 2]),
    /// RGBA(リニア、0.0-1.0想定)
    Color([f64; 4]),
}

impl Value {
    /// 線形補間。バリアント不一致はaを返す(型不一致はドキュメント検証層で弾く前提)。
    pub fn lerp(a: &Value, b: &Value, u: f64) -> Value {
        match (a, b) {
            (Value::F64(x), Value::F64(y)) => Value::F64(x + (y - x) * u),
            (Value::Vec2(x), Value::Vec2(y)) => {
                Value::Vec2(std::array::from_fn(|i| x[i] + (y[i] - x[i]) * u))
            }
            (Value::Color(x), Value::Color(y)) => {
                Value::Color(std::array::from_fn(|i| x[i] + (y[i] - x[i]) * u))
            }
            _ => a.clone(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(v) => Some(*v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_scalar_and_vector() {
        assert_eq!(
            Value::lerp(&Value::F64(0.0), &Value::F64(10.0), 0.25),
            Value::F64(2.5)
        );
        assert_eq!(
            Value::lerp(&Value::Vec2([0.0, 100.0]), &Value::Vec2([10.0, 200.0]), 0.5),
            Value::Vec2([5.0, 150.0])
        );
    }

    #[test]
    fn lerp_mismatched_variants_returns_first() {
        let a = Value::F64(1.0);
        let b = Value::Vec2([0.0, 0.0]);
        assert_eq!(Value::lerp(&a, &b, 0.5), a);
    }
}
