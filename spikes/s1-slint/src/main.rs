//! M0-S1スパイク(Slint版): UI基盤の合否判定。
//!
//! 検証項目(docs/specs/M0-spikes.md):
//! 1. **oc-gpu(コア)が作ったテクスチャをSlintに渡すE2E結線**(レビュー指摘#1):
//!    Slintのdevice/queueを`GpuCtx::from_device_queue`で共有し、oc-gpuの
//!    YUV変換シェーダの出力テクスチャを`Image::try_from`でそのまま表示する
//! 2. 30fpsでのテクスチャ更新がUI操作と共存するか
//! 3. 日本語ラベル表示と日本語IME入力(LineEditに変換しながら入力できるか)
//! 4. タイムライン風のカスタムウィジェット操作(ドラッグでプレイヘッド移動)
//!
//! 実行: `cargo run` (開発主機で。GUIが必要)

use std::cell::RefCell;
use std::rc::Rc;

use oc_core::ColorSpace;
use oc_gpu::{solid_yuv420p, GpuCtx, YuvToRgba};
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

struct Core {
    gpu: GpuCtx,
    conv: YuvToRgba,
}

/// oc-gpuのYUV変換を通したテクスチャを作る(コア→UIのE2E結線)。
/// 色相が時間で回るYUV値を作り、BT.709 limitedとして変換する。
fn render_via_core(core: &Core, t: f32) -> wgpu::Texture {
    let (y, u, v) = hue_to_yuv709((t * 40.0) % 360.0);
    let frame = solid_yuv420p(TEX_W, TEX_H, y, u, v, ColorSpace::Rec709Limited);
    core.conv.convert(&core.gpu, &frame)
}

/// 色相→BT.709 limited YUV(スパイク用の簡易変換)
fn hue_to_yuv709(h: f32) -> (u8, u8, u8) {
    let (r, g, b) = hsv(h, 0.6, 0.6);
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let cb = (b - y) / 1.8556;
    let cr = (r - y) / 1.5748;
    (
        (16.0 + 219.0 * y).round() as u8,
        (128.0 + 224.0 * cb).round() as u8,
        (128.0 + 224.0 * cr).round() as u8,
    )
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
    let core: Rc<RefCell<Option<Core>>> = Rc::new(RefCell::new(None));

    // Slintと同一のdevice/queueをoc-gpuに共有する(ゼロコピー結線の要)
    let core_setup = core.clone();
    app.window()
        .set_rendering_notifier(move |state, graphics_api| {
            let (
                slint::RenderingState::RenderingSetup,
                slint::GraphicsAPI::WGPU29 { device, queue, .. },
            ) = (state, graphics_api)
            else {
                return;
            };
            let gpu = GpuCtx::from_device_queue(device.clone(), queue.clone());
            let conv = YuvToRgba::new(&gpu);
            *core_setup.borrow_mut() = Some(Core { gpu, conv });
        })?;

    // 30fpsでコア経由のテクスチャを更新
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
            let core_ref = core.borrow();
            let Some(core) = core_ref.as_ref() else {
                return;
            };
            let texture = render_via_core(core, start.elapsed().as_secs_f32());
            if let Ok(img) = slint::Image::try_from(texture) {
                app.set_preview_texture(img);
            }
        },
    );

    app.run()?;
    Ok(())
}
