//! Settings/Background値のdrag意味。
//!
//! CompositionやText色とは別のdraftとcommit経路を持つため、共通gesture adapter
//! から切り離してこのmoduleに閉じる。

use crate::settings_pane::BackgroundFieldDraft;
use crate::value_drag::ValueDragTarget;
use crate::{settings_pane, Shell};

impl Shell {
    pub(crate) fn settings_value_drag_start_value(&self, target: ValueDragTarget) -> Option<f64> {
        match target {
            ValueDragTarget::AutoSaveIntervalMinutes => {
                Some(self.auto_save_config.interval_secs as f64 / 60.0)
            }
            ValueDragTarget::AutoSaveGenerations => Some(self.auto_save_config.generations as f64),
            ValueDragTarget::Background(channel) => {
                let composition = self.doc.view().composition().ok().flatten()?;
                Some(f64::from(composition.background[channel.index()]) * 255.0)
            }
            _ => None,
        }
    }

    pub(crate) fn write_settings_value_drag_draft(&mut self, target: ValueDragTarget, raw: f64) {
        use settings_pane::sections::{self, AutoSaveField};

        match target {
            ValueDragTarget::AutoSaveIntervalMinutes | ValueDragTarget::AutoSaveGenerations => {
                let mut config = self.auto_save_config;
                let field = match target {
                    ValueDragTarget::AutoSaveIntervalMinutes => AutoSaveField::IntervalMinutes,
                    ValueDragTarget::AutoSaveGenerations => AutoSaveField::Generations,
                    _ => return,
                };
                match field {
                    AutoSaveField::IntervalMinutes => {
                        let clamped_minutes = raw.clamp(
                            sections::MIN_AUTO_SAVE_INTERVAL_MINUTES,
                            sections::MAX_AUTO_SAVE_INTERVAL_MINUTES,
                        );
                        config.interval_secs = (clamped_minutes * 60.0).round() as u64;
                    }
                    AutoSaveField::Generations => {
                        config.generations = raw
                            .round()
                            .clamp(1.0, sections::MAX_AUTO_SAVE_GENERATIONS as f64)
                            as usize;
                    }
                }
                let text = sections::auto_save_field_display(field, &config);
                self.auto_save_draft = Some(sections::AutoSaveFieldDraft { field, text });
            }
            ValueDragTarget::Background(channel) => {
                // Compositionが無ければ投影も無いので、draftも作らない。
                if self.doc.view().composition().ok().flatten().is_none() {
                    return;
                }
                let clamped = raw.clamp(0.0, 255.0);
                let text = (clamped.round() as u32).to_string();
                self.background_draft = Some(BackgroundFieldDraft { channel, text });
            }
            _ => {}
        }
    }

    pub(crate) fn finish_settings_value_drag(&mut self, target: ValueDragTarget) {
        match target {
            ValueDragTarget::AutoSaveIntervalMinutes | ValueDragTarget::AutoSaveGenerations => {
                let field = match target {
                    ValueDragTarget::AutoSaveIntervalMinutes => {
                        settings_pane::sections::AutoSaveField::IntervalMinutes
                    }
                    ValueDragTarget::AutoSaveGenerations => {
                        settings_pane::sections::AutoSaveField::Generations
                    }
                    _ => return,
                };
                if let Err(error) = settings_pane::sections::commit_auto_save_field(
                    &mut self.auto_save_config,
                    &mut self.auto_save_draft,
                    field,
                ) {
                    self.status = Some(error);
                }
            }
            ValueDragTarget::Background(channel) => {
                if let Err(error) = settings_pane::commit_background_channel(
                    &mut self.doc,
                    &mut self.background_draft,
                    channel,
                ) {
                    self.status = Some(error);
                }
            }
            _ => {}
        }
    }
}
