use serde::{Deserialize, Serialize};

/// ベジェパスの1頂点。接線は**頂点からの相対**で持つ。
///
/// Lottie の `values/bezier` の `v` / `i` / `o` と同型(裁定85 の調査で確認)。
/// **3本の可変長配列ではなく1つの型にしてある** — 別々の配列にすると長さが食い違う
/// 不正状態が作れる。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathVertex {
    pub point: [f64; 2],
    pub in_tangent: [f64; 2],
    pub out_tangent: [f64; 2],
}

/// ベジェパス。マスク形状と shape のパス源の正本。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Path {
    pub vertices: Vec<PathVertex>,
    pub closed: bool,
}

/// パラメータ値。補間は同一バリアント間のみ定義される。
///
/// **バリアントの構成は Lottie 地図が正した**(裁定78):
/// - `Bool` を足した — 無い間は `F64(0.0/1.0)` で持っており、これは Lottie の
///   `int-boolean`(JSON に bool があるのに 0/1 で持つ)を**批判しながら同じ形**だった
/// - `Path` を足した — マスク形状。`List` の入れ子は禁止なので専用バリアントが要る
/// - `Vec3` / `AssetRef` / `List` を落とした — いずれも `next/` から参照ゼロで、
///   `Vec3` の用途は全部 3D 系(裁定65 で不採用)、素材参照は `LayerMeta` 側で
///   property ではなく、`List` は載せる物が無い
///
/// **列挙(blend mode / matte mode / mask mode 等)はここに入れない**。キーを打たない
/// 静止設定なので `LayerMeta` 側の component が持つ。入れると「補間できない `Value`」が増える。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    F64(f64),
    Vec2([f64; 2]),
    /// RGBA: 非線形sRGB・straight-alpha・各成分0.0–1.0(M2E-13。リニア化はレンダ側)
    Color([f64; 4]),
    /// 真偽。**補間は Hold**(0.5 で 0.5 になっても意味が無い)。
    Bool(bool),
    /// ベジェパス。頂点数が同じ時だけ頂点ごとに補間する。
    Path(Path),
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
            // 真偽の中間は意味を持たないので、切り替わるまで手前の値を保つ。
            (Value::Bool(_), Value::Bool(_)) => a.clone(),
            // 頂点数が違うパスの補間は定義しない(AE も頂点数を揃えないと補間しない)。
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

    /// 真偽の中間は意味を持たない。**0.5 で 0.5 になる型で持っていたのが元の欠陥**。
    #[test]
    fn bool_holds_until_the_next_key() {
        let a = Value::Bool(false);
        let b = Value::Bool(true);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::Bool(false));
        assert_eq!(Value::lerp(&a, &b, 0.99), Value::Bool(false));
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

    /// 頂点数が違うパスは補間しない(AE も揃えないと補間しない)。
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
