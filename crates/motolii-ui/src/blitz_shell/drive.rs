//! 運転席(driver seat)— shell を外から**駆動・観測**するための座席。
//!
//! [2026-08-18決定](../../../../docs/reviews/2026-08-18-cli-gui-driver-seat.md)の実行部で、
//! 製品UIは1つも変えない。3つの座席がここに在る:
//!
//! 1. [`ShellTranscript`] … **言う場所は1つ**。窓の一言(status)は全部ここを通る。
//!    帯は `latest()` を映し、`report()` された全文が順に残る。`eprintln!` で消える
//!    失敗を無くすための受け皿でもある(`pane.rs` のフェンス:
//!    `tests/shell_error_fence.rs`)。runner の `--status-log` はここを JSONL で外へ流す。
//! 2. [`ShellPrompts`] … **dialog の台本化**。New / Open / Export / 未保存確認の4本を
//!    trait の後ろへ置く。窓は [`NativePrompts`](現挙動そのまま)、テスト・CLI 駆動は
//!    [`ScriptedPrompts`] が答える。台本の既定は「答えない」= 何も起きない。
//! 3. [`DrivenShell`] … `egui_kittest` で `BlitzShellApp` を headless に回す運転席
//!    (テスト内に閉じる)。AccessKit ラベルでクリックし、`DroppedFile` を注入し、
//!    transcript を読む。窓も native dialog も開かない。
//!
//! ここに製品の判断は無い。何が起きるかは相変わらず `app.rs` が決める。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::app::UnsavedChoice;

/// transcript の1行。`seq` は 1 始まりの通し番号で、`--status-log` の JSONL
/// (`{"seq":n,"text":"…"}`)がそのまま名乗る番号でもある。
///
/// **JSONL の1行はこの型がそのまま名乗る**(`serde` の field 順 = 宣言順)。
/// 原因の側(`intent::IntentEvent`)も同じ形で、並べて読める。
///
/// **公開している**理由: 2026-08-18 の iced ホスト移行(M-0)で、`motolii-shell-iced`
/// が同じ帯を映すため。toolkit の型は1つも入っていないので U0a の境界は破れない。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StatusEvent {
    pub seq: u64,
    pub text: String,
}

/// 窓の一言を溜める唯一の場所。
///
/// `Clone` は同じ台帳の共有(`Arc`)で、pane へ配った clone が言ったことも
/// 帯と `--status-log` に出る。**何も黙って消えない**のがこの型の全部である。
///
/// **公開している**理由: 2026-08-18 の iced ホスト移行(M-0)。transcript は
/// host 非依存の契約(裁定文書の「意味は不変のまま iced へ移る」)なので、
/// `motolii-shell-iced` は同じ型の同じ帯を映す — 2つ目の台帳を作らせない。
#[derive(Clone, Default)]
pub struct ShellTranscript {
    entries: Arc<Mutex<Vec<StatusEvent>>>,
}

impl ShellTranscript {
    /// 一言を残す。帯はこの直後から `latest()` を映す。
    pub fn report(&self, text: impl Into<String>) {
        let mut entries = self.lock();
        let seq = entries.len() as u64 + 1;
        entries.push(StatusEvent {
            seq,
            text: text.into(),
        });
    }

    /// 残っている全文(順のまま)。
    pub fn entries(&self) -> Vec<StatusEvent> {
        self.lock().clone()
    }

    /// 帯が映す最新の一言。何も言われていなければ `None`(= 帯を出さない)。
    pub fn latest(&self) -> Option<String> {
        self.lock().last().map(|event| event.text.clone())
    }

    /// 既に `count` 行を見た側が、その後に増えた分だけ受け取る(`--status-log` 用)。
    /// **`entries()` の続きから**であって、別の台帳を持たない。
    pub fn since(&self, count: usize) -> Vec<StatusEvent> {
        let mut entries = self.entries();
        if count >= entries.len() {
            return Vec::new();
        }
        entries.drain(..count);
        entries
    }

    /// 溜まっている行数。
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// lock は毒されない(この型は panic しうるコードを中で呼ばない)。
    /// 万一毒されていても台帳を失わないよう、中身をそのまま取り出す。
    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<StatusEvent>> {
        self.entries
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// 人に訊く4本。**dialog はこの trait の後ろだけ**に居る。
///
/// 何を訊くかはここ、何が起きるかは `app.rs`(`decide_unsaved` 等)という分担は
/// 変えていない — 変えたのは「訊き手を差し替えられる」ことだけである。
pub(crate) trait ShellPrompts {
    /// New の場所。`None` は「やめた」。
    fn new_project_path(&mut self) -> Option<PathBuf>;
    /// Open するファイル。`None` は「やめた」。
    fn open_project_path(&mut self) -> Option<PathBuf>;
    /// 書き出し先。`project` は既定ファイル名の素。
    fn export_path(&mut self, project: &Path) -> Option<PathBuf>;
    /// 未保存の3択。閉じられた・知らない答えは `Cancel`(勝手に捨てない側へ倒す)。
    fn unsaved_choice(&mut self, project: &Path) -> UnsavedChoice;
}

/// 窓の訊き手。**現行の `rfd` 呼び出しをそのまま移したもの**で、文言も既定値も変えない。
pub(crate) struct NativePrompts;

impl ShellPrompts for NativePrompts {
    fn new_project_path(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("New Motolii project")
            .add_filter("Motolii project", &["json"])
            .set_file_name("untitled.json")
            .save_file()
    }

    fn open_project_path(&mut self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Open Motolii project")
            .add_filter("Motolii project", &["json"])
            .pick_file()
    }

    fn export_path(&mut self, project: &Path) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Export video")
            .add_filter("MP4 video", &["mp4"])
            .set_file_name(crate::export_seat::default_export_file_name(project))
            .save_file()
    }

    fn unsaved_choice(&mut self, project: &Path) -> UnsavedChoice {
        let name = project
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| project.display().to_string());
        let result = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title("Unsaved changes")
            .set_description(format!("{name} has unsaved changes."))
            .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
                "Save".to_owned(),
                "Discard".to_owned(),
                "Cancel".to_owned(),
            ))
            .show();
        match result {
            rfd::MessageDialogResult::Custom(label) if label == "Save" => UnsavedChoice::Save,
            rfd::MessageDialogResult::Custom(label) if label == "Discard" => UnsavedChoice::Discard,
            rfd::MessageDialogResult::Yes => UnsavedChoice::Save,
            rfd::MessageDialogResult::No => UnsavedChoice::Discard,
            _ => UnsavedChoice::Cancel,
        }
    }
}

/// 台本の訊き手。**native dialog を一切開かない。**
///
/// `Default` は全部「答えない」= dialog を閉じたのと同じ(未保存は `Cancel`)。
/// 答えを置いた口だけが動くので、テストは「触った口」だけを言えばよい。
#[derive(Debug, Clone, Default)]
pub(crate) struct ScriptedPrompts {
    pub new_project_path: Option<PathBuf>,
    pub open_project_path: Option<PathBuf>,
    pub export_path: Option<PathBuf>,
    pub unsaved_choice: Option<UnsavedChoice>,
}

impl ShellPrompts for ScriptedPrompts {
    fn new_project_path(&mut self) -> Option<PathBuf> {
        self.new_project_path.clone()
    }

    fn open_project_path(&mut self) -> Option<PathBuf> {
        self.open_project_path.clone()
    }

    fn export_path(&mut self, _project: &Path) -> Option<PathBuf> {
        self.export_path.clone()
    }

    fn unsaved_choice(&mut self, _project: &Path) -> UnsavedChoice {
        self.unsaved_choice.unwrap_or(UnsavedChoice::Cancel)
    }
}

/// headless の運転席。**テストの中だけに在る**(公開APIへ egui/kittest を漏らさない)。
#[cfg(test)]
pub(crate) use driven::DrivenShell;

#[cfg(test)]
mod driven {
    use std::path::{Path, PathBuf};

    use eframe::egui_wgpu::RenderState;
    use egui_kittest::kittest::Queryable as _;

    use super::{ScriptedPrompts, ShellPrompts, ShellTranscript};
    use crate::blitz_shell::app::BlitzShellApp;
    use crate::blitz_shell::intent::{Resume, UiIntent};

    /// 運転席の窓の大きさ(論理px)。`runner.rs` の最小寸法(980x650)に合わせる —
    /// 実行時より狭い窓でだけ通るテストにしないため。
    const SEAT_SIZE: [f32; 2] = [980.0, 650.0];

    /// `Harness` が抱える state。
    ///
    /// `BlitzShellApp` の構築には `egui::Context` が要るが、kittest は Context を
    /// 自分で作る。そこで**最初のフレームで座らせる**(kittest の Context を見てから)。
    struct Seat {
        build: Option<Box<dyn FnOnce(&egui::Context) -> BlitzShellApp>>,
        app: Option<BlitzShellApp>,
    }

    impl Seat {
        fn seat(&mut self, ctx: &egui::Context) -> &mut BlitzShellApp {
            if self.app.is_none() {
                let build = self.build.take().expect("Seat は1度だけ座る");
                self.app = Some(build(ctx));
            }
            self.app.as_mut().expect("直前に座らせた")
        }

        fn app(&self) -> &BlitzShellApp {
            self.app.as_ref().expect("最初のフレームで座っている")
        }
    }

    /// `egui_kittest` で `BlitzShellApp` を回す運転席。
    ///
    /// 窓も native dialog も開かない。見るのは AccessKit のラベルと
    /// [`ShellTranscript`] だけで、画素は見ない。
    pub(crate) struct DrivenShell {
        harness: egui_kittest::Harness<'static, Seat>,
        transcript: ShellTranscript,
    }

    impl DrivenShell {
        /// スタート画面(座席なし・fixture 展示でもない)から始める運転席。
        ///
        /// GPU が無い環境では `None` を返す — `motolii_testkit::gpu_or_skip` と
        /// 同じポリシー(理由を言って skip。`MOTOLII_REQUIRE_GPU=1` なら panic)。
        pub(crate) fn seatless(prompts: ScriptedPrompts) -> Option<Self> {
            Self::new(Box::new(prompts), None)
        }

        /// スタート画面から始めつつ、**Browser を実 media の入った folder へ座らせる**。
        ///
        /// 製品の既定(`docs/mocks`)には動画も音声も無く、カードを触る道が試せない。
        /// 変えるのは Browser が見る根だけで、他は `seatless` と1つも違わない。
        pub(crate) fn seatless_browsing(
            prompts: ScriptedPrompts,
            browser_root: &Path,
        ) -> Option<Self> {
            Self::new(Box::new(prompts), Some(browser_root.to_path_buf()))
        }

        fn new(prompts: Box<dyn ShellPrompts>, browser_root: Option<PathBuf>) -> Option<Self> {
            let (render_state, renderer) = headless_render_state()?;
            let mut prompts = Some(prompts);
            let seat = Seat {
                build: Some(Box::new(move |ctx: &egui::Context| {
                    BlitzShellApp::with_deps(
                        ctx,
                        render_state,
                        prompts.take().expect("prompts は1度だけ使う"),
                        // 運転席は常にスタート画面から始める。**利用者の
                        // 「最後に開いていた project」は読みも書きもしない**
                        // (テストが人の設定を踏まない)。
                        Resume::Nothing,
                        false,
                        browser_root,
                        None,
                    )
                })),
                app: None,
            };
            let harness = egui_kittest::Harness::builder()
                .with_size(SEAT_SIZE)
                .renderer(renderer)
                .build_ui_state(
                    |ui, seat: &mut Seat| {
                        let ctx = ui.ctx().clone();
                        seat.seat(&ctx).ui(ui);
                    },
                    seat,
                );
            let transcript = harness.state().app().transcript().clone();
            Some(Self {
                harness,
                transcript,
            })
        }

        /// フレームを進める。積んだ入力は1フレームに1件ずつ流れる(kittest の `step`)。
        pub(crate) fn run_frames(&mut self, frames: u32) {
            self.harness.run_steps(frames as usize);
        }

        /// そのラベルを含む面が出ているか。ラベルは完全一致では拾えない
        /// (スタート画面のボタンは "New Project…   Cmd+N" のように近道を連ねる)。
        pub(crate) fn label_visible(&self, label: &str) -> bool {
            self.harness
                .query_all_by_label_contains(label)
                .next()
                .is_some()
        }

        /// そのラベルを含む最初の面を押す。押した結果は次の `run_frames` で出る。
        pub(crate) fn click_label_containing(&self, label: &str) {
            let node = self
                .harness
                .query_all_by_label_contains(label)
                .next()
                .unwrap_or_else(|| {
                    panic!("ラベルに {label:?} を含む面が無い:\n{:#?}", self.harness)
                });
            node.click();
        }

        /// そのラベルを含む最初の面を**ダブルクリック**する。
        ///
        /// kittest の `click()` は1押しぶんの事象を queue へ積み、`step()` が
        /// **1事象 = 1フレーム**で流す。既定の `step_dt` は 0.25s なので、素直に
        /// 2回 `click()` すると押下の間隔が egui の double-click 判定窓
        /// (`max_double_click_delay` = 0.3s)から外れて、ただの単クリック2回になる。
        /// そこで 2 連打ぶんの押下/離しを**同じフレーム**の `RawInput` へ直接積む
        /// (同一フレーム = 時刻差 0 なので、窓の広さに依らず必ず 2 連打になる)。
        ///
        /// hover はひとつ前のフレームの席で決まるので、先に pointer を置くフレームを
        /// 1つ挟む。
        pub(crate) fn double_click_label_containing(&mut self, label: &str) {
            let center = {
                let node = self
                    .harness
                    .query_all_by_label_contains(label)
                    .next()
                    .unwrap_or_else(|| {
                        panic!("ラベルに {label:?} を含む面が無い:\n{:#?}", self.harness)
                    });
                node.rect().center()
            };
            self.harness
                .input_mut()
                .events
                .push(egui::Event::PointerMoved(center));
            self.harness.step();
            for _ in 0..2 {
                for pressed in [true, false] {
                    self.harness
                        .input_mut()
                        .events
                        .push(egui::Event::PointerButton {
                            pos: center,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        });
                }
            }
            self.harness.step();
        }

        /// 座席の Document に立っている track item の総数。
        /// **配置が起きたか**の審判(帯の一言だけを信じない)。座席が無ければ 0。
        pub(crate) fn track_item_count(&self) -> usize {
            self.harness
                .state()
                .app()
                .project()
                .map(|seat| {
                    seat.snapshot()
                        .tracks
                        .iter()
                        .map(|track| track.items.len())
                        .sum()
                })
                .unwrap_or(0)
        }

        /// 座席の writer 世代。編集が実際に通ったかの合図。座席が無ければ 0。
        pub(crate) fn revision(&self) -> u64 {
            self.harness
                .state()
                .app()
                .project()
                .map(|seat| seat.editor().revision())
                .unwrap_or(0)
        }

        /// OS ドロップの注入。native と同じ「path の入った `DroppedFile`」を積む。
        pub(crate) fn drop_file(&mut self, path: &Path) {
            self.harness
                .input_mut()
                .dropped_files
                .push(egui::DroppedFile {
                    path: Some(path.to_path_buf()),
                    ..Default::default()
                });
        }

        /// live project が座っているか。
        pub(crate) fn seated(&self) -> bool {
            self.harness.state().app().project().is_some()
        }

        /// 座っている project のパス。replay 後の座席と突き合わせる。
        pub(crate) fn project_path(&self) -> Option<PathBuf> {
            self.harness
                .state()
                .app()
                .project()
                .map(|seat| seat.path().to_path_buf())
        }

        /// 帯が映している一言(何も言われていなければ空文字)。
        pub(crate) fn latest_report(&self) -> String {
            self.transcript.latest().unwrap_or_default()
        }

        /// 言われた全文(**結果**のログ)。replay の照合に使う。
        pub(crate) fn reports(&self) -> Vec<String> {
            self.transcript
                .entries()
                .into_iter()
                .map(|event| event.text)
                .collect()
        }

        /// この駆動で記録された intent 列(**原因**のログ)。そのまま
        /// `ShellGateway::replay` へ渡せる。
        pub(crate) fn intents(&self) -> Vec<UiIntent> {
            self.harness.state().app().intent_journal().intents()
        }
    }

    /// headless の `RenderState` を手で組む。
    ///
    /// 先人と同じ形: Rerun も `egui_kittest` + 自前 `egui_wgpu::RenderState` で
    /// viewer を headless に回している。ここでは **shell が実行時に使うのと同じ
    /// device**(`GpuCtx::new_for_ui`。コンポジタの feature/limit を明示して作る)を
    /// egui へ渡す — toolkit 任せの device だと Stage/Blitz の要件が満たされない。
    ///
    /// 返すのは (app へ渡す `RenderState`, kittest の描画器)。同じ device を共有する。
    fn headless_render_state() -> Option<(RenderState, egui_kittest::wgpu::WgpuTestRenderer)> {
        let parts = match motolii_gpu::GpuCtx::new_for_ui() {
            Ok((_ctx, parts)) => parts,
            Err(error) => {
                // `gpu_or_skip` と同じ判断(ローカルは skip、CI は panic)。
                motolii_testkit::unavailable_dep("GPU adapter", &error.to_string());
                return None;
            }
        };
        let setup = eframe::egui_wgpu::WgpuSetup::Existing(eframe::egui_wgpu::WgpuSetupExisting {
            instance: parts.instance,
            adapter: parts.adapter,
            device: parts.device,
            queue: parts.queue,
        });
        let render_state = egui_kittest::wgpu::create_render_state(
            setup,
            eframe::egui_wgpu::RendererOptions::PREDICTABLE,
        );
        let renderer =
            egui_kittest::wgpu::WgpuTestRenderer::from_render_state(render_state.clone());
        Some((render_state, renderer))
    }
}
