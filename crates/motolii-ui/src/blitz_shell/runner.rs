//! 窓を開いて `BlitzShellApp` を回す実行部。
//!
//! ここが lib 側にあるのは U0a の境界規律のため: **公開APIは toolkit 型を漏らさない**。
//! bin(`main.rs`)は引数を [`BlitzShellLaunch`] に写して [`run_blitz_shell`] を呼ぶだけで、
//! eframe / egui はこの module の内側に閉じる(旧 `run_shell` と同じ家の型)。

use std::path::{Path, PathBuf};

use image::ImageEncoder as _;

use super::drive::ShellTranscript;
use super::intent::IntentJournal;
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
    /// 窓で**起きた原因**を機械可読で外へ流す先（`--intent-log`）。
    ///
    /// `--status-log` と同型の JSONL (`{"seq":n,"intent":{"kind":"…",…}}`) で、
    /// `IntentJournal` に増えた分だけ追記して flush する。並べて読むと
    /// 「何をしようとして（intent）、何が起きたか（status）」が揃う。
    /// 出た列はそのまま `ShellGateway::replay` へ食わせられる。
    pub intent_log: Option<PathBuf>,
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
            JsonlLog::create(&path, "status log")
                .map_err(|message| ShellError::AppConstruction(message.into()))?,
        ),
        None => None,
    };
    let intent_log = match launch.intent_log {
        Some(path) => Some(
            JsonlLog::create(&path, "intent log")
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
            let journal = inner.intent_journal().clone();
            Ok(Box::new(Harness {
                inner,
                transcript,
                journal,
                status_log,
                intent_log,
                written: 0,
                dispatched: 0,
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

/// 窓の記録の追記先（`--status-log` / `--intent-log`）。**同じ形の JSONL** で、
/// 中身（1行に何を書くか）は呼び手が決める。
struct JsonlLog {
    file: std::io::BufWriter<std::fs::File>,
    /// 失敗を言うときの名乗り（"status log" / "intent log"）。
    what: &'static str,
}

impl JsonlLog {
    fn create(path: &Path, what: &'static str) -> Result<Self, String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{what} {} を作れない: {error}", path.display()))?;
        }
        let file = std::fs::File::create(path)
            .map_err(|error| format!("{what} {} を作れない: {error}", path.display()))?;
        Ok(Self {
            file: std::io::BufWriter::new(file),
            what,
        })
    }

    /// 1行 = 1 件。台帳の event 型をそのまま流すので、行の形（field の順）は
    /// 型の宣言そのものである。書けなかった時は**黙らない**が、窓は落とさない
    /// （記録先が消えても編集中の project を失わせない）。
    fn append(&mut self, event: &impl serde::Serialize) {
        use std::io::Write as _;
        let what = self.what;
        let line = match serde_json::to_string(event) {
            Ok(line) => line,
            Err(error) => {
                println!("blitz-shell: {what} の1行を組めない: {error}");
                return;
            }
        };
        if let Err(error) = writeln!(self.file, "{line}").and_then(|()| self.file.flush()) {
            println!("blitz-shell: {what} へ書けない: {error}");
        }
    }
}

/// `BlitzShellApp` をそのまま包んで、`--screenshot` のときだけ 1枚撮って閉じ、
/// `--status-log` / `--intent-log` のときだけ**結果**と**原因**を流す。
/// **殻の側には撮影も記録の都合も1つも足さない。**
struct Harness {
    inner: BlitzShellApp,
    /// 殻の台帳（`inner` と同じもの）。毎フレーム増えた分だけ log へ流す。
    transcript: ShellTranscript,
    /// 原因のログ（`inner` と同じもの）。
    journal: IntentJournal,
    status_log: Option<JsonlLog>,
    intent_log: Option<JsonlLog>,
    /// status log へ既に流した行数。
    written: usize,
    /// intent log へ既に流した行数。
    dispatched: usize,
    shot: Option<Screenshot>,
    frame_count: u32,
}

impl eframe::App for Harness {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.inner.ui(ui);

        // 記録は描いた直後に流す。1行ずつ flush するので、窓が固まっても
        // 落ちても、そこまでの記録は外に残る。**原因を先に**流すのは、
        // 記録の順が「intent → その結果の status」だから。
        if let Some(log) = self.intent_log.as_mut() {
            for event in self.journal.since(self.dispatched) {
                log.append(&event);
            }
            self.dispatched = self.journal.len();
        }
        if let Some(log) = self.status_log.as_mut() {
            for event in self.transcript.since(self.written) {
                log.append(&event);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blitz_shell::{IntentEvent, UiIntent};

    /// `--intent-log` / `--status-log` の2本が**同型の JSONL**で並ぶ。
    /// intent 側は dialog の答え(path)まで運ぶので、出た列はそのまま replay
    /// に食わせられる。窓を開かずに、外へ出る1行の形そのものを固定する。
    #[test]
    fn the_two_logs_write_the_same_jsonl_shape() {
        let dir = motolii_testkit::tmp_dir("runner_jsonl");
        let intents = dir.join("nested/intents.jsonl");
        let statuses = dir.join("statuses.jsonl");

        // 親ディレクトリが無くても作る(記録先の不在で起動を落とさない)。
        let mut intent_log = JsonlLog::create(&intents, "intent log").expect("intent log を開く");
        let mut status_log = JsonlLog::create(&statuses, "status log").expect("status log を開く");
        intent_log.append(&IntentEvent {
            seq: 1,
            intent: UiIntent::NewProject {
                path: PathBuf::from("/tmp/fresh.json"),
            },
        });
        status_log.append(&super::super::drive::StatusEvent {
            seq: 1,
            text: "opened /tmp/fresh.json".to_owned(),
        });
        drop(intent_log);
        drop(status_log);

        assert_eq!(
            std::fs::read_to_string(&intents).expect("intent log を読む"),
            "{\"seq\":1,\"intent\":{\"kind\":\"new_project\",\"path\":\"/tmp/fresh.json\"}}\n",
            "原因の1行: 答え(path)ごと機械可読で出る"
        );
        assert_eq!(
            std::fs::read_to_string(&statuses).expect("status log を読む"),
            "{\"seq\":1,\"text\":\"opened /tmp/fresh.json\"}\n",
            "結果の1行は従来のまま(並べて読める)"
        );
    }
}
