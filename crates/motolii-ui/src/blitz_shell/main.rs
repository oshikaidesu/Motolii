//! 移植済みの Blitz パネルを1つの窓へ合体させて出すだけの実行体。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-shell
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --project my-project.json
//! cargo run -p motolii-ui --bin motolii-blitz-shell -- --screenshot out/shell.png [frames]
//! ```
//!
//! 中身は `crates/motolii-ui/src/blitz_shell/` にある。この bin は窓を開くだけで、
//! **見た目も配置もここで決めない。**
//!
//! `--project` は実プロジェクトを開き、Timeline / Stage がその Document
//! （writer の snapshot）を映す。**開けなければ起動失敗**で、fixture へ黙って落ちない。
//! 開く場所を窓より前に置いてあるのは、失敗を絵ではなく exit code とメッセージで
//! 返すため。`--project` 無しは従来どおり fixture 展示（開発動線・screenshot テスト）。
//!
//! `--screenshot` は窓を1枚だけ描いてPNGにし、そのまま終了する。
//! 合体した絵を**人が窓を開かずに確認する**ための口で、製品機能ではない。
//! `frames`(既定10)を待つのは、Browserパネルの画像取得が非同期で、
//! 1フレーム目ではサムネイルがまだ届かないため(`blitz_dump/main.rs:202-204` と同じ理由)。
//!
//! 窓の最小寸法は `ui/motolii-rn/src/productStyles.ts:4` の `shell`
//! (`minWidth: 980, minHeight: 650`) をそのまま使う。新しい値ではない。

use std::path::{Path, PathBuf};

use image::ImageEncoder as _;
use motolii_ui::blitz_shell::{BlitzShellApp, ProjectSeat};

/// productStyles.ts:4 `shell: {minWidth: 980, minHeight: 650}`
const MIN_WIDTH: f32 = 980.0;
const MIN_HEIGHT: f32 = 650.0;

/// 既定の待ちフレーム数。Browserの画像が届くのを待つ。
const DEFAULT_FRAMES: u32 = 10;

fn usage() -> ! {
    eprintln!(
        "usage: motolii-blitz-shell [--project <project.json>] [--screenshot <out.png> [frames]]"
    );
    std::process::exit(2);
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut project: Option<PathBuf> = None;
    let mut shot: Option<Screenshot> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--project" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                project = Some(PathBuf::from(path));
                index += 2;
            }
            "--screenshot" => {
                let Some(path) = args.get(index + 1) else {
                    usage()
                };
                let mut after = DEFAULT_FRAMES;
                index += 2;
                // frames は省略可能（次が数値のときだけ食う）。
                if let Some(frames) = args.get(index).and_then(|value| value.parse().ok()) {
                    after = frames;
                    index += 1;
                }
                shot = Some(Screenshot {
                    path: PathBuf::from(path),
                    after,
                    requested: false,
                });
            }
            _ => usage(),
        }
    }

    // project は**窓より先に**開く。開けなければ起動失敗（fixture へ黙って落ちない）。
    let seat = match project {
        Some(path) => match ProjectSeat::open(&path) {
            Ok(seat) => Some(seat),
            Err(error) => {
                eprintln!("motolii-blitz-shell: {error}");
                std::process::exit(1);
            }
        },
        None => None,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([MIN_WIDTH, MIN_HEIGHT])
            .with_min_inner_size([MIN_WIDTH, MIN_HEIGHT])
            .with_title("Motolii — Blitz shell"),
        ..Default::default()
    };
    eframe::run_native(
        "motolii-blitz-shell",
        options,
        Box::new(move |cc| {
            Ok(Box::new(Harness {
                inner: BlitzShellApp::with_seat(cc, seat),
                shot,
                frame_count: 0,
            }))
        }),
    )
}

struct Screenshot {
    path: PathBuf,
    after: u32,
    requested: bool,
}

/// `BlitzShellApp` をそのまま包んで、`--screenshot` のときだけ
/// 1枚撮って閉じる。**殻の側には撮影の都合を1つも足さない。**
struct Harness {
    inner: BlitzShellApp,
    shot: Option<Screenshot>,
    frame_count: u32,
}

impl eframe::App for Harness {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.inner.ui(ui, frame);
        let ctx = ui.ctx().clone();
        let ctx = &ctx;

        let Some(shot) = self.shot.as_mut() else {
            return;
        };
        self.frame_count += 1;

        // 届いていれば書いて閉じる。
        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = captured {
            let [width, height] = image.size;
            write_png(&shot.path, image.as_raw(), width as u32, height as u32);
            println!("blitz-shell: {} ({width}x{height})", shot.path.display());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if !shot.requested && self.frame_count >= shot.after {
            shot.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        // 撮るまでは自分で回す(入力が無いと再描画が来ないため)。
        ctx.request_repaint();
    }
}

/// 撮った1枚を8bit RGBAのPNGにする。`rgba` は行頭パディング無しの `w * h * 4` バイト
/// (`egui::ColorImage::as_raw()` がそのまま渡せる)。
fn write_png(path: &Path, rgba: &[u8], w: u32, h: u32) {
    let file = std::fs::File::create(path).expect("png create");
    image::codecs::png::PngEncoder::new(std::io::BufWriter::new(file))
        .write_image(rgba, w, h, image::ExtendedColorType::Rgba8)
        .expect("png write");
}
