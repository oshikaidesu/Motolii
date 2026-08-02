//! 固定React Timeline oracleのtime surfaceを製品snapshotへ投影するnative renderer。

use std::sync::Arc;

use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth};
use harfrust::{FontRef, ShaperData, UnicodeBuffer};
use motolii_core::RationalTime;
use motolii_doc::{Document, LayerId};
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Blob, Color, Fill, FontData},
    AaConfig, AaSupport, Glyph, RenderParams, Renderer, RendererOptions, Scene,
};

use crate::native_host_layout::NativeHostLayout;
use crate::timeline_projection::{TimelineBar, TimelineProjection};

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
const TIMELINE_PIXELS_PER_SECOND: f64 = 80.0;
const TIMELINE_ROW_HEIGHT: f64 = 34.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineIntervalPreview {
    pub(crate) layer: LayerId,
    pub(crate) start: RationalTime,
    pub(crate) end: RationalTime,
}

pub(crate) struct TimelinePrepareInput<'a> {
    pub(crate) layout: NativeHostLayout,
    pub(crate) document: &'a Document,
    pub(crate) projection: &'a TimelineProjection,
    pub(crate) primary: Option<LayerId>,
    pub(crate) playhead: RationalTime,
    pub(crate) interval_preview: Option<TimelineIntervalPreview>,
}

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
    timeline_horizontal_start: RationalTime,
    timeline_vertical_offset: f64,
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
            timeline_horizontal_start: RationalTime::ZERO,
            timeline_vertical_offset: 0.0,
        })
    }

    pub(crate) fn set_timeline_viewport(
        &mut self,
        timeline_horizontal_start: RationalTime,
        timeline_vertical_offset: f64,
    ) {
        self.timeline_horizontal_start = timeline_horizontal_start;
        self.timeline_vertical_offset = if timeline_vertical_offset.is_finite() {
            timeline_vertical_offset
        } else {
            0.0
        };
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
        input: TimelinePrepareInput<'_>,
    ) -> Result<TimelineSceneStats, NativeTimelineRendererError> {
        let scene_input = TimelineSceneInput {
            layout: input.layout,
            document: input.document,
            projection: input.projection,
            timeline_horizontal_start: self.timeline_horizontal_start,
            timeline_vertical_offset: self.timeline_vertical_offset,
            primary: input.primary,
            playhead: input.playhead,
            interval_preview: input.interval_preview,
        };
        let (scene, stats) = build_scene(&self.font, scene_input)?;
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

#[derive(Clone, Copy)]
struct TimelineSceneInput<'a> {
    layout: NativeHostLayout,
    document: &'a Document,
    projection: &'a TimelineProjection,
    timeline_horizontal_start: RationalTime,
    timeline_vertical_offset: f64,
    primary: Option<LayerId>,
    playhead: RationalTime,
    interval_preview: Option<TimelineIntervalPreview>,
}

#[derive(Clone, Copy)]
struct TimelineGeometryViewport {
    time_x: f64,
    visible_width: f64,
    visible_top: f64,
    horizontal_start: f64,
    row_height: f64,
    content_y: f64,
    visible_height: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TimelineSceneStats {
    pub(crate) rows: usize,
    pub(crate) bars: usize,
    pub(crate) keys: usize,
    pub(crate) text_runs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimelineBarGeometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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
    input: TimelineSceneInput<'_>,
) -> Result<(Scene, TimelineSceneStats), NativeTimelineRendererError> {
    let TimelineSceneInput {
        layout,
        document,
        projection,
        timeline_horizontal_start,
        timeline_vertical_offset,
        primary,
        playhead,
        interval_preview,
    } = input;
    let (Some(timeline), Some(timeline_logical)) = (layout.timeline_physical, layout.timeline)
    else {
        return Ok((Scene::new(), TimelineSceneStats::default()));
    };
    let scale = f64::from(timeline.width) / timeline_logical.width;
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
    let Some(time_surface) = timeline_time_surface_logical_rect(layout) else {
        return Ok((Scene::new(), TimelineSceneStats::default()));
    };
    let visible_span = (time_surface.width / TIMELINE_PIXELS_PER_SECOND).max(0.0);
    let visible_bottom = timeline_vertical_offset + time_surface.height;
    let row_count = projection
        .bars()
        .iter()
        .map(|bar| bar.band as usize + 1)
        .max()
        .unwrap_or(1);
    let row_height = TIMELINE_ROW_HEIGHT * scale;
    let visible_top = timeline_vertical_offset;
    let horizontal_start = timeline_horizontal_start.as_seconds_f64();
    let visible_width = time_surface.width * scale;

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
        visible_width,
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

    let tick_count = visible_span.ceil().max(1.0) as usize;
    for tick in 0..=tick_count {
        let tick_seconds = tick as f64;
        let tick_x = time_x + tick_seconds * TIMELINE_PIXELS_PER_SECOND * scale;
        if tick_x > time_x + visible_width {
            continue;
        }
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
                &format!("{:.2}", horizontal_start + tick_seconds),
                tick_x + 3.0 * scale,
                y + 43.0 * scale,
                8.0 * scale,
                SUB,
            )?;
        }
    }

    for row in 0..row_count {
        let row_y_logical = f64::from(row as u32) * TIMELINE_ROW_HEIGHT;
        let row_visible_top = row_y_logical;
        let row_visible_bottom = row_visible_top + TIMELINE_ROW_HEIGHT;
        if row_visible_bottom <= visible_top || row_visible_top >= visible_bottom {
            continue;
        }
        let row_y = content_y + (row_y_logical - visible_top) * scale;
        let content_bottom = content_y + time_surface.height * scale;
        let row_bottom = row_y + row_height;
        if row_bottom <= content_y || row_y >= content_bottom {
            continue;
        }
        let separator_y =
            (row_bottom - scale.max(1.0)).clamp(content_y, content_bottom - scale.max(1.0));
        fill(
            &mut scene,
            native_x,
            separator_y,
            right - native_x,
            scale.max(1.0),
            LINE,
        );
        let button_y = row_y + 7.0 * scale;
        if button_y >= content_y && button_y + 18.0 * scale <= content_bottom {
            outline(
                &mut scene,
                native_x + 7.0 * scale,
                button_y,
                18.0 * scale,
                18.0 * scale,
                LINE,
                scale.max(1.0),
            );
            outline(
                &mut scene,
                native_x + 29.0 * scale,
                button_y,
                18.0 * scale,
                18.0 * scale,
                LINE,
                scale.max(1.0),
            );
        }
        let label_y = row_y + 19.0 * scale;
        if label_y >= content_y && label_y <= content_bottom {
            text_runs += draw_text(
                &mut scene,
                font,
                "S",
                native_x + 13.0 * scale,
                label_y,
                8.0 * scale,
                INK,
            )?;
            text_runs += draw_text(
                &mut scene,
                font,
                "M",
                native_x + 35.0 * scale,
                label_y,
                8.0 * scale,
                INK,
            )?;
        }
    }

    for bar in projection.bars() {
        let preview = interval_preview.filter(|preview| preview.layer == bar.layer);
        let mut draw_bar = *bar;
        if let Some(preview) = preview {
            draw_bar.start = preview.start;
            draw_bar.end = preview.end;
        }
        if !band_visible(f64::from(draw_bar.band), visible_top, visible_bottom) {
            continue;
        }
        let Some(geometry) = timeline_bar_geometry(
            TimelineGeometryViewport {
                time_x,
                visible_width,
                visible_top,
                horizontal_start,
                row_height,
                content_y,
                visible_height: time_surface.height * scale,
            },
            &draw_bar,
        ) else {
            continue;
        };
        fill(
            &mut scene,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            BAR,
        );
        outline(
            &mut scene,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
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
        if primary == Some(bar.layer) {
            fill(
                &mut scene,
                geometry.x,
                geometry.y,
                (3.0 * scale).min(geometry.width),
                geometry.height,
                SELECTED,
            );
            fill(
                &mut scene,
                geometry.x + (geometry.width - 3.0 * scale).max(0.0),
                geometry.y,
                (3.0 * scale).min(geometry.width),
                geometry.height,
                SELECTED,
            );
        }
        fill(
            &mut scene,
            geometry.x + 7.0 * scale,
            geometry.y + 6.0 * scale,
            16.0 * scale,
            16.0 * scale,
            Color::from_rgba8(9, 10, 10, 210),
        );
        text_runs += draw_text(
            &mut scene,
            font,
            "L",
            geometry.x + 12.0 * scale,
            geometry.y + 18.0 * scale,
            8.0 * scale,
            INK,
        )?;
        let name = document.layers.display_name(bar.layer).unwrap_or("Layer");
        text_runs += draw_text(
            &mut scene,
            font,
            name,
            geometry.x + 31.0 * scale,
            geometry.y + 18.0 * scale,
            10.0 * scale,
            BAR_TEXT,
        )?;
    }

    for key in projection.keys() {
        if !band_visible(f64::from(key.band), visible_top, visible_bottom) {
            continue;
        }
        let key_x = time_x
            + (key.t.as_seconds_f64() - horizontal_start) * TIMELINE_PIXELS_PER_SECOND * scale;
        if !key_x.is_finite() || key_x < time_x || key_x > time_x + visible_width {
            continue;
        }
        let key_y = content_y
            + (f64::from(key.band) * TIMELINE_ROW_HEIGHT - visible_top) * scale
            + row_height / 2.0;
        let radius = 4.0 * scale;
        let key_y = key_y.clamp(
            content_y + radius,
            content_y + time_surface.height * scale - radius,
        );
        let diamond = vello::kurbo::BezPath::from_iter([
            vello::kurbo::PathEl::MoveTo((key_x, key_y - radius).into()),
            vello::kurbo::PathEl::LineTo((key_x + radius, key_y).into()),
            vello::kurbo::PathEl::LineTo((key_x, key_y + radius).into()),
            vello::kurbo::PathEl::LineTo((key_x - radius, key_y).into()),
            vello::kurbo::PathEl::ClosePath,
        ]);
        scene.fill(Fill::NonZero, Affine::IDENTITY, INK, None, &diamond);
    }
    let playhead_x = playhead_logical_x(
        visible_span,
        time_x,
        time_x + visible_width,
        playhead,
        timeline_horizontal_start,
        scale,
    );
    fill(
        &mut scene,
        playhead_x,
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

fn playhead_logical_x(
    visible_span: f64,
    time_x: f64,
    right: f64,
    playhead: RationalTime,
    horizontal_start: RationalTime,
    scale: f64,
) -> f64 {
    if !visible_span.is_finite() || visible_span <= 0.0 || scale <= 0.0 {
        return time_x;
    }
    let clamped_seconds = (playhead.as_seconds_f64() - horizontal_start.as_seconds_f64())
        .max(0.0)
        .min(visible_span);
    (time_x + clamped_seconds * TIMELINE_PIXELS_PER_SECOND * scale).clamp(time_x, right)
}

fn timeline_bar_geometry(
    viewport: TimelineGeometryViewport,
    bar: &TimelineBar,
) -> Option<TimelineBarGeometry> {
    let TimelineGeometryViewport {
        time_x,
        visible_width,
        visible_top,
        horizontal_start,
        row_height,
        content_y,
        visible_height,
    } = viewport;
    let scale = row_height / TIMELINE_ROW_HEIGHT;
    if !time_x.is_finite() || !visible_width.is_finite() {
        return None;
    }
    let bar_left = bar.start.as_seconds_f64() - horizontal_start;
    let bar_right = bar.end.as_seconds_f64() - horizontal_start;
    if !bar_left.is_finite() || !bar_right.is_finite() {
        return None;
    }
    let x = time_x + bar_left * (TIMELINE_PIXELS_PER_SECOND * scale);
    let bar_right = time_x + bar_right * (TIMELINE_PIXELS_PER_SECOND * scale);
    if !x.is_finite() || !bar_right.is_finite() {
        return None;
    }
    let y =
        content_y + (f64::from(bar.band) * TIMELINE_ROW_HEIGHT - visible_top) * scale + 5.0 * scale;
    let height = (row_height - 10.0 * scale).max(12.0 * scale);
    let right = time_x + visible_width;
    let bottom = content_y + visible_height;
    let clipped_x = x.max(time_x);
    let clipped_right = bar_right.min(right);
    let clipped_y = y.max(content_y);
    let clipped_bottom = (y + height).min(bottom);
    if clipped_right <= clipped_x || clipped_bottom <= clipped_y {
        return None;
    }
    (height > 0.0).then_some(TimelineBarGeometry {
        x: clipped_x,
        y: clipped_y,
        width: clipped_right - clipped_x,
        height: clipped_bottom - clipped_y,
    })
}

fn band_visible(row: f64, visible_top: f64, visible_bottom: f64) -> bool {
    let bar_top = row * TIMELINE_ROW_HEIGHT;
    let bar_bottom = bar_top + TIMELINE_ROW_HEIGHT;
    bar_bottom > visible_top && bar_top < visible_bottom
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
) -> Option<crate::native_host_layout::LogicalRect> {
    layout
        .timeline
        .map(|timeline| crate::native_host_layout::LogicalRect {
            x: timeline.x,
            y: timeline.y,
            width: KEY_TOOLS_WIDTH.min(timeline.width),
            height: timeline.height,
        })
}

pub(crate) fn timeline_time_surface_logical_rect(
    layout: NativeHostLayout,
) -> Option<crate::native_host_layout::LogicalRect> {
    let timeline = layout.timeline?;
    let x_offset = (KEY_TOOLS_WIDTH + BAND_RAIL_WIDTH).min(timeline.width);
    let y_offset = (HEADER_HEIGHT + RULER_HEIGHT).min(timeline.height);
    let width = timeline.width - x_offset;
    let height = timeline.height - y_offset;
    (width > 0.0 && height > 0.0).then_some(crate::native_host_layout::LogicalRect {
        x: timeline.x + x_offset,
        y: timeline.y + y_offset,
        width,
        height,
    })
}

pub(crate) fn timeline_ruler_logical_rect(
    layout: NativeHostLayout,
) -> Option<crate::native_host_layout::LogicalRect> {
    let timeline = layout.timeline?;
    let x_offset = (KEY_TOOLS_WIDTH + BAND_RAIL_WIDTH).min(timeline.width);
    let y_offset = HEADER_HEIGHT.min(timeline.height);
    let width = timeline.width - x_offset;
    let height = RULER_HEIGHT.min((timeline.height - y_offset).max(0.0));
    (width > 0.0 && height > 0.0).then_some(crate::native_host_layout::LogicalRect {
        x: timeline.x + x_offset,
        y: timeline.y + y_offset,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline_projection::TimelineBar;

    #[test]
    fn shipped_bar_geometry_stays_inside_the_time_surface() {
        let bar = TimelineBar {
            layer: motolii_doc::LayerId::from_raw(1),
            start: motolii_core::RationalTime::ZERO,
            end: motolii_core::RationalTime::try_new(5, 1).unwrap(),
            band: 0,
            x_start: 0.0,
            x_end: 1.0,
            y_top: 0.0,
            y_bottom: 1.0,
        };
        let geometry = timeline_bar_geometry(
            TimelineGeometryViewport {
                time_x: 100.0,
                visible_width: 800.0,
                visible_top: 0.0,
                horizontal_start: 0.0,
                row_height: 34.0,
                content_y: 55.0,
                visible_height: 145.0,
            },
            &bar,
        )
        .unwrap();

        assert_eq!(geometry.x, 100.0);
        assert_eq!(geometry.x + geometry.width, 500.0);
        assert!(geometry.y >= 55.0);
        assert!(geometry.y + geometry.height <= 200.0);
    }

    #[test]
    fn playhead_x_uses_the_current_time_and_clamps_to_the_composition() {
        assert_eq!(
            playhead_logical_x(
                5.0,
                100.0,
                500.0,
                RationalTime::ZERO,
                RationalTime::ZERO,
                1.0,
            ),
            100.0
        );
        assert_eq!(
            playhead_logical_x(
                5.0,
                100.0,
                500.0,
                RationalTime::try_new(5, 1).unwrap(),
                RationalTime::ZERO,
                1.0,
            ),
            500.0
        );
        assert_eq!(
            playhead_logical_x(
                5.0,
                100.0,
                500.0,
                RationalTime::try_new(12, 1).unwrap(),
                RationalTime::ZERO,
                1.0,
            ),
            500.0
        );
    }

    #[test]
    fn hidden_timeline_builds_an_empty_scene_without_reading_font_data() {
        let frame = motolii_core::FrameDesc::try_packed(
            1920,
            1080,
            motolii_core::PixelFormat::Rgba8Unorm,
            motolii_core::ColorSpace::Srgb,
            true,
        )
        .unwrap();
        let mut authority = crate::layout::PanelLayout::built_in();
        authority
            .apply(
                crate::layout::LayoutAction::Hide(crate::layout::PanelRole::Timeline),
                crate::layout::LayoutConstraints {
                    viewport_width: 1_000.0,
                    stage_min_width: 320.0,
                },
            )
            .unwrap();
        let layout = NativeHostLayout::try_new(1, 1_000, 800, 1.0, frame, &authority)
            .unwrap()
            .unwrap();
        let document = crate::static_preview::bootstrap_document().unwrap();
        let projection = crate::timeline_projection::project_timeline(
            &document,
            &crate::timeline_projection::TimelineMetrics {
                band_height: 1.0,
                units_per_second: document.composition.duration.as_seconds_f64().recip(),
                key_half_extent: 1.0,
            },
            &crate::timeline_projection::TimelineViewport {
                start: motolii_core::RationalTime::ZERO,
                end: document.composition.duration,
            },
        )
        .unwrap();
        let font = TimelineFont {
            bytes: Arc::from([]),
            face_index: 0,
        };

        let (_, stats) = build_scene(
            &font,
            TimelineSceneInput {
                layout,
                document: &document,
                projection: &projection,
                timeline_horizontal_start: RationalTime::ZERO,
                timeline_vertical_offset: 0.0,
                primary: None,
                playhead: RationalTime::ZERO,
                interval_preview: None,
            },
        )
        .unwrap();
        assert_eq!(stats, TimelineSceneStats::default());
    }

    #[test]
    fn bar_geometry_uses_viewport_start_and_fixed_scale() {
        let bar = TimelineBar {
            layer: motolii_doc::LayerId::from_raw(1),
            start: motolii_core::RationalTime::try_new(2, 1).unwrap(),
            end: motolii_core::RationalTime::try_new(3, 1).unwrap(),
            band: 1,
            x_start: 0.0,
            x_end: 1.0,
            y_top: 0.0,
            y_bottom: 1.0,
        };
        let geometry = timeline_bar_geometry(
            TimelineGeometryViewport {
                time_x: 100.0,
                visible_width: 800.0,
                visible_top: 0.0,
                horizontal_start: 1.0,
                row_height: 34.0,
                content_y: 55.0,
                visible_height: 145.0,
            },
            &bar,
        )
        .unwrap();
        assert_eq!(geometry.x, 180.0);
        assert_eq!(geometry.y, 94.0);
        assert_eq!(geometry.width, 80.0);
    }

    #[test]
    fn bar_geometry_clips_to_the_timeline_surface() {
        let bar = TimelineBar {
            layer: motolii_doc::LayerId::from_raw(1),
            start: motolii_core::RationalTime::try_new(-1, 1).unwrap(),
            end: motolii_core::RationalTime::try_new(2, 1).unwrap(),
            band: 0,
            x_start: 0.0,
            x_end: 1.0,
            y_top: 0.0,
            y_bottom: 1.0,
        };
        let geometry = timeline_bar_geometry(
            TimelineGeometryViewport {
                time_x: 100.0,
                visible_width: 160.0,
                visible_top: 0.0,
                horizontal_start: 0.0,
                row_height: 34.0,
                content_y: 55.0,
                visible_height: 34.0,
            },
            &bar,
        )
        .unwrap();
        assert_eq!(geometry.x, 100.0);
        assert_eq!(geometry.width, 160.0);
        assert!(geometry.y >= 55.0);
        assert!(geometry.y + geometry.height <= 89.0);
    }

    #[test]
    fn fixed_rows_keep_34_logical_pixels_at_retina_scale() {
        let bar = TimelineBar {
            layer: motolii_doc::LayerId::from_raw(1),
            start: motolii_core::RationalTime::ZERO,
            end: motolii_core::RationalTime::try_new(1, 1).unwrap(),
            band: 24,
            x_start: 0.0,
            x_end: 1.0,
            y_top: 0.0,
            y_bottom: 1.0,
        };
        let geometry = timeline_bar_geometry(
            TimelineGeometryViewport {
                time_x: 100.0,
                visible_width: 160.0,
                visible_top: 0.0,
                horizontal_start: 0.0,
                row_height: 68.0,
                content_y: 55.0,
                visible_height: 25.0 * 68.0,
            },
            &bar,
        )
        .unwrap();
        assert_eq!(geometry.y, 55.0 + 24.0 * 68.0 + 10.0);
        assert!(geometry.height <= 68.0);
    }
}
