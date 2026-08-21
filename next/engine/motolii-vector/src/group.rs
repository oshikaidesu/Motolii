//! シェイプ内の入れ子グループ(裁定173 H4「シェイプ内階層」)。
//!
//! 起点は `docs/reviews/2026-08-22-transform-hierarchy-decision.md` H4 と、その
//! 前提の H1(`motolii-store::view::world_affine` — 親が動くと子も動く・
//! キーフレームは各ノードローカルのまま・**合成だけが再帰**)。ここは同じ考え方を
//! shape 粒度へ持ち込む。
//!
//! **旧世界の先例**: `crates/motolii-doc/src/schema.rs::VectorContent::Group`
//! (`{ children: Vec<VectorContent> }` — children のみで transform は持たない)を
//! 「入れ子という構造」の先例として読んだ。transform 付きの入れ子は本裁定が新設する
//! (旧世界のコード自体には transform フィールドが無いことを実測確認済み —
//! `docs/reviews/2026-08-22-transform-hierarchy-seam-survey.md` §3.1)。
//!
//! **変換の型もアフィン代数も新設しない**: [`ShapeGroup::transform`] は
//! [`crate::RepeaterTransform`] をそのまま再利用する(`T(position)·R(rotation)·
//! S(scale)·T(-anchor)`、裁定58と同じ順序 — `motolii_core::LayerPlacement::
//! from_transform` の skew を除いた形と同型)。合成の行列代数も `ops::Affine`
//! (repeater が既に持つ、旧 `pathgeom.rs` 移植)をそのまま使い回す。この crate は
//! 元々 `next/core/*` を1つも引かない(裁定 crate 独立性、`Cargo.toml` 参照)ので、
//! `motolii-core::glam::Affine2` を新たに引き込んで**同じ意味の代数を2つ持つ**より、
//! 既存の `ops::Affine` を単一源として使い回す方が「無理な複製をしない」に合う
//! (H4 発注書 ALLOWLIST の「motolii-core 変更は最小」注記はこの判断で満たす —
//! 変更ゼロで済んだ)。
//!
//! **`Shape` 自体は1バイトも変えていない**。`next/engine/motolii-engine/src/
//! mask.rs::rasterize_mask_coverage` が今も `Shape { source, ops, fill, stroke }`
//! という4フィールドの struct literal を直接組んでおり、`Shape` へ新フィールドを
//! 足すとそこが壊れる(この crate の外・H4 の ALLOWLIST 外)。入れ子はここで
//! `Shape` を包む別の型 [`ShapeNode`] として足す。

use crate::ops::{self, Affine};
use crate::{Canvas, Fill, Path, RepeaterTransform, Shape, Stroke, VectorError};
use serde::{Deserialize, Serialize};

/// シェイプ内グループのノード。
///
/// **`#[serde(untagged)]`**: 既存の平坦 `Shape` の JSON(`source`/`ops`/`fill`/
/// `stroke` の4キー)をそのまま [`ShapeNode::Leaf`] として読める — 裁定173 H4
/// oracle「既存 flat shape の JSON 不変」はこの1点で成立する。`Shape` の必須キー
/// (`source`)と [`ShapeGroup`] の必須キー(`transform`/`children`)は重ならないので、
/// untagged の曖昧性(両方の variant が同時にマッチしうる状態)は生じない —
/// `tests/group.rs` の往復試験がこれを固定する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShapeNode {
    /// 既存の平坦 `Shape`(旧 JSON の着地点)。
    Leaf(Shape),
    /// 子 shape 列 + ローカル変換。
    Group(ShapeGroup),
}

/// 子 shape 列 + ローカル変換。
///
/// **変換キーはこのノード自身のローカル値**であり、子は自分のローカル値のまま —
/// world は [`flatten`] が行う合成でのみ生まれる(裁定173 H4、利用者裁定「シェイプ
/// のキーフレームにも適用され、再帰的に決まる」の実装形)。`members`/`children`
/// を親側とローカル側の二重に持たせていない(H1 決定文書 §4.4「二重帳簿の危険」と
/// 同じ理由で、正本は常にこの1本 — `ShapeGroup.children` が構造の唯一の正本)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeGroup {
    pub transform: RepeaterTransform,
    pub children: Vec<ShapeNode>,
}

/// 入れ子 tree を、既存のラスタライズ経路がそのまま描ける平坦な `Shape` の列へ
/// 展開する**純関数**(ラスタライズしない・キーフレーム評価もしない — 渡された
/// `RepeaterTransform` の値をそのまま合成するだけ)。
///
/// - 各 leaf はまず自分の ops を自分の local 空間で解決し(`resolve`)、その後
///   world 変換(祖先 `Group` の transform を `Affine::mul` で合成した行列)を
///   **頂点へ焼き込む**。返す `Shape` の `ops` は常に空 — 焼き込み済みの ops を
///   `render`/`render_tree` がもう一度適用してしまう二重適用を型で防ぐ。
/// - repeater が1つの leaf から複数 [`ops::Instance`] を作る場合、instance ごとに
///   1つの平坦 `Shape` を返す。instance の opacity は `raster::draw` が
///   `f.opacity * weight` で行う掛け算と同じ式を使って fill/stroke の opacity へ
///   先に畳んでおく(`render_tree` 側の `resolve` は ops が空なので opacity=1.0の
///   1 instance しか作らず、掛け算はここで完結している必要がある)。
/// - **空グループは単位元**: `children` が空なら再帰は何も `out` へ積まない。
///   「空なら何もしない」以上の特別扱いのコードを書いていない — これが単位元の
///   実装そのもの。
/// - **決定論的**: 合成は `Affine::mul` の純粋な行列積のみで、乱数・時刻・
///   グローバル状態を一切読まない。同じ `&[ShapeNode]` は常に同じ `Vec<Shape>`
///   を返す(`tests/group.rs::flatten_is_deterministic` が同じ入力を2回流して縛る)。
pub fn flatten(nodes: &[ShapeNode]) -> Result<Vec<Shape>, VectorError> {
    let mut out = Vec::new();
    flatten_into(nodes, &Affine::IDENTITY, &mut out)?;
    Ok(out)
}

fn flatten_into(
    nodes: &[ShapeNode],
    parent_world: &Affine,
    out: &mut Vec<Shape>,
) -> Result<(), VectorError> {
    for node in nodes {
        match node {
            ShapeNode::Leaf(shape) => {
                for instance in crate::resolve(shape)? {
                    out.push(Shape {
                        source: crate::PathSource::Bezier(transform_path(
                            &instance.path,
                            parent_world,
                        )),
                        ops: Vec::new(),
                        fill: shape.fill.as_ref().map(|f| Fill {
                            opacity: f.opacity * instance.opacity,
                            ..f.clone()
                        }),
                        stroke: shape.stroke.as_ref().map(|s| Stroke {
                            opacity: s.opacity * instance.opacity,
                            ..s.clone()
                        }),
                    });
                }
            }
            ShapeNode::Group(group) => {
                // world = parent_world ∘ own(own を先に適用) — H1 の
                // `parent_world * local`(`view.rs:942`)と同じ合成の向き。
                let own = ops::build_affine(&group.transform);
                let world = parent_world.mul(&own);
                flatten_into(&group.children, &world, out)?;
            }
        }
    }
    Ok(())
}

fn transform_path(path: &Path, m: &Affine) -> Path {
    path.iter()
        .map(|c| ops::apply_matrix_to_contour(c, m))
        .collect()
}

/// [`flatten`] してから、`render()` が使うのと同じラスタライズ経路
/// (`raster::new_pixmap`/`raster::draw`/`raster::finish`)へ渡す。**rasterizer 本体
/// (`raster.rs`/`coverage.rs`)は無改修** — ここが呼ぶのは既存の3つの `pub(crate)`
/// 関数のみで、シグネチャも中身も変えていない。
///
/// `render()`(単一 `Shape`)は変えていない — mask.rs 等の既存呼び手はそのまま。
/// 入れ子 tree の描画はこちらが唯一の口になる。
pub fn render_tree(nodes: &[ShapeNode], canvas: &Canvas) -> Result<crate::Raster, VectorError> {
    let mut pixmap = crate::raster::new_pixmap(canvas)?;
    let origin = crate::Point {
        x: canvas.origin_x as f64,
        y: canvas.origin_y as f64,
    };
    for shape in flatten(nodes)? {
        for instance in crate::resolve(&shape)? {
            crate::raster::draw(
                &mut pixmap,
                &instance.path,
                origin,
                shape.fill.as_ref(),
                shape.stroke.as_ref(),
                instance.opacity,
            );
        }
    }
    Ok(crate::raster::finish(pixmap, canvas))
}
