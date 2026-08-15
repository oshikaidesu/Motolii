use std::sync::Arc;

use lyon_path::{Path as LyonPath, math::point};
use lyon_tessellation::{
    FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
    geometry_builder::{VertexBuffers, simple_builder},
};
use motolii_doc::{
    CompositeOrder, LineJoin, PointType, TrimMode,
    pathgeom::{Contour, Path, Point, ResolvedPathOp, ResolvedTransform, Vertex},
};
use re_chunk::Chunk;
use re_log_types::TimePoint;
use re_sdk_types::archetypes::Mesh3D;
use transform_gizmo::math::{DQuat, DVec3, Transform};

use crate::host_bridge::HostStageGeometry;

use super::{
    DOCUMENT_RECT_FILL_COLOR, FIXTURE_RECT_FILL_COLOR, FIXTURE_RECT_STROKE_COLOR,
    STAGE_HOST_ERASE_COLOR,
};

/// Preview fixtureだけの既定値。DocumentのPathOp評価・保存値をここへ持ち込まない。
pub(super) fn preview_path_operation(id: &str) -> Option<ResolvedPathOp> {
    Some(match id {
        // Puckerは頂点だけでなくハンドルも変形するため、Bezierを経由する表示確認になる。
        "pucker-bloat" => ResolvedPathOp::PuckerBloat { amount: -0.42 },
        "zig-zag" => ResolvedPathOp::ZigZag {
            amount: 0.035,
            ridges: 3.0,
            point_type: PointType::Smooth,
        },
        "offset" => ResolvedPathOp::Offset {
            distance: 0.035,
            line_join: LineJoin::Round,
            miter_limit: 4.0,
        },
        "round-corners" => ResolvedPathOp::RoundCorners { radius: 0.055 },
        "trim" => ResolvedPathOp::Trim {
            start: 0.08,
            end: 0.72,
            offset: 0.0,
            mode: TrimMode::Parallel,
        },
        "twist" => ResolvedPathOp::Twist {
            angle: std::f64::consts::FRAC_PI_4,
            center: Point::ZERO,
        },
        "wiggle" => ResolvedPathOp::Wiggle {
            amp: 0.035,
            freq: 1.0,
            seed: 7,
        },
        "repeater" => ResolvedPathOp::Repeater {
            copies: 3.0,
            offset: 0.0,
            transform: ResolvedTransform {
                position: Point { x: 0.075, y: 0.055 },
                anchor: Point::ZERO,
                scale: Point { x: 0.88, y: 0.88 },
                rotation: 0.12,
            },
            composite: CompositeOrder::Above,
            start_opacity: 1.0,
            end_opacity: 1.0,
        },
        _ => return None,
    })
}

pub(super) fn rectangle_path(center_x: f32, center_y: f32) -> Path {
    let half_width = 0.14;
    let half_height = 0.10;
    Path {
        contours: vec![Contour {
            vertices: vec![
                Vertex::corner(Point {
                    x: f64::from(center_x - half_width),
                    y: f64::from(center_y - half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x + half_width),
                    y: f64::from(center_y - half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x + half_width),
                    y: f64::from(center_y + half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x - half_width),
                    y: f64::from(center_y + half_height),
                }),
            ],
            closed: true,
        }],
    }
}

fn path_from_canonical_corners(corners: [[f64; 2]; 4]) -> Path {
    Path {
        contours: vec![Contour {
            vertices: corners
                .into_iter()
                .map(|[x, y]| Vertex::corner(Point { x, y }))
                .collect(),
            closed: true,
        }],
    }
}

pub(super) fn path_meshes_from_canonical_corners(
    corners: [[f64; 2]; 4],
) -> Result<(MeshData, MeshData), String> {
    tessellate_path(&path_from_canonical_corners(corners), Transform::default())
}

pub(super) fn path_stroke_from_canonical_corners(
    corners: [[f64; 2]; 4],
) -> Result<MeshData, String> {
    path_meshes_from_canonical_corners(corners).map(|(_, stroke)| stroke)
}

fn host_layer_path(layer_id: &str) -> String {
    format!("motolii/document/layers/{layer_id}/path")
}

fn host_layer_fill_path(layer_id: &str) -> String {
    format!("motolii/document/layers/{layer_id}/fill")
}

pub(super) fn host_layer_mesh_paths(layer_id: &str) -> [String; 2] {
    [host_layer_fill_path(layer_id), host_layer_path(layer_id)]
}

pub(crate) fn host_layer_id_from_entity_path(entity_path: &str) -> Option<&str> {
    let layer = entity_path
        .strip_prefix('/')
        .unwrap_or(entity_path)
        .strip_prefix("motolii/document/layers/")?;
    let (layer_id, visualizer_leaf) = layer.rsplit_once('/')?;
    (!layer_id.is_empty() && matches!(visualizer_leaf, "fill" | "path")).then_some(layer_id)
}

/// Image 未着なら fill を出す。gizmo preview 中は stale Image の上に fill を残す。
pub(crate) fn host_layer_fill_is_visible(
    corners: [[f64; 2]; 4],
    evaluated_frame_active: bool,
    previewing: bool,
) -> bool {
    if evaluated_frame_active && !previewing {
        return false;
    }
    path_meshes_from_canonical_corners(corners)
        .map(|(fill, _)| !fill.indices.is_empty() && fill.vertices != hidden_layer_mesh().vertices)
        .unwrap_or(false)
}

pub(super) fn ingest_host_layer_meshes(
    stage: &mut re_view_spatial::SpatialStage,
    layer_id: &str,
    corners: [[f64; 2]; 4],
    hide_fill: bool,
) -> bool {
    let Ok((fill, stroke)) = path_meshes_from_canonical_corners(corners) else {
        return false;
    };
    let (fill_mesh, fill_color) = if hide_fill {
        (hidden_layer_mesh(), STAGE_HOST_ERASE_COLOR)
    } else {
        (fill, DOCUMENT_RECT_FILL_COLOR)
    };
    ingest_mesh(
        stage,
        &host_layer_fill_path(layer_id),
        fill_mesh,
        fill_color,
    ) && ingest_mesh(
        stage,
        &host_layer_path(layer_id),
        stroke,
        FIXTURE_RECT_STROKE_COLOR,
    )
}

fn lyon_path(path: &Path) -> LyonPath {
    let mut builder = LyonPath::builder();
    for contour in &path.contours {
        let Some(first) = contour.vertices.first() else {
            continue;
        };
        builder.begin(point(first.point.x as f32, first.point.y as f32));
        for pair in contour.vertices.windows(2) {
            add_segment(&mut builder, pair[0], pair[1]);
        }
        if contour.closed && contour.vertices.len() > 1 {
            add_segment(
                &mut builder,
                *contour.vertices.last().unwrap_or(first),
                *first,
            );
        }
        builder.end(contour.closed);
    }
    builder.build()
}

fn add_segment(builder: &mut lyon_path::path::Builder, from: Vertex, to: Vertex) {
    let control1 = Point {
        x: from.point.x + from.out_tangent.x,
        y: from.point.y + from.out_tangent.y,
    };
    let control2 = Point {
        x: to.point.x + to.in_tangent.x,
        y: to.point.y + to.in_tangent.y,
    };
    if from.out_tangent == Point::ZERO && to.in_tangent == Point::ZERO {
        builder.line_to(point(to.point.x as f32, to.point.y as f32));
    } else {
        builder.cubic_bezier_to(
            point(control1.x as f32, control1.y as f32),
            point(control2.x as f32, control2.y as f32),
            point(to.point.x as f32, to.point.y as f32),
        );
    }
}

pub(super) fn tessellate_path(
    path: &Path,
    transform: Transform,
) -> Result<(MeshData, MeshData), String> {
    // 正準座標での誤差上限。固定頂点数ではなく曲率に応じて分割する。
    const TOLERANCE: f32 = 0.000_1;
    let path = lyon_path(path);
    let mut fill = VertexBuffers::<_, u16>::new();
    let mut stroke = VertexBuffers::<_, u16>::new();
    FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::default().with_tolerance(TOLERANCE),
            &mut simple_builder(&mut fill),
        )
        .map_err(|error| format!("tessellate path fill: {error}"))?;
    StrokeTessellator::new()
        .tessellate_path(
            &path,
            &StrokeOptions::default()
                .with_line_width(0.008)
                .with_tolerance(TOLERANCE),
            &mut simple_builder(&mut stroke),
        )
        .map_err(|error| format!("tessellate path stroke: {error}"))?;
    Ok((
        MeshData::from(fill).transformed(transform),
        MeshData::from(stroke).transformed(transform),
    ))
}

#[derive(Debug, Clone)]
pub(super) struct MeshData {
    pub(super) vertices: Vec<[f32; 3]>,
    pub(super) indices: Vec<[u32; 3]>,
}

impl From<VertexBuffers<lyon_path::math::Point, u16>> for MeshData {
    fn from(buffers: VertexBuffers<lyon_path::math::Point, u16>) -> Self {
        Self {
            vertices: buffers
                .vertices
                .into_iter()
                .map(|point| [point.x, point.y, 0.0])
                .collect(),
            indices: buffers
                .indices
                .chunks_exact(3)
                .map(|triangle| {
                    [
                        u32::from(triangle[0]),
                        u32::from(triangle[1]),
                        u32::from(triangle[2]),
                    ]
                })
                .collect(),
        }
    }
}

impl MeshData {
    pub(super) fn transformed(mut self, transform: Transform) -> Self {
        let translation = DVec3::from(transform.translation);
        let rotation = DQuat::from(transform.rotation);
        let scale = DVec3::from(transform.scale);
        for vertex in &mut self.vertices {
            let point = rotation
                * (DVec3::new(
                    f64::from(vertex[0]),
                    f64::from(vertex[1]),
                    f64::from(vertex[2]),
                ) * scale)
                + translation;
            *vertex = [point.x as f32, point.y as f32, point.z as f32];
        }
        self
    }
}

pub(super) fn hidden_layer_mesh() -> MeshData {
    MeshData {
        vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        indices: vec![[0, 1, 2]],
    }
}

pub(super) fn ingest_mesh(
    stage: &mut re_view_spatial::SpatialStage,
    entity_path: &str,
    mesh: MeshData,
    color: u32,
) -> bool {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return true;
    }
    let normals = vec![[0.0, 0.0, 1.0]; mesh.vertices.len()];
    let colors = vec![color; mesh.vertices.len()];
    let mesh = Mesh3D::new(mesh.vertices)
        .with_triangle_indices(mesh.indices)
        .with_vertex_normals(normals)
        .with_vertex_colors(colors);
    let Ok(chunk) = Chunk::builder(entity_path)
        .with_archetype_auto_row(TimePoint::default(), &mesh)
        .build()
    else {
        return false;
    };
    stage.ingest_chunk(Arc::new(chunk)).is_ok()
}

/// content を viewport に aspect-fit した NDC 矩形 `[x, y, w, h]`（Y-up、中心原点）。
pub(crate) fn aspect_fit_ndc_rect(
    content_w: f32,
    content_h: f32,
    viewport_w: f32,
    viewport_h: f32,
) -> [f32; 4] {
    let scale = (viewport_w / content_w).min(viewport_h / content_h);
    let fitted_w = content_w * scale;
    let fitted_h = content_h * scale;
    let ndc_w = 2.0 * fitted_w / viewport_w;
    let ndc_h = 2.0 * fitted_h / viewport_h;
    [-0.5 * ndc_w, -0.5 * ndc_h, ndc_w, ndc_h]
}

/// canonical (Y-up, height=1, width=aspect) → aspect-fit NDC。
pub(crate) fn canonical_to_fit_ndc(cx: f64, cy: f64, aspect: f64, fit: [f32; 4]) -> [f32; 2] {
    let u = (cx / aspect + 0.5) as f32;
    let v = (cy + 0.5) as f32;
    [fit[0] + u * fit[2], fit[1] + v * fit[3]]
}

pub(crate) fn rgba_from_u32(color: u32) -> [f32; 4] {
    let r = ((color >> 24) & 0xff) as f32 / 255.0;
    let g = ((color >> 16) & 0xff) as f32 / 255.0;
    let b = ((color >> 8) & 0xff) as f32 / 255.0;
    let a = (color & 0xff) as f32 / 255.0;
    [r, g, b, a]
}

pub(crate) fn fixture_rect_fill_rgba() -> [f32; 4] {
    rgba_from_u32(FIXTURE_RECT_FILL_COLOR)
}

pub(crate) fn fixture_rect_stroke_rgba() -> [f32; 4] {
    rgba_from_u32(FIXTURE_RECT_STROKE_COLOR)
}

/// canonical corners → fixture と同じ mesh 空間（host 投影経路）。
/// `nx = 0.5 + cx*(h/w)`, `ny = 0.5 - cy` のあと fixture の `(n - 0.5)` 写像。
/// fixture 単独経路は呼ばない（正方近似を維持）。
pub(crate) fn mesh_vertices_from_canonical_corners(
    corners: [[f64; 2]; 4],
    viewport_width: u32,
    viewport_height: u32,
) -> [[f32; 3]; 4] {
    let w = f64::from(viewport_width.max(1));
    let h = f64::from(viewport_height.max(1));
    let aspect = h / w;
    corners.map(|[cx, cy]| {
        let nx = 0.5 + cx * aspect;
        let ny = 0.5 - cy;
        [nx as f32 - 0.5, ny as f32 - 0.5, 0.0]
    })
}

pub(crate) fn apply_move_preview_to_geometry(
    geometry: &HostStageGeometry,
    preview: Option<&(String, [f64; 2])>,
) -> HostStageGeometry {
    let Some((layer_id, delta)) = preview else {
        return geometry.clone();
    };
    let mut next = geometry.clone();
    for layer in &mut next.layers {
        if layer.layer_id == *layer_id {
            for corner in &mut layer.corners {
                corner[0] += delta[0];
                corner[1] += delta[1];
            }
            layer.position[0] += delta[0];
            layer.position[1] += delta[1];
        }
    }
    next
}
