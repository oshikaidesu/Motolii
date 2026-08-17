//! 上流無改変の pin のまま、`SpatialStage::copy_gpu_image` で渡した GPU-resident な
//! premultiplied RGBA texture が、3D Stage 上で画素ごとの alpha として合成されるかを確認する。

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
/// 対照群。CPU由来の普通の`Image`が同じ3D Stageで描かれるかを同時に見る。
const CONTROL_IMAGE_PATH: &str = "layer-stack/control-image";
/// 同じ`Image`を、自分自身ではなく親にtransformを置いた形で並べる。
/// `copy_gpu_image`は自分自身のpathへtransformを書くため、ここが効くかで原因が分かれる。
const CONTROL_PARENTED_IMAGE_PATH: &str = "layer-stack/control-parented/image";
const CONTROL_PARENTED_PARENT_PATH: &str = "layer-stack/control-parented";
/// 位置合わせの基準。既知の良品である`GridMap`を対照群と同じ矩形へ置く。
const CONTROL_GRID_PATH: &str = "layer-stack/control-grid/image";
const CONTROL_GRID_PARENT_PATH: &str = "layer-stack/control-grid";
const SCREENSHOT_DELAY_FRAMES: u32 = 16;
/// ここでhostが別のtextureへ描き始める。撮影はこれより後。
const SWAP_AT_FRAME: u32 = 8;

fn main() -> eframe::Result {
    let screenshot = parse_args();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Rerun copy_gpu_image alpha-layer probe"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Rerun copy_gpu_image alpha-layer probe",
        native_options,
        Box::new(move |creation_context| {
            setup_render_context(creation_context)?;
            Ok(Box::new(StageHost::new(screenshot)?))
        }),
    )
}

fn parse_args() -> PathBuf {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag, path] if flag == "--screenshot" => PathBuf::from(path),
        _ => {
            eprintln!("usage: rerun-vism-gpu-alpha-probe --screenshot <output.png>");
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
    /// Vism の出力に相当するGPU texture。host側が所有を持ち続ける。
    /// ダブルバッファを模して2枚持ち、途中で差し替える。
    filter_output: Vec<wgpu::Texture>,
    screenshot_requested: bool,
    captured: bool,
    error: Option<String>,
}

impl StageHost {
    fn new(screenshot: PathBuf) -> Result<Self, String> {
        let mut stage = SpatialStage::new(ApplicationId::from("rerun-vism-gpu-alpha-probe"))
            .map_err(|error| format!("create Rerun spatial stage: {error}"))?;
        ingest_background(&mut stage)?;
        ingest_control_image(&mut stage)?;
        ingest_transform(
            &mut stage,
            FOREGROUND_PARENT_PATH,
            &Transform3D::from_translation([0.0, 0.0, 0.05]),
        )?;
        Ok(Self {
            stage,
            screenshot,
            frame_count: 0,
            focus_frames: 3,
            filter_output: Vec::new(),
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
            if self.filter_output.is_empty() {
                self.filter_output = vec![
                    upload_premultiplied(
                        &render_state.device,
                        &render_state.queue,
                        "vism filter output A",
                        &premultiplied_blur_output(),
                    ),
                    upload_premultiplied(
                        &render_state.device,
                        &render_state.queue,
                        "vism filter output B",
                        &premultiplied_blur_with_markers(),
                    ),
                ];
            }
            // 途中で別のtextureへ差し替える。entity pathもtexture keyも同じなので、
            // importが毎回取り直していなければAが出続ける = ダブルバッファが効かない。
            let index = usize::from(self.frame_count >= SWAP_AT_FRAME);
            self.stage
                .copy_gpu_image(&render_ctx, FOREGROUND_IMAGE_PATH, &self.filter_output[index])
                .map_err(|error| format!("hand Vism GPU output to stage: {error}"))?;
            // 既定カメラのscene fitに任せる。前景が描かれない場合にbboxが無く、
            // focusが効いたかどうかで判定がぶれるのを避ける。
            let _ = self.focus_frames;
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
                        "rerun-vism-gpu-alpha-probe: captured {} ({}x{})",
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
        ui.label("Route: SpatialStage::copy_gpu_image -> Image visualizer -> RectangleRenderer.");
        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
        self.drive_capture(ui.ctx());
    }
}

/// 背面はCPU由来の不透明checkerboard。`copy_gpu_image`が置く前景と同じ矩形へ重ねる。
fn ingest_background(stage: &mut SpatialStage) -> Result<(), String> {
    let format =
        ImageFormat::from_color_model([WIDTH, HEIGHT], ColorModel::RGBA, ChannelDatatype::U8);
    let background = GridMap::new(checkerboard(), format, 1.0 / HEIGHT as f32);
    let chunk = Chunk::builder(BACKGROUND_IMAGE_PATH)
        .with_archetype(RowId::new(), TimePoint::STATIC, &background)
        .build()
        .map_err(|error| format!("build Rerun GridMap evidence: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(chunk))
        .map_err(|error| format!("ingest Rerun GridMap evidence: {error}"))?;

    // `copy_gpu_image` が前景へ置くのと同じ「原点中心・縦1.0」の矩形へ揃える。
    let aspect = WIDTH as f32 / HEIGHT as f32;
    ingest_transform(
        stage,
        BACKGROUND_PARENT_PATH,
        &Transform3D::from_translation([-0.5 * aspect, -0.5, 0.0]),
    )
}

/// 対照群。`copy_gpu_image`を使わない普通の`Image`を、同じ3D Stageの隣に置く。
/// これが出て前景が出ないなら、原因はGPU copy側にある。両方出ないならImage自体が3Dで描かれない。
fn ingest_control_image(stage: &mut SpatialStage) -> Result<(), String> {
    let format =
        ImageFormat::from_color_model([WIDTH, HEIGHT], ColorModel::RGBA, ChannelDatatype::U8);
    let control =
        re_sdk_types::archetypes::Image::new(straight_alpha_blur_output(), format.clone());
    // 同じ矩形へ既知の良品も置く。こちらが出て`Image`が出ないなら差はvisualizerにある。
    let grid = GridMap::new(straight_alpha_blur_output(), format.clone(), 1.0 / HEIGHT as f32);
    let grid_chunk = Chunk::builder(CONTROL_GRID_PATH)
        .with_archetype(RowId::new(), TimePoint::STATIC, &grid)
        .build()
        .map_err(|error| format!("build Rerun control GridMap: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(grid_chunk))
        .map_err(|error| format!("ingest Rerun control GridMap: {error}"))?;
    ingest_transform(
        stage,
        CONTROL_GRID_PARENT_PATH,
        // 同時対照。CPU経路の`GridMap`は無改変pinのままなので黒い矩形で残るはず。
        &Transform3D::from_translation([1.05, -0.5, 0.0]),
    )?;

    let chunk = Chunk::builder(CONTROL_IMAGE_PATH)
        .with_archetype(RowId::new(), TimePoint::STATIC, &control)
        .with_archetype(
            RowId::new(),
            TimePoint::STATIC,
            &Transform3D::from_translation_scale(
                [3.0, -0.5, 0.0],
                [1.0 / HEIGHT as f32, 1.0 / HEIGHT as f32, 1.0],
            ),
        )
        .build()
        .map_err(|error| format!("build Rerun control Image: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(chunk))
        .map_err(|error| format!("ingest Rerun control Image: {error}"))?;

    // 同じ`Image`。transformは自分ではなく親に置く。
    let parented = re_sdk_types::archetypes::Image::new(straight_alpha_blur_output(), format);
    let parented_chunk = Chunk::builder(CONTROL_PARENTED_IMAGE_PATH)
        .with_archetype(RowId::new(), TimePoint::STATIC, &parented)
        .build()
        .map_err(|error| format!("build Rerun parented Image: {error}"))?;
    stage
        .ingest_chunk(std::sync::Arc::new(parented_chunk))
        .map_err(|error| format!("ingest Rerun parented Image: {error}"))?;
    ingest_transform(
        stage,
        CONTROL_PARENTED_PARENT_PATH,
        &Transform3D::from_translation_scale(
            [-1.05 - 1.777, -0.5, 0.0],
            [1.0 / HEIGHT as f32, 1.0 / HEIGHT as f32, 1.0],
        ),
    )
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

/// Vism Filterの出力に相当する、透明gutterを持つpremultiplied RGBAをGPUへ置く。
fn upload_premultiplied(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    rgba: &[u8],
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH * 4),
            rows_per_image: Some(HEIGHT),
        },
        size,
    );
    queue.submit([]);
    texture
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

/// 差し替え後のフレーム。blurは同じだが、画像空間の四隅に不透明マーカーを置いて
/// (1) 差し替えが効いたか (2) 行0が3Dのどちら側へ行くか を同時に読めるようにする。
fn premultiplied_blur_with_markers() -> Vec<u8> {
    const MARK: u32 = 48;
    let mut rgba = premultiplied_blur_output();
    let mut put = |x0: u32, y0: u32, color: [u8; 3]| {
        for y in y0..y0 + MARK {
            for x in x0..x0 + MARK {
                let index = ((y * WIDTH + x) * 4) as usize;
                rgba[index] = color[0];
                rgba[index + 1] = color[1];
                rgba[index + 2] = color[2];
                rgba[index + 3] = 255;
            }
        }
    };
    // 画像の左上=緑、右下=白。alpha=255なのでRGBはそのままpremultiplied。
    put(0, 0, [40, 220, 80]);
    put(WIDTH - MARK, HEIGHT - MARK, [255, 255, 255]);
    rgba
}

/// 対照群用。同じ形状のstraight alpha版。CPU経路の`DontKnow`挙動もここで同時に見える。
fn straight_alpha_blur_output() -> Vec<u8> {
    let mut rgba = premultiplied_blur_output();
    for pixel in rgba.chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]) / 255.0;
        if alpha > 0.0 {
            pixel[0] = (f32::from(pixel[0]) / alpha).round().min(255.0) as u8;
            pixel[1] = (f32::from(pixel[1]) / alpha).round().min(255.0) as u8;
            pixel[2] = (f32::from(pixel[2]) / alpha).round().min(255.0) as u8;
        }
    }
    rgba
}

/// 出力rectは中心の素材より広く、外縁はalpha=0のgutterである。RGBはalphaで乗算済み。
fn premultiplied_blur_output() -> Vec<u8> {
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
            rgba[index] = (255.0 * alpha).round() as u8;
            rgba[index + 1] = (49.0 * alpha).round() as u8;
            rgba[index + 2] = (143.0 * alpha).round() as u8;
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
    fn foreground_gutter_is_fully_transparent() {
        let pixels = premultiplied_blur_output();
        assert_eq!(&pixels[0..4], &[0, 0, 0, 0]);
    }

    #[test]
    fn foreground_core_is_premultiplied() {
        let pixels = premultiplied_blur_output();
        let center = (((HEIGHT / 2) * WIDTH + WIDTH / 2) * 4) as usize;
        assert_eq!(pixels[center + 3], 235);
        assert_eq!(pixels[center], 235);
    }
}
