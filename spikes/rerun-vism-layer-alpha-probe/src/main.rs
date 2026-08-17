//! Rerunの標準GridMap visualizerがVismの透明Filter出力を3D Stageで層として表示できるかを確認する。

use std::path::{Path, PathBuf};
use std::time::Duration;

use image::ImageEncoder as _;
use re_chunk::{Chunk, RowId};
use re_log_types::{ApplicationId, TimePoint};
use re_renderer::RenderContext;
use re_sdk_types::archetypes::{GridMap, Transform3D};
use re_sdk_types::datatypes::{ChannelDatatype, ColorModel, ImageFormat};
use re_view_spatial::SpatialStage;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const BACKGROUND_PARENT_PATH: &str = "layer-stack/background";
const FOREGROUND_PARENT_PATH: &str = "layer-stack/foreground-filter";
const BACKGROUND_IMAGE_PATH: &str = "layer-stack/background/image";
const FOREGROUND_IMAGE_PATH: &str = "layer-stack/foreground-filter/image";
const SCREENSHOT_DELAY_FRAMES: u32 = 16;

fn main() -> eframe::Result {
    let screenshot = parse_args();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Rerun Image alpha-layer probe"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Rerun Image alpha-layer probe",
        native_options,
        Box::new(move |creation_context| {
            setup_render_context(creation_context)?;
            Ok(Box::new(StageHost::new(
                screenshot,
                creation_context.egui_ctx.clone(),
            )?))
        }),
    )
}

fn parse_args() -> PathBuf {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag, path] if flag == "--screenshot" => PathBuf::from(path),
        _ => {
            eprintln!("usage: rerun-vism-layer-alpha-probe --screenshot <output.png>");
            std::process::exit(2);
        }
    }
}

/// `SpatialStage`のpaint callbackが参照するRerun contextを一度だけ置く。
fn setup_render_context(cc: &eframe::CreationContext<'_>) -> Result<(), String> {
    let render_state = cc
        .wgpu_render_state
        .as_ref()
        .ok_or_else(|| "eframeのWGPU render stateが無い".to_owned())?;
    let render_ctx = RenderContext::new(
        &render_state.adapter,
        render_state.device.clone(),
        render_state.queue.clone(),
        render_state.target_format,
        re_renderer::RenderConfig::best_for_device_caps,
    )
    .map_err(|error| format!("create Rerun render context: {error}"))?;
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(render_ctx);
    Ok(())
}

struct StageHost {
    stage: SpatialStage,
    screenshot: PathBuf,
    frame_count: u32,
    focus_frames: u8,
    screenshot_requested: bool,
    captured: bool,
    error: Option<String>,
}

impl StageHost {
    fn new(screenshot: PathBuf, _egui_ctx: egui::Context) -> Result<Self, String> {
        let mut stage = SpatialStage::new(ApplicationId::from("rerun-vism-layer-alpha-probe"))
            .map_err(|error| format!("create Rerun spatial stage: {error}"))?;
        ingest_layer_stack(&mut stage)?;
        Ok(Self {
            stage,
            screenshot,
            frame_count: 0,
            focus_frames: 3,
            screenshot_requested: false,
            captured: false,
            error: None,
        })
    }

    fn show_stage(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Result<(), String> {
        let render_state = frame
            .wgpu_render_state()
            .ok_or_else(|| "eframeのWGPU render stateが無い".to_owned())?;
        let mut render_ctx = render_state
            .renderer
            .write()
            .callback_resources
            .remove::<RenderContext>()
            .ok_or_else(|| "Rerun render contextがcallback_resourcesに無い".to_owned())?;
        let result = (|| {
            if self.focus_frames > 0 {
                self.stage.focus_entity(FOREGROUND_IMAGE_PATH);
                self.focus_frames -= 1;
            }
            self.stage
                .show(ui, &mut render_ctx)
                .map_err(|error| error.to_string())
        })();
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(render_ctx);
        result
    }

    fn drive_capture(&mut self, ctx: &egui::Context) {
        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = captured.filter(|_| !self.captured) {
            let [width, height] = image.size;
            match write_png(
                &self.screenshot,
                image.as_raw(),
                width as u32,
                height as u32,
            ) {
                Ok(()) => {
                    println!(
                        "rerun-vism-layer-alpha-probe: captured {} ({}x{})",
                        self.screenshot.display(),
                        width,
                        height
                    );
                    self.captured = true;
                }
                Err(error) => self.error = Some(error),
            }
            return;
        }
        if !self.captured
            && !self.screenshot_requested
            && self.frame_count >= SCREENSHOT_DELAY_FRAMES
        {
            self.screenshot_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        if !self.captured {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

impl eframe::App for StageHost {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.frame_count += 1;
        self.error = self.show_stage(ui, frame).err();
        ui.label(
            "Expected: checkerboard remains visible through the transparent foreground gutter.",
        );
        ui.label(
            "Route: standard GridMap visualizer -> RectangleRenderer in the same SpatialStage.",
        );
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
        self.drive_capture(ui.ctx());
    }
}

/// 各素材のlocal planeはz=0のまま、親transformだけで2.5Dの前後関係を持たせる。
fn ingest_layer_stack(stage: &mut SpatialStage) -> Result<(), String> {
    let format =
        ImageFormat::from_color_model([WIDTH, HEIGHT], ColorModel::RGBA, ChannelDatatype::U8);
    let background = GridMap::new(checkerboard(), format.clone(), 1.0 / HEIGHT as f32);
    let foreground = GridMap::new(straight_alpha_blur_output(), format, 1.0 / HEIGHT as f32);
    ingest_grid_map(stage, BACKGROUND_IMAGE_PATH, &background)?;
    ingest_grid_map(stage, FOREGROUND_IMAGE_PATH, &foreground)?;

    let background = Transform3D::from_translation_scale(
        [-(WIDTH as f32 / HEIGHT as f32) * 0.5, -0.5, 0.0],
        [1.294, 1.294, 1.0],
    );
    let foreground = Transform3D::from_translation_scale(
        [-(WIDTH as f32 / HEIGHT as f32) * 0.5, -0.5, 0.02],
        [1.294, 1.294, 1.0],
    );
    ingest_transform(stage, BACKGROUND_PARENT_PATH, &background)?;
    ingest_transform(stage, FOREGROUND_PARENT_PATH, &foreground)
}

fn ingest_grid_map(stage: &mut SpatialStage, path: &str, grid_map: &GridMap) -> Result<(), String> {
    let chunk = Chunk::builder(path)
        .with_archetype(RowId::new(), TimePoint::STATIC, grid_map)
        .build()
        .map_err(|error| format!("build Rerun GridMap evidence: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(chunk))
        .map_err(|error| format!("ingest Rerun GridMap evidence: {error}"))
}

fn ingest_transform(
    stage: &mut SpatialStage,
    path: &str,
    transform: &Transform3D,
) -> Result<(), String> {
    let chunk = Chunk::builder(path)
        .with_archetype(RowId::new(), TimePoint::STATIC, transform)
        .build()
        .map_err(|error| format!("build Rerun layer transform: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(chunk))
        .map_err(|error| format!("ingest Rerun layer transform: {error}"))
}

fn checkerboard() -> Vec<u8> {
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = ((y * WIDTH + x) * 4) as usize;
            let tile = ((x / 32) + (y / 32)) % 2;
            let color = if tile == 0 {
                [36, 143, 237, 255]
            } else {
                [252, 189, 55, 255]
            };
            rgba[index..index + 4].copy_from_slice(&color);
        }
    }
    rgba
}

/// 出力rectは中心の素材より広く、外縁はalpha=0のstraight-alpha gutterである。
fn straight_alpha_blur_output() -> Vec<u8> {
    let mut rgba = vec![0; (WIDTH * HEIGHT * 4) as usize];
    let center_x = WIDTH as f32 * 0.50;
    let center_y = HEIGHT as f32 * 0.50;
    let solid_radius = HEIGHT as f32 * 0.16;
    let support_radius = HEIGHT as f32 * 0.37;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = ((y * WIDTH + x) * 4) as usize;
            let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
            let alpha = if distance <= solid_radius {
                0.92
            } else if distance < support_radius {
                let t = (distance - solid_radius) / (support_radius - solid_radius);
                0.92 * (1.0 - t).powi(3)
            } else {
                0.0
            };
            rgba[index] = 255;
            rgba[index + 1] = 49;
            rgba[index + 2] = 143;
            rgba[index + 3] = (255.0 * alpha).round() as u8;
        }
    }
    rgba
}

fn write_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("create capture directory {}: {error}", parent.display()))?;
    }
    let file = std::fs::File::create(path)
        .map_err(|error| format!("create capture PNG {}: {error}", path.display()))?;
    image::codecs::png::PngEncoder::new(std::io::BufWriter::new(file))
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|error| format!("encode capture PNG {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_has_a_transparent_gutter() {
        let pixels = straight_alpha_blur_output();
        assert_eq!(&pixels[0..4], &[255, 49, 143, 0]);
    }
}
