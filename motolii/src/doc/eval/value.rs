use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathVertex {
    pub point: [f64; 2],
    pub in_tangent: [f64; 2],
    pub out_tangent: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Path {
    pub vertices: Vec<PathVertex>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    F64(f64),
    Vec2([f64; 2]),
    Color([f64; 4]),
    Bool(bool),
    Path(Path),
    Enum(i64),
    LayerId(u64),
}

impl Value {
    pub fn lerp(a: &Value, b: &Value, u: f64) -> Value {
        match (a, b) {
            (Value::F64(x), Value::F64(y)) => Value::F64(x + (y - x) * u),
            (Value::Vec2(x), Value::Vec2(y)) => {
                Value::Vec2(std::array::from_fn(|i| x[i] + (y[i] - x[i]) * u))
            }
            (Value::Color(x), Value::Color(y)) => {
                Value::Color(std::array::from_fn(|i| x[i] + (y[i] - x[i]) * u))
            }
            (Value::Bool(_), Value::Bool(_)) => a.clone(),
            (Value::Enum(_), Value::Enum(_)) => a.clone(),
            (Value::LayerId(_), Value::LayerId(_)) => a.clone(),
            (Value::Path(x), Value::Path(y))
                if x.vertices.len() == y.vertices.len() && x.closed == y.closed =>
            {
                Value::Path(Path {
                    vertices: x
                        .vertices
                        .iter()
                        .zip(y.vertices.iter())
                        .map(|(x, y)| PathVertex {
                            point: std::array::from_fn(|i| {
                                x.point[i] + (y.point[i] - x.point[i]) * u
                            }),
                            in_tangent: std::array::from_fn(|i| {
                                x.in_tangent[i] + (y.in_tangent[i] - x.in_tangent[i]) * u
                            }),
                            out_tangent: std::array::from_fn(|i| {
                                x.out_tangent[i] + (y.out_tangent[i] - x.out_tangent[i]) * u
                            }),
                        })
                        .collect(),
                    closed: x.closed,
                })
            }
            _ => a.clone(),
        }
    }

    pub fn add(&self, other: &Value) -> Option<Value> {
        match (self, other) {
            (Value::F64(a), Value::F64(b)) => Some(Value::F64(a + b)),
            (Value::Vec2(a), Value::Vec2(b)) => {
                Some(Value::Vec2(std::array::from_fn(|i| a[i] + b[i])))
            }
            (Value::Color(a), Value::Color(b)) => {
                Some(Value::Color(std::array::from_fn(|i| a[i] + b[i])))
            }
            (Value::Path(a), Value::Path(b))
                if a.vertices.len() == b.vertices.len() && a.closed == b.closed =>
            {
                Some(Value::Path(Path {
                    vertices: a
                        .vertices
                        .iter()
                        .zip(b.vertices.iter())
                        .map(|(x, y)| PathVertex {
                            point: std::array::from_fn(|i| x.point[i] + y.point[i]),
                            in_tangent: std::array::from_fn(|i| x.in_tangent[i] + y.in_tangent[i]),
                            out_tangent: std::array::from_fn(|i| {
                                x.out_tangent[i] + y.out_tangent[i]
                            }),
                        })
                        .collect(),
                    closed: a.closed,
                }))
            }
            (Value::Bool(_), _) | (Value::Enum(_), _) | (Value::LayerId(_), _) => None,
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vec2(&self) -> Option<[f64; 2]> {
        match self {
            Value::Vec2(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<[f64; 4]> {
        match self {
            Value::Color(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Value::Path(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<i64> {
        match self {
            Value::Enum(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_layer_id(&self) -> Option<u64> {
        match self {
            Value::LayerId(v) => Some(*v),
            _ => None,
        }
    }
}
