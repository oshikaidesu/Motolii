//! R9: 実素材プレビュー(S1方式)。`render_frame`経路をGUIで確認する最小スパイク。
//!
//! 実行: `cargo run -- /path/to/project.json` (開発主機・GUI必須)

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};

use motolii_cli::prepare_project_export;
use motolii_gpu::{upload_rgba, GpuCtx, RgbaDownloader, YuvToRgba};
use motolii_render::RenderSession;
use slint::wgpu_29::wgpu;

slint::slint! {
    import { VerticalBox, Slider } from "std-widgets.slint";

    export component PreviewWindow inherits Window {
        title: "R9 Preview — render_frame path (same as export)";
        preferred-width: 960px;
        preferred-height: 720px;

        in-out property <image> preview-texture;
        in-out property <float> frame-slider: 0.0;
        in-out property <string> status-text: "";
        callback frame-changed(float);

        VerticalBox {
            Text { text: root.status-text; font-size: 14px; }
            Image {
                source: root.preview-texture;
                min-height: 480px;
                image-fit: contain;
            }
            Slider {
                minimum: 0.0;
                maximum: 1.0;
                value: root.frame-slider;
                changed(val) => {
                    root.frame-slider = val;
                    root.frame-changed(val);
                }
            }
            Text { text: "スライダーで書き出し範囲内のフレームをスクラブ"; font-size: 12px; }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_path = env_project_path()?;
    let prepared = prepare_project_export(&project_path)?;
    let export_frames = prepared.export_frames.max(1);

    let (gpu, parts) = GpuCtx::new_for_ui()?;
    if let Some(info) = &gpu.adapter_info {
        eprintln!("adapter: {} ({:?})", info.name, info.backend);
    }
    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Manual {
            instance: parts.instance,
            adapter: parts.adapter,
            device: parts.device,
            queue: parts.queue,
        })
        .select()?;

    let app = PreviewWindow::new()?;
    app.set_status_text(format!(
        "{} — {} frames @ {}/{} fps ({}x{})",
        project_path.display(),
        export_frames,
        prepared.info.fps.num(),
        prepared.info.fps.den(),
        prepared.info.width,
        prepared.info.height
    ).into());

    let frame_index = Arc::new(AtomicUsize::new(0));
    let requested = frame_index.clone();
    let (tx, rx) = mpsc::sync_channel::<wgpu::Texture>(1);

    let render_prepared = prepared.clone();
    std::thread::spawn(move || {
        let mut session = RenderSession::new(&gpu);
        let mut yuv = YuvToRgba::new(&gpu);
        let mut downloader = RgbaDownloader::new();
        let mut last_index = usize::MAX;
        loop {
            let index = requested.load(Ordering::Relaxed).min(export_frames - 1);
            if index != last_index {
                last_index = index;
                if let Ok(rgba) = render_prepared.render_export_frame_rgba(
                    &gpu,
                    index,
                    &mut session,
                    &mut yuv,
                    &mut downloader,
                ) {
                    let texture = upload_rgba(&gpu, &render_prepared.render_desc, &rgba);
                    let _ = tx.try_send(texture);
                } else {
                    eprintln!("r9-preview: render_export_frame_rgba failed for index {index}");
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    let frame_for_ui = frame_index.clone();
    let app_weak = app.as_weak();
    app.on_frame_changed(move |val| {
        let index = (val * (export_frames.saturating_sub(1)) as f32).round() as usize;
        frame_for_ui.store(index, Ordering::Relaxed);
        if let Some(app) = app_weak.upgrade() {
            app.set_status_text(format!("preview frame {index} / {}", export_frames - 1).into());
        }
    });

    let app_weak = app.as_weak();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            if let Ok(texture) = rx.try_recv() {
                match slint::Image::try_from(texture) {
                    Ok(img) => app.set_preview_texture(img),
                    Err(e) => eprintln!("r9-preview: Image::try_from failed: {e:?}"),
                }
            }
        },
    );

    app.run()?;
    Ok(())
}

fn env_project_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: r9-preview <project.json>")?;
    Ok(PathBuf::from(path))
}
