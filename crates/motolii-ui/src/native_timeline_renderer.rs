//! 固定React Timeline oracleのtime surfaceを製品snapshotへ投影するnative renderer。

use std::sync::Arc;

use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth};
use harfrust::{FontRef, ShaperData, UnicodeBuffer};
use motolii_doc::{Document, LayerId};
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Blob, Color, Fill, FontData},
    AaConfig, AaSupport, Glyph, RenderParams, Renderer, RendererOptions, Scene,
};

use crate::native_host_layout::{NativeHostLayout, PhysicalRect};
use crate::timeline_projection::TimelineProjection;

const KEY_TOOLS_WIDTH: f64 = 202.0;
const BAND_RAIL_WIDTH: f64 = 54.0;
const HEADER_HEIGHT: f64 = 30.0;
const RULER_HEIGHT: f64 = 25.0;

const BG: Color = Color::from_rgba8(5, 5, 6, 255);
const RAISED: Color = Color::from_rgba8(15, 16, 17, 255);
const LINE: Color = Color::from_rgba8(38, 41, 43, 255);
const LINE_STRONG: Color = Color::from_rgba8(87, 89, 92, 255);
const INK: Color = Color::from_rgba8(232, 232, 232, 255);
const SUB: Color = Color::from_rgba8(158, 158, 158, 255);
const ACTIVE: Color = Color::from_rgba8(212, 150, 79, 255);
const BAR: Color = Color::from_rgba8(204, 149, 135, 255);
const BAR_TEXT: Color = Color::from_rgba8(26, 27, 28, 255);
const SELECTED: Color = Color::from_rgba8(250, 191, 92, 255);
const PLAYHEAD: Color = Color::from_rgba8(242, 77, 87, 255);

pub(crate) struct NativeTimelineRenderer {
    renderer: Renderer,
    font: TimelineFont,
    overlay_texture: wgpu::Texture,
    overlay_view: wgpu::TextureView,
    overlay_bind_group: wgpu::BindGroup,
    overlay_bind_group_layout: wgpu::BindGroupLayout,
    overlay_pipeline: wgpu::RenderPipeline,
    width: u32,
    height: u32,
}

impl NativeTimelineRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Result<Self, NativeTimelineRendererError> {
        let renderer = Renderer::new(
            device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: std::num::NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )?;
        let font = TimelineFont::load()?;
        let overlay_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("motolii-native-timeline-overlay-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-native-timeline-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let (overlay_texture, overlay_view, overlay_bind_group) =
            create_overlay_target(device, &overlay_bind_group_layout, &sampler, width, height);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-native-timeline-overlay-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("native_timeline_overlay.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-native-timeline-overlay-pipeline-layout"),
            bind_group_layouts: &[Some(&overlay_bind_group_layout)],
            immediate_size: 0,
        });
        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-native-timeline-overlay-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        // 初回scene描画前にもtextureを完全に定義しておく。
        let _ = queue;
        Ok(Self {
            renderer,
            font,
            overlay_texture,
            overlay_view,
            overlay_bind_group,
            overlay_bind_group_layout,
            overlay_pipeline,
            width: width.max(1),
            height: height.max(1),
        })
    }

    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return;
        }
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-native-timeline-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let (texture, view, bind_group) = create_overlay_target(
            device,
            &self.overlay_bind_group_layout,
            &sampler,
            width,
            height,
        );
        self.overlay_texture = texture;
        self.overlay_view = view;
        self.overlay_bind_group = bind_group;
        self.width = width;
        self.height = height;
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: NativeHostLayout,
        document: &Document,
        projection: &TimelineProjection,
        primary: Option<LayerId>,
    ) -> Result<TimelineSceneStats, NativeTimelineRendererError> {
        let (scene, stats) = build_scene(&self.font, layout, document, projection, primary)?;
        self.renderer.render_to_texture(
            device,
            queue,
            &scene,
            &self.overlay_view,
            &RenderParams {
                base_color: Color::TRANSPARENT,
                width: self.width,
                height: self.height,
                antialiasing_method: AaConfig::Area,
            },
        )?;
        Ok(stats)
    }

    pub(crate) fn composite<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.overlay_pipeline);
        pass.set_bind_group(0, &self.overlay_bind_group, &[]);
        pass.set_viewport(0.0, 0.0, self.width as f32, self.height as f32, 0.0, 1.0);
        pass.set_scissor_rect(0, 0, self.width, self.height);
        pass.draw(0..6, 0..1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineSceneStats {
    pub(crate) rows: usize,
    pub(crate) bars: usize,
    pub(crate) keys: usize,
    pub(crate) text_runs: usize,
}

struct TimelineFont {
    bytes: Arc<[u8]>,
    face_index: u32,
}

impl TimelineFont {
    fn load() -> Result<Self, NativeTimelineRendererError> {
        let mut collection = Collection::new(CollectionOptions::default());
        collection.load_system_fonts();
        for family_name in ["Arial", "Helvetica", "SF Pro Text"] {
            let Some(family) = collection.family_by_name(family_name) else {
                continue;
            };
            let face = family
                .fonts()
                .iter()
                .find(|font| {
                    font.width() == FontWidth::NORMAL
                        && font.style() == FontStyle::Normal
                        && font.weight() == FontWeight::new(400.0)
                })
                .or_else(|| family.fonts().first());
            let Some(face) = face else {
                continue;
            };
            let Some(bytes) = face.load(None) else {
                continue;
            };
            return Ok(Self {
                bytes: Arc::from(bytes.as_ref()),
                face_index: face.index(),
            });
        }
        Err(NativeTimelineRendererError::FontUnavailable)
    }
}

fn build_scene(
    font: &TimelineFont,
    layout: NativeHostLayout,
    document: &Document,
    projection: &TimelineProjection,
    primary: Option<LayerId>,
) -> Result<(Scene, TimelineSceneStats), NativeTimelineRendererError> {
    let timeline = layout.timeline_physical;
    let scale = f64::from(timeline.width) / layout.timeline.width;
    let key_tools_width = KEY_TOOLS_WIDTH * scale;
    let rail_width = BAND_RAIL_WIDTH * scale;
    let header_height = HEADER_HEIGHT * scale;
    let ruler_height = RULER_HEIGHT * scale;
    let x = f64::from(timeline.x);
    let y = f64::from(timeline.y);
    let right = x + f64::from(timeline.width);
    let bottom = y + f64::from(timeline.height);
    let native_x = (x + key_tools_width).min(right);
    let time_x = (native_x + rail_width).min(right);
    let content_y = (y + header_height + ruler_height).min(bottom);
    let row_count = projection
        .bars()
        .iter()
        .map(|bar| bar.band as usize + 1)
        .max()
        .unwrap_or(1);
    let row_height = ((bottom - content_y) / row_count as f64).max(1.0);

    let mut scene = Scene::new();
    let mut text_runs = 0;
    fill(&mut scene, native_x, y, right - native_x, bottom - y, BG);
    fill(
        &mut scene,
        native_x,
        y,
        right - native_x,
        header_height,
        RAISED,
    );
    fill(
        &mut scene,
        native_x,
        y + header_height - scale.max(1.0),
        right - native_x,
        scale.max(1.0),
        LINE_STRONG,
    );
    fill(
        &mut scene,
        native_x + 10.0 * scale,
        y + 11.0 * scale,
        7.0 * scale,
        7.0 * scale,
        Color::from_rgba8(140, 77, 69, 255),
    );
    text_runs += draw_text(
        &mut scene,
        font,
        "TIMELINE",
        native_x + 25.0 * scale,
        y + 19.0 * scale,
        11.0 * scale,
        INK,
    )?;
    fill(
        &mut scene,
        native_x,
        y + header_height,
        rail_width,
        bottom - y - header_height,
        BG,
    );
    fill(
        &mut scene,
        time_x,
        y + header_height,
        right - time_x,
        ruler_height,
        BG,
    );
    fill(
        &mut scene,
        time_x - scale.max(1.0),
        y + header_height,
        scale.max(1.0),
        bottom - y - header_height,
        LINE_STRONG,
    );
    text_runs += draw_text(
        &mut scene,
        font,
        "S",
        native_x + 10.0 * scale,
        y + 43.0 * scale,
        8.0 * scale,
        SUB,
    )?;
    text_runs += draw_text(
        &mut scene,
        font,
        "M",
        native_x + 32.0 * scale,
        y + 43.0 * scale,
        8.0 * scale,
        SUB,
    )?;
    text_runs += draw_text(
        &mut scene,
        font,
        "TIME / BEAT",
        time_x + 8.0 * scale,
        y + 43.0 * scale,
        8.0 * scale,
        ACTIVE,
    )?;

    for tick in 0..=16 {
        let tick_x = time_x + (right - time_x) * f64::from(tick) / 16.0;
        let major = tick % 4 == 0;
        let tick_height = if major { 8.0 } else { 4.0 } * scale;
        fill(
            &mut scene,
            tick_x,
            content_y - tick_height,
            scale.max(1.0),
            tick_height,
            if major { LINE_STRONG } else { LINE },
        );
        fill(
            &mut scene,
            tick_x,
            content_y,
            scale.max(1.0),
            bottom - content_y,
            if major {
                LINE
            } else {
                Color::from_rgba8(17, 17, 18, 255)
            },
        );
        if major {
            text_runs += draw_text(
                &mut scene,
                font,
                &(tick / 4).to_string(),
                tick_x + 3.0 * scale,
                y + 43.0 * scale,
                8.0 * scale,
                SUB,
            )?;
        }
    }

    for row in 0..row_count {
        let row_y = content_y + row as f64 * row_height;
        fill(
            &mut scene,
            native_x,
            row_y + row_height - scale.max(1.0),
            right - native_x,
            scale.max(1.0),
            LINE,
        );
        outline(
            &mut scene,
            native_x + 7.0 * scale,
            row_y + 7.0 * scale,
            18.0 * scale,
            18.0 * scale,
            LINE,
            scale.max(1.0),
        );
        outline(
            &mut scene,
            native_x + 29.0 * scale,
            row_y + 7.0 * scale,
            18.0 * scale,
            18.0 * scale,
            LINE,
            scale.max(1.0),
        );
        text_runs += draw_text(
            &mut scene,
            font,
            "S",
            native_x + 13.0 * scale,
            row_y + 19.0 * scale,
            8.0 * scale,
            INK,
        )?;
        text_runs += draw_text(
            &mut scene,
            font,
            "M",
            native_x + 35.0 * scale,
            row_y + 19.0 * scale,
            8.0 * scale,
            INK,
        )?;
    }

    for bar in projection.bars() {
        let bar_x = time_x + bar.x_start.clamp(0.0, 1.0) * (right - time_x);
        let bar_right = time_x + bar.x_end.clamp(0.0, 1.0) * (right - time_x);
        let row_y = content_y + f64::from(bar.band) * row_height;
        let bar_y = row_y + 5.0 * scale;
        let bar_height = (row_height - 10.0 * scale).max(12.0 * scale);
        fill(&mut scene, bar_x, bar_y, bar_right - bar_x, bar_height, BAR);
        outline(
            &mut scene,
            bar_x,
            bar_y,
            bar_right - bar_x,
            bar_height,
            if primary == Some(bar.layer) {
                SELECTED
            } else {
                LINE_STRONG
            },
            if primary == Some(bar.layer) {
                2.0 * scale
            } else {
                scale.max(1.0)
            },
        );
        fill(
            &mut scene,
            bar_x + 7.0 * scale,
            bar_y + 6.0 * scale,
            16.0 * scale,
            16.0 * scale,
            Color::from_rgba8(9, 10, 10, 210),
        );
        text_runs += draw_text(
            &mut scene,
            font,
            "L",
            bar_x + 12.0 * scale,
            bar_y + 18.0 * scale,
            8.0 * scale,
            INK,
        )?;
        let name = document.layers.display_name(bar.layer).unwrap_or("Layer");
        text_runs += draw_text(
            &mut scene,
            font,
            name,
            bar_x + 31.0 * scale,
            bar_y + 18.0 * scale,
            10.0 * scale,
            BAR_TEXT,
        )?;
    }

    for key in projection.keys() {
        let key_x = time_x + key.center_x.clamp(0.0, 1.0) * (right - time_x);
        let key_y = content_y + f64::from(key.band) * row_height + row_height / 2.0;
        let radius = 4.0 * scale;
        let diamond = vello::kurbo::BezPath::from_iter([
            vello::kurbo::PathEl::MoveTo((key_x, key_y - radius).into()),
            vello::kurbo::PathEl::LineTo((key_x + radius, key_y).into()),
            vello::kurbo::PathEl::LineTo((key_x, key_y + radius).into()),
            vello::kurbo::PathEl::LineTo((key_x - radius, key_y).into()),
            vello::kurbo::PathEl::ClosePath,
        ]);
        scene.fill(Fill::NonZero, Affine::IDENTITY, INK, None, &diamond);
    }
    fill(
        &mut scene,
        time_x,
        y + header_height,
        (2.0 * scale).max(1.0),
        bottom - y - header_height,
        PLAYHEAD,
    );

    Ok((
        scene,
        TimelineSceneStats {
            rows: row_count,
            bars: projection.bars().len(),
            keys: projection.keys().len(),
            text_runs,
        },
    ))
}

fn fill(scene: &mut Scene, x: f64, y: f64, width: f64, height: f64, color: Color) {
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        color,
        None,
        &Rect::new(x, y, x + width, y + height),
    );
}

fn outline(
    scene: &mut Scene,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: Color,
    thickness: f64,
) {
    fill(scene, x, y, width, thickness, color);
    fill(scene, x, y + height - thickness, width, thickness, color);
    fill(scene, x, y, thickness, height, color);
    fill(scene, x + width - thickness, y, thickness, height, color);
}

fn draw_text(
    scene: &mut Scene,
    font: &TimelineFont,
    text: &str,
    origin_x: f64,
    baseline_y: f64,
    font_size: f64,
    color: Color,
) -> Result<usize, NativeTimelineRendererError> {
    let face = FontRef::from_index(&font.bytes, font.face_index)
        .map_err(|_| NativeTimelineRendererError::FontUnavailable)?;
    let data = ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let units_per_em = shaper.units_per_em();
    if units_per_em == 0 {
        return Err(NativeTimelineRendererError::FontUnavailable);
    }
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    let glyph_buffer = shaper.shape(buffer, &[]);
    let scale = font_size as f32 / units_per_em as f32;
    let mut pen_x = origin_x as f32;
    let mut pen_y = baseline_y as f32;
    let glyphs = glyph_buffer
        .glyph_infos()
        .iter()
        .zip(glyph_buffer.glyph_positions())
        .filter_map(|(info, position)| {
            if info.glyph_id == 0 {
                return None;
            }
            let x = pen_x + position.x_offset as f32 * scale;
            let y = pen_y - position.y_offset as f32 * scale;
            pen_x += position.x_advance as f32 * scale;
            pen_y -= position.y_advance as f32 * scale;
            Some(Glyph {
                id: info.glyph_id,
                x,
                y,
            })
        })
        .collect::<Vec<_>>();
    if glyphs.is_empty() {
        return Ok(0);
    }
    let font_data = FontData::new(
        Blob::new(Arc::new(FontBytes(Arc::clone(&font.bytes)))),
        font.face_index,
    );
    scene
        .draw_glyphs(&font_data)
        .font_size(font_size as f32)
        .transform(Affine::IDENTITY)
        .brush(color)
        .draw(Fill::NonZero, glyphs.into_iter());
    Ok(1)
}

struct FontBytes(Arc<[u8]>);

impl AsRef<[u8]> for FontBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn create_overlay_target(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("motolii-native-timeline-overlay-target"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("motolii-native-timeline-overlay-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    (texture, view, bind_group)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum NativeTimelineRendererError {
    #[error("native Timeline Vello renderer failed")]
    Renderer(#[from] vello::Error),
    #[error("native Timeline system font is unavailable")]
    FontUnavailable,
}

pub(crate) fn key_tools_logical_rect(
    layout: NativeHostLayout,
) -> crate::native_host_layout::LogicalRect {
    crate::native_host_layout::LogicalRect {
        x: layout.timeline.x,
        y: layout.timeline.y,
        width: KEY_TOOLS_WIDTH.min(layout.timeline.width),
        height: layout.timeline.height,
    }
}

#[allow(dead_code)]
fn _timeline_rect(layout: NativeHostLayout) -> PhysicalRect {
    layout.timeline_physical
}
