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

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f64, y: f64) -> PathVertex {
        PathVertex {
            point: [x, y],
            in_tangent: [0.0, 0.0],
            out_tangent: [0.0, 0.0],
        }
    }

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

    #[test]
    fn bool_holds_until_the_next_key() {
        let a = Value::Bool(false);
        let b = Value::Bool(true);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::Bool(false));
        assert_eq!(Value::lerp(&a, &b, 0.99), Value::Bool(false));
    }

    #[test]
    fn enum_holds_until_the_next_key() {
        let a = Value::Enum(0);
        let b = Value::Enum(3);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::Enum(0));
        assert_eq!(Value::lerp(&a, &b, 0.99), Value::Enum(0));
        assert_eq!(Value::Enum(3).as_enum(), Some(3));
        assert_eq!(Value::F64(3.0).as_enum(), None);
    }

    #[test]
    fn layer_id_holds_until_the_next_key() {
        let a = Value::LayerId(1);
        let b = Value::LayerId(2);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::LayerId(1));
        assert_eq!(Value::LayerId(7).as_layer_id(), Some(7));
        assert_eq!(Value::F64(7.0).as_layer_id(), None);
    }

    #[test]
    fn path_interpolates_vertex_by_vertex() {
        let a = Value::Path(Path {
            vertices: vec![vertex(0.0, 0.0), vertex(10.0, 0.0)],
            closed: true,
        });
        let b = Value::Path(Path {
            vertices: vec![vertex(0.0, 100.0), vertex(10.0, 100.0)],
            closed: true,
        });
        let Value::Path(mid) = Value::lerp(&a, &b, 0.5) else {
            panic!("path が返らない");
        };
        assert_eq!(mid.vertices[0].point, [0.0, 50.0]);
        assert_eq!(mid.vertices[1].point, [10.0, 50.0]);
        assert!(mid.closed);
    }

    #[test]
    fn path_with_different_vertex_counts_does_not_interpolate() {
        let a = Value::Path(Path {
            vertices: vec![vertex(0.0, 0.0)],
            closed: false,
        });
        let b = Value::Path(Path {
            vertices: vec![vertex(0.0, 0.0), vertex(1.0, 1.0)],
            closed: false,
        });
        assert_eq!(Value::lerp(&a, &b, 0.5), a);
    }
}
