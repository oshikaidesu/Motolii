use std::{sync::Arc, time::Instant};

use egui::{Event, PointerButton, Pos2, RawInput, Rect, Vec2};
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

use crate::renderer_core::PointerPhase;

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub(crate) struct EmbeddedSpatialStage {
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    spatial_stage: re_view_spatial::SpatialStage,
    input_events: Vec<Event>,
    started_at: Instant,
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

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
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

        let path = rectangle_path(x - 0.5, y - 0.5);
        let Some(path_operation) = preview_path_operation(path_operation_id) else {
            return false;
        };
        let Ok(path) = pathgeom::apply(&path, &path_operation, 0.0) else {
            return false;
        };
        let Ok((fill, stroke)) = tessellate_path(&path) else {
            return false;
        };

        ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/fill",
            fill,
            0xE9_8C_6AFF,
        ) && ingest_mesh(
            &mut self.spatial_stage,
            "motolii/fixtures/path-rectangle/stroke",
            stroke,
            0xEC_D8_FFFF,
        )
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

fn tessellate_path(path: &Path) -> Result<(MeshData, MeshData), String> {
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
    Ok((MeshData::from(fill), MeshData::from(stroke)))
}

#[derive(Debug)]
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
        let (fill, stroke) = tessellate_path(&path).expect("Bezier path tessellates");

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
            let (_, stroke) = tessellate_path(&path).expect("evaluated path tessellates");
            assert!(
                !stroke.indices.is_empty(),
                "{id} has a visible Stage outline"
            );
        }
    }
}
