//! Settingsの意味更新。窓の結線はdispatcherから委譲されるが、
//! 下書き・Document更新・拒否理由の責任はこのmoduleに閉じる。

use iced::Task;

use crate::settings_pane::BackgroundFieldDraft;
use crate::{settings_pane, value_drag::ValueDragTarget, Message, Shell};

impl Shell {
    /// pane ローカル `Message`(SET+ の [`settings_pane::sections::Message`])を
    /// 畳んで書き口へ渡す glue。sections.rs 冒頭 doc「結線互換の縫い目」の手順
    /// 2そのもの: 新項目2腕(`CompFieldInput`/`CompFieldSubmit` —
    /// `commit_comp_field` が read-modify-write の `Intent::SetComposition` を
    /// 1回出す)+ 旧腕は [`Self::update_settings_legacy`] へ丸ごと委譲。
    pub(crate) fn update_settings(
        &mut self,
        message: settings_pane::sections::Message,
    ) -> Task<Message> {
        use sections::{AutoSaveField, CompField};
        use settings_pane::sections;
        match message {
            sections::Message::Legacy(legacy) => return self.update_settings_legacy(legacy),
            sections::Message::CompFieldInput(field, text) => {
                self.comp_draft = Some(sections::CompFieldDraft { field, text });
            }
            sections::Message::CompFieldSubmit(field) => {
                if let Err(error) =
                    sections::commit_comp_field(&mut self.doc, &mut self.comp_draft, field)
                {
                    self.status = Some(error);
                }
            }
            sections::Message::AutoSaveToggle(enabled) => {
                self.auto_save_enabled = enabled;
            }
            sections::Message::AutoSaveFieldInput(field, text) => {
                self.auto_save_draft = Some(sections::AutoSaveFieldDraft { field, text });
            }
            sections::Message::AutoSaveFieldSubmit(field) => {
                if let Err(error) = sections::commit_auto_save_field(
                    &mut self.auto_save_config,
                    &mut self.auto_save_draft,
                    field,
                ) {
                    self.status = Some(error);
                }
            }
            // 裁定217 連続量 drag 化(E-5)。`start_value_drag` と同じ
            // 「press だけ own する」形 — move/release は window 全体購読
            // (`inspector_pointer_event`)を Inspector と共有する。
            sections::Message::CompFieldDragPressed(field) => {
                self.start_value_drag(match field {
                    CompField::Width => ValueDragTarget::CompWidth,
                    CompField::Height => ValueDragTarget::CompHeight,
                    CompField::Fps => ValueDragTarget::CompFps,
                    CompField::DurationFrames => ValueDragTarget::CompDuration,
                });
            }
            sections::Message::AutoSaveFieldDragPressed(field) => {
                self.start_value_drag(match field {
                    AutoSaveField::IntervalMinutes => ValueDragTarget::AutoSaveIntervalMinutes,
                    AutoSaveField::Generations => ValueDragTarget::AutoSaveGenerations,
                });
            }
        }
        Task::none()
    }

    /// 旧 `settings_pane::Message` の腕(SET+ 以前の全項目)。write ロジックの
    /// 実体は `motolii_settings_pane::{apply_background_preset,
    /// commit_background_channel, commit_ui_scale}`(自由関数、`&mut Document`/
    /// `&mut Tokens`/下書きを明示引数で受け取る形 — pane crate は `&mut self` を
    /// 持てないため)。ここでは `self.doc`/`self.tokens`/下書きフィールドを
    /// そのまま貸すだけで、拒否理由(`Result::Err`)を `self.status` へ write
    /// する以外の判断は持たない。
    fn update_settings_legacy(&mut self, message: settings_pane::Message) -> Task<Message> {
        match message {
            settings_pane::Message::ToggleSettingsPanel => {
                // S2(裁定182/188): 意味が「レイアウト分岐」→「窓 open/close」
                // へ変わった(probe §Q3)。トグル以外の腕は従来どおり
                // Task を返さない。
                return self.toggle_settings_window();
            }
            settings_pane::Message::BackgroundPreset(preset) => {
                if let Err(error) = settings_pane::apply_background_preset(&mut self.doc, preset) {
                    self.status = Some(error);
                }
            }
            settings_pane::Message::BackgroundChannelInput(channel, text) => {
                self.background_draft = Some(BackgroundFieldDraft { channel, text });
            }
            settings_pane::Message::BackgroundChannelSubmit(channel) => {
                if let Err(error) = settings_pane::commit_background_channel(
                    &mut self.doc,
                    &mut self.background_draft,
                    channel,
                ) {
                    self.status = Some(error);
                }
            }
            settings_pane::Message::UiScaleInput(text) => self.ui_scale_draft = Some(text),
            settings_pane::Message::UiScaleSubmit => {
                if let Err(error) =
                    settings_pane::commit_ui_scale(&mut self.tokens, &mut self.ui_scale_draft)
                {
                    self.status = Some(error);
                }
            }
            // 裁定217 連続量 drag 化(E-5)。`sections::Message::CompFieldDragPressed`
            // と同じ形。
            settings_pane::Message::BackgroundChannelDragPressed(channel) => {
                self.start_value_drag(ValueDragTarget::Background(channel));
            }
        }
        Task::none()
    }

    /// S2(裁定182/188): Settings の入口 — header の歯車が出す
    /// `ToggleSettingsPanel` を OS 窓の open/close へ配線する(浮かし第1号、
    /// 裁定188「Settings はだいたいポップアップだから」)。
    ///
    /// 台帳(`settings_window`)は**同期で先行記帳/先行抹消**する —
    /// `window::open` は Id を同期で採番し(fork `runtime/src/window.rs:260`)、
    /// close も「閉じるつもり」の時点で台帳から下ろす。runtime 無しの headless
    /// 試験(Task は走らない)でも open/close/再open の状態遷移が読めるのは
    /// この設計のため(`tests/suite/window_drive.rs` の oracle)。OS の閉じる
    /// ボタン経由は `Message::WindowClosed`(`close_events` 購読)が同じ抹消を
    /// 行う。
    fn toggle_settings_window(&mut self) -> Task<Message> {
        match self.settings_window.take() {
            Some(id) => iced::window::close(id),
            None => {
                let (id, open) = iced::window::open(iced::window::Settings {
                    // 小さめ・リサイズ可(発注どおり、probe 実証の形)。raw 値は
                    // pane の意匠値ではなく**窓の初期ジオメトリ**(トンマナ柵
                    // (裁定142)の対象マーカー外 — `Size::new` は widget 構築
                    // 呼び出しではない): 幅はプリセット4ボタン+数値欄が
                    // 折り返さない程度、高さは4行+見出し(probe の 420×320 と
                    // 同桁)。リサイズ可なので初期値以上の拘束は持たない。
                    size: iced::Size::new(480.0, 400.0),
                    resizable: true,
                    ..iced::window::Settings::default()
                });
                self.settings_window = Some(id);
                open.map(Message::SettingsWindowOpened)
            }
        }
    }
}
