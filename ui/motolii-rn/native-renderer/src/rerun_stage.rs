use std::{sync::Arc, time::Instant};

use egui::{
    Event, PointerButton, Pos2, RawInput, Rect, Vec2,
    epaint::{Mesh, Vertex as EguiVertex},
};
use glam::EulerRot;
use lyon_path::{Path as LyonPath, math::point};
use lyon_tessellation::{
    FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
    geometry_builder::{VertexBuffers, simple_builder},
};
use motolii_doc::{
    CompositeOrder, LineJoin, PointType, TrimMode,
    pathgeom::{self, Contour, Path, Point, ResolvedPathOp, ResolvedTransform, Vertex},
};
use re_chunk::Chunk;
use re_log_types::TimePoint;
use re_sdk_types::archetypes::Mesh3D;
use transform_gizmo::{
    Gizmo, GizmoConfig, GizmoInteraction, GizmoMode, GizmoOrientation,
    math::{DMat4, DQuat, DVec3, Pos2 as GizmoPos2, Rect as GizmoRect, Transform},
};

use crate::host_bridge::HostStageGeometry;
use crate::renderer_core::PointerPhase;

const FIXTURE_RECT_FILL_COLOR: u32 = 0xE9_8C_6AFF;
const FIXTURE_RECT_STROKE_COLOR: u32 = 0xEC_D8_FFFF;
const STAGE_HOST_ERASE_COLOR: u32 = 0x0000_0000;

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub(crate) struct EmbeddedSpatialStage {
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    spatial_stage: re_view_spatial::SpatialStage,
    input_events: Vec<Event>,
    started_at: Instant,
    gizmo: Gizmo,
    fixture_transform: Transform,
    fixture_item_id: String,
    /// Host `stage_geometry` 適用済みなら fixture 再ingestを止める。
    host_geometry_active: bool,
    host_layer_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StageTransformProjection {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) rotation_x: f64,
    pub(crate) rotation_y: f64,
    pub(crate) rotation_z: f64,
}

impl EmbeddedSpatialStage {
    pub(crate) fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let mut egui_renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );
        let render_ctx = re_renderer::RenderContext::new(
            adapter,
            device.clone(),
            queue.clone(),
            surface_format,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .map_err(|error| format!("create Rerun render context: {error}"))?;
        egui_renderer.callback_resources.insert(render_ctx);

        let mut stage = Self {
            egui_ctx: egui::Context::default(),
            egui_renderer,
            spatial_stage: re_view_spatial::SpatialStage::new(re_log_types::ApplicationId::from(
                "motolii-rn-stage",
            ))
            .map_err(|error| format!("create Rerun spatial stage: {error}"))?,
            input_events: Vec::new(),
            started_at: Instant::now(),
            gizmo: Gizmo::default(),
            fixture_transform: Transform::default(),
            fixture_item_id: "rectangle@0.500000,0.500000|pucker-bloat".into(),
            host_geometry_active: false,
            host_layer_ids: Vec::new(),
        };
        if !stage.set_created_item("rectangle@0.500000,0.500000|pucker-bloat") {
            return Err("seed path rectangle for embedded stage".into());
        }
        Ok(stage)
    }

    pub(crate) fn pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        let position = Pos2::new(x as f32, y as f32);
        self.input_events.push(Event::PointerMoved(position));

        match phase {
            PointerPhase::Down => self.input_events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }),
            PointerPhase::Up => self.input_events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }),
            PointerPhase::Cancel => self.input_events.push(Event::PointerGone),
            PointerPhase::Move => {}
        }
    }

    /// 一時gizmo値をInspectorへ投影する。Document値ではない。
    pub(crate) fn transform_projection(&self) -> StageTransformProjection {
        let (rotation_x, rotation_y, rotation_z) =
            DQuat::from(self.fixture_transform.rotation).to_euler(EulerRot::XYZ);
        let translation = DVec3::from(self.fixture_transform.translation);
        StageTransformProjection {
            x: translation.x,
            y: translation.y,
            z: translation.z,
            rotation_x: rotation_x.to_degrees(),
            rotation_y: rotation_y.to_degrees(),
            rotation_z: rotation_z.to_degrees(),
        }
    }

    /// Inspectorとgizmoが共有する一時値。Documentには書き込まない。
    pub(crate) fn set_transform_projection(&mut self, projection: StageTransformProjection) -> bool {
        let transform = Transform::from_scale_rotation_translation(
            DVec3::ONE,
            DQuat::from_euler(
                EulerRot::XYZ,
                projection.rotation_x.to_radians(),
                projection.rotation_y.to_radians(),
                projection.rotation_z.to_radians(),
            ),
            DVec3::new(projection.x, projection.y, projection.z),
        );
        self.fixture_transform = transform;
        let item_id = self.fixture_item_id.clone();
        self.set_created_item(&item_id)
    }

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
        if self.host_geometry_active {
            // host 投影が正本の間は fixture 文字列由来の rect を戻さない。
            return true;
        }
        if item_id.is_empty() {
            return true;
        }
        let Some((item, path_operation_id)) = item_id.split_once('|') else {
            return false;
        };
        let Some((kind, coordinates)) = item.split_once('@') else {
            return false;
        };
        if kind != "rectangle" {
            return false;
        }
        let Some((x, y)) = coordinates.split_once(',') else {
            return false;
        };
        let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
            return false;
        };
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return false;
        }

        self.fixture_item_id.clear();
        self.fixture_item_id.push_str(item_id);

        let path = rectangle_path(x - 0.5, y - 0.5);
        let Some(path_operation) = preview_path_operation(path_operation_id) else {
            return false;
        };
        let Ok(path) = pathgeom::apply(&path, &path_operation, 0.0) else {
            return false;
        };
        let Ok((fill, stroke)) = tessellate_path(&path, self.fixture_transform) else {
            return false;
        };

        ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/fill",
            fill,
            FIXTURE_RECT_FILL_COLOR,
        ) && ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/stroke",
            stroke,
            FIXTURE_RECT_STROKE_COLOR,
        )
    }

    pub(crate) fn clear_host_projection(&mut self) -> bool {
        if !self.host_geometry_active {
            return true;
        }
        let host_layers = std::mem::take(&mut self.host_layer_ids);
        for old_id in &host_layers {
            if !ingest_mesh(
                &mut self.spatial_stage,
                old_id,
                MeshData {
                    vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
                    indices: vec![[0, 1, 2]],
                },
                STAGE_HOST_ERASE_COLOR,
            ) {
                self.host_layer_ids = host_layers;
                return false;
            }
        }
        self.host_geometry_active = false;
        let item_id = self.fixture_item_id.clone();
        self.set_created_item(&item_id)
    }

    /// Host snapshot の stage_geometry で fixture を置換する。revision 側で gate 済み前提。
    /// `viewport_width/height` は host 投影の aspect 写像に使う（正方近似を避ける）。
    pub(crate) fn apply_host_stage_geometry(
        &mut self,
        geometry: &HostStageGeometry,
        viewport_width: u32,
        viewport_height: u32,
    ) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }
        // fixture entity を透明メッシュで上書きし、host layer だけを見せる。
        let cleared = MeshData {
            vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            indices: vec![[0, 1, 2]],
        };
        if !ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/fill",
            cleared.clone(),
            STAGE_HOST_ERASE_COLOR,
        ) || !ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/stroke",
            cleared,
            STAGE_HOST_ERASE_COLOR,
        ) {
            return false;
        }

        let next_ids: Vec<String> = geometry.layers.iter().map(|l| l.layer_id.clone()).collect();
        for old_id in &self.host_layer_ids {
            if next_ids.iter().any(|id| id == old_id) {
                continue;
            }
            let _ = ingest_mesh(
                &mut self.spatial_stage,
                old_id,
                MeshData {
                    vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
                    indices: vec![[0, 1, 2]],
                },
                STAGE_HOST_ERASE_COLOR,
            );
        }

        for layer in &geometry.layers {
            let mesh = mesh_from_canonical_corners(layer.corners, viewport_width, viewport_height);
            if !ingest_mesh(
                &mut self.spatial_stage,
                &layer.layer_id,
                mesh,
                FIXTURE_RECT_FILL_COLOR,
            ) {
                return false;
            }
        }
        self.host_layer_ids = next_ids;
        self.host_geometry_active = true;
        true
    }

    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        // Rerunのcallback command bufferより先にsurfaceを初期化する。
        // 後からClearすると、Rerunが同じsurfaceへ描いたMeshまで消してしまう。
        let mut clear_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Spatial Stage clear"),
        });
        {
            let _clear_pass = clear_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Rerun Spatial Stage clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.041,
                            b: 0.050,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        queue.submit([clear_encoder.finish()]);

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(width as f32, height as f32),
            )),
            time: Some(self.started_at.elapsed().as_secs_f64()),
            events: std::mem::take(&mut self.input_events),
            ..Default::default()
        };

        let egui_ctx = self.egui_ctx.clone();
        let mut stage_error = None;
        let full_output = egui_ctx.run_ui(raw_input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let mut render_ctx = self
                        .egui_renderer
                        .callback_resources
                        .remove::<re_renderer::RenderContext>()
                        .expect("Rerun render context is registered for the native Stage");
                    let result = self.spatial_stage.show(ui, &mut render_ctx);
                    self.egui_renderer.callback_resources.insert(render_ctx);
                    if let Err(error) = result {
                        stage_error = Some(error.to_string());
                    }
                    self.show_performance_gizmo(ui);
                });
        });
        if let Some(error) = stage_error {
            return Err(format!("run Rerun spatial stage: {error}"));
        }

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, delta);
        }
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Spatial Stage"),
        });
        let callback_command_buffers = self.egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Rerun Spatial Stage present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }
        queue.submit(
            callback_command_buffers
                .into_iter()
                .chain([encoder.finish()]),
        );

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        Ok(())
    }
}

impl EmbeddedSpatialStage {
    /// 性能評価用の一時3D transform。Document、D2、Undoには接続しない。
    fn show_performance_gizmo(&mut self, ui: &egui::Ui) {
        let viewport = ui.clip_rect();
        if viewport.width() <= 0.0 || viewport.height() <= 0.0 {
            return;
        }

        // Rerun embedded stageと同じ正面透視camera（z=0平面の高さは1.0）。
        let fov_y = 55.0_f64.to_radians();
        let distance = 0.5 / (fov_y * 0.5).tan();
        let view_matrix = DMat4::look_at_rh(DVec3::new(0.0, 0.0, distance), DVec3::ZERO, DVec3::Y);
        let projection_matrix = DMat4::perspective_infinite_rh(
            fov_y,
            f64::from(viewport.width() / viewport.height()),
            0.01,
        );
        self.gizmo.update_config(GizmoConfig {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            viewport: GizmoRect::from_min_max(
                GizmoPos2::new(viewport.min.x, viewport.min.y),
                GizmoPos2::new(viewport.max.x, viewport.max.y),
            ),
            modes: GizmoMode::all_translate() | GizmoMode::all_rotate(),
            orientation: GizmoOrientation::Global,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..Default::default()
        });

        let (cursor_pos, drag_started, dragging) = ui.input(|input| {
            (
                input.pointer.hover_pos().unwrap_or_default(),
                input.pointer.button_pressed(PointerButton::Primary),
                input.pointer.button_down(PointerButton::Primary),
            )
        });
        let interaction = GizmoInteraction {
            cursor_pos: (cursor_pos.x, cursor_pos.y),
            hovered: viewport.contains(cursor_pos),
            drag_started,
            dragging,
        };
        if let Some((_result, transforms)) =
            self.gizmo.update(interaction, &[self.fixture_transform])
            && let Some(transform) = transforms.first().copied()
        {
            self.fixture_transform = transform;
            let item_id = self.fixture_item_id.clone();
            let _ = self.set_created_item(&item_id);
        }

        let draw_data = self.gizmo.draw();
        ui.painter().add(Mesh {
            indices: draw_data.indices,
            vertices: draw_data
                .vertices
                .into_iter()
                .zip(draw_data.colors)
                .map(|(position, [r, g, b, a])| EguiVertex {
                    pos: Pos2::new(position[0], position[1]),
                    uv: Pos2::ZERO,
                    color: egui::Rgba::from_rgba_premultiplied(r, g, b, a).into(),
                })
                .collect(),
            ..Default::default()
        });
    }
}

/// Preview fixtureだけの既定値。DocumentのPathOp評価・保存値をここへ持ち込まない。
fn preview_path_operation(id: &str) -> Option<ResolvedPathOp> {
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

fn rectangle_path(center_x: f32, center_y: f32) -> Path {
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

fn tessellate_path(path: &Path, transform: Transform) -> Result<(MeshData, MeshData), String> {
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
struct MeshData {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
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
    fn transformed(mut self, transform: Transform) -> Self {
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

fn ingest_mesh(
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

fn mesh_from_canonical_corners(
    corners: [[f64; 2]; 4],
    viewport_width: u32,
    viewport_height: u32,
) -> MeshData {
    let vertices = mesh_vertices_from_canonical_corners(corners, viewport_width, viewport_height);
    MeshData {
        vertices: vertices.to_vec(),
        indices: vec![[0, 1, 2], [0, 2, 3]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pucker_preview_is_tessellated_as_curved_fill_and_stroke() {
        let path = pathgeom::apply(
            &rectangle_path(0.0, 0.0),
            &preview_path_operation("pucker-bloat").expect("fixture operation"),
            0.0,
        )
        .expect("fixture operation evaluates");
        let (fill, stroke) =
            tessellate_path(&path, Transform::default()).expect("Bezier path tessellates");

        assert!(
            fill.vertices.len() > 4,
            "curve uses adaptive vertices, not a rectangle fan"
        );
        assert!(!fill.indices.is_empty());
        assert!(
            stroke.vertices.len() > 8,
            "stroke follows the same curved path"
        );
    }

    #[test]
    fn every_visible_path_operation_has_a_preview_evaluation() {
        for id in [
            "pucker-bloat",
            "zig-zag",
            "offset",
            "round-corners",
            "trim",
            "twist",
            "wiggle",
            "repeater",
        ] {
            let path = pathgeom::apply(
                &rectangle_path(0.0, 0.0),
                &preview_path_operation(id).expect("known path operation"),
                0.0,
            )
            .expect("fixture operation evaluates");
            let (_, stroke) =
                tessellate_path(&path, Transform::default()).expect("evaluated path tessellates");
            assert!(
                !stroke.indices.is_empty(),
                "{id} has a visible Stage outline"
            );
        }
    }

    #[test]
    fn performance_gizmo_transform_moves_and_rotates_fixture_vertices() {
        let mesh = MeshData {
            vertices: vec![[1.0, 0.0, 0.0]],
            indices: vec![],
        }
        .transformed(Transform::from_scale_rotation_translation(
            DVec3::ONE,
            DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2),
            DVec3::new(0.5, -0.25, 0.75),
        ));

        let [x, y, z] = mesh.vertices[0];
        assert!((x - 0.5).abs() < 0.000_1);
        assert!((y - 0.75).abs() < 0.000_1);
        assert!((z - 0.75).abs() < 0.000_1);
    }

    #[test]
    fn canonical_corners_map_to_fixture_mesh_space() {
        // seed unit rect: center(0,0) size(1,1) / 正方 viewport では旧写像と一致
        let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
        let verts = mesh_vertices_from_canonical_corners(corners, 1, 1);
        // (nx,ny)=(cx+0.5, 0.5-cy) → (n-0.5) = (cx, -cy)
        assert_eq!(verts[0], [-0.5, 0.5, 0.0]);
        assert_eq!(verts[1], [0.5, 0.5, 0.0]);
        assert_eq!(verts[2], [0.5, -0.5, 0.0]);
        assert_eq!(verts[3], [-0.5, -0.5, 0.0]);
    }

    #[test]
    fn canonical_to_normalized_uses_viewport_aspect() {
        // w=2h → h/w=0.5。cx=0.5 → nx=0.5+0.25=0.75
        let corners = [[0.5, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let verts = mesh_vertices_from_canonical_corners(corners, 2, 1);
        let nx = verts[0][0] as f64 + 0.5;
        assert!((nx - 0.75).abs() < 1e-9);
    }
}
