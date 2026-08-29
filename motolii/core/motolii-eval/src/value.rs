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
/// **`Enum` / `LayerId` を effect 発注単位(裁定72)で足した**: effect の param 型語彙
/// (scalar / Bool / Enum / color / Vec2 / LayerId、effect-values の catalog から)を
/// 「全部 animatable なので既存の `KeyframeTrack` にそのまま乗る」ためには、この `Value`
/// 自身がその2つを表現できる必要がある。**F64 で代用しない**理由は `Bool` を足した理由と
/// 同じ形 — drop-down の選択肢 index も layer 参照も、連続量として linear 補間すると
/// 意味の無い中間状態(存在しない選択肢・存在しない層)が作れてしまう。
///
/// **列挙(blend mode / matte mode / mask mode 等)はここに入れない**。キーを打たない
/// 静止設定なので `LayerMeta` 側の component が持つ。入れると「補間できない `Value`」が増える。
/// `Value::Enum` はそれとは別物 — こちらは effect param のように「animatable だが
/// 中間値が無意味」なケース用(`Bool` と同じ扱い)。
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
    /// effect の drop-down param(`effect-values/drop-down v`)。plugin が定義する
    /// 選択肢表への index。**選択肢そのもの(文字列表)は Document が持たない**
    /// (裁定70 — plugin の一次資料)。**補間は Hold**(`Bool` と同じ理由 — 0.5 番目の
    /// 選択肢は存在しない)。
    Enum(i64),
    /// effect の layer param(`effect-values/layer v`)。**Lottie の `ind` ではなく
    /// 安定 `LayerId`**(裁定65 — 挿入削除で崩れる index を参照に使わない)。
    /// `crate::LayerId` を再輸出しない(motolii-eval は motolii-store に依存しない
    /// 下位 crate なので、生の `u64` で持ち store 側が `LayerId(v)` へ包み直す)。
    /// **補間は Hold**(層の参照先が半分だけ変わる中間状態は無い)。
    LayerId(u64),
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
            // 選択肢 index / layer 参照も同じ理由で Hold(存在しない中間状態を作らない)。
            (Value::Enum(_), Value::Enum(_)) => a.clone(),
            (Value::LayerId(_), Value::LayerId(_)) => a.clone(),
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

    /// 加算での合成(裁定213: 接続子は加算)。base の値へ modulator の寄与を積む
    /// ための操作 — `lerp` と対になるが意味は別: `lerp` は「a と b の間」、`add` は
    /// 「複数の寄与の和」。
    ///
    /// **変調できる型の境界は発明しない**——このファイル冒頭で `Bool`/`Enum`/
    /// `LayerId` の3つに「補間は Hold」と書いた理由(中間状態が存在しない)は、
    /// 加算にもそのまま刺さる: `true + true` や「0.5番目の選択肢」と同様
    /// 「存在しない選択肢を2つ足す」の意味は無い。**この3つは常に `None`** ——
    /// 呼び手(`motolii-store::StoreView`)は `None` を「この modulator は寄与
    /// できない、単一 source(base か、先に確定した値)が勝つ」として扱う。
    ///
    /// `F64`/`Vec2`/`Color` は成分ごとの和。**`Color` はアルファも含めた4成分を
    /// 区別せず和にする** — `lerp` の `Color` 実装が既に4成分を区別せず一様に
    /// 補間しており(このファイル内で確認できる)、`add` だけ特別に3成分+1成分へ
    /// 割ると「色は rgb と alpha で意味が違う」という、`Value` 自身がどこにも
    /// 書いていない境界を新しく発明することになる。表示域(0.0–1.0)へ収めるのは
    /// 既存の非線形 straight-alpha 表現と同じくレンダ側の責務(型 doc 参照)。
    ///
    /// **アリティは2本の独立な軸で決まる**(利用者裁定213 の追加整理): **instance**
    /// (1つの値が持つ要素の個数——`Path` の頂点)と**成分**(1要素の中の x/y/z/
    /// rgba)。`F64`=1 instance×1成分・`Vec2`=1×2・`Color`=1×4・`Path`=N×2。
    /// **成分軸**は新しい指し方を作らない——`Vec2`/`Color` の特定成分だけを
    /// 変調したければ、`position`/`position.x`/`position.y`(裁定61 の
    /// split-position、Lottie `s` フラグと同型、`next/reference/lottie-coverage.tsv:330-332`)
    /// と同じ「別の `PropertyId` で持つ」という**既存の語**を使う——この `add` の
    /// 型には成分を選ぶ引数を足さない(`PropertyId` が既に成分を表現できるので
    /// 新語は不要)。**instance 軸は今は 1 に固定**——`Path` の modulator は
    /// `lerp` と全く同じ条件(頂点数と `closed` が一致する時だけ)で全頂点を
    /// 一括して和にする(点・両接線とも)。**1本の modulator が特定の1頂点だけを
    /// 狙う**という instance 単位のアドレス指定は持たない(2人目の利用者が
    /// 要求したら、その時に `PropertyLink` 側へ instance 選択を足す——裁定13と
    /// 同じ「今は決めない」判断、`motolii-store::slot` モジュール doc 参照)。
    /// **近似しない**(`crate::Value` を使う `motolii-store::slot::translate_link`
    /// の「型不一致は黙って `None`」と同じ規約) —— 条件を満たさない・型が違う
    /// 場合はすべて `None`。
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
            // Hold 型(補間が中間状態を持たない3つ)は加算も無意味 — 単一 source が
            // 勝つ、という上のdoc通り常に None。
            (Value::Bool(_), _) | (Value::Enum(_), _) | (Value::LayerId(_), _) => None,
            // 型不一致・Path の頂点数/closed 不一致は近似しない。
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

    /// 真偽の中間は意味を持たない。**0.5 で 0.5 になる型で持っていたのが元の欠陥**。
    #[test]
    fn bool_holds_until_the_next_key() {
        let a = Value::Bool(false);
        let b = Value::Bool(true);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::Bool(false));
        assert_eq!(Value::lerp(&a, &b, 0.99), Value::Bool(false));
    }

    /// drop-down の選択肢 index も同じ理由で Hold(effect-values/drop-down、裁定72)。
    #[test]
    fn enum_holds_until_the_next_key() {
        let a = Value::Enum(0);
        let b = Value::Enum(3);
        assert_eq!(Value::lerp(&a, &b, 0.5), Value::Enum(0));
        assert_eq!(Value::lerp(&a, &b, 0.99), Value::Enum(0));
        assert_eq!(Value::Enum(3).as_enum(), Some(3));
        assert_eq!(Value::F64(3.0).as_enum(), None);
    }

    /// effect の layer 参照も同じ理由で Hold(effect-values/layer、裁定65/72)。
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
