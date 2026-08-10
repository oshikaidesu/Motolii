use std::sync::Arc;
use std::{borrow::Cow, path::Path};

use rerun::external::re_renderer::external::smallvec::smallvec;
use rerun::external::re_renderer::external::wgpu;
use rerun::external::{glam, re_renderer};

use crate::path2d::TriangleMesh;

mod gpu_data {
    use rerun::external::re_renderer::{self, wgpu_buffer_types};

    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct UniformBuffer {
        pub world_from_obj: wgpu_buffer_types::Mat4,
        pub gradient_line: [f32; 4],
        pub start_color: [f32; 4],
        pub end_color: [f32; 4],
        pub picking_object_id: re_renderer::PickingLayerObjectId,
        pub picking_instance_id: re_renderer::PickingLayerInstanceId,
        pub outline_mask: wgpu_buffer_types::UVec2RowPadded,
        pub _padding: [wgpu_buffer_types::PaddingRow; 16 - 9],
    }
}

pub struct Path2DRenderer {
    bind_group_layout: re_renderer::GpuBindGroupLayoutHandle,
    color_pipeline: re_renderer::GpuRenderPipelineHandle,
    picking_pipeline: re_renderer::GpuRenderPipelineHandle,
    outline_pipeline: re_renderer::GpuRenderPipelineHandle,
}

pub struct PathConfig<'a> {
    pub world_from_obj: glam::Affine3A,
    pub mesh: &'a TriangleMesh,
    pub paint: crate::path2d::PlanarPaint,
    pub draw_order: f32,
    pub picking_object_id: re_renderer::PickingLayerObjectId,
    pub outline_mask: re_renderer::OutlineMaskPreference,
}

#[derive(Clone)]
pub struct Path2DDrawData {
    paths: Vec<PathInstance>,
}

#[derive(Clone)]
struct PathInstance {
    bind_group: re_renderer::GpuBindGroup,
    vertex_buffer: Arc<wgpu::Buffer>,
    index_buffer: Arc<wgpu::Buffer>,
    index_count: u32,
    draw_order: f32,
    has_outline: bool,
}

impl re_renderer::renderer::DrawData for Path2DDrawData {
    type Renderer = Path2DRenderer;

    fn collect_drawables(
        &self,
        _view_info: &re_renderer::renderer::DrawableCollectionViewInfo,
        collector: &mut re_renderer::DrawableCollector<'_>,
    ) {
        use re_renderer::renderer::DrawDataDrawable;

        for (index, path) in self.paths.iter().enumerate() {
            let drawable = DrawDataDrawable {
                distance_sort_key: 0.0,
                secondary_sort_key: path.draw_order,
                draw_data_payload: index as u32,
            };
            collector.add_drawable(
                re_renderer::DrawPhase::Transparent | re_renderer::DrawPhase::PickingLayer,
                drawable,
            );
            if path.has_outline {
                collector.add_drawable(re_renderer::DrawPhase::OutlineMask, drawable);
            }
        }
    }
}

impl Path2DDrawData {
    pub fn new(ctx: &re_renderer::RenderContext) -> Self {
        let _ = ctx.renderer::<Path2DRenderer>();
        Self { paths: Vec::new() }
    }

    pub fn add_path(&mut self, ctx: &re_renderer::RenderContext, config: PathConfig<'_>) {
        if config.mesh.vertices.is_empty() || config.mesh.indices.is_empty() {
            return;
        }
        let renderer = ctx.renderer::<Path2DRenderer>();
        let premultiply = |color: [f32; 4]| {
            [
                color[0] * color[3],
                color[1] * color[3],
                color[2] * color[3],
                color[3],
            ]
        };
        let uniform = gpu_data::UniformBuffer {
            world_from_obj: config.world_from_obj.into(),
            gradient_line: [
                config.paint.start[0],
                config.paint.start[1],
                config.paint.end[0],
                config.paint.end[1],
            ],
            start_color: premultiply(config.paint.start_color),
            end_color: premultiply(config.paint.end_color),
            picking_object_id: config.picking_object_id,
            picking_instance_id: re_renderer::PickingLayerInstanceId(0),
            outline_mask: config.outline_mask.0.unwrap_or_default().into(),
            _padding: Default::default(),
        };
        let uniform_entry =
            re_renderer::create_and_fill_uniform_buffer(ctx, "Path2D uniform".into(), uniform);
        let bind_group = ctx.gpu_resources.bind_groups.alloc(
            &ctx.device,
            &ctx.gpu_resources,
            &re_renderer::BindGroupDesc {
                label: "Path2D bind group".into(),
                entries: smallvec![uniform_entry],
                layout: renderer.bind_group_layout,
            },
        );

        let vertex_bytes = bytemuck::cast_slice(&config.mesh.vertices);
        let vertex_buffer = mapped_buffer(
            &ctx.device,
            "Path2D vertices",
            vertex_bytes,
            wgpu::BufferUsages::VERTEX,
        );
        let index_bytes = bytemuck::cast_slice(&config.mesh.indices);
        let index_buffer = mapped_buffer(
            &ctx.device,
            "Path2D indices",
            index_bytes,
            wgpu::BufferUsages::INDEX,
        );
        self.paths.push(PathInstance {
            bind_group,
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            index_count: config.mesh.indices.len() as u32,
            draw_order: config.draw_order,
            has_outline: config.outline_mask.is_some(),
        });
    }
}

fn mapped_buffer(
    device: &wgpu::Device,
    label: &str,
    bytes: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage,
        mapped_at_creation: true,
    });
    buffer
        .slice(..)
        .get_mapped_range_mut()
        .copy_from_slice(bytes);
    buffer.unmap();
    buffer
}

impl re_renderer::renderer::Renderer for Path2DRenderer {
    type RendererDrawData = Path2DDrawData;

    fn create_renderer(ctx: &re_renderer::RenderContext) -> Self {
        use re_renderer::FileSystem as _;

        re_renderer::get_filesystem()
            .create_file(
                Path::new("shader/path2d.wgsl"),
                Cow::Borrowed(include_str!("../shader/path2d.wgsl")),
            )
            .expect("register embedded Path2D shader");
        let shader = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &re_renderer::include_shader_module!("../shader/path2d.wgsl"),
        );
        let bind_group_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            &ctx.device,
            &re_renderer::BindGroupLayoutDesc {
                label: "Path2DRenderer bindings".into(),
                entries: vec![wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                            gpu_data::UniformBuffer,
                        >()
                            as u64),
                    },
                    count: None,
                }],
            },
        );
        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &re_renderer::PipelineLayoutDesc {
                label: "Path2DRenderer".into(),
                entries: vec![ctx.global_bindings.layout, bind_group_layout],
            },
        );
        let base = re_renderer::RenderPipelineDesc {
            label: "Path2DRenderer color".into(),
            pipeline_layout,
            vertex_entrypoint: "vs_main".into(),
            vertex_handle: shader,
            fragment_entrypoint: "fs_main".into(),
            fragment_handle: shader,
            vertex_buffers: re_renderer::VertexBufferLayout::from_formats(
                [wgpu::VertexFormat::Float32x2].into_iter(),
            ),
            render_targets: smallvec![Some(wgpu::ColorTargetState {
                format: re_renderer::ViewBuilder::MAIN_TARGET_COLOR_FORMAT,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(re_renderer::ViewBuilder::MAIN_TARGET_DEFAULT_DEPTH_STATE_NO_WRITE),
            multisample: re_renderer::ViewBuilder::main_target_default_msaa_state(
                ctx.render_config(),
                false,
            ),
        };
        let pipelines = &ctx.gpu_resources.render_pipelines;
        let color_pipeline = pipelines.get_or_create(ctx, &base);
        let picking_pipeline = pipelines.get_or_create(
            ctx,
            &re_renderer::RenderPipelineDesc {
                label: "Path2DRenderer picking".into(),
                fragment_entrypoint: "fs_main_picking_layer".into(),
                render_targets: smallvec![Some(
                    re_renderer::PickingLayerProcessor::PICKING_LAYER_FORMAT.into(),
                )],
                depth_stencil: re_renderer::PickingLayerProcessor::PICKING_LAYER_DEPTH_STATE,
                multisample: re_renderer::PickingLayerProcessor::PICKING_LAYER_MSAA_STATE,
                ..base.clone()
            },
        );
        let outline_pipeline = pipelines.get_or_create(
            ctx,
            &re_renderer::RenderPipelineDesc {
                label: "Path2DRenderer outline".into(),
                fragment_entrypoint: "fs_main_outline_mask".into(),
                render_targets: smallvec![Some(
                    re_renderer::OutlineMaskProcessor::MASK_FORMAT.into()
                )],
                depth_stencil: re_renderer::OutlineMaskProcessor::MASK_DEPTH_STATE,
                ..base
            },
        );
        Self {
            bind_group_layout,
            color_pipeline,
            picking_pipeline,
            outline_pipeline,
        }
    }

    fn draw(
        &self,
        pipelines: &re_renderer::GpuRenderPipelinePoolAccessor<'_>,
        phase: re_renderer::DrawPhase,
        pass: &mut wgpu::RenderPass<'_>,
        instructions: &[re_renderer::renderer::DrawInstruction<'_, Path2DDrawData>],
    ) -> Result<(), re_renderer::renderer::DrawError> {
        let handle = match phase {
            re_renderer::DrawPhase::Transparent => self.color_pipeline,
            re_renderer::DrawPhase::PickingLayer => self.picking_pipeline,
            re_renderer::DrawPhase::OutlineMask => self.outline_pipeline,
            _ => unreachable!("Path2D subscribed to unexpected phase {phase:?}"),
        };
        pass.set_pipeline(pipelines.get(handle)?);
        for instruction in instructions {
            for drawable in instruction.drawables {
                let Some(path) = instruction
                    .draw_data
                    .paths
                    .get(drawable.draw_data_payload as usize)
                else {
                    continue;
                };
                pass.set_bind_group(1, &*path.bind_group, &[]);
                pass.set_vertex_buffer(0, path.vertex_buffer.slice(..));
                pass.set_index_buffer(path.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..path.index_count, 0, 0..1);
            }
        }
        Ok(())
    }
}
