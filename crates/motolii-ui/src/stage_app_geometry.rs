//! Document から Stage の layer 幾何を出す。Stage の絵を出す側が使う。
//!
//! **`rn_product_host/stage_projection.rs` から救出したもの。** RN 製品面を畳んだ
//! (2026-08-16)ときに、合体シェルの Stage がこの投影を呼んでいることが分かった。
//! 元は 444行のファイルの中にあり、音声・書き出し・転送・GPU まで抱えた module に
//! 同居していたが、この投影自体は `Document` と `Affine2D` しか見ない。
//!
//! 投影の本体(`project_stage_geometry`)は `stage_geometry_projection.rs` に残っている。
//! ここにあるのはその結果を「4隅 + TRS」へ畳む薄い層だけ。
//!
//! **元との差**: RN の wire 型(`WireStageGeometryProjection` / `WireStageGeometryLayer`)を
//! 経由していたのを直接 `AppStageGeometry` へ畳んだ。wire は RN へ JSON で渡すための
//! 中間表現で、渡す相手が居なくなったため。数値の扱いは変えていない。

use motolii_core::RationalTime;
use motolii_doc::Affine2D;
use motolii_eval::DataTracks;

use crate::stage_geometry_projection::{project_stage_geometry, StageLayerProjection};

/// rn_product_host/mod.rs:66 `MAX_STAGE_BOUNDS` の写し。
const MAX_STAGE_BOUNDS: usize = 64;

/// Stage 上の layer 1枚。`corners` は CCW、local 左下起点、world 空間。
#[derive(Debug, Clone, PartialEq)]
pub struct AppStageGeometryLayer {
    pub layer_id: String,
    pub corners: [[f64; 2]; 4],
    pub position: [f64; 2],
    pub rotation: f64,
    pub scale: [f64; 2],
}

/// Stage に出る layer の一覧。`MAX_STAGE_BOUNDS` を超えた分は落として印を立てる。
#[derive(Debug, Clone, PartialEq)]
pub struct AppStageGeometry {
    pub layers: Vec<AppStageGeometryLayer>,
    pub layers_truncated: bool,
}

/// Stage 上の編集操作。gizmo が出す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppStageTransformEdit {
    TranslateWorld([f64; 2]),
    RotateZ(f64),
    Scale([f64; 2]),
}

/// `Available` な layer だけを4隅へ畳む。評価に失敗した時は空で返す
/// (Stage 全体を落とさない — 元の `project_stage_geometry_wire` と同じ扱い)。
pub(crate) fn app_stage_geometry(
    document: &motolii_doc::Document,
    time: RationalTime,
) -> AppStageGeometry {
    let eval = motolii_doc::EvaluationTime::new(time);
    let Ok(projection) = project_stage_geometry(document, eval, &DataTracks::new()) else {
        return AppStageGeometry {
            layers: Vec::new(),
            layers_truncated: false,
        };
    };
    let mut layers = Vec::new();
    let mut available = 0usize;
    let mut layers_truncated = false;
    for (layer_id, layer) in projection.layers() {
        let StageLayerProjection::Available(geo) = layer else {
            continue;
        };
        available += 1;
        if layers.len() >= MAX_STAGE_BOUNDS {
            layers_truncated = true;
            continue;
        }
        let hw = geo.local_rect.size.width * 0.5;
        let hh = geo.local_rect.size.height * 0.5;
        let cx = geo.local_rect.center.x;
        let cy = geo.local_rect.center.y;
        // CCW・local 左下起点。world のみ(camera_view は使わない)。
        let local = [
            [cx - hw, cy - hh],
            [cx + hw, cy - hh],
            [cx + hw, cy + hh],
            [cx - hw, cy + hh],
        ];
        let corners = world_rect_corners(geo.world, local);
        let (position, rotation, scale) = world_layer_trs(geo.world, [cx, cy]);
        layers.push(AppStageGeometryLayer {
            layer_id: layer_id.get().to_string(),
            corners,
            position,
            rotation,
            scale,
        });
    }
    if available > MAX_STAGE_BOUNDS {
        layers_truncated = true;
    }
    AppStageGeometry {
        layers,
        layers_truncated,
    }
}

fn world_layer_trs(world: Affine2D, center: [f64; 2]) -> ([f64; 2], f64, [f64; 2]) {
    (
        world.transform_point(center[0], center[1]),
        world.m[3].atan2(world.m[0]),
        world.approx_scale(),
    )
}

fn world_rect_corners(world: Affine2D, local: [[f64; 2]; 4]) -> [[f64; 2]; 4] {
    let mut corners = local.map(|[x, y]| {
        let p = world.transform_point(x, y);
        [p[0], p[1]]
    });
    // world determinant が負なら反転して CCW に揃える。
    if world.m[0] * world.m[4] - world.m[1] * world.m[3] < 0.0 {
        corners.reverse();
    }
    corners
}
