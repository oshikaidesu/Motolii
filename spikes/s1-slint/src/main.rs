//! M0-S1スパイク(Slint版): UI基盤の合否判定。
//!
//! 検証項目(docs/specs/M0-spikes.md):
//! 1. **コンポジタ要件のデバイスをSlintと共有するE2E結線**(レビュー指摘#1対応):
//!    `GpuCtx::new_for_ui()`が要件(feature/limit)を明示してデバイスを生成し、
//!    `WGPUConfiguration::Manual`でSlintに渡す。Slint任せのデバイス生成
//!    (`default()`)は使わない — 後からfeatureを足せないため
//! 2. **レンダはUIスレッドで回さない**(レビュー指摘#2対応): レンダ専用スレッドが
//!    テクスチャを生成しチャネルで送る。UIスレッドは受け取ったテクスチャを
//!    `Image::try_from`で表示するだけ
//! 3. 日本語ラベル表示と日本語IME入力(LineEditに変換しながら入力できるか)
//! 4. タイムライン風のカスタムウィジェット操作(ドラッグでプレイヘッド移動)
//!
//! 実行: `cargo run` (開発主機で。GUIが必要)

use std::sync::mpsc;

use motoly_core::ColorSpace;
use motoly_gpu::{solid_yuv420p, GpuCtx, YuvToRgba};
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
                text: "プレビュー(wgpuテクスチャ直接埋め込み・別スレッドレンダ)";
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
    // コンポジタ要件(feature/limit)を明示したデバイスを自前で生成し、
    // 同じデバイスをSlintに渡す(WGPUConfiguration::Manual)。
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

    let app = SpikeWindow::new()?;

    // レンダ専用スレッド: motoly-gpuのYUV変換でテクスチャを生成し、チャネルで送る。
    // UIスレッドはレンダしない(レビュー指摘#2)。wgpuのDevice/QueueはSend+Sync。
    let (tx, rx) = mpsc::sync_channel::<wgpu::Texture>(1);
    std::thread::spawn(move || {
        let mut conv = YuvToRgba::new(&gpu);
        let start = std::time::Instant::now();
        loop {
            let t = start.elapsed().as_secs_f32();
            let (y, u, v) = hue_to_yuv709((t * 40.0) % 360.0);
            let frame = solid_yuv420p(TEX_W, TEX_H, y, u, v, ColorSpace::Rec709Limited);
            let texture = conv.convert(&gpu, &frame);
            // 満杯なら破棄(最新フレームだけを届ける。ブロックしない)
            let _ = tx.try_send(texture);
            std::thread::sleep(std::time::Duration::from_millis(33));
        }
    });

    // UIスレッド: 届いたテクスチャを表示するだけ
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
                if let Ok(img) = slint::Image::try_from(texture) {
                    app.set_preview_texture(img);
                }
            }
        },
    );

    app.run()?;
    Ok(())
}
