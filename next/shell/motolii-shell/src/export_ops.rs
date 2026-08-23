
use iced::Task;


use crate::{
    export_pane, Message, Shell,
};

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
            export_pane::Message::Export => self.start_export(),
            export_pane::Message::CancelExport => {
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
    /// 組んで export 実行」)。**逸脱**: `motolii_export::export_with_cancel`
    /// はフレーム単位の進捗コールバックを持たない同期の1回きりのバッチ呼び
    /// 出し(engine crate 冒頭 doc)なので、`export_progress` は開始時
    /// (0/total)と完了時(total/total)の2点だけ更新する「進捗 subscription」
    /// の型だけの実装になっている(RETURN 参照 — 真の非同期進捗ストリームは
    /// `motolii-export` 側にコールバック/チャンク実行を足す変更が要り、
    /// EXACT TARGET(shell のみ)の外)。UI スレッドは export 完了まで
    /// ブロックする(呼び出しは同期のまま — RETURN 逸脱)。
    pub(crate) fn start_export(&mut self) {
        let Some(out_path) = self.export_out_path.clone() else {
            self.status = Some("書き出し先が未設定".to_owned());
            return;
        };
        let Some(composition) = self.composition() else {
            self.status = Some("comp が無いので書き出せない".to_owned());
            return;
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
        let cancel = motolii_export::Cancel::new();
        self.export_cancel = Some(cancel.clone());
        self.export_progress = Some(export_pane::ExportProgress {
            frames_done: 0,
            frames_total: total,
        });
        let job = motolii_export::ExportJob {
            out_path,
            qp0: self.export_quality.qp0(),
        };
        let store = self.doc.view();
        let result = motolii_export::export_with_cancel(&mut self.engine, &store, &job, &cancel);
        self.export_cancel = None;
        match result {
            Ok(report) => {
                self.export_progress = Some(export_pane::ExportProgress {
                    frames_done: report.frames_written,
                    frames_total: total,
                });
                self.status = Some(format!(
                    "書き出し完了: {} ({} frames)",
                    report.out_path.display(),
                    report.frames_written
                ));
            }
            Err(error) => {
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

