
use std::path::PathBuf;

use re_renderer::{GpuRenderPipelineHandle, PipelineLayoutDesc, RenderContext, RenderPipelineDesc, ShaderModuleDesc};
#[cfg(load_shaders_from_disk)]
use re_renderer::{FileServer, new_recommended_file_resolver};
#[cfg(not(load_shaders_from_disk))]
use re_renderer::{FileSystem as _, get_filesystem};

pub(crate) const GRADIENT_SOURCE: &str = include_str!("shaders/gradient.wgsl");

pub(crate) const GRADIENT_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) const TRI_LED_SOURCE: &str = include_str!("shaders/tri_led.wgsl");

pub(crate) const TRI_LED_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) struct WgslFragmentProgram {
    pipeline: GpuRenderPipelineHandle,
}

impl WgslFragmentProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        name: &str,
        #[cfg_attr(load_shaders_from_disk, allow(unused_variables))] wgsl_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        #[cfg(load_shaders_from_disk)]
        let path = {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let abs_path = manifest_dir.join(format!("src/effects/shaders/{name}.wgsl"));
            let resolver = new_recommended_file_resolver();
            FileServer::get_mut(|fs| fs.watch(&resolver, &abs_path, false))
                .expect("{name}.wgsl exists next to wgsl_fragment.rs")
        };
        #[cfg(not(load_shaders_from_disk))]
        let path = {
            let path = PathBuf::from(format!("motolii-compositor/wgsl-fragment/{name}.wgsl"));
            get_filesystem()
                .create_file(&path, wgsl_source.to_owned().into())
                .expect("wgsl fragment source is valid utf8");
            path
        };

        let shader_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}").into(),
                source: path,
                extra_workaround_replacements: Vec::new(),
            },
        );

        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &PipelineLayoutDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}-pipeline-layout").into(),
                entries: vec![],
            },
        );

        let pipeline = ctx.gpu_resources.render_pipelines.get_or_create(
            ctx,
            &RenderPipelineDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}-pipeline").into(),
                pipeline_layout,
                vertex_entrypoint: "vs_main".to_owned(),
                vertex_handle: shader_handle,
                fragment_entrypoint: "fs_main".to_owned(),
                fragment_handle: shader_handle,
                vertex_buffers: Default::default(),
                render_targets: re_renderer::external::smallvec::smallvec![Some(
                    wgpu::ColorTargetState {
                        format: output_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }
                )],
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            },
        );

        Self { pipeline }
    }

    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        dst_view: &wgpu::TextureView,
    ) {
        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();
        let pipeline = render_pipelines
            .get(self.pipeline)
            .expect("wgsl fragment pipeline");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-wgsl-fragment-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
    }
}
