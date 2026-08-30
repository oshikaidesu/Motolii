
use std::path::PathBuf;

use re_renderer::{
    BindGroupLayoutDesc, FileSystem as _, GpuBindGroupLayoutHandle, GpuRenderPipelineHandle,
    PipelineLayoutDesc, RenderContext, RenderPipelineDesc, ShaderModuleDesc, get_filesystem,
};

pub(crate) const BLOOM_SOURCE: &str = include_str!("../../../../../vism/bloom.fs");

pub(crate) const ISF_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Debug, thiserror::Error)]
pub(crate) enum IsfError {
    #[error("ISF ファイルに `/*{{ ... }}*/` の JSON ヘッダが見つからない")]
    MissingHeader,
    #[error("ISF ヘッダの JSON を読めない: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("naga が GLSL を解析できない({stage:?}): {detail}")]
    GlslParse {
        stage: naga::ShaderStage,
        detail: String,
    },
    #[error("naga が生成した Module を検証できない: {0}")]
    Validate(String),
    #[error("naga が WGSL を書き出せない: {0}")]
    WgslWrite(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsfInputType {
    Image,
    Float,
    Bool,
    Point2D,
    Color,
}

impl IsfInputType {
    fn from_isf_name(name: &str) -> Option<Self> {
        match name {
            "image" => Some(Self::Image),
            "float" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "point2D" => Some(Self::Point2D),
            "color" => Some(Self::Color),
            _ => None,
        }
    }

    pub fn component_count(self) -> usize {
        match self {
            Self::Image => 0,
            Self::Float | Self::Bool => 1,
            Self::Point2D => 2,
            Self::Color => 4,
        }
    }

    fn glsl_uniform_type(self) -> &'static str {
        match self {
            Self::Image => "sampler2D",
            Self::Float | Self::Bool => "float",
            Self::Point2D => "vec2",
            Self::Color => "vec4",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IsfInput {
    pub name: String,
    pub ty: IsfInputType,
    pub default: [f32; 4],
    pub min: Option<[f32; 4]>,
    pub max: Option<[f32; 4]>,
    pub maps: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default)]
pub struct IsfManifest {
    pub description: Option<String>,
    pub inputs: Vec<IsfInput>,
}

impl IsfManifest {
    pub fn image_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty == IsfInputType::Image)
    }

    pub fn param_inputs(&self) -> impl Iterator<Item = &IsfInput> {
        self.inputs.iter().filter(|input| input.ty != IsfInputType::Image)
    }
}

fn assign_bindings(manifest: &IsfManifest) -> (Vec<usize>, Vec<usize>) {
    let mut images = Vec::new();
    let mut params = Vec::new();
    for (index, input) in manifest.inputs.iter().enumerate() {
        if input.ty == IsfInputType::Image {
            images.push(index);
        } else {
            params.push(index);
        }
    }
    (images, params)
}

fn image_texture_binding(image_index_in_order: usize) -> u32 {
    (image_index_in_order * 2) as u32
}

fn render_size_binding(param_order: &[usize]) -> u32 {
    param_order.len() as u32
}

pub(crate) fn parse_isf_source(source: &str) -> Result<(IsfManifest, String), IsfError> {
    let trimmed = source.trim_start();
    if !trimmed.starts_with("/*") {
        return Err(IsfError::MissingHeader);
    }
    let header_end = trimmed.find("*/").ok_or(IsfError::MissingHeader)?;
    let json_text = &trimmed[2..header_end];
    let body = trimmed[header_end + 2..].to_owned();

    let value: serde_json::Value = serde_json::from_str(json_text)?;
    let description = value
        .get("DESCRIPTION")
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let mut inputs = Vec::new();
    if let Some(array) = value.get("INPUTS").and_then(|v| v.as_array()) {
        for entry in array {
            let Some(name) = entry.get("NAME").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(ty) = entry
                .get("TYPE")
                .and_then(|v| v.as_str())
                .and_then(IsfInputType::from_isf_name)
            else {
                continue;
            };
            let default = read_components(entry.get("DEFAULT"));
            let min = entry.get("MIN").map(|v| read_components(Some(v)));
            let max = entry.get("MAX").map(|v| read_components(Some(v)));
            let maps = entry.get("MAPS").cloned();
            inputs.push(IsfInput {
                name: name.to_owned(),
                ty,
                default,
                min,
                max,
                maps,
            });
        }
    }
    Ok((IsfManifest { description, inputs }, body))
}

fn read_components(value: Option<&serde_json::Value>) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    let Some(value) = value else { return out };
    match value {
        serde_json::Value::Array(items) => {
            for (slot, item) in out.iter_mut().zip(items.iter()) {
                *slot = item.as_f64().unwrap_or(0.0) as f32;
            }
        }
        serde_json::Value::Number(n) => out[0] = n.as_f64().unwrap_or(0.0) as f32,
        serde_json::Value::Bool(b) => out[0] = if *b { 1.0 } else { 0.0 },
        _ => {}
    }
    out
}

const VERTEX_SOURCE: &str = r#"#version 450 core

layout(location = 0) out vec2 isf_FragNormCoord;

void main() {
    vec2 positions[3] = vec2[3](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 pos = positions[gl_VertexIndex];
    gl_Position = vec4(pos, 0.0, 1.0);
    isf_FragNormCoord = pos * 0.5 + 0.5;
}
"#;

fn wrap_fragment_source(
    manifest: &IsfManifest,
    image_order: &[usize],
    param_order: &[usize],
    filter_body: &str,
) -> String {
    let mut out = String::new();
    out.push_str("#version 450 core\n\n");
    out.push_str("layout(location = 0) in vec2 isf_FragNormCoord;\n");
    out.push_str("layout(location = 0) out vec4 gl_FragColor;\n\n");

    for (order_index, &index) in image_order.iter().enumerate() {
        let input = &manifest.inputs[index];
        let tex_binding = image_texture_binding(order_index);
        let samp_binding = tex_binding + 1;
        out.push_str(&format!(
            "layout(set = 0, binding = {tex_binding}) uniform texture2D {name}__tex;\n",
            name = input.name
        ));
        out.push_str(&format!(
            "layout(set = 0, binding = {samp_binding}) uniform sampler {name}__samp;\n",
            name = input.name
        ));
        out.push_str(&format!(
            "#define {name} sampler2D({name}__tex, {name}__samp)\n",
            name = input.name
        ));
    }
    out.push('\n');
    for (binding, &index) in param_order.iter().enumerate() {
        let input = &manifest.inputs[index];
        let glsl_ty = input.ty.glsl_uniform_type();
        out.push_str(&format!(
            "layout(set = 1, binding = {binding}) uniform Param_{name} {{ {glsl_ty} {name}; }};\n",
            name = input.name
        ));
    }
    out.push_str(&format!(
        "layout(set = 1, binding = {binding}) uniform RenderInfo {{ vec2 RENDERSIZE; }};\n\n",
        binding = render_size_binding(param_order)
    ));

    out.push_str("#define IMG_THIS_PIXEL(image) texture(image, isf_FragNormCoord)\n");
    out.push_str("#define IMG_NORM_PIXEL(image, coord) texture(image, coord)\n\n");

    out.push_str(filter_body);
    out
}

fn compile_glsl_to_wgsl(source: &str, stage: naga::ShaderStage) -> Result<String, IsfError> {
    let options = naga::front::glsl::Options::from(stage);
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&options, source)
        .map_err(|errors| IsfError::GlslParse {
            stage,
            detail: errors.to_string(),
        })?;
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| IsfError::Validate(e.to_string()))?;
    naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
        .map_err(|e| IsfError::WgslWrite(e.to_string()))
}

pub(crate) struct IsfProgram {
    manifest: IsfManifest,
    pipeline: GpuRenderPipelineHandle,
    texture_layout: GpuBindGroupLayoutHandle,
    params_layout: GpuBindGroupLayoutHandle,
    sampler: wgpu::Sampler,
    image_order: Vec<usize>,
    param_order: Vec<usize>,
}

impl IsfProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        isf_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Result<Self, IsfError> {
        let device = &ctx.device;
        let (manifest, filter_body) = parse_isf_source(isf_source)?;
        let (image_order, param_order) = assign_bindings(&manifest);

        let fragment_glsl = wrap_fragment_source(&manifest, &image_order, &param_order, &filter_body);
        let fragment_wgsl = compile_glsl_to_wgsl(&fragment_glsl, naga::ShaderStage::Fragment)?;
        let vertex_wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)?;

        #[cfg(load_shaders_from_disk)]
        let (vertex_path, fragment_path) = {
            let dir = std::env::temp_dir().join("motolii-isf-wgsl");
            std::fs::create_dir_all(&dir).map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            let vertex_path = dir.join("vertex.wgsl");
            let fragment_path = dir.join("fragment.wgsl");
            std::fs::write(&vertex_path, vertex_wgsl.as_bytes())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            std::fs::write(&fragment_path, fragment_wgsl.as_bytes())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            (vertex_path, fragment_path)
        };
        #[cfg(not(load_shaders_from_disk))]
        let (vertex_path, fragment_path) = {
            let vertex_path = PathBuf::from("motolii-compositor/isf/vertex.wgsl");
            let fragment_path = PathBuf::from("motolii-compositor/isf/fragment.wgsl");
            get_filesystem()
                .create_file(&vertex_path, vertex_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            get_filesystem()
                .create_file(&fragment_path, fragment_wgsl.into())
                .map_err(|e| IsfError::WgslWrite(e.to_string()))?;
            (vertex_path, fragment_path)
        };

        let vertex_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: "motolii-compositor-isf-vertex".into(),
                source: vertex_path,
                extra_workaround_replacements: Vec::new(),
            },
        );
        let fragment_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: "motolii-compositor-isf-fragment".into(),
                source: fragment_path,
                extra_workaround_replacements: Vec::new(),
            },
        );

        let mut texture_entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::with_capacity(image_order.len() * 2);
        for order_index in 0..image_order.len() {
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let texture_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: "motolii-compositor-isf-texture-layout".into(),
                entries: texture_entries,
            },
        );

        let mut param_entries: Vec<wgpu::BindGroupLayoutEntry> = (0..param_order.len() as u32)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        param_entries.push(wgpu::BindGroupLayoutEntry {
            binding: render_size_binding(&param_order),
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        let params_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: "motolii-compositor-isf-params-layout".into(),
                entries: param_entries,
            },
        );

        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &PipelineLayoutDesc {
                label: "motolii-compositor-isf-pipeline-layout".into(),
                entries: vec![texture_layout, params_layout],
            },
        );

        let pipeline = ctx.gpu_resources.render_pipelines.get_or_create(
            ctx,
            &RenderPipelineDesc {
                label: "motolii-compositor-isf-pipeline".into(),
                pipeline_layout,
                vertex_entrypoint: "main".to_owned(),
                vertex_handle,
                fragment_entrypoint: "main".to_owned(),
                fragment_handle,
                vertex_buffers: Default::default(),
                render_targets: re_renderer::external::smallvec::smallvec![Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            },
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-compositor-isf-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            manifest,
            pipeline,
            texture_layout,
            params_layout,
            sampler,
            image_order,
            param_order,
        })
    }

    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        let device = &ctx.device;
        let queue = &ctx.queue;
        let bind_group_layouts = ctx.gpu_resources.bind_group_layouts.resources();
        let texture_layout = bind_group_layouts
            .get(self.texture_layout)
            .expect("isf texture bind group layout");
        let params_layout = bind_group_layouts
            .get(self.params_layout)
            .expect("isf params bind group layout");

        let mut texture_entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.image_order.len() * 2);
        for order_index in 0..self.image_order.len() {
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding,
                resource: wgpu::BindingResource::TextureView(source_view),
            });
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding + 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            });
        }
        let texture_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-isf-texture-bind"),
            layout: texture_layout,
            entries: &texture_entries,
        });

        let mut buffers: Vec<wgpu::Buffer> = Vec::with_capacity(self.param_order.len() + 1);
        for &index in &self.param_order {
            let input = &self.manifest.inputs[index];
            let count = input.ty.component_count().max(1);
            let mut components = input.default;
            if let Some((_, value)) = params.iter().find(|(name, _)| name == &input.name) {
                components[0] = *value;
            }
            let mut bytes = vec![0u8; count * 4];
            for i in 0..count {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&components[i].to_le_bytes());
            }
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("motolii-compositor-isf-param"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, &bytes);
            buffers.push(buffer);
        }
        let render_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-compositor-isf-render-info"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut render_info_bytes = [0u8; 8];
        render_info_bytes[0..4].copy_from_slice(&render_size[0].to_le_bytes());
        render_info_bytes[4..8].copy_from_slice(&render_size[1].to_le_bytes());
        queue.write_buffer(&render_info_buffer, 0, &render_info_bytes);
        buffers.push(render_info_buffer);

        let param_entries: Vec<wgpu::BindGroupEntry> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: buffer.as_entire_binding(),
            })
            .collect();
        let params_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-compositor-isf-params-bind"),
            layout: params_layout,
            entries: &param_entries,
        });

        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();
        let pipeline = render_pipelines
            .get(self.pipeline)
            .expect("isf render pipeline");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-isf-pass"),
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
        pass.set_bind_group(0, &texture_bind, &[]);
        pass.set_bind_group(1, &params_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_isf_header_and_body_generically() {
        let (manifest, body) = parse_isf_source(BLOOM_SOURCE).expect("parse");
        assert_eq!(manifest.inputs.len(), 4);
        assert_eq!(manifest.image_inputs().count(), 1);
        assert_eq!(manifest.param_inputs().count(), 3);

        let threshold = manifest
            .inputs
            .iter()
            .find(|input| input.name == "threshold")
            .expect("threshold input");
        assert_eq!(threshold.ty, IsfInputType::Float);
        assert_eq!(threshold.default[0], 1.0);

        let intensity = manifest
            .inputs
            .iter()
            .find(|input| input.name == "intensity")
            .expect("intensity input");
        assert_eq!(intensity.default[0], 0.75);

        assert!(body.contains("void main()"));
        assert!(body.contains("IMG_THIS_PIXEL(inputImage)"));
    }

    #[test]
    fn compiles_the_wrapped_fragment_source_to_wgsl() {
        let (manifest, body) = parse_isf_source(BLOOM_SOURCE).expect("parse");
        let (image_order, param_order) = assign_bindings(&manifest);
        let glsl = wrap_fragment_source(&manifest, &image_order, &param_order, &body);
        let wgsl = compile_glsl_to_wgsl(&glsl, naga::ShaderStage::Fragment)
            .expect("naga: GLSL -> WGSL (fragment)");
        assert!(wgsl.contains("fn main"), "WGSL に main が無い:\n{wgsl}");
    }

    #[test]
    fn compiles_the_host_vertex_source_to_wgsl() {
        let wgsl = compile_glsl_to_wgsl(VERTEX_SOURCE, naga::ShaderStage::Vertex)
            .expect("naga: GLSL -> WGSL (vertex)");
        assert!(wgsl.contains("fn main"), "WGSL に main が無い:\n{wgsl}");
    }
}
