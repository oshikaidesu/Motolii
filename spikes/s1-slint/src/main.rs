//! M0-S1スパイク(Slint版): UI基盤の合否判定。
//!
//! 検証項目(docs/specs/M0-spikes.md):
//! 1. wgpuテクスチャのゼロコピー埋め込み(Slintと同一デバイスでレンダ→Image化)
//! 2. 30fpsでのテクスチャ更新がUI操作と共存するか
//! 3. 日本語ラベル表示と日本語IME入力(LineEditに変換しながら入力できるか)
//! 4. タイムライン風のカスタムウィジェット操作(ドラッグでプレイヘッド移動)
//!
//! 実行: `cargo run` (開発主機で。GUIが必要)

use std::cell::RefCell;
use std::rc::Rc;

use slint::wgpu_29::wgpu;

slint::slint! {
    import { LineEdit, VerticalBox } from "std-widgets.slint";

    export component SpikeWindow inherits Window {
        title: "S1 Slint スパイク — プレビュー/IME/タイムライン検証";
        preferred-width: 960px;
        preferred-height: 640px;

        in-out property <image> preview-texture;
        in-out property <float> playhead: 0.2; // 0..1
        in-out property <string> ime-result;
        callback playhead-moved(float);

        VerticalBox {
            Text {
                text: "プレビュー(wgpuテクスチャ直接埋め込み)";
                font-size: 14px;
            }
            Image {
                source: root.preview-texture;
                min-height: 360px;
                image-fit: contain;
            }

            Text { text: "日本語入力テスト(IMEで変換して確定できるか):"; }
            LineEdit {
                placeholder-text: "ここに日本語を入力…";
                edited(text) => { root.ime-result = text; }
            }
            Text { text: "入力内容: " + root.ime-result; }

            Text { text: "タイムライン風ドラッグ(プレイヘッド: " + round(root.playhead * 100) / 100 + ")"; }
            timeline := Rectangle {
                height: 48px;
                background: #202028;
                border-radius: 4px;

                // ビートグリッド風の目盛り
                for i in 16: Rectangle {
                    x: (parent.width / 16) * i;
                    width: 1px;
                    height: parent.height;
                    background: mod(i, 4) == 0 ? #50505c : #34343c;
                }

                // プレイヘッド
                Rectangle {
                    x: parent.width * root.playhead - self.width / 2;
                    width: 3px;
                    height: parent.height;
                    background: #ff5f45;
                }

                TouchArea {
                    moved => {
                        if (self.pressed) {
                            root.playhead = max(0.0, min(1.0, self.mouse-x / parent.width));
                            root.playhead-moved(root.playhead);
                        }
                    }
                    clicked => {
                        root.playhead = max(0.0, min(1.0, self.mouse-x / parent.width));
                        root.playhead-moved(root.playhead);
                    }
                }
            }
        }
    }
}

const TEX_W: u32 = 1280;
const TEX_H: u32 = 720;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
}

/// 動くテストパターンをGPU上のクリアだけで描く(CPU転送ゼロ)。
/// 本実装(M1)ではここが render_frame(t, Quality) に置き換わる。
fn render_pattern(gpu: &Gpu, t: f32) {
    let view = gpu.texture.create_view(&Default::default());
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // 背景: 時間で色相が回る(30fps更新の体感確認用)
    let (r, g, b) = hsv((t * 40.0) % 360.0, 0.5, 0.25);
    enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("bg"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: r as f64,
                    g: g as f64,
                    b: b as f64,
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

    gpu.queue.submit([enc.finish()]);
}

fn hsv(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h as u32 / 60) % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::default())
        .select()?;

    let app = SpikeWindow::new()?;
    let gpu: Rc<RefCell<Option<Gpu>>> = Rc::new(RefCell::new(None));

    // Slintと同一のdevice/queueを取得する(ゼロコピー共有の要)
    let gpu_setup = gpu.clone();
    app.window()
        .set_rendering_notifier(move |state, graphics_api| {
            let (
                slint::RenderingState::RenderingSetup,
                slint::GraphicsAPI::WGPU29 { device, queue, .. },
            ) = (state, graphics_api)
            else {
                return;
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("preview"),
                size: wgpu::Extent3d {
                    width: TEX_W,
                    height: TEX_H,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            *gpu_setup.borrow_mut() = Some(Gpu {
                device: device.clone(),
                queue: queue.clone(),
                texture,
            });
        })?;

    // 30fpsでテストパターンを更新
    let app_weak = app.as_weak();
    let start = std::time::Instant::now();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(33),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let gpu_ref = gpu.borrow();
            let Some(g) = gpu_ref.as_ref() else { return };
            render_pattern(g, start.elapsed().as_secs_f32());
            if let Ok(img) = slint::Image::try_from(g.texture.clone()) {
                app.set_preview_texture(img);
            }
        },
    );

    app.run()?;
    Ok(())
}
