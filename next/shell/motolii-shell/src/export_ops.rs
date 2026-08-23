//! owns: Export の実行(C-3、波C「書き出し」レーン、`docs/reviews/
//! 2026-08-23-shell-split-plan.md`「export_ops.rs」節)。
//!
//! OWNS-JUSTIFICATION(C): 測定器具ではなく **shell の統合点**として自前になる。
//!       ここが持つのは「注文を組み立て、専用スレッドへ渡し、進捗と cancel を
//!       UI へ返す」という接着だけで、実処理(render/encode/mux)は
//!       `motolii-export` / `motolii-media` に在る。接着そのものは iced の
//!       `Task::run` と `iced::stream::channel`(いずれも上流の口)を借りており、
//!       並行機構を発明していない。**借り先が無いのは「Document をスレッドを
//!       跨がせずにスナップショット経由で渡す」判断の部分**で、これは
//!       `Document`/`StoreView` が借用型である(意見1・裁定2 の帰結)ことから
//!       来るため、上流の汎用 crate では代替できない。
//!
//! ## 非同期化(旧実装は `update()` を同期でブロックしていた)
//! `motolii-export` crate 冒頭 doc の「shell(UI)側の非ブロッキング化への示唆」
//! がそのまま処方箋: **別スレッドで回す**。この workspace は `iced::time::every`
//! が使えない(`next/reference/KNOWN.md`)ので、周期通知は
//! `auto_save.rs::tick_subscription`/`transport.rs::tick_subscription` と同じ
//! `iced::stream::channel` + 専用 OS スレッドの手口に倣う。**違いは1点だけ**:
//! Export は「繰り返し tick する購読」ではなく「1回きりで完了する仕事」なので、
//! `iced::Subscription::run_with`(`fn(&D) -> S` という bare fn 制約があり、
//! クロージャで注文一式を持ち運べない)ではなく `iced::Task::run`
//! (`Stream` を直接受け、クロージャのキャプチャに制約が無い — 一回性の非同期
//! 処理そのものの標準口、`file_dialogs.rs` の `Task::perform` と同格)で包む。
//! **新しい並行機構は発明していない** — 使っている部品は
//! `iced::stream::channel` + `std::thread::spawn` + `iced::Task::run` の3つで、
//! いずれも本 crate の既存箇所(`auto_save.rs`/`transport.rs`/`Task::perform`)に
//! 先例がある。
//!
//! ## `Document`/`StoreView` はスレッドを跨げない
//! `StoreView<'_>` は `Document` を借用する型なので、そのまま背景スレッドへ
//! 渡すと借用がスレッド境界を跨ぐ。旧実装が UI スレッドをブロックしていた根本
//! 理由もここ(`&mut self.engine`/`&store` を直接使っていた)。
//! **新しい橋渡し機構は作らず**、既存の `Document::save`/`Document::load`
//! 往復(`persist.rs` doc「保存と読込」、M11 で実証済み)をスナップショットの
//! 運び役に使う: export 開始時に現在の Document を一時 `.rrd` へ `save` し、
//! 背景スレッドはそのファイルを `load` して**自分専用の** `Document`/`Engine`
//! を持つ。GPU 側も同じ理由で `self.engine`(stage presenter が使う device
//! 紐付きの Engine)を跨がせず、`motolii-export` の既存試験群
//! (`motolii-export/tests/*.rs`)と同じ `Engine::new()`(headless)を背景
//! スレッド側に新規で立てる。
//!
//! ## 音声 mux(GOALS M9「音声mux込み」)
//! `motolii_media::mux_soundtrack`/`mux_mixed_pcm` は実装済みだが、呼び手が
//! 無かった(`next/reference/KNOWN.md`「音声」節)。**PCM の作り手も実は
//! 既にある** — `motolii_audio::AudioProgram::from_view` + `AudioProgram::
//! mix_audio`(`preview/export 同一の mix_audio 入口` — `program.rs` doc)は
//! 元々「正準48kHz interleaved f32」を**同期で**返す純関数で、audio
//! コールバック向けの `MixProducer`(リアルタイム専用)を経由しない。書き出し
//! 側が要る「PCM の作り手」はこれで足りていた — 無理に新設していない。
//! 流れ: `AudioProgram::from_view` で export 範囲に音声を持つ layer があるかを
//! 判定 → 無ければ映像を直接 `out_path` へ書く(mux を呼ばない、無駄な
//! ffmpeg 起動をしない)→ あれば映像を一時 path へ書き、`mix_audio` を1回
//! 呼んで PCM を作り、`write_f32le_wav_stereo_48k` で一時 WAV へ、
//! `mux_mixed_pcm` で最終 `out_path` へ。
//!
//! ## cancel で残骸なし(GOALS M9)
//! 音声が無い経路は `motolii_export::export_range_with_progress` が中断時に
//! 自前で `remove_partial`(`out_path` を消す)を呼ぶので、そのまま出ている
//! 保証を継承する。音声がある経路は**映像を最初から `out_path` ではなく
//! 一時 path へ書く**ため、中断時に消えるのは一時 path だけで、
//! `out_path` はそもそも一度も触られない(**中断してもユーザーの指定した
//! 出力先には何も現れない**、旧実装より厳しい保証)。一時 WAV・
//! スナップショット `.rrd` も成功/失敗/中断のどの分岐でも必ず消す
//! (`run_export_job` の全 return 経路)。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use iced::Task;

use motolii_export::{Cancel, ExportError, ExportJob};
use motolii_store::Document;

use crate::{export_pane, Message, Shell};

// ---------------------------------------------------------------------------
// 背景スレッドとの受け渡し
// ---------------------------------------------------------------------------

/// 背景スレッドから `Task::run` 経由で届く進捗/完了イベント。
#[derive(Debug, Clone)]
pub enum ExportEvent {
    /// フレームを1本書き終えるたび(`motolii_export::export_range_with_progress`
    /// の `on_progress` と同じ刻み)。
    Progress { frames_done: i64, frames_total: i64 },
    /// 実行終了(成功・失敗・中断のいずれも1本の腕へ畳む — 呼び手側の分岐を
    /// 1箇所にするため)。
    Finished(Result<ExportOutcome, String>),
}

/// 書き出し成功時の報告。「報告 = 現物」(GOALS M9)を保つため、実際に
/// encoder へ渡し終えたフレーム数(`ExportReport::frames_written`)をそのまま運ぶ。
#[derive(Debug, Clone)]
pub struct ExportOutcome {
    pub out_path: PathBuf,
    pub frames_written: i64,
    /// 音声を mux したか(音声を持つ layer が1本も無ければ `false` —
    /// mux 自体を呼ばない)。
    pub audio_muxed: bool,
}

/// 背景スレッドへ渡す注文一式。**`Document`/`Engine` を直接持たせない** —
/// module doc「`Document`/`StoreView` はスレッドを跨げない」参照。
struct ExportRunSpec {
    /// 現在の Document のスナップショット(`Document::save` 済み)。
    snapshot_path: PathBuf,
    out_path: PathBuf,
    qp0: bool,
    /// 半開 `[start_frame, end_frame)`(comp フレーム単位、
    /// `export_pane::effective_range` の出力そのまま)。
    start_frame: i64,
    end_frame: i64,
}

static EXPORT_SNAPSHOT_SEQ: AtomicU64 = AtomicU64::new(0);

/// export 開始のたびに一意な一時スナップショット path を作る(同時に複数の
/// export は UI 上できない想定だが、直前の export の残骸と衝突しないよう
/// 呼び出しごとに別名にする)。
fn next_snapshot_path() -> PathBuf {
    let seq = EXPORT_SNAPSHOT_SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "motolii-export-snapshot-{}-{}.rrd",
        std::process::id(),
        seq
    ))
}

impl Shell {
    // ---- Export 窓(B09、第6波、`export_pane` crate doc「shell 結線」節) ----

    /// Export 窓の open/close(`toggle_settings_window` と同じ型)。
    pub(crate) fn toggle_export_window(&mut self) -> Task<Message> {
        match self.export_window.take() {
            Some(id) => iced::window::close(id),
            None => {
                let (id, open) = iced::window::open(iced::window::Settings {
                    size: iced::Size::new(420.0, 360.0),
                    resizable: true,
                    ..iced::window::Settings::default()
                });
                self.export_window = Some(id);
                open.map(Message::ExportWindowOpened)
            }
        }
    }

    /// `Message::Export` の畳み(crate doc「shell 結線」手順3)。
    pub(crate) fn update_export(&mut self, message: export_pane::Message) -> Task<Message> {
        match message {
            export_pane::Message::ToggleExportDialog => return self.toggle_export_window(),
            export_pane::Message::QualitySelect(quality) => self.export_quality = quality,
            export_pane::Message::RangeSelect(range) => self.export_range = range,
            // 2026-08-22 第2波(File 束の rfd 非同期化と同時発注): 書き出し先の
            // 選択。`file_dialogs.rs::FileDialogs::pick_export_path` を非同期に
            // 呼び、結果を `OutputPathChosen` で畳んで戻す。
            export_pane::Message::PickOutputPath => {
                let default_name = self.export_default_file_name();
                return Task::perform(self.dialogs.pick_export_path(default_name), |path| {
                    Message::Export(export_pane::Message::OutputPathChosen(path))
                });
            }
            export_pane::Message::OutputPathChosen(Some(path)) => self.export_out_path = Some(path),
            export_pane::Message::OutputPathChosen(None) => {}
            export_pane::Message::Export => return self.start_export(),
            export_pane::Message::CancelExport => {
                // 中断チェックはフレーム境界(`export_range_with_progress` の
                // ループ)で見ている — ここは `AtomicBool` を立てるだけ。
                if let Some(cancel) = &self.export_cancel {
                    cancel.cancel();
                }
            }
        }
        Task::none()
    }

    /// Export 書き出し先 dialog の初期ファイル名。`current_path`(開いている
    /// project)があればその stem を使う(`<name>.mp4`)、無ければ既定
    /// `"untitled.mp4"`(`file_dialogs.rs::RfdDialogs::pick_save_path` の
    /// 既定 file name と同じ考え方)。
    pub(crate) fn export_default_file_name(&self) -> String {
        let stem = self
            .current_path
            .as_deref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .unwrap_or("untitled");
        format!("{stem}.mp4")
    }

    /// Export 実行(crate doc「shell 結線」手順3「`Export` = `ExportJob` を
    /// 組んで export 実行」)。**非ブロッキング**: 実際の render/encode/mux は
    /// 専用 OS スレッドへ逃がし、`Task::run` が背景スレッドからの進捗/完了を
    /// `Message::ExportProgressed` へ翻訳して戻す(module doc 参照)。
    /// `Shell::update` はこの `Task` を返した時点で即座に戻るので、export 中も
    /// UI スレッドは他の `Message` を受け続ける(cancel ボタンが届く)。
    pub(crate) fn start_export(&mut self) -> Task<Message> {
        let Some(out_path) = self.export_out_path.clone() else {
            self.status = Some("書き出し先が未設定".to_owned());
            return Task::none();
        };
        let Some(composition) = self.composition() else {
            self.status = Some("comp が無いので書き出せない".to_owned());
            return Task::none();
        };
        let duration = composition.duration_frames;
        let range = export_pane::effective_range(
            self.export_range,
            self.timeline_work_area().map(|area| export_pane::WorkAreaFrames {
                start: area.start,
                end: area.end,
            }),
            duration,
        );
        let total = range.frame_count();

        // `Document`/`StoreView` はスレッドを跨げない(module doc)ので、
        // 既存の save/load 往復でスナップショットを渡す。
        let snapshot_path = next_snapshot_path();
        if let Err(error) = self.doc.save(&snapshot_path) {
            self.status = Some(format!("書き出し用の下書きを作れない: {error}"));
            return Task::none();
        }

        let cancel = Cancel::new();
        self.export_cancel = Some(cancel.clone());
        self.export_progress = Some(export_pane::ExportProgress {
            frames_done: 0,
            frames_total: total,
        });

        let spec = ExportRunSpec {
            snapshot_path,
            out_path,
            qp0: self.export_quality.qp0(),
            start_frame: range.start,
            end_frame: range.end,
        };

        Task::run(export_stream(spec, cancel), Message::ExportProgressed)
    }

    /// `Message::ExportProgressed` の受け口(`lib.rs::update` の腕、RETURN 参照)。
    pub(crate) fn update_export_progressed(&mut self, event: ExportEvent) {
        match event {
            ExportEvent::Progress {
                frames_done,
                frames_total,
            } => {
                self.export_progress = Some(export_pane::ExportProgress {
                    frames_done,
                    frames_total,
                });
            }
            ExportEvent::Finished(Ok(outcome)) => {
                self.export_cancel = None;
                self.export_progress = Some(export_pane::ExportProgress {
                    frames_done: outcome.frames_written,
                    frames_total: outcome.frames_written,
                });
                let audio_note = if outcome.audio_muxed {
                    "・音声あり"
                } else {
                    ""
                };
                self.status = Some(format!(
                    "書き出し完了: {} ({} frames{audio_note})",
                    outcome.out_path.display(),
                    outcome.frames_written
                ));
            }
            ExportEvent::Finished(Err(error)) => {
                self.export_cancel = None;
                self.export_progress = None;
                self.status = Some(format!("書き出しできない: {error}"));
            }
        }
    }

    /// Export の実行結果/実行中状態の読み口(B09、第6波)。`None` = 実行
    /// していない。運転席が「Export → 完了して進捗が total/total になる」を
    /// 確かめる口。
    pub fn export_progress(&self) -> Option<export_pane::ExportProgress> {
        self.export_progress
    }

    /// Export の品質選択の読み口(B09、第6波)。運転席が
    /// `Message::Export(QualitySelect(_))` の反映を確かめる口。
    pub fn export_quality(&self) -> export_pane::ExportQuality {
        self.export_quality
    }

    /// Export の範囲選択の読み口(B09、第6波)。同上。
    pub fn export_range(&self) -> export_pane::ExportRange {
        self.export_range
    }

    /// 書き出し先(`export_pane::Message::PickOutputPath` が
    /// `FileDialogs::pick_export_path` の結果で埋める、2026-08-22 第2波)。
    /// **運転席が見るための口**(`current_path`/`export_quality` と同じ形)。
    pub fn export_out_path(&self) -> Option<&std::path::Path> {
        self.export_out_path.as_deref()
    }
}

// ---------------------------------------------------------------------------
// 背景スレッド(`iced::stream::channel` + `std::thread::spawn` — module doc)
// ---------------------------------------------------------------------------

/// `auto_save::tick_stream`/`transport::tick_subscription` と同じ手口。
/// **違い**: 繰り返し tick するのではなく、専用スレッドが1回の仕事
/// (`run_export_job`)を最後まで終えたら自然に終了する(`output` の全 clone が
/// drop されるとチャンネルが閉じ、`Task::run` の stream も終わる — `pending()`
/// で生かし続ける必要が無い、一回性の仕事だから)。
fn export_stream(
    spec: ExportRunSpec,
    cancel: Cancel,
) -> impl iced::futures::Stream<Item = ExportEvent> {
    iced::stream::channel(
        32,
        move |output: iced::futures::channel::mpsc::Sender<ExportEvent>| async move {
            // sleep 相当(実際は render/encode/ffmpeg 呼び出し)はブロッキング
            // なので、async executor 上ではなく専用 OS スレッドで行う
            // (`auto_save.rs` doc と同じ理由)。
            std::thread::spawn(move || {
                let mut output = output;
                let result = run_export_job(&spec, &cancel, |progress| {
                    let _ = output.try_send(ExportEvent::Progress {
                        frames_done: progress.frames_done,
                        frames_total: progress.frames_total,
                    });
                });
                let _ = output.try_send(ExportEvent::Finished(result));
                // `output` はここでスコープを抜けて drop される → 受け手が
                // 無くなれば channel が閉じ、stream が終わる。
            });
        },
    )
}

/// **実際の書き出し本体。iced/Task を一切知らない同期関数**(テストが
/// `Task`/実行時を経由せずここへ直接入れる — `export_ops.rs` の RETURN 参照)。
///
/// 手順:
/// 1. スナップショットから独立した `Document`/`Engine` を組む
/// 2. `AudioProgram::from_view` で音声を持つ layer の有無を見る
/// 3. 無ければ映像を `out_path` へ直接書く(mux を呼ばない)
/// 4. あれば映像を一時 path へ書き、`mix_audio` → WAV → `mux_mixed_pcm` で
///    `out_path` へ
///
/// 全 return 経路でスナップショット `.rrd`(と、作っていれば一時動画/WAV)を
/// 必ず消す — 「cancel で残骸なし」(GOALS M9)をスナップショット自身にも適用する。
fn run_export_job(
    spec: &ExportRunSpec,
    cancel: &Cancel,
    mut on_progress: impl FnMut(motolii_export::ExportProgress),
) -> Result<ExportOutcome, String> {
    let cleanup_snapshot = || {
        let _ = std::fs::remove_file(&spec.snapshot_path);
    };

    let doc = match Document::load(&spec.snapshot_path) {
        Ok(doc) => doc,
        Err(error) => {
            cleanup_snapshot();
            return Err(format!("書き出し用の下書きを読めない: {error}"));
        }
    };
    let view = doc.view();

    let composition = match view.composition() {
        Ok(Some(composition)) => composition,
        Ok(None) => {
            cleanup_snapshot();
            return Err("comp が無いので書き出せない".to_owned());
        }
        Err(error) => {
            cleanup_snapshot();
            return Err(format!("{error}"));
        }
    };
    let fps = composition.fps;

    // 音声(あれば)。`AudioProgram::from_view`/`mix_audio` は preview の
    // 音声経路(`transport.rs::open_real_playback`)と同じ純関数
    // (`program.rs` doc「preview/export同一のmix_audio入口」) — 書き出し
    // 専用の第二経路を作らない。
    let mut caches = HashMap::new();
    let audio_program = match motolii_audio::AudioProgram::from_view(&view, &mut caches) {
        Ok(program) => Some(program),
        Err(error) => {
            // 音声側の失敗で映像書き出しごと失わない(D6注記なし・意図的な
            // 判断、RETURN 参照): 無音声の mp4 を出しつつ理由を状態帯へ残す。
            // `has_audio = false` として続行する。
            drop(error);
            None
        }
    };
    let has_audio = audio_program
        .as_ref()
        .is_some_and(|program| !program.sources().is_empty());

    let mut engine = match motolii_engine::Engine::new() {
        Ok(engine) => engine,
        Err(error) => {
            cleanup_snapshot();
            return Err(format!("engine を作れない: {error}"));
        }
    };

    let parent = spec
        .out_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = spec
        .out_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");

    // 音声が無ければ映像を最終 `out_path` へ直接書く。あれば**一時 path**へ
    // 書く — cancel/mux失敗で `out_path` が一度も触られない保証はここで作る
    // (module doc「cancel で残骸なし」)。
    let video_target = if has_audio {
        parent.join(format!(".{stem}.motolii-export-video.mp4"))
    } else {
        spec.out_path.clone()
    };

    let job = ExportJob {
        out_path: video_target.clone(),
        qp0: spec.qp0,
    };
    let range = spec.start_frame..spec.end_frame;

    let report = motolii_export::export_range_with_progress(
        &mut engine,
        &view,
        &job,
        range,
        cancel,
        |progress| on_progress(progress),
    );

    let report = match report {
        Ok(report) => report,
        Err(ExportError::Cancelled) => {
            // `export_range_with_progress` が `video_target` を既に消している
            // (`remove_partial`)。`out_path` はまだ書いていないので無傷。
            cleanup_snapshot();
            return Err("中断された(残骸は消してある)".to_owned());
        }
        Err(error) => {
            cleanup_snapshot();
            if has_audio {
                let _ = std::fs::remove_file(&video_target);
            }
            return Err(format!("{error}"));
        }
    };

    if !has_audio {
        cleanup_snapshot();
        return Ok(ExportOutcome {
            out_path: spec.out_path.clone(),
            frames_written: report.frames_written,
            audio_muxed: false,
        });
    }

    let program = audio_program.expect("has_audio は audio_program.is_some() を前提にした判定");
    let outcome = mux_audio_into(&program, fps, spec, &video_target, report.frames_written);
    let _ = std::fs::remove_file(&video_target);
    cleanup_snapshot();
    outcome
}

/// 音声(あれば)を映像へ mux して最終 `out_path` へ書く。失敗時は
/// `out_path` を消してから返す(半端な file を残さない)。
fn mux_audio_into(
    program: &motolii_audio::AudioProgram,
    fps: motolii_core::Fps,
    spec: &ExportRunSpec,
    video_path: &Path,
    frames_written: i64,
) -> Result<ExportOutcome, String> {
    let start_t = motolii_core::RationalTime::try_from_frame(spec.start_frame, fps)
        .map_err(|error| format!("開始時刻を写せない: {error}"))?;
    let start_sample = motolii_audio::time_to_canonical_frames(start_t);

    let video_duration = motolii_core::RationalTime::try_from_frame(frames_written, fps)
        .map_err(|error| format!("映像尺を写せない: {error}"))?;
    let sample_count = motolii_audio::time_to_canonical_frames(video_duration) as usize;

    let (pcm, _mix_report) = program
        .mix_audio(start_sample, sample_count, None)
        .map_err(|error| format!("音声を混ぜられない: {error}"))?;

    let parent = spec
        .out_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = spec
        .out_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("export");
    let wav_path = parent.join(format!(".{stem}.motolii-export-audio.wav"));

    if let Err(error) = motolii_media::write_f32le_wav_stereo_48k(&wav_path, &pcm) {
        return Err(format!("音声下書きを書けない: {error}"));
    }

    let mux_result = motolii_media::mux_mixed_pcm(&motolii_media::MixedPcmMuxRequest {
        video_path,
        pcm_wav_path: &wav_path,
        output_path: &spec.out_path,
        video_duration: Some(video_duration),
    });
    let _ = std::fs::remove_file(&wav_path);

    match mux_result {
        Ok(_report) => Ok(ExportOutcome {
            out_path: spec.out_path.clone(),
            frames_written,
            audio_muxed: true,
        }),
        Err(error) => {
            let _ = std::fs::remove_file(&spec.out_path);
            Err(format!("音声muxに失敗: {error}"))
        }
    }
}

// ---------------------------------------------------------------------------
// テスト向けの薄い口(iced/Task を経由せず `run_export_job` を直接叩く)。
// ---------------------------------------------------------------------------

/// テスト専用: `Document` をその場で snapshot して `run_export_job` を同期に
/// 1回走らせる。`Task`/`iced::stream::channel` の非同期機構を経由しない
/// (export_drive.rs の音声 mux 試験がこれを使う — 意図は「フレームを
/// `Task` から汲み取れるか」ではなく「実際に出た mp4 に音声トラックが
/// 在るか」なので、production の非同期経路と分けて確かめる)。
#[doc(hidden)]
pub fn run_export_job_for_test(
    doc: &Document,
    out_path: &Path,
    qp0: bool,
    start_frame: i64,
    end_frame: i64,
) -> Result<ExportOutcome, String> {
    let snapshot_path = next_snapshot_path();
    doc.save(&snapshot_path)
        .map_err(|error| format!("snapshot 失敗: {error}"))?;
    let spec = ExportRunSpec {
        snapshot_path,
        out_path: out_path.to_path_buf(),
        qp0,
        start_frame,
        end_frame,
    };
    let cancel = Cancel::new();
    run_export_job(&spec, &cancel, |_| {})
}
