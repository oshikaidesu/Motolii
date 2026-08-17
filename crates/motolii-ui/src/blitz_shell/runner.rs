//! 窓を開いて `BlitzShellApp` を回す実行部。
//!
//! ここが lib 側にあるのは U0a の境界規律のため: **公開APIは toolkit 型を漏らさない**。
//! bin(`main.rs`)は引数を [`BlitzShellLaunch`] に写して [`run_blitz_shell`] を呼ぶだけで、
//! eframe / egui はこの module の内側に閉じる(旧 `run_shell` と同じ家の型)。

use std::path::{Path, PathBuf};

use image::ImageEncoder as _;

use super::drive::ShellTranscript;
use super::{BlitzShellApp, ProjectSeat};
use crate::ShellError;

/// productStyles.ts:4 `shell: {minWidth: 980, minHeight: 650}`
const MIN_WIDTH: f32 = 980.0;
const MIN_HEIGHT: f32 = 650.0;

/// 既定の待ちフレーム数。Browserの画像が届くのを待つ。
pub const DEFAULT_SCREENSHOT_FRAMES: u32 = 10;

/// `--screenshot` の要求。窓を1枚だけ描いてPNGにし、そのまま終了する。
/// 合体した絵を**人が窓を開かずに確認する**ための口で、製品機能ではない。
pub struct ScreenshotRequest {
    pub path: PathBuf,
    /// 待つフレーム数。Browserパネルの画像取得が非同期で、
    /// 1フレーム目ではサムネイルがまだ届かないため(`blitz_dump/main.rs:202-204` と同じ理由)。
    pub frames: u32,
}

/// 起動要求。toolkit 型を含まない。
pub struct BlitzShellLaunch {
    /// 実プロジェクト。開けなければ**窓より前に**失敗を返す(fixture へ黙って落ちない)。
    /// `None` は従来どおり fixture 展示(開発動線・screenshot テスト)。
    pub project: Option<PathBuf>,
    pub screenshot: Option<ScreenshotRequest>,
    /// fixture 展示（開発動線・screenshot テスト用）。既定 false = 座席なしは
    /// スタート画面。
    pub fixture: bool,
    /// 窓が言ったことを機械可読で外へ流す先（`--status-log`）。
    ///
    /// 毎フレーム、`ShellTranscript` に増えた分だけ JSONL
    /// (`{"seq":n,"text":"…"}`) を追記して flush する。CLI から窓を駆動する実行は
    /// **必ず失敗の記録を持つ**ための口で、製品機能ではない。
    pub status_log: Option<PathBuf>,
}

/// 窓を開いて shell を回す。公開 API はこの1本で、署名は toolkit-free。
pub fn run_blitz_shell(launch: BlitzShellLaunch) -> Result<(), ShellError> {
    // project は**窓より先に**開く。失敗は絵ではなく Err で返す。
    let seat = match launch.project {
        Some(path) => Some(
            ProjectSeat::open(&path)
                .map_err(|message| ShellError::AppConstruction(message.into()))?,
        ),
        None => None,
    };
    let launch_fixture = launch.fixture;
    let shot = launch.screenshot.map(|request| Screenshot {
        path: request.path,
        after: request.frames,
        requested: false,
    });
    // 記録先は**窓より先に**開く。開けないなら起動失敗にする — 「記録している
    // つもりで何も残っていない」実行を作らない。
    let status_log = match launch.status_log {
        Some(path) => Some(
            StatusLog::create(&path)
                .map_err(|message| ShellError::AppConstruction(message.into()))?,
        ),
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
            let inner = BlitzShellApp::with_seat(cc, seat, launch_fixture);
            let transcript = inner.transcript().clone();
            Ok(Box::new(Harness {
                inner,
                transcript,
                status_log,
                written: 0,
                shot,
                frame_count: 0,
            }))
        }),
    )
    .map_err(|error| ShellError::Runtime(Box::new(error)))
}

struct Screenshot {
    path: PathBuf,
    after: u32,
    requested: bool,
}

/// 窓が言ったことの追記先（`--status-log`）。
struct StatusLog {
    file: std::io::BufWriter<std::fs::File>,
}

impl StatusLog {
    fn create(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("status log {} を作れない: {error}", path.display()))?;
        }
        let file = std::fs::File::create(path)
            .map_err(|error| format!("status log {} を作れない: {error}", path.display()))?;
        Ok(Self {
            file: std::io::BufWriter::new(file),
        })
    }

    /// 1行 = 1 report。書けなかった時は**黙らない**が、窓は落とさない
    /// （記録先が消えても編集中の project を失わせない）。
    fn append(&mut self, seq: u64, text: &str) {
        use std::io::Write as _;
        let line = serde_json::json!({ "seq": seq, "text": text });
        if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
            println!("blitz-shell: status log へ書けない: {error}");
        }
    }
}

/// `BlitzShellApp` をそのまま包んで、`--screenshot` のときだけ
/// 1枚撮って閉じ、`--status-log` のときだけ言われたことを流す。
/// **殻の側には撮影も記録の都合も1つも足さない。**
struct Harness {
    inner: BlitzShellApp,
    /// 殻の台帳（`inner` と同じもの）。毎フレーム増えた分だけ log へ流す。
    transcript: ShellTranscript,
    status_log: Option<StatusLog>,
    /// log へ既に流した行数。
    written: usize,
    shot: Option<Screenshot>,
    frame_count: u32,
}

impl eframe::App for Harness {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.inner.ui(ui);

        // 言われたことは描いた直後に流す。1行ずつ flush するので、窓が固まっても
        // 落ちても、そこまでの記録は外に残る。
        if let Some(log) = self.status_log.as_mut() {
            for event in self.transcript.since(self.written) {
                log.append(event.seq, &event.text);
            }
            self.written = self.transcript.len();
        }

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
