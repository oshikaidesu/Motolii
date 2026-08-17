//! 書き出し面の座席: Export ボタンの**後ろ**にある開始列・進捗・キャンセル。
//!
//! 経路は既存の書き出しそのもの(第二経路を作らない):
//! `motolii_export::export_document_video`(cancel 口付きの
//! `export_document_video_cancellable`)を、CLI `document_export.rs` と同じ
//! `ExportJob` の組み方で呼ぶ。違いは入力だけ — file を開き直さず、
//! **現 writer snapshot**(`ProjectSeat::snapshot`)から出す。未保存編集も
//! 書き出しに含まれる(編集ソフトの普通)。
//!
//! thread の形は「1 書き出し = 1 thread」: UI thread は `ExportRun::start` で
//! 起こして `try_finish` を poll するだけで、書き出し中も固まらない。GPU は
//! thread の中で `GpuCtx::new_headless()` を別に立てる(CLI と同じ形。shell の
//! render_state と device を共有しない — 二重 device が既存の形)。
//!
//! 進捗: export 側に途中経過の callback 口は無い(`ExportReport` は完了後の
//! 要約のみ。2026-08-18 実測)。v0 は indeterminate(スピナー+経過秒)で、
//! frame 数ベースの進捗は residual。
//!
//! dialog(`rfd`)はここに無い。`blitz_shell::app` が `prompt_export_path` で
//! path を決めてから渡す(`blitz_shell_export.rs` のテストと同じ関数境界)。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Instant;

use motolii_doc::Document;
use motolii_eval::DataTracks;
use motolii_export::{export_document_video_cancellable, ExportError, ExportJob, ExportReport};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;

/// Export ボタンの enabled と二重起動防止の判断。**同じ関数を両方が見る**:
/// project が座っていて(`seated`)、実行中の書き出しが無い(`!running`)時だけ
/// 開始できる。
pub fn can_start_export(seated: bool, running: bool) -> bool {
    seated && !running
}

/// save dialog の既定 file 名 `{project名}.mp4`。名前が取れない path でも
/// 空の既定を dialog に出さない。
pub fn default_export_file_name(project_path: &Path) -> String {
    match project_path.file_stem().map(|stem| stem.to_string_lossy()) {
        Some(stem) if !stem.is_empty() => format!("{stem}.mp4"),
        _ => "export.mp4".to_owned(),
    }
}

/// 1回の書き出しの入力。`ExportJob` の GUI 版(lifetime 無し・snapshot 起点)。
/// 組み方の先例は CLI `document_export.rs`。
pub struct ExportRequest {
    /// 現 writer snapshot(`ProjectSeat::snapshot`)。file を開き直さない。
    pub document: Arc<Document>,
    /// asset path 解決の根。CLI と同じ規約で project file の親。
    pub project_root: Option<PathBuf>,
    /// dialog(または テスト)が決めた保存先。
    pub output_path: PathBuf,
    /// `None` = composition 全長(製品 v0 の既定)。テストの審判が短く切るのに使う。
    pub frame_count: Option<usize>,
    /// 検証用のほぼロスレス。製品 v0 は `false`。
    pub qp0: bool,
}

/// 書き出しの終わり方。UI の status はこれをそのまま言葉にする。
#[derive(Debug)]
pub enum ExportFinish {
    /// 完了。`frames_written` 等は report が持つ。
    Done(ExportReport),
    /// キャンセルで早期終了。出力 file は残っていない。
    Cancelled,
    /// 失敗(理由付き)。
    Failed(String),
}

/// thread の中身。**テストはこれを同期で駆動する**(thread と channel の殻を
/// 挟まず同じ列を審判できる)。cancel が既に立っていれば何も作らない。
pub fn run_export_worker(request: &ExportRequest, cancel: &AtomicBool) -> ExportFinish {
    if cancel.load(Ordering::Relaxed) {
        return ExportFinish::Cancelled;
    }
    let runtime = match first_party_runtime() {
        Ok(runtime) => runtime,
        Err(error) => return ExportFinish::Failed(error.to_string()),
    };
    // export は headless GpuCtx を別に立てるのが既存 CLI の形(render_state 共有なし)。
    let gpu = match GpuCtx::new_headless() {
        Ok(gpu) => gpu,
        Err(error) => return ExportFinish::Failed(error.to_string()),
    };
    let result = export_document_video_cancellable(
        &gpu,
        &ExportJob {
            doc: &request.document,
            runtime: &runtime,
            output_path: &request.output_path,
            project_root: request.project_root.as_deref(),
            frame_count: request.frame_count,
            qp0: request.qp0,
            data_tracks: DataTracks::new(),
        },
        cancel,
    );
    match result {
        Ok(report) => ExportFinish::Done(report),
        // cancel の掃除(部分出力の削除)は export 側が済ませている。
        Err(ExportError::Cancelled) => ExportFinish::Cancelled,
        Err(error) => ExportFinish::Failed(error.to_string()),
    }
}

/// 実行中の書き出し1件。UI(status 帯)が持ち、毎フレーム `try_finish` を poll する。
pub struct ExportRun {
    output_path: PathBuf,
    cancel: Arc<AtomicBool>,
    started: Instant,
    /// worker からの唯一の返事(1件で閉じる)。
    receiver: Receiver<ExportFinish>,
    /// 返事を受けたら join して thread を回収する。
    handle: Option<std::thread::JoinHandle<()>>,
    /// 1回答えたら以後は `None`(UI が二重に status を書かない)。
    answered: bool,
}

impl ExportRun {
    /// 書き出し thread を起こす。呼んだ時点から `try_finish` が答えるまで
    /// UI は自由に動いてよい(書き出し中も固まらない)。
    pub fn start(request: ExportRequest) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = sync_channel(1);
        let worker_cancel = Arc::clone(&cancel);
        let output_path = request.output_path.clone();
        let handle = std::thread::Builder::new()
            .name("motolii-export".to_owned())
            .spawn(move || {
                let finish = run_export_worker(&request, &worker_cancel);
                // UI 側が run を捨てていたら受け手が居ないだけ — worker は静かに終わる。
                let _ = sender.send(finish);
            })
            .expect("spawn export thread");
        Self {
            output_path,
            cancel,
            started: Instant::now(),
            receiver,
            handle: Some(handle),
            answered: false,
        }
    }

    /// 書き出し先(status の言葉に使う)。
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    /// 経過秒。進捗口が無い v0 の「まだ生きている」表示(indeterminate+経過秒)。
    pub fn elapsed_seconds(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    /// キャンセルを頼む。frame loop が次の周で見て早期終了する
    /// (即死ではない — 掃除して `Cancelled` が返る)。
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// 既にキャンセルを頼んだか(Cancel ボタンの二度押し disabled)。
    pub fn cancel_requested(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// 終わっていれば**1回だけ**答える。UI は `Some` を受けたら run を捨てる。
    pub fn try_finish(&mut self) -> Option<ExportFinish> {
        if self.answered {
            return None;
        }
        let finish = match self.receiver.try_recv() {
            Ok(finish) => finish,
            Err(TryRecvError::Empty) => return None,
            // worker が返事より先に死ぬのは panic だけ。黙って走行中に見せない。
            Err(TryRecvError::Disconnected) => {
                ExportFinish::Failed("export thread died".to_owned())
            }
        };
        self.answered = true;
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        Some(finish)
    }
}
