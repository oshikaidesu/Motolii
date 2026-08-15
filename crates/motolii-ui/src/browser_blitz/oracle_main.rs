//! C6 POSITIVE ORACLE の実行体: 元寸PNG 45枚を表示して60秒 panic しないこと。
//!
//! ```text
//! MOTOLII_BROWSER_DIR=/path/to/folder \
//!   cargo run -p motolii-ui --bin motolii-browser-blitz-oracle
//! ```
//! `MOTOLII_BROWSER_SECONDS` で走行秒数、`MOTOLII_BROWSER_MAX` で枚数上限を変える。
//!
//! ドラッグ中の絵はホスト(egui)側が描く。Blitz面は格子だけを持つ。

use std::path::PathBuf;
use std::time::{Duration, Instant};

use motolii_ui::browser_blitz::{BrowserBlitzPanel, DEFAULT_MAX_ITEMS};

const W: u32 = 900;
const H: u32 = 520;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn env_usize(key: &str, fallback: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

fn main() -> eframe::Result<()> {
    let dir = PathBuf::from(
        std::env::var("MOTOLII_BROWSER_DIR").unwrap_or_else(|_| "docs/mocks".to_string()),
    );
    let max_items = env_usize("MOTOLII_BROWSER_MAX", DEFAULT_MAX_ITEMS);
    let seconds = env_usize("MOTOLII_BROWSER_SECONDS", 60) as u64;

    eframe::run_native(
        "C6 oracle: Browser (Blitz)",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([940.0, 640.0]),
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(Oracle::new(cc, dir, max_items, seconds)))),
    )
}

struct Oracle {
    panel: BrowserBlitzPanel,
    view: wgpu::TextureView,
    texture_id: egui::TextureId,
    started: Instant,
    limit: Duration,
    frames: u64,
    note: String,
}

impl Oracle {
    fn new(
        cc: &eframe::CreationContext<'_>,
        dir: PathBuf,
        max_items: usize,
        seconds: u64,
    ) -> Self {
        let state = cc.wgpu_render_state.as_ref().expect("wgpu render state");
        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("browser-blitz-oracle"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_id =
            state
                .renderer
                .write()
                .register_native_texture(&state.device, &view, wgpu::FilterMode::Linear);
        let panel = BrowserBlitzPanel::from_folder(
            &state.device,
            &state.adapter,
            &state.queue,
            FORMAT,
            W,
            H,
            &dir,
            max_items,
        )
        .expect("browser panel");
        eprintln!("C6 oracle: {} → {} 件", dir.display(), panel.items().len());
        Self {
            panel,
            view,
            texture_id,
            started: Instant::now(),
            limit: Duration::from_secs(seconds),
            frames: 0,
            note: String::new(),
        }
    }
}

impl eframe::App for Oracle {
    // このworkspaceのeframeは `App::ui` を要求する(probeと同じ形)。
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();
        {
            let elapsed = self.started.elapsed();
            ui.label(format!(
                "elapsed {:.1}s / {:.0}s  frames {}  items {}  atlas images {}  {}",
                elapsed.as_secs_f32(),
                self.limit.as_secs_f32(),
                self.frames,
                self.panel.items().len(),
                self.panel.cached_image_count(),
                self.note
            ));
            ui.separator();

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(W as f32, H as f32),
                egui::Sense::click_and_drag(),
            );
            if let Some(pos) = response.hover_pos().or_else(|| ctx.pointer_latest_pos()) {
                let (x, y) = ((pos.x - rect.min.x) as f64, (pos.y - rect.min.y) as f64);
                self.panel.pointer_move(x, y);
                if response.drag_started() || response.clicked() {
                    self.panel.pointer_down(x, y);
                }
                if response.drag_stopped() {
                    if let Some(release) = self.panel.pointer_up(x, y) {
                        // 配置intentの意味は未決。ここでDocumentへ触らない。
                        self.note = format!(
                            "release: {} @({:.0},{:.0}) inside={}",
                            release.item.name, release.pointer.0, release.pointer.1,
                            release.inside_panel
                        );
                    }
                }
            }

            let state = frame.wgpu_render_state().expect("wgpu render state");
            let (device, queue) = (state.device.clone(), state.queue.clone());
            self.panel.render(&device, &queue, &self.view);
            self.frames += 1;

            ui.painter().image(
                self.texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            // ドラッグ中の絵はホスト側が持つ(probe P6の原則)。
            if let (Some(drag), Some(pos)) = (self.panel.drag(), ctx.pointer_latest_pos()) {
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(pos, egui::vec2(120.0, 22.0)),
                    2.0,
                    egui::Color32::from_black_alpha(200),
                );
                ui.painter().text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    &self.panel.items()[drag.index].name,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }

            if elapsed >= self.limit {
                println!(
                    "C6 POSITIVE ORACLE PASS: items={} frames={} atlas_images={} elapsed={:.1}s",
                    self.panel.items().len(),
                    self.frames,
                    self.panel.cached_image_count(),
                    elapsed.as_secs_f32()
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}
